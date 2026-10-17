use super::selection::{self, Group, Registrations};
use super::{RecordedRequest, ReseedOutcome, ReseedReport, ReseedStatus};
use crate::cow::ignored::{Origin, Patterns, Policy};
#[cfg(test)]
use crate::cow::payload::checked_git;
use crate::cow::payload::{
    Entry, IgnoreRules, Stamp, confined_parents, directory_identity, entries, file_bytes,
    payload_digest, relative, stamp,
};
use crate::freshen::history::{BookmarkNames, TargetHistory};
use crate::freshen::source::SourceSelection;
use crate::{cow, jj::JjClient, links, metadata::WorkspaceMetadataStore};
use anyhow::{Context, Result, bail};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, ErrorKind, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

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
    /// Deprecated alias retained so frozen v1 preparation policy can be resumed.
    prepare: Option<prepare::Settings>,
    #[serde(skip)]
    legacy_prepare: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

/// Write-ahead phases, not claims that a non-transactional external step completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Phase {
    Prepared,
    Clearing,
    Empty,
    Newing,
    Filling,
    Expanding,
    Verified,
    CopyingIgnored,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pending {
    schema_version: u32,
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
    base_commit: Option<String>,
    sparse: Vec<String>,
    clones: Vec<PathBuf>,
    entries: Vec<Entry>,
    old_paths: Vec<PathBuf>,
    ignored: Vec<PathBuf>,
    skipped: Vec<String>,
    replace_content: bool,
    phase: Phase,
    source_stamp: Vec<Stamp>,
    source_payload_digest: String,
    validated_source_wc_commit: String,
    target_stamp: Vec<Stamp>,
    config_root: PathBuf,
    link_config: Vec<(String, Option<Vec<u8>>)>,
    #[serde(default)]
    preparation_id: Option<String>,
    /// Up's immutable name/exact-tip intent, never recomputed
    /// during recovery. Absent in legacy and ordinary reseed plans.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    history: Option<TargetHistory>,
    /// Runtime observations are not part of immutable target intent.
    #[serde(default)]
    execution_diagnostics: Vec<String>,
}

struct Store {
    root: PathBuf,
    _lock: File,
}
impl Store {
    fn lock(client: &JjClient) -> Result<Self> {
        let config = client.repo_config_path()?;
        WorkspaceMetadataStore::from_repo_config_path(&config)?.ensure_initialized()?;
        let parent = config
            .parent()
            .context("repository config has no parent")?
            .canonicalize()?;
        let root = parent.join("jj-waltz/reseed");
        fs::create_dir_all(&root)?;
        if root.canonicalize()? != root {
            bail!("reseed storage contains a symlink");
        }
        let lock_path = root.join("operation.lock");
        if fs::symlink_metadata(&lock_path).is_ok_and(|m| !m.is_file()) {
            bail!("invalid reseed lock");
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)?;
        lock.try_lock_exclusive()
            .context("another jw reseed/prepare is running; pause other writers before retrying")?;
        Ok(Self { root, _lock: lock })
    }
    fn path(&self, name: &str, suffix: &str) -> PathBuf {
        // Bound path length even for long literal names. The embedded exact name
        // is validated on every read, so a digest collision never grants identity.
        let encoded = Sha256::digest(name.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        self.root.join(format!("workspace-{encoded}.{suffix}.json"))
    }
    fn pending(&self, name: &str) -> Result<Option<Pending>> {
        let path = self.path(name, "pending");
        match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
            Ok(m) if !m.is_file() => bail!("pending record is not a regular file"),
            Ok(_) => {}
        }
        let record: Pending = serde_json::from_slice(&fs::read(path)?)?;
        if record.schema_version != 1 || record.name != name {
            bail!("invalid reseed pending identity/version");
        }
        Ok(Some(record))
    }
    fn save(&self, pending: &Pending) -> Result<()> {
        atomic_json(&self.path(&pending.name, "pending"), pending)
    }
    fn reserve_bookmarks(&self, names: &mut BookmarkNames) -> Result<()> {
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry
                .file_name()
                .to_string_lossy()
                .ends_with(".pending.json")
            {
                continue;
            }
            if !entry.file_type()?.is_file() {
                bail!("pending record is not a regular file");
            }
            let plan: Pending = serde_json::from_slice(&fs::read(entry.path())?)?;
            if plan.schema_version != 1 || self.path(&plan.name, "pending") != entry.path() {
                bail!("invalid pending identity/version");
            }
            reserve_history(&plan, names);
        }
        prepare::reserve_bookmarks(self, names)
    }
    fn finish(&self, pending: &Pending) -> Result<()> {
        atomic_json(&self.path(&pending.name, "receipt"), pending)?;
        fs::remove_file(self.path(&pending.name, "pending"))?;
        File::open(&self.root)?.sync_all()?;
        Ok(())
    }
}

fn reserve_history(plan: &Pending, names: &mut BookmarkNames) {
    if let Some(bookmark) = plan.history.as_ref().and_then(|h| h.bookmark.as_ref()) {
        names.reserve(&bookmark.name);
    }
}

fn atomic_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let parent = path.parent().context("record has no parent")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    {
        // Tree stamps make journals large. Serde writes individual tokens, so
        // writing directly to the file turns every field into several syscalls
        // at every write-ahead boundary. Flush the buffer before the same durable
        // file-sync/rename/directory-sync sequence; no boundary is weakened.
        let mut writer = BufWriter::new(&mut temporary);
        serde_json::to_writer_pretty(&mut writer, value)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
    }
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|e| e.error)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

