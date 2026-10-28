//! Source-qualified ordinary authority, evaluated during production inspection.
//! No realized-root set, payload census, native commands or mutable admission map.
use crate::frozen::FrozenIgnore;
use crate::ignored::Patterns;
use anyhow::{Context, Result, bail};
use diff_dirs::{Action, CheckError, Container, FileKind, Names, Operations, Visitor};
use std::collections::BTreeSet;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug)]
pub struct SourceQualified {
    pub patterns: Arc<Patterns>,
    pub source_ignore: Arc<FrozenIgnore>,
    pub desired_ignore: Arc<FrozenIgnore>,
    /// Whole S/D native leaf names; ancestors are structural, not exclusions.
    pub native: Arc<BTreeSet<PathBuf>>,
    /// Active links only. A source==target private-directory rule is not a link.
    pub links: BTreeSet<PathBuf>,
    pub devices: (u64, u64),
}

#[derive(Debug, PartialEq, Eq)]
enum Disposition {
    Preserve,
    Observe,
    Mirror,
}

fn overlaps(paths: &BTreeSet<PathBuf>, path: &Path) -> bool {
    path.ancestors().any(|p| paths.contains(p)) || paths.range(path.to_path_buf()..).next().is_some_and(|p| p.starts_with(path))
}

impl SourceQualified {
    fn candidate(&self, path: &Path) -> Result<Option<PathBuf>> {
        let mut current = PathBuf::new();
        for part in path.components() {
            current.push(part);
            if self.patterns.matches(&current)? && self.source_ignore.ignored(&current, true) {
                return Ok(Some(current));
            }
        }
        Ok(None)
    }

    /// Unknown means genuinely present but not yet typed. Never use it as
    /// absence, or suppress a possible selected/native conflict before late.
    fn disposition(&self, path: &Path, source: Option<FileKind>) -> Result<Disposition> {
        // Ancestor authority is independent of this entry's source presence or
        // kind. Do not decode the final name as a selection directory yet.
        let inside = self.candidate(path.parent().unwrap_or_else(|| Path::new("")))?.is_some();
        let admin = path.file_name().is_some_and(|n| n == ".jj" || n == ".git");
        if admin {
            if inside {
                bail!("administrative entry inside selected ordinary root");
            }
            return Ok(Disposition::Preserve);
        }
        // Reachability invariant: only a real source directory can admit its Q
        // boundary. Target-only candidates and source links stop at that name.
        // Thus a visited descendant of Q already has source-qualified authority,
        // including during opaque source-copy and target-removal recursion.
        if inside {
            return Ok(Disposition::Mirror);
        }
        let Some(kind) = source else { return Ok(Disposition::Preserve) };
        if !matches!(kind, FileKind::Unknown | FileKind::Directory) {
            return Ok(Disposition::Preserve);
        }
        // A listing without a kind hint may still describe an unrelated byte-
        // named leaf. Let late facts distinguish it from a routing directory;
        // that directory retains the selector's existing UTF-8 requirement.
        if kind == FileKind::Unknown && path.file_name().is_some_and(|name| name.to_str().is_none()) {
            return Ok(Disposition::Observe);
        }
        if self.patterns.matches(path)? && self.source_ignore.ignored(path, true) {
            if kind == FileKind::Unknown {
                return Ok(Disposition::Observe);
            }
            if overlaps(&self.native, path) {
                bail!("selected ordinary root overlaps native tracked leaf");
            }
            if overlaps(&self.links, path) {
                bail!("selected ordinary root overlaps active link");
            }
            if !self.desired_ignore.ignored(path, true) {
                bail!("selected ordinary root is not ignored under frozen desired rules");
            }
            return Ok(Disposition::Mirror);
        }
        // A declared link owns this whole name even if a broad wildcard could
        // otherwise search below it. Only an actual Q boundary above conflicts.
        if self.links.contains(path) {
            return Ok(Disposition::Preserve);
        }
        if !self.patterns.may_match_below(path)? {
            return Ok(Disposition::Preserve);
        }
        if self.native.contains(path) {
            if kind == FileKind::Unknown {
                return Ok(Disposition::Observe);
            }
            bail!("native leaf or active link blocks ordinary selection route");
        }
        Ok(Disposition::Observe)
    }
}

