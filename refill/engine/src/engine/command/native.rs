use super::*;
use jj_waltz_native_jj::jj::{adopt_tree_state, read_tree_state};
mod listed;
use listed::TrackedPhase;

#[derive(Debug, Default)]
pub struct NativePathIndex {
    pub paths: Arc<BTreeSet<PathBuf>>,
}
impl NativePathIndex {
    pub fn contains(&self, path: &Path) -> bool {
        self.paths.contains(path)
    }
    pub fn descendants(&self, path: &Path) -> bool {
        self.paths.range(path.to_path_buf()..).next().is_some_and(|p| p.starts_with(path))
    }
    pub fn overlaps(&self, path: &Path) -> bool {
        path.ancestors().any(|p| self.contains(p)) || self.descendants(path)
    }
}
#[derive(Debug)]
pub struct NativeTree {
    pub key: String,
    pub leaves: Arc<Vec<Entry>>,
    pub index: Arc<NativePathIndex>,
}

/// Consumed by the native stage. Nothing in the ordinary cohort retains donor
/// cache bytes or O membership. S/D exclusion paths are deliberately separate.
pub(super) struct NativeInputs {
    pub source: Arc<NativeTree>,
    pub desired: Arc<NativeTree>,
    pub old_trees: BTreeMap<String, Arc<NativeTree>>,
    pub donor_tree_state: Vec<u8>,
    pub additions: Vec<Entry>,
    pub tracked: TrackedPhase,
}

