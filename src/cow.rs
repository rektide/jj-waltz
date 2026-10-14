//! Copy-on-write cloning of checkout files into a new workspace.
//!
//! A clone shares storage with its source until either copy changes: APFS
//! `clonefile(2)` on macOS and `FICLONE` reflinks on Linux filesystems such as
//! Btrfs and XFS. JJ still owns the checkout. The workspace adapter creates the
//! workspace with empty sparse patterns, clones the source's tracked files into
//! it, and lets JJ reconcile them against the creation base.
//!
//! The reseed helpers (`probe_between`, `validate_tree`, `clone_tree`, and
//! `remove_path`) are the filesystem adapter for in-place reseeding. They prove
//! cross-filesystem cloning up front, fail closed on administrative or special
//! content, and never fall back to ordinary copying.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Explain why files under `source` cannot be cloned into directories created in
/// `destination_parent`, or return `None` when cloning is available.
///
/// This probes the destination filesystem without touching either checkout, so
/// callers can choose a full checkout before any workspace mutation.
pub(crate) fn unsupported_reason(source: &Path, destination_parent: &Path) -> Option<String> {
    imp::unsupported_reason(source, destination_parent)
}

/// Clone workspace-relative `files` from `source` into the existing `destination`.
///
/// Symlinks are recreated. Files missing from the source are skipped, leaving JJ to
/// check them out, as are paths that are no longer regular files or symlinks.
/// Permission bits are restored explicitly instead of relying on reflink
/// internals, and modification times are preserved so JJ can trust the copy.
pub(crate) fn clone_files(source: &Path, files: &[PathBuf], destination: &Path) -> Result<()> {
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
pub(crate) fn probe_between(source: &Path, destination: &Path) -> Result<()> {
    imp::probe_between(source, destination)
}

/// Check that the tree at `root` is safe to clone or clear.
///
/// Inspection uses only `symlink_metadata`, never traversing a symlink. This
/// rejects a symlink or non-directory root, `.jj` and `.git` administrative
/// entries at any depth (nested repositories fail closed), special entries such
/// as sockets, FIFOs, and devices, and entries whose device differs from the
/// root's (mounted children). Same-device bind mounts are invisible to metadata
/// and are not rejected. Only descendants are checked: whether `root` itself is
/// a mount point is decided by its caller, such as [`remove_path`] comparing
/// against the parent device.
pub(crate) fn validate_tree(root: &Path) -> Result<()> {
    imp::validate_tree(root)
}

/// Strictly clone the tree at `source` to the not-yet-existing `destination`.
///
/// Validation runs first and fails closed on nested repositories, special
/// entries, mounted children, and a symlinked source root. Regular files are
/// reflinked with their permission bits and modification times restored,
/// symlinks are recreated as links and never traversed, and directories are
/// created fresh: an existing destination is an error, so callers clear the
/// selected path first. Directory modes are applied only after their contents
/// are copied, so restrictive source directories clone cleanly. A failed
/// reflink is an error; there is no ordinary-copy fallback. Administrative
/// names are skipped defensively even though validation already rejected them.
pub(crate) fn clone_tree(source: &Path, destination: &Path) -> Result<()> {
    imp::clone_tree(source, destination)
}

/// Remove exactly the selected `path` and nothing else.
///
/// A missing path succeeds so callers can clear idempotently. A selected
/// symlink is unlinked without traversing it. A selected directory is verified
/// before anything is removed: its final name must not be administrative
/// (`.jj` or `.git` are never deleted through this helper), its device must
/// match its parent's so a mounted selected leaf fails closed, and
/// [`validate_tree`] must accept its contents, rejecting nested administrative
/// entries, special files, and mounted children before the recursive removal.
/// Same-device bind mounts are invisible to metadata and are not rejected.
pub(crate) fn remove_path(path: &Path) -> Result<()> {
    imp::remove_path(path)
}

#[cfg(unix)]
mod imp {
    use anyhow::{Context, Result, bail, ensure};
    use std::collections::HashSet;
    use std::ffi::OsStr;
    use std::fs::{self, Metadata};
    use std::io::ErrorKind;
    use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
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
        let directory = match tempfile::Builder::new()
            .prefix(".jw-cow-probe-")
            .tempdir_in(destination_parent)
        {
            Ok(directory) => directory,
            Err(error) => return Some(format!("cannot create private clone probe: {error}")),
        };
        let probe = directory.path().join("source");
        let probe_clone = directory.path().join("clone");
        let result =
            fs::write(&probe, b"probe").and_then(|()| reflink_copy::reflink(&probe, &probe_clone));
        result.err().map(|error| {
            format!(
                "the filesystem at {} cannot clone files: {error}",
                destination_parent.display()
            )
        })
    }

    pub(super) fn clone_files(source: &Path, files: &[PathBuf], destination: &Path) -> Result<()> {
        let mut created_directories = HashSet::new();
        for relative in files {
            let from = source.join(relative);
            let to = destination.join(relative);
            let metadata = match fs::symlink_metadata(&from) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == ErrorKind::NotFound => continue,
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("failed to inspect {}", from.display()));
                }
            };
            let file_type = metadata.file_type();
            if !file_type.is_file() && !file_type.is_symlink() {
                continue;
            }
            if let Some(parent) = to.parent()
                && created_directories.insert(parent.to_path_buf())
            {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            if file_type.is_symlink() {
                let link = fs::read_link(&from)
                    .with_context(|| format!("failed to read {}", from.display()))?;
                symlink(&link, &to)
                    .with_context(|| format!("failed to create {}", to.display()))?;
            } else {
                reflink_copy::reflink(&from, &to).with_context(|| {
                    format!("failed to clone {} to {}", from.display(), to.display())
                })?;
                preserve_mode(&to, &metadata)?;
                preserve_modified_time(&to, &metadata)?;
            }
        }
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
            .with_context(|| {
                format!("cannot create private clone probe in {}", source.display())
            })?;
        let destination_directory = tempfile::Builder::new()
            .prefix(".jw-cow-probe-")
            .tempdir_in(destination)
            .with_context(|| {
                format!(
                    "cannot create private clone probe in {}",
                    destination.display()
                )
            })?;
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
        let metadata = fs::symlink_metadata(root)
            .with_context(|| format!("failed to inspect {}", root.display()))?;
        ensure!(
            !metadata.file_type().is_symlink(),
            "refusing {}: it is a symbolic link",
            root.display()
        );
        ensure!(
            metadata.is_dir(),
            "refusing {}: it is not a directory",
            root.display()
        );
        validate_directory(root, metadata.dev())
    }

    fn validate_directory(directory: &Path, device: u64) -> Result<()> {
        for entry in fs::read_dir(directory)
            .with_context(|| format!("failed to read {}", directory.display()))?
        {
            let entry = entry.with_context(|| format!("failed to read {}", directory.display()))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .with_context(|| format!("failed to inspect {}", path.display()))?;
            ensure!(
                !is_admin_name(&entry.file_name()),
                "refusing {}: it is an administrative entry",
                path.display()
            );
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                // Cloned as a link; its target is never inspected.
                continue;
            }
            ensure!(
                metadata.dev() == device,
                "refusing {}: it is a mounted filesystem boundary",
                path.display()
            );
            if file_type.is_dir() {
                validate_directory(&path, device)?;
            } else {
                ensure!(
                    file_type.is_file(),
                    "refusing {}: it is an unsupported special file",
                    path.display()
                );
            }
        }
        Ok(())
    }

    pub(super) fn clone_tree(source: &Path, destination: &Path) -> Result<()> {
        validate_tree(source)?;
        clone_directory(source, destination)
    }

    fn clone_directory(source_directory: &Path, destination_directory: &Path) -> Result<()> {
        let metadata = fs::symlink_metadata(source_directory)
            .with_context(|| format!("failed to inspect {}", source_directory.display()))?;
        fs::create_dir(destination_directory)
            .with_context(|| format!("failed to create {}", destination_directory.display()))?;
        for entry in fs::read_dir(source_directory)
            .with_context(|| format!("failed to read {}", source_directory.display()))?
        {
            let entry =
                entry.with_context(|| format!("failed to read {}", source_directory.display()))?;
            let name = entry.file_name();
            // validate_tree already rejected administrative entries; skipping
            // them here too keeps the clone safe if validation ever loosens.
            if is_admin_name(&name) {
                continue;
            }
            let from = entry.path();
            let to = destination_directory.join(&name);
            let metadata = fs::symlink_metadata(&from)
                .with_context(|| format!("failed to inspect {}", from.display()))?;
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                let link = fs::read_link(&from)
                    .with_context(|| format!("failed to read {}", from.display()))?;
                symlink(&link, &to)
                    .with_context(|| format!("failed to create {}", to.display()))?;
            } else if file_type.is_dir() {
                clone_directory(&from, &to)?;
            } else if file_type.is_file() {
                reflink_copy::reflink(&from, &to).with_context(|| {
                    format!("failed to clone {} to {}", from.display(), to.display())
                })?;
                preserve_mode(&to, &metadata)?;
                preserve_modified_time(&to, &metadata)?;
            } else {
                bail!(
                    "refusing {}: it is an unsupported special file",
                    from.display()
                );
            }
        }
        // Restrictive source modes are applied only after the directory is
        // fully populated, so cloning never needs to write through them.
        preserve_mode(destination_directory, &metadata)
    }

    pub(super) fn remove_path(path: &Path) -> Result<()> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(error).with_context(|| format!("failed to inspect {}", path.display()));
            }
        };
        let name = path.file_name().with_context(|| {
            format!(
                "refusing to remove {}: it has no final component",
                path.display()
            )
        })?;
        ensure!(
            !is_admin_name(name),
            "refusing to remove {}: it is an administrative entry",
            path.display()
        );
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            // Unlink exactly what was named; a symlink's target is never touched.
            return fs::remove_file(path)
                .with_context(|| format!("failed to remove {}", path.display()));
        }
        // Reject a mounted selected leaf: comparing descendants against this
        // directory alone cannot see a mount rooted at the leaf itself.
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .with_context(|| format!("cannot determine the parent of {}", path.display()))?;
        let parent_device = fs::symlink_metadata(parent)
            .map(|metadata| metadata.dev())
            .with_context(|| format!("cannot inspect {}", parent.display()))?;
        ensure!(
            metadata.dev() == parent_device,
            "refusing to remove {}: it is a mounted filesystem boundary",
            path.display()
        );
        // validate_tree's messages already name the path and the reason.
        validate_tree(path)?;
        fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))
    }

    /// Administrative entries belong to JJ or Git and are never cloned or
    /// deleted by these helpers.
    fn is_admin_name(name: &OsStr) -> bool {
        matches!(name.to_str(), Some(".jj") | Some(".git"))
    }

    /// Restore the source permission bits explicitly so the clone cannot depend
    /// on reflink internals or the process umask.
    fn preserve_mode(target: &Path, metadata: &Metadata) -> Result<()> {
        let mode = metadata.permissions().mode() & 0o7777;
        fs::set_permissions(target, fs::Permissions::from_mode(mode))
            .with_context(|| format!("failed to set permissions on {}", target.display()))
    }

    /// `clonefile(2)` clones timestamps along with the data.
    #[cfg(target_os = "macos")]
    fn preserve_modified_time(_target: &Path, _metadata: &Metadata) -> Result<()> {
        Ok(())
    }

    /// A reflink shares extents with a freshly created inode. Restore the source
    /// modification time so JJ can trust the copied working-copy state.
    #[cfg(not(target_os = "macos"))]
    fn preserve_modified_time(target: &Path, metadata: &Metadata) -> Result<()> {
        let modified = metadata.modified()?;
        fs::File::open(target)
            .and_then(|file| file.set_modified(modified))
            .with_context(|| format!("failed to set modification time on {}", target.display()))
    }
}