impl Visitor for SourceQualified {
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        if names.is_root() {
            return Ok(());
        }
        for name in names.entries_mut() {
            match self.disposition(name.relative_path(), name.source().map(|f| f.kind())) {
                Ok(Disposition::Preserve) => name.preserve_tree(),
                Ok(_) => {}
                Err(e) => return Err(name.refuse(e.to_string())),
            }
        }
        Ok(())
    }

    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        let root = batch.is_root();
        for entry in batch.entries_mut() {
            if root {
                entry.set_action(Action::Directory {
                    container: Container::Keep,
                    descend: true,
                });
                continue;
            }
            let source = entry.source().map(|f| {
                let m = f.metadata();
                if m.is_dir() {
                    FileKind::Directory
                } else if m.is_file() {
                    FileKind::RegularFile
                } else if m.is_symlink() {
                    FileKind::Symlink
                } else {
                    FileKind::Other
                }
            });
            let disposition = self.disposition(entry.relative_path(), source).map_err(|e| entry.refuse(e.to_string()))?;
            if disposition == Disposition::Preserve {
                entry.preserve_tree();
                continue;
            }
            for (fact, device) in [(entry.source(), self.devices.0), (entry.target(), self.devices.1)] {
                if let Some(fact) = fact {
                    let m = fact.metadata();
                    if m.dev() != device {
                        return Err(entry.refuse("mounted filesystem boundary"));
                    }
                    if !(m.is_dir() || m.is_file() || m.is_symlink()) {
                        return Err(entry.refuse("unsupported special entry"));
                    }
                }
            }
            if disposition == Disposition::Observe {
                if entry.target().is_some_and(|f| !f.metadata().is_dir()) {
                    return Err(entry.refuse("ordinary routing ancestor is not a real target directory"));
                }
                entry.set_action(Action::Directory {
                    container: if entry.target().is_some() {
                        Container::Keep
                    } else {
                        Container::CreateIfNeeded
                    },
                    descend: true,
                });
            }
        }
        Ok(())
    }
}

