//! Source preparation is a batch prerequisite, never a target-recovery mutation.
//! This journal retains frozen target intent until every receipt is accounted for.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Settings {
    pub remote: Option<String>,
    pub command: Option<Vec<String>>,
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
    wc_commit: String,
    stamp: Vec<Stamp>,
    ignored: Vec<PathBuf>,
    payload_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Batch {
    schema_version: u32,
    pub id: String,
    pub request: ReseedRequest,
    source: PathBuf,
    source_identity: Identity,
    source_jj_identity: Identity,
    source_name: String,
    repository: PathBuf,
    config_root: PathBuf,
    local_config: Option<Vec<u8>>,
    remote: String,
    command: Option<Vec<String>>,
    pub source_rev: String,
    old_operation: String,
    old_commit: String,
    selected_commit: Option<String>,
    pub goal: Option<String>,
    pub phase: PreparationPhase,
    targets: Vec<Target>,
    pub plans: Option<Vec<Pending>>,
    verified_source: Option<VerifiedSource>,
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
        let batch: Batch = serde_json::from_slice(&fs::read(entry.path())?)?;
        if batch.schema_version != 1 || path(store, &batch.source) != entry.path() {
            bail!("invalid source preparation journal identity/version");
        }
        result.push(batch);
    }
    Ok(result)
}