#[cfg(not(unix))]
mod imp {
    use anyhow::{Result, bail};
    use std::path::Path;

    pub(super) fn unsupported_reason(_source: &Path, _destination_parent: &Path) -> Option<String> {
        Some("copy-on-write cloning is supported on macOS and Linux only".to_owned())
    }

    pub(super) fn clone_files(
        _source: &Path,
        _files: &[std::path::PathBuf],
        _destination: &Path,
    ) -> Result<()> {
        bail!("copy-on-write cloning is supported on macOS and Linux only")
    }

    pub(super) fn probe_between(_source: &Path, _destination: &Path) -> Result<()> {
        bail!("copy-on-write cloning is supported on macOS and Linux only")
    }

    pub(super) fn validate_tree(_root: &Path) -> Result<()> {
        bail!("tree validation is supported on macOS and Linux only")
    }

    pub(super) fn clone_tree(_source: &Path, _destination: &Path) -> Result<()> {
        bail!("copy-on-write cloning is supported on macOS and Linux only")
    }

    pub(super) fn remove_path(_path: &Path) -> Result<()> {
        bail!("safe removal is supported on macOS and Linux only")
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::os::unix::net::UnixListener;

    /// Run the test only when real reflinks work between the two paths, honoring
    /// `JW_TEST_REQUIRE_COW`.
    fn require_reflink(source: &Path, destination: &Path) -> bool {
        match probe_between(source, destination) {
            Ok(()) => true,
            Err(reason) => {
                assert!(
                    std::env::var_os("JW_TEST_REQUIRE_COW").is_none_or(|value| value.is_empty()),
                    "real cloning required: {reason:#}"
                );
                eprintln!("skipping reflink test: {reason:#}");
                false
            }
        }
    }

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
        fs::set_permissions(
            source.join("nested/deeper/tool.sh"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        symlink("deeper/tool.sh", source.join("nested/link")).unwrap();
        fs::write(source.join(".env"), "SECRET=1\n").unwrap();
        let files = [
            "nested/deeper/tool.sh",
            "nested/link",
            "deleted.txt",
            "nested",
        ]
        .map(PathBuf::from);

        clone_files(&source, &files, &destination).expect("clone files");

        let tool = destination.join("nested/deeper/tool.sh");
        assert_eq!(fs::read_to_string(&tool).unwrap(), "#!/bin/sh\n");
        assert_eq!(
            fs::metadata(&tool).unwrap().permissions().mode() & 0o777,
            0o755
        );
        assert_eq!(
            fs::read_link(destination.join("nested/link")).unwrap(),
            Path::new("deeper/tool.sh")
        );
        assert!(!destination.join(".env").exists());
        assert!(!destination.join("deleted.txt").exists());
        // Sharing storage must never couple later edits or share an inode.
        fs::write(&tool, "destination edit\n").unwrap();
        assert_eq!(
            fs::read_to_string(source.join("nested/deeper/tool.sh")).unwrap(),
            "#!/bin/sh\n"
        );
        fs::write(source.join("nested/deeper/tool.sh"), "source edit\n").unwrap();
        assert_eq!(fs::read_to_string(&tool).unwrap(), "destination edit\n");
    }

    #[test]
    fn probe_preserves_existing_files_symlinks_and_directories() {
        let temp = tempfile::tempdir().unwrap();
        let victim = temp.path().join("victim");
        fs::write(&victim, "untouched").unwrap();
        let legacy = temp
            .path()
            .join(format!(".jw-cow-probe-{}", std::process::id()));
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
        assert_eq!(
            fs::read_to_string(other.join("source")).unwrap(),
            "also caller-owned"
        );
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
        assert_eq!(
            fs::read_dir(&destination).unwrap().count(),
            destination_entries
        );
        assert_eq!(
            fs::read_to_string(source.join("kept.txt")).unwrap(),
            "kept\n"
        );
        assert_eq!(
            fs::read_to_string(destination.join("kept.txt")).unwrap(),
            "kept\n"
        );
    }

    #[test]
    fn probe_between_rejects_missing_paths_and_file_destinations() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        fs::create_dir(&source).unwrap();
        let file = temp.path().join("file");
        fs::write(&file, "content\n").unwrap();

        assert!(probe_between(&temp.path().join("missing"), &temp.path()).is_err());
        assert!(probe_between(&source, &temp.path().join("missing")).is_err());
        assert!(probe_between(&file, &temp.path()).is_err());
        assert!(probe_between(&source, &file).is_err());
    }

    #[test]
    fn clone_tree_clones_regular_files_symlinks_and_directories_strictly() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("nested/tool.sh"), "#!/bin/sh\n").unwrap();
        fs::set_permissions(
            source.join("nested/tool.sh"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        fs::write(source.join("nested/private.txt"), "quiet\n").unwrap();
        fs::set_permissions(
            source.join("nested/private.txt"),
            fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        fs::write(source.join("nested/inner.txt"), "inner\n").unwrap();
        fs::write(source.join(".env"), "SECRET=1\n").unwrap();
        symlink("tool.sh", source.join("nested/link")).unwrap();
        // A restrictive directory mode must not prevent cloning its contents.
        fs::set_permissions(source.join("nested"), fs::Permissions::from_mode(0o555)).unwrap();

        if !require_reflink(&source, temp.path()) {
            fs::set_permissions(source.join("nested"), fs::Permissions::from_mode(0o755)).unwrap();
            return;
        }

        clone_tree(&source, &destination).expect("clone tree");

        let nested = destination.join("nested");
        assert_eq!(
            fs::metadata(&nested).unwrap().permissions().mode() & 0o7777,
            0o555
        );
        // Restore writable modes so the tempdir cleanup can descend.
        fs::set_permissions(&nested, fs::Permissions::from_mode(0o755)).unwrap();
        fs::set_permissions(source.join("nested"), fs::Permissions::from_mode(0o755)).unwrap();

        let tool = nested.join("tool.sh");
        assert_eq!(fs::read_to_string(&tool).unwrap(), "#!/bin/sh\n");
        assert_eq!(
            fs::metadata(&tool).unwrap().permissions().mode() & 0o7777,
            0o755
        );
        assert_eq!(
            fs::metadata(nested.join("private.txt"))
                .unwrap()
                .permissions()
                .mode()
                & 0o7777,
            0o600
        );
        assert_eq!(
            fs::read_to_string(nested.join("inner.txt")).unwrap(),
            "inner\n"
        );
        assert_eq!(
            fs::read_to_string(destination.join(".env")).unwrap(),
            "SECRET=1\n"
        );
        assert_eq!(
            fs::read_link(nested.join("link")).unwrap(),
            Path::new("tool.sh")
        );
        assert_eq!(
            fs::metadata(&tool).unwrap().modified().unwrap(),
            fs::metadata(source.join("nested/tool.sh"))
                .unwrap()
                .modified()
                .unwrap()
        );
        // Sharing is copy-on-write: later edits never couple the trees.
        fs::write(&tool, "destination edit\n").unwrap();
        assert_eq!(
            fs::read_to_string(source.join("nested/tool.sh")).unwrap(),
            "#!/bin/sh\n"
        );
    }

    #[test]
    fn clone_tree_rejects_nested_repositories_before_creating_destination() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        fs::create_dir_all(source.join("vendor/pkg/.git")).unwrap();
        fs::write(
            source.join("vendor/pkg/.git/HEAD"),
            "ref: refs/heads/main\n",
        )
        .unwrap();
        symlink("../..", source.join("vendor/pkg/.jj")).unwrap();
        let destination = temp.path().join("destination");

        let error = clone_tree(&source, &destination).unwrap_err();
        assert!(error.to_string().contains("administrative"), "{error:#}");
        assert!(!destination.exists());
        assert_eq!(
            fs::read_to_string(source.join("vendor/pkg/.git/HEAD")).unwrap(),
            "ref: refs/heads/main\n"
        );
    }

    #[test]
    fn clone_tree_rejects_symlink_sources_and_existing_destinations() {
        let temp = tempfile::tempdir().unwrap();
        let real = temp.path().join("real");
        fs::create_dir(&real).unwrap();
        fs::write(real.join("file.txt"), "content\n").unwrap();
        let link = temp.path().join("link");
        symlink(&real, &link).unwrap();

        let error = clone_tree(&link, &temp.path().join("destination")).unwrap_err();
        assert!(error.to_string().contains("symbolic link"), "{error:#}");
        assert!(!temp.path().join("destination").exists());

        let destination = temp.path().join("destination");
        fs::create_dir(&destination).unwrap();
        fs::write(destination.join("preexisting.txt"), "untouched\n").unwrap();
        let error = clone_tree(&real, &destination).unwrap_err();
        assert!(error.to_string().contains("failed to create"), "{error:#}");
        assert_eq!(
            fs::read_to_string(destination.join("preexisting.txt")).unwrap(),
            "untouched\n"
        );
    }

    #[test]
    fn clone_tree_rejects_special_entries() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        fs::create_dir(&source).unwrap();
        UnixListener::bind(source.join("socket")).unwrap();

        let error = clone_tree(&source, &temp.path().join("destination")).unwrap_err();
        assert!(error.to_string().contains("special"), "{error:#}");
        assert!(!temp.path().join("destination").exists());
    }

