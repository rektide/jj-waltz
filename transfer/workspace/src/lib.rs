//! JW-specific transfer mechanisms over CoW and diff-dirs.
//!
//! The crate root owns copy-on-write cloning of checkout files into a new
//! workspace. A clone shares storage with its source until either copy changes:
//! APFS `clonefile(2)` on macOS and `FICLONE` reflinks on Linux filesystems such
//! as Btrfs and XFS. JJ still owns the checkout. The workspace adapter creates
//! the workspace with empty sparse patterns, clones the source's tracked files
//! into it, and lets JJ reconcile them against the creation base.
//!
//! Modules: `transfer` owns the root-bound early/late visitor and shared command
//! pool over diffdirs; listed tracked prefills use diffdirs' strict create-only
//! installer. `ignored` owns the ignored-payload path policy and pattern
//! selection, receiving resolved pattern layers rather than loading application
//! global configuration. `frozen` owns immutable ignore matching and the legacy
//! oracle adapter; `streaming` owns source-qualified production cursor policy;
//! `tracked` owns native tracked-leaf queries and path-safety primitives;
//! `topology` owns the Linux mount guard. The replacement refill selects required
//! CoW with a small root-level probe, then installs native leaves strictly. The
//! legacy CLI route still performs required clones without that probe. Neither
//! route downgrades a failed required clone. Lifecycle validation/cleanup
//! additionally uses `probe_between`, `validate_tree`, `validate_same_mount` and
//! `remove_path`. Standalone validation detects kernel mount IDs, so same-device
//! bind mounts are rejected too.

use anyhow::Result;
use std::path::{Path, PathBuf};

#[cfg(unix)]
pub mod frozen;
pub mod ignored;
#[cfg(unix)]
pub mod streaming;
#[cfg(unix)]
pub mod topology;
#[cfg(unix)]
pub mod tracked;
#[cfg(unix)]
pub mod transfer;

/// Explain why files under `source` cannot be cloned into directories created in
/// `destination_parent`, or return `None` when cloning is available.
///
/// This probes the destination filesystem without touching either checkout, so
/// callers can choose a full checkout before any workspace mutation.
pub fn unsupported_reason(source: &Path, destination_parent: &Path) -> Option<String> {
    imp::unsupported_reason(source, destination_parent)
}

/// Clone workspace-relative `files` from `source` into the existing `destination`.
///
/// Symlinks are recreated. Files missing from the source are skipped, leaving JJ to
/// check them out, as are paths that are no longer regular files or symlinks.
/// Permission bits are restored explicitly instead of relying on reflink
/// internals, and modification times are preserved so JJ can trust the copy.
pub fn clone_files(source: &Path, files: &[PathBuf], destination: &Path) -> Result<()> {
    imp::clone_files(source, files, destination)
}

/// Probe that real reflinks work from `source` to `destination`.
///
/// Writes a private temporary file on the source filesystem and reflinks it into
/// a private temporary directory under `destination`, proving the exact
/// cross-filesystem capability reseeding will use; the destination-only probe in
/// [`unsupported_reason`] cannot distinguish shared storage from a same-directory
/// accident. Existing content is never touched, and there is no fallback: an
/// error here means cloning is unavailable. Call this before any destructive
/// mutation.
pub fn probe_between(source: &Path, destination: &Path) -> Result<()> {
    imp::probe_between(source, destination)
}

/// Check that the tree at `root` is safe to clone or clear.
///
/// Inspection uses only `symlink_metadata`, never traversing a symlink. This
/// rejects a symlink or non-directory root, `.jj` and `.git` administrative
/// entries at any depth (nested repositories fail closed), special entries such
/// as sockets, FIFOs, and devices, and entries on a different mounted
/// filesystem than their parent directory, checked with [`validate_same_mount`]
/// at every node. Only descendants are checked: whether `root` itself is a
/// mount point is left to its caller, keeping a mounted source or target root
/// eligible.
pub fn validate_tree(root: &Path) -> Result<()> {
    imp::validate_tree(root)
}

