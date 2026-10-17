//! Source-only preparation and narrowly scoped recovery of legacy target batches.
use super::*;
use crate::freshen::source::SourceSelection;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Settings {
    pub remote: Option<String>,
    pub command: Option<Vec<String>>,
    pub commands: Option<Vec<Vec<String>>>,
}

impl Settings {
    pub(super) fn argv(&self) -> Result<Vec<Vec<String>>> {
        if self.command.is_some() && self.commands.is_some() {
            bail!("freshen.command and freshen.commands cannot both be present, even when empty");
        }
        let argv = match (&self.command, &self.commands) {
            (Some(argv), None) => vec![argv.clone()],
            (None, Some(argv)) => argv.clone(),
            _ => Vec::new(),
        };
        validate_argv(&argv)?;
        Ok(argv)
    }
}

fn configured_argv(settings: Option<&Settings>) -> Result<Vec<Vec<String>>> {
    settings
        .map(Settings::argv)
        .transpose()
        .map(Option::unwrap_or_default)
}

fn validate_argv(commands: &[Vec<String>]) -> Result<()> {
    for (index, argv) in commands.iter().enumerate() {
        if argv.is_empty() || argv[0].is_empty() || argv.iter().any(|arg| arg.contains('\0')) {
            bail!(
                "freshen command {} must be a nonempty argv with a nonempty program and no NUL bytes",
                index + 1
            );
        }
    }
    Ok(())
}

