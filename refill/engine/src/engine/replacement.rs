//! Callable first vertical slice. Not a second CLI mode: the public driver moves
//! here only after conditional routes and cohort/preparation integration land.
use super::{Identity, Inventory, bookmarks, checked_context, ensure_quiescence, load_config, repository, require_full, selection, snapshot, validate_target};
use crate::ReseedRequest;
use anyhow::{Context, Result, bail, ensure};
use jj_waltz_native_jj::jj::{CommitFacts, JjClient, JjVersion, adopt_tree_state, commit_id_revset, read_tree_state};
use jj_waltz_transfer_workspace::{
    self as cow,
    frozen::FrozenIgnores,
    ignored::{Patterns, Policy},
    streaming::{SourceQualified, sync_one},
    topology::Topology,
    tracked::{self, Entry},
};
use jj_waltz_workspace_state::{
    config::Config,
    links::{self, LoadedWorkspaceLinks},
    metadata::WorkspaceMetadataStore,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

#[derive(Debug)]
pub struct RefillOneReport {
    pub name: String,
    pub source_commit: String,
    pub desired_commit: String,
    pub old_commit: String,
    pub final_commit: String,
    pub change_id: String,
    pub native_install: diff_dirs_sync::Progress,
    pub explicit_clear: diff_dirs_sync::Progress,
    pub ordinary: Option<diff_dirs_sync::coordinated::LaneReport>,
    pub links: links::LinkApplyReport,
    pub native_ms: u128,
    pub ordinary_ms: u128,
    pub finish_ms: u128,
    pub elapsed_ms: u128,
}

/// Execute one real N1 refill from an explicit caller directory. Supports plain
/// execution only in this integration slice. All filesystem/native work is real;
/// no payload discovery or dry-run preflight is inserted before execution.
pub fn refill_one_from(cwd: &Path, request: ReseedRequest, mut diagnostic: impl FnMut(&str)) -> Result<RefillOneReport> {
    let started = Instant::now();
    ensure!(
        !request.prepare && !request.dry_run,
        "N1 integration seam does not yet implement prepare/dry-run"
    );
    ensure!(
        std::env::var_os("GIT_CONFIG_PARAMETERS").is_none_or(|v| v.is_empty()) && std::env::var_os("GIT_CONFIG_COUNT").is_none_or(|v| v.is_empty() || v == "0"),
        "unset active Git config environment overrides before reseed"
    );
    let caller = JjClient::new(cwd);
    // Exact tested cache profile, not a fallback ladder or a compatibility claim
    // for all the versions supported by unrelated lifecycle commands.
    let version = caller.version()?;
    ensure!(
        version == JjVersion::new(0, 45, 1),
        "source-state refill currently supports tested JJ 0.45.1 only; found {version}"
    );
    let caller_root = PathBuf::from(caller.run(["--ignore-working-copy", "workspace", "root"])?.trimmed_stdout()?).canonicalize()?;
    let caller = JjClient::new(caller_root);
    let inventory = Inventory::load(&caller).context("resolve native registrations")?;
    let config = load_config(&inventory.default)?;
    let raw_source = request.source.as_ref().or(config.source.as_ref()).unwrap_or(&inventory.default);
    let source = if raw_source.is_absolute() {
        raw_source.clone()
    } else if request.source.is_some() {
        cwd.join(raw_source)
    } else {
        inventory.default.join(raw_source)
    }
    .canonicalize()?;
    let mut request = request;
    if request.names.is_empty() && request.groups.is_empty() {
        ensure!(
            inventory.root(&inventory.current)? != inventory.default,
            "primary checkout requires literal target names or --group"
        );
        request.names.push(inventory.current.clone());
    }
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let selection = selection::resolve(&request, &config.groups, &inventory.registered, &inventory.default, home.as_deref(), &source)?;
    if let Some(warning) = selection.warning() {
        diagnostic(&warning);
    }
    selection.validate()?;
    let [selected] = selection.selections.as_slice() else {
        bail!("N1 integration seam requires exactly one receiver");
    };
    let name = selected.name.clone();
    let root = inventory.root(&name)?.canonicalize()?;
    let client = JjClient::new(&root);
    let donor = JjClient::new(&source);
    let metadata = WorkspaceMetadataStore::from_repo_config_path(caller.repo_config_path()?)?;
    let repo = repository(&caller)?;
    let source_context = checked_context(&donor)?;
    ensure!(
        source_context.jj.workspace_root.as_ref() == Some(&source) && repository(&donor)? == repo,
        "donor root/repository binding differs"
    );
    validate_target(&caller, &metadata, &inventory.default, &source, &name, &root)?;
    let source_identity = Identity::read(&source)?;
    let source_jj = Identity::read(&source.join(".jj"))?;
    let identity = Identity::read(&root)?;
    let jj_identity = Identity::read(&root.join(".jj"))?;
    for native in [&donor, &client] {
        ensure_quiescence(native)?;
        require_full(native)?;
        Identity::read(&native.cwd().join(".jj/working_copy"))?;
        ensure!(
            fs::read(native.cwd().join(".jj/working_copy/type"))? == b"local",
            "source-state refill requires native local working copies"
        );
        for file in ["type", "checkout", "tree_state"] {
            ensure!(
                fs::symlink_metadata(native.cwd().join(".jj/working_copy").join(file))?.is_file(),
                "unsupported native state layout"
            );
        }
    }
    let topology = Topology::capture()?;
    topology.workspace(&source, true)?;
    topology.workspace(&root, false)?;
    topology.separate(&[source.clone(), root.clone()])?;
    cow::probe_between(&source, &root).context("required CoW route admission")?;
    let inherited = Config::load_inherited_copy_ignored(request.copy_ignored.as_deref(), config.copy_ignored.as_deref(), request.no_copy_ignored)?;
    let policy = Policy::resolve(
        request.copy_ignored.as_deref(),
        config.copy_ignored.as_deref(),
        inherited.as_deref(),
        request.no_copy_ignored,
    );
    let patterns = Arc::new(Patterns::compile(&policy.patterns)?);
    let loaded_config = links::load_link_config(&inventory.default)?;
    let loaded_links = LoadedWorkspaceLinks::new(loaded_config.as_ref(), &root)?;
    let active_links = loaded_links.active_sources();
    let native_started = Instant::now();
    let operation = donor.operation_id()?;
    let assertion = request
        .source_at
        .as_ref()
        .or(config.source_at.as_ref())
        .map(|a| donor.resolve_one_at(&operation, a))
        .transpose()?;
    if let Some(assertion) = assertion {
        native_diff(&donor, &assertion.commit_id).context("explicit source_at live assertion (may have recorded donor edits)")?;
    }
    let operation = donor.operation_id()?;
    let source_commit = donor.resolve_one_at(&operation, "@")?.commit_id;
    let desired = request
        .at
        .as_ref()
        .map(|r| caller.resolve_one_at(&operation, r))
        .transpose()?
        .map_or_else(|| source_commit.clone(), |r| r.commit_id);

    // The only explicit preservation snapshot. Native clear will snapshot again;
    // new snapshots empty-sparse, final diff snapshots adopted full S or D.
    snapshot(&client).context("save old receiving work")?;
    let operation = caller.operation_id()?;
    let bindings = caller.workspace_bindings_at(&operation)?;
    let old = bindings.get(&name).context("receiver disappeared during preservation")?.commit_id.clone();
    diagnostic(&format!("{}: saved old tracked work at {old}", name));
    let _bookmark_guard = if request.allow_unbookmarked {
        bookmarks::Guard::AllowUnbookmarked
    } else {
        bookmarks::Coverage::capture(&caller, &operation, &BTreeSet::from([old.clone()]))?.require(&old)?
    };
    ensure!(
        donor.resolve_one_at(&operation, "@")?.commit_id == source_commit,
        "donor coordinate changed during receiving snapshots; refused before clearing"
    );
    let membership =
        Membership::capture(&caller, &operation, &[&source_commit, &desired, &old]).context("native membership capture after saving O; no receiver cleared")?;
    let source_entries = membership.get(&source_commit);
    let desired_entries = membership.get(&desired);
    let old_entries = membership.get(&old);
    let native = Arc::new(
        source_entries
            .iter()
            .chain(desired_entries.iter())
            .map(|e| e.path.clone())
            .collect::<BTreeSet<_>>(),
    );
    for link in &active_links {
        ensure!(
            !overlap(&native, link),
            "configured active link overlaps S/D tracked ownership: {}",
            link.display()
        );
    }
    let state = read_tree_state(&source)?;
    let mut ignores = FrozenIgnores::default();
    for (commit, key) in &membership.keys {
        ignores.same_tree(commit, key);
    }
    let visitor = if !patterns.is_empty() || !active_links.is_empty() {
        let source_ignore = ignores.get(&donor, &operation, &source_commit, source_entries)?;
        let desired_ignore = ignores.get(&client, &operation, &desired, desired_entries)?;
        Some(SourceQualified {
            patterns,
            source_ignore,
            desired_ignore,
            native: Arc::clone(&native),
            links: active_links,
            devices: (source_identity.device, identity.device),
        })
    } else {
        None
    };

    let mut phase = "clear tracked checkout";
    let mut installed = diff_dirs_sync::Progress::default();
    let mut cleared = diff_dirs_sync::Progress::default();
    let mut ordinary = None;
    let result = (|| {
        client.run_refill(["sparse", "set", "--clear"])?;
        ensure!(client.resolve_one("@")?.commit_id == old, "receiving O changed during native clear snapshot");
        refuse_residual(&root, old_entries)?;
        phase = "establish receiving child";
        client.run_refill(["new", &commit_id_revset(&desired)?])?;
        let child = current_facts(&client)?;
        ensure!(child.parents == [desired.clone()], "new child does not have exact D parent");
        if request.replace_content {
            phase = "explicit replace-content";
            cleared = match diff_dirs_sync::clear_with_visitor(&root, cow::transfer::WorkspaceClear) {
                Ok(report) => report.completed,
                Err(failure) => {
                    cleared = failure.completed.clone();
                    return Err(failure.into());
                }
            };
        }
        phase = "strict full-S install";
        installed = match tracked::seed(&source, &root, source_entries, (source_identity.device, identity.device)) {
            Ok(progress) => progress,
            Err(failure) => {
                installed = failure.completed.clone();
                return Err(failure.into());
            }
        };
        phase = "D-addition namespace admission";
        admit_additions(&root, source_entries, desired_entries)?;
        phase = "adopt captured source tree_state";
        adopt_tree_state(&root, &state)?;
        if source_commit != desired {
            phase = "native S-to-D restore";
            client.run_refill(["restore", "--from", &commit_id_revset(&desired)?])?;
        }
        phase = "final native diff";
        native_diff(&client, &desired)?;
        let final_facts = verify_native(&client, &name, &repo, &desired, &child.change_id, &identity, &jj_identity)?;
        let native_ms = native_started.elapsed().as_millis();
        phase = "ordinary coordinated transfer";
        let ordinary_started = Instant::now();
        ordinary = match visitor {
            Some(visitor) => sync_one(&source, &root, &name, visitor)?,
            None => None,
        };
        let ordinary_ms = ordinary_started.elapsed().as_millis();
        phase = "loaded link finish";
        let finish_started = Instant::now();
        if let Some(rules) = ignores_for_links(&mut ignores, &client, &operation, &desired, desired_entries, &loaded_links)? {
            for check in loaded_links.inspect() {
                if check.source != check.target && check.target_exists {
                    ensure!(
                        rules.ignored(check.source.strip_prefix(&root)?, false),
                        "active link is not ignored under D: {}",
                        check.source.display()
                    );
                }
            }
        }
        let links = links::apply_loaded_workspace_links(&loaded_links)?;
        phase = "finish native identity";
        // No new snapshot: enforced ordinary/link exclusions cannot alter D.
        let finished = verify_native(&client, &name, &repo, &desired, &child.change_id, &identity, &jj_identity)?;
        ensure!(finished.commit_id == final_facts.commit_id, "native commit changed during ordinary/link finish");
        source_identity.verify(&source)?;
        source_jj.verify(&source.join(".jj"))?;
        Ok(RefillOneReport {
            name: name.clone(),
            source_commit: source_commit.clone(),
            desired_commit: desired.clone(),
            old_commit: old.clone(),
            final_commit: finished.commit_id,
            change_id: child.change_id,
            native_install: installed.clone(),
            explicit_clear: cleared.clone(),
            ordinary: ordinary.clone(),
            links,
            native_ms,
            ordinary_ms,
            finish_ms: finish_started.elapsed().as_millis(),
            elapsed_ms: started.elapsed().as_millis(),
        })
    })();
    result.with_context(|| format!("reseed {name}: phase {phase}; saved O={old}; completed native install={installed:?}, explicit clear={cleared:?}, ordinary={ordinary:?}; partial effects remain, no rollback"))
}

fn ignores_for_links(
    cache: &mut FrozenIgnores,
    client: &JjClient,
    operation: &str,
    desired: &str,
    entries: &[Entry],
    links: &LoadedWorkspaceLinks,
) -> Result<Option<Arc<cow::frozen::FrozenIgnore>>> {
    if links.active_sources().is_empty() {
        return Ok(None);
    }
    Ok(Some(cache.get(client, operation, desired, entries)?))
}

fn native_diff(client: &JjClient, desired: &str) -> Result<()> {
    let output = client.run_refill(["diff", "--from", "@", "--to", &commit_id_revset(desired)?, "--summary"])?;
    ensure!(
        output.stdout_bytes().is_empty(),
        "native tracked tree differs from exact desired tree (snapshot performed): {}",
        output.stdout()?
    );
    Ok(())
}

fn current_facts(client: &JjClient) -> Result<CommitFacts> {
    client.commit_facts_at(&client.operation_id()?, "@")
}

fn verify_native(client: &JjClient, name: &str, repo: &Path, desired: &str, change: &str, identity: &Identity, jj_identity: &Identity) -> Result<CommitFacts> {
    identity.verify(client.cwd())?;
    jj_identity.verify(&client.cwd().join(".jj"))?;
    ensure!(repository(client)? == repo, "receiving native topology changed");
    match fs::symlink_metadata(client.cwd().join(".git")) {
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
        Ok(_) => bail!("receiving workspace acquired Git administration"),
    }
    require_full(client)?;
    let operation = client.operation_id()?;
    let bindings = client.workspace_bindings_at(&operation)?;
    let facts = client.commit_facts_at(&operation, "@")?;
    ensure!(
        bindings.get(name).is_some_and(|b| b.commit_id == facts.commit_id && b.change_id == change),
        "receiving registration/change identity changed"
    );
    let current = client.current_workspace_target_names()?;
    ensure!(current == [name], "receiving workspace name changed");
    let actual_root = client.run_at(&operation, ["workspace", "root", "--name", name])?;
    ensure!(
        PathBuf::from(actual_root.trimmed_stdout()?).canonicalize()? == client.cwd(),
        "receiving registered root changed"
    );
    ensure!(
        facts.ordinary_empty_child() && facts.parents == [desired] && facts.change_id == change,
        "receiver is not an exact full-sparse empty child of D with its own change identity"
    );
    Ok(facts)
}

fn overlap(paths: &BTreeSet<PathBuf>, path: &Path) -> bool {
    path.ancestors().any(|p| paths.contains(p)) || paths.range(path.to_path_buf()..).next().is_some_and(|p| p.starts_with(path))
}

/// Shared revision membership. Native empty-single-parent facts supply a cheap
/// whole-tree equality proof, so ordinary empty children of S/D do not cause a
/// second file list or ignore extraction. No changed-path query is needed.
struct Membership {
    keys: BTreeMap<String, String>,
    trees: BTreeMap<String, Arc<Vec<Entry>>>,
}
impl Membership {
    fn capture(client: &JjClient, operation: &str, commits: &[&String]) -> Result<Self> {
        let mut facts = BTreeMap::new();
        for commit in commits {
            if !facts.contains_key(*commit) {
                let fact = client.commit_facts_at(operation, &commit_id_revset(commit)?)?;
                ensure!(!fact.conflicted && !fact.divergent, "unsupported conflicted/divergent native revision {commit}");
                facts.insert((*commit).clone(), fact);
            }
        }
        let mut result = Self {
            keys: BTreeMap::new(),
            trees: BTreeMap::new(),
        };
        for commit in commits {
            let mut key = commit.as_str();
            while let Some(fact) = facts.get(key) {
                if !fact.empty || fact.parents.len() != 1 {
                    break;
                }
                key = &fact.parents[0];
            }
            result.keys.insert((*commit).clone(), key.to_owned());
            if !result.trees.contains_key(key) {
                result.trees.insert(key.to_owned(), Arc::new(tracked::entries(client, operation, commit)?));
            }
        }
        Ok(result)
    }
    fn get(&self, commit: &str) -> &Arc<Vec<Entry>> {
        &self.trees[&self.keys[commit]]
    }
}

/// Listed coordinates only; never repair a skipped native clear by deleting a
/// local name or enumerating the descendants of a former tracked leaf.
fn refuse_residual(root: &Path, old: &[Entry]) -> Result<()> {
    for entry in old {
        tracked::confined_parents(root, &entry.path)?;
        match fs::symlink_metadata(root.join(&entry.path)) {
            Ok(_) => bail!("recorded O path remains after native clear: {}", entry.path.display()),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

fn admit_additions(root: &Path, source: &[Entry], desired: &[Entry]) -> Result<()> {
    let source = source.iter().map(|e| e.path.clone()).collect::<BTreeSet<_>>();
    for entry in desired {
        if source.contains(&entry.path) {
            continue;
        }
        let mut path = PathBuf::new();
        for part in entry.path.components() {
            path.push(part);
            match fs::symlink_metadata(root.join(&path)) {
                // S-owned leaves can undergo native leaf/directory transitions.
                Ok(_) if source.contains(&path) => break,
                Ok(m) if m.is_dir() && path != entry.path => {}
                Ok(m) if m.is_dir() && source.range(path.clone()..).next().is_some_and(|p| p.starts_with(&path)) => break,
                Ok(_) => bail!("D-only path obstructed by unrelated receiving name: {}", path.display()),
                Err(e) if e.kind() == ErrorKind::NotFound => break,
                Err(e) => return Err(e.into()),
            }
        }
    }
    Ok(())
}
