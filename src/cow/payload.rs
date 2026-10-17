//! Frozen native ignore rules and payload proofs shared by creation and reseeding.
use crate::{cow, jj::JjClient};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

/// Git's native ignore parser, over an isolated tree of frozen desired .gitignore
/// files and copies of JJ's backend/global excludes. No target revision is changed.
#[derive(Debug)]
pub(crate) struct IgnoreRules {
    pub(crate) directory: tempfile::TempDir,
}
impl IgnoreRules {
    pub(crate) fn new(client: &JjClient, operation: &str, commit: &str, entries: &[Entry], retain_untracked: bool) -> Result<Self> {
        let directory = tempfile::tempdir()?;
        checked_git(directory.path(), ["init", "--quiet", "--template="])?;
        fs::create_dir_all(directory.path().join(".git/info"))?;
        let mut ignores = BTreeMap::new();
        // On-disk untracked nested ignores are part of a retained checkout's
        // semantics; tracked files are subsequently overridden by the desired tree.
        let old_commit = client.resolve_one_at(operation, "@")?.commit_id;
        let old_entries = self::entries(client, operation, &old_commit)?;
        for item in if retain_untracked { stamp(client.cwd())? } else { Vec::new() } {
            if item.path.file_name().is_some_and(|n| n == ".gitignore") && !entries.iter().chain(&old_entries).any(|e| e.path == item.path) {
                let path = client.cwd().join(&item.path);
                if fs::symlink_metadata(&path)?.is_file() {
                    ignores.insert(item.path, fs::read(path)?);
                }
            }
        }
        for entry in entries.iter().filter(|e| e.path.file_name().is_some_and(|n| n == ".gitignore")) {
            if entry.kind != "file" {
                bail!("symlinked ignore files cannot be verified safely");
            }
            ignores.insert(entry.path.clone(), file_bytes(client, operation, commit, &entry.path)?);
        }
        for (path, bytes) in ignores {
            let path = directory.path().join(path);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(path, bytes)?;
        }
        let backend = client.run_at_unchecked(operation, ["git", "root"])?;
        let backend = if backend.success() {
            Some(PathBuf::from(backend.trimmed_stdout()?).canonicalize()?)
        } else {
            None
        };
        let mut command = git_command(client.cwd());
        if let Some(backend) = &backend {
            command.arg("--git-dir").arg(backend);
        }
        let output = command.args(["config", "--path", "--get", "core.excludesFile"]).output()?;
        let exclude = if output.status.success() {
            let path = PathBuf::from(std::str::from_utf8(&output.stdout)?.trim_end_matches(['\r', '\n']));
            if path.is_absolute() { Some(path) } else { Some(client.cwd().join(path)) }
        } else if output.status.code() == Some(1) {
            std::env::var_os("XDG_CONFIG_HOME")
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
                .map(|p| p.join("git/ignore"))
        } else {
            bail!("cannot read Git excludes configuration: {}", String::from_utf8_lossy(&output.stderr));
        };
        let global = directory.path().join(".git/jw-global-exclude");
        fs::write(&global, read_optional(exclude.as_deref())?)?;
        let global = global.to_str().context("ignore probe path not UTF-8")?;
        checked_git(directory.path(), ["config", "core.excludesFile", global])?;
        let info = backend.as_ref().map(|p| p.join("info/exclude"));
        fs::write(directory.path().join(".git/info/exclude"), read_optional(info.as_deref())?)?;
        Ok(Self { directory })
    }
    pub(crate) fn require_ignored(&self, path: &Path) -> Result<()> {
        self.check(path, true)
    }
    pub(crate) fn require_ignored_file(&self, path: &Path) -> Result<()> {
        self.check(path, false)
    }
    fn check(&self, path: &Path, directory: bool) -> Result<()> {
        if !self.is_ignored(path, directory)? {
            bail!("selected path is not ignored under the frozen desired rules: {}", path.display());
        }
        Ok(())
    }
    pub(crate) fn is_ignored_directory(&self, path: &Path) -> Result<bool> {
        self.is_ignored(path, true)
    }
    fn is_ignored(&self, path: &Path, directory: bool) -> Result<bool> {
        relative(path)?;
        let mut child = isolated_git(self.directory.path())
            .args(["check-ignore", "--no-index", "-z", "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut input = path.to_str().context("ignored path not UTF-8")?.as_bytes().to_vec();
        if directory {
            input.push(b'/');
        }
        input.push(0);
        child.stdin.take().unwrap().write_all(&input)?;
        let output = child.wait_with_output()?;
        if output.status.success() && output.stdout == input {
            Ok(true)
        } else if output.status.code() == Some(1) && output.stdout.is_empty() && output.stderr.is_empty() {
            Ok(false)
        } else {
            bail!(
                "ignore-rule probe failed or returned malformed output for {}: {}",
                path.display(),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

fn read_optional(path: Option<&Path>) -> Result<Vec<u8>> {
    match path.map(fs::read).transpose() {
        Ok(bytes) => Ok(bytes.unwrap_or_default()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e).context("cannot read exclude file"),
    }
}

fn git_command(root: &Path) -> Command {
    let mut command = Command::new("git");
    command.current_dir(root);
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_CEILING_DIRECTORIES",
        "GIT_CONFIG_COUNT",
        "GIT_CONFIG_PARAMETERS",
    ] {
        command.env_remove(key);
    }
    command
}
fn isolated_git(root: &Path) -> Command {
    let mut command = git_command(root);
    command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env_remove("GIT_TEMPLATE_DIR")
        .args(["-c", "init.templateDir="]);
    command
}
pub(crate) fn checked_git<I, S>(root: &Path, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let output = isolated_git(root).args(args).output()?;
    if !output.status.success() {
        bail!("ignore-rule probe failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Entry {
    pub(crate) path: PathBuf,
    pub(crate) kind: String,
    pub(crate) executable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Stamp {
    pub(crate) path: PathBuf,
    pub(crate) device: u64,
    pub(crate) inode: u64,
    pub(crate) size: u64,
    pub(crate) mode: u32,
    pub(crate) mtime: i64,
    pub(crate) mtime_ns: i64,
    pub(crate) ctime: i64,
    pub(crate) ctime_ns: i64,
}

pub(crate) fn directory_identity(path: &Path) -> Result<(u64, u64)> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir() {
        bail!("identity is not a real directory: {}", path.display());
    }
    Ok((metadata.dev(), metadata.ino()))
}

pub(crate) fn relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(part) if part != ".jj" && part != ".git"))
    {
        bail!("unsafe repository-relative path: {}", path.display());
    }
    Ok(())
}

pub(crate) fn entries(client: &JjClient, operation: &str, commit: &str) -> Result<Vec<Entry>> {
    let output = client
        .run_at(
            operation,
            [
                "file",
                "list",
                "-r",
                commit,
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

pub(crate) fn file_bytes(client: &JjClient, operation: &str, commit: &str, path: &Path) -> Result<Vec<u8>> {
    // Filesets are expressions, not literal paths. `file:` with a JSON-quoted
    // string preserves newlines, spaces, quotes and names resembling selectors.
    let fileset = format!("file:{}", serde_json::to_string(path.to_str().context("tracked path not UTF-8")?)?);
    let output = client.run_at(operation, ["file", "show", "-r", commit, &fileset])?;
    let diagnostics = output.stderr();
    if !diagnostics.is_empty() {
        eprintln!("{diagnostics}");
    }
    Ok(output.stdout_bytes().to_vec())
}

pub(crate) fn confined_parents(root: &Path, path: &Path) -> Result<()> {
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

pub(crate) fn stamp(root: &Path) -> Result<Vec<Stamp>> {
    fn visit(root: &Path, path: &Path, stamps: &mut Vec<Stamp>) -> Result<()> {
        let mut children = fs::read_dir(path)?.collect::<std::io::Result<Vec<_>>>()?;
        children.sort_by_key(|c| c.file_name());
        for child in children {
            let path = child.path();
            if path.parent() == Some(root) && [".jj", ".git"].iter().any(|n| child.file_name() == *n) {
                continue;
            }
            let m = fs::symlink_metadata(&path)?;
            stamps.push(Stamp {
                path: path.strip_prefix(root)?.to_path_buf(),
                device: m.dev(),
                inode: m.ino(),
                size: m.len(),
                mode: m.mode(),
                mtime: m.mtime(),
                mtime_ns: m.mtime_nsec(),
                ctime: m.ctime(),
                ctime_ns: m.ctime_nsec(),
            });
            if m.is_dir() {
                visit(root, &path, stamps)?;
            }
        }
        Ok(())
    }
    let mut stamps = Vec::new();
    visit(root, root, &mut stamps)?;
    Ok(stamps)
}

/// Logical dependency snapshot identity survives inode/ctime changes when an
/// operator restores the frozen source. Administrative and special files never
/// become digest input (the same strict validation precedes hashing and copying).
pub(crate) fn payload_digest(root: &Path, paths: &[PathBuf]) -> Result<String> {
    fn visit(root: &Path, relative_path: &Path, hash: &mut Sha256) -> Result<()> {
        use std::os::unix::ffi::OsStrExt;
        relative(relative_path)?;
        confined_parents(root, relative_path)?;
        let path = root.join(relative_path);
        let m = fs::symlink_metadata(&path)?;
        let name = relative_path.as_os_str().as_bytes();
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name);
        hash.update((m.mode() & 0o7777).to_le_bytes());
        if m.is_dir() {
            hash.update(b"D");
            let mut children = fs::read_dir(&path)?.collect::<std::io::Result<Vec<_>>>()?;
            children.sort_by_key(|e| e.file_name());
            hash.update((children.len() as u64).to_le_bytes());
            for child in children {
                visit(root, &relative_path.join(child.file_name()), hash)?;
            }
        } else if m.is_file() {
            hash.update(b"F");
            hash.update(m.len().to_le_bytes());
            let mut file = File::open(path)?;
            let mut buffer = [0; 65536];
            loop {
                let count = file.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                hash.update(&buffer[..count]);
            }
        } else if m.file_type().is_symlink() {
            hash.update(b"L");
            let target = fs::read_link(path)?;
            let target = target.as_os_str().as_bytes();
            hash.update((target.len() as u64).to_le_bytes());
            hash.update(target);
        } else {
            bail!("unsupported dependency entry: {}", path.display());
        }
        Ok(())
    }
    let mut hash = Sha256::new();
    for path in paths {
        cow::validate_tree(&root.join(path))?;
        visit(root, path, &mut hash)?;
    }
    Ok(hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
}
