//! Static policy over facts already observed by comparison/copy/removal.
use anyhow::{Context, Result, bail};
use diff_dirs::{Check, CheckError, Entry, Side};
use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// One command-owned pool, borrowed only by filesystem comparison/application.
/// JJ operations and frozen rule caches stay on the caller thread.
#[derive(Debug, Default)]
pub(super) struct Execution {
    #[cfg(feature = "payload-parallel")]
    pool: Option<rayon::ThreadPool>,
}
impl Execution {
    pub(super) fn new(has_payload: bool) -> Result<Self> {
        #[cfg(feature = "payload-parallel")]
        {
            let pool = if has_payload {
                let workers = std::thread::available_parallelism()?.get().min(4);
                Some(
                    rayon::ThreadPoolBuilder::new()
                        .num_threads(workers)
                        .thread_name(|index| format!("jw-payload-{index}"))
                        .build()?,
                )
            } else {
                None
            };
            Ok(Self { pool })
        }
        #[cfg(not(feature = "payload-parallel"))]
        {
            let _ = has_payload;
            Ok(Self {})
        }
    }

    pub(super) fn mirror<C: Check>(&self, applier: &diff_dirs_sync::Applier<C>) -> Result<diff_dirs_sync::Report, diff_dirs_sync::Failure> {
        #[cfg(feature = "payload-parallel")]
        if let Some(pool) = &self.pool {
            let jobs = std::num::NonZeroUsize::new(pool.current_num_threads()).expect("pool has workers");
            return applier.mirror_in_pool(pool, diff_dirs::ParallelLimits::new(jobs, 256));
        }
        applier.mirror()
    }
}

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
    let boundaries: BTreeSet<_> = selected.iter().map(|path| root.join(path)).collect();
    fn visit(root: &Path, path: &Path, allow_git: bool, boundaries: &BTreeSet<PathBuf>) -> Result<()> {
        for child in fs::read_dir(path).with_context(|| format!("cannot read {}", path.display()))? {
            let child = child?;
            let path = child.path();
            let name = child.file_name();
            if path.parent() == Some(root) && (name == ".jj" || (allow_git && name == ".git")) {
                continue;
            }
            if name == ".jj" || name == ".git" {
                bail!("refusing {}: it is an administrative entry", path.display());
            }
            let metadata = fs::symlink_metadata(&path)?;
            #[cfg(not(target_os = "linux"))]
            crate::cow::validate_same_mount(path.parent().unwrap(), &path)?;
            if boundaries.contains(&path) {
                continue;
            }
            if metadata.is_dir() {
                visit(root, &path, allow_git, boundaries)?;
            } else if !metadata.is_file() && !metadata.is_symlink() {
                bail!("special entry is unsupported: {}", path.display());
            }
        }
        Ok(())
    }
    visit(root, root, allow_git, &boundaries)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outside_prunes_only_exact_precomputed_payload_boundaries() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("pkg/cache/.jj")).unwrap();
        fs::create_dir_all(temp.path().join("pkg/cache-other")).unwrap();
        outside(temp.path(), false, &["pkg/cache".into()]).unwrap();
        fs::create_dir_all(temp.path().join("pkg/cache-other/.git")).unwrap();
        assert!(outside(temp.path(), false, &["pkg/cache".into()]).is_err());
    }

    #[cfg(feature = "payload-parallel")]
    #[test]
    fn command_pool_is_lazy_capped_and_has_matching_admission() {
        assert!(Execution::new(false).unwrap().pool.is_none());
        let execution = Execution::new(true).unwrap();
        let workers = execution.pool.as_ref().unwrap().current_num_threads();
        assert_eq!(workers, std::thread::available_parallelism().unwrap().get().min(4));
    }
}