/// One preflight traversal of a selected ignored payload root: the full
/// [`validate_tree`] checks plus a warning for every source-bound absolute
/// symlink found along the way, instead of a second dedicated walk.
#[cfg_attr(target_os = "linux", allow(dead_code))]
pub fn validate_payload_root(source: &Path, root: &Path, warnings: &mut Vec<String>) -> Result<()> {
    imp::validate_payload_root(source, root, warnings)
}

/// Check that `child` lives on the same mounted filesystem as `parent`.
///
/// Both paths are inspected without traversing symlinks, so a symlinked child
/// is compared by its own inode. On Linux the comparison extends device numbers
/// with kernel mount IDs from `statx(2)`, catching same-device bind mounts that
/// device numbers alone would miss; when mount topology cannot be verified the
/// check fails closed. macOS compares device numbers only, which is sufficient
/// there because bind mounts are a Linux feature.
pub fn validate_same_mount(parent: &Path, child: &Path) -> Result<()> {
    imp::validate_same_mount(parent, child)
}

/// Remove exactly the selected `path` and nothing else.
///
/// A missing path succeeds so callers can clear idempotently. A selected
/// symlink is unlinked without traversing it, and a selected special entry
/// (socket, FIFO, device) is rejected rather than deleted. A selected directory
/// is verified before anything is removed or relaxed: its final name must not
/// be administrative (`.jj` or `.git` are never deleted through this helper),
/// [`validate_same_mount`] must accept it against its parent so a mounted
/// selected leaf fails closed, and [`validate_tree`] must pass over its
/// contents, rejecting nested administrative entries, special files, and
/// mounted children. Only then are owner write and traverse bits added to the
/// validated directories in the subtree — never to symlinks, never outside the
/// selected path — so clones of read-only source directories can be replaced
/// without the caller fixing modes first. Content swapped in after validation
/// is not re-checked.
pub fn remove_path(path: &Path) -> Result<()> {
    imp::remove_path(path)
}

#[cfg(unix)]
mod imp {
    use anyhow::{Context, Result, bail, ensure};
    #[cfg(target_os = "linux")]
    use std::ffi::CString;
    use std::ffi::OsStr;
    use std::fs;
    use std::io::ErrorKind;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::path::{Path, PathBuf};

    pub(super) fn unsupported_reason(source: &Path, destination_parent: &Path) -> Option<String> {
        let device = |path: &Path| {
            fs::metadata(path)
                .map(|metadata| metadata.dev())
                .map_err(|error| format!("cannot inspect {}: {error}", path.display()))
        };
        match (device(source), device(destination_parent)) {
            (Ok(source_device), Ok(destination_device)) if source_device != destination_device => {
                return Some(format!(
                    "{} and {} are on different filesystems",
                    source.display(),
                    destination_parent.display()
                ));
            }
            (Err(reason), _) | (_, Err(reason)) => return Some(reason),
            _ => {}
        }

        // Exclusive creation and mode 0700 keep both files private. TempDir only
        // cleans up the directory it created, never a caller's existing paths.
        let directory = match tempfile::Builder::new().prefix(".jw-cow-probe-").tempdir_in(destination_parent) {
            Ok(directory) => directory,
            Err(error) => return Some(format!("cannot create private clone probe: {error}")),
        };
        let probe = directory.path().join("source");
        let probe_clone = directory.path().join("clone");
        let result = fs::write(&probe, b"probe").and_then(|()| reflink_copy::reflink(&probe, &probe_clone));
        result
            .err()
            .map(|error| format!("the filesystem at {} cannot clone files: {error}", destination_parent.display()))
    }

