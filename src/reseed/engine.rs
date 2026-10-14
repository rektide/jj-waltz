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

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalConfig {
    source: Option<PathBuf>,
    source_at: Option<String>,
    #[serde(default)]
    copy_ignored: Vec<String>,
    #[serde(default)]
    groups: BTreeMap<String, Vec<String>>,
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
        // Collision-free UTF-8 encoding, without treating literal names as paths.
        let encoded = name
            .as_bytes()
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

pub(super) fn run(request: ReseedRequest) -> Result<ReseedReport> {
    let client = JjClient::current()?;
    // Existing convenience inventory performs normal JJ commands. Do not use it
    // here: merely discovering a pending current workspace must not snapshot it.
    let inventory = Inventory::load(&client)?;
    let default = inventory.default.clone();
    let config = if request.resume && request.groups.is_empty() {
        LocalConfig::default()
    } else {
        load_config(&default)?
    };
    let names = selections(&request, &config)?;
    let store = Store::lock(&client)?;
    let metadata = WorkspaceMetadataStore::from_repo_config_path(client.repo_config_path()?)?;
    let repository_path = repository(&client)?;
    let mut plans = Vec::new();
    if request.resume {
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
            .context("reseed requires --from or source in .jwreseed.local.toml")?;
        let source = if raw_source.is_absolute() {
            raw_source.clone()
        } else if request.source.is_some() {
            client.cwd().join(raw_source)
        } else {
            default.join(raw_source)
        }
        .canonicalize()?;
        let source_client = JjClient::new(&source);
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
        ensure_quiescence(&source_client)?;
        require_full(&source_client)?;
        require_snapshot_disabled_sparse(&source_client)?;
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
        }
        // Full scans freeze dirty tracked work before tree/revision resolution.
        snapshot(&source_client)?;
        for root in &roots {
            snapshot(&JjClient::new(root))?;
        }
        let operation = client.operation_id()?;
        let source_operation = operation.clone();
        let source_wc = source_client
            .resolve_one_at(&source_operation, "@")?
            .commit_id;
        let source_rev = request
            .source_at
            .as_ref()
            .or(config.source_at.as_ref())
            .map(String::as_str)
            .context("reseed requires --source-at or source_at in .jwreseed.local.toml")?;
        let source_commit = source_client
            .resolve_one_at(&source_operation, source_rev)?
            .commit_id;
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
        verify_disk(
            &source_client,
            &source_operation,
            &source_commit,
            &source,
            &source_entries,
        )?;
        let source_stamp = stamp(&source)?;
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
            });
        }
    }
    let first = plans.first().context("select at least one workspace")?;
    let mut report = ReseedReport {
        dry_run: request.dry_run,
        source: first.source.clone(),
        source_commit_id: first.source_commit.clone(),
        targets: Vec::new(),
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
        let outcome = execute(&store, &default, &mut plan, request.resume);
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
    Ok(report)
}

struct Inventory {
    default: PathBuf,
    roots: BTreeMap<String, PathBuf>,
}
impl Inventory {
    fn load(client: &JjClient) -> Result<Self> {
        let operation = client.operation_id()?;
        let mut roots = BTreeMap::new();
        for name in client.workspace_names()? {
            let output =
                client.run_at_unchecked(&operation, ["workspace", "root", "--name", &name])?;
            if output.success() {
                let path = PathBuf::from(output.trimmed_stdout()?);
                if let Ok(path) = path.canonicalize() {
                    roots.insert(name, path);
                }
            }
        }
        let context = crate::context::discover(Some(client.cwd()));
        let default = roots
            .get("default")
            .cloned()
            .or(context.jj.primary_checkout)
            .context("cannot identify default/primary workspace")?
            .canonicalize()?;
        Ok(Self { default, roots })
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

fn load_config(default: &Path) -> Result<LocalConfig> {
    let path = default.join(".jwreseed.local.toml");
    match fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text).with_context(|| format!("invalid {}", path.display())),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(LocalConfig::default()),
        Err(e) => Err(e).with_context(|| format!("cannot read {}", path.display())),
    }
}

