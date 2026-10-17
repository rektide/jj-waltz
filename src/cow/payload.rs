//! Frozen native ignore rules and shared tree/identity primitives for creation and reseeding.
use crate::jj::JjClient;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{ErrorKind, Write};
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
    pub(crate) fn new(client: &JjClient, operation: &str, commit: &str, entries: &[Entry]) -> Result<Self> {
        let directory = tempfile::tempdir()?;
        checked_git(directory.path(), ["init", "--quiet", "--template="])?;
        fs::create_dir_all(directory.path().join(".git/info"))?;
        let mut ignores = BTreeMap::new();
        // Ignore classification uses the frozen tracked .gitignore files of the
        // pinned tree plus backend/global excludes; the checkout's untracked
        // .gitignore files are deliberately not hunted with a tree walk.
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