    pub(super) fn clone_files(source: &Path, files: &[PathBuf], destination: &Path) -> Result<()> {
        // Preserve the creation adapter's missing/non-leaf skip contract, but
        // delegate all installation, collision handling and metadata restoration
        // to the same create-only kernel used by other callers. No directories
        // are enumerated and no unlisted file can be installed.
        let mut leaves = Vec::with_capacity(files.len());
        for relative in files {
            crate::tracked::relative(relative)?;
            let from = source.join(relative);
            let metadata = match fs::symlink_metadata(&from) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == ErrorKind::NotFound => continue,
                Err(error) => {
                    return Err(error).with_context(|| format!("failed to inspect {}", from.display()));
                }
            };
            if metadata.is_file() || metadata.is_symlink() {
                leaves.push(relative);
            }
        }
        diff_dirs_sync::seed_leaves(source, destination, leaves, diff_dirs_sync::CopyPolicy::ReflinkRequired, diff_dirs::NoCheck)?;
        Ok(())
    }

    pub(super) fn probe_between(source: &Path, destination: &Path) -> Result<()> {
        // The device comparison gives a precise early message; the reflink
        // attempt below remains the actual proof.
        let device = |path: &Path| {
            fs::metadata(path)
                .map(|metadata| metadata.dev())
                .with_context(|| format!("cannot inspect {}", path.display()))
        };
        let source_device = device(source)?;
        let destination_device = device(destination)?;
        ensure!(
            source_device == destination_device,
            "{} and {} are on different filesystems; cloning cannot cross them",
            source.display(),
            destination.display()
        );

        // Exclusive creation and mode 0700 keep the probes private, and TempDir
        // only cleans up the directories it created.
        let source_directory = tempfile::Builder::new()
            .prefix(".jw-cow-probe-")
            .tempdir_in(source)
            .with_context(|| format!("cannot create private clone probe in {}", source.display()))?;
        let destination_directory = tempfile::Builder::new()
            .prefix(".jw-cow-probe-")
            .tempdir_in(destination)
            .with_context(|| format!("cannot create private clone probe in {}", destination.display()))?;
        let probe = source_directory.path().join("source");
        let probe_clone = destination_directory.path().join("clone");
        fs::write(&probe, b"probe").with_context(|| format!("cannot write {}", probe.display()))?;
        reflink_copy::reflink(&probe, &probe_clone).with_context(|| {
            format!(
                "the filesystem at {} cannot clone files from {}: real reflinks are required",
                destination.display(),
                source.display()
            )
        })?;
        Ok(())
    }

    pub(super) fn validate_tree(root: &Path) -> Result<()> {
        let metadata = fs::symlink_metadata(root).with_context(|| format!("failed to inspect {}", root.display()))?;
        ensure!(!metadata.file_type().is_symlink(), "refusing {}: it is a symbolic link", root.display());
        ensure!(metadata.is_dir(), "refusing {}: it is not a directory", root.display());
        validate_directory(root, None, false)
    }

    #[cfg_attr(target_os = "linux", allow(dead_code))]
    pub(super) fn validate_payload_root(source: &Path, root: &Path, warnings: &mut Vec<String>) -> Result<()> {
        let path = source.join(root);
        let metadata = fs::symlink_metadata(&path).with_context(|| format!("failed to inspect {}", path.display()))?;
        ensure!(!metadata.file_type().is_symlink(), "refusing {}: it is a symbolic link", path.display());
        ensure!(metadata.is_dir(), "refusing {}: it is not a directory", path.display());
        validate_directory(&path, Some((source, warnings)), false)
    }

    fn validate_directory(directory: &Path, mut links: Option<(&Path, &mut Vec<String>)>, topology_checked: bool) -> Result<()> {
        for entry in fs::read_dir(directory).with_context(|| format!("failed to read {}", directory.display()))? {
            let entry = entry.with_context(|| format!("failed to read {}", directory.display()))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).with_context(|| format!("failed to inspect {}", path.display()))?;
            ensure!(!is_admin_name(&entry.file_name()), "refusing {}: it is an administrative entry", path.display());
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                // Cloned as a link; its target is never inspected, but an
                // absolute one is reported as not self-contained.
                if let Some((base, warnings)) = links.as_mut() {
                    let target = fs::read_link(&path).with_context(|| format!("failed to read {}", path.display()))?;
                    if target.is_absolute() {
                        warnings.push(format!(
                            "absolute dependency symlink {} -> {} (not self-contained)",
                            path.strip_prefix(*base).expect("walk starts at base").display(),
                            target.display()
                        ));
                    }
                }
                continue;
            }
            if !topology_checked {
                validate_same_mount(directory, &path)?;
            }
            if file_type.is_dir() {
                validate_directory(&path, links.as_mut().map(|(base, warnings)| (*base, &mut **warnings)), topology_checked)?;
            } else {
                ensure!(file_type.is_file(), "refusing {}: it is an unsupported special file", path.display());
            }
        }
        Ok(())
    }

    pub(super) fn validate_same_mount(parent: &Path, child: &Path) -> Result<()> {
        let parent_metadata = fs::symlink_metadata(parent).with_context(|| format!("failed to inspect {}", parent.display()))?;
        let child_metadata = fs::symlink_metadata(child).with_context(|| format!("failed to inspect {}", child.display()))?;
        ensure!(
            parent_metadata.dev() == child_metadata.dev(),
            "refusing {}: it is a mounted filesystem boundary",
            child.display()
        );
        #[cfg(target_os = "linux")]
        {
            // Device numbers miss same-device bind mounts; the kernel mount ID
            // does not. Failure to verify the topology fails closed.
            ensure!(
                linux_mount_id(parent)? == linux_mount_id(child)?,
                "refusing {}: it is a mounted filesystem boundary",
                child.display()
            );
        }
        Ok(())
    }

    /// Kernel mount ID of the filesystem holding `path`, never traversing
    /// symlinks. Requires Linux 5.8+ (`statx` with `STATX_MNT_ID`).
    #[cfg(target_os = "linux")]
    pub(super) fn linux_mount_id(path: &Path) -> Result<u64> {
        let encoded = CString::new(path.as_os_str().as_encoded_bytes()).with_context(|| format!("cannot inspect {}", path.display()))?;
        let mut buffer: libc::statx = unsafe { std::mem::zeroed() };
        let status = unsafe {
            libc::statx(
                libc::AT_FDCWD,
                encoded.as_ptr(),
                libc::AT_SYMLINK_NOFOLLOW | libc::AT_NO_AUTOMOUNT,
                libc::STATX_MNT_ID,
                &mut buffer,
            )
        };
        if status != 0 {
            return Err(std::io::Error::last_os_error()).with_context(|| format!("cannot verify mount topology at {}", path.display()));
        }
        if buffer.stx_mask & libc::STATX_MNT_ID == 0 {
            bail!("cannot verify mount topology at {}: the kernel does not report mount IDs", path.display());
        }
        Ok(buffer.stx_mnt_id)
    }

    pub(super) fn remove_path(path: &Path) -> Result<()> {
        remove_path_with_topology(path, false)
    }

    pub(super) fn remove_path_with_topology(path: &Path, topology_checked: bool) -> Result<()> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(error).with_context(|| format!("failed to inspect {}", path.display()));
            }
        };
        let name = path
            .file_name()
            .with_context(|| format!("refusing to remove {}: it has no final component", path.display()))?;
        ensure!(!is_admin_name(name), "refusing to remove {}: it is an administrative entry", path.display());
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            // Unlink exactly what was named; a symlink's target is never touched.
            return fs::remove_file(path).with_context(|| format!("failed to remove {}", path.display()));
        }
        if !file_type.is_dir() {
            ensure!(file_type.is_file(), "refusing to remove {}: it is an unsupported special file", path.display());
            return fs::remove_file(path).with_context(|| format!("failed to remove {}", path.display()));
        }
        // Reject a mounted selected leaf: checking descendants against this
        // directory alone cannot see a mount rooted at the leaf itself.
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .with_context(|| format!("cannot determine the parent of {}", path.display()))?;
        if !topology_checked {
            validate_same_mount(parent, path)?;
        }
        // validate_tree's messages already name the path and the reason.
        if topology_checked {
            validate_directory(path, None, true)?;
        } else {
            validate_tree(path)?;
        }
        ensure_removable(path)?;
        fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))
    }

    /// Grant owner write and traverse bits to every directory in the
    /// already-validated `directory` tree. Cloned read-only source directories
    /// must not block their authorized replacement, and the bits are added only
    /// after validation so a rejected tree is never modified.
    fn ensure_removable(directory: &Path) -> Result<()> {
        let metadata = fs::symlink_metadata(directory).with_context(|| format!("failed to inspect {}", directory.display()))?;
        let mode = metadata.permissions().mode() & 0o7777;
        if mode & 0o300 != 0o300 {
            fs::set_permissions(directory, fs::Permissions::from_mode(mode | 0o300))
                .with_context(|| format!("failed to make {} removable", directory.display()))?;
        }
        for entry in fs::read_dir(directory).with_context(|| format!("failed to read {}", directory.display()))? {
            let path = entry.with_context(|| format!("failed to read {}", directory.display()))?.path();
            let metadata = fs::symlink_metadata(&path).with_context(|| format!("failed to inspect {}", path.display()))?;
            if metadata.file_type().is_dir() {
                ensure_removable(&path)?;
            }
        }
        Ok(())
    }

    /// Administrative entries belong to JJ or Git and are never cloned or
    /// deleted by these helpers.
    fn is_admin_name(name: &OsStr) -> bool {
        matches!(name.to_str(), Some(".jj") | Some(".git"))
    }
}