pub(super) fn find_resume(store: &Store, names: &[String]) -> Result<Option<Batch>> {
    let matching = batches(store)?
        .into_iter()
        .filter(|b| b.request.names.iter().any(|n| names.contains(n)))
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

pub(super) fn validate_resume_request(request: &ReseedRequest) -> Result<()> {
    if request.freshen
        || request.source.is_some()
        || request.source_at.is_some()
        || request.at.is_some()
        || !request.copy_ignored.is_empty()
        || request.no_copy_ignored
        || request.replace_content
    {
        bail!("--resume uses recorded preparation and target plans; overrides are not allowed");
    }
    Ok(())
}

pub(super) fn ensure_available(store: &Store, source: &Path, own: Option<&Batch>) -> Result<()> {
    for batch in batches(store)? {
        if batch.source == source && own.is_none_or(|own| own.id != batch.id) {
            bail!(
                "source has an active preparation batch in phase {:?}; use `jw reseed {:?} --resume`; source progress was not rolled back",
                batch.phase,
                batch.request.names
            );
        }
    }
    Ok(())
}

fn remote(source: &Path, settings: Option<&Settings>) -> Result<String> {
    let chosen = settings
        .and_then(|s| s.remote.as_deref())
        .unwrap_or("origin");
    if chosen.is_empty() {
        bail!("prepare.remote must not be empty");
    }
    let output = JjClient::new(source).run(["--ignore-working-copy", "git", "remote", "list"])?;
    if !output
        .stdout()?
        .lines()
        .any(|line| line.split_whitespace().next() == Some(chosen))
    {
        bail!(
            "freshen remote {chosen:?} is not configured in the source; set prepare.remote (origin fallback requires origin)"
        );
    }
    Ok(chosen.to_owned())
}

pub(super) fn preflight(
    store: &Store,
    metadata: &WorkspaceMetadataStore,
    inventory: &Inventory,
    source: &Path,
    ignored: &[String],
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
                "source is borrowed by unfinished target {}; resume its frozen reseed before freshening the donor",
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
    remote(source, settings)?;
    let command = settings.and_then(|s| s.command.as_ref());
    if command.is_some_and(|argv| argv.is_empty() || argv[0].is_empty()) {
        bail!("prepare.command must be a nonempty argv with a nonempty program");
    }
    if !ignored.is_empty() && command.is_none() {
        bail!(
            "freshening selected ignored dependency roots requires prepare.command; old installed data is not refreshed"
        );
    }
    // Validate pattern grammar, topology and missing-root policy before mutation.
    expand_ignored(source, ignored)?;
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
    rev: &str,
    settings: Option<&Settings>,
    ignored: &[String],
) -> Result<String> {
    Ok(format!(
        "Freshen donor {:?}; remote {:?}; prospective post-fetch goal {:?}; preparation argv {:?}, cwd {:?}; ignored selection {:?}; {}",
        source,
        remote(source, settings)?,
        rev,
        settings.and_then(|s| s.command.as_ref()),
        source,
        ignored,
        if settings.and_then(|s| s.command.as_ref()).is_none() {
            "tracked-source preparation only"
        } else {
            "trusted configured program (no implicit shell)"
        }
    ))
}

pub(super) fn summary(batch: &Batch) -> String {
    format!(
        "Recorded freshen: remote {:?}, exact source goal {:?}, preparation argv {:?}, cwd {:?}; {}",
        batch.remote,
        batch.goal,
        batch.command,
        batch.source,
        if batch.command.is_none() {
            "tracked-source preparation only"
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
    inventory: &Inventory,
    names: &[String],
    request: &ReseedRequest,
    rev: &str,
    settings: Option<&Settings>,
    ignored: &[String],
    local_config: Option<Vec<u8>>,
) -> Result<Batch> {
    verify_policy(default, &local_config)?;
    let operation = client.operation_id()?;
    let source_client = JjClient::new(source);
    let old_commit = source_client.resolve_one_at(&operation, "@")?.commit_id;
    require_unconflicted(&source_client, &operation, &old_commit)?;
    let mut targets = Vec::new();
    let facts = client.workspace_target_facts_at(&operation)?;
    for name in names {
        let root = inventory.root(name)?.to_path_buf();
        let fact = facts.get(name).context("target disappeared")?;
        if fact.conflicted || fact.divergent {
            bail!("workspace {name} is conflicted or divergent");
        }
        let current_entries = entries(&JjClient::new(&root), &operation, &fact.commit_id)?;
        verify_disk(
            &JjClient::new(&root),
            &operation,
            &fact.commit_id,
            &root,
            &current_entries,
        )?;
        targets.push(Target {
            name: name.clone(),
            root_identity: Identity::read(&root)?,
            jj_identity: Identity::read(&root.join(".jj"))?,
            stamp: stamp(&root)?,
            root,
            commit: fact.commit_id.clone(),
        });
    }
    let mut frozen_request = request.clone();
    frozen_request.names = names.to_vec();
    frozen_request.groups.clear();
    frozen_request.on_unmatched = None;
    frozen_request.source = Some(source.to_path_buf());
    frozen_request.source_at = Some(rev.into());
    if frozen_request.copy_ignored.is_empty() && !frozen_request.no_copy_ignored {
        frozen_request.copy_ignored = ignored.to_vec();
    }
    let nonce = tempfile::NamedTempFile::new_in(&store.root)?;
    let id = Sha256::digest(serde_json::to_vec(&(
        source,
        &operation,
        &old_commit,
        names,
        &targets,
        nonce.path(),
    ))?)
    .iter()
    .map(|b| format!("{b:02x}"))
    .collect();
    let batch = Batch {
        schema_version: 1,
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
        remote: remote(source, settings)?,
        command: settings.and_then(|s| s.command.clone()),
        source_rev: rev.into(),
        old_operation: operation,
        old_commit,
        selected_commit: None,
        goal: None,
        phase: PreparationPhase::Fetching,
        targets,
        plans: None,
        verified_source: None,
    };
    save(store, &batch)?;
    Ok(batch)
}

fn validate(batch: &Batch) -> Result<()> {
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
    let (ignored, _) = expand_ignored(&batch.source, &batch.request.copy_ignored)?;
    let source_entries = entries(&client, &operation, goal)?;
    let rules = IgnoreRules::new(&client, &operation, goal, &source_entries, true)?;
    for root in &ignored {
        rules.require_ignored(root)?;
    }
    let payload_digest = payload_digest(&batch.source, &ignored)?;
    if stamp(&batch.source)? != before || current_commit(&client)?.1 != wc_commit {
        bail!("source changed during preparation proof capture");
    }
    Ok(VerifiedSource {
        wc_commit,
        stamp: before,
        ignored,
        payload_digest,
    })
}

pub(super) fn verify_verified(batch: &Batch) -> Result<()> {
    validate(batch)?;
    let proof = batch
        .verified_source
        .as_ref()
        .context("verified preparation lacks frozen source proof")?;
    let client = JjClient::new(&batch.source);
    let (operation, current) = current_commit(&client)?;
    let before = stamp(&batch.source)?;
    let (ignored, _) = expand_ignored(&batch.source, &batch.request.copy_ignored)?;
    if current != proof.wc_commit
        || ignored != proof.ignored
        || payload_digest(&batch.source, &ignored)? != proof.payload_digest
    {
        bail!(
            "verified preparation source/payload changed before target freeze; pending retained; restore frozen verified donor rather than adopting changed dependencies"
        );
    }
    if before != proof.stamp {
        // Exact payload restoration may have new inode/ctime. Never adopt new
        // bytes: reprove the tracked tree as well as the frozen logical payload.
        let goal = batch.goal.as_ref().context("missing frozen goal")?;
        let source_entries = entries(&client, &operation, goal)?;
        verify_disk(&client, &operation, goal, &batch.source, &source_entries)?;
    }
    if stamp(&batch.source)? != before || current_commit(&client)?.1 != current {
        bail!("verified donor changed during proof validation");
    }
    Ok(())
}

pub(super) fn verify_targets(batch: &Batch) -> Result<()> {
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
    let result = (|| {
        validate(batch)?;
        verify_targets(batch)?;
        let client = JjClient::new(&batch.source);
        if batch.goal.is_none() {
            if current_commit(&client)?.1 != batch.old_commit {
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
                "Freshen: fetching remote {:?} in {:?}; previous work preserved at {}",
                batch.remote, batch.source, batch.old_commit
            ));
            let output = client.run([
                "--config=fsmonitor.backend=none",
                "git",
                "fetch",
                "--remote",
                &batch.remote,
            ])?;
            diagnostics(output.stderr());
            let operation = client.operation_id()?;
            batch.goal = Some(
                client
                    .resolve_one_at(&operation, &batch.source_rev)?
                    .commit_id,
            );
            batch.phase = PreparationPhase::Selecting;
            save(store, batch)?;
        }
        let goal = batch.goal.clone().context("missing frozen goal")?;
        let (operation, current) = current_commit(&client)?;
        require_unconflicted(&client, &operation, &goal)?;
        let goal_entries = entries(&client, &operation, &goal)?;
        // Freeze/check known future ignored policy as soon as fetch made the
        // goal knowable. Installer-created wildcard matches are checked later.
        let rules = IgnoreRules::new(&client, &operation, &goal, &goal_entries, true)?;
        let (mut selected_roots, _) = expand_ignored(&batch.source, &batch.request.copy_ignored)?;
        for pattern in &batch.request.copy_ignored {
            if !pattern.contains(['*', '?']) {
                selected_roots.push(PathBuf::from(pattern));
            }
        }
        for root in selected_roots {
            rules.require_ignored(&root)?;
            if goal_entries.iter().any(|entry| overlap(&root, &entry.path)) {
                bail!(
                    "ignored root overlaps tracked source goal: {}",
                    root.display()
                );
            }
        }
        for entry in goal_entries.iter().filter(|entry| entry.kind == "symlink") {
            symlink_bytes(&client, &operation, &goal, &entry.path)?;
        }
        if batch.selected_commit.is_none() {
            if current != batch.old_commit
                && !exact_empty_child(&client, &operation, &current, &goal)?
            {
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
                let output = client.run(["--config=fsmonitor.backend=none", "new", &goal])?;
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
                "Freshen: exact post-fetch goal {goal}; preparation argv {:?}, cwd {:?}; {}",
                batch.command,
                batch.source,
                if batch.command.is_none() {
                    "tracked-source preparation only"
                } else {
                    "no implicit shell; command may be rerun on resume"
                }
            ));
            batch.phase = PreparationPhase::Preparing;
            save(store, batch)?;
            validate(batch)?;
            if let Some(argv) = &batch.command {
                let status = Command::new(&argv[0])
                    .args(&argv[1..])
                    .current_dir(&batch.source)
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .context("could not run configured preparation argv")?;
                if !status.success() {
                    bail!(
                        "configured preparation command failed with {status}; ignored installation may be incomplete"
                    );
                }
            }
            validate(batch)?;
            snapshot(&client)?;
            let (operation, current) = current_commit(&client)?;
            if !changed_paths(&client, &operation, &current, &goal)?.is_empty() {
                bail!(
                    "preparer changed tracked tree (including lockfiles); restore frozen source child/tree before resume"
                );
            }
            verify_disk(&client, &operation, &goal, &batch.source, &goal_entries)?;
            batch.selected_commit = Some(current);
            validate(batch)?;
            verify_targets(batch)?;
            batch.verified_source = Some(capture_verified(batch)?);
            batch.phase = PreparationPhase::Verified;
            save(store, batch)?;
        }
        verify_verified(batch)?;
        Ok(())
    })();
    result.with_context(|| format!("source preparation stopped in phase {:?}, frozen goal {:?}; source progress was not rolled back; targets not refilled; resume recorded batch {:?}", batch.phase, batch.goal, batch.request.names))
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