/// First vertical seam: real one-lane coordinated application at workspace
/// roots. Fail immediately; never use drive's healthy-peer continuation policy.
/// Empty selection does not even construct/bind a runtime.
pub fn sync_one(source: &Path, target: &Path, name: &str, visitor: SourceQualified) -> Result<Option<diff_dirs_sync::coordinated::LaneReport>> {
    use diff_dirs::coordinated::TargetId;
    use diff_dirs_sync::coordinated::{SyncCursor, SyncRuntime, SyncTarget};
    if visitor.patterns.is_empty() {
        return Ok(None);
    }
    let runtime = SyncRuntime::new(vec![SyncTarget {
        id: TargetId(name.to_owned()),
        // Fresh, process-local runtime only; never reconstructed with a changed
        // policy. Do not serialize the whole native index into a descriptor.
        descriptor: "jw-source-qualified-v2;frozen-process-local-n1".into(),
        applier: diff_dirs_sync::Applier::new(source, target, diff_dirs_sync::CopyPolicy::ReflinkRequired)?.visitor(visitor),
    }]);
    let mut cursor = SyncCursor::new(&runtime, diff_dirs::Execution::Sequential)?;
    let result = (|| {
        while let Some(group) = cursor.next_group(&runtime)? {
            for lane in &group.outcomes {
                if let Err(error) = &lane.outcome {
                    return Err(anyhow::Error::new(Arc::clone(error)).context("ordinary inspection"));
                }
            }
            for target in cursor.pending() {
                cursor.apply(&runtime, &group.token, &target)?;
            }
            cursor.settle()?;
        }
        Ok::<_, anyhow::Error>(())
    })();
    result.with_context(|| format!("ordinary transfer partial reports: {:?}", cursor.reports()))?;
    let report = cursor.reports().pop().context("missing ordinary lane report")?;
    if !report.satisfied {
        bail!("ordinary lane did not satisfy policy: {report:?}");
    }
    Ok(Some(report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::symlink;

    struct Fixture {
        _temp: tempfile::TempDir,
        source: PathBuf,
        target: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/streaming-tests");
            fs::create_dir_all(&scratch).unwrap();
            let temp = tempfile::tempdir_in(scratch.canonicalize().unwrap()).unwrap();
            let source = temp.path().join("source");
            let target = temp.path().join("target");
            fs::create_dir_all(source.join(".jj")).unwrap();
            fs::create_dir_all(target.join(".jj")).unwrap();
            Self { _temp: temp, source, target }
        }
        fn visitor(&self, patterns: &[&str], native: &[&str], links: &[&str]) -> SourceQualified {
            SourceQualified {
                patterns: Arc::new(Patterns::compile(&patterns.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap()),
                source_ignore: Arc::new(FrozenIgnore::test_rules(b"cache/\n**/node_modules/\nforced/\n")),
                desired_ignore: Arc::new(FrozenIgnore::test_rules(b"cache/\n**/node_modules/\nforced/\n")),
                native: Arc::new(native.iter().map(PathBuf::from).collect()),
                links: links.iter().map(PathBuf::from).collect(),
                devices: (fs::metadata(&self.source).unwrap().dev(), fs::metadata(&self.target).unwrap().dev()),
            }
        }
        fn run(&self, patterns: &[&str]) -> Result<Option<diff_dirs_sync::coordinated::LaneReport>> {
            sync_one(&self.source, &self.target, "test", self.visitor(patterns, &[], &[]))
        }
    }

    #[test]
    fn pure_known_unknown_and_overlap_ordering() {
        let f = Fixture::new();
        let v = f.visitor(&["**/node_modules", "forced"], &["forced/file", "blocked"], &["linked"]);
        for owned in ["forced", "blocked"] {
            assert_eq!(v.disposition(Path::new(owned), Some(FileKind::Unknown)).unwrap(), Disposition::Observe);
            assert!(v.disposition(Path::new(owned), Some(FileKind::Directory)).is_err(), "{owned}");
            assert_eq!(v.disposition(Path::new(owned), Some(FileKind::Symlink)).unwrap(), Disposition::Preserve);
        }
        assert_eq!(v.disposition(Path::new("linked"), Some(FileKind::Unknown)).unwrap(), Disposition::Preserve);
        assert_eq!(v.disposition(Path::new("node_modules"), None).unwrap(), Disposition::Preserve);
        assert_eq!(v.disposition(Path::new("node_modules/old"), None).unwrap(), Disposition::Mirror);
        assert!(v.disposition(Path::new("node_modules/.git"), None).is_err());
        let v = f.visitor(&["forced"], &[], &["forced"]);
        assert!(v.disposition(Path::new("forced"), Some(FileKind::Directory)).is_err());
        let mut v = f.visitor(&["cache"], &[], &[]);
        v.desired_ignore = Arc::new(FrozenIgnore::default());
        assert!(
            v.disposition(Path::new("cache"), Some(FileKind::Directory))
                .unwrap_err()
                .to_string()
                .contains("not ignored")
        );
    }

    #[test]
    fn non_utf8_leaf_classification_defers_unknown_without_losing_ancestor_authority() {
        let f = Fixture::new();
        let v = f.visitor(&["cache"], &[], &[]);
        let name = PathBuf::from(OsString::from_vec(vec![0xff]));
        for kind in [None, Some(FileKind::RegularFile), Some(FileKind::Symlink), Some(FileKind::Other)] {
            assert_eq!(v.disposition(&name, kind).unwrap(), Disposition::Preserve, "{kind:?}");
        }
        assert_eq!(v.disposition(&name, Some(FileKind::Unknown)).unwrap(), Disposition::Observe);
        assert!(
            v.disposition(&name, Some(FileKind::Directory))
                .unwrap_err()
                .to_string()
                .contains("directory selection entry not UTF-8")
        );
        // Once Q was admitted, even absent sources (including opaque removal
        // interiors) retain mirror authority without decoding their byte names.
        let inside = Path::new("cache").join(&name).join(&name);
        for kind in [
            None,
            Some(FileKind::Unknown),
            Some(FileKind::Directory),
            Some(FileKind::RegularFile),
            Some(FileKind::Symlink),
        ] {
            assert_eq!(v.disposition(&inside, kind).unwrap(), Disposition::Mirror, "{kind:?}");
        }
    }

    #[test]
    fn target_only_non_utf8_local_is_preserved_through_real_cursor() {
        let f = Fixture::new();
        let name = OsString::from_vec(vec![0xff]);
        fs::write(f.target.join(&name), "local").unwrap();
        let mut v = f.visitor(&["cache"], &[], &[]);
        v.source_ignore = Arc::new(FrozenIgnore::default());
        v.desired_ignore = Arc::new(FrozenIgnore::default());
        let done = sync_one(&f.source, &f.target, "byte-local", v).unwrap().unwrap();
        assert!(done.satisfied);
        assert_eq!(done.totals.completed, Default::default());
        assert_eq!(fs::read(f.target.join(&name)).unwrap(), b"local");
    }

    #[test]
    fn source_non_utf8_unselected_leaves_preserve_local_collisions_and_absence() {
        let f = Fixture::new();
        for (index, paired) in [(0xff, true), (0xfe, false)] {
            let file = OsString::from_vec(vec![index, b'f']);
            let link = OsString::from_vec(vec![index, b'l']);
            fs::write(f.source.join(&file), "donor").unwrap();
            symlink("donor-destination", f.source.join(&link)).unwrap();
            if paired {
                fs::write(f.target.join(&file), "local").unwrap();
                symlink("local-destination", f.target.join(&link)).unwrap();
            }
        }
        let done = f.run(&["cache"]).unwrap().unwrap();
        assert!(done.satisfied);
        assert_eq!(done.totals.completed, Default::default());
        assert_eq!(fs::read(f.target.join(OsString::from_vec(vec![0xff, b'f']))).unwrap(), b"local");
        assert_eq!(
            fs::read_link(f.target.join(OsString::from_vec(vec![0xff, b'l']))).unwrap(),
            Path::new("local-destination")
        );
        for suffix in *b"fl" {
            assert!(fs::symlink_metadata(f.target.join(OsString::from_vec(vec![0xfe, suffix]))).is_err());
        }
    }

    #[test]
    fn target_only_non_utf8_selected_interiors_are_removed_through_real_cursor() {
        let f = Fixture::new();
        let name = OsString::from_vec(vec![0xff]);
        fs::create_dir_all(f.source.join("cache")).unwrap();
        fs::create_dir_all(f.target.join("cache/stale").join(&name)).unwrap();
        fs::create_dir_all(f.target.join("cache").join(&name)).unwrap();
        fs::write(f.target.join("cache").join(OsString::from_vec(vec![0xfe])), "paired child").unwrap();
        fs::write(f.target.join("cache/stale").join(&name).join(&name), "opaque removal child").unwrap();
        fs::write(f.target.join("cache").join(&name).join(&name), "byte directory child").unwrap();
        let done = f.run(&["cache"]).unwrap().unwrap();
        assert!(done.satisfied);
        assert!(f.target.join("cache").is_dir());
        assert_eq!(fs::read_dir(f.target.join("cache")).unwrap().count(), 0);
        assert_eq!(done.totals.completed.files_removed, 3);
        assert_eq!(done.totals.completed.directories_removed, 3);
        assert_eq!(f.run(&["cache"]).unwrap().unwrap().totals.completed, Default::default());
    }

    #[test]
    fn source_qualified_mirror_and_target_only_root_retention_through_real_cursor() {
        let f = Fixture::new();
        crate::probe_between(&f.source, &f.target).unwrap();
        fs::create_dir_all(f.source.join("pkg/cache/new")).unwrap();
        fs::create_dir_all(f.target.join("pkg/cache/old/deep")).unwrap();
        fs::create_dir_all(f.target.join("vanished/cache")).unwrap();
        fs::create_dir_all(f.source.join("notignored/node_modules")).unwrap();
        fs::create_dir_all(f.target.join("notignored")).unwrap();
        fs::write(f.source.join("pkg/cache/new/file"), "new").unwrap();
        fs::write(f.source.join("notignored/node_modules/file"), "nested match").unwrap();
        fs::write(f.target.join("pkg/cache/old/deep/file"), "old").unwrap();
        fs::write(f.target.join("vanished/cache/local"), "retain").unwrap();
        fs::write(f.source.join(".env"), "donor").unwrap();
        fs::write(f.target.join(".env"), "local").unwrap();
        symlink("elsewhere", f.source.join("cache")).unwrap();
        fs::create_dir_all(f.target.join("cache")).unwrap();
        fs::write(f.target.join("cache/local"), "retain link candidate").unwrap();
        let patterns = ["**/cache", "notignored", "**/node_modules"];
        let report = f.run(&patterns).unwrap().unwrap();
        assert!(report.satisfied);
        assert_eq!(report.totals.completed.files_reflinked, 2);
        assert_eq!(report.totals.completed.files_copied, 0);
        assert!(!f.target.join("pkg/cache/old").exists());
        assert_eq!(fs::read(f.target.join("vanished/cache/local")).unwrap(), b"retain");
        assert_eq!(fs::read(f.target.join("cache/local")).unwrap(), b"retain link candidate");
        assert_eq!(fs::read(f.target.join(".env")).unwrap(), b"local");
        assert_eq!(f.run(&patterns).unwrap().unwrap().totals.completed, Default::default());
    }

    #[test]
    fn missing_unselected_route_without_matches_creates_no_skeleton_and_empty_policy_never_binds() {
        let f = Fixture::new();
        fs::create_dir_all(f.source.join("pkg/no-match")).unwrap();
        let done = f.run(&["**/node_modules"]).unwrap().unwrap();
        assert!(done.satisfied);
        assert_eq!(done.totals.completed, Default::default());
        assert!(!f.target.join("pkg").exists());
        // Nonexistent roots are fine only because empty policy does no runtime I/O.
        assert!(
            sync_one(
                Path::new("/nonexistent-source"),
                Path::new("/nonexistent-target"),
                "empty",
                f.visitor(&[], &[], &[])
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn wildcard_selection_discovers_through_absent_routes_without_unmatched_skeletons() {
        let f = Fixture::new();
        crate::probe_between(&f.source, &f.target).unwrap();
        fs::create_dir_all(f.source.join("packages/a/node_modules/empty")).unwrap();
        fs::create_dir_all(f.source.join("packages/a/src/unmatched")).unwrap();
        fs::create_dir_all(f.source.join("packages/b/src/unmatched")).unwrap();
        fs::write(f.source.join("packages/a/node_modules/file"), "opaque payload").unwrap();
        fs::create_dir_all(f.source.join("paired/nested/node_modules")).unwrap();
        fs::write(f.source.join("paired/nested/node_modules/file"), "paired payload").unwrap();
        fs::create_dir_all(f.target.join("paired/nested/node_modules/stale/deep")).unwrap();
        fs::write(f.target.join("paired/nested/node_modules/stale/deep/file"), "remove").unwrap();
        fs::create_dir_all(f.target.join("target-only/node_modules")).unwrap();
        fs::write(f.target.join("target-only/node_modules/local"), "retain absent source root").unwrap();
        let done = f.run(&["**/node_modules"]).unwrap().unwrap();
        assert!(done.satisfied);
        assert_eq!(done.totals.completed.files_reflinked, 2);
        assert_eq!(done.totals.completed.files_copied, 0);
        assert_eq!(done.totals.completed.directories_created, 4);
        assert!(f.target.join("packages/a/node_modules/empty").is_dir());
        assert!(!f.target.join("packages/a/src").exists());
        assert!(!f.target.join("packages/b").exists());
        assert!(!f.target.join("paired/nested/node_modules/stale").exists());
        assert_eq!(fs::read(f.target.join("target-only/node_modules/local")).unwrap(), b"retain absent source root");
        assert_eq!(f.run(&["**/node_modules"]).unwrap().unwrap().totals.completed, Default::default());
        fs::write(f.target.join("packages/a/node_modules/file"), "private").unwrap();
        assert_eq!(fs::read(f.source.join("packages/a/node_modules/file")).unwrap(), b"opaque payload");
    }

    #[test]
    fn selected_collision_opaque_copy_and_nested_administration_refusal() {
        let f = Fixture::new();
        crate::probe_between(&f.source, &f.target).unwrap();
        fs::create_dir_all(f.source.join("cache/new")).unwrap();
        fs::write(f.source.join("cache/new/file"), "data").unwrap();
        symlink(".jj", f.target.join("cache")).unwrap();
        fs::create_dir_all(f.source.join("cache/new/.git")).unwrap();
        assert!(format!("{:#}", f.run(&["cache"]).unwrap_err()).contains("administrative"));
        assert_eq!(fs::read_link(f.target.join("cache")).unwrap(), Path::new(".jj"));
        fs::remove_dir(f.source.join("cache/new/.git")).unwrap();
        let socket = std::os::unix::net::UnixListener::bind(f.source.join("cache/new/socket")).unwrap();
        assert!(format!("{:#}", f.run(&["cache"]).unwrap_err()).contains("special"));
        assert_eq!(fs::read_link(f.target.join("cache")).unwrap(), Path::new(".jj"));
        drop(socket);
        fs::remove_file(f.source.join("cache/new/socket")).unwrap();
        // Top-level native administration is excluded pair-wide before metadata.
        let _hidden = std::os::unix::net::UnixListener::bind(f.source.join(".jj/socket")).unwrap();
        assert_eq!(f.run(&["cache"]).unwrap().unwrap().totals.completed.files_reflinked, 1);
        fs::create_dir_all(f.source.join("cache/new/.git")).unwrap();
        assert!(format!("{:#}", f.run(&["cache"]).unwrap_err()).contains("administrative"));
        let v = f.visitor(&["cache"], &["cache/forced"], &[]);
        assert!(format!("{:#}", sync_one(&f.source, &f.target, "overlap", v).unwrap_err()).contains("overlaps native"));
    }
}