    #[test]
    fn validate_tree_accepts_files_directories_and_symlinks() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        fs::create_dir_all(root.join("nested/deeper")).unwrap();
        fs::write(root.join("nested/deeper/file.txt"), "content\n").unwrap();
        fs::set_permissions(
            root.join("nested/deeper/file.txt"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
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
        assert_eq!(
            fs::read_to_string(target.join("inner.txt")).unwrap(),
            "kept\n"
        );

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
        assert_eq!(
            fs::read_to_string(tree.join("sub/file.txt")).unwrap(),
            "kept\n"
        );
        assert_eq!(
            fs::read_to_string(tree.join("sub/.jj/marker")).unwrap(),
            "kept\n"
        );

        // A special entry aborts the removal before anything is deleted.
        let special = temp.path().join("special");
        fs::create_dir(&special).unwrap();
        fs::write(special.join("keep.txt"), "kept\n").unwrap();
        UnixListener::bind(special.join("socket")).unwrap();
        let error = remove_path(&special).unwrap_err();
        assert!(error.to_string().contains("special"), "{error:#}");
        assert_eq!(
            fs::read_to_string(special.join("keep.txt")).unwrap(),
            "kept\n"
        );

        // Administrative names are never selected for removal, even directly.
        let administrative = temp.path().join("administrative");
        fs::create_dir_all(administrative.join(".jj")).unwrap();
        fs::write(administrative.join(".jj/marker"), "kept\n").unwrap();
        let error = remove_path(&administrative.join(".jj")).unwrap_err();
        assert!(error.to_string().contains("administrative"), "{error:#}");
        assert_eq!(
            fs::read_to_string(administrative.join(".jj/marker")).unwrap(),
            "kept\n"
        );
        fs::write(administrative.join(".git"), "gitdir: elsewhere\n").unwrap();
        let error = remove_path(&administrative.join(".git")).unwrap_err();
        assert!(error.to_string().contains("administrative"), "{error:#}");
        assert!(administrative.join(".git").exists());
    }
}