pub(super) struct Membership {
    pub keys: BTreeMap<String, String>,
    trees: BTreeMap<String, Arc<NativeTree>>,
}
impl Membership {
    pub fn capture(client: &JjClient, operation: &str, commits: &[String]) -> Result<Self> {
        let mut facts = BTreeMap::new();
        for commit in commits {
            if !facts.contains_key(commit) {
                let fact = client.commit_facts_at(operation, &commit_id_revset(commit)?)?;
                ensure!(!fact.conflicted && !fact.divergent, "unsupported conflicted/divergent native revision {commit}");
                facts.insert(commit.clone(), fact);
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
            result.keys.insert(commit.clone(), key.to_owned());
            if !result.trees.contains_key(key) {
                let leaves = Arc::new(tracked::entries(client, operation, commit)?);
                let index = Arc::new(NativePathIndex {
                    paths: Arc::new(leaves.iter().map(|e| e.path.clone()).collect()),
                });
                result.trees.insert(
                    key.to_owned(),
                    Arc::new(NativeTree {
                        key: key.to_owned(),
                        leaves,
                        index,
                    }),
                );
            }
        }
        Ok(result)
    }
    pub fn get(&self, commit: &str) -> Arc<NativeTree> {
        Arc::clone(&self.trees[&self.keys[commit]])
    }
    pub fn count(&self) -> usize {
        self.trees.len()
    }
}

pub(super) struct SavedReceiver {
    pub binding: Arc<BoundWorkspace>,
    pub old_commit: String,
    pub guard: bookmarks::Guard,
    pub links: LoadedWorkspaceLinks,
}

pub(super) fn inputs(
    membership: &Membership,
    source: &PreparedSource,
    desired: &str,
    saved: &[SavedReceiver],
    route: RenewalRoute,
    replace_content: bool,
    capture: &mut CaptureCounts,
) -> Result<NativeInputs> {
    let source_tree = membership.get(&source.report.commit);
    let desired_tree = membership.get(desired);
    let additions = desired_tree.leaves.iter().filter(|e| !source_tree.index.contains(&e.path)).cloned().collect();
    let donor_tree_state = read_tree_state(&source.binding.root)?;
    capture.donor_cache_reads = 1;
    let tracked = TrackedPhase::prepare(membership, source, saved, route, replace_content)?;
    capture.desired_manifests = 1;
    capture.owned_manifests = tracked.owned_count();
    Ok(NativeInputs {
        source: source_tree,
        desired: desired_tree,
        old_trees: saved.iter().map(|s| (s.old_commit.clone(), membership.get(&s.old_commit))).collect(),
        donor_tree_state,
        additions,
        tracked,
    })
}

/// One complete native transition, called serially for a vector of receivers.
/// Both routes use the provider's complete manifest executor. Native authority,
/// adoption and JJ convergence remain here; there is no JW filesystem copier.
pub(super) fn establish(
    source: &PreparedSource,
    desired: &str,
    saved: &SavedReceiver,
    input: &NativeInputs,
    replace_content: bool,
    report: &mut ReceiverReport,
) -> Result<()> {
    let started = Instant::now();
    report.status = ReceiverStatus::Partial;
    report.native.state = Establishment::InProgress;
    report.native.source_state_adopted = Some(false);
    let result = (|| {
        let binding = &saved.binding;
        let client = binding.client();
        // A preceding native receiver may rewrite history. Never silently repin
        // an O captured by the preservation barrier.
        // This is the per-receiver guard portion of Capture, not another
        // membership extraction or an additional public wire phase.
        let old = measured(report, Phase::Capture, |_| {
            let operation = client.operation_id()?;
            source.binding.verify_at(&operation)?;
            ensure!(
                source.binding.client().resolve_one_at(&operation, "@")?.commit_id == source.report.commit,
                "donor coordinate changed before native transition"
            );
            binding.verify_at(&operation)?;
            let old = client.resolve_one_at(&operation, "@")?;
            ensure!(old.commit_id == saved.old_commit, "saved receiver O moved before native transition");
            bookmarks::verify_batch(&client, [&saved.guard])?;
            Ok(old)
        })?;
        report.native.temporary_source_installation = source.report.commit != desired;
        if report.route == RenewalRoute::ClearThenPopulate {
            measured(report, Phase::NativeClear, |_| {
                client.run_refill(["sparse", "set", "--clear"])?;
                ensure!(
                    client.resolve_one("@")?.commit_id == saved.old_commit,
                    "receiving O changed during native clear snapshot"
                );
                refuse_residual(&binding.root, &input.old_trees[&saved.old_commit].leaves)
            })?;
        }
        let route = report.route;
        let parent = if route == RenewalRoute::ClearThenPopulate {
            desired
        } else {
            &saved.old_commit
        };
        let child = measured(report, Phase::NativeChild, |report| {
            client.run_refill(["new", &commit_id_revset(parent)?])?;
            let operation = client.operation_id()?;
            let child = client.commit_facts_at(&operation, "@")?;
            report.child = Some(child.clone());
            ensure!(
                child.parents == [parent] && child.ordinary_empty_child(),
                "new child does not have its exact native parent"
            );
            ensure!(
                child.change_id != old.change_id && child.change_id != source.report.change,
                "new receiving child reused a source/old change identity"
            );
            if route == RenewalRoute::ReplaceAuthorized {
                ensure!(
                    client
                        .resolve_one_at(&operation, format!("change_id({})", serde_json::to_string(&old.change_id)?))?
                        .commit_id
                        == saved.old_commit,
                    "saved O was rewritten during same-tree child creation"
                );
            }
            binding.verify_at(&operation)?;
            Ok(child)
        })?;
        if replace_content {
            measured(report, Phase::ExplicitClear, |report| {
                match diff_dirs_sync::clear_with_visitor(&binding.root, cow::transfer::WorkspaceClear) {
                    Ok(done) => report.native.explicit_clear = Some(done.completed),
                    Err(failure) => {
                        report.native.explicit_clear = Some(failure.completed.clone());
                        return Err(failure.into());
                    }
                }
                Ok(())
            })?;
        }
        measured(report, Phase::Install, |report| input.tracked.install(saved, report))?;
        measured(report, Phase::AdmitDesired, |_| {
            admit_additions(&binding.root, &input.source.index, &input.additions)
        })?;
        measured(report, Phase::Adopt, |_| adopt_tree_state(&binding.root, &input.donor_tree_state))?;
        report.native.source_state_adopted = Some(true);
        if source.report.commit != desired {
            measured(report, Phase::Restore, |_| {
                client.run_refill(["restore", "--from", &commit_id_revset(desired)?])?;
                Ok(())
            })?;
        }
        if report.route == RenewalRoute::ReplaceAuthorized {
            measured(report, Phase::Reparent, |report| {
                let operation = client.operation_id()?;
                binding.verify_at(&operation)?;
                let current = client.commit_facts_at(&operation, "@")?;
                report.child = Some(current.clone());
                ensure!(
                    current.change_id == child.change_id && current.parents == [saved.old_commit.as_str()],
                    "new receiving C changed before reparenting"
                );
                let descendants = client.run_at(
                    &operation,
                    [
                        "log",
                        "--no-graph",
                        "-r",
                        &format!("({})::", commit_id_revset(&current.commit_id)?),
                        "-T",
                        "commit_id ++ \"\\n\"",
                    ],
                )?;
                ensure!(
                    descendants.stdout()?.lines().collect::<Vec<_>>() == [current.commit_id.as_str()],
                    "new receiving C acquired outside descendants"
                );
                client.run_refill(["rebase", "-r", "@", "-d", &commit_id_revset(desired)?])?;
                Ok(())
            })?;
        }
        let final_child = measured(report, Phase::NativeVerify, |_| {
            native_diff(&client, desired)?;
            verify_native(binding, desired, &child.change_id)
        })?;
        report.child = Some(final_child);
        report.native.state = Establishment::Established;
        Ok(())
    })();
    report.native.elapsed = Some(started.elapsed());
    result
}

fn measured<T>(report: &mut ReceiverReport, phase: Phase, run: impl FnOnce(&mut ReceiverReport) -> Result<T>) -> Result<T> {
    report.reached = phase;
    report.native.attempted.push(phase);
    let started = Instant::now();
    let result = run(report);
    report.native.timings.push(PhaseTiming {
        phase,
        elapsed: started.elapsed(),
        succeeded: result.is_ok(),
    });
    result
}

pub(super) fn native_diff(client: &JjClient, desired: &str) -> Result<()> {
    let output = client.run_refill(["diff", "--from", "@", "--to", &commit_id_revset(desired)?, "--summary"])?;
    ensure!(
        output.stdout_bytes().is_empty(),
        "native tracked tree differs from exact desired tree (snapshot performed): {}",
        output.stdout()?
    );
    Ok(())
}
pub(super) fn verify_native(binding: &BoundWorkspace, desired: &str, change: &str) -> Result<CommitFacts> {
    let client = binding.client();
    let operation = client.operation_id()?;
    binding.verify_at(&operation)?;
    binding::require_full_at(&client, &operation)?;
    let facts = client.commit_facts_at(&operation, "@")?;
    let bindings = client.workspace_bindings_at(&operation)?;
    ensure!(
        bindings
            .get(&binding.name)
            .is_some_and(|b| b.commit_id == facts.commit_id && b.change_id == change),
        "receiving registration/change identity changed"
    );
    ensure!(
        facts.ordinary_empty_child() && facts.parents == [desired] && facts.change_id == change,
        "receiver is not an exact full-sparse empty child of D with its own change identity"
    );
    Ok(facts)
}
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
fn admit_additions(root: &Path, source: &NativePathIndex, additions: &[Entry]) -> Result<()> {
    for entry in additions {
        let mut path = PathBuf::new();
        for part in entry.path.components() {
            path.push(part);
            match fs::symlink_metadata(root.join(&path)) {
                Ok(_) if source.contains(&path) => break,
                Ok(m) if m.is_dir() && path != entry.path => {}
                Ok(m) if m.is_dir() && source.descendants(&path) => break,
                Ok(_) => bail!("D-only path obstructed by unrelated receiving name: {}", path.display()),
                Err(e) if e.kind() == ErrorKind::NotFound => break,
                Err(e) => return Err(e.into()),
            }
        }
    }
    Ok(())
}
