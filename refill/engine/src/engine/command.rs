//! Shared command boundary. CLI cutover and contextual provider adoption are
//! deliberately separate; there is only one vector-ready native orchestration.
use super::{Identity, LocalConfig, PreparationLock, bookmarks, load_config, prepare, selection, snapshot};
use crate::ReseedRequest;
use anyhow::{Context, Result, bail, ensure};
use jj_waltz_native_jj::{
    jj::{CommitFacts, JjClient, commit_id_revset},
    refill::{NativeProfile, RegistrySnapshot},
};
use jj_waltz_transfer_workspace::{
    self as cow,
    frozen::{FrozenIgnore, FrozenIgnores},
    ignored::{Patterns, Policy},
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

mod binding;
mod native;
mod ordinary;
mod report;
mod source;
use binding::Resolution;
pub use binding::{BoundWorkspace, RepositoryBinding};
use native::{Membership, SavedReceiver, native_diff};
pub use native::{NativePathIndex, NativeTree};
pub use ordinary::ContextualOrdinary;
pub use report::*;
pub use source::*;

pub struct ResolvedCommand {
    started: Instant,
    resolution: Resolution,
    request: ReseedRequest,
    pub route: RenewalRoute,
    pub source: ResolvedSource,
    pub receivers: Vec<Arc<BoundWorkspace>>,
    patterns: Arc<Patterns>,
    pattern_declarations: Vec<String>,
    links: Vec<LoadedWorkspaceLinks>,
}

/// Ordinary data has one cohort-shaped call seam, not an N1 loop. Its owner must
/// update actual lane outcomes before returning a failure. No native-only inputs
/// (O membership or reusable cache bytes) cross this interface.
pub trait OrdinaryTransfer {
    fn admit(&self, receivers: usize, enabled: bool) -> Result<()>;
    fn transfer(
        &mut self,
        cohort: &OrdinaryCohort,
        outcomes: &mut [OrdinaryOutcome],
    ) -> std::result::Result<Option<diff_dirs_inspect::SourceDiagnostics>, OrdinaryFailure>;
}
#[derive(Debug)]
pub struct OrdinaryFailure {
    pub lane: Option<usize>,
    pub diagnostics: Option<Box<diff_dirs_inspect::SourceDiagnostics>>,
    pub source: anyhow::Error,
}
pub struct OrdinaryCohort {
    pub source: Arc<BoundWorkspace>,
    pub patterns: Arc<Patterns>,
    pub native: Arc<NativePathIndex>,
    pub lanes: Vec<OrdinaryLane>,
}
pub struct OrdinaryLane {
    pub binding: Arc<BoundWorkspace>,
    pub source_ignore: Arc<FrozenIgnore>,
    pub desired_ignore: Arc<FrozenIgnore>,
    pub active_links: BTreeSet<PathBuf>,
}

#[derive(Debug)]
pub enum DesiredPlan {
    Recorded(String),
    AfterLiveSourceAssertion,
    FuturePreparedSource,
    PostPreparationRevset(String),
}
#[derive(Debug)]
pub struct CommandPlan {
    pub profile: NativeProfile,
    pub label: &'static str,
    pub source: SourcePlan,
    pub desired: DesiredPlan,
    pub route: RenewalRoute,
    pub receivers: Vec<(String, PathBuf)>,
    pub patterns: Vec<String>,
    pub clone_capability_probed: bool,
    pub preservation_pending: bool,
    pub obstructions_inspected: bool,
}

impl ResolvedCommand {
    pub fn resolve(cwd: &Path, mut request: ReseedRequest, route: RenewalRoute, diagnostic: &mut dyn FnMut(&str)) -> Result<Self> {
        let started = Instant::now();
        let resolution = Resolution::capture(cwd, &request, false)?;
        let registry = &resolution.registry;
        if request.names.is_empty() && request.groups.is_empty() {
            ensure!(
                registry.current_name != registry.primary_name,
                "primary checkout requires literal target names or --group"
            );
            request.names.push(registry.current_name.clone());
        }
        let home = std::env::var_os("HOME").map(PathBuf::from);
        let selected = selection::resolve(
            &request,
            &resolution.config.groups,
            &registry.registered,
            &registry.primary_root,
            home.as_deref(),
            &resolution.source.root,
        )?;
        if let Some(warning) = selected.warning() {
            diagnostic(&warning);
        }
        selected.validate()?;
        let inherited = Config::load_inherited_copy_ignored(
            request.copy_ignored.as_deref(),
            resolution.config.copy_ignored.as_deref(),
            request.no_copy_ignored,
        )?;
        let policy = Policy::resolve(
            request.copy_ignored.as_deref(),
            resolution.config.copy_ignored.as_deref(),
            inherited.as_deref(),
            request.no_copy_ignored,
        );
        let patterns = Arc::new(Patterns::compile(&policy.patterns)?);
        let link_config = links::load_link_config(&registry.primary_root)?;
        let registrations = jj_waltz_workspace_state::ownership::refill_registrations_at(&resolution.caller, &registry.operation)?;
        let topology = Topology::capture()?;
        topology.workspace(&resolution.source.root, true)?;
        let mut roots = vec![resolution.source.root.clone()];
        let mut receivers = Vec::new();
        let mut links = Vec::new();
        for selected in selected.selections {
            let root = registry.roots.get(&selected.name).context("receiver has no usable registered root")?;
            ensure!(
                root != &registry.primary_root && root != &resolution.source.root,
                "primary/source is not an eligible receiver"
            );
            let binding = Arc::new(BoundWorkspace::capture(root, registry, Arc::clone(&resolution.source.repository), true)?);
            ensure!(binding.name == selected.name, "selected root registration differs");
            resolution.check_ownership(&binding, &registrations)?;
            topology.workspace(root, false)?;
            roots.push(root.clone());
            links.push(LoadedWorkspaceLinks::new(link_config.as_ref(), root)?);
            receivers.push(binding);
        }
        topology.separate(&roots)?;
        ensure!(!receivers.is_empty(), "select at least one workspace");
        if request.prepare {
            resolution.check_ownership(&resolution.source, &registrations)?;
        }
        let source = ResolvedSource::from_resolution(&resolution, &request);
        Ok(Self {
            started,
            resolution,
            request,
            route,
            source,
            receivers,
            patterns,
            pattern_declarations: policy.patterns,
            links,
        })
    }

    /// Returns before probes, producers, snapshots, membership, runtime creation,
    /// metadata initialization or payload inspection. All helper JJ reads are
    /// frozen; divergent operation heads are refused rather than reconciled.
    pub fn preview(&self) -> Result<CommandPlan> {
        let client = self.source.binding.client();
        let operation = client.operation_id()?;
        let source = if self.request.prepare {
            self.source.preview()?
        } else {
            let commit = client.resolve_one_at(&operation, "@")?.commit_id;
            SourcePlan {
                recorded_commit: commit.clone(),
                prospective: None,
                assertion_pending: self.source.assertion.is_some(),
                argv: Vec::new(),
                resulting_commit: self.source.assertion.is_none().then_some(commit),
                evidence: self.source.assertion.is_none().then_some(SourceEvidence::AssumedPrepared),
            }
        };
        let desired = match (&self.request.at, self.request.prepare) {
            (Some(revset), true) => DesiredPlan::PostPreparationRevset(revset.clone()),
            (None, true) => DesiredPlan::FuturePreparedSource,
            (Some(revset), false) => DesiredPlan::Recorded(self.resolution.caller.resolve_one_at(&operation, revset)?.commit_id),
            (None, false) if source.assertion_pending => DesiredPlan::AfterLiveSourceAssertion,
            (None, false) => DesiredPlan::Recorded(source.recorded_commit.clone()),
        };
        Ok(CommandPlan {
            profile: self.source.binding.repository.profile.clone(),
            label: "PLANNED — payload/path obstructions not inspected",
            source,
            desired,
            route: self.route,
            receivers: self.receivers.iter().map(|b| (b.name.clone(), b.root.clone())).collect(),
            patterns: self.pattern_declarations.clone(),
            clone_capability_probed: false,
            preservation_pending: true,
            obstructions_inspected: false,
        })
    }

    pub fn execute(self, transfer: &mut impl OrdinaryTransfer, diagnostic: &mut dyn FnMut(&str)) -> std::result::Result<CommandReport, CommandFailure> {
        self.execute_handoff(None, transfer, diagnostic)
    }

    /// A producer can hand its actual S directly to refill. This performs a
    /// coordinate barrier, not another live source proof or implicit resnapshot.
    pub fn execute_prepared(
        self,
        source: PreparedSource,
        transfer: &mut impl OrdinaryTransfer,
        diagnostic: &mut dyn FnMut(&str),
    ) -> std::result::Result<CommandReport, CommandFailure> {
        self.execute_handoff(Some(source), transfer, diagnostic)
    }

    fn execute_handoff(
        self,
        handoff: Option<PreparedSource>,
        transfer: &mut impl OrdinaryTransfer,
        diagnostic: &mut dyn FnMut(&str),
    ) -> std::result::Result<CommandReport, CommandFailure> {
        let started = self.started;
        let mut setup_done = false;
        let mut report = CommandReport {
            profile: self.source.binding.repository.profile.clone(),
            source: None,
            preparation: None,
            receivers: self
                .receivers
                .iter()
                .map(|b| ReceiverReport {
                    name: b.name.clone(),
                    root: b.root.clone(),
                    route: self.route,
                    old: None,
                    desired: None,
                    child: None,
                    reached: Phase::Resolve,
                    status: ReceiverStatus::NotStarted,
                    native: NativeOutcome::default(),
                    ordinary: OrdinaryOutcome::default(),
                    finish: FinishOutcome::default(),
                })
                .collect(),
            capture: CaptureCounts::default(),
            tracked_source: None,
            ordinary_source: None,
            setup: Default::default(),
            ordinary_elapsed: None,
            elapsed: Default::default(),
        };
        let mut phase = Phase::Resolve;
        let mut lane = None;
        let result = (|| {
            ensure!(!self.request.dry_run, "use preview() for a read-only PLANNED result");
            transfer.admit(self.receivers.len(), !self.patterns.is_empty())?;
            for binding in &self.receivers {
                cow::probe_between(&self.source.binding.root, &binding.root).context("required CoW route admission")?;
            }
            phase = Phase::Source;
            let source = match handoff {
                Some(prepared) => {
                    ensure!(
                        Arc::ptr_eq(&prepared.binding.repository, &self.source.binding.repository)
                            || prepared.binding.repository.path == self.source.binding.repository.path,
                        "prepared source repository differs"
                    );
                    ensure!(
                        prepared.binding.root == self.source.binding.root && prepared.binding.name == self.source.binding.name,
                        "prepared source binding differs"
                    );
                    prepared
                }
                None if self.request.prepare => match prepare_source(&self.source, diagnostic) {
                    Ok(source) => source,
                    Err(failure) => {
                        report.preparation = Some(failure.completed.as_ref().clone());
                        return Err(failure.into());
                    }
                },
                None => self.source.existing()?,
            };
            report.source = Some(source.report.clone());
            report.preparation = source.report.preparation.clone();
            let operation = source.binding.client().operation_id()?;
            let desired = self
                .request
                .at
                .as_ref()
                .map(|r| self.resolution.caller.resolve_one_at(&operation, r))
                .transpose()?
                .map_or_else(|| source.report.commit.clone(), |r| r.commit_id);
            let mut saved = Vec::new();
            phase = Phase::Preserve;
            for (index, (binding, links)) in self.receivers.iter().zip(self.links).enumerate() {
                lane = Some(index);
                report.receivers[index].reached = phase;
                snapshot(&binding.client()).context("save old receiving work")?;
                let operation = binding.client().operation_id()?;
                let old = binding.client().resolve_one_at(&operation, "@")?.commit_id;
                diagnostic(&format!("{}: saved old tracked work at {old}", binding.name));
                report.receivers[index].old = Some(SavedWork {
                    commit: old.clone(),
                    history: HistoryCoverage::Pending,
                });
                report.receivers[index].desired = Some(desired.clone());
                saved.push(SavedReceiver {
                    binding: Arc::clone(binding),
                    old_commit: old,
                    guard: bookmarks::Guard::AllowUnbookmarked,
                    links,
                });
            }
            lane = None;
            let caller = &self.resolution.caller;
            let operation = caller.operation_id()?;
            let coverage = if self.request.allow_unbookmarked {
                None
            } else {
                Some(bookmarks::Coverage::capture(
                    caller,
                    &operation,
                    &saved.iter().map(|s| s.old_commit.clone()).collect(),
                )?)
            };
            for (index, saved) in saved.iter_mut().enumerate() {
                lane = Some(index);
                if let Some(coverage) = &coverage {
                    saved.guard = coverage.require(&saved.old_commit)?;
                }
                report.receivers[index].old.as_mut().unwrap().history = HistoryCoverage::from_guard(&saved.guard);
            }
            // Source and every original receiver coordinate must still hold after
            // all preservation snapshots. No destructive refill before this.
            lane = None;
            source.binding.verify_at(&operation)?;
            ensure!(
                source.binding.client().resolve_one_at(&operation, "@")?.commit_id == source.report.commit,
                "donor coordinate changed during receiving snapshots; refused before clearing"
            );
            for (index, saved) in saved.iter().enumerate() {
                lane = Some(index);
                saved.binding.verify_at(&operation)?;
                ensure!(
                    saved.binding.client().resolve_one_at(&operation, "@")?.commit_id == saved.old_commit,
                    "saved receiver O moved during preservation; refused before clearing"
                );
            }
            bookmarks::verify_batch(caller, saved.iter().map(|s| &s.guard))?;
            lane = None;
            phase = Phase::Capture;
            let commits = std::iter::once(source.report.commit.clone())
                .chain(std::iter::once(desired.clone()))
                .chain(saved.iter().map(|s| s.old_commit.clone()))
                .collect::<Vec<_>>();
            let membership = Membership::capture(caller, &operation, &commits)?;
            report.capture.distinct_trees = membership.count();
            report.capture.membership_queries = membership.count();
            report.capture.indexes = membership.count();
            let input = native::inputs(&membership, &source, &desired, &saved, self.route, self.request.replace_content)?;
            report.capture.donor_cache_reads = 1;
            report.capture.desired_manifests = 1;
            report.capture.owned_manifests = input.tracked.owned_count();
            let native_paths = if Arc::ptr_eq(&input.source, &input.desired) {
                Arc::clone(&input.source.index)
            } else {
                Arc::new(NativePathIndex {
                    paths: Arc::new(input.source.index.paths.union(&input.desired.index.paths).cloned().collect()),
                })
            };
            let mut ignores = FrozenIgnores::default();
            for (commit, key) in &membership.keys {
                ignores.same_tree(commit, key);
            }
            let mut ordinary = OrdinaryCohort {
                source: Arc::clone(&source.binding),
                patterns: Arc::clone(&self.patterns),
                native: native_paths,
                lanes: Vec::new(),
            };
            for (index, saved) in saved.iter().enumerate() {
                lane = Some(index);
                let active_links = saved.links.active_sources();
                for path in &active_links {
                    ensure!(
                        !ordinary.native.overlaps(path),
                        "configured active link overlaps S/D tracked ownership: {}",
                        path.display()
                    );
                }
                let needs_rules = !self.patterns.is_empty() || !active_links.is_empty();
                let desired_ignore = if needs_rules {
                    ignores.get(&saved.binding.client(), &operation, &desired, &input.desired.leaves)?
                } else {
                    Arc::new(FrozenIgnore::default())
                };
                let source_ignore = if !self.patterns.is_empty() {
                    ignores.get(&source.binding.client(), &operation, &source.report.commit, &input.source.leaves)?
                } else {
                    Arc::new(FrozenIgnore::default())
                };
                ordinary.lanes.push(OrdinaryLane {
                    binding: Arc::clone(&saved.binding),
                    source_ignore,
                    desired_ignore,
                    active_links,
                });
            }
            report.setup = started.elapsed();
            setup_done = true;
            for (index, saved) in saved.iter().enumerate() {
                lane = Some(index);
                let result = native::establish(&source, &desired, saved, &input, self.request.replace_content, &mut report.receivers[index]);
                report.tracked_source = Some(input.tracked.source.diagnostics());
                phase = report.receivers[index].reached;
                result?;
            }
            // Explicit end of native-only lifetimes. No old membership/cache can
            // be accidentally kept alive through ordinary transfer by a caller.
            drop(input);
            drop(membership);
            drop(ignores);
            phase = Phase::Ordinary;
            lane = None;
            let ordinary_started = Instant::now();
            if self.patterns.is_empty() {
                for r in &mut report.receivers {
                    r.ordinary.state = OrdinaryState::Disabled;
                }
            } else {
                let mut outcomes = report.receivers.iter().map(|r| r.ordinary.clone()).collect::<Vec<_>>();
                let result = transfer.transfer(&ordinary, &mut outcomes);
                for (r, outcome) in report.receivers.iter_mut().zip(outcomes) {
                    r.ordinary = outcome;
                    r.reached = phase;
                }
                report.ordinary_elapsed = Some(ordinary_started.elapsed());
                match result {
                    Ok(diagnostics) => report.ordinary_source = diagnostics,
                    Err(failure) => {
                        report.ordinary_source = failure.diagnostics.map(|snapshot| *snapshot);
                        lane = failure.lane;
                        return Err(failure.source);
                    }
                }
                ensure!(
                    report.receivers.iter().all(|r| r.ordinary.state == OrdinaryState::Satisfied),
                    "ordinary cohort did not satisfy every lane"
                );
            }
            for (index, saved) in saved.iter().enumerate() {
                lane = Some(index);
                let finish_started = Instant::now();
                phase = Phase::Links;
                report.receivers[index].reached = phase;
                let result = finish(saved, &ordinary.lanes[index], &desired, &mut report.receivers[index]);
                report.receivers[index].finish.elapsed = Some(finish_started.elapsed());
                phase = report.receivers[index].reached;
                result?;
            }
            lane = None;
            phase = Phase::Finish;
            let operation = source.binding.client().operation_id()?;
            source.binding.verify_at(&operation)?;
            ensure!(
                source.binding.client().resolve_one_at(&operation, "@")?.commit_id == source.report.commit,
                "donor coordinate changed during refill"
            );
            Ok(())
        })();
        report.elapsed = started.elapsed();
        if !setup_done {
            report.setup = report.elapsed;
        }
        match result {
            Ok(()) => Ok(report),
            Err(source) => {
                if let Some(index) = lane {
                    report.receivers[index].status = ReceiverStatus::Failed;
                }
                let receiver = lane.map(|i| report.receivers[i].name.clone());
                Err(CommandFailure {
                    report: Box::new(report),
                    location: FailureLocation { phase, receiver },
                    source,
                })
            }
        }
    }
}

fn finish(saved: &SavedReceiver, ordinary: &OrdinaryLane, desired: &str, report: &mut ReceiverReport) -> Result<()> {
    for check in saved.links.inspect() {
        if check.source != check.target && check.target_exists {
            ensure!(
                ordinary.desired_ignore.ignored(check.source.strip_prefix(&saved.binding.root)?, false),
                "active link is not ignored under D: {}",
                check.source.display()
            );
        }
    }
    match links::apply_loaded_workspace_links(&saved.links) {
        Ok(done) => report.finish.links = done,
        Err(failure) => {
            report.finish.links = failure.completed.clone();
            report.finish.failed_path = failure.path.clone();
            return Err(failure.into());
        }
    }
    report.reached = Phase::Finish;
    let child = report.child.as_ref().context("missing established native child")?;
    let facts = native::verify_native(&saved.binding, desired, &child.change_id)?;
    ensure!(facts.commit_id == child.commit_id, "native commit changed during ordinary/link finish");
    report.finish.complete = true;
    report.status = ReceiverStatus::Complete;
    report.reached = Phase::Complete;
    Ok(())
}
