//! Compiled workspace boundaries over diffdirs' directory-local early/late batches.
//!
//! The legacy constructor is deliberately bounded: an empty selection authorizes
//! no auxiliary writes. It does not select a future broad-transfer default.
use anyhow::{Result, bail};
use diff_dirs::{Action, CheckError, Container, Names, Operations, Visitor};
use diff_dirs_sync::{Applier, CopyPolicy};
use std::collections::BTreeSet;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

/// One command-owned pool, borrowed by every selected workspace transfer.
/// Native JJ commands remain on the caller thread; no pool is created per root.
#[derive(Debug, Default)]
pub struct Execution {
    #[cfg(feature = "payload-parallel")]
    pool: Option<rayon::ThreadPool>,
}
impl Execution {
    pub fn new(has_payload: bool) -> Result<Self> {
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

    pub fn run<P: diff_dirs::Policy, M: diff_dirs::Comparator>(&self, applier: &Applier<P, M>) -> Result<diff_dirs_sync::Report, diff_dirs_sync::Failure> {
        #[cfg(feature = "payload-parallel")]
        if let Some(pool) = &self.pool {
            let jobs = std::num::NonZeroUsize::new(pool.current_num_threads()).expect("pool has workers");
            return applier.run_in_pool(pool, diff_dirs::ParallelLimits::new(jobs, 256));
        }
        applier.run()
    }
}

/// Frozen native/target-owned boundaries, not a discovered entry/operation map.
///
/// Early preservation suppresses *both* sides before child stat/readlink. Late
/// keeps routing containers independently of child work and refuses a routing
/// type collision rather than replacing unrelated local data. Within admitted
/// roots the normal finalized mirror actions drive writes **and** descent,
/// including one-sided preparation and coupled replacements.
///
/// For example, selecting `packages/a/cache` routes through `packages` and `a`,
/// but early-preserves `packages/a/local` even if it exists on both sides. A
/// target file at `packages` is not an interchangeable routing directory: late
/// refuses it instead of implicitly replacing it to reach the selected cache.
#[derive(Debug)]
pub struct Auxiliary {
    roots: BTreeSet<PathBuf>,
    routes: BTreeSet<PathBuf>,
    devices: (u64, u64),
    new_targets_only: bool,
}
impl Auxiliary {
    /// Compatibility boundary for existing copy_ignored callers. Native JJ leaf
    /// sets (source and desired) must be disjoint in *both* prefix directions:
    /// leaf == root, leaf below root, or leaf above root all conflict. Shared
    /// directory ancestors do not. No Git/JJ/filesystem probes occur in callbacks.
    pub fn legacy<'a>(roots: &[PathBuf], tracked: impl IntoIterator<Item = &'a Path>, devices: (u64, u64)) -> Result<Self> {
        let tracked = tracked.into_iter().collect::<BTreeSet<_>>();
        let mut routes = BTreeSet::new();
        let mut selected = BTreeSet::new();
        for root in roots {
            if root.as_os_str().is_empty()
                || root
                    .components()
                    .any(|part| !matches!(part, Component::Normal(name) if name != ".jj" && name != ".git"))
            {
                bail!("unsafe auxiliary root: {}", root.display());
            }
            if root.ancestors().any(|ancestor| tracked.contains(ancestor)) || tracked.range(root.as_path()..).next().is_some_and(|leaf| leaf.starts_with(root))
            {
                bail!("auxiliary root overlaps JJ tracked leaf: {}", root.display());
            }
            for ancestor in root.ancestors().skip(1) {
                routes.insert(ancestor.to_owned());
            }
            selected.insert(root.clone());
        }
        Ok(Self {
            roots: selected,
            routes,
            devices,
            new_targets_only: false,
        })
    }

    /// Creation keeps its create-only contract even at the late collision seam.
    pub fn new_targets_only(mut self) -> Self {
        self.new_targets_only = true;
        self
    }

