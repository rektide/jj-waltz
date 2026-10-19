//! Static policy over facts already observed by comparison/copy/removal.
use crate::cow::ignored::{Patterns, Prefix, Selection};
use anyhow::{Context, Result, bail};
use diff_dirs::{Check, CheckError, Entry, Side};
use std::collections::BTreeSet;
use std::fs;
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
    let boundaries: BTreeSet<_> = selected.iter().map(PathBuf::as_path).collect();
    outside_with(root, allow_git, &(), &mut |path, _, _| Ok((!boundaries.contains(path)).then_some(())))
}

/// Discover authorized outer roots during the donor's existing outside-payload
/// validation iteration, rather than prewalking the same directory prefixes.
pub(super) fn select_outside<'a>(
    source: &Path,
    patterns: &Patterns,
    tracked: impl IntoIterator<Item = &'a Path>,
    is_ignored: impl FnMut(&Path) -> Result<bool>,
) -> Result<Selection> {
    crate::cow::payload::directory_identity(source)?;
    let mut selector = patterns.selector(tracked, is_ignored);
    let prefix = selector.root();
    outside_with(source, true, &prefix, &mut |path, metadata, prefix| {
        if !metadata.is_dir() {
            return Ok(Some(Prefix::default()));
        }
        let next = selector.advance(prefix, path.file_name().expect("visited entry has a name"))?;
        Ok((!selector.select(path, &next)?).then_some(next))
    })?;
    Ok(selector.finish())
}

/// A boundary returns None to prune an entry, or the state for its children.
/// Frozen roots and live selection share this same filesystem iteration.
fn outside_with<S>(root: &Path, allow_git: bool, state: &S, boundary: &mut impl FnMut(&Path, &fs::Metadata, &S) -> Result<Option<S>>) -> Result<()> {
    fn visit<S>(root: &Path, path: &Path, allow_git: bool, state: &S, boundary: &mut impl FnMut(&Path, &fs::Metadata, &S) -> Result<Option<S>>) -> Result<()> {
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
            let Some(next) = boundary(path.strip_prefix(root).expect("walk starts at root"), &metadata, state)? else {
                continue;
            };
            if metadata.is_dir() {
                visit(root, &path, allow_git, &next, boundary)?;
            } else if !metadata.is_file() && !metadata.is_symlink() {
                bail!("special entry is unsupported: {}", path.display());
            }
        }
        Ok(())
    }
    visit(root, root, allow_git, state, boundary)
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

    #[test]
    fn donor_iteration_selects_the_same_outer_roots_without_entering_payloads() {
        let temp = tempfile::tempdir().unwrap();
        for path in [
            "build/tracked",
            "build/packages/a/node_modules",
            "packages/a/node_modules/deep/target/.git",
            "packages/b/target",
            "packages/λ/dist",
            "unmatched/ordinary",
            ".jj/node_modules",
            ".git/target",
        ] {
            fs::create_dir_all(temp.path().join(path)).unwrap();
        }
        fs::write(temp.path().join("packages/b/target-file"), "not a directory").unwrap();
        std::os::unix::fs::symlink("packages/a", temp.path().join("linked")).unwrap();
        let patterns = Patterns::compile(&[
            "**/node_modules/".into(),
            "**/target".into(),
            "**/build/".into(),
            "packages/?/dist/".into(),
            "packages/a/node_modules/deep/target".into(),
            "absent/".into(),
        ])
        .unwrap();
        let tracked = [PathBuf::from("build/tracked/file")];
        let mut checked = Vec::new();
        let selected = select_outside(temp.path(), &patterns, tracked.iter().map(PathBuf::as_path), |path| {
            checked.push(path.to_owned());
            Ok(path != Path::new("build"))
        })
        .unwrap();
        let expected = patterns
            .select(temp.path(), tracked.iter().map(PathBuf::as_path), |path| Ok(path != Path::new("build")))
            .unwrap();
        assert_eq!(selected.roots, expected.roots);
        assert_eq!(selected.skipped, expected.skipped);
        assert_eq!(
            selected.roots,
            [
                "build/packages/a/node_modules",
                "packages/a/node_modules",
                "packages/b/target",
                "packages/λ/dist"
            ]
            .map(PathBuf::from)
        );
        assert_eq!(selected.skipped.len(), 2);
        assert!(!checked.iter().any(|path| path.starts_with("linked")
            || path.starts_with(".jj")
            || path.starts_with(".git")
            || path == Path::new("packages/a/node_modules/deep/target")));
        // Selection pruned the bad interior; the unchanged streaming payload
        // guard still rejects it when payload comparison/copy actually reaches it.
        assert!(diff_dirs::audit(temp.path().join("packages/a/node_modules"), Side::Original, &Guard::default()).is_err());
    }

    #[test]
    fn donor_iteration_authorizes_roots_before_pruning_and_propagates_probe_errors() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("cache/nested")).unwrap();
        let patterns = Patterns::compile(&["cache/".into(), "cache/nested".into()]).unwrap();
        let tracked = [PathBuf::from("cache/nested/forced-tracked")];
        let error = select_outside(temp.path(), &patterns, tracked.iter().map(PathBuf::as_path), |_| Ok(true)).unwrap_err();
        assert!(error.to_string().contains("overlaps tracked"), "{error:#}");
        let error = select_outside(temp.path(), &patterns, [], |_| bail!("frozen ignore probe failed")).unwrap_err();
        assert!(error.to_string().contains("frozen ignore probe failed"), "{error:#}");

        fs::create_dir(temp.path().join("cache/nested/.jj")).unwrap();
        assert_eq!(
            select_outside(temp.path(), &patterns, [], |_| Ok(true)).unwrap().roots,
            [PathBuf::from("cache")]
        );
        let error = select_outside(temp.path(), &patterns, [], |_| Ok(false)).unwrap_err();
        assert!(error.to_string().contains("administrative"), "{error:#}");
    }

    #[test]
    fn donor_iteration_still_validates_unmatched_regions_and_empty_selections() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("ordinary/deeper")).unwrap();
        std::os::unix::fs::symlink("ordinary", temp.path().join("cache")).unwrap();
        for patterns in [vec![], vec!["cache/".into()], vec!["missing/".into()]] {
            let patterns = Patterns::compile(&patterns).unwrap();
            let selected = select_outside(temp.path(), &patterns, [], |_| panic!("no real directory matches")).unwrap();
            assert!(selected.roots.is_empty());
        }
        fs::create_dir(temp.path().join("ordinary/deeper/.git")).unwrap();
        for patterns in [vec![], vec!["cache/".into()], vec!["missing/".into()]] {
            let patterns = Patterns::compile(&patterns).unwrap();
            let error = select_outside(temp.path(), &patterns, [], |_| panic!("no real directory matches")).unwrap_err();
            assert!(error.to_string().contains("administrative"), "{error:#}");
        }
    }
}
