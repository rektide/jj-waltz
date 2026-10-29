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
    NativeClear,
    NativeChild,
    ExplicitClear,
    Install,
    AdmitDesired,
    Adopt,
    Restore,
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
            Self::NativeClear => "clear tracked checkout",
            Self::NativeChild => "establish receiving child",
            Self::ExplicitClear => "explicit replace-content",
            Self::Install => "strict full-S install",
            Self::AdmitDesired => "D-addition namespace admission",
            Self::Adopt => "adopt captured source tree_state",
            Self::Restore => "native S-to-D restore",
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
    /// Actual successful S installations. Not a final-D storage-sharing count.
    pub installed: diff_dirs_sync::Progress,
    pub explicit_clear: diff_dirs_sync::Progress,
    pub manifest_complete: bool,
    pub state: Establishment,
    /// Stock JJ commands may have unenumerated effects even on process failure.
    pub attempted: Vec<Phase>,
    pub elapsed_ms: u128,
}
#[derive(Debug, Clone, Default)]
pub struct OrdinaryOutcome {
    pub state: OrdinaryState,
    /// None with Partial means the old N1 adapter cannot expose actual progress;
    /// the provider adapter must retain cursor reports, not manufacture zeros.
    pub report: Option<diff_dirs_sync::coordinated::LaneReport>,
}
#[derive(Debug, Clone, Default)]
pub struct FinishOutcome {
    pub links: links::LinkApplyReport,
    pub failed_path: Option<PathBuf>,
    pub complete: bool,
    pub elapsed_ms: u128,
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
}
#[derive(Debug)]
pub struct CommandReport {
    pub profile: NativeProfile,
    pub source: Option<SourceReport>,
    pub preparation: Option<PreparationReport>,
    pub receivers: Vec<ReceiverReport>,
    pub capture: CaptureCounts,
    pub setup_ms: u128,
    pub ordinary_ms: u128,
    pub elapsed_ms: u128,
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
            write!(
                f,
                "; receiver native files_reflinked: {}, files_copied: {}",
                receiver.native.installed.files_reflinked, receiver.native.installed.files_copied,
            )?;
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
            write!(f, "; receiver links created: {}", receiver.finish.links.linked)?;
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

    fn failure() -> CommandFailure {
        let mut ordinary = diff_dirs_sync::coordinated::LaneReport {
            target: diff_dirs::coordinated::TargetId("target".into()),
            totals: Default::default(),
            observed: Default::default(),
            retained: 0,
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
                installed: diff_dirs_sync::Progress {
                    files_reflinked: 3,
                    ..Default::default()
                },
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
                capture: CaptureCounts::default(),
                setup_ms: 0,
                ordinary_ms: 0,
                elapsed_ms: 0,
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