    fn inside(&self, path: &Path) -> bool {
        path.ancestors().any(|ancestor| self.roots.contains(ancestor))
    }
}
impl Visitor for Auxiliary {
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        if names.is_root() {
            return Ok(());
        }
        for name in names.entries_mut() {
            let path = name.relative_path();
            if !self.inside(path) && !self.routes.contains(path) {
                name.preserve_tree();
            } else if name.name() == ".jj" || name.name() == ".git" {
                return Err(name.refuse("it is an administrative entry"));
            }
        }
        Ok(())
    }

    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        let root = batch.is_root();
        for entry in batch.entries_mut() {
            for (fact, device) in [(entry.source(), self.devices.0), (entry.target(), self.devices.1)] {
                if fact.is_some_and(|fact| fact.metadata().dev() != device) {
                    return Err(entry.refuse("it is a mounted filesystem boundary"));
                }
            }
            if !root && self.roots.contains(entry.relative_path()) && entry.source().is_some_and(|fact| !fact.metadata().is_dir()) {
                return Err(entry.refuse("legacy auxiliary source root is not a real directory"));
            }
            if !root && self.new_targets_only && self.roots.contains(entry.relative_path()) && entry.target().is_some() {
                return Err(entry.refuse("auxiliary creation target is obstructed"));
            }
            if root || !self.inside(entry.relative_path()) {
                if entry.source().is_some_and(|fact| !fact.metadata().is_dir()) || entry.target().is_some_and(|fact| !fact.metadata().is_dir()) {
                    return Err(entry.refuse("auxiliary routing ancestor is not a real directory"));
                }
                // In particular, do not accept a target-only routing directory's
                // baseline Remove. Its unselected descendants are target-owned.
                entry.set_action(Action::Directory {
                    container: if entry.target().is_some() { Container::Keep } else { Container::Create },
                    descend: true,
                });
            }
        }
        Ok(())
    }
}

/// Explicit --replace-content clearing, retaining the target's own `.jj` in
/// place throughout. This is not donor metadata adoption or a workspace swap.
/// Nested administration refuses before its immediate batch is changed; as with
/// transfer, an I/O failure can leave earlier batches completed, not rolled back.
pub struct WorkspaceClear;
impl Visitor for WorkspaceClear {
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        if names.is_root() {
            return Ok(());
        }
        let at_root = names.dir().as_os_str().is_empty();
        for name in names.entries_mut() {
            if at_root && name.name() == ".jj" {
                name.preserve_tree();
            } else if name.name() == ".jj" || name.name() == ".git" {
                return Err(name.refuse("it is an administrative entry"));
            }
        }
        Ok(())
    }
}

