//! Shared workspace inventory and query facts.
//!
//! `WorkspaceInventory` reads the native workspace registry and the recorded
//! navigation state without mutating anything. Mutation orchestration —
//! creation, switching, removal — lives in the lifecycle crate.
use anyhow::{Context, Result, anyhow, bail};
use std::fs;
use std::path::{Path, PathBuf};

use jj_waltz_native_jj::context;
use jj_waltz_native_jj::jj::JjClient;

pub const PREVIOUS_WORKSPACE_FILE: &str = "jw-prev-workspace";
pub const WORKSPACE_BOOKMARK_FILE: &str = "jw-bookmark";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceEntry {
    pub name: String,
    pub root: Option<PathBuf>,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct WorkspaceInventory {
    pub(super) entries: Vec<WorkspaceEntry>,
    current: String,
    previous: Option<String>,
    previous_error: Option<String>,
    default: Option<String>,
    current_root: PathBuf,
    pub(super) primary_root: Option<PathBuf>,
}

impl WorkspaceInventory {
    pub fn load() -> Result<Self> {
        let current_root = canonicalize_dir(&workspace_root_current()?)?;
        let names = workspace_names()?;
        let mut entries = names
            .iter()
            .map(|name| {
                Ok(WorkspaceEntry {
                    name: name.clone(),
                    root: workspace_root_by_name_direct(name)?,
                    is_current: false,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let matches = entries
            .iter()
            .filter_map(|entry| match entry.root.as_deref().map(canonicalize_dir) {
                Some(Ok(root)) if root == current_root => Some(entry.name.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let current = match matches.as_slice() {
            [] => {
                let target_candidates = current_workspace_names_by_target()?;
                let candidates = entries
                    .iter()
                    .filter(|entry| entry.root.is_none() && target_candidates.contains(&entry.name))
                    .map(|entry| entry.name.clone())
                    .collect::<Vec<_>>();
                match candidates.as_slice() {
                    [name] => {
                        entries.iter_mut().find(|entry| entry.name == *name).expect("candidate came from entries").root = Some(current_root.clone());
                        name.clone()
                    }
                    _ => bail!("could not determine current workspace for root {}", current_root.display()),
                }
            }
            [name] => name.clone(),
            _ => bail!("multiple workspaces match current root {}: {}", current_root.display(), matches.join(", ")),
        };

        for entry in &mut entries {
            entry.is_current = entry.name == current;
        }

        // Placement follows JJ repository-storage ownership, not Git colocation or a
        // naming convention. The owner can still exist after its workspace is forgotten.
        let repository = context::repository_path(&current_root).map_err(anyhow::Error::msg)?;
        let primary_root = repository
            .parent()
            .filter(|parent| parent.file_name().is_some_and(|name| name == ".jj"))
            .and_then(Path::parent)
            .filter(|root| root.join(".jj/repo").is_dir() && root.join(".jj/repo").canonicalize().ok().as_ref() == Some(&repository))
            .map(Path::to_path_buf);
        let default = names.iter().find(|name| *name == "default").cloned().or_else(|| {
            primary_root.as_deref().and_then(|primary| {
                entries
                    .iter()
                    .find(|entry| {
                        entry
                            .root
                            .as_deref()
                            .is_some_and(|root| canonicalize_dir(root).ok().as_deref() == Some(primary))
                    })
                    .map(|entry| entry.name.clone())
            })
        });

        let (previous, previous_error) = read_previous_workspace(&current_root, &names)?;

        Ok(Self {
            entries,
            current,
            previous,
            previous_error,
            default,
            current_root,
            primary_root,
        })
    }

    pub fn entries(&self) -> &[WorkspaceEntry] {
        &self.entries
    }

    pub fn current_name(&self) -> &str {
        &self.current
    }

    pub fn current_root(&self) -> &Path {
        &self.current_root
    }

    pub fn previous(&self) -> Option<&str> {
        self.previous.as_deref()
    }

    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }

    pub fn primary_root(&self) -> Option<&Path> {
        self.primary_root.as_deref()
    }

    /// The physical JJ store owner, never a workspace merely named `default`.
    /// Recheck the real directory and repository pointer before trusting local policy.
    pub fn primary_policy_root(&self) -> Result<&Path> {
        let primary = self
            .primary_root
            .as_deref()
            .context("cannot verify primary checkout for ignored-copy policy; use --copy-ignored or --no-copy-ignored")?;
        let pointer = primary.join(".jj/repo");
        let repository = context::repository_path(&self.current_root).map_err(anyhow::Error::msg)?;
        if !fs::symlink_metadata(primary)?.is_dir() || !fs::symlink_metadata(&pointer)?.is_dir() || pointer.canonicalize()? != repository {
            bail!("primary policy owner no longer owns the JJ repository");
        }
        Ok(primary)
    }

    pub fn previous_name(&self) -> Result<&str> {
        self.previous
            .as_deref()
            .ok_or_else(|| anyhow!("{}", self.previous_error.as_deref().unwrap_or("no previous workspace recorded")))
    }

    pub fn default_name(&self) -> Result<&str> {
        self.default.as_deref().ok_or_else(|| anyhow!("could not determine default workspace"))
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|entry| entry.name == name)
    }

    pub fn resolve(&self, token: &str) -> Result<String> {
        match token {
            "@" => Ok(self.current.clone()),
            "-" => Ok(self.previous_name()?.to_owned()),
            "^" | "default" => Ok(self.default_name()?.to_owned()),
            other => Ok(other.to_owned()),
        }
    }

    pub fn root(&self, name: &str) -> Result<PathBuf> {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .and_then(|entry| entry.root.clone())
            .ok_or_else(|| anyhow!("workspace not found: {name}"))
    }

    pub fn marker(&self, name: &str) -> char {
        if self.current == name {
            '@'
        } else if self.previous.as_deref() == Some(name) {
            '-'
        } else if self.default.as_deref() == Some(name) {
            '^'
        } else {
            ' '
        }
    }

    pub fn record_created(&mut self, name: &str, root: &Path) {
        self.entries.push(WorkspaceEntry {
            name: name.to_owned(),
            root: Some(root.to_path_buf()),
            is_current: false,
        });
    }
}

pub fn current_workspace_name() -> Result<String> {
    Ok(WorkspaceInventory::load()?.current)
}

fn workspace_root_by_name_direct(name: &str) -> Result<Option<PathBuf>> {
    let output = JjClient::current()?.run_unchecked(["workspace", "root", "--name", name])?;

    if output.success() {
        return Ok(Some(PathBuf::from(output.trimmed_stdout()?)));
    }
    let message = output.stderr();
    let message = if message.is_empty() {
        "workspace root lookup failed".to_owned()
    } else {
        message
    };
    let missing_checkout = message.contains("Cannot resolve absolute workspace path")
        && (message.contains("No such file or directory") || message.contains("os error 2") || message.contains("os error 3"))
        || message.contains("Workspace has no recorded path");
    if missing_checkout {
        Ok(None)
    } else {
        bail!("failed to resolve workspace root for {name}: {message}")
    }
}

pub fn workspace_root_current() -> Result<PathBuf> {
    let output = JjClient::current()?.run(["workspace", "root"])?;
    Ok(PathBuf::from(output.trimmed_stdout()?))
}

pub fn path_for_workspace(token: &str) -> Result<PathBuf> {
    let inventory = WorkspaceInventory::load()?;
    let name = inventory.resolve(token)?;
    inventory.root(&name)
}

pub fn completion_workspace_candidates() -> Result<Vec<(String, String)>> {
    let inventory = WorkspaceInventory::load()?;

    let mut candidates = Vec::new();

    for entry in inventory.entries() {
        let description = if inventory.current_name() == entry.name {
            "Existing workspace (current)"
        } else if inventory.previous().is_some_and(|previous| previous == entry.name) {
            "Existing workspace (previous)"
        } else if inventory.default().is_some_and(|default| default == entry.name) {
            "Existing workspace (default)"
        } else {
            "Existing workspace"
        };
        candidates.push((entry.name.clone(), description.to_owned()));
    }

    candidates.push(("@".to_owned(), "Current workspace".to_owned()));
    candidates.push(("-".to_owned(), "Previous workspace".to_owned()));
    candidates.push(("^".to_owned(), "Default workspace".to_owned()));

    Ok(candidates)
}

pub fn workspace_state_file(root: &Path) -> PathBuf {
    root.join(".jj").join(PREVIOUS_WORKSPACE_FILE)
}

pub fn workspace_bookmark_file(root: &Path) -> PathBuf {
    root.join(".jj").join(WORKSPACE_BOOKMARK_FILE)
}

pub fn legacy_workspace_bookmark(root: &Path) -> Result<Option<String>> {
    let path = workspace_bookmark_file(root);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read workspace bookmark {}", path.display()));
        }
    };
    let names = contents.lines().map(str::trim).filter(|line| !line.is_empty()).collect::<Vec<_>>();
    match names.as_slice() {
        [name] => Ok(Some((*name).to_owned())),
        _ => bail!("workspace bookmark record is invalid: {}", path.display()),
    }
}

pub fn validate_workspace_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("workspace name cannot be empty")
    }
    if name.contains('\n') {
        bail!("workspace name cannot contain newlines")
    }
    if name.contains("..") {
        bail!("workspace name cannot contain '..'")
    }
    if name.starts_with('/') {
        bail!("workspace name cannot start with '/'")
    }
    if name.starts_with('-') {
        bail!("workspace name cannot start with '-'")
    }
    if name.contains(':') {
        bail!("workspace name cannot contain ':'")
    }
    Ok(())
}

fn read_previous_workspace(root: &Path, names: &[String]) -> Result<(Option<String>, Option<String>)> {
    let state_path = workspace_state_file(root);
    let contents = match fs::read_to_string(&state_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((None, Some("no previous workspace recorded".to_owned())));
        }
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read workspace state {}", state_path.display()));
        }
    };
    let recorded = contents.lines().map(str::trim).filter(|line| !line.is_empty()).collect::<Vec<_>>();
    if recorded.is_empty() {
        return Ok((None, Some("no previous workspace recorded".to_owned())));
    }
    if recorded.len() != 1 {
        return Ok((None, Some(format!("previous workspace record is invalid: {}", state_path.display()))));
    }
    if names.iter().any(|name| name == recorded[0]) {
        Ok((Some(recorded[0].to_owned()), None))
    } else {
        Ok((None, Some("no previous workspace recorded".to_owned())))
    }
}

fn workspace_names() -> Result<Vec<String>> {
    JjClient::current()?.workspace_names()
}

fn current_workspace_names_by_target() -> Result<Vec<String>> {
    JjClient::current()?.current_workspace_target_names()
}

fn canonicalize_dir(path: &Path) -> Result<PathBuf> {
    path.canonicalize().with_context(|| format!("failed to resolve {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_workspace_names() {
        assert!(validate_workspace_name("feature").is_ok());
        assert!(validate_workspace_name("").is_err());
        assert!(validate_workspace_name("../bad").is_err());
        assert!(validate_workspace_name("-bad").is_err());
        assert!(validate_workspace_name("bad:name").is_err());
    }
}