#[cfg(not(unix))]
mod imp {
    use anyhow::{Result, bail};
    use std::path::Path;

    pub(super) fn unsupported_reason(_source: &Path, _destination_parent: &Path) -> Option<String> {
        Some("copy-on-write cloning is supported on macOS and Linux only".to_owned())
    }

    pub(super) fn clone_files(_source: &Path, _files: &[std::path::PathBuf], _destination: &Path) -> Result<()> {
        bail!("copy-on-write cloning is supported on macOS and Linux only")
    }

    pub(super) fn probe_between(_source: &Path, _destination: &Path) -> Result<()> {
        bail!("copy-on-write cloning is supported on macOS and Linux only")
    }

    pub(super) fn validate_tree(_root: &Path) -> Result<()> {
        bail!("tree validation is supported on macOS and Linux only")
    }

    pub(super) fn validate_same_mount(_parent: &Path, _child: &Path) -> Result<()> {
        bail!("mount boundary validation is supported on macOS and Linux only")
    }

    pub(super) fn validate_payload_root(_source: &Path, _root: &Path, _warnings: &mut Vec<String>) -> Result<()> {
        bail!("tree validation is supported on macOS and Linux only")
    }

    pub(super) fn remove_path(_path: &Path) -> Result<()> {
        bail!("safe removal is supported on macOS and Linux only")
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use std::ffi::CString;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::os::unix::net::UnixListener;

    #[test]
    fn clones_listed_files_and_symlinks_only() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        if let Some(reason) = unsupported_reason(temp.path(), temp.path()) {
            assert!(
                std::env::var_os("JW_TEST_REQUIRE_COW").is_none_or(|value| value.is_empty()),
                "real cloning required: {reason}"
            );
            eprintln!("skipping clone test: {reason}");
            return;
        }

        fs::create_dir_all(source.join("nested/deeper")).unwrap();
        fs::create_dir_all(destination.join(".jj")).unwrap();
        fs::write(source.join("nested/deeper/tool.sh"), "#!/bin/sh\n").unwrap();
        fs::set_permissions(source.join("nested/deeper/tool.sh"), fs::Permissions::from_mode(0o755)).unwrap();
        symlink("deeper/tool.sh", source.join("nested/link")).unwrap();
        fs::write(source.join(".env"), "SECRET=1\n").unwrap();
        let files = ["nested/deeper/tool.sh", "nested/link", "deleted.txt", "nested"].map(PathBuf::from);

        clone_files(&source, &files, &destination).expect("clone files");

        let tool = destination.join("nested/deeper/tool.sh");
        assert_eq!(fs::read_to_string(&tool).unwrap(), "#!/bin/sh\n");
        assert_eq!(fs::metadata(&tool).unwrap().permissions().mode() & 0o777, 0o755);
        assert_eq!(fs::read_link(destination.join("nested/link")).unwrap(), Path::new("deeper/tool.sh"));
        assert!(!destination.join(".env").exists());
        assert!(!destination.join("deleted.txt").exists());
        // Sharing storage must never couple later edits or share an inode.
        fs::write(&tool, "destination edit\n").unwrap();
        assert_eq!(fs::read_to_string(source.join("nested/deeper/tool.sh")).unwrap(), "#!/bin/sh\n");
        fs::write(source.join("nested/deeper/tool.sh"), "source edit\n").unwrap();
        assert_eq!(fs::read_to_string(&tool).unwrap(), "destination edit\n");
    }

