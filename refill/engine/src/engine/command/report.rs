use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RenewalRoute {
    #[default]
    ClearThenPopulate,
    ReplaceAuthorized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Resolve,
    Source,
    Preserve,
    Capture,
    NativeSetup,
    NativeClear,
    NativeChild,
    ExplicitClear,
    Install,
    AdmitDesired,
    Adopt,
    Restore,
    Reparent,
    NativeVerify,
    Ordinary,
    Links,
    Finish,
    Complete,
}
impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Resolve => "resolve",
            Self::Source => "source preparation/assertion",
            Self::Preserve => "save receiving work",
            Self::Capture => "native membership capture",
            Self::NativeSetup => "native transition guards",
            Self::NativeClear => "clear tracked checkout",
            Self::NativeChild => "establish receiving child",
            Self::ExplicitClear => "explicit replace-content",
            Self::Install => "full-S tracked installation",
            Self::AdmitDesired => "D-addition namespace admission",
            Self::Adopt => "adopt captured source tree_state",
            Self::Restore => "native S-to-D restore",
            Self::Reparent => "reparent new receiving change",
            Self::NativeVerify => "final native diff",
            Self::Ordinary => "ordinary coordinated transfer",
            Self::Links => "loaded link finish",
            Self::Finish => "finish native identity",
            Self::Complete => "complete",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverStatus {
    NotStarted,
    Partial,
    Complete,
    Failed,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Establishment {
    #[default]
    Unstarted,
    InProgress,
    Established,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OrdinaryState {
    Disabled,
    #[default]
    Unstarted,
    Partial,
    Satisfied,
}

#[derive(Debug, Clone, Default)]
pub struct NativeOutcome {
    /// Actual provider-visible S installations, owned removals and directory
    /// scope effects. Not a final-D storage-sharing count or stock-JJ effects.
    pub completed: Option<diff_dirs_sync::Progress>,
    pub explicit_clear: Option<diff_dirs_sync::Progress>,
    /// Actual provider coverage, retained even on an unsuccessful lane. None
    /// means no lane returned coverage, not an empty complete manifest.
    pub manifest: Option<diff_dirs_sync::ManifestCoverage>,
    /// Actual distinct provider deviation/retention outcomes, not syscall counts.
    pub policy: Option<diff_dirs_sync::Policy>,
    /// Whether S is only a temporary installation before native D convergence.
    pub temporary_source_installation: bool,
    /// None before native entry; false until adoption returns successfully.
    /// True is not native conformance or command completion.
    pub source_state_adopted: Option<bool>,
    pub state: Establishment,
    /// Stock JJ commands may have unenumerated effects even on process failure.
    pub attempted: Vec<Phase>,
    /// Measured boundaries, including failed attempts. An absent optional phase
    /// has no row; elapsed time is never inferred by subtracting other phases.
    pub timings: Vec<PhaseTiming>,
    pub elapsed: Option<std::time::Duration>,
}
#[derive(Debug, Clone)]
pub struct PhaseTiming {
    pub phase: Phase,
    pub elapsed: std::time::Duration,
    pub succeeded: bool,
}
impl NativeOutcome {
    pub fn tracked_install_elapsed(&self) -> Option<std::time::Duration> {
        self.timings.iter().find(|step| step.phase == Phase::Install).map(|step| step.elapsed)
    }
}
#[derive(Debug, Clone, Default)]
pub struct OrdinaryOutcome {
    pub state: OrdinaryState,
    /// Actual provider cursor report, including failure prefixes. None means no
    /// valid cursor could be constructed; it never stands in for zero work.
    pub report: Option<diff_dirs_sync::coordinated::LaneReport>,
}
#[derive(Debug, Clone, Default)]
pub struct FinishOutcome {
    /// None before a link application returns, including pre-link inspection
    /// failure. Returned zero work is Some, never inferred from phase timing.
    pub links: Option<links::LinkApplyReport>,
    pub failed_path: Option<PathBuf>,
    pub complete: bool,
    pub elapsed: Option<std::time::Duration>,
}
#[derive(Debug, Clone)]
pub enum HistoryCoverage {
    Pending,
    Bypassed,
    Covered {
        work_tip: String,
        name: String,
        remote: Option<String>,
        target: String,
    },
}
impl HistoryCoverage {
    pub(super) fn from_guard(guard: &bookmarks::Guard) -> Self {
        match guard.coverage() {
            None => Self::Bypassed,
            Some((work_tip, name, remote, target)) => Self::Covered {
                work_tip: work_tip.into(),
                name: name.into(),
                remote: remote.map(str::to_owned),
                target: target.into(),
            },
        }
    }
}
#[derive(Debug, Clone)]
pub struct SavedWork {
    pub commit: String,
    pub history: HistoryCoverage,
}
#[derive(Debug, Clone)]
pub struct ReceiverReport {
    pub name: String,
    pub root: PathBuf,
    pub route: RenewalRoute,
    pub old: Option<SavedWork>,
    pub desired: Option<String>,
    pub child: Option<CommitFacts>,
    pub reached: Phase,
    pub status: ReceiverStatus,
    pub native: NativeOutcome,
    pub ordinary: OrdinaryOutcome,
    pub finish: FinishOutcome,
}
#[derive(Debug, Clone, Default)]
pub struct CaptureCounts {
    pub distinct_trees: usize,
    pub membership_queries: usize,
    pub indexes: usize,
    pub donor_cache_reads: usize,
    pub desired_manifests: usize,
    pub owned_manifests: usize,
}
#[derive(Debug)]
pub struct CommandReport {
    pub profile: NativeProfile,
    pub source: Option<SourceReport>,
    pub preparation: Option<PreparationReport>,
    pub receivers: Vec<ReceiverReport>,
    /// None until a valid membership/capture result supplies known counts.
    pub capture: Option<CaptureCounts>,
    /// One snapshot per source phase, never multiplied across receiver rows.
    pub tracked_source: Option<diff_dirs_inspect::SourceDiagnostics>,
    pub ordinary_source: Option<diff_dirs_inspect::SourceDiagnostics>,
    pub setup: std::time::Duration,
    pub ordinary_elapsed: Option<std::time::Duration>,
    pub elapsed: std::time::Duration,
}
#[derive(Debug)]
pub struct FailureLocation {
    pub phase: Phase,
    pub receiver: Option<String>,
}
#[derive(Debug)]
pub struct CommandFailure {
    pub report: Box<CommandReport>,
    pub location: FailureLocation,
    pub source: anyhow::Error,
}
impl std::fmt::Display for CommandFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("reseed")?;
        if let Some(name) = &self.location.receiver {
            write!(f, " receiver {name:?}")?;
        }
        let receivers = &self.report.receivers;
        write!(
            f,
            ": phase {}: {:#}; known effects: {}/{} receivers complete, {} native established",
            self.location.phase,
            self.source,
            receivers.iter().filter(|r| r.status == ReceiverStatus::Complete).count(),
            receivers.len(),
            receivers.iter().filter(|r| r.native.state == Establishment::Established).count(),
        )?;
        if let Some(receiver) = self.location.receiver.as_ref().and_then(|name| receivers.iter().find(|r| &r.name == name)) {
            if let Some(progress) = &receiver.native.completed {
                write!(
                    f,
                    "; receiver native files_reflinked: {}, files_copied: {}",
                    progress.files_reflinked, progress.files_copied
                )?;
            } else {
                f.write_str("; receiver native progress unavailable")?;
            }
            if let Some(report) = &receiver.ordinary.report {
                write!(
                    f,
                    "; ordinary files_reflinked: {}, files_copied: {}",
                    report.totals.completed.files_reflinked, report.totals.completed.files_copied,
                )?;
            } else {
                write!(f, "; ordinary {:?}", receiver.ordinary.state)?;
                if matches!(receiver.ordinary.state, OrdinaryState::Partial | OrdinaryState::Satisfied) {
                    f.write_str(" (progress unavailable)")?;
                }
            }
            if let Some(links) = &receiver.finish.links {
                write!(f, "; receiver links created: {}", links.linked)?;
            } else {
                f.write_str("; receiver link progress unavailable")?;
            }
        }
        f.write_str("; partial effects may remain, no rollback")
    }
}
impl std::error::Error for CommandFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn unentered_native_and_finish_have_no_returned_effects_adoption_or_timing() {
        let native = NativeOutcome::default();
        assert!(native.completed.is_none());
        assert!(native.explicit_clear.is_none());
        assert!(native.manifest.is_none());
        assert!(native.policy.is_none());
        assert!(native.source_state_adopted.is_none());
        assert!(native.elapsed.is_none());
        assert!(native.tracked_install_elapsed().is_none());
        let finish = FinishOutcome::default();
        assert!(finish.links.is_none());
        assert!(finish.elapsed.is_none());
    }

    fn failure() -> CommandFailure {
        let mut ordinary = diff_dirs_sync::coordinated::LaneReport {
            target: diff_dirs::coordinated::TargetId("target".into()),
            totals: Default::default(),
            observed: Default::default(),
            retained: 0,
            deviated: 0,
            satisfied: true,
        };
        ordinary.totals.completed.files_reflinked = 1;
        let receiver = ReceiverReport {
            name: "target".into(),
            root: "/receiver".into(),
            route: RenewalRoute::ClearThenPopulate,
            old: None,
            desired: None,
            child: None,
            reached: Phase::Links,
            status: ReceiverStatus::Failed,
            native: NativeOutcome {
                completed: Some(diff_dirs_sync::Progress {
                    files_reflinked: 3,
                    ..Default::default()
                }),
                state: Establishment::Established,
                ..Default::default()
            },
            ordinary: OrdinaryOutcome {
                state: OrdinaryState::Satisfied,
                report: Some(ordinary),
            },
            finish: FinishOutcome::default(),
        };
        let mut earlier = receiver.clone();
        earlier.name = "earlier".into();
        earlier.status = ReceiverStatus::Complete;
        earlier.finish.complete = true;
        CommandFailure {
            report: Box::new(CommandReport {
                profile: NativeProfile::classify(format!("jj {}", jj_waltz_native_jj::refill::TESTED_BUILD)).unwrap(),
                source: None,
                preparation: None,
                receivers: vec![earlier, receiver],
                capture: None,
                tracked_source: None,
                ordinary_source: None,
                setup: Default::default(),
                ordinary_elapsed: None,
                elapsed: Default::default(),
            }),
            location: FailureLocation {
                phase: Phase::Links,
                receiver: Some("target".into()),
            },
            source: anyhow::Error::new(std::io::Error::new(ErrorKind::PermissionDenied, "link refused")).context("create required link"),
        }
    }

    #[test]
    fn command_failure_text_does_not_expand_structured_path_vectors() {
        let mut failure = failure();
        let text = failure.to_string();
        let receiver = &mut failure.report.receivers[1];
        let hidden_path = PathBuf::from("not-for-default-display".repeat(16));
        receiver.root = hidden_path.clone();
        receiver.finish.failed_path = Some(hidden_path.clone());
        receiver.native.attempted = vec![Phase::Install; 256];
        receiver.ordinary.report.as_mut().unwrap().observed.pruned = vec![hidden_path; 256];
        assert_eq!(failure.to_string(), text);
        assert!(text.len() < 512, "{text}");
        assert!(text.starts_with("reseed receiver \"target\": phase loaded link finish: create required link: link refused"));
        assert!(text.contains("1/2 receivers complete, 2 native established"));
        assert!(text.contains("native files_reflinked: 3"));
        assert!(text.contains("ordinary files_reflinked: 1"));
        assert!(text.contains("no rollback"));
        assert_eq!(failure.report.receivers[1].ordinary.report.as_ref().unwrap().observed.pruned.len(), 256);
    }

    #[test]
    fn command_failure_retains_original_cause_and_structured_report() {
        let failure = failure();
        assert_eq!(failure.source().unwrap().to_string(), "create required link");
        assert_eq!(failure.source.downcast_ref::<std::io::Error>().unwrap().kind(), ErrorKind::PermissionDenied);
        let error = anyhow::Error::new(failure);
        assert_eq!(
            error.chain().find_map(|cause| cause.downcast_ref::<std::io::Error>()).unwrap().kind(),
            ErrorKind::PermissionDenied
        );
        assert_eq!(
            error.downcast_ref::<CommandFailure>().unwrap().report.receivers[0].status,
            ReceiverStatus::Complete
        );
    }

    #[test]
    fn command_failure_without_receiver_omits_receiver_label() {
        let mut failure = failure();
        failure.location.receiver = None;
        failure.location.phase = Phase::Ordinary;
        let text = failure.to_string();
        assert!(text.starts_with("reseed: phase ordinary coordinated transfer:"));
        assert!(!text.contains("receiver native"));
        assert!(!text.contains("None"));
    }

    #[test]
    fn command_failure_does_not_invent_unknown_ordinary_progress() {
        let mut failure = failure();
        failure.report.receivers[1].ordinary = OrdinaryOutcome {
            state: OrdinaryState::Partial,
            report: None,
        };
        let text = failure.to_string();
        assert!(text.contains("ordinary Partial (progress unavailable)"));
        assert!(!text.contains("ordinary files_reflinked:"));
    }
}
