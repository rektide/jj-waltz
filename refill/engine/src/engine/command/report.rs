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
pub struct SavedWork {
    pub commit: String,
    pub history: String,
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
        write!(
            f,
            "reseed {:?}: phase {}; partial effects remain, no rollback; {:?}: {}",
            self.location.receiver, self.location.phase, self.report.receivers, self.source
        )
    }
}
impl std::error::Error for CommandFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}
