//! In-place CoW renewal through profile-gated JJ transitions and one cohort.
//! Work is snapshotted and bookmark coverage checked before mutation. Interrupted
//! refills require deliberate out-of-band recovery, never automatic repair.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenewalRoute {
    #[default]
    Refresh,
    ClearThenPopulate,
    ReplaceAuthorized,
}

pub use diff_dirs::StatChecks;

/// Optional regular-install read-back, independent of refresh comparison.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Postcheck {
    #[default]
    None,
    Mtime,
}
impl From<Postcheck> for diff_dirs_sync::Postcheck {
    fn from(value: Postcheck) -> Self {
        match value {
            Postcheck::None => Self::None,
            Postcheck::Mtime => Self::Mtime,
        }
    }
}

mod stat_checks_serde {
    use super::StatChecks;
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(checks: &StatChecks, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(checks)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<StatChecks, D::Error> {
        String::deserialize(deserializer)?.parse().map_err(serde::de::Error::custom)
    }
}

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
    pub renewal_route: RenewalRoute,
    #[serde(default, with = "stat_checks_serde")]
    pub refresh_check: StatChecks,
    #[serde(default)]
    pub postcheck: Postcheck,
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn request_defaults_and_serialized_checks_select_the_same_public_contract() {
        let request = ReseedRequest::default();
        assert_eq!(request.renewal_route, RenewalRoute::Refresh);
        assert_eq!(request.refresh_check.to_string(), "mtime,size");
        assert_eq!(request.postcheck, Postcheck::None);
        let mut value = serde_json::to_value(&request).unwrap();
        value.as_object_mut().unwrap().remove("renewal_route");
        value.as_object_mut().unwrap().remove("refresh_check");
        value.as_object_mut().unwrap().remove("postcheck");
        let decoded: ReseedRequest = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(decoded.renewal_route, RenewalRoute::Refresh);
        assert_eq!(decoded.refresh_check, request.refresh_check);
        assert_eq!(decoded.postcheck, Postcheck::None);
        value["refresh_check"] = serde_json::json!("mode,size");
        let decoded: ReseedRequest = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(decoded.refresh_check.to_string(), "size,mode");
        for malformed in ["", "mtime,mtime", "unknown"] {
            value["refresh_check"] = serde_json::json!(malformed);
            assert!(serde_json::from_value::<ReseedRequest>(value.clone()).is_err());
        }
    }
}