    #[test]
    fn probe_preserves_existing_files_symlinks_and_directories() {
        let temp = tempfile::tempdir().unwrap();
        let victim = temp.path().join("victim");
        fs::write(&victim, "untouched").unwrap();
        let legacy = temp.path().join(format!(".jw-cow-probe-{}", std::process::id()));
        let legacy_clone = legacy.with_extension("clone");
        symlink(&victim, &legacy).unwrap();
        fs::write(&legacy_clone, "caller-owned").unwrap();
        let other = temp.path().join(".jw-cow-probe-existing");
        fs::create_dir(&other).unwrap();
        fs::write(other.join("source"), "also caller-owned").unwrap();
        let before = fs::read_dir(temp.path()).unwrap().count();
        unsupported_reason(temp.path(), temp.path());
        assert_eq!(fs::read_to_string(&victim).unwrap(), "untouched");
        assert_eq!(fs::read_link(&legacy).unwrap(), victim);
        assert_eq!(fs::read_to_string(&legacy_clone).unwrap(), "caller-owned");
        assert_eq!(fs::read_to_string(other.join("source")).unwrap(), "also caller-owned");
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), before);
    }

    #[test]
    fn probe_between_proves_reflinks_without_leaving_residue() {
        let temp = tempfile::tempdir().unwrap();
        if let Some(reason) = unsupported_reason(temp.path(), temp.path()) {
            assert!(
                std::env::var_os("JW_TEST_REQUIRE_COW").is_none_or(|value| value.is_empty()),
                "real cloning required: {reason}"
            );
            eprintln!("skipping reflink test: {reason}");
            return;
        }
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("kept.txt"), "kept\n").unwrap();
        fs::create_dir(&destination).unwrap();
        fs::write(destination.join("kept.txt"), "kept\n").unwrap();
        let source_entries = fs::read_dir(&source).unwrap().count();
        let destination_entries = fs::read_dir(&destination).unwrap().count();

        probe_between(&source, &destination).expect("probe between");

        assert_eq!(fs::read_dir(&source).unwrap().count(), source_entries);
        assert_eq!(fs::read_dir(&destination).unwrap().count(), destination_entries);
        assert_eq!(fs::read_to_string(source.join("kept.txt")).unwrap(), "kept\n");
        assert_eq!(fs::read_to_string(destination.join("kept.txt")).unwrap(), "kept\n");
    }

    #[test]
    fn probe_between_rejects_missing_paths_and_file_destinations() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        fs::create_dir(&source).unwrap();
        let file = temp.path().join("file");
        fs::write(&file, "content\n").unwrap();

        assert!(probe_between(&temp.path().join("missing"), temp.path()).is_err());
        assert!(probe_between(&source, &temp.path().join("missing")).is_err());
        assert!(probe_between(&file, temp.path()).is_err());
        assert!(probe_between(&source, &file).is_err());
    }

    #[test]
    fn validate_tree_accepts_files_directories_and_symlinks() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        fs::create_dir_all(root.join("nested/deeper")).unwrap();
        fs::write(root.join("nested/deeper/file.txt"), "content\n").unwrap();
        fs::set_permissions(root.join("nested/deeper/file.txt"), fs::Permissions::from_mode(0o755)).unwrap();
        symlink("deeper", root.join("nested/link")).unwrap();
        symlink("../nested/deeper/file.txt", root.join("plain-link")).unwrap();

        validate_tree(&root).expect("plain tree");
    }

    #[test]
    fn validate_tree_rejects_unsafe_roots() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("file");
        fs::write(&file, "content\n").unwrap();
        let target = temp.path().join("target");
        fs::create_dir(&target).unwrap();
        let link = temp.path().join("link");
        symlink(&target, &link).unwrap();

        let error = validate_tree(&temp.path().join("missing")).unwrap_err();
        assert!(error.to_string().contains("failed to inspect"), "{error:#}");
        let error = validate_tree(&file).unwrap_err();
        assert!(error.to_string().contains("not a directory"), "{error:#}");
        let error = validate_tree(&link).unwrap_err();
        assert!(error.to_string().contains("symbolic link"), "{error:#}");
    }

    #[test]
    fn validate_tree_rejects_administrative_entries_at_any_depth() {
        let temp = tempfile::tempdir().unwrap();

        let jj_directory = temp.path().join("jj-directory");
        fs::create_dir_all(jj_directory.join(".jj")).unwrap();
        let error = validate_tree(&jj_directory).unwrap_err();
        assert!(error.to_string().contains("administrative"), "{error:#}");

        let git_file = temp.path().join("git-file");
        fs::create_dir(&git_file).unwrap();
        fs::write(git_file.join(".git"), "gitdir: elsewhere\n").unwrap();
        let error = validate_tree(&git_file).unwrap_err();
        assert!(error.to_string().contains("administrative"), "{error:#}");

        let deep = temp.path().join("deep");
        fs::create_dir_all(deep.join("a/b/.git")).unwrap();
        let error = validate_tree(&deep).unwrap_err();
        assert!(error.to_string().contains("administrative"), "{error:#}");

        let linked = temp.path().join("linked");
        fs::create_dir(&linked).unwrap();
        symlink("..", linked.join(".jj")).unwrap();
        let error = validate_tree(&linked).unwrap_err();
        assert!(error.to_string().contains("administrative"), "{error:#}");
    }

    #[test]
    fn validate_tree_rejects_special_entries() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("keep.txt"), "kept\n").unwrap();
        UnixListener::bind(root.join("socket")).unwrap();

        let error = validate_tree(&root).unwrap_err();
        assert!(error.to_string().contains("special"), "{error:#}");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn validate_same_mount_accepts_children_and_rejects_distinct_mounts() {
        let temp = tempfile::tempdir().unwrap();
        let parent = temp.path().join("parent");
        fs::create_dir(&parent).unwrap();
        let file = parent.join("file");
        fs::write(&file, "content\n").unwrap();
        let directory = parent.join("directory");
        fs::create_dir(&directory).unwrap();
        let link = parent.join("link");
        symlink("file", &link).unwrap();

        // A symlinked child is compared by its own inode, never traversed.
        validate_same_mount(&parent, &parent).expect("identical paths");
        validate_same_mount(&parent, &file).expect("file child");
        validate_same_mount(&parent, &directory).expect("directory child");
        validate_same_mount(&parent, &link).expect("symlink child");

        // Kernel mount IDs resolve, agree within one filesystem, and separate
        // distinct filesystems such as procfs.
        assert!(imp::linux_mount_id(&parent).unwrap() > 0);
        assert_eq!(imp::linux_mount_id(&parent).unwrap(), imp::linux_mount_id(&file).unwrap());
        assert_ne!(imp::linux_mount_id(&parent).unwrap(), imp::linux_mount_id(Path::new("/proc")).unwrap());

        // A child on another mounted filesystem fails closed, whichever stage
        // of the comparison rejects it.
        let error = validate_same_mount(&parent, Path::new("/proc")).unwrap_err();
        assert!(error.to_string().contains("mounted filesystem boundary"), "{error:#}");

        // Nested trees pass: every node is compared only against its parent,
        // so the tree's own root being a mount stays acceptable.
        fs::create_dir(directory.join("nested")).unwrap();
        validate_tree(&directory).expect("nested tree");
    }

    #[test]
    fn remove_path_rejects_special_leaf_entries() {
        let temp = tempfile::tempdir().unwrap();
        let socket = temp.path().join("socket");
        UnixListener::bind(&socket).unwrap();
        let error = remove_path(&socket).unwrap_err();
        assert!(error.to_string().contains("special"), "{error:#}");
        assert!(socket.exists());

        #[cfg(target_os = "linux")]
        {
            let fifo = temp.path().join("fifo");
            let encoded = CString::new(fifo.as_os_str().as_encoded_bytes()).unwrap();
            assert_eq!(unsafe { libc::mkfifo(encoded.as_ptr(), 0o644) }, 0);
            let error = remove_path(&fifo).unwrap_err();
            assert!(error.to_string().contains("special"), "{error:#}");
            assert!(fifo.exists());
        }

        // Regular leaves are still removed exactly.
        let file = temp.path().join("file");
        fs::write(&file, "content\n").unwrap();
        remove_path(&file).expect("remove file");
        assert!(!file.exists());
    }

    #[test]
    fn remove_path_removes_exactly_the_selected_entry() {
        let temp = tempfile::tempdir().unwrap();
        assert!(remove_path(&temp.path().join("missing")).is_ok());

        let file = temp.path().join("file");
        fs::write(&file, "content\n").unwrap();
        remove_path(&file).expect("remove file");
        assert!(!file.exists());

        // A selected symlink is unlinked; its target is never traversed.
        let target = temp.path().join("target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("inner.txt"), "kept\n").unwrap();
        let parent = temp.path().join("parent");
        fs::create_dir(&parent).unwrap();
        let link = parent.join("link");
        symlink(&target, &link).unwrap();
        remove_path(&link).expect("remove symlink");
        assert!(!link.exists());
        assert_eq!(fs::read_to_string(target.join("inner.txt")).unwrap(), "kept\n");

        // A real tree is removed entirely while its parent survives.
        let tree = temp.path().join("tree");
        fs::create_dir_all(tree.join("nested/deeper")).unwrap();
        fs::write(tree.join("nested/deeper/file.txt"), "content\n").unwrap();
        symlink("deeper/file.txt", tree.join("nested/relative-link")).unwrap();
        remove_path(&tree).expect("remove tree");
        assert!(!tree.exists());
        assert!(temp.path().is_dir());
    }

    #[test]
    fn remove_path_fails_closed_before_removing_anything() {
        let temp = tempfile::tempdir().unwrap();

        // A nested repository aborts the removal before anything is deleted.
        let tree = temp.path().join("nested-repo");
        fs::create_dir_all(tree.join("sub/.jj")).unwrap();
        fs::write(tree.join("keep.txt"), "kept\n").unwrap();
        fs::write(tree.join("sub/file.txt"), "kept\n").unwrap();
        fs::write(tree.join("sub/.jj/marker"), "kept\n").unwrap();
        let error = remove_path(&tree).unwrap_err();
        assert!(error.to_string().contains("administrative"), "{error:#}");
        assert_eq!(fs::read_to_string(tree.join("keep.txt")).unwrap(), "kept\n");
        assert_eq!(fs::read_to_string(tree.join("sub/file.txt")).unwrap(), "kept\n");
        assert_eq!(fs::read_to_string(tree.join("sub/.jj/marker")).unwrap(), "kept\n");

        // A special entry aborts the removal before anything is deleted.
        let special = temp.path().join("special");
        fs::create_dir(&special).unwrap();
        fs::write(special.join("keep.txt"), "kept\n").unwrap();
        UnixListener::bind(special.join("socket")).unwrap();
        let error = remove_path(&special).unwrap_err();
        assert!(error.to_string().contains("special"), "{error:#}");
        assert_eq!(fs::read_to_string(special.join("keep.txt")).unwrap(), "kept\n");

        // Administrative names are never selected for removal, even directly.
        let administrative = temp.path().join("administrative");
        fs::create_dir_all(administrative.join(".jj")).unwrap();
        fs::write(administrative.join(".jj/marker"), "kept\n").unwrap();
        let error = remove_path(&administrative.join(".jj")).unwrap_err();
        assert!(error.to_string().contains("administrative"), "{error:#}");
        assert_eq!(fs::read_to_string(administrative.join(".jj/marker")).unwrap(), "kept\n");
        fs::write(administrative.join(".git"), "gitdir: elsewhere\n").unwrap();
        let error = remove_path(&administrative.join(".git")).unwrap_err();
        assert!(error.to_string().contains("administrative"), "{error:#}");
        assert!(administrative.join(".git").exists());
    }
}
