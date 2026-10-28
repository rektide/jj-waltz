//! In-place CoW reseeding through stock JJ's sparse checkout interface.
//! Work is snapshotted and bookmark coverage checked before mutation. Interrupted
//! refills require deliberate out-of-band recovery, never automatic repair.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnmatchedPolicy {
    #[default]
    Error,
    Warn,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReseedRequest {
    /// Literal targets; empty names/groups select the current variant.
    pub names: Vec<String>,
    pub groups: Vec<String>,
    /// Override group selection-miss policy; positional names are always strict.
    pub on_unmatched: Option<UnmatchedPolicy>,
    /// Explicit donor root; otherwise local configuration or the verified primary checkout.
    pub source: Option<PathBuf>,
    /// Donor tree assertion, or upstream goal when preparation is requested.
    pub source_at: Option<String>,
    /// Prepare the donor once before reseeding; plain reseed uses its existing state.
    #[serde(default)]
    pub prepare: bool,
    pub at: Option<String>,
    /// None inherits local/global policy; Some([]) explicitly disables inheritance.
    pub copy_ignored: Option<Vec<String>>,
    pub no_copy_ignored: bool,
    pub replace_content: bool,
    /// Explicitly bypass only the default bookmarked-work preflight requirement.
    #[serde(default)]
    pub allow_unbookmarked: bool,
    pub dry_run: bool,
}

/// Source-only preparation, independent of the invoking workspace's role.
#[derive(Debug, Clone, Default)]
pub struct PrepareRequest {
    pub source: Option<PathBuf>,
    pub source_at: Option<String>,
    pub copy_ignored: Option<Vec<String>>,
    pub no_copy_ignored: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReseedReport {
    pub dry_run: bool,
    pub source: PathBuf,
    /// None when a freshen preview cannot know the post-fetch exact goal.
    pub source_commit_id: Option<String>,
    pub preparation_summary: Option<String>,
    pub targets: Vec<ReseedOutcome>,
    pub selections: Vec<ReseedSelection>,
    pub skipped: Vec<ReseedSelectionMiss>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReseedSelection {
    pub name: String,
    pub root: PathBuf,
    pub origins: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ReseedSelectionMiss {
    /// Losslessly escaped, quoted candidate name, pattern, or pathname.
    pub candidate: String,
    pub reason: String,
}

impl ReseedReport {
    pub fn has_failures(&self) -> bool {
        self.targets.iter().any(|target| target.status == ReseedStatus::Failed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReseedOutcome {
    pub name: String,
    pub status: ReseedStatus,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ReseedStatus {
    Planned,
    Succeeded,
    Failed,
}

pub fn run(request: ReseedRequest) -> Result<ReseedReport> {
    run_with_diagnostics(request, |diagnostic| eprintln!("{diagnostic}"))
}

/// Emit selection diagnostics before planning, including when planning later fails.
pub fn run_with_diagnostics(request: ReseedRequest, diagnostic: impl FnMut(&str)) -> Result<ReseedReport> {
    run_internal(request, false, diagnostic)
}

pub fn prepare(request: PrepareRequest) -> Result<ReseedReport> {
    prepare_with_diagnostics(request, |diagnostic| eprintln!("{diagnostic}"))
}

pub fn prepare_with_diagnostics(request: PrepareRequest, diagnostic: impl FnMut(&str)) -> Result<ReseedReport> {
    run_internal(
        ReseedRequest {
            source: request.source,
            source_at: request.source_at,
            copy_ignored: request.copy_ignored,
            no_copy_ignored: request.no_copy_ignored,
            dry_run: request.dry_run,
            ..Default::default()
        },
        true,
        diagnostic,
    )
}

fn run_internal(request: ReseedRequest, source_only: bool, diagnostic: impl FnMut(&str)) -> Result<ReseedReport> {
    #[cfg(unix)]
    {
        let mut diagnostic = diagnostic;
        engine::run(request, source_only, &mut diagnostic)
    }
    #[cfg(not(unix))]
    {
        let _ = request;
        let _ = source_only;
        let _ = diagnostic;
        anyhow::bail!("reseed requires Unix filesystem identity and CoW support")
    }
}

#[cfg(unix)]
mod engine;

pub mod freshen;