/// Completion is recorded only after successful exit and source/target proofs.
/// An in-flight step may have run arbitrary external side effects before a crash.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommandSequence {
    argv: Vec<Vec<String>>,
    completed: usize,
    in_flight: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(super) enum PreparationPhase {
    Fetching,
    Selecting,
    Preparing,
    Verified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Target {
    name: String,
    root: PathBuf,
    root_identity: Identity,
    jj_identity: Identity,
    commit: String,
    stamp: Vec<Stamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct VerifiedSource {
    /// Historical WC at verification; restoration must not rewrite provenance.
    wc_commit: String,
    change_id: String,
    stamp: Vec<Stamp>,
    ignored: Vec<PathBuf>,
    #[serde(default)]
    selection_diagnostics: Vec<String>,
    payload_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Batch {
    schema_version: u32,
    pub id: String,
    pub request: RecordedRequest,
    source: PathBuf,
    source_identity: Identity,
    source_jj_identity: Identity,
    source_name: String,
    repository: PathBuf,
    config_root: PathBuf,
    local_config: Option<Vec<u8>>,
    remote: String,
    /// V3 up intent. Absent v1/v2 journals retain legacy keep-target semantics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_selection: Option<SourceSelection>,
    /// V1 journal input only; new records freeze command_sequence instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    command: Option<Vec<String>>,
    #[serde(default)]
    command_sequence: Option<CommandSequence>,
    #[serde(skip)]
    legacy_v1: bool,
    pub source_rev: String,
    old_operation: String,
    old_commit: String,
    selected_commit: Option<String>,
    pub goal: Option<String>,
    pub phase: PreparationPhase,
    targets: Vec<Target>,
    pub plans: Option<Vec<Pending>>,
    verified_source: Option<VerifiedSource>,
    accepted_verified_wc_commit: Option<String>,
}

fn config_bytes(root: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(root.join(".jwreseed.local.toml")) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub(super) fn verify_policy(root: &Path, frozen: &Option<Vec<u8>>) -> Result<()> {
    if &config_bytes(root)? != frozen {
        bail!("trusted local preparation configuration changed; frozen preparation retained");
    }
    Ok(())
}

fn path(store: &Store, source: &Path) -> PathBuf {
    store.path(&format!("source:{}", source.display()), "preparation")
}

pub(super) fn save(store: &Store, batch: &Batch) -> Result<()> {
    batch.validate_commands()?;
    atomic_json(&path(store, &batch.source), batch)
}

fn batches(store: &Store) -> Result<Vec<Batch>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(&store.root)? {
        let entry = entry?;
        if !entry
            .file_name()
            .to_string_lossy()
            .ends_with(".preparation.json")
        {
            continue;
        }
        if !entry.file_type()?.is_file() {
            bail!("preparation journal is not a regular file");
        }
        let mut batch: Batch = serde_json::from_slice(&fs::read(entry.path())?)?;
        if path(store, &batch.source) != entry.path() {
            bail!("invalid source preparation journal identity/version");
        }
        batch.load_compatible()?;
        result.push(batch);
    }
    Ok(result)
}

impl Batch {
    fn load_compatible(&mut self) -> Result<()> {
        match self.schema_version {
            1 if !self.request.prepare && !self.request.up && self.source_selection.is_none() => {
                if self.command_sequence.is_some() {
                    bail!("v1 preparation journal unexpectedly contains command sequence progress");
                }
                let argv = self
                    .command
                    .take()
                    .map(|argv| vec![argv])
                    .unwrap_or_default();
                let verified = self.phase == PreparationPhase::Verified;
                self.command_sequence = Some(CommandSequence {
                    completed: if verified { argv.len() } else { 0 },
                    in_flight: !verified
                        && self.phase == PreparationPhase::Preparing
                        && !argv.is_empty(),
                    argv,
                });
                self.schema_version = 2;
                self.legacy_v1 = true;
            }
            2 if self.command.is_none()
                && !self.request.prepare
                && !self.request.up
                && self.source_selection.is_none() => {}
            3 if self.command.is_none()
                && !self.request.prepare
                && self.request.up
                && self.source_selection.is_some() => {}
            4 if self.command.is_none()
                && self.request.prepare
                && self.source_selection.is_some() =>
            {
                self.validate_source_only()?;
            }
            _ => bail!("invalid source preparation journal version or legacy command field"),
        }
        self.validate_commands()?;
        if (self.request.up || self.request.prepare) != self.source_selection.is_some() {
            bail!("preparation workflow/source selection mismatch");
        }
        // Legacy v1/v2 requests already froze an effective array, including [].
        // Option preserves that wire form as Some(array), never new inheritance.
        if self.request.copy_ignored.is_none() {
            bail!("preparation journal lacks frozen ignored-copy selection");
        }
        Ok(())
    }

    fn validate_source_only(&self) -> Result<()> {
        if self.request.up
            || self.request.freshen
            || self.request.no_freshen_bookmark
            || !self.request.names.is_empty()
            || !self.request.groups.is_empty()
            || self.request.on_unmatched.is_some()
            || self.request.at.is_some()
            || self.request.replace_content
            || !self.targets.is_empty()
            || self.plans.as_ref().is_some_and(|plans| !plans.is_empty())
        {
            bail!("source-only preparation journal contains target intent");
        }
        Ok(())
    }

    fn sequence(&self) -> Result<&CommandSequence> {
        self.command_sequence
            .as_ref()
            .context("preparation journal lacks frozen command sequence")
    }

    fn validate_commands(&self) -> Result<()> {
        let sequence = self.sequence()?;
        validate_argv(&sequence.argv)?;
        if sequence.completed > sequence.argv.len()
            || (sequence.in_flight && sequence.completed == sequence.argv.len())
            || (matches!(
                self.phase,
                PreparationPhase::Fetching | PreparationPhase::Selecting
            ) && (sequence.completed != 0 || sequence.in_flight))
            || (self.phase == PreparationPhase::Verified
                && (sequence.completed != sequence.argv.len() || sequence.in_flight))
        {
            bail!("invalid preparation command completion/in-flight progress");
        }
        Ok(())
    }
}

pub(super) fn find_resume(
    store: &Store,
    names: &[String],
    default: &Path,
    source_only: bool,
) -> Result<Option<Batch>> {
    let matching = batches(store)?
        .into_iter()
        .filter(|b| {
            if b.request.prepare != source_only {
                return false;
            }
            if names.is_empty() {
                b.request.names.is_empty() && b.config_root == default
            } else {
                b.request.names.iter().any(|n| names.contains(n))
            }
        })
        .collect::<Vec<_>>();
    if matching.len() > 1 {
        bail!("resume targets belong to different preparation batches; resume them separately");
    }
    let batch = matching.into_iter().next();
    if let Some(b) = &batch
        && names.iter().any(|n| !b.request.names.contains(n))
    {
        bail!("resume mixes recorded preparation targets with unrelated targets");
    }
    Ok(batch)
}

pub(super) fn validate_resume_request(request: &RecordedRequest) -> Result<()> {
    if request.freshen
        || request.no_freshen_bookmark
        || request.source.is_some()
        || request.source_at.is_some()
        || request.at.is_some()
        || request.copy_ignored.is_some()
        || request.no_copy_ignored
        || request.replace_content
    {
        bail!("--resume uses recorded preparation and target plans; overrides are not allowed");
    }
    Ok(())
}

fn resume_command(request: &RecordedRequest) -> Result<String> {
    let mut argv = vec![
        "jw",
        if request.prepare { "prepare" } else { "reseed" },
        "--resume",
    ];
    if !request.names.is_empty() {
        argv.push("--");
        argv.extend(request.names.iter().map(String::as_str));
    }
    shlex::try_join(argv).context("cannot quote recorded resume command")
}

pub(super) fn ensure_available(store: &Store, source: &Path, own: Option<&Batch>) -> Result<()> {
    for batch in batches(store)? {
        if batch.source == source && own.is_none_or(|own| own.id != batch.id) {
            bail!(
                "source has an active preparation batch in phase {:?}; use `{}`; source progress was not rolled back",
                batch.phase,
                resume_command(&batch.request)?
            );
        }
    }
    Ok(())
}

pub(super) fn preflight(
    store: &Store,
    metadata: &WorkspaceMetadataStore,
    inventory: &Inventory,
    source: &Path,
    settings: Option<&Settings>,
) -> Result<()> {
    // Inspect every legacy/current target pending record, not just this selection.
    for entry in fs::read_dir(&store.root)? {
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
        let p: Pending = serde_json::from_slice(&fs::read(entry.path())?)?;
        if p.schema_version != 1 || store.path(&p.name, "pending") != entry.path() {
            bail!("invalid pending identity/version");
        }
        if p.source == source || p.source.canonicalize().ok().as_deref() == Some(source) {
            bail!(
                "source is borrowed by unfinished target {}; resume its frozen refill before freshening the donor",
                p.name
            );
        }
    }
    let context = crate::context::discover(Some(source));
    let name = context
        .jj
        .workspace_name
        .as_deref()
        .context("source workspace name missing")?;
    if inventory.root(name)? != source {
        bail!("source registration changed");
    }
    validate_ownership(source, name, metadata)?;
    configured_argv(settings)?;
    // The coordinator compiles new/unverified selectors before fetch. Verified
    // recovery validates recorded roots instead of reinterpreting old patterns.
    Ok(())
}

fn validate_ownership(source: &Path, name: &str, metadata: &WorkspaceMetadataStore) -> Result<()> {
    let record = metadata.get(name)?;
    if record
        .as_ref()
        .is_some_and(|m| m.external_owner.is_some() || m.owned_git_worktree.is_some())
        || crate::ownership::detect(source)?.is_some()
    {
        bail!("freshen does not mutate external/app-owned or linked Git donors");
    }
    crate::ownership::ensure_removal_allowed(
        &JjClient::new(source),
        name,
        Some(source),
        record.as_ref(),
    )?;
    Ok(())
}

pub(super) fn describe(
    source: &Path,
    selected: &SourceSelection,
    settings: Option<&Settings>,
    ignored: &[String],
) -> Result<String> {
    let commands = configured_argv(settings)?;
    Ok(format!(
        "Prepare source {:?}; pinned {:?} bookmark {:?}@{:?}; prospective post-fetch goal {:?} (currently {:?}); preparation argv sequence {:?}, cwd {:?}; ignored selection {:?}; {}",
        source,
        selected.kind,
        selected.name,
        selected.remote,
        selected.source_at,
        selected.old_goal,
        commands,
        source,
        ignored,
        if commands.is_empty() {
            "tracked-source preparation only; no configured commands; selected ignored payload is existing data, not freshly built"
        } else {
            "trusted configured programs, sequentially (no implicit shell)"
        }
    ))
}

pub(super) fn summary(batch: &Batch) -> String {
    let sequence = batch
        .command_sequence
        .as_ref()
        .expect("validated command sequence");
    format!(
        "Recorded preparation ({}): remote {:?}, exact source goal {:?}, preparation argv sequence {:?}, cwd {:?}; commands completed {}/{}{}, {}; {}",
        if batch.request.prepare {
            "source-only prepare"
        } else if batch.request.up {
            "legacy up (reseed --resume)"
        } else {
            "legacy reseed"
        },
        batch.remote,
        batch.goal,
        sequence.argv,
        batch.source,
        sequence.completed,
        sequence.argv.len(),
        if sequence.in_flight {
            "; in-flight command may retry on resume"
        } else {
            ""
        },
        if batch.legacy_v1 {
            "v1 single-command journal loaded compatibly; only Verified proves legacy completion"
        } else {
            if batch.request.prepare {
                "v4 source-only preparation journal"
            } else if batch.request.up {
                "v3 up sequence journal"
            } else {
                "v2 legacy reseed sequence journal"
            }
        },
        if sequence.argv.is_empty() {
            "tracked-source preparation only; no configured commands; selected ignored payload is existing data, not freshly built"
        } else if batch.phase != PreparationPhase::Verified {
            "configured preparation incomplete; targets not refilled"
        } else {
            "verified configured preparation"
        }
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn begin(
    store: &Store,
    client: &JjClient,
    source: &Path,
    default: &Path,
    request: &RecordedRequest,
    settings: Option<&Settings>,
    ignored: &[String],
    local_config: Option<Vec<u8>>,
    source_selection: SourceSelection,
) -> Result<Batch> {
    if !request.prepare || !request.names.is_empty() {
        bail!("new source preparation must be source-only; legacy batches are resume-only");
    }
    verify_policy(default, &local_config)?;
    let operation = client.operation_id()?;
    let source_client = JjClient::new(source);
    let old_commit = source_client.resolve_one_at(&operation, "@")?.commit_id;
    require_unconflicted(&source_client, &operation, &old_commit)?;
    let mut frozen_request = request.clone();
    let source_rev = source_selection.source_at.clone().unwrap_or_else(|| {
        format!(
            "{}@{}",
            serde_json::to_string(&source_selection.name).expect("JSON string"),
            serde_json::to_string(&source_selection.remote).expect("JSON string"),
        )
    });
    frozen_request.groups.clear();
    frozen_request.on_unmatched = None;
    frozen_request.source = Some(source.to_path_buf());
    frozen_request.source_at = Some(source_rev.clone());
    frozen_request.copy_ignored = Some(ignored.to_vec());
    let nonce = tempfile::NamedTempFile::new_in(&store.root)?;
    let id = Sha256::digest(serde_json::to_vec(&(
        source,
        &operation,
        &old_commit,
        nonce.path(),
    ))?)
    .iter()
    .map(|b| format!("{b:02x}"))
    .collect();
    let batch = Batch {
        schema_version: 4,
        id,
        request: frozen_request,
        source: source.to_path_buf(),
        source_identity: Identity::read(source)?,
        source_jj_identity: Identity::read(&source.join(".jj"))?,
        source_name: crate::context::discover(Some(source))
            .jj
            .workspace_name
            .context("source name missing")?,
        repository: repository(client)?,
        config_root: default.to_path_buf(),
        local_config,
        remote: source_selection.remote.clone(),
        source_selection: Some(source_selection),
        command: None,
        command_sequence: Some(CommandSequence {
            argv: configured_argv(settings)?,
            completed: 0,
            in_flight: false,
        }),
        legacy_v1: false,
        source_rev,
        old_operation: operation,
        old_commit,
        selected_commit: None,
        goal: None,
        phase: PreparationPhase::Fetching,
        targets: Vec::new(),
        plans: None,
        verified_source: None,
        accepted_verified_wc_commit: None,
    };
    batch.validate_source_only()?;
    save(store, &batch)?;
    Ok(batch)
}

fn validate(batch: &Batch) -> Result<()> {
    batch.validate_commands()?;
    batch.source_identity.verify(&batch.source)?;
    batch.source_jj_identity.verify(&batch.source.join(".jj"))?;
    let client = JjClient::new(&batch.source);
    if repository(&client)? != batch.repository
        || crate::context::discover(Some(&batch.source))
            .jj
            .workspace_name
            .as_ref()
            != Some(&batch.source_name)
    {
        bail!("preparation source/repository registration changed");
    }
    verify_policy(&batch.config_root, &batch.local_config)?;
    let metadata = WorkspaceMetadataStore::from_repo_config_path(client.repo_config_path()?)?;
    validate_ownership(&batch.source, &batch.source_name, &metadata)?;
    ensure_quiescence(&client)?;
    require_full(&client)?;
    validate_contents(&batch.source, true)?;
    Ok(())
}

fn capture_verified(batch: &Batch) -> Result<VerifiedSource> {
    let client = JjClient::new(&batch.source);
    let (operation, wc_commit) = current_commit(&client)?;
    let goal = batch.goal.as_ref().context("verified source lacks goal")?;
    if !exact_empty_child(&client, &operation, &wc_commit, goal)? {
        bail!("source changed before preparation proof capture");
    }
    let before = stamp(&batch.source)?;
    let source_entries = entries(&client, &operation, goal)?;
    let rules = IgnoreRules::new(&client, &operation, goal, &source_entries, true)?;
    let (ignored, selection_diagnostics) = select_ignored(
        &batch.source,
        batch
            .request
            .copy_ignored
            .as_deref()
            .context("missing frozen ignored-copy selection")?,
        &rules,
        &source_entries,
    )?;
    let payload_digest = payload_digest(&batch.source, &ignored)?;
    if stamp(&batch.source)? != before || current_commit(&client)?.1 != wc_commit {
        bail!("source changed during preparation proof capture");
    }
    Ok(VerifiedSource {
        change_id: client.resolve_one_at(&operation, &wc_commit)?.change_id,
        wc_commit,
        stamp: before,
        ignored,
        selection_diagnostics,
        payload_digest,
    })
}

pub(super) fn verify_verified(batch: &mut Batch) -> Result<()> {
    validate(batch)?;
    let proof = batch
        .verified_source
        .as_ref()
        .context("verified preparation lacks frozen source proof")?;
    let client = JjClient::new(&batch.source);
    let (operation, current) = current_commit(&client)?;
    let before = stamp(&batch.source)?;
    let goal = batch.goal.as_ref().context("missing frozen goal")?;
    let source_entries = entries(&client, &operation, goal)?;
    let rules = IgnoreRules::new(&client, &operation, goal, &source_entries, true)?;
    validate_selected_ignored(&batch.source, &proof.ignored, &rules, &source_entries)?;
    require_unconflicted(&client, &operation, &current)?;
    if client.resolve_one_at(&operation, &current)?.change_id != proof.change_id
        || !exact_empty_child(&client, &operation, &current, goal)?
        || payload_digest(&batch.source, &proof.ignored)? != proof.payload_digest
    {
        bail!(
            "verified preparation source/payload changed before target freeze; pending retained; restore frozen verified donor rather than adopting changed dependencies"
        );
    }
    if before != proof.stamp || current != proof.wc_commit {
        // Exact payload restoration may have new inode/ctime. Never adopt new
        // bytes: reprove the tracked tree as well as the frozen logical payload.
        verify_disk(&client, &operation, goal, &batch.source, &source_entries)?;
    }
    if stamp(&batch.source)? != before || current_commit(&client)?.1 != current {
        bail!("verified donor changed during proof validation");
    }
    batch.source_identity.verify(&batch.source)?;
    batch.source_jj_identity.verify(&batch.source.join(".jj"))?;
    batch.accepted_verified_wc_commit = Some(current);
    Ok(())
}

pub(super) fn verified_selection(batch: &Batch) -> Result<(&[PathBuf], &[String])> {
    if batch.phase != PreparationPhase::Verified {
        bail!("preparation payload selection is not verified");
    }
    let proof = batch
        .verified_source
        .as_ref()
        .context("missing frozen verified source selection")?;
    Ok((&proof.ignored, &proof.selection_diagnostics))
}

pub(super) fn verify_targets(batch: &Batch) -> Result<()> {
    if batch.targets.is_empty() {
        return Ok(());
    }
    let client = JjClient::new(&batch.source);
    let operation = client.operation_id()?;
    let facts = client.workspace_target_facts_at(&operation)?;
    for target in &batch.targets {
        target.root_identity.verify(&target.root)?;
        target.jj_identity.verify(&target.root.join(".jj"))?;
        if facts
            .get(&target.name)
            .is_none_or(|f| f.commit_id != target.commit)
            || current_commit(&JjClient::new(&target.root))?.1 != target.commit
            || stamp(&target.root)? != target.stamp
        {
            bail!(
                "target {} changed or became stale during source preparation; no target snapshot/refill performed",
                target.name
            );
        }
        // Stock JJ's status detects stale WC state. Bytes were checked first, so
        // this cannot absorb installer edits or a half-filled pending target.
        verified_snapshot(&JjClient::new(&target.root))?;
        if current_commit(&JjClient::new(&target.root))?.1 != target.commit
            || stamp(&target.root)? != target.stamp
        {
            bail!(
                "target {} changed during post-preparation validation",
                target.name
            );
        }
    }
    Ok(())
}

pub(super) fn execute(
    store: &Store,
    batch: &mut Batch,
    diagnostic: &mut dyn FnMut(&str),
) -> Result<()> {
    let resume = resume_command(&batch.request)?;
    let result = (|| {
        validate(batch)?;
        verify_targets(batch)?;
        let client = JjClient::new(&batch.source);
        if batch.goal.is_none() {
            if let Some(selected) = &batch.source_selection {
                selected.verify_source_at(&client, &client.operation_id()?)?;
            } else if current_commit(&client)?.1 != batch.old_commit {
                bail!(
                    "source changed before frozen fetch goal; restore recorded source work before resume"
                );
            }
            let old_entries = entries(&client, &batch.old_operation, &batch.old_commit)?;
            verify_disk(
                &client,
                &batch.old_operation,
                &batch.old_commit,
                &batch.source,
                &old_entries,
            )?;
            diagnostic(&format!(
                "Prepare: fetching remote {:?} in {:?}; previous work preserved at {}",
                batch.remote, batch.source, batch.old_commit
            ));
            let output = if let Some(selected) = &batch.source_selection {
                client.fetch_bookmark(&selected.remote, &selected.name)?
            } else {
                client.run([
                    "--config=fsmonitor.backend=none",
                    "git",
                    "fetch",
                    "--remote",
                    &batch.remote,
                ])?
            };
            diagnostics(output.stderr());
            let operation = client.operation_id()?;
            batch.goal = Some(match &batch.source_selection {
                Some(selected) => selected.resolve_fetched(&client, &operation)?,
                None => {
                    client
                        .resolve_one_at(&operation, &batch.source_rev)?
                        .commit_id
                }
            });
            batch.phase = PreparationPhase::Selecting;
            save(store, batch)?;
        }
        let goal = batch.goal.clone().context("missing frozen goal")?;
        let (operation, current) = current_commit(&client)?;
        require_unconflicted(&client, &operation, &goal)?;
        let goal_entries = entries(&client, &operation, &goal)?;
        // Freeze/check known future ignored policy as soon as fetch made the
        // goal knowable. Installer-created wildcard matches are checked later.
        if batch.phase != PreparationPhase::Verified {
            let rules = IgnoreRules::new(&client, &operation, &goal, &goal_entries, true)?;
            select_ignored(
                &batch.source,
                batch
                    .request
                    .copy_ignored
                    .as_deref()
                    .context("missing frozen ignored-copy selection")?,
                &rules,
                &goal_entries,
            )?;
        }
        for entry in goal_entries.iter().filter(|entry| entry.kind == "symlink") {
            symlink_bytes(&client, &operation, &goal, &entry.path)?;
        }
        if batch.selected_commit.is_none() {
            let unchanged_source = if let Some(selected) = &batch.source_selection {
                selected.verify_source_at(&client, &operation).is_ok()
            } else {
                current == batch.old_commit
            };
            if !unchanged_source && !exact_empty_child(&client, &operation, &current, &goal)? {
                bail!(
                    "source changed during interrupted jj new; preparation retained without snapshot"
                );
            }
            if !exact_empty_child(&client, &operation, &current, &goal)? {
                // A fetch may have legitimately rewritten the old source commit.
                // Verify its bytes before allowing JJ's normal snapshot in new.
                let old_entries = entries(&client, &batch.old_operation, &batch.old_commit)?;
                verify_disk(
                    &client,
                    &batch.old_operation,
                    &batch.old_commit,
                    &batch.source,
                    &old_entries,
                )?;
                batch.phase = PreparationPhase::Selecting;
                save(store, batch)?;
                validate(batch)?;
                let output = client.run([
                    "--config=fsmonitor.backend=none",
                    "new",
                    &crate::jj::commit_id_revset(&goal)?,
                ])?;
                diagnostics(output.stderr());
            }
            let (operation, current) = current_commit(&client)?;
            if !exact_empty_child(&client, &operation, &current, &goal)? {
                bail!("source JJ new did not select expected empty child");
            }
            batch.selected_commit = Some(current);
            batch.phase = PreparationPhase::Preparing;
            save(store, batch)?;
            validate(batch)?;
        }
        // No snapshot until exact tracked disk bytes have been checked. A failed
        // installer that edited tracked files requires deliberate restoration.
        let (operation, current) = current_commit(&client)?;
        let selected = batch
            .selected_commit
            .as_ref()
            .context("missing selected source child")?;
        if client.resolve_one_at(&operation, &current)?.change_id
            != client.resolve_one_at(&operation, selected)?.change_id
            || !exact_empty_child(&client, &operation, &current, &goal)?
        {
            bail!(
                "source selected revision changed; restore the recorded source child before resume"
            );
        }
        verify_disk(&client, &operation, &goal, &batch.source, &goal_entries).context(
            "restore installer-modified tracked files to the frozen source goal before resume",
        )?;
        if batch.phase != PreparationPhase::Verified {
            diagnostic(&format!(
                "Prepare: exact post-fetch goal {goal}; {}",
                summary(batch),
            ));
            batch.phase = PreparationPhase::Preparing;
            save(store, batch)?;
            validate(batch)?;
            let previously_completed = batch.sequence()?.completed;
            while batch.sequence()?.completed < batch.sequence()?.argv.len() {
                let sequence = batch.sequence()?;
                let index = sequence.completed;
                let argv = sequence.argv[index].clone();
                diagnostic(&format!(
                    "Prepare: command {}/{} argv {:?}, cwd {:?}; {}",
                    index + 1,
                    sequence.argv.len(),
                    argv,
                    batch.source,
                    if sequence.in_flight {
                        "retrying recorded in-flight command; prior attempt may have performed external side effects (not exactly-once)"
                    } else {
                        "sequential argv execution; no implicit shell"
                    }
                ));
                batch.command_sequence.as_mut().unwrap().in_flight = true;
                save(store, batch)?;
                validate(batch)?;
                let status = Command::new(&argv[0])
                    .args(&argv[1..])
                    .current_dir(&batch.source)
                    .stdin(Stdio::inherit())
                    .stdout(std::io::stderr())
                    .stderr(Stdio::inherit())
                    .status()
                    .with_context(|| format!("could not run configured preparation argv {}; in-flight command may retry on resume", index + 1))?;
                if !status.success() {
                    bail!(
                        "configured preparation command failed with {status} (step {}); ignored installation may be incomplete; completed earlier commands will not replay; this in-flight command may retry on resume",
                        index + 1
                    );
                }
                batch.selected_commit =
                    Some(verify_command_result(batch, &client, &goal, &goal_entries)?);
                let sequence = batch.command_sequence.as_mut().unwrap();
                sequence.completed += 1;
                sequence.in_flight = false;
                // A crash before this durable acknowledgement can replay this
                // command even if its external side effects already succeeded.
                save(store, batch)?;
            }
            // Also applies to no-command preparation and to a resumed sequence
            // whose final completion was durable before proof capture stopped.
            if batch.sequence()?.completed == previously_completed {
                batch.selected_commit =
                    Some(verify_command_result(batch, &client, &goal, &goal_entries)?);
            }
            batch.verified_source = Some(capture_verified(batch)?);
            batch.phase = PreparationPhase::Verified;
            save(store, batch)?;
        }
        verify_verified(batch)?;
        save(store, batch)?;
        diagnostic(&format!(
            "Prepare: donor verified at exact goal {:?}; {}; source preparation is not rolled back if later planning fails",
            batch.goal,
            if batch.targets.is_empty() {
                "source-only completion; no targets selected"
            } else {
                "targets not yet refilled"
            },
        ));
        Ok(())
    })();
    result.with_context(|| format!("source preparation stopped in phase {:?}, frozen goal {:?}; source progress was not rolled back; targets not refilled; use `{resume}`", batch.phase, batch.goal))
}

fn verify_command_result(
    batch: &Batch,
    client: &JjClient,
    goal: &str,
    goal_entries: &[Entry],
) -> Result<String> {
    validate(batch)?;
    // Check the tracked tree without adopting newly created untracked outputs.
    // The guarded verifier snapshots tracked edits and compares against the goal.
    verify_disk(client, &client.operation_id()?, goal, &batch.source, goal_entries)
        .context("preparer changed tracked tree (including lockfiles); restore frozen source child/tree before resume")?;
    let (operation, current) = current_commit(client)?;
    let selected = batch
        .selected_commit
        .as_ref()
        .context("missing selected source child")?;
    if client.resolve_one_at(&operation, &current)?.change_id
        != client.resolve_one_at(&operation, selected)?.change_id
        || !exact_empty_child(client, &operation, &current, goal)?
    {
        bail!("source selected revision changed; restore the recorded source child before resume");
    }
    verify_targets(batch)?;
    Ok(current)
}

/// Compare the immutable plan, not merely an ID field or a workspace name.
fn same_plan(original: &Pending, actual: &Pending) -> Result<bool> {
    let mut normalized = actual.clone();
    normalized.phase = original.phase;
    normalized.desired_commit = original.desired_commit.clone();
    normalized.target_operation = original.target_operation.clone();
    normalized.source_stamp = original.source_stamp.clone();
    normalized.validated_source_wc_commit = original.validated_source_wc_commit.clone();
    normalized.execution_diagnostics = original.execution_diagnostics.clone();
    Ok(serde_json::to_vec(&normalized)? == serde_json::to_vec(original)?)
}

pub(super) fn handoff(store: &Store, plan: &Pending, dry_run: bool) -> Result<Option<Pending>> {
    if let Some(pending) = store.pending(&plan.name)? {
        if !same_plan(plan, &pending)? {
            bail!(
                "foreign or changed pending plan for {}; frozen preparation handoff refused",
                plan.name
            );
        }
        return Ok(Some(pending));
    }
    let receipt_path = store.path(&plan.name, "receipt");
    match fs::symlink_metadata(&receipt_path) {
        Ok(m) if !m.is_file() => bail!("receipt is not a regular file"),
        Ok(_) => {
            let receipt: Pending = serde_json::from_slice(&fs::read(receipt_path)?)?;
            if receipt.preparation_id == plan.preparation_id {
                if receipt.phase != Phase::Done || !same_plan(plan, &receipt)? {
                    bail!("preparation receipt plan identity mismatch");
                }
                return Ok(None);
            }
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    if !dry_run {
        store.save(plan)?;
    }
    Ok(Some(plan.clone()))
}

pub(super) fn finish_if_complete(store: &Store, batch: &Batch) -> Result<()> {
    let plans = batch.plans.as_ref().context("batch targets not frozen")?;
    for plan in plans {
        if store.pending(&plan.name)?.is_some() {
            return Ok(());
        }
        if handoff(store, plan, true)?.is_some() {
            return Ok(());
        }
    }
    // Keep an immutable receipt for provenance; cleanup is only this completed
    // operation's active marker, not donor/target pruning or reconciliation.
    let active = path(store, &batch.source);
    let receipt = store.path(&format!("preparation:{}", batch.id), "receipt");
    atomic_json(&receipt, batch)?;
    fs::remove_file(active)?;
    File::open(&store.root)?.sync_all()?;
    Ok(())
}

pub(super) fn source(batch: &Batch) -> &Path {
    &batch.source
}

/// Include not-yet-created bookmarks in other batches' atomic target intents.
pub(super) fn reserve_bookmarks(store: &Store, names: &mut BookmarkNames) -> Result<()> {
    for batch in batches(store)? {
        for plan in batch.plans.iter().flatten() {
            reserve_history(plan, names);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resume_guidance_quotes_literal_names_and_has_no_source_only_placeholder() {
        let mut request = RecordedRequest {
            prepare: true,
            ..Default::default()
        };
        assert_eq!(resume_command(&request).unwrap(), "jw prepare --resume");
        request.prepare = false;
        request.names = vec![
            "worker with spaces".into(),
            "quote'worker".into(),
            "--worker".into(),
        ];
        let command = resume_command(&request).unwrap();
        assert_eq!(
            shlex::split(&command).unwrap(),
            [
                "jw",
                "reseed",
                "--resume",
                "--",
                "worker with spaces",
                "quote'worker",
                "--worker"
            ]
        );
    }

    #[test]
    fn local_command_settings_preserve_presence_and_order() {
        for text in [
            "command = []\ncommands = []",
            "command = ['cargo']\ncommands = []",
            "command = []",
            "commands = [[]]",
            "commands = [['']]",
        ] {
            let settings: Settings = toml::from_str(text).unwrap();
            assert!(settings.argv().is_err(), "{text}");
        }
        let settings: Settings =
            toml::from_str("commands = [['bun', 'install'], ['bun', 'run', 'build']]").unwrap();
        assert_eq!(
            settings.argv().unwrap(),
            [vec!["bun", "install"], vec!["bun", "run", "build"]]
        );
        assert!(
            toml::from_str::<Settings>("commands = []")
                .unwrap()
                .argv()
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            toml::from_str::<Settings>("command = ['cargo', 'build']")
                .unwrap()
                .argv()
                .unwrap(),
            [vec!["cargo", "build"]]
        );
    }
}