fn selections(request: &ReseedRequest, config: &LocalConfig) -> Result<Vec<String>> {
    let mut names = request.names.clone();
    for group in &request.groups {
        names.extend(
            config
                .groups
                .get(group)
                .with_context(|| format!("unknown reseed group {group}"))?
                .iter()
                .cloned(),
        );
    }
    let mut seen = BTreeSet::new();
    names.retain(|name| seen.insert(name.clone()));
    if names.is_empty() {
        bail!("reseed requires literal workspace names or --group; never selects all implicitly");
    }
    for name in &names {
        if name.is_empty() || name.contains('\0') {
            bail!("reseed requires a nonempty literal workspace name without NUL");
        }
    }
    Ok(names)
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

fn require_snapshot_disabled_sparse(client: &JjClient) -> Result<()> {
    // Adding the existing full pattern is nondestructive. Probe real behavior,
    // not --help (JJ advertises the global flag even when sparse rejects it).
    let output = client.run_unchecked(["--ignore-working-copy", "sparse", "set", "--add", "."])?;
    if !output.success() {
        bail!(
            "JJ lacks snapshot-disabled sparse mutation required for safe reseed recovery; no target was changed: {}",
            output.stderr()
        );
    }
    diagnostics(output.stderr());
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
    Ok(output.stdout_bytes().to_vec())
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
        if bytes != file_bytes(client, operation, commit, &entry.path)? {
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
    let identity = Identity::read(root)?;
    for child in fs::read_dir(root)? {
        let child = child?;
        if child.file_name() == ".jj" || (allow_git && child.file_name() == ".git") {
            continue;
        }
        let metadata = fs::symlink_metadata(child.path())?;
        if metadata.dev() != identity.device {
            bail!("mounted child is unsupported: {}", child.path().display());
        }
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
    Ok(hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
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
                    if !old.iter().any(|e| e.path == path) {
                        bail!(
                            "untracked obstruction at {}; use --replace-content only if losing it is intended",
                            path.display()
                        );
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

fn preflight_links(
    default: &Path,
    root: &Path,
    entries: &[Entry],
    ignored: &[PathBuf],
    replace: bool,
    rules: &IgnoreRules,
) -> Result<()> {
    if let Some(config) = links::load_link_config(default)? {
        for check in links::inspect_loaded_workspace_links(&config, root) {
            let path = check
                .source
                .strip_prefix(root)
                .context("link source escapes target")?;
            relative(path)?;
            // A source-equals-target override deliberately keeps a private directory.
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
            if check.required && !check.target_exists {
                bail!("required link target missing: {}", check.target.display());
            }
            if !replace
                && matches!(
                    check.state,
                    links::LinkCheckState::Conflicting | links::LinkCheckState::Unreadable(_)
                )
            {
                bail!("configured link conflict at {}", path.display());
            }
            rules.require_ignored_file(path)?;
        }
    }
    Ok(())
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
        checked_git(directory.path(), ["init", "--quiet"])?;
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
        .env("GIT_CONFIG_SYSTEM", "/dev/null");
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

fn component_glob(pattern: &str, text: &str) -> bool {
    let p = pattern.chars().collect::<Vec<_>>();
    let t = text.chars().collect::<Vec<_>>();
    let mut matches = vec![false; t.len() + 1];
    matches[0] = true;
    for c in p {
        let mut next = vec![false; t.len() + 1];
        if c == '*' {
            next[0] = matches[0];
        }
        for i in 1..=t.len() {
            next[i] = if c == '*' {
                matches[i] || next[i - 1]
            } else {
                matches[i - 1] && (c == '?' || c == t[i - 1])
            };
        }
        matches = next;
    }
    matches[t.len()]
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
    if link_config_bytes(&plan.config_root)? != plan.link_config {
        bail!("workspace link configuration changed since frozen plan; pending retained");
    }
    let source = JjClient::new(&plan.source);
    require_full(&source)?;
    let operation = source.operation_id()?;
    let current = source.resolve_one_at(&operation, "@")?.commit_id;
    let before = stamp(&plan.source)?;
    if !resume && (current != plan.validated_source_wc_commit || before != plan.source_stamp) {
        bail!("source contents changed; restore the frozen source before --resume");
    }
    if resume && !changed_paths(&source, &operation, &current, &plan.source_commit)?.is_empty() {
        bail!("source tracked revision differs from the frozen tree; restore it before --resume");
    }
    let entries = entries(&source, &plan.source_operation, &plan.source_commit)?;
    verify_disk(
        &source,
        &plan.source_operation,
        &plan.source_commit,
        &plan.source,
        &entries,
    )?;
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
    if payload_digest(&plan.source, &plan.ignored)? != plan.source_payload_digest {
        bail!(
            "source ignored payload differs from the frozen plan; restore its bytes/types/modes before --resume"
        );
    }
    if stamp(&plan.source)? != before {
        bail!("source changed during content validation; pause writers before retrying");
    }
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
        snapshot(&client)?;
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
    // Remove placeholder prefills before clearing sparsity. A failed expansion
    // can leave JJ unable to delete a modified placeholder; both operations are
    // snapshot-disabled and followed by exact planned-path removal.
    transition(store, plan, Phase::Clearing)?;
    if resume {
        for path in &plan.clones {
            remove_recorded_file(&plan.root, path, true)?;
        }
    }
    let output = client.run([
        "--ignore-working-copy",
        "--config=fsmonitor.backend=none",
        "sparse",
        "set",
        "--clear",
    ])?;
    diagnostics(output.stderr());
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
            let output = client.run(["--config=fsmonitor.backend=none", "new", &base])?;
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
    // Restore stock sparse semantics without snapshotting the staged prefill.
    transition(store, plan, Phase::Expanding)?;
    let output = client.run([
        "--ignore-working-copy",
        "--config=fsmonitor.backend=none",
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
    preflight_links(
        default,
        &plan.root,
        &plan.entries,
        &plan.ignored,
        false,
        &rules,
    )?;
    transition(store, plan, Phase::CopyingIgnored)?;
    validate_source(plan, resume)?;
    for path in &plan.ignored {
        confined_parents(&plan.root, path)?;
        cow::remove_path(&plan.root.join(path))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_selection_deduplicates_groups_without_selectors() {
        let config: LocalConfig = toml::from_str("[groups]\nworkers=['a','b','a']\n").unwrap();
        let request = ReseedRequest {
            names: vec!["b".into(), "one-off".into()],
            groups: vec!["workers".into()],
            ..Default::default()
        };
        assert_eq!(
            selections(&request, &config).unwrap(),
            ["b", "one-off", "a"]
        );
        assert!(selections(&ReseedRequest::default(), &config).is_err());
        assert_eq!(
            selections(
                &ReseedRequest {
                    names: vec!["@".into(), "^".into(), "-".into()],
                    ..Default::default()
                },
                &config
            )
            .unwrap(),
            ["@", "^", "-"]
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
}
