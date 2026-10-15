//! Frozen, in-place CoW reseeding through stock JJ's sparse checkout interface.
//!
//! The repository-scoped journal is intentionally separate from lifecycle metadata.
//! A pending journal always wins over ordinary snapshotting: recovery first
//! rebuilds and verifies the recorded current tree from frozen public JJ queries.
//! Only then may stock JJ's sparse commands snapshot that known-complete tree.
//! Repair uses only frozen, snapshot-disabled queries; sparse mutations scope
//! `snapshot.auto-track=none()`, and reset expands only from sparse-empty.

use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum UnmatchedPolicy {
    #[default]
    Error,
    Warn,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReseedRequest {
    pub names: Vec<String>,
    pub groups: Vec<String>,
    /// Override group selection-miss policy; positional names are always strict.
    pub on_unmatched: Option<UnmatchedPolicy>,
    /// Explicit donor root; otherwise local configuration or the verified primary checkout.
    pub source: Option<PathBuf>,
    pub source_at: Option<String>,
    pub at: Option<String>,
    pub copy_ignored: Vec<String>,
    pub no_copy_ignored: bool,
    pub replace_content: bool,
    pub dry_run: bool,
    pub resume: bool,
    /// Fetch and prepare the chosen donor before freezing target plans.
    pub freshen: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReseedSelection {
    pub name: String,
    pub root: PathBuf,
    pub origins: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReseedSelectionMiss {
    /// Losslessly escaped, quoted candidate name, pattern, or pathname.
    pub candidate: String,
    pub reason: String,
}

impl ReseedReport {
    pub fn has_failures(&self) -> bool {
        self.targets
            .iter()
            .any(|target| matches!(target.status, ReseedStatus::Failed | ReseedStatus::Pending))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReseedOutcome {
    pub name: String,
    pub status: ReseedStatus,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReseedStatus {
    Planned,
    Succeeded,
    Failed,
    Pending,
}

pub fn run(request: ReseedRequest) -> Result<ReseedReport> {
    run_with_diagnostics(request, |diagnostic| eprintln!("{diagnostic}"))
}

/// Emit selection diagnostics before planning, including when planning later fails.
pub fn run_with_diagnostics(
    request: ReseedRequest,
    diagnostic: impl FnMut(&str),
) -> Result<ReseedReport> {
    #[cfg(unix)]
    {
        let mut diagnostic = diagnostic;
        engine::run(request, &mut diagnostic)
    }
    #[cfg(not(unix))]
    {
        let _ = request;
        let _ = diagnostic;
        anyhow::bail!("reseed requires Unix filesystem identity and CoW support")
    }
}

#[cfg(unix)]
#[path = "reseed/engine.rs"]
mod engine;

#[cfg(unix)]
#[path = "reseed/selection.rs"]
mod selection;
