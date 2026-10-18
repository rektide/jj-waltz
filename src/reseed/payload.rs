//! Static policy over facts already observed by comparison/copy/removal.
use anyhow::{Context, Result, bail};
use diff_dirs::{Check, CheckError, Entry, Side};
use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Default)]
pub(super) struct Guard {
    warnings: Mutex<BTreeSet<String>>,
}
impl Check for Guard {
    fn check(&self, entry: Entry<'_>) -> Result<(), CheckError> {
        if entry.name == ".git" || entry.name == ".jj" {
            return Err("it is an administrative entry".into());
        }
        let kind = entry.metadata.file_type();
        if !kind.is_dir() && !kind.is_file() && !kind.is_symlink() {
            return Err("it is an unsupported special file".into());
        }
        // Diagnostic-only side channel; ordinary entries never lock or format paths.
        if entry.side == Side::Original
            && let Some(link) = entry.link_target.filter(|link| link.is_absolute())
        {
            self.warnings.lock().unwrap().insert(format!(
                "absolute dependency symlink {} -> {} (not self-contained)",
                entry.path.display(),
                link.display()
            ));
        }
        Ok(())
    }
}
impl Guard {
    pub(super) fn warnings(&self) -> Vec<String> {
        self.warnings.lock().unwrap().iter().cloned().collect()
    }
}

/// Validate retained/tracked regions, stopping exactly at selected payload roots.
/// Their interiors belong to streaming execution (or explicit dry-run audit).
pub(super) fn outside(root: &Path, allow_git: bool, selected: &[PathBuf]) -> Result<()> {
    fn visit(root: &Path, path: &Path, allow_git: bool, selected: &[PathBuf]) -> Result<()> {
        for child in fs::read_dir(path).with_context(|| format!("cannot read {}", path.display()))? {
            let child = child?;
            let path = child.path();
            let name = child.file_name();
            if child.path().parent() == Some(root) && (name == ".jj" || (allow_git && name == ".git")) {
                continue;
            }
            if name == ".jj" || name == ".git" {
                bail!("refusing {}: it is an administrative entry", path.display());
            }
            let metadata = fs::symlink_metadata(&path)?;
            #[cfg(not(target_os = "linux"))]
            crate::cow::validate_same_mount(path.parent().unwrap(), &path)?;
            if selected.iter().any(|p| root.join(p) == path) {
                continue;
            }
            if metadata.is_dir() {
                visit(root, &path, allow_git, selected)?;
            } else if !metadata.is_file() && !metadata.is_symlink() {
                bail!("special entry is unsupported: {}", path.display());
            }
        }
        Ok(())
    }
    visit(root, root, allow_git, selected)
}

pub(super) fn audit_target(root: &Path, selected: &[PathBuf], guard: &Guard) -> Result<()> {
    for path in selected {
        let path = root.join(path);
        match fs::symlink_metadata(&path) {
            Ok(_) => {
                diff_dirs::audit(&path, Side::Target, guard)?;
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
