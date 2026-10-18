//! Command-local Linux topology front guard. No entry-path mount lookups.
use anyhow::{Context, Result, bail, ensure};
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct Mount {
    device: Vec<u8>,
    root: PathBuf,
    point: PathBuf,
    filesystem: Vec<u8>,
}

#[cfg(target_os = "linux")]
fn decode(bytes: &[u8]) -> Result<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            let code = bytes.get(index + 1..index + 4).context("truncated mountinfo escape")?;
            ensure!(code.iter().all(|c| (b'0'..=b'7').contains(c)), "invalid mountinfo escape");
            let value = (code[0] - b'0') as u16 * 64 + (code[1] - b'0') as u16 * 8 + (code[2] - b'0') as u16;
            ensure!(value > 0 && value <= 255, "invalid mountinfo byte");
            out.push(value as u8);
            index += 4;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    Ok(std::ffi::OsString::from_vec(out).into())
}

#[cfg(target_os = "linux")]
fn parse(bytes: &[u8]) -> Result<Vec<Mount>> {
    bytes
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            let fields: Vec<_> = line.split(|b| *b == b' ').collect();
            let separator = fields.iter().position(|field| *field == b"-").context("mountinfo lacks separator")?;
            ensure!(separator >= 6 && fields.len() >= separator + 4, "malformed mountinfo record");
            // Parent IDs can be absent from this namespace/chroot's visible table.
            let root = decode(fields[3])?;
            let point = decode(fields[4])?;
            // Pseudo filesystems such as nsfs legitimately expose `net:[inode]`
            // roots. They cannot locate workspace storage, but unrelated mounts
            // must not make the entire namespace unparseable.
            ensure!(point.is_absolute(), "mountinfo mount points must be absolute");
            Ok(Mount {
                device: fields[2].to_vec(),
                root,
                point,
                filesystem: fields[separator + 1].to_vec(),
            })
        })
        .collect()
}