pub fn applier<'a>(
    source: &Path,
    target: &Path,
    visitor: &'a Auxiliary,
) -> Result<Applier<diff_dirs_sync::FullPolicy<diff_dirs::NoCheck, diff_dirs::NoScope, diff_dirs_sync::RemoveAll, diff_dirs::VisitorRef<'a, Auxiliary>>>> {
    Ok(Applier::new(source, target, CopyPolicy::ReflinkRequired)?.visitor(visitor.borrowed()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::os::unix::net::UnixListener;

    struct Fixture {
        _temp: tempfile::TempDir,
        source: PathBuf,
        target: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            // Canonicalize: the two-level crate manifest path plus `..` components
            // would otherwise push the fixture's unix-socket path past SUN_LEN.
            let scratch = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../.test-agent/visitor-tests")
                .canonicalize()
                .unwrap_or_else(|error| panic!("resolve visitor-tests scratch: {error}"));
            fs::create_dir_all(&scratch).unwrap();
            let temp = tempfile::tempdir_in(scratch).unwrap();
            let source = temp.path().join("source");
            let target = temp.path().join("target");
            fs::create_dir_all(source.join(".jj")).unwrap();
            fs::create_dir_all(target.join(".jj")).unwrap();
            Self { _temp: temp, source, target }
        }
        fn visitor(&self, roots: &[&str]) -> Auxiliary {
            Auxiliary::legacy(
                &roots.iter().map(PathBuf::from).collect::<Vec<_>>(),
                [],
                (fs::metadata(&self.source).unwrap().dev(), fs::metadata(&self.target).unwrap().dev()),
            )
            .unwrap()
        }
        fn cow(&self) -> bool {
            if let Err(error) = crate::probe_between(&self.source, &self.target) {
                assert!(
                    std::env::var_os("JW_TEST_REQUIRE_COW").is_none_or(|value| value.is_empty()),
                    "real reflink required: {error:#}"
                );
                eprintln!("skipping real reflink case: {error:#}");
                return false;
            }
            true
        }
    }

    #[test]
    fn root_visitor_reconciles_both_and_one_sided_batches_without_touching_local_names() {
        let f = Fixture::new();
        if !f.cow() {
            return;
        }
        fs::create_dir_all(f.source.join("pkg/cache/nested")).unwrap();
        fs::create_dir_all(f.source.join("new/cache")).unwrap();
        fs::create_dir_all(f.target.join("pkg/cache/old/deep")).unwrap();
        fs::create_dir_all(f.target.join("pkg/local/.git")).unwrap();
        fs::write(f.source.join("pkg/cache/nested/tool"), "#!/bin/sh\n").unwrap();
        fs::set_permissions(f.source.join("pkg/cache/nested/tool"), fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(f.source.join("new/cache/file"), "new\n").unwrap();
        fs::write(f.target.join("pkg/cache/old/deep/file"), "remove\n").unwrap();
        fs::write(f.target.join("pkg/local/.git/HEAD"), "target-owned\n").unwrap();
        fs::write(f.target.join(".env"), "unignored local secret\n").unwrap();
        fs::write(f.source.join(".env"), "donor secret must not transfer\n").unwrap();
        symlink("nested//./tool", f.source.join("pkg/cache/link")).unwrap();
        // If early preservation accidentally entered native metadata, this would
        // be an unsupported entry rather than a harmless hidden boundary.
        let _socket = UnixListener::bind(f.source.join(".jj/socket")).unwrap();
        let root_inode = fs::metadata(&f.target).unwrap().ino();
        let jj_inode = fs::metadata(f.target.join(".jj")).unwrap().ino();
        let visitor = f.visitor(&["pkg/cache", "new/cache"]);
        let done = Execution::new(true).unwrap().run(&applier(&f.source, &f.target, &visitor).unwrap()).unwrap();
        assert!(done.policy.satisfied);
        assert_eq!(done.completed.files_copied, 0);
        assert_eq!(done.completed.files_reflinked, 2);
        assert!(!f.target.join("pkg/cache/old").exists());
        assert_eq!(fs::read(f.target.join("new/cache/file")).unwrap(), b"new\n");
        assert_eq!(
            fs::read_link(f.target.join("pkg/cache/link")).unwrap().as_os_str(),
            Path::new("nested//./tool").as_os_str()
        );
        let src = fs::metadata(f.source.join("pkg/cache/nested/tool")).unwrap();
        let dst = fs::metadata(f.target.join("pkg/cache/nested/tool")).unwrap();
        assert_eq!(src.modified().unwrap(), dst.modified().unwrap());
        assert_eq!(dst.permissions().mode() & 0o7777, 0o755);
        assert_eq!(fs::read(f.target.join("pkg/local/.git/HEAD")).unwrap(), b"target-owned\n");
        assert_eq!(fs::read(f.target.join(".env")).unwrap(), b"unignored local secret\n");
        assert_eq!(fs::metadata(&f.target).unwrap().ino(), root_inode);
        assert_eq!(fs::metadata(f.target.join(".jj")).unwrap().ino(), jj_inode);
        fs::write(f.target.join("pkg/cache/nested/tool"), "private edit\n").unwrap();
        assert_eq!(fs::read(f.source.join("pkg/cache/nested/tool")).unwrap(), b"#!/bin/sh\n");
    }

    #[test]
    fn clean_payload_has_zero_writes_and_does_not_claim_mode_or_byte_equality() {
        let f = Fixture::new();
        if !f.cow() {
            return;
        }
        fs::create_dir_all(f.source.join("cache/readonly")).unwrap();
        fs::write(f.source.join("cache/readonly/tool"), "tool\n").unwrap();
        fs::set_permissions(f.source.join("cache/readonly"), fs::Permissions::from_mode(0o555)).unwrap();
        let visitor = f.visitor(&["cache"]);
        let transfer = applier(&f.source, &f.target, &visitor).unwrap();
        transfer.run().unwrap();
        assert_eq!(fs::metadata(f.target.join("cache/readonly")).unwrap().permissions().mode() & 0o7777, 0o555);
        fs::set_permissions(f.target.join("cache/readonly/tool"), fs::Permissions::from_mode(0o600)).unwrap();
        let done = Execution::new(true).unwrap().run(&transfer).unwrap();
        assert_eq!(done.completed, diff_dirs_sync::Progress::default());
        assert!(done.policy.satisfied);
        assert_eq!(fs::metadata(f.target.join("cache/readonly/tool")).unwrap().permissions().mode() & 0o7777, 0o600);
        // Clearing uses the shared remover, including read-only directories, and
        // never removes or relaxes the retained native metadata container.
        let jj_inode = fs::metadata(f.target.join(".jj")).unwrap().ino();
        diff_dirs_sync::clear_with_visitor(&f.target, WorkspaceClear).unwrap();
        assert!(!f.target.join("cache").exists());
        assert_eq!(fs::metadata(f.target.join(".jj")).unwrap().ino(), jj_inode);
        fs::set_permissions(f.source.join("cache/readonly"), fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn empty_compatibility_selection_and_presence_do_not_broaden_authority() {
        let f = Fixture::new();
        fs::create_dir_all(f.source.join("cache")).unwrap();
        fs::write(f.source.join("cache/file"), "source\n").unwrap();
        fs::write(f.target.join("ordinary"), "local\n").unwrap();
        let visitor = f.visitor(&[]);
        let done = applier(&f.source, &f.target, &visitor).unwrap().run().unwrap();
        assert_eq!(done.completed, diff_dirs_sync::Progress::default());
        assert!(!f.target.join("cache").exists());
        assert_eq!(fs::read(f.target.join("ordinary")).unwrap(), b"local\n");
    }

    #[test]
    fn tracked_leaf_overlap_gate_checks_both_prefix_directions_but_not_shared_directories() {
        let roots = [PathBuf::from("pkg/cache")];
        for leaf in ["pkg", "pkg/cache", "pkg/cache/file"] {
            assert!(Auxiliary::legacy(&roots, [Path::new(leaf)], (0, 0)).is_err(), "{leaf}");
        }
        Auxiliary::legacy(&roots, [Path::new("pkg/source.rs"), Path::new("pkg/cache-other/file")], (0, 0)).unwrap();
    }

    #[test]
    fn joint_preview_and_execution_refuse_routing_collision_and_create_only_obstruction() {
        let f = Fixture::new();
        fs::create_dir_all(f.source.join("pkg/cache")).unwrap();
        fs::write(f.source.join("pkg/cache/file"), "new\n").unwrap();
        fs::write(f.target.join("pkg"), "ordinary local leaf\n").unwrap();
        let visitor = f.visitor(&["pkg/cache"]);
        let transfer = applier(&f.source, &f.target, &visitor).unwrap();
        assert!(transfer.preview().unwrap_err().to_string().contains("routing ancestor"));
        assert_eq!(transfer.run().unwrap_err().completed, diff_dirs_sync::Progress::default());
        assert_eq!(fs::read(f.target.join("pkg")).unwrap(), b"ordinary local leaf\n");
        fs::remove_file(f.target.join("pkg")).unwrap();
        fs::create_dir_all(f.target.join("pkg/cache")).unwrap();
        let visitor = visitor.new_targets_only();
        let transfer = applier(&f.source, &f.target, &visitor).unwrap();
        assert!(transfer.preview().unwrap_err().to_string().contains("obstructed"));
        assert!(transfer.run().is_err());
        assert!(!f.target.join("pkg/cache/file").exists());
    }

    #[test]
    fn staged_collision_refusal_keeps_old_link_and_preview_does_not_write() {
        let f = Fixture::new();
        fs::create_dir_all(f.source.join("cache/nested/.git")).unwrap();
        symlink(".jj", f.target.join("cache")).unwrap();
        let visitor = f.visitor(&["cache"]);
        let transfer = applier(&f.source, &f.target, &visitor).unwrap();
        assert!(transfer.preview().unwrap_err().to_string().contains("administrative"));
        assert_eq!(transfer.run().unwrap_err().completed, diff_dirs_sync::Progress::default());
        assert_eq!(fs::read_link(f.target.join("cache")).unwrap(), Path::new(".jj"));
        fs::remove_dir_all(f.source.join("cache/nested/.git")).unwrap();
        let _socket = UnixListener::bind(f.source.join("cache/nested/socket")).unwrap();
        assert!(transfer.preview().is_err());
        assert!(transfer.run().is_err());
        assert_eq!(fs::read_link(f.target.join("cache")).unwrap(), Path::new(".jj"));
    }

    #[test]
    fn shared_seed_refuses_existing_leaf_and_required_cross_device_clone_without_fallback() {
        let f = Fixture::new();
        fs::write(f.source.join("file"), "donor\n").unwrap();
        fs::write(f.target.join("file"), "local\n").unwrap();
        assert!(crate::clone_files(&f.source, &["file".into()], &f.target).is_err());
        assert_eq!(fs::read(f.target.join("file")).unwrap(), b"local\n");
        // This environment's approved tmpfs supplies a real clone failure, not
        // fault injection or an ordinary-copy emulation. Other hosts may lack it.
        if !Path::new("/tmp/opencode").is_dir() {
            return;
        }
        let external = tempfile::tempdir_in("/tmp/opencode").unwrap();
        if fs::metadata(external.path()).unwrap().dev() == fs::metadata(&f.source).unwrap().dev() {
            return;
        }
        assert!(crate::clone_files(&f.source, &["file".into()], external.path()).is_err());
        assert!(!external.path().join("file").exists());
        assert_eq!(fs::read_dir(external.path()).unwrap().count(), 0);
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
