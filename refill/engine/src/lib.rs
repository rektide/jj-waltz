//! In-place CoW reseeding through stock JJ's sparse checkout interface.
//! Work is snapshotted and bookmark coverage checked before mutation. Interrupted
//! refills require deliberate out-of-band recovery, never automatic repair.

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
    #[serde(default)]
    pub renewal_route: command::RenewalRoute,
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
    pub dry_run: bool,
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

#[cfg(unix)]
mod engine;

/// The shared command and source-only producer used by the public CLI.
#[cfg(unix)]
pub use engine::command;

pub mod freshen;