pub(super) fn run(
    request: RecordedRequest,
    diagnostic: &mut dyn FnMut(&str),
) -> Result<ReseedReport> {
    let aggregate_preview = request.dry_run && !request.prepare && !request.resume;
    let mut readiness = Readiness::default();
    match run_inner(request, diagnostic, &mut readiness) {
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
        self.failures
            .entry(name.to_owned())
            .or_insert_with(|| ReseedOutcome {
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

fn run_inner(
    mut request: RecordedRequest,
    diagnostic: &mut dyn FnMut(&str),
    readiness: &mut Readiness,
) -> Result<ReseedReport> {
    if request.resume && (!request.groups.is_empty() || request.on_unmatched.is_some()) {
        bail!(
            "--resume requires literal names and does not accept group selection or --on-unmatched"
        );
    }
    if std::env::var_os("GIT_CONFIG_PARAMETERS").is_some_and(|value| !value.is_empty())
        || std::env::var_os("GIT_CONFIG_COUNT")
            .is_some_and(|value| !value.is_empty() && value != "0")
    {
        bail!(
            "unset active GIT_CONFIG_COUNT/GIT_CONFIG_PARAMETERS before reseed; JJ and the isolated ignore verifier may interpret those overrides differently"
        );
    }
    let caller = JjClient::current()?;
    let caller_cwd = caller.cwd().to_path_buf();
    // Frozen file paths are repository-relative. Root all subsequent fileset
    // queries too, even when jw was invoked from a workspace subdirectory.
    let caller_root = PathBuf::from(
        caller
            .run(["--ignore-working-copy", "workspace", "root"])?
            .trimmed_stdout()?,
    )
    .canonicalize()?;
    let client = JjClient::new(caller_root);
    // Existing convenience inventory performs normal JJ commands. Do not use it
    // here: merely discovering a pending current workspace must not snapshot it.
    let inventory = Inventory::load(&client)?;
    let default = inventory.default.clone();
    let implicit_current = request.names.is_empty() && request.groups.is_empty();
    let source_only = request.prepare
        || (implicit_current && request.resume && inventory.root(&inventory.current)? == default);
    if implicit_current {
        if inventory.root(&inventory.current)? == default && !source_only {
            bail!(
                "reseed at the primary checkout requires literal target names or --group; never selects all implicitly"
            );
        }
        // Resolve to one registered literal identity before source/target refresh
        // or pending lookup. Do not introduce navigation-token interpretation.
        if !source_only {
            request.names.push(inventory.current.clone());
        }
    }
    let (mut config, mut preparation_config) = if request.resume && request.groups.is_empty() {
        (LocalConfig::default(), None)
    } else {
        load_config(&default)?
    };
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
        )?
    };
    if let Some(warning) = selection.warning() {
        diagnostic(&warning);
    }
    if request.dry_run && !source_only && !request.resume {
        let raw_source = request
            .source
            .as_ref()
            .or(config.source.as_ref())
            .unwrap_or(&default);
        let source = if raw_source.is_absolute() {
            raw_source.clone()
        } else if request.source.is_some() {
            caller_cwd.join(raw_source)
        } else {
            default.join(raw_source)
        };
        readiness.preview = Some(ReseedReport {
            dry_run: true,
            source,
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
    let names = selection
        .selections
        .iter()
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    let store = Store::lock(&client)?;
    let metadata = WorkspaceMetadataStore::from_repo_config_path(client.repo_config_path()?)?;
    let repository_path = repository(&client)?;
    let mut plans = Vec::new();
    let mut completed = Vec::new();
    let mut batch = if request.resume {
        prepare::find_resume(&store, &names, &default, request.prepare)?
    } else {
        None
    };
    if batch.is_none() && request.resume && source_only {
        bail!(
            "no pending source-only preparation for this command; legacy batches require `jw reseed --resume` with their recorded target names"
        );
    }
    let recovering_preparation = batch.as_ref().is_some_and(|b| b.plans.is_none());
    if recovering_preparation {
        let record = batch.as_ref().unwrap();
        prepare::validate_resume_request(&request)?;
        // Source preparation is still shared batch work. Once target plans
        // freeze, each recorded target/receipt can instead resume independently.
        if names != record.request.names {
            bail!(
                "preparation resume requires all recorded literal target names in original order: {:?}",
                record.request.names
            );
        }
        let dry_run = request.dry_run;
        request = record.request.clone();
        request.dry_run = dry_run;
        (config, preparation_config) = load_config(&default)?;
    }
    if config.legacy_prepare {
        diagnostic(
            "warning: [prepare] is a deprecated local alias for [freshen]; rename it after any pending preparation finishes (frozen policy bytes must not change during recovery)",
        );
    }
    if let Some(frozen) = batch.as_ref().and_then(|b| b.plans.as_ref()) {
        prepare::validate_resume_request(&request)?;
        for name in &names {
            let original = frozen
                .iter()
                .find(|p| &p.name == name)
                .context("target not in frozen preparation batch")?;
            match prepare::handoff(&store, original, request.dry_run)? {
                Some(plan) => plans.push(plan),
                None => completed.push(ReseedOutcome {
                    name: name.clone(),
                    status: ReseedStatus::Succeeded,
                    message: "already completed recorded preparation batch".into(),
                }),
            }
        }
        for plan in &plans {
            if inventory.root(&plan.name)?.canonicalize()? != plan.root
                || plan.repository != repository_path
            {
                bail!(
                    "pending workspace/repository identity changed for {}",
                    plan.name
                );
            }
            validate_target(
                &client,
                &metadata,
                &default,
                &plan.source,
                &plan.name,
                &plan.root,
            )?;
            validate_identities(plan)?;
        }
        if source_only {
            prepare::verify_verified(batch.as_mut().unwrap())?;
        }
    } else if request.resume {
        if request.source.is_some()
            || request.source_at.is_some()
            || request.at.is_some()
            || request.copy_ignored.is_some()
            || request.no_copy_ignored
            || request.replace_content
            || request.no_freshen_bookmark
        {
            bail!(
                "--resume uses the frozen pending plan; source/revision/content overrides are not allowed"
            );
        }
        for name in &names {
            let mut pending = store
                .pending(name)?
                .with_context(|| format!("no pending reseed for {name}"))?;
            if inventory.root(name)?.canonicalize()? != pending.root
                || pending.repository != repository_path
            {
                bail!("pending workspace/repository identity changed for {name}");
            }
            validate_target(
                &client,
                &metadata,
                &default,
                &pending.source,
                name,
                &pending.root,
            )?;
            validate_identities(&pending)?;
            ensure_quiescence(&JjClient::new(&pending.source))?;
            ensure_quiescence(&JjClient::new(&pending.root))?;
            validate_source(&mut pending, true)?;
            validate_contents(&pending.root, false)?;
            cow::probe_between(&pending.source, &pending.root)?;
            plans.push(pending);
        }
    } else {
        // Pending detection precedes every source/target snapshot, even when called inside a target.
        for name in &names {
            match store.pending(name) {
                Ok(Some(_)) => readiness.block(name, anyhow::anyhow!(
                    "workspace {name} has a pending reseed/refill; use `jw reseed --resume {name}` before snapshotting it"
                )),
                Ok(None) => {}
                Err(error) => readiness.block(name, error),
            }
        }
        if !request.dry_run {
            readiness.require_unblocked()?;
        }
        let raw_source = request
            .source
            .as_ref()
            .or(config.source.as_ref())
            .unwrap_or(&default);
        let source = if raw_source.is_absolute() {
            raw_source.clone()
        } else if request.source.is_some() {
            caller_cwd.join(raw_source)
        } else {
            default.join(raw_source)
        }
        .canonicalize()?;
        let source_client = JjClient::new(&source);
        prepare::ensure_available(&store, &source, batch.as_ref())?;
        let root = source_client.run(["--ignore-working-copy", "workspace", "root"])?;
        if PathBuf::from(root.trimmed_stdout()?).canonicalize()? != source {
            bail!("source must name a JJ workspace root");
        }
        if repository(&source_client)? != repository_path {
            bail!("source must belong to the same JJ repository as the targets");
        }
        checked_context(&source_client)?;
        for (name, root) in &inventory.roots {
            if root == &source && store.pending(name)?.is_some() {
                bail!("source has a pending reseed; resume it before using it as a source");
            }
        }
        let source_rev = request
            .source_at
            .as_ref()
            .or(config.source_at.as_ref())
            .map(String::as_str);
        if request.freshen && source_rev.is_none() {
            bail!("reseed --freshen requires --source-at or source_at in .jwreseed.local.toml");
        }
        let ignored_policy = if let Some(batch) = &batch {
            Policy::recorded(
                batch
                    .request
                    .copy_ignored
                    .as_deref()
                    .context("preparation lacks frozen ignored-copy selection")?,
            )
        } else {
            Policy::load(
                request.copy_ignored.as_deref(),
                config.copy_ignored.as_deref(),
                request.no_copy_ignored,
            )?
        };
        if !batch
            .as_ref()
            .is_some_and(|batch| batch.phase == prepare::PreparationPhase::Verified)
        {
            Patterns::compile(&ignored_policy.patterns)?;
        }
        if ignored_policy.origin != Origin::None {
            diagnostic(&ignored_policy.describe());
        }
        ensure_quiescence(&source_client)?;
        require_full(&source_client)?;
        validate_contents(&source, true)?;
        let preparation_blockers = prepare::source_blockers(&store)?;
        let mut roots = BTreeSet::new();
        for name in &names {
            if readiness.failures.contains_key(name) {
                continue;
            }
            let result = (|| {
                let root = inventory.root(name)?.canonicalize()?;
                if let Some(reason) = preparation_blockers.get(&root) {
                    bail!("target has unfinished preparation: {reason}");
                }
                if roots.contains(&root) {
                    bail!("multiple selected workspaces alias {}", root.display());
                }
                validate_target(&client, &metadata, &default, &source, name, &root)?;
                ensure_quiescence(&JjClient::new(&root))?;
                require_full(&JjClient::new(&root))?;
                validate_contents(&root, false)?;
                if request.up || request.freshen {
                    cow::probe_between(&source, &root)?;
                }
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
        if request.prepare || request.up || request.freshen {
            let ignored_patterns = ignored_policy.patterns.as_slice();
            prepare::verify_policy(&default, &preparation_config)?;
            prepare::preflight(
                &store,
                &metadata,
                &inventory,
                &source,
                config.freshen.as_ref(),
            )?;
            if batch.is_none() {
                snapshot(&source_client)?;
                for root in &roots {
                    snapshot(&JjClient::new(root))?;
                }
                let (source_operation, source_commit) = current_commit(&source_client)?;
                let source_before = stamp(&source)?;
                let source_entries = entries(&source_client, &source_operation, &source_commit)?;
                verify_disk(
                    &source_client,
                    &source_operation,
                    &source_commit,
                    &source,
                    &source_entries,
                )?;
                let selected = SourceSelection::select(
                    &source_client,
                    &source_operation,
                    source_rev,
                    config.freshen.as_ref().and_then(|s| s.remote.as_deref()),
                )?;
                if stamp(&source)? != source_before
                    || current_commit(&source_client)?.1 != source_commit
                {
                    bail!(
                        "source changed during upstream selection; pause writers before retrying"
                    );
                }
                if request.dry_run {
                    let mut summary = prepare::describe(
                        &source,
                        &selected,
                        config.freshen.as_ref(),
                        ignored_patterns,
                    )?;
                    if source_only {
                        summary.push_str("; source-only prepare, no targets or target refill");
                    }
                    diagnostic(&summary);
                    return Ok(ReseedReport {
                        dry_run: true,
                        source,
                        source_commit_id: None,
                        preparation_summary: Some(summary),
                        targets: names.iter().map(|name| ReseedOutcome {
                            name: name.clone(),
                            status: ReseedStatus::Planned,
                            message: format!(
                                "new empty change on exact fetched goal (not a rebase); {}; ignored selection {:?}; no fetch/new/preparation/bookmark/refill performed; real work snapshotted",
                                if request.no_freshen_bookmark {
                                    "automatic freshen bookmark disabled; history retained"
                                } else {
                                    "freshen timestamp bookmark if snapshotted work is not upstream; exact ancestry checked after fetch"
                                },
                                ignored_patterns,
                            ),
                        }).collect(),
                        selections: selection.selections,
                        skipped: selection.skipped,
                    });
                }
                batch = Some(prepare::begin(
                    &store,
                    &client,
                    &source,
                    &default,
                    &request,
                    config.freshen.as_ref(),
                    ignored_patterns,
                    preparation_config.clone(),
                    selected,
                )?);
            }
            if request.dry_run {
                diagnostic(
                    "Preparation resume dry-run: stored goal/policy; no fetch/new/preparation/refill performed",
                );
                let b = batch.as_ref().unwrap();
                return Ok(ReseedReport {
                    dry_run: true,
                    source,
                    source_commit_id: b.goal.clone(),
                    preparation_summary: Some(format!(
                        "Resume recorded preparation phase {:?}, source goal {:?}; {}; no preparation actions performed",
                        b.phase,
                        b.source_rev,
                        prepare::summary(b)
                    )),
                    targets: names
                        .iter()
                        .map(|name| ReseedOutcome {
                            name: name.clone(),
                            status: ReseedStatus::Planned,
                            message: format!("resume recorded source phase {:?}", b.phase),
                        })
                        .collect(),
                    selections: selection.selections,
                    skipped: selection.skipped,
                });
            }
            prepare::execute(&store, batch.as_mut().unwrap(), diagnostic)?;
        }
        // A new prepare action ends at its source proof. It never enters target
        // planning, snapshots the invoking worker, or allocates history bookmarks.
        if request.prepare {
            let prepared = batch.as_mut().context("prepare lacks source journal")?;
            prepared.plans = Some(Vec::new());
            prepare::save(&store, prepared)?;
            let report = ReseedReport {
                dry_run: false,
                source,
                source_commit_id: prepared.goal.clone(),
                preparation_summary: Some(prepare::summary(prepared)),
                targets: Vec::new(),
                selections: Vec::new(),
                skipped: Vec::new(),
            };
            prepare::finish_if_complete(&store, prepared)?;
            return Ok(report);
        }
        // Assert the donor's prepared tree before snapshotting any target.
        snapshot(&source_client)?;
        let source_operation = client.operation_id()?;
        let source_wc = source_client
            .resolve_one_at(&source_operation, "@")?
            .commit_id;
        let source_commit = match batch.as_ref() {
            Some(b) => b
                .goal
                .clone()
                .context("prepared source lacks frozen goal")?,
            None => match source_rev {
                Some(rev) => {
                    source_client
                        .resolve_one_at(&source_operation, rev)?
                        .commit_id
                }
                None => source_wc.clone(),
            },
        };
        require_unconflicted(&source_client, &source_operation, &source_commit)?;
        if !changed_paths(
            &source_client,
            &source_operation,
            &source_wc,
            &source_commit,
        )?
        .is_empty()
        {
            bail!(
                "source tracked tree does not match source_at; prepare the baseline before reseeding"
            );
        }
        let source_entries = entries(&source_client, &source_operation, &source_commit)?;
        let source_stamp = stamp(&source)?;
        verify_disk(
            &source_client,
            &source_operation,
            &source_commit,
            &source,
            &source_entries,
        )?;
        if let Some(report) = readiness.preview.as_mut() {
            report.source = source.clone();
            report.source_commit_id = Some(source_commit.clone());
        }
        // Full scans freeze dirty target work only after donor validation.
        if let Some(b) = batch.as_ref() {
            prepare::verify_targets(b)?;
        } else {
            for name in &names {
                if readiness.failures.contains_key(name) {
                    continue;
                }
                let result = (|| {
                    let target = JjClient::new(inventory.root(name)?);
                    let before = request
                        .dry_run
                        .then(|| current_commit(&target))
                        .transpose()?;
                    snapshot(&target)?;
                    if let Some((_, before)) = before {
                        readiness
                            .snapshotted
                            .insert(name.clone(), current_commit(&target)?.1 != before);
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    readiness.block(name, error);
                }
            }
        }
        if !request.dry_run {
            readiness.require_unblocked()?;
        }
        let operation = client.operation_id()?;
        if source_client.resolve_one_at(&operation, "@")?.commit_id != source_wc
            || stamp(&source)? != source_stamp
        {
            bail!("source changed during target refresh; pause writers before retrying");
        }
        // Keep the already asserted source commit pinned, but freeze all journal
        // facts at the final operation shared with the refreshed target trees.
        let source_operation = operation.clone();
        let base = if request.up {
            Some(source_commit.clone())
        } else {
            request
                .at
                .as_ref()
                .map(|rev| client.resolve_one_at(&operation, rev).map(|r| r.commit_id))
                .transpose()?
        };
        let mut bookmark_names = if request.up {
            let mut allocator = BookmarkNames::at(&client, &operation, jiff::Timestamp::now())?;
            store.reserve_bookmarks(&mut allocator)?;
            Some(allocator)
        } else {
            None
        };
        let source_rules = IgnoreRules::new(
            &source_client,
            &source_operation,
            &source_commit,
            &source_entries,
            true,
        )?;
        let (ignored, skipped) = if let Some(batch) = &batch {
            let (roots, diagnostics) = prepare::verified_selection(batch)?;
            validate_selected_ignored(&source, roots, &source_rules, &source_entries)?;
            (roots.to_vec(), diagnostics.to_vec())
        } else {
            select_ignored(
                &source,
                &ignored_policy.patterns,
                &source_rules,
                &source_entries,
            )?
        };
        let source_payload_digest = payload_digest(&source, &ignored)?;
        for path in &ignored {
            cow::validate_tree(&source.join(path))?;
        }
        if stamp(&source)? != source_stamp || current_commit(&source_client)?.1 != source_wc {
            bail!("source changed during initial verified capture; pause writers before retrying");
        }
        let facts = client.workspace_target_facts_at(&operation)?;
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
                let tree_commit = base.as_ref().unwrap_or(&fact.commit_id);
                require_unconflicted(&client, &operation, tree_commit)?;
                let target_entries = entries(&client, &operation, tree_commit)?;
                let target_rules = IgnoreRules::new(
                    &JjClient::new(&root),
                    &operation,
                    tree_commit,
                    &target_entries,
                    !request.replace_content,
                )?;
                for path in &ignored {
                    target_rules.require_ignored(path)?;
                    if target_entries.iter().any(|e| overlap(path, &e.path)) {
                        bail!("ignored root overlaps tracked files: {}", path.display());
                    }
                    confined_parents(&root, path)?;
                }
                let changed = changed_paths(&client, &operation, &source_commit, tree_commit)?;
                let target_paths = target_entries
                    .iter()
                    .map(|e| &e.path)
                    .collect::<BTreeSet<_>>();
                let clones = source_entries
                    .iter()
                    .filter(|e| target_paths.contains(&e.path) && !changed.contains(&e.path))
                    .map(|e| e.path.clone())
                    .collect::<Vec<_>>();
                let old_entries = entries(&client, &operation, &fact.commit_id)?;
                verify_disk(
                    &JjClient::new(&root),
                    &operation,
                    &fact.commit_id,
                    &root,
                    &old_entries,
                )?;
                // Symlink extraction is a real CLI capability, not inferred from
                // file-list typing. Prove both current and future trees before any
                // target's sparse checkout or pending state is changed.
                if !symlink_commits.contains(tree_commit) {
                    verify_symlink_support(&client, &operation, tree_commit, &target_entries)?;
                    symlink_commits.insert(tree_commit);
                }
                preflight_obstructions(
                    &root,
                    &old_entries,
                    &target_entries,
                    request.replace_content,
                )?;
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
                        old_paths: &old_entries
                            .iter()
                            .map(|e| e.path.clone())
                            .collect::<Vec<_>>(),
                        ignored: &ignored,
                        replace: request.replace_content,
                        client: &client,
                        operation: &operation,
                        commit: tree_commit,
                    },
                )?;
                cow::probe_between(&source, &root)?;
                let mut warnings = skipped.clone();
                warnings.extend(absolute_links(&source, &ignored)?);
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
                Ok(Pending {
                    schema_version: 1,
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
                    sparse: vec![".".into()],
                    clones,
                    entries: target_entries,
                    old_paths: old_entries.into_iter().map(|e| e.path).collect(),
                    ignored: ignored.clone(),
                    skipped: warnings,
                    replace_content: request.replace_content,
                    phase: Phase::Prepared,
                    source_stamp: source_stamp.clone(),
                    source_payload_digest: source_payload_digest.clone(),
                    validated_source_wc_commit: source_wc.clone(),
                    target_stamp: stamp(&root)?,
                    config_root: default.clone(),
                    link_config: link_config_bytes(&default)?,
                    preparation_id: batch.as_ref().map(|b| b.id.clone()),
                    history: bookmark_names
                        .as_mut()
                        .map(|allocator| {
                            TargetHistory::plan(
                                &client,
                                &operation,
                                &fact.commit_id,
                                &source_commit,
                                request.no_freshen_bookmark,
                                allocator,
                            )
                        })
                        .transpose()?,
                    execution_diagnostics: Vec::new(),
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
    }
    if let Some(b) = batch.as_mut()
        && b.plans.is_none()
    {
        prepare::verify_verified(b)?;
        b.plans = Some(plans.clone());
        prepare::save(&store, b)?;
        for plan in &plans {
            prepare::handoff(&store, plan, request.dry_run)?;
        }
    }
    let first = plans.first().or_else(|| {
        batch
            .as_ref()
            .and_then(|b| b.plans.as_ref())
            .and_then(|p| p.first())
    });
    let (report_source, report_commit) = match first {
        Some(first) => (first.source.clone(), first.source_commit.clone()),
        None if request.dry_run && readiness.preview.is_some() => {
            let preview = readiness.preview.as_ref().unwrap();
            (
                preview.source.clone(),
                preview
                    .source_commit_id
                    .clone()
                    .context("source proof unavailable")?,
            )
        }
        None => {
            let b = batch.as_ref().context("select at least one workspace")?;
            if !b.request.names.is_empty() {
                bail!("select at least one workspace");
            }
            (
                prepare::source(b).to_path_buf(),
                b.goal
                    .clone()
                    .context("source-only preparation lacks frozen goal")?,
            )
        }
    };
    let mut report = ReseedReport {
        dry_run: request.dry_run,
        source: report_source,
        source_commit_id: Some(report_commit),
        preparation_summary: batch.as_ref().map(prepare::summary),
        targets: completed,
        selections: selection.selections,
        skipped: selection.skipped,
    };
    if request.dry_run {
        report.targets.extend(readiness.failures.values().cloned());
    }
    for mut plan in plans {
        if request.dry_run {
            let consequence = match &plan.base_commit {
                Some(base) => format!(
                    "RESET checkout to a new child of {base}; tracked work saved in JJ history at {} (not a feature rebase)",
                    plan.old_commit
                ),
                None => format!(
                    "KEEP current feature tree and uncommitted work at {}",
                    plan.old_commit
                ),
            };
            report.targets.push(ReseedOutcome {
                name: plan.name.clone(),
                status: ReseedStatus::Planned,
                message: if request.resume {
                    describe(&plan, true)
                } else {
                    format!(
                        "READY; {consequence}; {}; {}",
                        readiness
                            .details
                            .get(&plan.name)
                            .map(String::as_str)
                            .unwrap_or("no new target snapshot"),
                        describe(&plan, true)
                    )
                },
            });
            continue;
        }
        let started = std::time::Instant::now();
        let recovering_target = request.resume || plan.preparation_id.is_some();
        let outcome = execute(&store, &default, &mut plan, recovering_target);
        let (status, message) = match outcome {
            Ok(()) => (
                ReseedStatus::Succeeded,
                format!(
                    "{}; elapsed {} ms",
                    describe(&plan, false),
                    started.elapsed().as_millis()
                ),
            ),
            Err(error) => (
                if store.pending(&plan.name)?.is_some() {
                    ReseedStatus::Pending
                } else {
                    ReseedStatus::Failed
                },
                format!("{error:#}"),
            ),
        };
        report.targets.push(ReseedOutcome {
            name: plan.name.clone(),
            status,
            message,
        });
    }
    if let Some(b) = batch.as_ref()
        && !request.dry_run
    {
        prepare::finish_if_complete(&store, b)?;
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
            let output =
                client.run_at_unchecked(&operation, ["workspace", "root", "--name", name])?;
            registered.insert(name.clone(), None);
            if output.success() {
                let path = PathBuf::from(output.trimmed_stdout()?);
                registered.insert(name.clone(), Some(path.clone()));
                if let Ok(path) = path.canonicalize() {
                    roots.insert(name.clone(), path);
                }
            }
        }
        let context = checked_context(client)
            .context("cannot verify primary/caller JJ workspace topology")?;
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
                fs::symlink_metadata(&pointer).is_ok_and(|m| m.is_dir())
                    && pointer.canonicalize().ok().as_ref() == Some(&expected_repository)
            })
            .collect::<Vec<_>>();
        if owners.len() != 1 {
            bail!(
                "cannot verify primary JJ workspace: repository owner registration is missing or ambiguous"
            );
        }
        let (primary_name, default) = owners[0];
        // Context may omit primary_checkout for a JJ-only checkout nested inside
        // an unrelated Git checkout. The registered physical repository owner
        // still proves JJ identity; never infer it from the enclosing Git root.
        if context
            .jj
            .primary_checkout
            .as_ref()
            .is_some_and(|root| root != default)
            || context
                .jj
                .primary_workspace
                .as_ref()
                .is_some_and(|name| name != primary_name)
        {
            bail!("primary JJ workspace discovery disagrees with repository owner registration");
        }
        let primary = checked_context(&JjClient::new(default))
            .context("cannot verify primary JJ workspace topology")?;
        if primary.jj.workspace_root.as_ref() != Some(default)
            || primary.jj.workspace_name.as_ref() != Some(primary_name)
            || repository(&JjClient::new(default))? != expected_repository
            || context
                .jj
                .workspace_name
                .as_ref()
                .and_then(|name| roots.get(name))
                != Some(&client.cwd().canonicalize()?)
        {
            bail!("primary/caller JJ workspace root registration is inconsistent");
        }
        Ok(Self {
            current: context
                .jj
                .workspace_name
                .context("cannot determine registered current workspace")?,
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
            context
                .diagnostics
                .iter()
                .map(|d| d.message.as_str())
                .collect::<Vec<_>>()
                .join("; ")
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

fn load_config(default: &Path) -> Result<(LocalConfig, Option<Vec<u8>>)> {
    let path = default.join(".jwreseed.local.toml");
    match fs::read(&path) {
        Ok(bytes) => {
            let mut config: LocalConfig = toml::from_str(std::str::from_utf8(&bytes)?)
                .with_context(|| format!("invalid {}", path.display()))?;
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
            Ok((config, Some(bytes)))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok((LocalConfig::default(), None)),
        Err(e) => Err(e).with_context(|| format!("cannot read {}", path.display())),
    }
}

fn validate_target(
    client: &JjClient,
    metadata: &WorkspaceMetadataStore,
    default: &Path,
    source: &Path,
    name: &str,
    root: &Path,
) -> Result<()> {
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
    if record
        .as_ref()
        .is_some_and(|m| m.external_owner.is_some() || m.owned_git_worktree.is_some())
    {
        bail!("workspace {name} has external/colocated ownership");
    }
    crate::ownership::ensure_removal_allowed(client, name, Some(root), record.as_ref())?;
    // Several workspaces can deliberately point at the same commit. Commit-level
    // current-working-copy predicates alone therefore do not identify a root.
    if context.jj.workspace_name.as_deref() != Some(name)
        || context.jj.workspace_root.as_deref() != Some(root)
    {
        bail!("workspace root registration does not uniquely match {name}");
    }
    Ok(())
}

fn ensure_quiescence(client: &JjClient) -> Result<()> {
    if client.uses_watchman()? {
        bail!(
            "Watchman workspaces are unsupported for reseed; disable its monitor and background snapshot triggers and pause writers first"
        );
    }
    let trigger = client.run_unchecked([
        "--ignore-working-copy",
        "config",
        "get",
        "fsmonitor.watchman.register-snapshot-trigger",
    ])?;
    if trigger.success() && trigger.trimmed_stdout()? == "true" {
        bail!(
            "Watchman snapshot triggers are unsupported; stop background triggers before reseeding"
        );
    }
    let backend = client.run_unchecked([
        "--ignore-working-copy",
        "config",
        "get",
        "fsmonitor.backend",
    ])?;
    if backend.success() && !matches!(backend.trimmed_stdout()?.as_str(), "none" | "") {
        bail!("unsupported filesystem monitor; reseed requires full-scan quiescent workspaces");
    }
    Ok(())
}

fn require_full(client: &JjClient) -> Result<()> {
    let output = client.run(["--ignore-working-copy", "sparse", "list"])?;
    if output.stdout()?.trim_end_matches(['\r', '\n']) != "." {
        bail!(
            "reseed initially supports full sparse patterns only: {}",
            client.cwd().display()
        );
    }
    Ok(())
}

fn snapshot(client: &JjClient) -> Result<()> {
    let output = client.run([
        "--config=fsmonitor.backend=none",
        "--config=snapshot.auto-update-stale=false",
        "status",
    ])?;
    diagnostics(output.stderr());
    Ok(())
}

fn verified_snapshot(client: &JjClient) -> Result<()> {
    // Retained ignored data can become unignored after --at changes .gitignore.
    // Reseed is not permission to auto-track it into the frozen desired commit.
    let output = client.run([
        "--config=fsmonitor.backend=none",
        "--config=snapshot.auto-track=none()",
        "--config=snapshot.auto-update-stale=false",
        "status",
    ])?;
    diagnostics(output.stderr());
    Ok(())
}

fn diagnostics(text: String) {
    if !text.is_empty() {
        eprintln!("{text}");
    }
}

fn require_unconflicted(client: &JjClient, operation: &str, commit: &str) -> Result<()> {
    let output = client.run_at(
        operation,
        [
            "log",
            "--no-graph",
            "-r",
            commit,
            "-T",
            "json(conflict) ++ \"\\n\"",
        ],
    )?;
    if serde_json::from_str::<bool>(output.stdout()?.trim())? {
        bail!("reseed does not support conflicted trees");
    }
    Ok(())
}

fn changed_paths(
    client: &JjClient,
    operation: &str,
    from: &str,
    to: &str,
) -> Result<BTreeSet<PathBuf>> {
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
    let fileset = format!(
        "file:{}",
        serde_json::to_string(path.to_str().context("tracked path not UTF-8")?)?
    );
    let output = client.run_at(
        operation,
        [
            "diff", "--from", "root()", "--to", commit, "--git", &fileset,
        ],
    )?;
    diagnostics(output.stderr());
    parse_added_symlink(output.stdout_bytes()).with_context(|| format!("JJ cannot expose the frozen symlink target at {}; capability gap detected before materialization", path.display()))
}

/// JJ computes whole-tree copy records before applying a diff's fileset. Batch
/// capability checks so that cost is paid once per group, not once per symlink.
/// Keep the individual immutable extractor for recovery and future-link queries.
fn verify_symlink_support(
    client: &JjClient,
    operation: &str,
    commit: &str,
    entries: &[Entry],
) -> Result<()> {
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
            args.push(format!(
                "file:{}",
                serde_json::to_string(path.to_str().context("tracked path not UTF-8")?)?
            ));
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
        *remaining = remaining
            .strip_prefix(expected)
            .context("unexpected symlink diff header or boundary")?;
        Ok(())
    }
    fn line<'a>(remaining: &mut &'a [u8]) -> Result<&'a [u8]> {
        let end = remaining
            .iter()
            .position(|byte| *byte == b'\n')
            .context("truncated symlink diff line")?
            + 1;
        let (line, rest) = remaining.split_at(end);
        *remaining = rest;
        Ok(line)
    }
    for path in paths {
        let start = remaining;
        let path = path.to_str().context("tracked path not UTF-8")?;
        // Consume known pathname bytes literally. Splitting at apparent headers
        // would let newline filenames spoof another file or a payload hunk.
        consume(
            &mut remaining,
            format!("diff --git a/{path} b/{path}\nnew file mode 120000\nindex ").as_bytes(),
        )?;
        let index = std::str::from_utf8(line(&mut remaining)?)?.trim_end_matches('\n');
        let (old, new) = index
            .split_once("..")
            .context("invalid symlink diff index")?;
        if old.is_empty()
            || !old.bytes().all(|b| b == b'0')
            || new.is_empty()
            || !new.bytes().all(|b| b.is_ascii_hexdigit())
        {
            bail!("invalid added symlink diff index");
        }
        consume(
            &mut remaining,
            format!("--- /dev/null\n+++ b/{path}\n").as_bytes(),
        )?;
        let hunk = std::str::from_utf8(line(&mut remaining)?)?;
        let range = hunk
            .strip_prefix("@@ -0,0 +1")
            .and_then(|s| s.strip_suffix(" @@\n"))
            .context("JJ did not provide a full-addition symlink hunk")?;
        let count = if range.is_empty() {
            1
        } else {
            range
                .strip_prefix(',')
                .context("invalid symlink hunk range")?
                .parse::<usize>()?
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
    let lines = diff
        .split_inclusive(|byte| *byte == b'\n')
        .collect::<Vec<_>>();
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
        result.extend_from_slice(
            line.strip_prefix(b"+")
                .context("symlink diff contains a non-addition")?,
        );
    }
    match &body[count..] {
        [] => {}
        [line]
            if *line == b"\\ No newline at end of file\n"
                || *line == b"\\ No newline at end of file" =>
        {
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
/// additional files and disables automatic stale-tree updates. Do not use it on partial
/// recovery trees: those must first pass `verify_reconstructed_disk`.
fn verify_disk(
    client: &JjClient,
    _operation: &str,
    commit: &str,
    root: &Path,
    _entries: &[Entry],
) -> Result<()> {
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
        bail!(
            "tracked tree differs from frozen desired tree in {}; JJ snapshot performed",
            root.display()
        );
    }
    Ok(())
}

/// Recovery-only raw proof: sparse state may still be empty or expansion may
/// have been interrupted. No snapshot-capable query is safe until repair is
/// complete; retain byte/type/mode verification against pinned immutable data.
fn verify_reconstructed_disk(
    client: &JjClient,
    operation: &str,
    commit: &str,
    root: &Path,
    entries: &[Entry],
) -> Result<()> {
    for entry in entries {
        confined_parents(root, &entry.path)?;
        let path = root.join(&entry.path);
        let metadata = fs::symlink_metadata(&path)
            .with_context(|| format!("materialization incomplete at {}", path.display()))?;
        let bytes = if entry.kind == "symlink" {
            use std::os::unix::ffi::OsStrExt;
            if !metadata.file_type().is_symlink() {
                bail!("tracked symlink type changed: {}", path.display());
            }
            fs::read_link(&path)?.as_os_str().as_bytes().to_vec()
        } else {
            if !metadata.is_file()
                || (metadata.permissions().mode() & 0o111 != 0) != entry.executable
            {
                bail!("tracked file type/mode changed: {}", path.display());
            }
            fs::read(&path)?
        };
        let expected = if entry.kind == "symlink" {
            symlink_bytes(client, operation, commit, &entry.path)?
        } else {
            file_bytes(client, operation, commit, &entry.path)?
        };
        if bytes != expected {
            bail!(
                "tracked bytes differ from frozen desired tree at {}; no snapshot performed",
                path.display()
            );
        }
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

fn preflight_obstructions(
    root: &Path,
    old: &[Entry],
    desired: &[Entry],
    replace: bool,
) -> Result<()> {
    if replace {
        return Ok(());
    }
    let old_paths = old
        .iter()
        .map(|entry| entry.path.as_path())
        .collect::<BTreeSet<_>>();
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
                    bail!(
                        "configured link sources overlap: {} and {}",
                        left.source.display(),
                        right.source.display()
                    );
                }
            }
        }
        for check in checks {
            let path = check
                .source
                .strip_prefix(root)
                .context("link source escapes target")?;
            relative(path)?;
            let target_exists = future.exists(&check.target, 0)?;
            if check.required && !target_exists {
                bail!(
                    "required link target will be absent after reseed: {}",
                    check.target.display()
                );
            }
            // A source-equals-target override deliberately keeps a private directory.
            // It performs no link mutation, but required existence must still be
            // proved against the future tree (replace-content may erase it).
            if check.source == check.target {
                continue;
            }
            if entries.iter().any(|e| overlap(path, &e.path))
                || ignored.iter().any(|p| overlap(path, p))
            {
                bail!(
                    "configured link overlaps reseed content: {}",
                    path.display()
                );
            }
            if !replace
                && matches!(
                    check.state,
                    links::LinkCheckState::Conflicting | links::LinkCheckState::Unreadable(_)
                )
            {
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
            bail!(
                "future link target contains a symlink cycle: {}",
                path.display()
            );
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
                    std::ffi::OsString::from_vec(symlink_bytes(
                        self.client,
                        self.operation,
                        self.commit,
                        path,
                    )?)
                    .into(),
                ));
            }
            return Ok(FutureNode::File);
        }
        if self.entries.iter().any(|e| e.path.starts_with(path))
            || self.ignored.iter().any(|p| p.starts_with(path))
        {
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
            FutureNode::Directory
                if self.old_paths.iter().any(|p| p.starts_with(path))
                    && !self.has_retained_child(path)? =>
            {
                Ok(FutureNode::Missing)
            }
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
                if !self.old_paths.iter().any(|p| p.starts_with(&path))
                    || self.has_retained_child(&path)?
                {
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
        Err(e) if matches!(e.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
            Ok(FutureNode::Missing)
        }
        Err(e) => Err(e.into()),
    }
}

fn describe(plan: &Pending, planned: bool) -> String {
    let mode = plan
        .base_commit
        .as_ref()
        .map(|id| format!("new child of {id}"))
        .unwrap_or_else(|| format!("keep {}", plan.old_commit));
    let clone_paths = plan.clones.iter().collect::<BTreeSet<_>>();
    let clone_bytes: u64 = plan
        .source_stamp
        .iter()
        .filter(|s| clone_paths.contains(&s.path) && s.mode & 0o170000 == 0o100000)
        .map(|s| s.size)
        .sum();
    let payload = plan
        .source_stamp
        .iter()
        .filter(|s| {
            plan.ignored.iter().any(|p| s.path.starts_with(p)) && s.mode & 0o170000 == 0o100000
        })
        .collect::<Vec<_>>();
    let ignored_bytes: u64 = payload.iter().map(|s| s.size).sum();
    let mut description = format!(
        "root {}; {mode}; {} {} tracked paths ({clone_bytes} bytes), {} {} paths; {} ignored roots [{}] ({} files, {ignored_bytes} bytes); {}{}",
        plan.root.display(),
        if planned {
            "would reflink"
        } else {
            "reflinked"
        },
        plan.clones.len(),
        if planned {
            "JJ would materialize"
        } else {
            "JJ materialized"
        },
        plan.entries.len().saturating_sub(plan.clones.len()),
        if planned { "would replace" } else { "replaced" },
        plan.ignored
            .iter()
            .map(|p| format!("{:?}", p))
            .collect::<Vec<_>>()
            .join(", "),
        payload.len(),
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
    if let Some(history) = &plan.history {
        description.push_str(&format!("; {}", history.describe(planned)));
    }
    description
}

fn select_ignored(
    source: &Path,
    patterns: &[String],
    rules: &IgnoreRules,
    entries: &[Entry],
) -> Result<(Vec<PathBuf>, Vec<String>)> {
    let selected = Patterns::compile(patterns)?.select(
        source,
        entries.iter().map(|entry| entry.path.as_path()),
        |path| rules.is_ignored_directory(path),
    )?;
    Ok((selected.roots, selected.skipped))
}

fn validate_selected_ignored(
    source: &Path,
    roots: &[PathBuf],
    rules: &IgnoreRules,
    entries: &[Entry],
) -> Result<()> {
    cow::ignored::validate_selected(
        source,
        roots,
        entries.iter().map(|entry| entry.path.as_path()),
        |path| rules.is_ignored_directory(path),
    )
}

fn absolute_links(source: &Path, roots: &[PathBuf]) -> Result<Vec<String>> {
    fn visit(source: &Path, path: &Path, warnings: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                visit(source, &entry.path(), warnings)?;
            } else if entry.file_type()?.is_symlink() {
                let target = fs::read_link(entry.path())?;
                if target.is_absolute() {
                    warnings.push(format!(
                        "absolute dependency symlink {} -> {} (not self-contained)",
                        entry.path().strip_prefix(source)?.display(),
                        target.display()
                    ));
                }
            }
        }
        Ok(())
    }
    let mut warnings = Vec::new();
    for root in roots {
        visit(source, &source.join(root), &mut warnings)?;
    }
    Ok(warnings)
}

fn validate_identities(plan: &Pending) -> Result<()> {
    plan.root_identity.verify(&plan.root)?;
    plan.jj_identity.verify(&plan.root.join(".jj"))?;
    plan.source_identity.verify(&plan.source)?;
    plan.source_jj_identity.verify(&plan.source.join(".jj"))?;
    if repository(&JjClient::new(&plan.root))? != plan.repository
        || repository(&JjClient::new(&plan.source))? != plan.repository
    {
        bail!("repository identity changed since pending plan");
    }
    match fs::symlink_metadata(plan.root.join(".git")) {
        Ok(_) => bail!("target acquired Git metadata since frozen plan; pending retained"),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error).context("cannot recheck target Git metadata"),
    }
    if plan.sparse != ["."] {
        bail!("pending sparse pattern unsupported");
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

fn validate_source(plan: &mut Pending, resume: bool) -> Result<()> {
    plan.source_identity.verify(&plan.source)?;
    plan.source_jj_identity.verify(&plan.source.join(".jj"))?;
    if link_config_bytes(&plan.config_root)? != plan.link_config {
        bail!("workspace link configuration changed since frozen plan; pending retained");
    }
    let source = JjClient::new(&plan.source);
    require_full(&source)?;
    let operation = source.operation_id()?;
    let current = source.resolve_one_at(&operation, "@")?.commit_id;
    let before = stamp(&plan.source)?;
    let unchanged = current == plan.validated_source_wc_commit && before == plan.source_stamp;
    if !resume && !unchanged {
        bail!("source contents changed; restore the frozen source before --resume");
    }
    // Reuse only a previously verified tree/payload bound to exact filesystem
    // stamps and WC identity. Source recovery compares an intact tracked tree
    // and dependency digest before updating stamps; only the target can be partial.
    let mut validated_current = current.clone();
    if !unchanged || !plan.ignored.is_empty() {
        let entries = entries(&source, &plan.source_operation, &plan.source_commit)?;
        if !unchanged {
            #[cfg(test)]
            SOURCE_LOGICAL_CHECKS.with(|count| count.set(count.get() + 1));
            verify_disk(
                &source,
                &plan.source_operation,
                &plan.source_commit,
                &plan.source,
                &entries,
            )?;
            validated_current = current_commit(&source)?.1;
            if payload_digest(&plan.source, &plan.ignored)? != plan.source_payload_digest {
                bail!(
                    "source ignored payload differs from the frozen plan; restore its bytes/types/modes before --resume"
                );
            }
        }
        if !plan.ignored.is_empty() {
            // Backend/global excludes live outside the source tree stamp. Read
            // their effective rules again even when tracked/payload bytes are
            // reusable; do not cache authorization to copy ignored data.
            let rules = IgnoreRules::new(
                &source,
                &plan.source_operation,
                &plan.source_commit,
                &entries,
                true,
            )?;
            for path in &plan.ignored {
                rules.require_ignored(path)?;
            }
        }
    }
    if stamp(&plan.source)? != before || current_commit(&source)?.1 != validated_current {
        bail!("source changed during content validation; pause writers before retrying");
    }
    plan.source_identity.verify(&plan.source)?;
    plan.source_jj_identity.verify(&plan.source.join(".jj"))?;
    if resume {
        plan.source_stamp = before;
        plan.validated_source_wc_commit = validated_current;
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
    let output = client.run_at(
        operation,
        [
            "log",
            "--no-graph",
            "-r",
            commit,
            "-T",
            "json(empty) ++ \"\\n\"",
        ],
    )?;
    Ok(parents.len() == 1
        && parents[0].commit_id == base
        && serde_json::from_str::<bool>(output.stdout()?.trim())?)
}

fn transition(store: &Store, plan: &mut Pending, phase: Phase) -> Result<()> {
    plan.phase = phase;
    store.save(plan)?;
    fault(phase)
}

/// Repair from immutable public tree data, without running ANY snapshot-capable
/// target command. The plan's future --at tree is not necessarily its current
/// commit: before Newing completes, repair must restore the original feature tree.
fn reconstruct_current_tree(plan: &Pending) -> Result<()> {
    use std::os::unix::ffi::OsStringExt;
    let client = JjClient::new(&plan.root);
    let sparse = client.run(["--ignore-working-copy", "sparse", "list"])?;
    if !matches!(sparse.stdout()?.trim_end_matches(['\r', '\n']), "" | ".") {
        bail!("pending target sparse patterns changed; recovery refused without snapshot");
    }
    let expected = entries(&client, &plan.target_operation, &plan.desired_commit)?;
    let expected_paths = expected
        .iter()
        .map(|e| e.path.clone())
        .collect::<BTreeSet<_>>();
    // Remove only recorded tracked leftovers absent from the CURRENT expected
    // tree. A prior file now used as a directory is never recursively deleted.
    let mut obsolete = plan
        .old_paths
        .iter()
        .chain(plan.entries.iter().map(|e| &e.path))
        .filter(|p| !expected_paths.contains(*p))
        .cloned()
        .collect::<Vec<_>>();
    obsolete.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
    obsolete.dedup();
    for path in &obsolete {
        remove_recorded_file(&plan.root, path, false)?;
    }
    if !plan.replace_content {
        for entry in &plan.entries {
            if plan
                .old_paths
                .iter()
                .any(|path| path != &entry.path && path.starts_with(&entry.path))
                && fs::symlink_metadata(plan.root.join(&entry.path)).is_ok_and(|m| m.is_dir())
            {
                // A declared tracked-directory-to-leaf transition may leave
                // empty checkout parent directories. Remove only these empty
                // directories with remove_dir, never their retained contents.
                remove_empty_directories(&plan.root.join(&entry.path))?;
            }
        }
    }
    let changed = changed_paths(
        &client,
        &plan.target_operation,
        &plan.source_commit,
        &plan.desired_commit,
    )?;
    let source_entries = entries(
        &JjClient::new(&plan.source),
        &plan.source_operation,
        &plan.source_commit,
    )?;
    let source_paths = source_entries
        .iter()
        .map(|e| &e.path)
        .collect::<BTreeSet<_>>();
    let clones = expected
        .iter()
        .filter(|e| source_paths.contains(&e.path) && !changed.contains(&e.path))
        .map(|e| e.path.clone())
        .collect::<BTreeSet<_>>();
    for (index, entry) in expected.iter().enumerate() {
        confined_parents(&plan.root, &entry.path)?;
        let path = plan.root.join(&entry.path);
        match fs::symlink_metadata(&path) {
            Ok(m) if m.is_dir() && plan.replace_content => cow::remove_path(&path)?,
            Ok(m) if m.is_dir() => remove_empty_directories(&path)?,
            Ok(_) => cow::remove_path(&path)?,
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        if clones.contains(&entry.path) {
            fault_during_repair(index)?;
            continue;
        }
        let bytes = if entry.kind == "symlink" {
            symlink_bytes(
                &client,
                &plan.target_operation,
                &plan.desired_commit,
                &entry.path,
            )?
        } else {
            file_bytes(
                &client,
                &plan.target_operation,
                &plan.desired_commit,
                &entry.path,
            )?
        };
        fs::create_dir_all(path.parent().context("tracked path has no parent")?)?;
        confined_parents(&plan.root, &entry.path)?;
        if entry.kind == "symlink" {
            std::os::unix::fs::symlink(std::ffi::OsString::from_vec(bytes), &path)?;
        } else {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?;
            file.write_all(&bytes)?;
            file.set_permissions(fs::Permissions::from_mode(if entry.executable {
                0o755
            } else {
                0o644
            }))?;
        }
        fault_during_repair(index)?;
    }
    cow::clone_files(
        &plan.source,
        &clones.into_iter().collect::<Vec<_>>(),
        &plan.root,
    )?;
    verify_reconstructed_disk(
        &client,
        &plan.target_operation,
        &plan.desired_commit,
        &plan.root,
        &expected,
    )?;
    if current_commit(&client)?.1 != plan.desired_commit {
        bail!("target commit changed during reconstruction; no snapshot performed");
    }
    Ok(())
}

fn remove_empty_directories(root: &Path) -> Result<()> {
    fn collect(root: &Path, directories: &mut Vec<PathBuf>) -> Result<()> {
        for child in fs::read_dir(root)? {
            let child = child?;
            if !child.file_type()?.is_dir() {
                bail!(
                    "retained unrecorded entry obstructs recorded tracked file: {}; recovery did not delete it",
                    child.path().display()
                );
            }
            collect(&child.path(), directories)?;
        }
        directories.push(root.to_path_buf());
        Ok(())
    }
    cow::validate_tree(root)?;
    let mut directories = Vec::new();
    collect(root, &mut directories)?;
    for directory in directories {
        fs::remove_dir(directory)?;
    }
    Ok(())
}

/// Only a target that crossed the Prepared boundary needs reconstruction before
/// snapshotting. A preparation handoff (including --resume) is still untouched.
fn execute(store: &Store, default: &Path, plan: &mut Pending, resume: bool) -> Result<()> {
    let reconstruct = plan.phase != Phase::Prepared;
    if default != plan.config_root {
        bail!("default workspace changed since frozen plan");
    }
    validate_identities(plan)?;
    validate_source(plan, resume)?;
    let client = JjClient::new(&plan.root);
    ensure_quiescence(&client)?;
    validate_contents(&plan.root, false)?;
    for warning in plan
        .skipped
        .iter()
        .filter(|warning| warning.starts_with("absolute dependency symlink "))
    {
        diagnostics(format!("Warning: {warning}"));
    }
    let (operation, current) = current_commit(&client)?;
    if plan.phase == Phase::Newing && current != plan.old_commit {
        let base = plan
            .base_commit
            .as_deref()
            .context("pending new-child intent lacks base")?;
        if !exact_empty_child(&client, &operation, &current, base)? {
            bail!("target changed during interrupted jj new; pending retained without snapshot");
        }
        plan.desired_commit = current.clone();
        plan.target_operation = operation.clone();
        store.save(plan)?;
    } else if current != plan.desired_commit {
        bail!(
            "target WC revision changed since frozen plan: recorded desired {}, observed {}; pending retained without another target snapshot; verification may have recorded rejected edits; preserve those changes before deliberately restoring the recorded target revision and retrying --resume",
            plan.desired_commit,
            current,
        );
    }
    if !reconstruct && !resume {
        require_full(&client)?;
        if current_commit(&client)?.1 != plan.old_commit || stamp(&plan.root)? != plan.target_stamp
        {
            bail!(
                "target contents changed after batch preflight; reseed refused before sparse mutation"
            );
        }
        store.save(plan)?;
    } else if plan.phase == Phase::Prepared {
        require_full(&client)?;
        if stamp(&plan.root)? != plan.target_stamp {
            bail!(
                "target contents changed after pending preparation; refused without deleting or snapshotting them"
            );
        }
    }
    if let Some(history) = &plan.history {
        if history.target.commit_id != plan.old_commit
            || plan.base_commit.as_deref() != Some(history.upstream_goal.as_str())
            || history.upstream_goal != plan.source_commit
        {
            bail!("target history does not match frozen advancement intent");
        }
        // Also covers a preparation handoff or interruption before bookmark
        // creation. Later phases verify the SAME pair before any tree repair.
        store.save(plan)?;
        history.ensure_bookmark(&client)?;
        fault_after_bookmark()?;
    }
    // Stock sparse mutations reject --ignore-working-copy. Recovery therefore
    // reconstructs the entire CURRENT commit before a normal sparse command may
    // snapshot it. The write-ahead Clearing phase covers interruptions during
    // that repair as well as during sparse clearing itself.
    if reconstruct {
        transition(store, plan, Phase::Clearing)?;
        reconstruct_current_tree(plan)?;
    } else {
        verify_disk(
            &client,
            &plan.target_operation,
            &plan.desired_commit,
            &plan.root,
            &plan.entries,
        )?;
        if current_commit(&client)?.1 != plan.desired_commit {
            bail!("target WC revision changed during tracked-tree verification; pending retained");
        }
        transition(store, plan, Phase::Clearing)?;
    }
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
        bail!("sparse clearing changed the recorded CURRENT commit; pending retained");
    }
    for path in plan
        .old_paths
        .iter()
        .chain(plan.entries.iter().map(|e| &e.path))
    {
        remove_recorded_file(&plan.root, path, false)?;
    }
    transition(store, plan, Phase::Empty)?;
    if let Some(base) = plan.base_commit.clone() {
        let (operation, current) = current_commit(&client)?;
        if (plan.history.is_some() && current == plan.old_commit)
            || !exact_empty_child(&client, &operation, &current, &base)?
        {
            // A crash after `new` but before its journal update is distinguished
            // from concurrent graph changes by the write-ahead Newing phase.
            transition(store, plan, Phase::Newing)?;
            let output = client.run([
                "--config=fsmonitor.backend=none",
                "--config=snapshot.auto-track=none()",
                "--config=snapshot.auto-update-stale=false",
                "new",
                &crate::jj::commit_id_revset(&base)?,
            ])?;
            diagnostics(output.stderr());
            fault_after_new()?;
            let (operation, desired) = current_commit(&client)?;
            if !exact_empty_child(&client, &operation, &desired, &base)? {
                bail!("JJ did not create the expected empty child");
            }
            plan.desired_commit = desired;
            plan.target_operation = operation;
            store.save(plan)?;
        }
    }
    // Remaining contents are retained unless replacement was explicitly selected.
    if plan.replace_content {
        validate_contents(&plan.root, false)?;
        for child in fs::read_dir(&plan.root)? {
            let child = child?;
            if child.file_name() != ".jj" {
                cow::remove_path(&child.path())?;
            }
        }
    } else {
        for path in &plan.ignored {
            confined_parents(&plan.root, path)?;
            cow::remove_path(&plan.root.join(path))?;
        }
    }
    transition(store, plan, Phase::Filling)?;
    validate_source(plan, resume)?;
    for path in &plan.clones {
        confined_parents(&plan.root, path)?;
    }
    cow::clone_files(&plan.source, &plan.clones, &plan.root)?;
    // A normal sparse reset snapshots only the existing EMPTY sparse selection,
    // not the staged prefills outside it. JJ then materializes all differences.
    transition(store, plan, Phase::Expanding)?;
    let output = client.run([
        "--config=fsmonitor.backend=none",
        "--config=snapshot.auto-track=none()",
        "--config=snapshot.auto-update-stale=false",
        "sparse",
        "reset",
    ])?;
    diagnostics(output.stderr());
    fault_after_expansion()?;
    validate_identities(plan)?;
    verify_disk(
        &client,
        &plan.target_operation,
        &plan.desired_commit,
        &plan.root,
        &plan.entries,
    )?;
    if current_commit(&client)?.1 != plan.desired_commit {
        bail!("materialization verification changed the desired WC commit; pending retained");
    }
    transition(store, plan, Phase::Verified)?;
    let rules = IgnoreRules::new(
        &client,
        &plan.target_operation,
        &plan.desired_commit,
        &plan.entries,
        !plan.replace_content,
    )?;
    for path in &plan.ignored {
        rules.require_ignored(path)?;
    }
    // Link existence is checked against the actual filesystem only after ignored
    // payload restoration. Early future-state preflight already proved required
    // targets; testing live existence here would reject freshly cleared caches.
    transition(store, plan, Phase::CopyingIgnored)?;
    validate_source(plan, resume)?;
    for path in &plan.ignored {
        confined_parents(&plan.root, path)?;
        cow::remove_path(&plan.root.join(path))?;
        fs::create_dir_all(
            plan.root
                .join(path)
                .parent()
                .context("ignored root has no parent")?,
        )?;
        confined_parents(&plan.root, path)?;
        cow::clone_tree(&plan.source.join(path), &plan.root.join(path))?;
    }
    validate_source(plan, resume)?;
    let link_report = links::apply_workspace_links(default, &plan.root)?;
    if link_report.has_entries() {
        plan.execution_diagnostics.push(format!(
            "links: {} created, {} satisfied, {} skipped",
            link_report.linked, link_report.satisfied, link_report.skipped_missing_target
        ));
    }
    verify_disk(
        &client,
        &plan.target_operation,
        &plan.desired_commit,
        &plan.root,
        &plan.entries,
    )?;
    if current_commit(&client)?.1 != plan.desired_commit {
        bail!("ignored/link contents changed tracked tree; pending retained");
    }
    require_full(&client)?;
    validate_identities(plan)?;
    if let Some(base) = &plan.base_commit {
        let (operation, current) = current_commit(&client)?;
        if !exact_empty_child(&client, &operation, &current, base)? {
            bail!("desired child no longer has the exact frozen parent");
        }
    }
    transition(store, plan, Phase::Done)?;
    store.finish(plan)
}

fn remove_recorded_file(root: &Path, relative_path: &Path, must_be_file: bool) -> Result<()> {
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
        Ok(m) if m.is_dir() && must_be_file => bail!(
            "recorded prefill replaced by a directory; refusing destructive recovery: {}",
            path.display()
        ),
        Ok(m) if m.is_dir() => {}
        Ok(_) => cow::remove_path(&path)?,
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

#[cfg(test)]
thread_local! {
    static FAIL_AT: std::cell::Cell<Option<Phase>> = const { std::cell::Cell::new(None) };
    static FAIL_AFTER_EXPANSION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static FAIL_AFTER_NEW: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static FAIL_AFTER_BOOKMARK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static FAIL_DURING_REPAIR: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
    static SOURCE_LOGICAL_CHECKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
fn fault(phase: Phase) -> Result<()> {
    #[cfg(test)]
    if FAIL_AT.with(|fail| fail.get() == Some(phase)) {
        bail!("injected interruption at {phase:?}");
    }
    let _ = phase;
    Ok(())
}
fn fault_after_expansion() -> Result<()> {
    #[cfg(test)]
    if FAIL_AFTER_EXPANSION.with(std::cell::Cell::get) {
        bail!("injected interruption after sparse expansion");
    }
    Ok(())
}
fn fault_after_new() -> Result<()> {
    #[cfg(test)]
    if FAIL_AFTER_NEW.with(std::cell::Cell::get) {
        bail!("injected interruption after jj new");
    }
    Ok(())
}
fn fault_after_bookmark() -> Result<()> {
    #[cfg(test)]
    if FAIL_AFTER_BOOKMARK.with(std::cell::Cell::get) {
        bail!("injected interruption after freshen bookmark");
    }
    Ok(())
}
fn fault_during_repair(index: usize) -> Result<()> {
    #[cfg(test)]
    if FAIL_DURING_REPAIR.with(|fail| fail.get() == Some(index)) {
        bail!("injected interruption during tree repair");
    }
    let _ = index;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_journal_flushes_large_records_and_preserves_previous_on_serialization_failure() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("journal.json");
        let value = (0..10_000)
            .map(|index| format!("tracked/path-{index:05}"))
            .collect::<Vec<_>>();
        atomic_json(&path, &value).unwrap();
        let previous = fs::read(&path).unwrap();
        assert!(previous.ends_with(b"\n"));
        assert_eq!(
            serde_json::from_slice::<Vec<String>>(&previous).unwrap(),
            value
        );

        struct FailedRecord;
        impl Serialize for FailedRecord {
            fn serialize<S: serde::Serializer>(
                &self,
                _: S,
            ) -> std::result::Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("injected serialization failure"))
            }
        }
        assert!(atomic_json(&path, &FailedRecord).is_err());
        assert_eq!(fs::read(&path).unwrap(), previous);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    fn verification_fixture() -> (tempfile::TempDir, JjClient) {
        let temporary = tempfile::tempdir().unwrap();
        let client = JjClient::new(temporary.path());
        client.run(["git", "init", "--no-colocate"]).unwrap();
        client
            .run(["config", "set", "--repo", "fsmonitor.backend", "none"])
            .unwrap();
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
                    fs::write(temporary.path().join(format!("tracked-{index}")), b"file\n")
                        .unwrap();
                }
                snapshot(&client).unwrap();
                let (operation, commit) = current_commit(&client).unwrap();
                let entries = entries(&client, &operation, &commit).unwrap();
                assert_eq!(entries.len(), count);
                fs::write(&log, "").unwrap();
                verify_disk(&client, &operation, &commit, temporary.path(), &entries).unwrap();
                let commands = fs::read_to_string(&log).unwrap();
                assert_eq!(commands.lines().count(), 2, "{commands}");
                assert!(
                    commands.contains("--ignore-working-copy sparse list"),
                    "{commands}"
                );
                assert!(commands.contains("--config=fsmonitor.backend=none --config=snapshot.auto-track=none() --config=snapshot.auto-update-stale=false diff"), "{commands}");
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
        fs::write(&wrapper, "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$JW_VERIFY_PROCESS_LOG\"\nPATH=\"$JW_VERIFY_ORIGINAL_PATH\" exec \"$JW_VERIFY_REAL_JJ\" \"$@\"\n").unwrap();
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
        let path = std::env::join_paths(
            std::iter::once(directory.path().to_path_buf())
                .chain(std::env::split_paths(&original_path)),
        )
        .unwrap();
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
        assert!(verify_reconstructed_disk(&client, &operation, &commit, root, &entries).is_err());
        assert_eq!(current_commit(&client).unwrap(), before);
        fs::write(root.join("tracked"), b"frozen\n").unwrap();
        verify_reconstructed_disk(&client, &operation, &commit, root, &entries).unwrap();
        client.run(["sparse", "reset"]).unwrap();
        client
            .run([
                "config",
                "set",
                "--repo",
                "snapshot.auto-update-stale",
                "true",
            ])
            .unwrap();
        let other = root.join("other");
        client
            .run([
                "workspace",
                "add",
                "--name",
                "other",
                other.to_str().unwrap(),
            ])
            .unwrap();
        JjClient::new(&other)
            .run([
                "--ignore-working-copy",
                "restore",
                "--from",
                "root()",
                "--to",
                &commit,
            ])
            .unwrap();
        fs::write(root.join("tracked"), b"must not snapshot stale tree\n").unwrap();
        let before = current_commit(&client).unwrap();
        assert!(verify_disk(&client, &operation, &commit, root, &entries).is_err());
        assert_eq!(current_commit(&client).unwrap(), before);
        assert_eq!(
            fs::read(root.join("tracked")).unwrap(),
            b"must not snapshot stale tree\n"
        );
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
        let mut diff =
            b"diff --git a/link b/link\nnew file mode 120000\n@@ -0,0 +1,2 @@\n".to_vec();
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
        fs::write(
            directory.path().join("old-directory/secret"),
            "unrecorded\n",
        )
        .unwrap();
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
        assert_eq!(
            fs::read(outside.join("child")).unwrap(),
            b"private external data\n"
        );
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
        let rules = IgnoreRules {
            directory: tempfile::tempdir().unwrap(),
        };
        checked_git(rules.directory.path(), ["init", "--quiet"]).unwrap();
        fs::write(
            rules.directory.path().join(".gitignore"),
            "dependency-link\n",
        )
        .unwrap();
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
        fs::write(
            default.join(".jwlinks.toml"),
            "[[link]]\nsource='private'\ntarget='private'\nrequired=true\n",
        )
        .unwrap();
        preflight_links(&default, &root, &entries, &ignored, false, &rules, &future).unwrap();
        future.replace = true;
        assert!(
            preflight_links(&default, &root, &entries, &ignored, true, &rules, &future).is_err()
        );
        fs::write(
            default.join(".jwlinks.toml"),
            "[[link]]\nsource='dependency-link'\ntarget='private'\nrequired=true\n",
        )
        .unwrap();
        assert!(
            preflight_links(&default, &root, &entries, &ignored, true, &rules, &future).is_err()
        );
        fs::write(
            default.join(".jwlinks.toml"),
            "[[link]]\nsource='dependency-link'\ntarget='cache/fresh'\nrequired=true\n",
        )
        .unwrap();
        preflight_links(&default, &root, &entries, &ignored, true, &rules, &future).unwrap();
        future.replace = false;
        preflight_links(&default, &root, &entries, &ignored, false, &rules, &future).unwrap();
    }

    fn fixture(reset: bool) -> Option<(tempfile::TempDir, Store, Pending)> {
        if Command::new("jj").arg("--version").output().is_err() {
            return None;
        }
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("source");
        let target = temporary.path().join("target");
        let output = Command::new("jj")
            .args(["git", "init", "--no-colocate"])
            .arg(&source)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let client = JjClient::new(&source);
        client
            .run(["config", "set", "--repo", "fsmonitor.backend", "none"])
            .unwrap();
        client
            .run([
                "config",
                "set",
                "--repo",
                "fsmonitor.watchman.register-snapshot-trigger",
                "false",
            ])
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
        let old = target_client
            .resolve_one_at(&operation, "@")
            .unwrap()
            .commit_id;
        let desired_tree = if reset { &source_commit } else { &old };
        let source_entries = entries(&client, &operation, &source_commit).unwrap();
        let target_entries = entries(&target_client, &operation, desired_tree).unwrap();
        let changed = changed_paths(&client, &operation, &source_commit, desired_tree).unwrap();
        let clones = source_entries
            .iter()
            .filter(|e| !changed.contains(&e.path))
            .map(|e| e.path.clone())
            .collect();
        let pending = Pending {
            schema_version: 1,
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
            base_commit: reset.then_some(source_commit.clone()),
            sparse: vec![".".into()],
            clones,
            entries: target_entries,
            old_paths: entries(&target_client, &operation, desired_tree)
                .unwrap()
                .into_iter()
                .map(|e| e.path)
                .collect(),
            ignored: vec!["cache".into()],
            skipped: Vec::new(),
            replace_content: false,
            phase: Phase::Prepared,
            source_stamp: stamp(&source).unwrap(),
            source_payload_digest: payload_digest(&source, &["cache".into()]).unwrap(),
            validated_source_wc_commit: source_commit.clone(),
            target_stamp: stamp(&target).unwrap(),
            config_root: source.clone(),
            link_config: link_config_bytes(&source).unwrap(),
            preparation_id: None,
            history: None,
            execution_diagnostics: Vec::new(),
        };
        let store = Store::lock(&client).unwrap();
        Some((temporary, store, pending))
    }

    #[test]
    fn prepared_handoff_never_reconstructs_even_when_resuming() {
        let Some((_temporary, store, mut plan)) = fixture(true) else {
            return;
        };
        let default = plan.source.clone();
        plan.preparation_id = Some("prepared-handoff".into());
        store.save(&plan).unwrap();
        // Any invocation of reconstruction would fail while processing its
        // first path. Prepared handoffs must use the intact-tree proof instead.
        FAIL_DURING_REPAIR.with(|f| f.set(Some(0)));
        let outcome = execute(&store, &default, &mut plan, true);
        FAIL_DURING_REPAIR.with(|f| f.set(None));
        outcome.unwrap();
        assert_eq!(plan.phase, Phase::Done);
        let target = JjClient::new(&plan.root);
        let (operation, commit) = current_commit(&target).unwrap();
        assert!(exact_empty_child(&target, &operation, &commit, &plan.source_commit).unwrap());
    }

    #[test]
    fn identity_recheck_refuses_replaced_repository_pointer_and_new_git_metadata() {
        let Some((_temporary, _store, plan)) = fixture(false) else {
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
        assert_eq!(
            fs::read(plan.root.join("different")).unwrap(),
            b"feature change\n"
        );
    }

    #[test]
    fn resumes_partial_expansion_without_snapshotting_corrupted_prefill() {
        let Some((_temporary, store, mut plan)) = fixture(false) else {
            return;
        };
        let default = plan.source.clone();
        FAIL_AFTER_EXPANSION.with(|f| f.set(true));
        assert!(execute(&store, &default, &mut plan, false).is_err());
        FAIL_AFTER_EXPANSION.with(|f| f.set(false));
        let target = JjClient::new(&plan.root);
        assert_eq!(current_commit(&target).unwrap().1, plan.old_commit);
        fs::write(plan.root.join("shared"), "partial corruption").unwrap();
        let mut pending = store.pending("worker").unwrap().unwrap();
        assert_eq!(pending.phase, Phase::Expanding);
        execute(&store, &default, &mut pending, true).unwrap();
        assert_eq!(current_commit(&target).unwrap().1, plan.old_commit);
        assert_eq!(
            fs::read(plan.root.join("different")).unwrap(),
            b"feature change\n"
        );
        assert_eq!(
            fs::read(plan.root.join("shared")).unwrap(),
            b"shared baseline\n"
        );
        assert_eq!(fs::read(plan.root.join(".env")).unwrap(), b"retained\n");
        plan.root_identity.verify(&plan.root).unwrap();
        plan.jj_identity.verify(&plan.root.join(".jj")).unwrap();
        assert!(store.pending("worker").unwrap().is_none());
    }

    #[test]
    fn rejected_materialized_tree_snapshot_blocks_resume_and_preserves_recorded_edits() {
        let Some((_temporary, store, mut plan)) = fixture(false) else {
            return;
        };
        // The keep-mode fixture already has the complete desired tracked tree.
        // Model a completed sparse reset whose post-materialization proof rejects
        // a concurrent edit after recording it in JJ.
        plan.phase = Phase::Expanding;
        store.save(&plan).unwrap();
        let pending_path = store.path(&plan.name, "pending");
        let pending_before = fs::read(&pending_path).unwrap();
        let target = JjClient::new(&plan.root);
        let rejected_bytes = b"concurrent edit retained by verification\n";
        fs::write(plan.root.join("shared"), rejected_bytes).unwrap();
        let error = verify_disk(
            &target,
            &plan.target_operation,
            &plan.desired_commit,
            &plan.root,
            &plan.entries,
        )
        .unwrap_err();
        assert!(error.to_string().contains("JJ snapshot performed"));
        let (operation, observed) = current_commit(&target).unwrap();
        assert_ne!(observed, plan.desired_commit);
        assert_eq!(
            file_bytes(&target, &operation, &observed, Path::new("shared")).unwrap(),
            rejected_bytes,
        );

        let default = plan.source.clone();
        let error = execute(&store, &default, &mut plan, true).unwrap_err();
        let message = error.to_string();
        assert!(message.contains(&format!("recorded desired {}", plan.desired_commit)));
        assert!(message.contains(&format!("observed {observed}")));
        assert!(message.contains("verification may have recorded rejected edits"));
        assert!(message.contains("preserve those changes before deliberately restoring"));
        assert_eq!(current_commit(&target).unwrap().1, observed);
        assert_eq!(fs::read(plan.root.join("shared")).unwrap(), rejected_bytes);
        assert_eq!(fs::read(&pending_path).unwrap(), pending_before);
        let pending = store.pending(&plan.name).unwrap().unwrap();
        assert_eq!(pending.phase, Phase::Expanding);
        assert_eq!(pending.desired_commit, plan.desired_commit);
    }

    #[test]
    fn interrupted_new_is_recovered_from_exact_child_without_duplicate_changes() {
        let Some((_temporary, store, mut plan)) = fixture(true) else {
            return;
        };
        let default = plan.source.clone();
        FAIL_AFTER_NEW.with(|f| f.set(true));
        assert!(execute(&store, &default, &mut plan, false).is_err());
        FAIL_AFTER_NEW.with(|f| f.set(false));
        let child = current_commit(&JjClient::new(&plan.root)).unwrap().1;
        assert_ne!(child, plan.old_commit);
        let mut pending = store.pending("worker").unwrap().unwrap();
        assert_eq!(pending.phase, Phase::Newing);
        execute(&store, &default, &mut pending, true).unwrap();
        assert_eq!(pending.desired_commit, child);
        assert_eq!(current_commit(&JjClient::new(&plan.root)).unwrap().1, child);
    }

    fn history_plan(plan: &Pending, disabled: bool) -> TargetHistory {
        let client = JjClient::new(&plan.root);
        let mut names = crate::freshen::history::BookmarkNames::at(
            &client,
            &plan.target_operation,
            jiff::Timestamp::now(),
        )
        .unwrap();
        TargetHistory::plan(
            &client,
            &plan.target_operation,
            &plan.old_commit,
            &plan.source_commit,
            disabled,
            &mut names,
        )
        .unwrap()
    }

    #[test]
    fn frozen_history_handoff_survives_lost_bookmark_success_and_advances_without_rebase() {
        let Some((_temporary, store, mut plan)) = fixture(true) else {
            return;
        };
        let default = plan.source.clone();
        let target = JjClient::new(&plan.root);
        plan.history = Some(history_plan(&plan, false));
        plan.preparation_id = Some("history-batch".into());
        target.create_bookmark("feature", &plan.old_commit).unwrap();
        let original = plan.clone();
        // The existing atomic batch handoff carries the immutable name+tip.
        prepare::handoff(&store, &original, false).unwrap();
        FAIL_AFTER_BOOKMARK.with(|f| f.set(true));
        let error = execute(&store, &default, &mut plan, true).unwrap_err();
        FAIL_AFTER_BOOKMARK.with(|f| f.set(false));
        assert!(error.to_string().contains("after freshen bookmark"));
        let mut pending = store.pending("worker").unwrap().unwrap();
        assert_eq!(pending.phase, Phase::Prepared);
        assert_eq!(pending.history, original.history);
        assert_eq!(current_commit(&target).unwrap().1, plan.old_commit);
        assert_eq!(
            fs::read(plan.root.join("different")).unwrap(),
            b"feature change\n"
        );
        // Handoff comparison must include the preservation pair, not only batch ID.
        let mut changed = pending.clone();
        changed
            .history
            .as_mut()
            .unwrap()
            .bookmark
            .as_mut()
            .unwrap()
            .name
            .push_str("-foreign");
        store.save(&changed).unwrap();
        assert!(prepare::handoff(&store, &original, false).is_err());
        store.save(&pending).unwrap();
        execute(&store, &default, &mut pending, true).unwrap();
        let operation = target.operation_id().unwrap();
        let refs = target.bookmark_refs_at(&operation).unwrap();
        let preservation = pending.history.as_ref().unwrap().bookmark.as_ref().unwrap();
        assert_eq!(
            refs.iter()
                .filter(|r| r.remote.is_none() && r.name.starts_with("freshen-"))
                .count(),
            1
        );
        for name in [preservation.name.as_str(), "feature"] {
            assert_eq!(
                refs.iter()
                    .find(|r| r.remote.is_none() && r.name == name)
                    .unwrap()
                    .normal_target(),
                Some(plan.old_commit.as_str())
            );
        }
        assert!(
            exact_empty_child(
                &target,
                &operation,
                &pending.desired_commit,
                &plan.source_commit
            )
            .unwrap()
        );
        assert_eq!(
            fs::read(plan.root.join("different")).unwrap(),
            b"source baseline\n"
        );
        assert_eq!(
            file_bytes(
                &target,
                &operation,
                &preservation.tip,
                Path::new("different")
            )
            .unwrap(),
            b"feature change\n"
        );
    }

    #[test]
    fn changed_preservation_bookmark_blocks_later_phase_repair_without_snapshot() {
        let Some((_temporary, store, mut plan)) = fixture(true) else {
            return;
        };
        let default = plan.source.clone();
        let target = JjClient::new(&plan.root);
        plan.history = Some(history_plan(&plan, false));
        FAIL_AT.with(|f| f.set(Some(Phase::Filling)));
        assert!(execute(&store, &default, &mut plan, false).is_err());
        FAIL_AT.with(|f| f.set(None));
        let mut pending = store.pending("worker").unwrap().unwrap();
        assert_eq!(pending.phase, Phase::Filling);
        let bookmark = pending
            .history
            .as_ref()
            .unwrap()
            .bookmark
            .as_ref()
            .unwrap()
            .clone();
        // External, deliberate movement during interruption. Resume may not undo
        // it, allocate a second name, repair files, or snapshot partial contents.
        target
            .run([
                "--ignore-working-copy",
                "bookmark",
                "set",
                &bookmark.name,
                "-r",
                &plan.source_commit,
                "--allow-backwards",
            ])
            .unwrap();
        fs::write(plan.root.join("different"), b"partial bytes\n").unwrap();
        let before = current_commit(&target).unwrap();
        let error = execute(&store, &default, &mut pending, true).unwrap_err();
        assert!(error.to_string().contains("another target"), "{error:#}");
        assert_eq!(current_commit(&target).unwrap(), before);
        assert_eq!(
            fs::read(plan.root.join("different")).unwrap(),
            b"partial bytes\n"
        );
        assert_eq!(
            store.pending("worker").unwrap().unwrap().history,
            plan.history
        );
    }

    #[test]
    fn history_opt_out_still_leaves_exact_prior_commit_recoverable() {
        let Some((_temporary, store, mut plan)) = fixture(true) else {
            return;
        };
        let default = plan.source.clone();
        let target = JjClient::new(&plan.root);
        plan.history = Some(history_plan(&plan, true));
        assert!(!plan.history.as_ref().unwrap().published);
        execute(&store, &default, &mut plan, false).unwrap();
        let operation = target.operation_id().unwrap();
        assert!(
            !target
                .bookmark_refs_at(&operation)
                .unwrap()
                .iter()
                .any(|r| r.name.starts_with("freshen-"))
        );
        assert!(
            exact_empty_child(
                &target,
                &operation,
                &plan.desired_commit,
                &plan.source_commit
            )
            .unwrap()
        );
        assert_eq!(
            file_bytes(
                &target,
                &operation,
                &plan.old_commit,
                Path::new("different")
            )
            .unwrap(),
            b"feature change\n"
        );
    }

    #[test]
    fn every_write_ahead_boundary_retains_a_resumable_plan() {
        for phase in [
            Phase::Clearing,
            Phase::Empty,
            Phase::Newing,
            Phase::Filling,
            Phase::Expanding,
            Phase::Verified,
            Phase::CopyingIgnored,
            Phase::Done,
        ] {
            let Some((_temporary, store, mut plan)) = fixture(true) else {
                return;
            };
            let default = plan.source.clone();
            FAIL_AT.with(|f| f.set(Some(phase)));
            assert!(
                execute(&store, &default, &mut plan, false).is_err(),
                "{phase:?}"
            );
            FAIL_AT.with(|f| f.set(None));
            let mut pending = store.pending("worker").unwrap().unwrap();
            execute(&store, &default, &mut pending, true).unwrap();
            pending.root_identity.verify(&pending.root).unwrap();
            pending
                .jj_identity
                .verify(&pending.root.join(".jj"))
                .unwrap();
            assert!(store.pending("worker").unwrap().is_none());
        }
    }

    #[test]
    fn ignore_rules_use_desired_nested_files_and_backend_excludes() {
        let Some((_temporary, _store, plan)) = fixture(false) else {
            return;
        };
        let client = JjClient::new(&plan.source);
        fs::create_dir_all(plan.source.join("packages/p")).unwrap();
        fs::write(plan.source.join("packages/p/.gitignore"), "node_modules/\n").unwrap();
        let backend = PathBuf::from(
            client
                .run(["--ignore-working-copy", "git", "root"])
                .unwrap()
                .trimmed_stdout()
                .unwrap(),
        );
        fs::create_dir_all(backend.join("info")).unwrap();
        fs::write(backend.join("info/exclude"), "backend-cache/\n").unwrap();
        snapshot(&client).unwrap();
        let (operation, commit) = current_commit(&client).unwrap();
        let entries = entries(&client, &operation, &commit).unwrap();
        let rules = IgnoreRules::new(&client, &operation, &commit, &entries, true).unwrap();
        rules
            .require_ignored(Path::new("packages/p/node_modules"))
            .unwrap();
        rules.require_ignored(Path::new("backend-cache")).unwrap();
        assert!(rules.require_ignored(Path::new("unignored")).is_err());
        assert!(rules.require_ignored_file(Path::new("cache")).is_err());
    }

    #[test]
    fn adversarial_git_template_cannot_escape_ignore_probe() {
        if std::env::var_os("JW_RESEED_TEMPLATE_CHILD").is_some() {
            let Some((_temporary, _store, plan)) = fixture(false) else {
                return;
            };
            let client = JjClient::new(&plan.source);
            let entries = entries(&client, &plan.source_operation, &plan.source_commit).unwrap();
            let rules = IgnoreRules::new(
                &client,
                &plan.source_operation,
                &plan.source_commit,
                &entries,
                true,
            )
            .unwrap();
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
    fn prepared_resume_refuses_new_unsnapshotted_target_edits() {
        let Some((_temporary, store, mut plan)) = fixture(false) else {
            return;
        };
        let default = plan.source.clone();
        store.save(&plan).unwrap();
        fs::write(plan.root.join("shared"), "new user work\n").unwrap();
        let error = execute(&store, &default, &mut plan, true).unwrap_err();
        assert!(format!("{error:#}").contains("target contents changed after pending preparation"));
        assert_eq!(
            fs::read(plan.root.join("shared")).unwrap(),
            b"new user work\n"
        );
        assert_eq!(
            store.pending("worker").unwrap().unwrap().phase,
            Phase::Prepared
        );
    }

    #[test]
    fn native_unmanaged_workspaces_initialize_only_repository_manifest() {
        let Some((_temporary, store, mut plan)) = fixture(false) else {
            return;
        };
        let client = JjClient::new(&plan.source);
        let metadata =
            WorkspaceMetadataStore::from_repo_config_path(client.repo_config_path().unwrap())
                .unwrap();
        assert!(metadata.list().unwrap().is_empty());
        assert!(metadata.get("worker").unwrap().is_none());
        validate_target(
            &client,
            &metadata,
            &plan.source,
            &plan.source,
            "worker",
            &plan.root,
        )
        .unwrap();
        let default = plan.source.clone();
        execute(&store, &default, &mut plan, false).unwrap();
        assert!(metadata.get("worker").unwrap().is_none());
        assert!(metadata.list().unwrap().is_empty());
        assert!(metadata.root().join("manifest.json").is_file());
        assert!(
            fs::read_dir(metadata.root().join("workspaces"))
                .unwrap()
                .next()
                .is_none()
        );
    }

    #[test]
    fn native_reset_type_transitions_preserve_unrelated_and_external_secrets() {
        let Some((temporary, store, mut plan)) = fixture(false) else {
            return;
        };
        let source = JjClient::new(&plan.source);
        let target = JjClient::new(&plan.root);
        let outside = temporary.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("child"), b"private external data\n").unwrap();
        fs::create_dir(plan.root.join("directory-to-file")).unwrap();
        fs::write(
            plan.root.join("directory-to-file/old-child"),
            b"old tracked data\n",
        )
        .unwrap();
        std::os::unix::fs::symlink(&outside, plan.root.join("symlink-to-directory")).unwrap();
        snapshot(&target).unwrap();
        source.run(["new"]).unwrap();
        fs::write(plan.source.join("directory-to-file"), b"new file\n").unwrap();
        fs::create_dir(plan.source.join("symlink-to-directory")).unwrap();
        fs::write(
            plan.source.join("symlink-to-directory/child"),
            b"new child\n",
        )
        .unwrap();
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
        plan.validated_source_wc_commit = source_commit.clone();
        plan.base_commit = Some(source_commit);
        plan.old_commit = old_commit.clone();
        plan.desired_commit = old_commit;
        plan.clones = desired_entries.iter().map(|e| e.path.clone()).collect();
        plan.entries = desired_entries;
        plan.old_paths = old_entries.into_iter().map(|e| e.path).collect();
        plan.source_stamp = stamp(&plan.source).unwrap();
        plan.target_stamp = stamp(&plan.root).unwrap();
        let default = plan.source.clone();
        execute(&store, &default, &mut plan, false).unwrap();
        assert_eq!(
            fs::read(plan.root.join("directory-to-file")).unwrap(),
            b"new file\n"
        );
        assert_eq!(
            fs::read(plan.root.join("symlink-to-directory/child")).unwrap(),
            b"new child\n"
        );
        assert_eq!(fs::read(plan.root.join(".env")).unwrap(), b"retained\n");
        assert_eq!(
            fs::read(outside.join("child")).unwrap(),
            b"private external data\n"
        );
        plan.root_identity.verify(&plan.root).unwrap();
        plan.jj_identity.verify(&plan.root.join(".jj")).unwrap();
    }

    #[test]
    fn reseed_store_never_repairs_corrupt_or_nonempty_manifestless_metadata() {
        let Some((_temporary, _store, plan)) = fixture(false) else {
            return;
        };
        let client = JjClient::new(&plan.source);
        let metadata =
            WorkspaceMetadataStore::from_repo_config_path(client.repo_config_path().unwrap())
                .unwrap();
        let manifest = metadata.root().join("manifest.json");
        fs::write(&manifest, b"not valid json\n").unwrap();
        assert!(Store::lock(&client).is_err());
        assert_eq!(fs::read(&manifest).unwrap(), b"not valid json\n");
        fs::remove_file(&manifest).unwrap();
        assert!(Store::lock(&client).is_err());
        assert!(!manifest.exists());
    }

    #[test]
    fn restored_ignored_bytes_can_resume_despite_new_inode_and_ctime() {
        let Some((_temporary, store, mut plan)) = fixture(false) else {
            return;
        };
        let default = plan.source.clone();
        FAIL_AFTER_EXPANSION.with(|f| f.set(true));
        assert!(execute(&store, &default, &mut plan, false).is_err());
        FAIL_AFTER_EXPANSION.with(|f| f.set(false));
        let dependency = plan.source.join("cache/dependency");
        fs::write(&dependency, "changed dependencies\n").unwrap();
        let mut pending = store.pending("worker").unwrap().unwrap();
        assert!(execute(&store, &default, &mut pending, true).is_err());
        fs::remove_file(&dependency).unwrap();
        fs::write(&dependency, b"fresh dependency\n").unwrap();
        execute(&store, &default, &mut pending, true).unwrap();
        assert_eq!(
            fs::read(pending.root.join("cache/dependency")).unwrap(),
            b"fresh dependency\n"
        );
        assert!(store.pending("worker").unwrap().is_none());
    }

    #[test]
    fn unchanged_verified_source_reuses_proof_but_changes_require_logical_restoration() {
        let Some((_temporary, _store, mut plan)) = fixture(false) else {
            return;
        };
        SOURCE_LOGICAL_CHECKS.with(|count| count.set(0));
        validate_source(&mut plan, false).unwrap();
        assert_eq!(SOURCE_LOGICAL_CHECKS.with(std::cell::Cell::get), 0);
        fs::write(plan.source.join("shared"), b"changed tracked bytes\n").unwrap();
        assert!(validate_source(&mut plan, false).is_err());
        assert!(validate_source(&mut plan, true).is_err());
        assert_eq!(SOURCE_LOGICAL_CHECKS.with(std::cell::Cell::get), 1);
        fs::write(plan.source.join("shared"), b"shared baseline\n").unwrap();
        validate_source(&mut plan, true).unwrap();
        assert_eq!(SOURCE_LOGICAL_CHECKS.with(std::cell::Cell::get), 2);
        validate_source(&mut plan, false).unwrap();
        assert_eq!(SOURCE_LOGICAL_CHECKS.with(std::cell::Cell::get), 2);
        fs::write(
            plan.source.join("cache/dependency"),
            b"changed ignored dependency\n",
        )
        .unwrap();
        assert!(validate_source(&mut plan, false).is_err());
        assert!(validate_source(&mut plan, true).is_err());
        assert_eq!(SOURCE_LOGICAL_CHECKS.with(std::cell::Cell::get), 3);
        fs::write(plan.source.join("cache/dependency"), b"fresh dependency\n").unwrap();
        validate_source(&mut plan, true).unwrap();
        assert_eq!(SOURCE_LOGICAL_CHECKS.with(std::cell::Cell::get), 4);
        validate_source(&mut plan, false).unwrap();
        assert_eq!(SOURCE_LOGICAL_CHECKS.with(std::cell::Cell::get), 4);
    }

    #[test]
    fn verified_source_fast_path_still_rechecks_backend_ignored_rules() {
        let Some((_temporary, _store, mut plan)) = fixture(false) else {
            return;
        };
        let source = JjClient::new(&plan.source);
        source.run(["new"]).unwrap();
        let selected = PathBuf::from(format!("jw-private-cache-{}", std::process::id()));
        fs::rename(plan.source.join("cache"), plan.source.join(&selected)).unwrap();
        fs::write(plan.source.join(".gitignore"), ".env\n").unwrap();
        let backend = PathBuf::from(
            source
                .run(["--ignore-working-copy", "git", "root"])
                .unwrap()
                .trimmed_stdout()
                .unwrap(),
        );
        fs::create_dir_all(backend.join("info")).unwrap();
        let exclude = backend.join("info/exclude");
        fs::write(&exclude, format!("{}/\n", selected.display())).unwrap();
        snapshot(&source).unwrap();
        let (operation, commit) = current_commit(&source).unwrap();
        let source_entries = entries(&source, &operation, &commit).unwrap();
        verify_disk(&source, &operation, &commit, &plan.source, &source_entries).unwrap();
        plan.source_operation = operation;
        plan.source_commit = commit.clone();
        plan.source_wc_commit = commit.clone();
        plan.validated_source_wc_commit = commit;
        plan.ignored = vec![selected];
        plan.source_payload_digest = payload_digest(&plan.source, &plan.ignored).unwrap();
        plan.source_stamp = stamp(&plan.source).unwrap();
        SOURCE_LOGICAL_CHECKS.with(|count| count.set(0));
        validate_source(&mut plan, false).unwrap();
        fs::write(exclude, "").unwrap();
        assert_eq!(stamp(&plan.source).unwrap(), plan.source_stamp);
        assert!(validate_source(&mut plan, false).is_err());
        assert_eq!(SOURCE_LOGICAL_CHECKS.with(std::cell::Cell::get), 0);
    }

    #[test]
    fn interruption_during_reconstruction_never_snapshots_partial_repair() {
        let Some((_temporary, store, mut plan)) = fixture(false) else {
            return;
        };
        let default = plan.source.clone();
        FAIL_AFTER_EXPANSION.with(|f| f.set(true));
        assert!(execute(&store, &default, &mut plan, false).is_err());
        FAIL_AFTER_EXPANSION.with(|f| f.set(false));
        let target = JjClient::new(&plan.root);
        let mut pending = store.pending("worker").unwrap().unwrap();
        FAIL_DURING_REPAIR.with(|f| f.set(Some(1)));
        assert!(execute(&store, &default, &mut pending, true).is_err());
        FAIL_DURING_REPAIR.with(|f| f.set(None));
        assert_eq!(current_commit(&target).unwrap().1, plan.old_commit);
        let mut pending = store.pending("worker").unwrap().unwrap();
        assert_eq!(pending.phase, Phase::Clearing);
        execute(&store, &default, &mut pending, true).unwrap();
        assert_eq!(current_commit(&target).unwrap().1, plan.old_commit);
    }
}
