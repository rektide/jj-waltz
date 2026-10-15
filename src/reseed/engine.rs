use super::selection::{self, Group, Registrations, component_glob};
use super::{ReseedOutcome, ReseedReport, ReseedRequest, ReseedStatus};
use crate::{cow, jj::JjClient, links, metadata::WorkspaceMetadataStore};
use anyhow::{Context, Result, bail};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

#[path = "prepare.rs"]
mod prepare;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalConfig {
    source: Option<PathBuf>,
    source_at: Option<String>,
    #[serde(default)]
    copy_ignored: Vec<String>,
    #[serde(default)]
    groups: BTreeMap<String, Group>,
    prepare: Option<prepare::Settings>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    device: u64,
    inode: u64,
}
impl Identity {
    fn read(path: &Path) -> Result<Self> {
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_dir() {
            bail!("identity is not a real directory: {}", path.display());
        }
        Ok(Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }
    fn verify(&self, path: &Path) -> Result<()> {
        if &Self::read(path)? != self {
            bail!("directory identity changed: {}", path.display());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: PathBuf,
    kind: String,
    executable: bool,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Stamp {
    path: PathBuf,
    device: u64,
    inode: u64,
    size: u64,
    mode: u32,
    mtime: i64,
    mtime_ns: i64,
    ctime: i64,
    ctime_ns: i64,
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
            .context("another jw reseed is running; pause other writers before retrying")?;
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
    fn finish(&self, pending: &Pending) -> Result<()> {
        atomic_json(&self.path(&pending.name, "receipt"), pending)?;
        fs::remove_file(self.path(&pending.name, "pending"))?;
        File::open(&self.root)?.sync_all()?;
        Ok(())
    }
}

fn atomic_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let parent = path.parent().context("record has no parent")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer_pretty(&mut temporary, value)?;
    temporary.write_all(b"\n")?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|e| e.error)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

pub(super) fn run(
    mut request: ReseedRequest,
    diagnostic: &mut dyn FnMut(&str),
) -> Result<ReseedReport> {
    if request.freshen && request.resume {
        bail!("--freshen and --resume conflict; resume uses recorded preparation only");
    }
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
    let (mut config, mut preparation_config) = if request.resume && request.groups.is_empty() {
        (LocalConfig::default(), None)
    } else {
        load_config(&default)?
    };
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let selection = selection::resolve(
        &request,
        &config.groups,
        &inventory.registered,
        &default,
        home.as_deref(),
    )?;
    if let Some(warning) = selection.warning() {
        diagnostic(&warning);
    }
    selection.validate()?;
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
        prepare::find_resume(&store, &names)?
    } else {
        None
    };
    let recovering_preparation = batch.as_ref().is_some_and(|b| b.plans.is_none());
    if recovering_preparation {
        let record = batch.as_ref().unwrap();
        prepare::validate_resume_request(&request)?;
        if names != record.request.names {
            bail!(
                "preparation resume requires all recorded literal target names: {:?}",
                record.request.names
            );
        }
        let dry_run = request.dry_run;
        request = record.request.clone();
        request.dry_run = dry_run;
        (config, preparation_config) = load_config(&default)?;
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
    } else if request.resume {
        if request.source.is_some()
            || request.source_at.is_some()
            || request.at.is_some()
            || !request.copy_ignored.is_empty()
            || request.no_copy_ignored
            || request.replace_content
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
            if store.pending(name)?.is_some() {
                bail!(
                    "workspace {name} has a pending reseed; use `jw reseed --resume {name}` before snapshotting it"
                );
            }
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
        for (name, root) in &inventory.roots {
            if root == &source && store.pending(name)?.is_some() {
                bail!("source has a pending reseed; resume it before using it as a source");
            }
        }
        let source_rev = request
            .source_at
            .as_ref()
            .or(config.source_at.as_ref())
            .map(String::as_str)
            .context("reseed requires --source-at or source_at in .jwreseed.local.toml")?;
        ensure_quiescence(&source_client)?;
        require_full(&source_client)?;
        validate_contents(&source, true)?;
        let mut roots = BTreeSet::new();
        for name in &names {
            let root = inventory.root(name)?.canonicalize()?;
            if !roots.insert(root.clone()) {
                bail!("multiple selected workspaces alias {}", root.display());
            }
            validate_target(&client, &metadata, &default, &source, name, &root)?;
            ensure_quiescence(&JjClient::new(&root))?;
            require_full(&JjClient::new(&root))?;
            validate_contents(&root, false)?;
            if request.freshen {
                cow::probe_between(&source, &root)?;
            }
        }
        if request.freshen {
            let ignored_patterns = if request.no_copy_ignored {
                &[][..]
            } else if request.copy_ignored.is_empty() {
                config.copy_ignored.as_slice()
            } else {
                request.copy_ignored.as_slice()
            };
            prepare::verify_policy(&default, &preparation_config)?;
            prepare::preflight(
                &store,
                &metadata,
                &inventory,
                &source,
                ignored_patterns,
                config.prepare.as_ref(),
            )?;
            if batch.is_none() {
                snapshot(&source_client)?;
                for root in &roots {
                    snapshot(&JjClient::new(root))?;
                }
                if request.dry_run {
                    let summary = prepare::describe(
                        &source,
                        source_rev,
                        config.prepare.as_ref(),
                        ignored_patterns,
                    )?;
                    diagnostic(&summary);
                    return Ok(ReseedReport {
                        dry_run: true, source, source_commit_id: None, preparation_summary: Some(summary),
                        targets: names.iter().map(|name| ReseedOutcome { name: name.clone(), status: ReseedStatus::Planned, message: format!("{}; ignored selection {:?}; no fetch/new/preparation/refill performed; real work snapshotted", request.at.as_ref().map(|at| format!("reset at prospective {at}")).unwrap_or_else(|| "keep snapshotted target revision".into()), ignored_patterns) }).collect(),
                        selections: selection.selections, skipped: selection.skipped,
                    });
                }
                batch = Some(prepare::begin(
                    &store,
                    &client,
                    &source,
                    &default,
                    &inventory,
                    &names,
                    &request,
                    source_rev,
                    config.prepare.as_ref(),
                    ignored_patterns,
                    preparation_config.clone(),
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
                        "Resume recorded preparation phase {:?}, source goal {:?}, exact goal {:?}; no preparation actions performed",
                        b.phase, b.source_rev, b.goal
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
            None => {
                source_client
                    .resolve_one_at(&source_operation, source_rev)?
                    .commit_id
            }
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
        // Full scans freeze dirty target work only after donor validation.
        if let Some(b) = batch.as_ref() {
            prepare::verify_targets(b)?;
        } else {
            for root in &roots {
                snapshot(&JjClient::new(root))?;
            }
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
        let base = request
            .at
            .as_ref()
            .map(|rev| client.resolve_one_at(&operation, rev).map(|r| r.commit_id))
            .transpose()?;
        let ignored_patterns = if request.no_copy_ignored {
            &[][..]
        } else if request.copy_ignored.is_empty() {
            config.copy_ignored.as_slice()
        } else {
            request.copy_ignored.as_slice()
        };
        let (ignored, skipped) = expand_ignored(&source, ignored_patterns)?;
        let source_payload_digest = payload_digest(&source, &ignored)?;
        let source_rules = IgnoreRules::new(
            &source_client,
            &source_operation,
            &source_commit,
            &source_entries,
            true,
        )?;
        for path in &ignored {
            source_rules.require_ignored(path)?;
            cow::validate_tree(&source.join(path))?;
        }
        if stamp(&source)? != source_stamp || current_commit(&source_client)?.1 != source_wc {
            bail!("source changed during initial verified capture; pause writers before retrying");
        }
        let facts = client.workspace_target_facts_at(&operation)?;
        for name in &names {
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
                if source_entries
                    .iter()
                    .chain(&target_entries)
                    .any(|e| overlap(path, &e.path))
                {
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
            for entry in target_entries.iter().filter(|e| e.kind == "symlink") {
                symlink_bytes(&client, &operation, tree_commit, &entry.path)?;
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
            plans.push(Pending {
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
            });
        }
    }
    if let Some(b) = batch.as_mut() {
        if b.plans.is_none() {
            b.plans = Some(plans.clone());
            prepare::save(&store, b)?;
            for plan in &plans {
                prepare::handoff(&store, plan, request.dry_run)?;
            }
        }
    }
    let first = plans
        .first()
        .or_else(|| {
            batch
                .as_ref()
                .and_then(|b| b.plans.as_ref())
                .and_then(|p| p.first())
        })
        .context("select at least one workspace")?;
    let mut report = ReseedReport {
        dry_run: request.dry_run,
        source: first.source.clone(),
        source_commit_id: Some(first.source_commit.clone()),
        preparation_summary: batch.as_ref().map(prepare::summary),
        targets: completed,
        selections: selection.selections,
        skipped: selection.skipped,
    };
    for mut plan in plans {
        if request.dry_run {
            report.targets.push(ReseedOutcome {
                name: plan.name.clone(),
                status: ReseedStatus::Planned,
                message: describe(&plan, true),
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
    if let Some(b) = batch.as_ref() {
        if !request.dry_run {
            prepare::finish_if_complete(&store, b)?;
        }
    }
    Ok(report)
}

struct Inventory {
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
        let context = crate::context::discover(Some(client.cwd()));
        let expected_repository = repository(client)?;
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
        let primary = crate::context::discover(Some(default));
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
    context
        .jj
        .repository_path
        .context("cannot verify JJ repository identity")?
        .canonicalize()
        .context("cannot resolve JJ repository identity")
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
        Ok(bytes) => Ok((
            toml::from_str(std::str::from_utf8(&bytes)?)
                .with_context(|| format!("invalid {}", path.display()))?,
            Some(bytes),
        )),
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
    let context = crate::context::discover(Some(root));
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
    let output = client.run(["--config=fsmonitor.backend=none", "status"])?;
    diagnostics(output.stderr());
    Ok(())
}

fn verified_snapshot(client: &JjClient) -> Result<()> {
    // Retained ignored data can become unignored after --at changes .gitignore.
    // Reseed is not permission to auto-track it into the frozen desired commit.
    let output = client.run([
        "--config=fsmonitor.backend=none",
        "--config=snapshot.auto-track=none()",
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

fn relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(part) if part != ".jj" && part != ".git"))
    {
        bail!("unsafe repository-relative path: {}", path.display());
    }
    Ok(())
}

fn entries(client: &JjClient, operation: &str, commit: &str) -> Result<Vec<Entry>> {
    let output = client
        .run_at(
            operation,
            [
                "file",
                "list",
                "-r",
                commit,
                "-T",
                r#"json(path) ++ "\t" ++ json(file_type) ++ "\t" ++ json(executable) ++ "\n""#,
            ],
        )
        .context("JJ lacks reseed JSON tree-entry template support; upgrade JJ")?;
    let mut entries = Vec::new();
    for line in output.stdout()?.lines() {
        let fields = line.split('\t').collect::<Vec<_>>();
        let [path, kind, executable] = fields.as_slice() else {
            bail!("malformed JJ tree entry");
        };
        let entry = Entry {
            path: PathBuf::from(serde_json::from_str::<String>(path)?),
            kind: serde_json::from_str(kind)?,
            executable: serde_json::from_str(executable)?,
        };
        relative(&entry.path)?;
        if !matches!(entry.kind.as_str(), "file" | "symlink") {
            bail!(
                "unsupported tracked entry {} ({})",
                entry.path.display(),
                entry.kind
            );
        }
        entries.push(entry);
    }
    Ok(entries)
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

fn file_bytes(client: &JjClient, operation: &str, commit: &str, path: &Path) -> Result<Vec<u8>> {
    // Filesets are expressions, not literal paths. `file:` with a JSON-quoted
    // string preserves newlines, spaces, quotes and names resembling selectors.
    let fileset = format!(
        "file:{}",
        serde_json::to_string(path.to_str().context("tracked path not UTF-8")?)?
    );
    let output = client.run_at(operation, ["file", "show", "-r", commit, &fileset])?;
    diagnostics(output.stderr());
    Ok(output.stdout_bytes().to_vec())
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

fn verify_disk(
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

fn confined_parents(root: &Path, path: &Path) -> Result<()> {
    relative(path)?;
    let mut parent = root.to_path_buf();
    for component in path.parent().unwrap_or(Path::new("")).components() {
        parent.push(component);
        match fs::symlink_metadata(&parent) {
            Ok(m) if m.is_dir() => {}
            Ok(_) => bail!("path parent is not a real directory: {}", parent.display()),
            Err(e) if e.kind() == ErrorKind::NotFound => break,
            Err(e) => return Err(e.into()),
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

fn stamp(root: &Path) -> Result<Vec<Stamp>> {
    fn visit(root: &Path, path: &Path, stamps: &mut Vec<Stamp>) -> Result<()> {
        let mut children = fs::read_dir(path)?.collect::<std::io::Result<Vec<_>>>()?;
        children.sort_by_key(|c| c.file_name());
        for child in children {
            let path = child.path();
            if path.parent() == Some(root)
                && [".jj", ".git"].iter().any(|n| child.file_name() == *n)
            {
                continue;
            }
            let m = fs::symlink_metadata(&path)?;
            stamps.push(Stamp {
                path: path.strip_prefix(root)?.to_path_buf(),
                device: m.dev(),
                inode: m.ino(),
                size: m.len(),
                mode: m.mode(),
                mtime: m.mtime(),
                mtime_ns: m.mtime_nsec(),
                ctime: m.ctime(),
                ctime_ns: m.ctime_nsec(),
            });
            if m.is_dir() {
                visit(root, &path, stamps)?;
            }
        }
        Ok(())
    }
    let mut stamps = Vec::new();
    visit(root, root, &mut stamps)?;
    Ok(stamps)
}

/// Logical dependency snapshot identity survives inode/ctime changes when an
/// operator restores the frozen source. Administrative and special files never
/// become digest input (the same strict validation precedes hashing and copying).
fn payload_digest(root: &Path, paths: &[PathBuf]) -> Result<String> {
    fn visit(root: &Path, relative_path: &Path, hash: &mut Sha256) -> Result<()> {
        use std::os::unix::ffi::OsStrExt;
        relative(relative_path)?;
        confined_parents(root, relative_path)?;
        let path = root.join(relative_path);
        let m = fs::symlink_metadata(&path)?;
        let name = relative_path.as_os_str().as_bytes();
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name);
        hash.update((m.mode() & 0o7777).to_le_bytes());
        if m.is_dir() {
            hash.update(b"D");
            let mut children = fs::read_dir(&path)?.collect::<std::io::Result<Vec<_>>>()?;
            children.sort_by_key(|e| e.file_name());
            hash.update((children.len() as u64).to_le_bytes());
            for child in children {
                visit(root, &relative_path.join(child.file_name()), hash)?;
            }
        } else if m.is_file() {
            hash.update(b"F");
            hash.update(m.len().to_le_bytes());
            let mut file = File::open(path)?;
            let mut buffer = [0; 65536];
            loop {
                let count = file.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                hash.update(&buffer[..count]);
            }
        } else if m.file_type().is_symlink() {
            hash.update(b"L");
            let target = fs::read_link(path)?;
            let target = target.as_os_str().as_bytes();
            hash.update((target.len() as u64).to_le_bytes());
            hash.update(target);
        } else {
            bail!("unsupported dependency entry: {}", path.display());
        }
        Ok(())
    }
    let mut hash = Sha256::new();
    for path in paths {
        cow::validate_tree(&root.join(path))?;
        visit(root, path, &mut hash)?;
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
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
    for entry in desired {
        // An old tracked path is removed by sparse clearing. Everything else
        // belongs to the operator and cannot silently be overwritten by checkout.
        let mut path = PathBuf::new();
        for component in entry.path.components() {
            path.push(component);
            match fs::symlink_metadata(root.join(&path)) {
                Ok(m) if path == entry.path || !m.is_dir() => {
                    if m.is_dir() && tracked_only_directory(root, &path, old)? {
                        continue;
                    }
                    if !old.iter().any(|e| e.path == path) {
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

fn tracked_only_directory(root: &Path, path: &Path, old: &[Entry]) -> Result<bool> {
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
        } else if !old.iter().any(|e| e.path == path) {
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
    let clone_bytes: u64 = plan
        .source_stamp
        .iter()
        .filter(|s| plan.clones.contains(&s.path) && s.mode & 0o170000 == 0o100000)
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
    format!(
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
        if plan.skipped.is_empty() {
            String::new()
        } else {
            format!("; {}", plan.skipped.join("; "))
        }
    )
}

/// Git's native ignore parser, over an isolated tree of frozen desired .gitignore
/// files and copies of JJ's backend/global excludes. No target revision is changed.
struct IgnoreRules {
    directory: tempfile::TempDir,
}
impl IgnoreRules {
    fn new(
        client: &JjClient,
        operation: &str,
        commit: &str,
        entries: &[Entry],
        retain_untracked: bool,
    ) -> Result<Self> {
        let directory = tempfile::tempdir()?;
        checked_git(directory.path(), ["init", "--quiet", "--template="])?;
        fs::create_dir_all(directory.path().join(".git/info"))?;
        let mut ignores = BTreeMap::new();
        // On-disk untracked nested ignores are part of a retained checkout's
        // semantics; tracked files are subsequently overridden by the desired tree.
        let old_commit = client.resolve_one_at(operation, "@")?.commit_id;
        let old_entries = self::entries(client, operation, &old_commit)?;
        for item in if retain_untracked {
            stamp(client.cwd())?
        } else {
            Vec::new()
        } {
            if item.path.file_name().is_some_and(|n| n == ".gitignore")
                && !entries
                    .iter()
                    .chain(&old_entries)
                    .any(|e| e.path == item.path)
            {
                let path = client.cwd().join(&item.path);
                if fs::symlink_metadata(&path)?.is_file() {
                    ignores.insert(item.path, fs::read(path)?);
                }
            }
        }
        for entry in entries
            .iter()
            .filter(|e| e.path.file_name().is_some_and(|n| n == ".gitignore"))
        {
            if entry.kind != "file" {
                bail!("symlinked ignore files cannot be verified safely");
            }
            ignores.insert(
                entry.path.clone(),
                file_bytes(client, operation, commit, &entry.path)?,
            );
        }
        for (path, bytes) in ignores {
            let path = directory.path().join(path);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(path, bytes)?;
        }
        let backend = client.run_at_unchecked(operation, ["git", "root"])?;
        let backend = if backend.success() {
            Some(PathBuf::from(backend.trimmed_stdout()?).canonicalize()?)
        } else {
            None
        };
        let mut command = git_command(client.cwd());
        if let Some(backend) = &backend {
            command.arg("--git-dir").arg(backend);
        }
        let output = command
            .args(["config", "--path", "--get", "core.excludesFile"])
            .output()?;
        let exclude = if output.status.success() {
            let path =
                PathBuf::from(std::str::from_utf8(&output.stdout)?.trim_end_matches(['\r', '\n']));
            if path.is_absolute() {
                Some(path)
            } else {
                Some(client.cwd().join(path))
            }
        } else if output.status.code() == Some(1) {
            std::env::var_os("XDG_CONFIG_HOME")
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
                .map(|p| p.join("git/ignore"))
        } else {
            bail!(
                "cannot read Git excludes configuration: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        let global = directory.path().join(".git/jw-global-exclude");
        fs::write(&global, read_optional(exclude.as_deref())?)?;
        let global = global.to_str().context("ignore probe path not UTF-8")?;
        checked_git(directory.path(), ["config", "core.excludesFile", global])?;
        let info = backend.as_ref().map(|p| p.join("info/exclude"));
        fs::write(
            directory.path().join(".git/info/exclude"),
            read_optional(info.as_deref())?,
        )?;
        Ok(Self { directory })
    }
    fn require_ignored(&self, path: &Path) -> Result<()> {
        self.check(path, true)
    }
    fn require_ignored_file(&self, path: &Path) -> Result<()> {
        self.check(path, false)
    }
    fn check(&self, path: &Path, directory: bool) -> Result<()> {
        relative(path)?;
        let mut child = isolated_git(self.directory.path())
            .args(["check-ignore", "--no-index", "-z", "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut input = path
            .to_str()
            .context("ignored path not UTF-8")?
            .as_bytes()
            .to_vec();
        if directory {
            input.push(b'/');
        }
        input.push(0);
        child.stdin.take().unwrap().write_all(&input)?;
        let output = child.wait_with_output()?;
        if !output.status.success() || output.stdout != input {
            bail!(
                "selected path is not ignored under the frozen desired rules: {}{}",
                path.display(),
                if output.stderr.is_empty() {
                    String::new()
                } else {
                    format!(": {}", String::from_utf8_lossy(&output.stderr))
                }
            );
        }
        Ok(())
    }
}

fn read_optional(path: Option<&Path>) -> Result<Vec<u8>> {
    match path.map(fs::read).transpose() {
        Ok(bytes) => Ok(bytes.unwrap_or_default()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e).context("cannot read exclude file"),
    }
}

fn git_command(root: &Path) -> Command {
    let mut command = Command::new("git");
    command.current_dir(root);
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_CEILING_DIRECTORIES",
        "GIT_CONFIG_COUNT",
        "GIT_CONFIG_PARAMETERS",
    ] {
        command.env_remove(key);
    }
    command
}
fn isolated_git(root: &Path) -> Command {
    let mut command = git_command(root);
    command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env_remove("GIT_TEMPLATE_DIR")
        .args(["-c", "init.templateDir="]);
    command
}
fn checked_git<I, S>(root: &Path, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let output = isolated_git(root).args(args).output()?;
    if !output.status.success() {
        bail!(
            "ignore-rule probe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

fn expand_ignored(root: &Path, patterns: &[String]) -> Result<(Vec<PathBuf>, Vec<String>)> {
    fn expand(
        root: &Path,
        path: &Path,
        components: &[&str],
        found: &mut BTreeSet<PathBuf>,
    ) -> Result<()> {
        if components.is_empty() {
            if fs::symlink_metadata(root.join(path)).is_ok_and(|m| m.is_dir()) {
                relative(path)?;
                found.insert(path.to_path_buf());
            }
            return Ok(());
        }
        let children = match fs::read_dir(root.join(path)) {
            Ok(children) => children,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        if components[0] == "**" {
            expand(root, path, &components[1..], found)?;
        }
        for child in children {
            let child = child?;
            let name = child.file_name();
            if name == ".jj" || name == ".git" {
                continue;
            }
            if !child.file_type()?.is_dir() {
                continue;
            }
            let name = name.to_str().context("directory-glob entry not UTF-8")?;
            let path = path.join(name);
            if components[0] == "**" {
                expand(root, &path, components, found)?;
            } else if component_glob(components[0], name) {
                expand(root, &path, &components[1..], found)?;
            }
        }
        Ok(())
    }
    let mut found = BTreeSet::new();
    let mut skipped = Vec::new();
    for pattern in patterns {
        relative(Path::new(pattern))?;
        if pattern.contains(['[', ']', '\\']) {
            bail!("directory selections support literal components, *, ?, and ** only: {pattern}");
        }
        let mut matches = BTreeSet::new();
        expand(
            root,
            Path::new(""),
            &pattern.split('/').collect::<Vec<_>>(),
            &mut matches,
        )?;
        if matches.is_empty() {
            skipped.push(format!(
                "ignored selection {pattern:?} matched no source directories"
            ));
        }
        found.extend(matches);
    }
    let mut roots: Vec<PathBuf> = Vec::new();
    for path in found {
        if !roots.iter().any(|r| path.starts_with(r)) {
            roots.push(path);
        }
    }
    Ok((roots, skipped))
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
    // stamps and WC identity. Changed source recovery still performs the full
    // frozen byte/type/mode and dependency digest proof before updating stamps.
    if !unchanged || !plan.ignored.is_empty() {
        let entries = entries(&source, &plan.source_operation, &plan.source_commit)?;
        if !unchanged {
            if !changed_paths(&source, &operation, &current, &plan.source_commit)?.is_empty() {
                bail!(
                    "source tracked revision differs from the frozen tree; restore it before --resume"
                );
            }
            #[cfg(test)]
            SOURCE_LOGICAL_CHECKS.with(|count| count.set(count.get() + 1));
            verify_disk(
                &source,
                &plan.source_operation,
                &plan.source_commit,
                &plan.source,
                &entries,
            )?;
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
    if stamp(&plan.source)? != before || current_commit(&source)?.1 != current {
        bail!("source changed during content validation; pause writers before retrying");
    }
    plan.source_identity.verify(&plan.source)?;
    plan.source_jj_identity.verify(&plan.source.join(".jj"))?;
    if resume {
        plan.source_stamp = before;
        plan.validated_source_wc_commit = current;
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
    verify_disk(
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

/// Reconstruct every recorded tracked path before any ordinary snapshot. This is
/// deliberately idempotent even after sparse expansion or partial ignored copying.
fn execute(store: &Store, default: &Path, plan: &mut Pending, resume: bool) -> Result<()> {
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
    if resume && plan.phase == Phase::Newing && current != plan.old_commit {
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
        bail!("target WC revision changed since frozen plan; pending retained without snapshot");
    }
    if !resume {
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
    // Stock sparse mutations reject --ignore-working-copy. Recovery therefore
    // reconstructs the entire CURRENT commit before a normal sparse command may
    // snapshot it. The write-ahead Clearing phase covers interruptions during
    // that repair as well as during sparse clearing itself.
    transition(store, plan, Phase::Clearing)?;
    if resume {
        reconstruct_current_tree(plan)?;
    } else {
        let expected = entries(&client, &plan.target_operation, &plan.desired_commit)?;
        verify_disk(
            &client,
            &plan.target_operation,
            &plan.desired_commit,
            &plan.root,
            &expected,
        )?;
    }
    let output = client.run([
        "--config=fsmonitor.backend=none",
        "--config=snapshot.auto-track=none()",
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
        if !exact_empty_child(&client, &operation, &current, &base)? {
            // A crash after `new` but before its journal update is distinguished
            // from concurrent graph changes by the write-ahead Newing phase.
            transition(store, plan, Phase::Newing)?;
            let output = client.run([
                "--config=fsmonitor.backend=none",
                "--config=snapshot.auto-track=none()",
                "new",
                &base,
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
    require_full(&client)?;
    if current_commit(&client)?.1 != plan.desired_commit {
        bail!("target revision changed while materializing; no snapshot performed");
    }
    verified_snapshot(&client)?;
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
        plan.skipped.push(format!(
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
    verified_snapshot(&client)?;
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
    fn directory_component_globs_are_confined_and_never_follow_symlinks() {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir_all(directory.path().join("packages/a/node_modules")).unwrap();
        fs::create_dir_all(directory.path().join("packages/b/node_modules")).unwrap();
        fs::create_dir_all(directory.path().join(".jj/node_modules")).unwrap();
        std::os::unix::fs::symlink(
            directory.path().join("packages/a"),
            directory.path().join("packages/linked"),
        )
        .unwrap();
        let (roots, skipped) = expand_ignored(
            directory.path(),
            &["packages/*/node_modules".into(), "absent".into()],
        )
        .unwrap();
        assert_eq!(
            roots,
            [
                PathBuf::from("packages/a/node_modules"),
                PathBuf::from("packages/b/node_modules")
            ]
        );
        assert_eq!(skipped.len(), 1);
        assert!(expand_ignored(directory.path(), &["../*".into()]).is_err());
        assert!(component_glob("package-?", "package-λ"));
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
        };
        let store = Store::lock(&client).unwrap();
        Some((temporary, store, pending))
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