pub(super) struct Topology {
    #[cfg(target_os = "linux")]
    mounts: Vec<Mount>,
}
impl Topology {
    pub(super) fn capture() -> Result<Self> {
        Ok(Self {
            #[cfg(target_os = "linux")]
            mounts: parse(&std::fs::read("/proc/self/mountinfo").context("cannot verify Linux mount topology")?)?,
        })
    }
    pub(super) fn workspace(&self, root: &Path, allow_git: bool) -> Result<()> {
        #[cfg(target_os = "linux")]
        self.location(root)?;
        #[cfg(target_os = "linux")]
        for mount in &self.mounts {
            if mount.point != root && mount.point.starts_with(root) {
                let relative = mount.point.strip_prefix(root)?;
                let first = relative.components().next().unwrap().as_os_str();
                if first == ".jj" || (allow_git && first == ".git") {
                    continue;
                }
                bail!("refusing {}: it is a mounted filesystem boundary", mount.point.display());
            }
        }
        #[cfg(not(target_os = "linux"))]
        let _ = (root, allow_git);
        Ok(())
    }
    #[cfg(target_os = "linux")]
    fn location(&self, path: &Path) -> Result<(&[u8], PathBuf)> {
        let mut covering: Vec<_> = self.mounts.iter().filter(|m| path.starts_with(&m.point)).collect();
        covering.sort_by_key(|m| m.point.components().count());
        // A covered ancestor can hide a lexically deeper mount. Longest prefix
        // alone would then map the workspace through that hidden child and miss
        // aliases of the actual visible storage. Refuse the entire ambiguous
        // chain, not just duplicate points at the deepest prefix.
        ensure!(
            !covering.windows(2).any(|pair| pair[0].point == pair[1].point),
            "covered/stacked workspace mounts cannot be verified safely"
        );
        // Traversing autofs can create new mounts and invalidate this upfront
        // snapshot even without an independent writer. Conservatively include
        // covered ancestors; do not trigger automounts during recursive work.
        ensure!(
            !covering.iter().any(|mount| mount.filesystem == b"autofs"),
            "workspace traversal through autofs cannot use a stable topology snapshot"
        );
        let mount = covering.pop().context("workspace is not covered by mountinfo")?;
        ensure!(mount.root.is_absolute(), "workspace mount lacks a filesystem-relative root");
        Ok((&mount.device, mount.root.join(path.strip_prefix(&mount.point)?)))
    }
    pub(super) fn separate(&self, roots: &[PathBuf]) -> Result<()> {
        #[cfg(target_os = "linux")]
        for (index, root) in roots.iter().enumerate() {
            let (device, path) = self.location(root)?;
            for other in &roots[..index] {
                let (other_device, other_path) = self.location(other)?;
                ensure!(
                    device != other_device || (!path.starts_with(&other_path) && !other_path.starts_with(&path)),
                    "workspace roots overlap through mounted aliases: {} and {}",
                    root.display(),
                    other.display()
                );
            }
        }
        #[cfg(not(target_os = "linux"))]
        let _ = roots;
        Ok(())
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    #[test]
    fn native_escaped_paths_optional_fields_and_file_mounts() {
        use std::os::unix::ffi::OsStrExt;
        let mounts = parse(b"21 999 0:4 / / rw shared:1 - btrfs /dev/a rw\n22 21 0:4 /data /ws/a\\040b\\377 rw - btrfs /dev/a rw\n23 21 0:4 /leaf /ws/file rw - btrfs /dev/a rw\n").unwrap();
        assert_eq!(mounts[1].point.as_os_str().as_bytes(), b"/ws/a b\xff");
        assert!(Topology { mounts }.workspace(Path::new("/ws"), false).is_err());
    }
    #[test]
    fn workspace_mount_allowed_but_bind_aliases_rejected() {
        let topology = Topology {
            mounts: parse(b"1 0 0:4 / / rw - btrfs disk rw\n2 1 0:4 /home/source /target rw - btrfs disk rw\n").unwrap(),
        };
        topology.workspace(Path::new("/target"), false).unwrap();
        assert!(topology.separate(&["/home/source".into(), "/target".into()]).is_err());
    }

    #[test]
    fn overmounted_ancestor_must_not_map_through_hidden_lexical_child() {
        let topology = Topology {
            mounts: parse(b"1 0 0:4 / / rw - btrfs disk rw\n2 1 0:4 /A /a rw - btrfs disk rw\n3 2 0:4 /X /a/b rw - btrfs disk rw\n4 1 0:4 /Y /a rw - btrfs disk rw\n5 1 0:4 /Y/b/repo /target rw - btrfs disk rw\n").unwrap(),
        };
        // Longest lexical prefix would wrongly return /X/repo, not /Y/b/repo,
        // accepting /target as separate storage. Ambiguity must refuse first.
        let error = topology.location(Path::new("/a/b/repo")).unwrap_err();
        assert!(error.to_string().contains("covered/stacked"));
        assert!(topology.workspace(Path::new("/a/b/repo"), false).is_err());
        assert!(topology.separate(&["/a/b/repo".into(), "/target".into()]).is_err());
    }

    #[test]
    fn autofs_root_and_ancestor_refuse_but_unrelated_and_excluded_admin_do_not() {
        let topology = Topology {
            mounts: parse(b"1 0 0:4 / / rw - btrfs disk rw\n2 1 0:5 / /auto rw - autofs systemd-1 rw\n3 2 0:4 /actual /auto/loaded rw - btrfs disk rw\n4 1 0:6 / /safe/.jj/auto rw - autofs systemd-1 rw\n").unwrap(),
        };
        for path in ["/auto", "/auto/repo", "/auto/loaded/repo"] {
            assert!(topology.workspace(Path::new(path), false).unwrap_err().to_string().contains("autofs"));
        }
        topology.workspace(Path::new("/safe"), false).unwrap();
        assert!(topology.workspace(Path::new("/"), false).is_err());
    }
}
