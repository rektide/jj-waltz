//! Repository-relative tracked-leaf queries and path-safety primitives shared
//! by creation planning, reseeding, and rollback. Reads go through the native
//! JJ client; nothing here mutates a repository.
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

use jj_waltz_native_jj::jj::JjClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub path: PathBuf,
    pub kind: String,
    pub executable: bool,
}

pub fn directory_identity(path: &Path) -> Result<(u64, u64)> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir() {
        bail!("identity is not a real directory: {}", path.display());
    }
    Ok((metadata.dev(), metadata.ino()))
}

pub fn relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(part) if part != ".jj" && part != ".git"))
    {
        bail!("unsafe repository-relative path: {}", path.display());
    }
    Ok(())
}

pub fn entries(client: &JjClient, operation: &str, commit: &str) -> Result<Vec<Entry>> {
    let output = client
        .run_at(
            operation,
            [
                "file",
                "list",
                "-r",
                &jj_waltz_native_jj::jj::commit_id_revset(commit)?,
                "-T",
                r#"json(path) ++ "\t" ++ json(file_type) ++ "\t" ++ json(executable) ++ "\n""#,
            ],
        )
        .context("JJ lacks JSON tree-entry template support; upgrade JJ")?;
    let mut entries = Vec::new();
    for line in output.stdout()?.lines() {
        let fields = line.split('\t').collect::<Vec<_>>();
        let [path, kind, executable] = fields.as_slice() else {
            bail!("malformed JJ tree entry");
        };
        let entry = Entry {
            path: PathBuf::from(serde_json::from_str::<String>(path)?),
            kind: serde_json::from_str(kind)?,
            executable: serde_json::from_str(executable)?,
        };
        relative(&entry.path)?;
        if !matches!(entry.kind.as_str(), "file" | "symlink") {
            bail!("unsupported tracked entry {} ({})", entry.path.display(), entry.kind);
        }
        entries.push(entry);
    }
    Ok(entries)
}

pub fn file_bytes(client: &JjClient, operation: &str, commit: &str, path: &Path) -> Result<Vec<u8>> {
    // Filesets are expressions, not literal paths. `file:` with a JSON-quoted
    // string preserves newlines, spaces, quotes and names resembling selectors.
    let fileset = format!("file:{}", serde_json::to_string(path.to_str().context("tracked path not UTF-8")?)?);
    let output = client.run_at(operation, ["file", "show", "-r", &jj_waltz_native_jj::jj::commit_id_revset(commit)?, &fileset])?;
    let diagnostics = output.stderr();
    if !diagnostics.is_empty() {
        eprintln!("{diagnostics}");
    }
    Ok(output.stdout_bytes().to_vec())
}

pub fn confined_parents(root: &Path, path: &Path) -> Result<()> {
    relative(path)?;
    let mut parent = root.to_path_buf();
    for component in path.parent().unwrap_or(Path::new("")).components() {
        parent.push(component);
        match fs::symlink_metadata(&parent) {
            Ok(m) if m.is_dir() => {}
            Ok(_) => bail!("path parent is not a real directory: {}", parent.display()),
            Err(e) if e.kind() == ErrorKind::NotFound => break,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

/// Strict installation of *every* recorded source leaf. Unlike the creation
/// adapter this has no pre-stat filter and never skips a missing/changed kind.
/// Failure retains production partial progress; required reflinks never downgrade.
pub fn seed(source: &Path, target: &Path, entries: &[Entry], devices: (u64, u64)) -> Result<diff_dirs_sync::Progress, diff_dirs_sync::Failure> {
    use diff_dirs::Check;
    struct NativeLeaves<'a> {
        entries: std::collections::BTreeMap<&'a Path, &'a Entry>,
        devices: (u64, u64),
    }
    impl Check for NativeLeaves<'_> {
        fn check(&self, e: diff_dirs::Entry<'_>) -> Result<(), diff_dirs::CheckError> {
            use std::os::unix::fs::PermissionsExt;
            let device = if e.side == diff_dirs::Side::Original {
                self.devices.0
            } else {
                self.devices.1
            };
            if e.metadata.dev() != device {
                return Err("native seed crossed a device boundary".into());
            }
            if e.side == diff_dirs::Side::Original && !e.metadata.is_dir() {
                let path = e.path.strip_prefix(e.root)?;
                let expected = self.entries.get(path).ok_or("native seed observed unlisted leaf")?;
                if (expected.kind == "file" && !e.metadata.is_file())
                    || (expected.kind == "symlink" && !e.metadata.is_symlink())
                    || (expected.kind == "file" && expected.executable != (e.metadata.permissions().mode() & 0o111 != 0))
                {
                    return Err("prepared source leaf kind/mode differs from recorded membership".into());
                }
            }
            Ok(())
        }
    }
    diff_dirs_sync::seed_leaves(
        source,
        target,
        entries.iter().map(|e| &e.path),
        diff_dirs_sync::CopyPolicy::ReflinkRequired,
        NativeLeaves {
            entries: entries.iter().map(|e| (e.path.as_path(), e)).collect(),
            devices,
        },
    )
}
