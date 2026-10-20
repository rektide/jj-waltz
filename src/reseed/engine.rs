use super::selection::{self, Group, Registrations};
use super::{ReseedOutcome, ReseedReport, ReseedRequest, ReseedStatus};
use crate::cow::ignored::{Origin, Patterns, Policy};
#[cfg(test)]
use crate::cow::payload::checked_git;
use crate::cow::payload::{Entry, FrozenRules, IgnoreRules, confined_parents, directory_identity, entries, relative};
use crate::cow::{topology, transfer};
use crate::freshen::source::SourceSelection;
use crate::{cow, jj::JjClient, links, metadata::WorkspaceMetadataStore};
use anyhow::{Context, Result, bail};
use diff_dirs::Check;
use fs2::FileExt;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::ErrorKind;
#[cfg(test)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::rc::Rc;

#[path = "bookmarks.rs"]
mod bookmarks;
mod payload;
#[path = "prepare.rs"]
mod prepare;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalConfig {
    source: Option<PathBuf>,
    source_at: Option<String>,
    copy_ignored: Option<Vec<String>>,
    #[serde(default)]
    groups: BTreeMap<String, Group>,
    freshen: Option<prepare::Settings>,
    /// Deprecated local alias for freshen settings.
    prepare: Option<prepare::Settings>,
    #[serde(skip)]
    legacy_prepare: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Identity {
    device: u64,
    inode: u64,
}
impl Identity {
    fn read(path: &Path) -> Result<Self> {
        let (device, inode) = directory_identity(path)?;
        Ok(Self { device, inode })
    }
    fn verify(&self, path: &Path) -> Result<()> {
        if &Self::read(path)? != self {
            bail!("directory identity changed: {}", path.display());
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Plan {
    name: String,
    root: PathBuf,
    root_identity: Identity,
    jj_identity: Identity,
    repository: PathBuf,
    source: PathBuf,
    source_identity: Identity,
    source_jj_identity: Identity,
    source_operation: String,
    source_commit: String,
    source_wc_commit: String,
    target_operation: String,
    old_commit: String,
    desired_commit: String,
    base_commit: String,
    clones: Vec<PathBuf>,
    entries: Vec<Entry>,
    old_paths: Vec<PathBuf>,
    ignored: Vec<PathBuf>,
    source_rules: Rc<IgnoreRules>,
    source_entries: Rc<Vec<Entry>>,
    target_rules: Rc<IgnoreRules>,
    skipped: Vec<String>,
    replace_content: bool,
    config_root: PathBuf,
    link_config: Vec<(String, Option<Vec<u8>>)>,
    bookmark_guard: bookmarks::Guard,
    execution_diagnostics: Vec<String>,
}

struct PreparationLock {
    _lock: File,
}
impl PreparationLock {
    fn lock(client: &JjClient) -> Result<Self> {
        let config = client.repo_config_path()?;
        WorkspaceMetadataStore::from_repo_config_path(&config)?.ensure_initialized()?;
        let parent = config.parent().context("repository config has no parent")?.canonicalize()?;
        let root = parent.join("jj-waltz/reseed");
        fs::create_dir_all(&root)?;
        if root.canonicalize()? != root {
            bail!("reseed storage contains a symlink");
        }
        let lock_path = root.join("operation.lock");
        if fs::symlink_metadata(&lock_path).is_ok_and(|m| !m.is_file()) {
            bail!("invalid reseed lock");
        }
        let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(lock_path)?;
        lock.try_lock_exclusive()
            .context("another jw preparation is running; pause other writers before retrying")?;
        Ok(Self { _lock: lock })
    }
}

pub(super) fn run(request: ReseedRequest, source_only: bool, diagnostic: &mut dyn FnMut(&str)) -> Result<ReseedReport> {
    let aggregate_preview = request.dry_run && !source_only;
    let mut readiness = Readiness::default();
    match run_inner(request, source_only, diagnostic, &mut readiness) {
        Err(error) if aggregate_preview => match readiness.not_assessed_report(&error) {
            Some(report) => Ok(report),
            None => Err(error),
        },
        result => result,
    }
}

/// Observations from the ordinary planning pipeline, never a second, weaker
/// preflight implementation. A shared failure cannot turn an unchecked target
/// into READY; target-local failures do not hide the other targets' checks.
#[derive(Default)]
struct Readiness {
    preview: Option<ReseedReport>,
    failures: BTreeMap<String, ReseedOutcome>,
    snapshotted: BTreeMap<String, bool>,
    details: BTreeMap<String, String>,
}

impl Readiness {
    fn block(&mut self, name: &str, error: anyhow::Error) {
        self.failures.entry(name.to_owned()).or_insert_with(|| ReseedOutcome {
            name: name.to_owned(),
            status: ReseedStatus::Failed,
            message: format!("{error:#}"),
        });
    }

    fn require_unblocked(&self) -> Result<()> {
        if !self.failures.is_empty() {
            bail!(
                "reseed preflight blocked; no targets refilled: {}",
                self.failures
                    .values()
                    .map(|target| format!("{}: {}", target.name, target.message))
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
        Ok(())
    }

    fn not_assessed_report(&mut self, error: &anyhow::Error) -> Option<ReseedReport> {
        let mut report = self.preview.take()?;
        report.targets.extend(self.failures.values().cloned());
        for selected in &report.selections {
            if !self.failures.contains_key(&selected.name) {
                report.targets.push(ReseedOutcome {
                    name: selected.name.clone(),
                    status: ReseedStatus::Failed,
                    message: format!("not assessed: shared preflight failed: {error:#}"),
                });
            }
        }
        (!report.targets.is_empty()).then_some(report)
    }
}

fn run_inner(mut request: ReseedRequest, source_only: bool, diagnostic: &mut dyn FnMut(&str), readiness: &mut Readiness) -> Result<ReseedReport> {
    if std::env::var_os("GIT_CONFIG_PARAMETERS").is_some_and(|value| !value.is_empty())
        || std::env::var_os("GIT_CONFIG_COUNT").is_some_and(|value| !value.is_empty() && value != "0")
    {
        bail!(
            "unset active GIT_CONFIG_COUNT/GIT_CONFIG_PARAMETERS before reseed; JJ and the isolated ignore verifier may interpret those overrides differently"
        );
    }
    let caller = JjClient::current()?;
    let caller_cwd = caller.cwd().to_path_buf();
    // Frozen file paths are repository-relative. Root all subsequent fileset
    // queries too, even when jw was invoked from a workspace subdirectory.
    let caller_root = PathBuf::from(caller.run(["--ignore-working-copy", "workspace", "root"])?.trimmed_stdout()?).canonicalize()?;
    let client = JjClient::new(caller_root);
    // Discovery must not snapshot a potentially partial workspace.
    let inventory = Inventory::load(&client)?;
    let default = inventory.default.clone();
    let implicit_current = request.names.is_empty() && request.groups.is_empty();
    if implicit_current {
        if inventory.root(&inventory.current)? == default && !source_only {
            bail!("reseed at the primary checkout requires literal target names or --group; never selects all implicitly");
        }
        // Resolve to one registered literal identity before source/target refresh.
        if !source_only {
            request.names.push(inventory.current.clone());
        }
    }
    let config = load_config(&default)?;
    // Resolve donor path once, before dynamic membership. Keep a failed lookup
    // for shared preflight so dry-run can still report the selected candidates.
    let raw_source = request.source.as_ref().or(config.source.as_ref()).unwrap_or(&default);
    let source_path = if raw_source.is_absolute() {
        raw_source.clone()
    } else if request.source.is_some() {
        caller_cwd.join(raw_source)
    } else {
        default.join(raw_source)
    };
    let resolved_source = source_path.canonicalize();
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let mut selection = if source_only {
        selection::SelectionPlan::default()
    } else {
        selection::resolve(
            &request,
            &config.groups,
            &inventory.registered,
            &default,
            home.as_deref(),
            resolved_source.as_deref().unwrap_or(&source_path),
        )?
    };
    if let Some(warning) = selection.warning() {
        diagnostic(&warning);
    }
    if request.dry_run && !source_only {
        readiness.preview = Some(ReseedReport {
            dry_run: true,
            source: source_path,
            source_commit_id: None,
            preparation_summary: None,
            targets: Vec::new(),
            selections: selection.selections.clone(),
            skipped: selection.skipped.clone(),
        });
        for issue in &selection.issues {
            readiness.failures.insert(issue.name.clone(), issue.clone());
        }
    }
    if !source_only && (!request.dry_run || selection.selections.is_empty()) {
        selection.validate()?;
    }
    if implicit_current && !source_only {
        selection.selections[0].origins = vec!["current".into()];
        if let Some(preview) = readiness.preview.as_mut() {
            preview.selections = selection.selections.clone();
        }
    }
    let names = selection.selections.iter().map(|s| s.name.clone()).collect::<Vec<_>>();
    let metadata = WorkspaceMetadataStore::from_repo_config_path(client.repo_config_path()?)?;
    let repository_path = repository(&client)?;
    let mut plans = Vec::new();
    if config.legacy_prepare {
        diagnostic("warning: [prepare] is a deprecated local alias for [freshen]; rename it to [freshen]");
    }
    let source = resolved_source?;
    let source_client = JjClient::new(&source);
    let root = source_client.run(["--ignore-working-copy", "workspace", "root"])?;
    if PathBuf::from(root.trimmed_stdout()?).canonicalize()? != source {
        bail!("source must name a JJ workspace root");
    }
    if repository(&source_client)? != repository_path {
        bail!("source must belong to the same JJ repository as the targets");
    }
    checked_context(&source_client)?;
    let source_rev = request.source_at.as_ref().or(config.source_at.as_ref()).map(String::as_str);
    let ignored_policy = Policy::load(request.copy_ignored.as_deref(), config.copy_ignored.as_deref(), request.no_copy_ignored)?;
    let ignored_patterns = Patterns::compile(&ignored_policy.patterns)?;
    if ignored_policy.origin != Origin::None {
        diagnostic(&ignored_policy.describe());
    }
    ensure_quiescence(&source_client)?;
    require_full(&source_client)?;
    // INSERT MORE CHECKS HERE: batch/root front guards, before traversal or mutation.
    // Stable mount topology and path bindings are required until this batch ends.
    // Linux captures once, including file bind mounts; no payload-entry mount tax
    // or synthetic reflink probe. Actual required clones can fail after progress.
    let topology = topology::Topology::capture()?;
    topology.workspace(&source, true)?;
    if source_only || request.prepare {
        validate_contents(&source, true)?;
    }
    let mut roots = BTreeSet::new();
    for name in &names {
        if readiness.failures.contains_key(name) {
            continue;
        }
        let result = (|| {
            let root = inventory.root(name)?.canonicalize()?;
            if roots.contains(&root) {
                bail!("multiple selected workspaces alias {}", root.display());
            }
            validate_target(&client, &metadata, &default, &source, name, &root)?;
            ensure_quiescence(&JjClient::new(&root))?;
            require_full(&JjClient::new(&root))?;
            topology.workspace(&root, false)?;
            let mut all_roots = vec![source.clone()];
            all_roots.extend(roots.iter().cloned());
            all_roots.push(root.clone());
            topology.separate(&all_roots)?;
            roots.insert(root);
            Ok(())
        })();
        if let Err(error) = result {
            readiness.block(name, error);
        }
    }
    if !request.dry_run {
        readiness.require_unblocked()?;
    }
    let mut preparation_summary = None;
    let mut prepared_goal = None;
    if source_only || request.prepare {
        // Serialize preparation only. Dropping this guard also releases the lock
        // on every error/preview return, before any reseed planning or refill.
        let _lock = PreparationLock::lock(&client)?;
        let ignored_patterns = ignored_policy.patterns.as_slice();
        prepare::preflight(&metadata, &inventory, &source, config.freshen.as_ref())?;
        snapshot(&source_client)?;
        let (source_operation, source_commit) = current_commit(&source_client)?;
        let source_entries = entries(&source_client, &source_operation, &source_commit)?;
        verify_disk(&source_client, &source_operation, &source_commit, &source, &source_entries)?;
        let selected = SourceSelection::select(
            &source_client,
            &source_operation,
            source_rev,
            config.freshen.as_ref().and_then(|s| s.remote.as_deref()),
        )?;
        if current_commit(&source_client)?.1 != source_commit {
            bail!("source changed during upstream selection; pause writers before retrying");
        }
        if request.dry_run {
            let mut summary = prepare::describe(&source, &selected, config.freshen.as_ref(), ignored_patterns)?;
            summary.push_str(if source_only {
                "; source-only prepare, no targets or target refill"
            } else {
                "; refill readiness deferred until preparation completes; no target snapshots or refill"
            });
            diagnostic(&summary);
            let mut targets = readiness.failures.values().cloned().collect::<Vec<_>>();
            for name in &names {
                if !readiness.failures.contains_key(name) {
                    targets.push(ReseedOutcome {
                        name: name.clone(),
                        status: ReseedStatus::Planned,
                        message: "NOT ASSESSED; refill readiness requires the prepared donor; run plain reseed --dry-run after prepare".into(),
                    });
                }
            }
            return Ok(ReseedReport {
                dry_run: true,
                source,
                source_commit_id: None,
                preparation_summary: Some(summary),
                targets,
                selections: selection.selections,
                skipped: selection.skipped,
            });
        }
        let goal = prepare::execute(&source, config.freshen.as_ref(), ignored_patterns, selected, diagnostic)?;
        preparation_summary = Some(prepare::completion_summary(config.freshen.as_ref())?);
        if source_only {
            return Ok(ReseedReport {
                dry_run: false,
                source,
                source_commit_id: Some(goal),
                preparation_summary,
                targets: Vec::new(),
                selections: Vec::new(),
                skipped: Vec::new(),
            });
        }
        prepared_goal = Some(goal);
    }
    // Assert the donor's prepared tree before snapshotting any target.
    snapshot(&source_client)?;
    let source_operation = client.operation_id()?;
    let source_wc = source_client.resolve_one_at(&source_operation, "@")?.commit_id;
    // A preparation goal can have been prospective before fetch. Keep the exact
    // verified result rather than resolving a caller expression a second time.
    let source_commit = match prepared_goal {
        Some(goal) => goal,
        None => match source_rev {
            Some(rev) => source_client.resolve_one_at(&source_operation, rev)?.commit_id,
            None => source_wc.clone(),
        },
    };
    require_unconflicted(&source_client, &source_operation, &source_commit)?;
    if !changed_paths(&source_client, &source_operation, &source_wc, &source_commit)?.is_empty() {
        bail!("source tracked tree does not match source_at; prepare the baseline before reseeding");
    }
    let source_entries = entries(&source_client, &source_operation, &source_commit)?;
    verify_disk(&source_client, &source_operation, &source_commit, &source, &source_entries)?;
    if let Some(report) = readiness.preview.as_mut() {
        report.source = source.clone();
        report.source_commit_id = Some(source_commit.clone());
    }
    // Full scans freeze dirty target work only after donor validation.
    for name in &names {
        if readiness.failures.contains_key(name) {
            continue;
        }
        let result = (|| {
            let target = JjClient::new(inventory.root(name)?);
            let before = request.dry_run.then(|| current_commit(&target)).transpose()?;
            snapshot(&target)?;
            if let Some((_, before)) = before {
                readiness.snapshotted.insert(name.clone(), current_commit(&target)?.1 != before);
            }
            Ok(())
        })();
        if let Err(error) = result {
            readiness.block(name, error);
        }
    }
    if !request.dry_run {
        readiness.require_unblocked()?;
    }
    let operation = client.operation_id()?;
    if source_client.resolve_one_at(&operation, "@")?.commit_id != source_wc {
        bail!("source changed during target refresh; pause writers before retrying");
    }
    // Pin the source and target trees at the final shared operation.
    let source_operation = operation.clone();
    // The sync target is the donor's current working-copy commit unless --at
    // explicitly overrides it; --source-at stays a tree assertion only.
    let base = match request.at.as_ref() {
        Some(rev) => client.resolve_one_at(&operation, rev)?.commit_id,
        None => source_wc.clone(),
    };
    let mut frozen_rules = FrozenRules::default();
    let source_rules = frozen_rules.get(&source_client, &source_operation, &source_commit, &source_entries)?;
    let selected = payload::select_outside(&source, &ignored_patterns, source_entries.iter().map(|entry| entry.path.as_path()), |path| {
        source_rules.is_ignored_directory(path)
    })?;
    let (ignored, skipped) = (selected.roots, selected.skipped);
    let payload_guard = payload::Guard::default();
    #[cfg(not(target_os = "linux"))]
    for path in &ignored {
        cow::validate_payload_root(&source, path, &mut Vec::new())?;
    }
    let source_entries = Rc::new(source_entries);
    if current_commit(&source_client)?.1 != source_wc {
        bail!("source changed during initial verified capture; pause writers before retrying");
    }
    let facts = client.workspace_target_facts_at(&operation)?;
    let bookmark_coverage = if !request.allow_unbookmarked {
        let commits = names
            .iter()
            .filter(|name| !readiness.failures.contains_key(*name))
            .filter_map(|name| facts.get(name).map(|fact| fact.commit_id.clone()))
            .collect();
        Some(bookmarks::Coverage::capture(&client, &operation, &commits)?)
    } else {
        None
    };
    let mut symlink_commits = BTreeSet::new();
    for name in &names {
        if readiness.failures.contains_key(name) {
            continue;
        }
        let planned = (|| {
            let root = inventory.root(name)?.canonicalize()?;
            let fact = facts.get(name).context("selected workspace disappeared")?;
            if fact.conflicted || fact.divergent {
                bail!("workspace {name} is conflicted or divergent");
            }
            let bookmark_guard = if request.allow_unbookmarked {
                bookmarks::Guard::AllowUnbookmarked
            } else {
                bookmark_coverage.as_ref().context("missing bookmark preflight")?.require(&fact.commit_id)?
            };
            let tree_commit = &base;
            require_unconflicted(&client, &operation, tree_commit)?;
            let target_entries = entries(&client, &operation, tree_commit)?;
            let changed = changed_paths(&client, &operation, &source_commit, tree_commit)?;
            if changed.is_empty() {
                frozen_rules.same_tree(tree_commit, &source_commit);
            }
            let target_rules = frozen_rules.get(&JjClient::new(&root), &operation, tree_commit, &target_entries)?;
            for path in &ignored {
                target_rules.require_ignored(path)?;
                if target_entries.iter().any(|e| overlap(path, &e.path)) {
                    bail!("ignored root overlaps tracked files: {}", path.display());
                }
                confined_parents(&root, path)?;
            }
            payload::outside(&root, false, if request.replace_content { &[] } else { &ignored })?;
            if request.dry_run {
                // Joint read-only planning uses the exact execution visitor,
                // including collision/removability checks; side-local audit is
                // not a substitute. No retained operation map or clone probe.
                let visitor = transfer::Auxiliary::legacy(
                    &ignored,
                    source_entries.iter().chain(&target_entries).map(|entry| entry.path.as_path()),
                    (Identity::read(&source)?.device, Identity::read(&root)?.device),
                )?;
                diff_dirs_sync::Applier::new_checked(&source, &root, diff_dirs_sync::CopyPolicy::ReflinkRequired, payload_guard.borrowed())?
                    .visitor(visitor)
                    .preview()?;
            }
            #[cfg(not(target_os = "linux"))]
            validate_contents(&root, false)?;
            let target_paths = target_entries.iter().map(|e| &e.path).collect::<BTreeSet<_>>();
            let clones = source_entries
                .iter()
                .filter(|e| target_paths.contains(&e.path) && !changed.contains(&e.path))
                .map(|e| e.path.clone())
                .collect::<Vec<_>>();
            let old_entries = entries(&client, &operation, &fact.commit_id)?;
            verify_disk(&JjClient::new(&root), &operation, &fact.commit_id, &root, &old_entries)?;
            // Symlink extraction is a real CLI capability, not inferred from
            // file-list typing. Prove both current and future trees before any
            // target's sparse checkout is changed.
            if !symlink_commits.contains(tree_commit) {
                verify_symlink_support(&client, &operation, tree_commit, &target_entries)?;
                symlink_commits.insert(tree_commit);
            }
            preflight_obstructions(&root, &old_entries, &target_entries, request.replace_content)?;
            preflight_links(
                &default,
                &root,
                &target_entries,
                &ignored,
                request.replace_content,
                &target_rules,
                &FutureContent {
                    root: &root,
                    source: &source,
                    entries: &target_entries,
                    old_paths: &old_entries.iter().map(|e| e.path.clone()).collect::<Vec<_>>(),
                    ignored: &ignored,
                    replace: request.replace_content,
                    client: &client,
                    operation: &operation,
                    commit: tree_commit,
                },
            )?;
            let mut warnings = skipped.clone();
            warnings.extend(payload_guard.warnings());
            if request.dry_run {
                readiness.details.insert(
                    name.clone(),
                    format!(
                        "{}; {}",
                        if readiness.snapshotted.get(name).copied().unwrap_or(false) {
                            "new working-copy updates snapshotted into JJ history"
                        } else {
                            "no new working-copy updates recorded by snapshot"
                        },
                        if fact.empty {
                            "current working-copy tree matches its parent(s)"
                        } else {
                            "uncommitted working-copy changes present"
                        },
                    ),
                );
            }
            Ok(Plan {
                name: name.clone(),
                root: root.clone(),
                root_identity: Identity::read(&root)?,
                jj_identity: Identity::read(&root.join(".jj"))?,
                repository: repository_path.clone(),
                source: source.clone(),
                source_identity: Identity::read(&source)?,
                source_jj_identity: Identity::read(&source.join(".jj"))?,
                source_operation: source_operation.clone(),
                source_commit: source_commit.clone(),
                source_wc_commit: source_wc.clone(),
                target_operation: operation.clone(),
                old_commit: fact.commit_id.clone(),
                desired_commit: fact.commit_id.clone(),
                base_commit: base.clone(),
                clones,
                entries: target_entries,
                old_paths: old_entries.into_iter().map(|e| e.path).collect(),
                ignored: ignored.clone(),
                source_rules: Rc::clone(&source_rules),
                source_entries: Rc::clone(&source_entries),
                target_rules,
                skipped: warnings,
                replace_content: request.replace_content,
                config_root: default.clone(),
                link_config: link_config_bytes(&default)?,
                execution_diagnostics: Vec::new(),
                bookmark_guard,
            })
        })();
        match planned {
            Ok(plan) => plans.push(plan),
            Err(error) => readiness.block(name, error),
        }
    }
    if !request.dry_run {
        readiness.require_unblocked()?;
    }
    let first = plans.first();
    let (report_source, report_commit) = match first {
        Some(first) => (first.source.clone(), first.source_commit.clone()),
        None if request.dry_run && readiness.preview.is_some() => {
            let preview = readiness.preview.as_ref().unwrap();
            (preview.source.clone(), preview.source_commit_id.clone().context("source proof unavailable")?)
        }
        None => bail!("select at least one workspace"),
    };
    let mut report = ReseedReport {
        dry_run: request.dry_run,
        source: report_source,
        source_commit_id: Some(report_commit),
        preparation_summary,
        targets: Vec::new(),
        selections: selection.selections,
        skipped: selection.skipped,
    };
    if request.dry_run {
        report.targets.extend(readiness.failures.values().cloned());
    } else {
        // Check the entire batch immediately before execution as well as each
        // target's own clearing boundary. Never refill an initially-invalid subset.
        bookmarks::verify_batch(&client, plans.iter().map(|plan| &plan.bookmark_guard))?;
    }
    let payload_execution = transfer::Execution::new(!request.dry_run && plans.iter().any(|plan| !plan.ignored.is_empty()))?;
    for mut plan in plans {
        if request.dry_run {
            let consequence = format!("SYNC to {}; tracked work saved in JJ history at {}", plan.base_commit, plan.old_commit);
            report.targets.push(ReseedOutcome {
                name: plan.name.clone(),
                status: ReseedStatus::Planned,
                message: format!(
                    "READY; {consequence}; {}; {}",
                    readiness.details.get(&plan.name).map(String::as_str).unwrap_or("no new target snapshot"),
                    describe(&plan, true)
                ),
            });
            continue;
        }
        let started = std::time::Instant::now();
        let outcome = execute(&default, &mut plan, &payload_execution);
        let (status, message) = match outcome {
            Ok(()) => (
                ReseedStatus::Succeeded,
                format!("{}; elapsed {} ms", describe(&plan, false), started.elapsed().as_millis()),
            ),
            Err(error) => (ReseedStatus::Failed, format!("{error:#}")),
        };
        report.targets.push(ReseedOutcome {
            name: plan.name.clone(),
            status,
            message,
        });
    }
    Ok(report)
}

struct Inventory {
    current: String,
    default: PathBuf,
    roots: BTreeMap<String, PathBuf>,
    registered: Registrations,
}
impl Inventory {
    fn load(client: &JjClient) -> Result<Self> {
        let operation = client.operation_id()?;
        let mut roots = BTreeMap::new();
        let mut registered = Registrations::new();
        for name in client.workspace_target_facts_at(&operation)?.keys() {
            let output = client.run_at_unchecked(&operation, ["workspace", "root", "--name", name])?;
            registered.insert(name.clone(), None);
            if output.success() {
                let path = PathBuf::from(output.trimmed_stdout()?);
                registered.insert(name.clone(), Some(path.clone()));
                if let Ok(path) = path.canonicalize() {
                    roots.insert(name.clone(), path);
                }
            }
        }
        let context = checked_context(client).context("cannot verify primary/caller JJ workspace topology")?;
        let expected_repository = context
            .jj
            .repository_path
            .as_ref()
            .context("cannot verify JJ repository identity")?
            .canonicalize()?;
        // A workspace named `default` need not own the repository. Only the
        // unique registered checkout containing the actual repository qualifies.
        let owners = roots
            .iter()
            .filter(|(_, root)| {
                let pointer = root.join(".jj/repo");
                fs::symlink_metadata(&pointer).is_ok_and(|m| m.is_dir()) && pointer.canonicalize().ok().as_ref() == Some(&expected_repository)
            })
            .collect::<Vec<_>>();
        if owners.len() != 1 {
            bail!("cannot verify primary JJ workspace: repository owner registration is missing or ambiguous");
        }
        let (primary_name, default) = owners[0];
        // Context may omit primary_checkout for a JJ-only checkout nested inside
        // an unrelated Git checkout. The registered physical repository owner
        // still proves JJ identity; never infer it from the enclosing Git root.
        if context.jj.primary_checkout.as_ref().is_some_and(|root| root != default)
            || context.jj.primary_workspace.as_ref().is_some_and(|name| name != primary_name)
        {
            bail!("primary JJ workspace discovery disagrees with repository owner registration");
        }
        let primary = checked_context(&JjClient::new(default)).context("cannot verify primary JJ workspace topology")?;
        if primary.jj.workspace_root.as_ref() != Some(default)
            || primary.jj.workspace_name.as_ref() != Some(primary_name)
            || repository(&JjClient::new(default))? != expected_repository
            || context.jj.workspace_name.as_ref().and_then(|name| roots.get(name)) != Some(&client.cwd().canonicalize()?)
        {
            bail!("primary/caller JJ workspace root registration is inconsistent");
        }
        Ok(Self {
            current: context.jj.workspace_name.context("cannot determine registered current workspace")?,
            default: default.clone(),
            roots,
            registered,
        })
    }
    fn root(&self, name: &str) -> Result<&Path> {
        self.roots
            .get(name)
            .map(PathBuf::as_path)
            .with_context(|| format!("workspace {name} has no usable recorded root"))
    }
}

fn repository(client: &JjClient) -> Result<PathBuf> {
    // Native metadata already names the repository. Full Git/JJ topology is
    // inspected at selection/ownership boundaries, not rediscovered for every
    // repeated identity proof while refilling the same registered directories.
    crate::context::repository_path(client.cwd())
        .map_err(anyhow::Error::msg)?
        .canonicalize()
        .context("cannot resolve JJ repository identity")
}

fn checked_context(client: &JjClient) -> Result<crate::context::ContextReport> {
    let context = crate::context::discover(Some(client.cwd()));
    if context
        .diagnostics
        .iter()
        .any(|d| matches!(d.severity, crate::context::DiagnosticSeverity::Error))
    {
        bail!(
            "inconsistent checkout/repository topology: {}",
            context.diagnostics.iter().map(|d| d.message.as_str()).collect::<Vec<_>>().join("; ")
        );
    }
    Ok(context)
}

fn link_config_bytes(root: &Path) -> Result<Vec<(String, Option<Vec<u8>>)>> {
    [".jwlinks.toml", ".jwlinks.local.toml"]
        .into_iter()
        .map(|name| {
            let bytes = match fs::read(root.join(name)) {
                Ok(bytes) => Some(bytes),
                Err(e) if e.kind() == ErrorKind::NotFound => None,
                Err(e) => return Err(e.into()),
            };
            Ok((name.to_owned(), bytes))
        })
        .collect()
}

fn load_config(default: &Path) -> Result<LocalConfig> {
    let path = default.join(".jwreseed.local.toml");
    match fs::read(&path) {
        Ok(bytes) => {
            let mut config: LocalConfig = toml::from_str(std::str::from_utf8(&bytes)?).with_context(|| format!("invalid {}", path.display()))?;
            selection::validate_groups(&config.groups)?;
            if config.freshen.is_some() && config.prepare.is_some() {
                bail!("[freshen] and legacy [prepare] cannot both be present; keep only [freshen]");
            }
            if let Some(legacy) = config.prepare.take() {
                config.freshen = Some(legacy);
                config.legacy_prepare = true;
            }
            if let Some(settings) = &config.freshen {
                settings.argv()?;
            }
            Ok(config)
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(LocalConfig::default()),
        Err(e) => Err(e).with_context(|| format!("cannot read {}", path.display())),
    }
}

fn validate_target(client: &JjClient, metadata: &WorkspaceMetadataStore, default: &Path, source: &Path, name: &str, root: &Path) -> Result<()> {
    if root == default || root == source || root.starts_with(source) || source.starts_with(root) {
        bail!("workspace {name} is primary/source or overlaps the source; not an eligible target");
    }
    let context = checked_context(&JjClient::new(root))?;
    if context
        .jj
        .primary_checkout
        .as_ref()
        .is_some_and(|p| p.canonicalize().ok().as_deref() == Some(root))
    {
        bail!("cannot reseed the primary store workspace");
    }
    if repository(&JjClient::new(root))? != repository(client)? {
        bail!("target repository differs");
    }
    Identity::read(&root.join(".jj"))?;
    if fs::symlink_metadata(root.join(".git")).is_ok() {
        bail!("workspace {name} is colocated; reseed supports JJ-only targets");
    }
    let record = metadata.get(name)?;
    if record.as_ref().is_some_and(|m| m.external_owner.is_some() || m.owned_git_worktree.is_some()) {
        bail!("workspace {name} has external/colocated ownership");
    }
    crate::ownership::ensure_removal_allowed(client, name, Some(root), record.as_ref())?;
    // Several workspaces can deliberately point at the same commit. Commit-level
    // current-working-copy predicates alone therefore do not identify a root.
    if context.jj.workspace_name.as_deref() != Some(name) || context.jj.workspace_root.as_deref() != Some(root) {
        bail!("workspace root registration does not uniquely match {name}");
    }
    Ok(())
}

fn ensure_quiescence(client: &JjClient) -> Result<()> {
    if client.uses_watchman()? {
        bail!("Watchman workspaces are unsupported for reseed; disable its monitor and background snapshot triggers and pause writers first");
    }
    let trigger = client.run_unchecked(["--ignore-working-copy", "config", "get", "fsmonitor.watchman.register-snapshot-trigger"])?;
    if trigger.success() && trigger.trimmed_stdout()? == "true" {
        bail!("Watchman snapshot triggers are unsupported; stop background triggers before reseeding");
    }
    let backend = client.run_unchecked(["--ignore-working-copy", "config", "get", "fsmonitor.backend"])?;
    if backend.success() && !matches!(backend.trimmed_stdout()?.as_str(), "none" | "") {
        bail!("unsupported filesystem monitor; reseed requires full-scan quiescent workspaces");
    }
    Ok(())
}

fn require_full(client: &JjClient) -> Result<()> {
    let output = client.run(["--ignore-working-copy", "sparse", "list"])?;
    if output.stdout()?.trim_end_matches(['\r', '\n']) != "." {
        bail!(
            "workspace is sparse or mid-refill; reseed requires full sparse patterns ('.'): {}; recover out of band with `jj new <bookmark>` and `jj sparse reset` in {}, then re-run reseed; jw will not repair it",
            client.cwd().display(),
            client.cwd().display(),
        );
    }
    Ok(())
}

fn snapshot(client: &JjClient) -> Result<()> {
    let output = client.run(["--config=fsmonitor.backend=none", "--config=snapshot.auto-update-stale=false", "status"])?;
    diagnostics(output.stderr());
    Ok(())
}

fn diagnostics(text: String) {
    if !text.is_empty() {
        eprintln!("{text}");
    }
}

fn require_unconflicted(client: &JjClient, operation: &str, commit: &str) -> Result<()> {
    let output = client.run_at(operation, ["log", "--no-graph", "-r", commit, "-T", "json(conflict) ++ \"\\n\""])?;
    if serde_json::from_str::<bool>(output.stdout()?.trim())? {
        bail!("reseed does not support conflicted trees");
    }
    Ok(())
}

fn changed_paths(client: &JjClient, operation: &str, from: &str, to: &str) -> Result<BTreeSet<PathBuf>> {
    let output = client
        .run_at(
            operation,
            [
                "diff",
                "--from",
                from,
                "--to",
                to,
                "-T",
                r#"json(source.path()) ++ "\n" ++ json(target.path()) ++ "\n""#,
            ],
        )
        .context("JJ lacks reseed JSON diff endpoint support; upgrade JJ")?;
    parse_changed_paths(output.stdout()?)
}

fn parse_changed_paths(output: &str) -> Result<BTreeSet<PathBuf>> {
    let mut paths = BTreeSet::new();
    let lines = output.lines().collect::<Vec<_>>();
    if lines.len() % 2 != 0 {
        bail!("malformed JJ diff endpoints");
    }
    for line in lines {
        let path = PathBuf::from(serde_json::from_str::<String>(line)?);
        relative(&path)?;
        paths.insert(path);
    }
    Ok(paths)
}

/// `file show` intentionally skips symlinks on supported stock JJ releases. A
/// root-to-commit native Git diff exposes the complete symlink payload instead.
/// Parse only an added 120000-mode file; never treat a warning/empty stdout as an
/// empty target. Unsupported binary representations fail in source preflight.
fn symlink_bytes(client: &JjClient, operation: &str, commit: &str, path: &Path) -> Result<Vec<u8>> {
    let fileset = format!("file:{}", serde_json::to_string(path.to_str().context("tracked path not UTF-8")?)?);
    let output = client.run_at(operation, ["diff", "--from", "root()", "--to", commit, "--git", &fileset])?;
    diagnostics(output.stderr());
    parse_added_symlink(output.stdout_bytes()).with_context(|| {
        format!(
            "JJ cannot expose the frozen symlink target at {}; capability gap detected before materialization",
            path.display()
        )
    })
}

/// JJ computes whole-tree copy records before applying a diff's fileset. Batch
/// capability checks so that cost is paid once per group, not once per symlink.
/// Keep the individual immutable extractor for future-link queries.
fn verify_symlink_support(client: &JjClient, operation: &str, commit: &str, entries: &[Entry]) -> Result<()> {
    let paths = entries
        .iter()
        .filter(|entry| entry.kind == "symlink")
        .map(|entry| entry.path.as_path())
        .collect::<Vec<_>>();
    // Bound argv size while preserving JJ file-list/tree-diff traversal order.
    for paths in paths.chunks(128) {
        let mut args = vec![
            "--config=diff.git.show-path-prefix=true".to_owned(),
            "diff".to_owned(),
            "--from".to_owned(),
            "root()".to_owned(),
            "--to".to_owned(),
            commit.to_owned(),
            "--git".to_owned(),
        ];
        for path in paths {
            args.push(format!("file:{}", serde_json::to_string(path.to_str().context("tracked path not UTF-8")?)?));
        }
        let output = client.run_at(operation, &args)?;
        diagnostics(output.stderr());
        parse_added_symlink_batch(output.stdout_bytes(), paths)
            .context("JJ cannot expose frozen symlink targets; capability gap detected before materialization")?;
    }
    Ok(())
}

fn parse_added_symlink_batch(mut remaining: &[u8], paths: &[&Path]) -> Result<()> {
    fn consume(remaining: &mut &[u8], expected: &[u8]) -> Result<()> {
        *remaining = remaining.strip_prefix(expected).context("unexpected symlink diff header or boundary")?;
        Ok(())
    }
    fn line<'a>(remaining: &mut &'a [u8]) -> Result<&'a [u8]> {
        let end = remaining.iter().position(|byte| *byte == b'\n').context("truncated symlink diff line")? + 1;
        let (line, rest) = remaining.split_at(end);
        *remaining = rest;
        Ok(line)
    }
    for path in paths {
        let start = remaining;
        let path = path.to_str().context("tracked path not UTF-8")?;
        // Consume known pathname bytes literally. Splitting at apparent headers
        // would let newline filenames spoof another file or a payload hunk.
        consume(&mut remaining, format!("diff --git a/{path} b/{path}\nnew file mode 120000\nindex ").as_bytes())?;
        let index = std::str::from_utf8(line(&mut remaining)?)?.trim_end_matches('\n');
        let (old, new) = index.split_once("..").context("invalid symlink diff index")?;
        if old.is_empty() || !old.bytes().all(|b| b == b'0') || new.is_empty() || !new.bytes().all(|b| b.is_ascii_hexdigit()) {
            bail!("invalid added symlink diff index");
        }
        consume(&mut remaining, format!("--- /dev/null\n+++ b/{path}\n").as_bytes())?;
        let hunk = std::str::from_utf8(line(&mut remaining)?)?;
        let range = hunk
            .strip_prefix("@@ -0,0 +1")
            .and_then(|s| s.strip_suffix(" @@\n"))
            .context("JJ did not provide a full-addition symlink hunk")?;
        let count = if range.is_empty() {
            1
        } else {
            range.strip_prefix(',').context("invalid symlink hunk range")?.parse::<usize>()?
        };
        if count == 0 {
            bail!("empty symlink targets are unsupported");
        }
        for _ in 0..count {
            if !line(&mut remaining)?.starts_with(b"+") {
                bail!("symlink diff contains a non-addition");
            }
        }
        if remaining.starts_with(b"\\ No newline at end of file\n") {
            consume(&mut remaining, b"\\ No newline at end of file\n")?;
        }
        parse_added_symlink(&start[..start.len() - remaining.len()])?;
    }
    if !remaining.is_empty() {
        bail!("symlink diff contains unexpected files or trailing data");
    }
    Ok(())
}

fn parse_added_symlink(diff: &[u8]) -> Result<Vec<u8>> {
    let lines = diff.split_inclusive(|byte| *byte == b'\n').collect::<Vec<_>>();
    if !lines.contains(&b"new file mode 120000\n".as_slice()) {
        bail!("JJ did not provide added symlink mode");
    }
    // Human headers are NOT C-quoted by JJ. A newline pathname can spoof a
    // hunk inside them. The last full-addition hunk is authoritative: every
    // subsequent payload line is +prefixed and cannot spoof an unprefixed @@.
    let (index, count) = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let line = std::str::from_utf8(line).ok()?.trim_end_matches('\n');
            let range = line.strip_prefix("@@ -0,0 +1")?.strip_suffix(" @@")?;
            let count = if range.is_empty() {
                1
            } else {
                range.strip_prefix(',')?.parse::<usize>().ok()?
            };
            (count > 0).then_some((index, count))
        })
        .next_back()
        .context("JJ did not provide a full-addition symlink hunk")?;
    let body = lines.get(index + 1..).context("missing symlink body")?;
    if body.len() < count {
        bail!("truncated symlink diff body");
    }
    let mut result = Vec::new();
    for line in &body[..count] {
        result.extend_from_slice(line.strip_prefix(b"+").context("symlink diff contains a non-addition")?);
    }
    match &body[count..] {
        [] => {}
        [line] if *line == b"\\ No newline at end of file\n" || *line == b"\\ No newline at end of file" => {
            if result.pop() != Some(b'\n') {
                bail!("malformed symlink diff newline marker");
            }
        }
        _ => bail!("symlink diff contains extra data after its complete body"),
    }
    if result.is_empty() {
        bail!("empty symlink targets are unsupported");
    }
    Ok(result)
}

/// Verify an intact, full-sparse workspace using JJ's tracked-tree semantics.
/// This deliberately snapshots (and can record a rejected edit), but never tracks
/// additional files and disables automatic stale-tree updates. Sparse/partial
/// trees are refused before any snapshot-capable command.
fn verify_disk(client: &JjClient, _operation: &str, commit: &str, root: &Path, _entries: &[Entry]) -> Result<()> {
    if client.cwd().canonicalize()? != root.canonicalize()? {
        bail!("tracked-tree verification root differs from JJ workspace");
    }
    require_full(client)?;
    // No --at-operation or --ignore-working-copy here: the live tracked files
    // must enter JJ's tree before comparison with the immutable frozen commit.
    let output = client.run([
        "--config=fsmonitor.backend=none",
        "--config=snapshot.auto-track=none()",
        "--config=snapshot.auto-update-stale=false",
        "diff",
        "--from",
        "@",
        "--to",
        &crate::jj::commit_id_revset(commit)?,
        "--summary",
    ])?;
    diagnostics(output.stderr());
    if !output.stdout_bytes().is_empty() {
        bail!("tracked tree differs from frozen desired tree in {}; JJ snapshot performed", root.display());
    }
    Ok(())
}

fn validate_contents(root: &Path, allow_git: bool) -> Result<()> {
    Identity::read(root)?;
    for child in fs::read_dir(root)? {
        let child = child?;
        if child.file_name() == ".jj" || (allow_git && child.file_name() == ".git") {
            continue;
        }
        let metadata = fs::symlink_metadata(child.path())?;
        cow::validate_same_mount(root, &child.path())?;
        if metadata.is_dir() {
            cow::validate_tree(&child.path())?;
        } else if !metadata.is_file() && !metadata.file_type().is_symlink() {
            bail!("special entry is unsupported: {}", child.path().display());
        }
    }
    Ok(())
}

fn overlap(left: &Path, right: &Path) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

fn preflight_obstructions(root: &Path, old: &[Entry], desired: &[Entry], replace: bool) -> Result<()> {
    if replace {
        return Ok(());
    }
    let old_paths = old.iter().map(|entry| entry.path.as_path()).collect::<BTreeSet<_>>();
    for entry in desired {
        // An old tracked path is removed by sparse clearing. Everything else
        // belongs to the operator and cannot silently be overwritten by checkout.
        let mut path = PathBuf::new();
        for component in entry.path.components() {
            path.push(component);
            match fs::symlink_metadata(root.join(&path)) {
                Ok(m) if path == entry.path || !m.is_dir() => {
                    if m.is_dir() && tracked_only_directory(root, &path, &old_paths)? {
                        continue;
                    }
                    if !old_paths.contains(path.as_path()) {
                        bail!(
                            "untracked obstruction at {}; use --replace-content only if losing it is intended",
                            path.display()
                        );
                    }
                    if !m.is_dir() {
                        // A recorded ancestor leaf will disappear during sparse
                        // clearing. Do not inspect through its old symlink (or
                        // ask the OS for children of an old regular file).
                        break;
                    }
                }
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::NotFound => break,
                Err(e) => return Err(e.into()),
            }
        }
    }
    Ok(())
}

fn tracked_only_directory(root: &Path, path: &Path, old: &BTreeSet<&Path>) -> Result<bool> {
    let children = fs::read_dir(root.join(path))?.collect::<std::io::Result<Vec<_>>>()?;
    if children.is_empty() {
        return Ok(false);
    }
    for child in children {
        let path = path.join(child.file_name());
        if child.file_type()?.is_dir() {
            if !tracked_only_directory(root, &path, old)? {
                return Ok(false);
            }
        } else if !old.contains(path.as_path()) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn preflight_links(
    default: &Path,
    root: &Path,
    entries: &[Entry],
    ignored: &[PathBuf],
    replace: bool,
    rules: &IgnoreRules,
    future: &FutureContent<'_>,
) -> Result<()> {
    if let Some(config) = links::load_link_config(default)? {
        let checks = links::inspect_loaded_workspace_links(&config, root);
        for (index, left) in checks.iter().enumerate() {
            for right in checks.iter().skip(index + 1) {
                if overlap(&left.source, &right.source) {
                    bail!("configured link sources overlap: {} and {}", left.source.display(), right.source.display());
                }
            }
        }
        for check in checks {
            let path = check.source.strip_prefix(root).context("link source escapes target")?;
            relative(path)?;
            let target_exists = future.exists(&check.target, 0)?;
            if check.required && !target_exists {
                bail!("required link target will be absent after reseed: {}", check.target.display());
            }
            // A source-equals-target override deliberately keeps a private directory.
            // It performs no link mutation, but required existence must still be
            // proved against the future tree (replace-content may erase it).
            if check.source == check.target {
                continue;
            }
            if entries.iter().any(|e| overlap(path, &e.path)) || ignored.iter().any(|p| overlap(path, p)) {
                bail!("configured link overlaps reseed content: {}", path.display());
            }
            if !replace && matches!(check.state, links::LinkCheckState::Conflicting | links::LinkCheckState::Unreadable(_)) {
                bail!("configured link conflict at {}", path.display());
            }
            if target_exists {
                rules.require_ignored_file(path)?;
            }
        }
    }
    Ok(())
}

struct FutureContent<'a> {
    root: &'a Path,
    source: &'a Path,
    entries: &'a [Entry],
    old_paths: &'a [PathBuf],
    ignored: &'a [PathBuf],
    replace: bool,
    client: &'a JjClient,
    operation: &'a str,
    commit: &'a str,
}
enum FutureNode {
    Directory,
    File,
    Symlink(PathBuf),
    Missing,
}
impl FutureContent<'_> {
    fn exists(&self, path: &Path, depth: usize) -> Result<bool> {
        if depth > 40 {
            bail!("future link target contains a symlink cycle: {}", path.display());
        }
        if path == self.root {
            return Ok(true);
        }
        let Ok(relative) = path.strip_prefix(self.root) else {
            return existing_target(path);
        };
        let components = relative.components().collect::<Vec<_>>();
        let mut current = self.root.to_path_buf();
        for (index, component) in components.iter().enumerate() {
            match component {
                Component::CurDir => continue,
                Component::ParentDir => {
                    current.pop();
                    if !current.starts_with(self.root) {
                        for component in &components[index + 1..] {
                            current.push(component.as_os_str());
                        }
                        return existing_target(&current);
                    }
                }
                Component::Normal(name) => current.push(name),
                _ => bail!("invalid future target path"),
            }
            let relative = current.strip_prefix(self.root)?;
            match self.node(relative)? {
                FutureNode::Directory => {}
                FutureNode::File => return Ok(index + 1 == components.len()),
                FutureNode::Missing => return Ok(false),
                FutureNode::Symlink(target) => {
                    let mut target = if target.is_absolute() {
                        target
                    } else {
                        current.parent().context("link has no parent")?.join(target)
                    };
                    for component in &components[index + 1..] {
                        target.push(component.as_os_str());
                    }
                    return self.exists(&target, depth + 1);
                }
            }
        }
        Ok(true)
    }
    fn node(&self, path: &Path) -> Result<FutureNode> {
        if path.as_os_str().is_empty() {
            return Ok(FutureNode::Directory);
        }
        if let Some(entry) = self.entries.iter().find(|e| e.path == path) {
            if entry.kind == "symlink" {
                use std::os::unix::ffi::OsStringExt;
                return Ok(FutureNode::Symlink(
                    std::ffi::OsString::from_vec(symlink_bytes(self.client, self.operation, self.commit, path)?).into(),
                ));
            }
            return Ok(FutureNode::File);
        }
        if self.entries.iter().any(|e| e.path.starts_with(path)) || self.ignored.iter().any(|p| p.starts_with(path)) {
            return Ok(FutureNode::Directory);
        }
        if self.ignored.iter().any(|p| path.starts_with(p)) {
            return node_on_disk(&self.source.join(path));
        }
        if self.replace || self.old_paths.iter().any(|p| p == path) {
            return Ok(FutureNode::Missing);
        }
        let actual = self.root.join(path);
        match node_on_disk(&actual)? {
            FutureNode::Directory if self.old_paths.iter().any(|p| p.starts_with(path)) && !self.has_retained_child(path)? => Ok(FutureNode::Missing),
            node => Ok(node),
        }
    }
    fn has_retained_child(&self, path: &Path) -> Result<bool> {
        for child in fs::read_dir(self.root.join(path))? {
            let child = child?;
            let path = path.join(child.file_name());
            if self.ignored.iter().any(|p| path.starts_with(p)) {
                continue;
            }
            if child.file_type()?.is_dir() {
                if !self.old_paths.iter().any(|p| p.starts_with(&path)) || self.has_retained_child(&path)? {
                    return Ok(true);
                }
            } else if !self.old_paths.contains(&path) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
fn existing_target(path: &Path) -> Result<bool> {
    match fs::metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if matches!(e.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => Ok(false),
        Err(e) => Err(e.into()),
    }
}
fn node_on_disk(path: &Path) -> Result<FutureNode> {
    match fs::symlink_metadata(path) {
        Ok(m) if m.is_dir() => Ok(FutureNode::Directory),
        Ok(m) if m.file_type().is_symlink() => Ok(FutureNode::Symlink(fs::read_link(path)?)),
        Ok(m) if m.is_file() => Ok(FutureNode::File),
        Ok(_) => bail!("special future link target unsupported: {}", path.display()),
        Err(e) if matches!(e.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => Ok(FutureNode::Missing),
        Err(e) => Err(e.into()),
    }
}

fn describe(plan: &Plan, planned: bool) -> String {
    let mode = format!("sync {}", plan.base_commit);
    let description = format!(
        "root {}; {mode}; {} {} tracked paths, {} {} paths; {} ignored roots [{}]; {}{}",
        plan.root.display(),
        if planned { "would reflink" } else { "reflinked" },
        plan.clones.len(),
        if planned { "JJ would materialize" } else { "JJ materialized" },
        plan.entries.len().saturating_sub(plan.clones.len()),
        if planned { "would reconcile" } else { "reconciled" },
        plan.ignored.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>().join(", "),
        if plan.replace_content {
            "replace all non-administrative content"
        } else {
            "retain unrelated ignored/untracked content"
        },
        if plan.skipped.is_empty() && plan.execution_diagnostics.is_empty() {
            String::new()
        } else {
            format!(
                "; {}",
                plan.skipped
                    .iter()
                    .chain(&plan.execution_diagnostics)
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        }
    );
    format!("{description}; {}", plan.bookmark_guard.describe())
}

fn select_ignored(source: &Path, patterns: &[String], rules: &IgnoreRules, entries: &[Entry]) -> Result<(Vec<PathBuf>, Vec<String>)> {
    let selected = Patterns::compile(patterns)?.select_with_topology(
        source,
        entries.iter().map(|entry| entry.path.as_path()),
        |path| rules.is_ignored_directory(path),
        cfg!(target_os = "linux"),
    )?;
    Ok((selected.roots, selected.skipped))
}

fn validate_selected_ignored(source: &Path, roots: &[PathBuf], rules: &IgnoreRules, entries: &[Entry]) -> Result<()> {
    cow::ignored::validate_selected_with_topology(
        source,
        roots,
        entries.iter().map(|entry| entry.path.as_path()),
        |path| rules.is_ignored_directory(path),
        cfg!(target_os = "linux"),
    )
}

fn validate_identities(plan: &Plan) -> Result<()> {
    plan.root_identity.verify(&plan.root)?;
    plan.jj_identity.verify(&plan.root.join(".jj"))?;
    plan.source_identity.verify(&plan.source)?;
    plan.source_jj_identity.verify(&plan.source.join(".jj"))?;
    if repository(&JjClient::new(&plan.root))? != plan.repository || repository(&JjClient::new(&plan.source))? != plan.repository {
        bail!("repository identity changed during reseed");
    }
    match fs::symlink_metadata(plan.root.join(".git")) {
        Ok(_) => bail!("target acquired Git metadata since preflight"),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error).context("cannot recheck target Git metadata"),
    }
    for path in plan
        .clones
        .iter()
        .chain(&plan.old_paths)
        .chain(&plan.ignored)
        .chain(plan.entries.iter().map(|e| &e.path))
    {
        relative(path)?;
    }
    Ok(())
}

fn validate_source(plan: &Plan) -> Result<()> {
    plan.source_identity.verify(&plan.source)?;
    plan.source_jj_identity.verify(&plan.source.join(".jj"))?;
    if link_config_bytes(&plan.config_root)? != plan.link_config {
        bail!("workspace link configuration changed since preflight");
    }
    let source = JjClient::new(&plan.source);
    require_full(&source)?;
    let operation = source.operation_id()?;
    let current = source.resolve_one_at(&operation, "@")?.commit_id;
    if current != plan.source_wc_commit {
        bail!(
            "the donor {} changed while this command was running; pause writers and run reseed again",
            plan.source.display(),
        );
    }
    verify_disk(&source, &plan.source_operation, &plan.source_commit, &plan.source, &[])?;
    if !plan.ignored.is_empty() {
        validate_selected_ignored(&plan.source, &plan.ignored, &plan.source_rules, &plan.source_entries)?;
    }
    if current_commit(&source)?.1 != plan.source_wc_commit {
        bail!("donor changed during tracked-tree verification; pause writers before retrying");
    }
    Ok(())
}

fn current_commit(client: &JjClient) -> Result<(String, String)> {
    let operation = client.operation_id()?;
    let commit = client.resolve_one_at(&operation, "@")?.commit_id;
    Ok((operation, commit))
}

fn exact_empty_child(client: &JjClient, operation: &str, commit: &str, base: &str) -> Result<bool> {
    let parents = client.resolve_all_at(operation, format!("parents({commit})"))?;
    let output = client.run_at(operation, ["log", "--no-graph", "-r", commit, "-T", "json(empty) ++ \"\\n\""])?;
    Ok(parents.len() == 1 && parents[0].commit_id == base && serde_json::from_str::<bool>(output.stdout()?.trim())?)
}

/// A single forward pass. Nothing persists intermediate progress, and no path
/// attempts to reconstruct a partially filled target.
///
/// The root and target-owned `.jj` stay in place throughout. Empty sparse
/// patterns suppress disk checkout, not the file tree inherited by `jj new`.
/// Only create-only tracked prefills happen while sparse-empty; snapshot-capable
/// verification waits for expansion. Placeholder stats can require reading those
/// recorded files even with new-file auto-tracking disabled. This path does not
/// adopt donor tree-state or checkout/name bindings.
fn execute(default: &Path, plan: &mut Plan, payload_execution: &transfer::Execution) -> Result<()> {
    if default != plan.config_root {
        bail!("default workspace changed since frozen plan");
    }
    validate_identities(plan)?;
    validate_source(plan)?;
    let client = JjClient::new(&plan.root);
    ensure_quiescence(&client)?;
    payload::outside(&plan.root, false, if plan.replace_content { &[] } else { &plan.ignored })?;
    for warning in plan.skipped.iter().filter(|warning| warning.starts_with("absolute dependency symlink ")) {
        diagnostics(format!("Warning: {warning}"));
    }
    require_full(&client)?;
    if current_commit(&client)?.1 != plan.old_commit {
        bail!("target WC revision changed after batch preflight; refused before sparse mutation");
    }
    // Snapshot tracked edits without auto-tracking new data, then reject any
    // change to the work already covered by the batch bookmark proof.
    verify_disk(&client, &plan.target_operation, &plan.old_commit, &plan.root, &[])?;
    if current_commit(&client)?.1 != plan.old_commit {
        bail!("target WC revision changed during tracked-tree verification");
    }
    bookmarks::verify_batch(&client, [&plan.bookmark_guard])?;
    let result = (|| {
        let output = client.run([
            "--config=fsmonitor.backend=none",
            "--config=snapshot.auto-track=none()",
            "--config=snapshot.auto-update-stale=false",
            "sparse",
            "set",
            "--clear",
        ])?;
        diagnostics(output.stderr());
        if current_commit(&client)?.1 != plan.desired_commit {
            bail!("sparse clearing changed the pinned current commit");
        }
        for path in plan.old_paths.iter().chain(plan.entries.iter().map(|e| &e.path)) {
            remove_recorded_file(&plan.root, path)?;
        }
        {
            let base = plan.base_commit.clone();
            if current_commit(&client)?.1 != plan.desired_commit {
                bail!("target changed before jj new");
            }
            let output = client.run([
                "--config=fsmonitor.backend=none",
                "--config=snapshot.auto-track=none()",
                "--config=snapshot.auto-update-stale=false",
                "new",
                &crate::jj::commit_id_revset(&base)?,
            ])?;
            diagnostics(output.stderr());
            let (operation, desired) = current_commit(&client)?;
            if !exact_empty_child(&client, &operation, &desired, &base)? {
                bail!("JJ did not create the expected empty child");
            }
            plan.desired_commit = desired;
            plan.target_operation = operation;
        }
        // Remaining contents are retained unless replacement was explicitly selected.
        if plan.replace_content {
            payload::outside(&plan.root, false, &[])?;
            diff_dirs_sync::clear_with_visitor(&plan.root, transfer::WorkspaceClear)?;
        }
        validate_source(plan)?;
        for path in &plan.clones {
            confined_parents(&plan.root, path)?;
        }
        cow::clone_files(&plan.source, &plan.clones, &plan.root)?;
        // A normal sparse reset snapshots only the existing EMPTY sparse selection,
        // not the staged prefills outside it. JJ then materializes all differences.
        if current_commit(&client)?.1 != plan.desired_commit {
            bail!("target changed before sparse expansion");
        }
        let output = client.run([
            "--config=fsmonitor.backend=none",
            "--config=snapshot.auto-track=none()",
            "--config=snapshot.auto-update-stale=false",
            "sparse",
            "reset",
        ])?;
        diagnostics(output.stderr());
        validate_identities(plan)?;
        verify_disk(&client, &plan.target_operation, &plan.desired_commit, &plan.root, &plan.entries)?;
        if current_commit(&client)?.1 != plan.desired_commit {
            bail!("materialization verification changed the desired WC commit");
        }
        for path in &plan.ignored {
            plan.target_rules.require_ignored(path)?;
        }
        // Link existence is checked against the actual filesystem only after ignored
        // payload restoration. Early future-state preflight already proved required
        // targets; testing live existence here would reject freshly cleared caches.
        validate_source(plan)?;
        let guard = payload::Guard::default();
        if !plan.ignored.is_empty() {
            let visitor = transfer::Auxiliary::legacy(
                &plan.ignored,
                plan.source_entries.iter().chain(&plan.entries).map(|entry| entry.path.as_path()),
                (plan.source_identity.device, plan.root_identity.device),
            )?;
            let applier =
                diff_dirs_sync::Applier::new_checked(&plan.source, &plan.root, diff_dirs_sync::CopyPolicy::ReflinkRequired, guard.borrowed())?.visitor(visitor);
            let result = payload_execution.run(&applier);
            for warning in guard.warnings() {
                if !plan.execution_diagnostics.contains(&warning) {
                    diagnostics(format!("Warning: {warning}"));
                    plan.execution_diagnostics.push(warning);
                }
            }
            result?;
        }
        validate_source(plan)?;
        let link_report = links::apply_workspace_links(default, &plan.root)?;
        if link_report.has_entries() {
            plan.execution_diagnostics.push(format!(
                "links: {} created, {} satisfied, {} skipped",
                link_report.linked, link_report.satisfied, link_report.skipped_missing_target
            ));
        }
        verify_disk(&client, &plan.target_operation, &plan.desired_commit, &plan.root, &plan.entries)?;
        if current_commit(&client)?.1 != plan.desired_commit {
            bail!("ignored/link contents changed tracked tree");
        }
        require_full(&client)?;
        validate_identities(plan)?;
        {
            let base = plan.base_commit.clone();
            let (operation, current) = current_commit(&client)?;
            if !exact_empty_child(&client, &operation, &current, &base)? {
                bail!("desired child no longer has the exact frozen parent");
            }
        }
        Ok(())
    })();
    result.with_context(|| plan.bookmark_guard.recovery(&plan.root, &plan.old_commit))
}

fn remove_recorded_file(root: &Path, relative_path: &Path) -> Result<()> {
    relative(relative_path)?;
    // A former file can now be the parent directory of desired entries. Never
    // recursively remove it: unrelated ignored children remain operator-owned.
    let mut parent = root.to_path_buf();
    for component in relative_path.parent().unwrap_or(Path::new("")).components() {
        parent.push(component);
        match fs::symlink_metadata(&parent) {
            Ok(m) if m.is_dir() => {}
            Ok(_) => return Ok(()), // old tracked ancestor file/type transition
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.into()),
        }
    }
    let path = root.join(relative_path);
    match fs::symlink_metadata(&path) {
        Ok(m) if m.is_dir() => {}
        Ok(_) => cow::remove_path(&path)?,
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verification_fixture() -> (tempfile::TempDir, JjClient) {
        let temporary = tempfile::tempdir().unwrap();
        let client = JjClient::new(temporary.path());
        client.run(["git", "init", "--no-colocate"]).unwrap();
        client.run(["config", "set", "--repo", "fsmonitor.backend", "none"]).unwrap();
        fs::write(temporary.path().join("tracked"), b"frozen\n").unwrap();
        snapshot(&client).unwrap();
        (temporary, client)
    }

    #[test]
    fn intact_verification_process_count_is_independent_of_tracked_files() {
        if let Some(log) = std::env::var_os("JW_VERIFY_PROCESS_LOG") {
            let (temporary, client) = verification_fixture();
            for count in [1, 256] {
                for index in 1..count {
                    fs::write(temporary.path().join(format!("tracked-{index}")), b"file\n").unwrap();
                }
                snapshot(&client).unwrap();
                let (operation, commit) = current_commit(&client).unwrap();
                let entries = entries(&client, &operation, &commit).unwrap();
                assert_eq!(entries.len(), count);
                fs::write(&log, "").unwrap();
                verify_disk(&client, &operation, &commit, temporary.path(), &entries).unwrap();
                let commands = fs::read_to_string(&log).unwrap();
                assert_eq!(commands.lines().count(), 2, "{commands}");
                assert!(commands.contains("--ignore-working-copy sparse list"), "{commands}");
                assert!(
                    commands.contains("--config=fsmonitor.backend=none --config=snapshot.auto-track=none() --config=snapshot.auto-update-stale=false diff"),
                    "{commands}"
                );
                assert!(!commands.contains("file show"), "{commands}");
            }
            for count in [0usize, 43, 129] {
                for index in 0..count {
                    let path = temporary.path().join(format!("link-{index:03}"));
                    if !path.is_symlink() {
                        std::os::unix::fs::symlink("tracked", path).unwrap();
                    }
                }
                snapshot(&client).unwrap();
                let (operation, commit) = current_commit(&client).unwrap();
                let entries = entries(&client, &operation, &commit).unwrap();
                fs::write(&log, "").unwrap();
                verify_symlink_support(&client, &operation, &commit, &entries).unwrap();
                let commands = fs::read_to_string(&log).unwrap();
                assert_eq!(commands.lines().count(), count.div_ceil(128), "{commands}");
            }
            return;
        }
        // Child-process environment keeps the executable wrapper isolated from
        // parallel tests. Preserve shim identity and restore the original PATH
        // when delegating so a shim's system fallback cannot recurse here.
        let original_path = std::env::var_os("PATH").unwrap();
        let real_jj = std::env::split_paths(&original_path)
            .map(|directory| directory.join("jj"))
            .find(|path| path.is_file())
            .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let wrapper = directory.path().join("jj");
        fs::write(
            &wrapper,
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$JW_VERIFY_PROCESS_LOG\"\nPATH=\"$JW_VERIFY_ORIGINAL_PATH\" exec \"$JW_VERIFY_REAL_JJ\" \"$@\"\n",
        )
        .unwrap();
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
        let path = std::env::join_paths(std::iter::once(directory.path().to_path_buf()).chain(std::env::split_paths(&original_path))).unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "reseed::engine::tests::intact_verification_process_count_is_independent_of_tracked_files",
                "--nocapture",
            ])
            .env("PATH", path)
            .env("JW_VERIFY_ORIGINAL_PATH", original_path)
            .env("JW_VERIFY_REAL_JJ", real_jj)
            .env("JW_VERIFY_PROCESS_LOG", directory.path().join("commands"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn batched_symlink_proof_preserves_unusual_path_and_payload_boundaries() {
        let (temporary, client) = verification_fixture();
        for (path, target) in [
            ("a", "ordinary"),
            ("b\n@@ -0,0 +1,999 @@\n", "one\ntwo\n"),
            ("c space", "diff --git a/fake b/fake\nnew file mode 120000"),
            ("d\n+++ b/fake", "+target\n\\ No newline at end of file\n"),
        ] {
            let path = temporary.path().join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::os::unix::fs::symlink(target, path).unwrap();
        }
        snapshot(&client).unwrap();
        let (operation, commit) = current_commit(&client).unwrap();
        let entries = entries(&client, &operation, &commit).unwrap();
        verify_symlink_support(&client, &operation, &commit, &entries).unwrap();
        let diff = b"diff --git a/a b/a\nnew file mode 120000\nindex 0000000000..1234567890\n--- /dev/null\n+++ b/a\n@@ -0,0 +1 @@\n+target\n\\ No newline at end of file\n";
        parse_added_symlink_batch(diff, &[Path::new("a")]).unwrap();
        assert!(parse_added_symlink_batch(diff, &[Path::new("b")]).is_err());
        assert!(parse_added_symlink_batch(diff, &[Path::new("a"), Path::new("b")]).is_err());
        assert!(parse_added_symlink_batch(&diff[..diff.len() - 3], &[Path::new("a")]).is_err());
        assert!(parse_added_symlink_batch(diff, &[]).is_err());
    }

    #[test]
    fn intact_verification_detects_tracked_changes_without_autotracking() {
        let (temporary, client) = verification_fixture();
        let root = temporary.path();
        std::os::unix::fs::symlink("tracked", root.join("link")).unwrap();
        snapshot(&client).unwrap();
        let (operation, commit) = current_commit(&client).unwrap();
        let entries = entries(&client, &operation, &commit).unwrap();
        fs::write(root.join("untracked"), b"leave me untracked\n").unwrap();
        verify_disk(&client, &operation, &commit, root, &entries).unwrap();
        assert_eq!(current_commit(&client).unwrap().1, commit);
        fs::write(root.join("tracked"), b"changed\n").unwrap();
        assert!(verify_disk(&client, &operation, &commit, root, &entries).is_err());
        fs::write(root.join("tracked"), b"frozen\n").unwrap();
        verify_disk(&client, &operation, &commit, root, &entries).unwrap();
        fs::set_permissions(root.join("tracked"), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(verify_disk(&client, &operation, &commit, root, &entries).is_err());
        fs::set_permissions(root.join("tracked"), fs::Permissions::from_mode(0o644)).unwrap();
        fs::remove_file(root.join("link")).unwrap();
        std::os::unix::fs::symlink("elsewhere", root.join("link")).unwrap();
        assert!(verify_disk(&client, &operation, &commit, root, &entries).is_err());
        fs::remove_file(root.join("link")).unwrap();
        std::os::unix::fs::symlink("tracked", root.join("link")).unwrap();
        verify_disk(&client, &operation, &commit, root, &entries).unwrap();
        fs::remove_file(root.join("tracked")).unwrap();
        assert!(verify_disk(&client, &operation, &commit, root, &entries).is_err());
    }

    #[test]
    fn intact_verification_refuses_sparse_and_stale_trees_before_snapshot() {
        let (temporary, client) = verification_fixture();
        let root = temporary.path();
        let (operation, commit) = current_commit(&client).unwrap();
        let entries = entries(&client, &operation, &commit).unwrap();
        client.run(["sparse", "set", "--clear"]).unwrap();
        fs::write(root.join("tracked"), b"partial staged prefill\n").unwrap();
        let before = current_commit(&client).unwrap();
        assert!(verify_disk(&client, &operation, &commit, root, &entries).is_err());
        assert_eq!(current_commit(&client).unwrap(), before);
        fs::write(root.join("tracked"), b"frozen\n").unwrap();
        client.run(["sparse", "reset"]).unwrap();
        client.run(["config", "set", "--repo", "snapshot.auto-update-stale", "true"]).unwrap();
        let other = root.join("other");
        client.run(["workspace", "add", "--name", "other", other.to_str().unwrap()]).unwrap();
        JjClient::new(&other)
            .run(["--ignore-working-copy", "restore", "--from", "root()", "--to", &commit])
            .unwrap();
        fs::write(root.join("tracked"), b"must not snapshot stale tree\n").unwrap();
        let before = current_commit(&client).unwrap();
        assert!(verify_disk(&client, &operation, &commit, root, &entries).is_err());
        assert_eq!(current_commit(&client).unwrap(), before);
        assert_eq!(fs::read(root.join("tracked")).unwrap(), b"must not snapshot stale tree\n");
    }

    #[test]
    fn escaped_diff_excludes_both_rename_endpoints() {
        let paths = parse_changed_paths("\"before name\"\n\"after\\nname\"\n").unwrap();
        assert!(paths.contains(Path::new("before name")));
        assert!(paths.contains(Path::new("after\nname")));
        assert!(parse_changed_paths("\"only-one\"\n").is_err());
        assert!(parse_changed_paths("\"../escape\"\n\"okay\"\n").is_err());
    }

    #[test]
    fn added_symlink_parser_preserves_targets_and_newline_state() {
        let header = b"diff --git a/link b/link\nnew file mode 120000\n--- /dev/null\n+++ b/link\n@@ -0,0 +1,1 @@\n";
        let mut diff = header.to_vec();
        diff.extend_from_slice(b"+../target with space\n\\ No newline at end of file\n");
        assert_eq!(parse_added_symlink(&diff).unwrap(), b"../target with space");
        let mut diff = b"diff --git a/link b/link\nnew file mode 120000\n@@ -0,0 +1,2 @@\n".to_vec();
        diff.extend_from_slice(b"+line one\n+line two\n");
        assert_eq!(parse_added_symlink(&diff).unwrap(), b"line one\nline two\n");
        assert!(parse_added_symlink(b"Warning: skipped symlink").is_err());
        let diff = b"diff --git a/newline\n@@ -0,0 +1,999 @@\n b/newline\nnew file mode 120000\n+++ b/newline\n@@ -0,0 +1,999 @@\n@@ -0,0 +1 @@\n++diff --git a/x b/y  \n\\ No newline at end of file\n";
        assert_eq!(parse_added_symlink(diff).unwrap(), b"+diff --git a/x b/y  ");
    }

    #[test]
    fn tracked_directory_type_transition_never_consumes_unrecorded_children() {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir(directory.path().join("old-directory")).unwrap();
        fs::write(directory.path().join("old-directory/tracked"), "old\n").unwrap();
        let old = [Entry {
            path: "old-directory/tracked".into(),
            kind: "file".into(),
            executable: false,
        }];
        let desired = [Entry {
            path: "old-directory".into(),
            kind: "file".into(),
            executable: false,
        }];
        preflight_obstructions(directory.path(), &old, &desired, false).unwrap();
        fs::write(directory.path().join("old-directory/secret"), "unrecorded\n").unwrap();
        assert!(preflight_obstructions(directory.path(), &old, &desired, false).is_err());
        preflight_obstructions(directory.path(), &old, &desired, true).unwrap();
    }

    #[test]
    fn tracked_symlink_parent_transition_never_inspects_external_children() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("target");
        let outside = temp.path().join("outside");
        fs::create_dir(&root).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("child"), "private external data\n").unwrap();
        std::os::unix::fs::symlink(&outside, root.join("tracked-link")).unwrap();
        let old = [Entry {
            path: "tracked-link".into(),
            kind: "symlink".into(),
            executable: false,
        }];
        let desired = [Entry {
            path: "tracked-link/child".into(),
            kind: "file".into(),
            executable: false,
        }];
        preflight_obstructions(&root, &old, &desired, false).unwrap();
        assert_eq!(fs::read(outside.join("child")).unwrap(), b"private external data\n");
    }

    #[test]
    fn required_links_are_preflighted_against_future_private_and_copied_content() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("target");
        let source = temp.path().join("source");
        let default = temp.path().join("default");
        fs::create_dir_all(root.join("private")).unwrap();
        fs::create_dir_all(source.join("cache")).unwrap();
        fs::create_dir(&default).unwrap();
        fs::write(source.join("cache/fresh"), "payload\n").unwrap();
        let rules = IgnoreRules::from_directory(tempfile::tempdir().unwrap());
        checked_git(rules.directory.path(), ["init", "--quiet"]).unwrap();
        fs::write(rules.directory.path().join(".gitignore"), "dependency-link\n").unwrap();
        let client = JjClient::new(&root);
        let entries = [Entry {
            path: "tracked/file".into(),
            kind: "file".into(),
            executable: false,
        }];
        let ignored = [PathBuf::from("cache")];
        let mut future = FutureContent {
            root: &root,
            source: &source,
            entries: &entries,
            old_paths: &[],
            ignored: &ignored,
            replace: false,
            client: &client,
            operation: "",
            commit: "",
        };
        fs::write(default.join(".jwlinks.toml"), "[[link]]\nsource='private'\ntarget='private'\nrequired=true\n").unwrap();
        preflight_links(&default, &root, &entries, &ignored, false, &rules, &future).unwrap();
        future.replace = true;
        assert!(preflight_links(&default, &root, &entries, &ignored, true, &rules, &future).is_err());
        fs::write(
            default.join(".jwlinks.toml"),
            "[[link]]\nsource='dependency-link'\ntarget='private'\nrequired=true\n",
        )
        .unwrap();
        assert!(preflight_links(&default, &root, &entries, &ignored, true, &rules, &future).is_err());
        fs::write(
            default.join(".jwlinks.toml"),
            "[[link]]\nsource='dependency-link'\ntarget='cache/fresh'\nrequired=true\n",
        )
        .unwrap();
        preflight_links(&default, &root, &entries, &ignored, true, &rules, &future).unwrap();
        future.replace = false;
        preflight_links(&default, &root, &entries, &ignored, false, &rules, &future).unwrap();
    }

    fn fixture() -> Option<(tempfile::TempDir, PreparationLock, Plan)> {
        if Command::new("jj").arg("--version").output().is_err() {
            return None;
        }
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("source");
        let target = temporary.path().join("target");
        let output = Command::new("jj").args(["git", "init", "--no-colocate"]).arg(&source).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let client = JjClient::new(&source);
        client.run(["config", "set", "--repo", "fsmonitor.backend", "none"]).unwrap();
        client
            .run(["config", "set", "--repo", "fsmonitor.watchman.register-snapshot-trigger", "false"])
            .unwrap();
        fs::write(source.join("shared"), b"shared baseline\n").unwrap();
        fs::write(source.join("different"), b"source baseline\n").unwrap();
        std::os::unix::fs::symlink("shared", source.join("link")).unwrap();
        std::os::unix::fs::symlink("shared", source.join("changed-link")).unwrap();
        fs::write(source.join(".gitignore"), b"cache/\n.env\n").unwrap();
        fs::create_dir(source.join("cache")).unwrap();
        fs::write(source.join("cache/dependency"), b"fresh dependency\n").unwrap();
        snapshot(&client).unwrap();
        client
            .run([
                "--config=git.colocate=false",
                "workspace",
                "add",
                "--name",
                "worker",
                "-r",
                "@",
                target.to_str().unwrap(),
            ])
            .unwrap();
        let target_client = JjClient::new(&target);
        fs::write(target.join("different"), b"feature change\n").unwrap();
        fs::remove_file(target.join("changed-link")).unwrap();
        std::os::unix::fs::symlink("different", target.join("changed-link")).unwrap();
        fs::write(target.join(".env"), b"retained\n").unwrap();
        snapshot(&target_client).unwrap();
        if let Err(error) = cow::probe_between(&source, &target) {
            assert!(
                std::env::var_os("JW_TEST_REQUIRE_COW").is_none_or(|s| s.is_empty()),
                "real CoW required: {error:#}"
            );
            eprintln!("skipping real reseed test: {error:#}");
            return None;
        }
        let operation = client.operation_id().unwrap();
        let source_commit = client.resolve_one_at(&operation, "@").unwrap().commit_id;
        let old = target_client.resolve_one_at(&operation, "@").unwrap().commit_id;
        // The default sync contract: base is the donor's current commit, the
        // desired tree is that base, and old_paths are the target's own
        // tracked paths that sparse clearing deletes.
        let source_entries = entries(&client, &operation, &source_commit).unwrap();
        let target_entries = entries(&target_client, &operation, &source_commit).unwrap();
        let changed = changed_paths(&client, &operation, &source_commit, &source_commit).unwrap();
        let clones = source_entries.iter().filter(|e| !changed.contains(&e.path)).map(|e| e.path.clone()).collect();
        let pending = Plan {
            name: "worker".into(),
            root: target.clone(),
            root_identity: Identity::read(&target).unwrap(),
            jj_identity: Identity::read(&target.join(".jj")).unwrap(),
            repository: repository(&client).unwrap(),
            source: source.clone(),
            source_identity: Identity::read(&source).unwrap(),
            source_jj_identity: Identity::read(&source.join(".jj")).unwrap(),
            source_operation: operation.clone(),
            source_commit: source_commit.clone(),
            source_wc_commit: source_commit.clone(),
            target_operation: operation.clone(),
            old_commit: old.clone(),
            desired_commit: old.clone(),
            base_commit: source_commit.clone(),
            clones,
            source_rules: Rc::new(IgnoreRules::new(&client, &operation, &source_commit, &source_entries).unwrap()),
            target_rules: Rc::new(IgnoreRules::new(&target_client, &operation, &source_commit, &target_entries).unwrap()),
            source_entries: Rc::new(source_entries),
            entries: target_entries,
            old_paths: entries(&target_client, &operation, &old).unwrap().into_iter().map(|e| e.path).collect(),
            ignored: vec!["cache".into()],
            skipped: Vec::new(),
            replace_content: false,
            config_root: source.clone(),
            link_config: link_config_bytes(&source).unwrap(),
            bookmark_guard: bookmarks::Guard::AllowUnbookmarked,
            execution_diagnostics: Vec::new(),
        };
        let store = PreparationLock::lock(&client).unwrap();
        Some((temporary, store, pending))
    }

    #[test]
    fn identity_recheck_refuses_replaced_repository_pointer_and_new_git_metadata() {
        let Some((_temporary, _store, plan)) = fixture() else {
            return;
        };
        validate_identities(&plan).unwrap();
        let pointer = plan.root.join(".jj/repo");
        let original = fs::read(&pointer).unwrap();
        fs::write(&pointer, plan.root.to_str().unwrap()).unwrap();
        let error = validate_identities(&plan).unwrap_err();
        assert!(error.to_string().contains("repository identity changed"));
        fs::write(pointer, original).unwrap();
        validate_identities(&plan).unwrap();

        fs::write(plan.root.join(".git"), "gitdir: operator-owned\n").unwrap();
        let error = validate_identities(&plan).unwrap_err();
        assert!(error.to_string().contains("target acquired Git metadata"));
        assert_eq!(fs::read(plan.root.join("different")).unwrap(), b"feature change\n");
    }

    #[test]
    fn ignore_rules_use_desired_nested_files_and_backend_excludes() {
        let Some((_temporary, _store, plan)) = fixture() else {
            return;
        };
        let client = JjClient::new(&plan.source);
        fs::create_dir_all(plan.source.join("packages/p")).unwrap();
        fs::write(plan.source.join("packages/p/.gitignore"), "node_modules/\n").unwrap();
        let backend = PathBuf::from(client.run(["--ignore-working-copy", "git", "root"]).unwrap().trimmed_stdout().unwrap());
        fs::create_dir_all(backend.join("info")).unwrap();
        fs::write(backend.join("info/exclude"), "backend-cache/\n").unwrap();
        snapshot(&client).unwrap();
        let (operation, commit) = current_commit(&client).unwrap();
        let entries = entries(&client, &operation, &commit).unwrap();
        let rules = IgnoreRules::new(&client, &operation, &commit, &entries).unwrap();
        rules.require_ignored(Path::new("packages/p/node_modules")).unwrap();
        rules.require_ignored(Path::new("backend-cache")).unwrap();
        assert!(rules.require_ignored(Path::new("unignored")).is_err());
        assert!(rules.require_ignored_file(Path::new("cache")).is_err());
    }

    #[test]
    fn adversarial_git_template_cannot_escape_ignore_probe() {
        if std::env::var_os("JW_RESEED_TEMPLATE_CHILD").is_some() {
            let Some((_temporary, _store, plan)) = fixture() else {
                return;
            };
            let client = JjClient::new(&plan.source);
            let entries = entries(&client, &plan.source_operation, &plan.source_commit).unwrap();
            let rules = IgnoreRules::new(&client, &plan.source_operation, &plan.source_commit, &entries).unwrap();
            rules.require_ignored(Path::new("cache")).unwrap();
            assert_eq!(
                fs::read(std::env::var_os("JW_RESEED_TEMPLATE_MARKER").unwrap()).unwrap(),
                b"outside marker must survive\n"
            );
            return;
        }
        // Process-scoped environment avoids unsafe global env mutation or races
        // with another unit test. Git's inherited template contains a symlink
        // exactly at the file the ignore adapter must populate privately.
        let temp = tempfile::tempdir().unwrap();
        let marker = temp.path().join("outside-marker");
        let template = temp.path().join("template");
        fs::write(&marker, b"outside marker must survive\n").unwrap();
        fs::create_dir_all(template.join("info")).unwrap();
        std::os::unix::fs::symlink(&marker, template.join("info/exclude")).unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "reseed::engine::tests::adversarial_git_template_cannot_escape_ignore_probe",
                "--nocapture",
            ])
            .env("JW_RESEED_TEMPLATE_CHILD", "1")
            .env("JW_RESEED_TEMPLATE_MARKER", &marker)
            .env("GIT_TEMPLATE_DIR", &template)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fs::read(marker).unwrap(), b"outside marker must survive\n");
    }

    #[test]
    fn fresh_execute_refuses_new_unsnapshotted_target_edits() {
        let Some((_temporary, _store, mut plan)) = fixture() else {
            return;
        };
        let default = plan.source.clone();
        fs::write(plan.root.join("shared"), "new user work\n").unwrap();
        let error = execute(&default, &mut plan, &transfer::Execution::new(true).unwrap()).unwrap_err();
        assert!(format!("{error:#}").contains("tracked tree differs"));
        assert_eq!(fs::read(plan.root.join("shared")).unwrap(), b"new user work\n");
    }

    #[test]
    fn native_unmanaged_workspaces_initialize_only_repository_manifest() {
        let Some((_temporary, _store, mut plan)) = fixture() else {
            return;
        };
        let client = JjClient::new(&plan.source);
        let metadata = WorkspaceMetadataStore::from_repo_config_path(client.repo_config_path().unwrap()).unwrap();
        assert!(metadata.list().unwrap().is_empty());
        assert!(metadata.get("worker").unwrap().is_none());
        validate_target(&client, &metadata, &plan.source, &plan.source, "worker", &plan.root).unwrap();
        let default = plan.source.clone();
        execute(&default, &mut plan, &transfer::Execution::new(true).unwrap()).unwrap();
        assert!(metadata.get("worker").unwrap().is_none());
        assert!(metadata.list().unwrap().is_empty());
        assert!(metadata.root().join("manifest.json").is_file());
        assert!(fs::read_dir(metadata.root().join("workspaces")).unwrap().next().is_none());
    }

    #[test]
    fn native_reset_type_transitions_preserve_unrelated_and_external_secrets() {
        let Some((temporary, _store, mut plan)) = fixture() else {
            return;
        };
        let source = JjClient::new(&plan.source);
        let target = JjClient::new(&plan.root);
        let outside = temporary.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("child"), b"private external data\n").unwrap();
        fs::create_dir(plan.root.join("directory-to-file")).unwrap();
        fs::write(plan.root.join("directory-to-file/old-child"), b"old tracked data\n").unwrap();
        std::os::unix::fs::symlink(&outside, plan.root.join("symlink-to-directory")).unwrap();
        snapshot(&target).unwrap();
        source.run(["new"]).unwrap();
        fs::write(plan.source.join("directory-to-file"), b"new file\n").unwrap();
        fs::create_dir(plan.source.join("symlink-to-directory")).unwrap();
        fs::write(plan.source.join("symlink-to-directory/child"), b"new child\n").unwrap();
        snapshot(&source).unwrap();
        let operation = source.operation_id().unwrap();
        let source_commit = source.resolve_one_at(&operation, "@").unwrap().commit_id;
        let old_commit = target.resolve_one_at(&operation, "@").unwrap().commit_id;
        let old_entries = entries(&target, &operation, &old_commit).unwrap();
        let desired_entries = entries(&source, &operation, &source_commit).unwrap();
        preflight_obstructions(&plan.root, &old_entries, &desired_entries, false).unwrap();
        plan.source_operation = operation.clone();
        plan.target_operation = operation;
        plan.source_commit = source_commit.clone();
        plan.source_wc_commit = source_commit.clone();
        plan.base_commit = source_commit;
        plan.old_commit = old_commit.clone();
        plan.desired_commit = old_commit;
        plan.clones = desired_entries.iter().map(|e| e.path.clone()).collect();
        plan.entries = desired_entries;
        plan.old_paths = old_entries.into_iter().map(|e| e.path).collect();
        let default = plan.source.clone();
        execute(&default, &mut plan, &transfer::Execution::new(true).unwrap()).unwrap();
        assert_eq!(fs::read(plan.root.join("directory-to-file")).unwrap(), b"new file\n");
        assert_eq!(fs::read(plan.root.join("symlink-to-directory/child")).unwrap(), b"new child\n");
        assert_eq!(fs::read(plan.root.join(".env")).unwrap(), b"retained\n");
        assert_eq!(fs::read(outside.join("child")).unwrap(), b"private external data\n");
        plan.root_identity.verify(&plan.root).unwrap();
        plan.jj_identity.verify(&plan.root.join(".jj")).unwrap();
    }

    #[test]
    fn reseed_store_never_repairs_corrupt_or_nonempty_manifestless_metadata() {
        let Some((_temporary, _store, plan)) = fixture() else {
            return;
        };
        let client = JjClient::new(&plan.source);
        let metadata = WorkspaceMetadataStore::from_repo_config_path(client.repo_config_path().unwrap()).unwrap();
        let manifest = metadata.root().join("manifest.json");
        fs::write(&manifest, b"not valid json\n").unwrap();
        assert!(PreparationLock::lock(&client).is_err());
        assert_eq!(fs::read(&manifest).unwrap(), b"not valid json\n");
        fs::remove_file(&manifest).unwrap();
        assert!(PreparationLock::lock(&client).is_err());
        assert!(!manifest.exists());
    }

    #[test]
    fn live_source_validation_reuses_command_frozen_backend_ignored_rules() {
        let Some((_temporary, _store, mut plan)) = fixture() else {
            return;
        };
        let source = JjClient::new(&plan.source);
        source.run(["new"]).unwrap();
        let selected = PathBuf::from(format!("jw-private-cache-{}", std::process::id()));
        fs::rename(plan.source.join("cache"), plan.source.join(&selected)).unwrap();
        fs::write(plan.source.join(".gitignore"), ".env\n").unwrap();
        let backend = PathBuf::from(source.run(["--ignore-working-copy", "git", "root"]).unwrap().trimmed_stdout().unwrap());
        fs::create_dir_all(backend.join("info")).unwrap();
        let exclude = backend.join("info/exclude");
        fs::write(&exclude, format!("{}/\n", selected.display())).unwrap();
        snapshot(&source).unwrap();
        let (operation, commit) = current_commit(&source).unwrap();
        let source_entries = entries(&source, &operation, &commit).unwrap();
        verify_disk(&source, &operation, &commit, &plan.source, &source_entries).unwrap();
        plan.source_rules = Rc::new(IgnoreRules::new(&source, &operation, &commit, &source_entries).unwrap());
        plan.source_entries = Rc::new(source_entries);
        plan.source_operation = operation;
        plan.source_commit = commit.clone();
        plan.source_wc_commit = commit.clone();
        plan.ignored = vec![selected];
        validate_source(&plan).unwrap();
        fs::write(exclude, "").unwrap();
        validate_source(&plan).unwrap();
        let next_rules = IgnoreRules::new(&source, &plan.source_operation, &commit, &plan.source_entries).unwrap();
        assert!(next_rules.require_ignored(&plan.ignored[0]).is_err());
    }

    #[test]
    fn frozen_rule_cache_keys_effective_excludes_and_reuses_native_context() {
        let (_temporary, client) = verification_fixture();
        let (operation, commit) = current_commit(&client).unwrap();
        let entries = entries(&client, &operation, &commit).unwrap();
        let mut cache = FrozenRules::default();
        let first = cache.get(&client, &operation, &commit, &entries).unwrap();
        let repeated = cache.get(&client, &operation, &commit, &entries).unwrap();
        assert!(Rc::ptr_eq(&first, &repeated));
        client.run(["new"]).unwrap();
        let (next_operation, equal_commit) = current_commit(&client).unwrap();
        assert!(changed_paths(&client, &next_operation, &commit, &equal_commit).unwrap().is_empty());
        cache.same_tree(&equal_commit, &commit);
        let equal = cache.get(&client, &next_operation, &equal_commit, &entries).unwrap();
        assert!(Rc::ptr_eq(&first, &equal));
        let private = PathBuf::from(format!("jw-frozen-rule-cache-{}", std::process::id()));
        assert!(!first.is_ignored_directory(&private).unwrap());
        let backend = PathBuf::from(client.run(["--ignore-working-copy", "git", "root"]).unwrap().trimmed_stdout().unwrap());
        fs::create_dir_all(backend.join("info")).unwrap();
        fs::write(backend.join("info/exclude"), format!("{}/\n", private.display())).unwrap();
        let changed = cache.get(&client, &operation, &commit, &entries).unwrap();
        assert!(!Rc::ptr_eq(&first, &changed));
        assert!(changed.is_ignored_directory(&private).unwrap());
        assert!(!first.is_ignored_directory(&private).unwrap());
    }
}
