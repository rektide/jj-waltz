//! Frozen, in-place CoW reseeding through stock JJ's sparse checkout interface.
//!
//! The repository-scoped journal is intentionally separate from lifecycle metadata.
//! A pending journal always wins over ordinary snapshotting: recovery first
//! rebuilds and verifies the recorded current tree from frozen public JJ queries.
//! Only then may stock JJ's sparse commands snapshot that known-complete tree.
//! Repair uses only frozen, snapshot-disabled queries; sparse mutations scope
//! `snapshot.auto-track=none()`, and reset expands only from sparse-empty.

use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct ReseedRequest {
    pub names: Vec<String>,
    pub groups: Vec<String>,
    /// Explicit donor root; otherwise local configuration or the verified primary checkout.
    pub source: Option<PathBuf>,
    pub source_at: Option<String>,
    pub at: Option<String>,
    pub copy_ignored: Vec<String>,
    pub no_copy_ignored: bool,
    pub replace_content: bool,
    pub dry_run: bool,
    pub resume: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReseedReport {
    pub dry_run: bool,
    pub source: PathBuf,
    pub source_commit_id: String,
    pub targets: Vec<ReseedOutcome>,
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
    #[cfg(unix)]
    {
        engine::run(request)
    }
    #[cfg(not(unix))]
    {
        let _ = request;
        anyhow::bail!("reseed requires Unix filesystem identity and CoW support")
    }
}

#[cfg(unix)]
#[path = "reseed/engine.rs"]
mod engine;
