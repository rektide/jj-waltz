//! Frozen native ignore rules and shared tree/identity primitives for creation and reseeding.
use crate::jj::JjClient;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Excludes {
    global: Vec<u8>,
    info: Vec<u8>,
}

/// Command-owned cache. Equal pinned coordinates reuse extracted rule bytes;
/// equal coordinates AND effective excludes share the sandbox and native oracle.
#[derive(Default)]
pub(crate) struct FrozenRules {
    files: BTreeMap<String, BTreeMap<PathBuf, Vec<u8>>>,
    rules: BTreeMap<(String, Excludes), Rc<IgnoreRules>>,
    equivalent: BTreeMap<String, String>,
}
impl FrozenRules {
    /// Caller supplies a JJ whole-tracked-tree equality proof, not merely equal
    /// revisions or matching .gitignore path names. Excludes remain independent.
    pub(crate) fn same_tree(&mut self, commit: &str, equivalent: &str) {
        if commit != equivalent {
            self.equivalent.insert(commit.to_owned(), equivalent.to_owned());
        }
    }
    pub(crate) fn get(&mut self, client: &JjClient, operation: &str, commit: &str, entries: &[Entry]) -> Result<Rc<IgnoreRules>> {
        let excludes = Excludes::capture(client, operation)?;
        let coordinate = self.equivalent.get(commit).map(String::as_str).unwrap_or(commit);
        let key = (coordinate.to_owned(), excludes);
        if let Some(rules) = self.rules.get(&key) {
            return Ok(Rc::clone(rules));
        }
        if !self.files.contains_key(coordinate) {
            self.files.insert(coordinate.to_owned(), ignore_files(client, operation, commit, entries)?);
        }
        let rules = Rc::new(IgnoreRules::build(&self.files[coordinate], &key.1)?);
        self.rules.insert(key, Rc::clone(&rules));
        Ok(rules)
    }
}

fn ignore_files(client: &JjClient, operation: &str, commit: &str, entries: &[Entry]) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut ignores = BTreeMap::new();
    for entry in entries.iter().filter(|e| e.path.file_name().is_some_and(|n| n == ".gitignore")) {
        if entry.kind != "file" {
            bail!("symlinked ignore files cannot be verified safely");
        }
        ignores.insert(entry.path.clone(), file_bytes(client, operation, commit, &entry.path)?);
    }
    Ok(ignores)
}

/// Git's native ignore parser, over an isolated tree of frozen desired .gitignore
/// files and copies of JJ's backend/global excludes. No target revision is changed.
#[derive(Debug)]
pub(crate) struct IgnoreRules {
    pub(crate) directory: tempfile::TempDir,
    oracle: RefCell<Option<Oracle>>,
    answers: RefCell<BTreeMap<(PathBuf, bool), bool>>,
}

#[derive(Debug)]
struct Oracle {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    stderr: Option<std::thread::JoinHandle<std::io::Result<Vec<u8>>>>,
}
impl Oracle {
    fn stop(&mut self) -> Vec<u8> {
        drop(self.input.take());
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.stderr
            .take()
            .and_then(|reader| reader.join().ok())
            .and_then(Result::ok)
            .unwrap_or_default()
    }
    fn start(root: &Path) -> Result<Self> {
        let mut child = isolated_git(root)
            .env("GIT_FLUSH", "1")
            .args(["check-ignore", "--no-index", "--stdin", "-z", "--verbose", "--non-matching"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stderr = child.stderr.take().unwrap();
        let reader = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            // Drain even excessive diagnostics so the child cannot block on stderr.
            let mut chunk = [0; 4096];
            loop {
                let count = stderr.read(&mut chunk)?;
                if count == 0 {
                    break;
                }
                if bytes.len() < 65536 {
                    bytes.extend_from_slice(&chunk[..count]);
                }
            }
            Ok(bytes)
        });
        Ok(Self {
            input: child.stdin.take(),
            output: BufReader::new(child.stdout.take().unwrap()),
            child,
            stderr: Some(reader),
        })
    }
    fn query(&mut self, input: &[u8]) -> Result<bool> {
        self.input.as_mut().context("closed ignore oracle")?.write_all(input)?;
        self.input.as_mut().unwrap().flush()?;
        let mut fields = Vec::with_capacity(4);
        for _ in 0..4 {
            let mut field = Vec::new();
            let count = (&mut self.output).take(1024 * 1024).read_until(0, &mut field)?;
            if count == 0 || field.pop() != Some(0) {
                bail!("ignore-rule probe failed or returned malformed output");
            }
            fields.push(field);
        }
        if fields[3] != input[..input.len() - 1] {
            bail!("ignore-rule probe returned a mismatched path");
        }
        Ok(!fields[2].is_empty() && fields[2][0] != b'!')
    }
}
impl Drop for Oracle {
    fn drop(&mut self) {
        self.stop();
    }
}

impl IgnoreRules {
    pub(crate) fn new(client: &JjClient, operation: &str, commit: &str, entries: &[Entry]) -> Result<Self> {
        Self::build(&ignore_files(client, operation, commit, entries)?, &Excludes::capture(client, operation)?)
    }
    fn build(ignores: &BTreeMap<PathBuf, Vec<u8>>, excludes: &Excludes) -> Result<Self> {
        let directory = tempfile::tempdir()?;
        checked_git(directory.path(), ["init", "--quiet", "--template="])?;
        fs::create_dir_all(directory.path().join(".git/info"))?;
        // Ignore classification uses the frozen tracked .gitignore files of the
        // pinned tree plus backend/global excludes; the checkout's untracked
        // .gitignore files are deliberately not hunted with a tree walk.
        for (path, bytes) in ignores {
            let path = directory.path().join(path);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(path, bytes)?;
        }
        let global = directory.path().join(".git/jw-global-exclude");
        fs::write(&global, &excludes.global)?;
        let global = global.to_str().context("ignore probe path not UTF-8")?;
        checked_git(directory.path(), ["config", "core.excludesFile", global])?;
        fs::write(directory.path().join(".git/info/exclude"), &excludes.info)?;
        Ok(Self::from_directory(directory))
    }
}
impl Excludes {
    fn capture(client: &JjClient, operation: &str) -> Result<Self> {
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
        let info = backend.as_ref().map(|p| p.join("info/exclude"));
        Ok(Self {
            global: read_optional(exclude.as_deref())?,
            info: read_optional(info.as_deref())?,
        })
    }
}
impl IgnoreRules {
    pub(crate) fn from_directory(directory: tempfile::TempDir) -> Self {
        Self {
            directory,
            oracle: RefCell::new(None),
            answers: RefCell::new(BTreeMap::new()),
        }
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
        let key = (path.to_path_buf(), directory);
        if let Some(answer) = self.answers.borrow().get(&key) {
            return Ok(*answer);
        }
        let mut input = path.to_str().context("ignored path not UTF-8")?.as_bytes().to_vec();
        if directory {
            input.push(b'/');
        }
        input.push(0);
        let mut oracle = self.oracle.borrow_mut();
        if oracle.is_none() {
            *oracle = Some(Oracle::start(self.directory.path())?);
        }
        let answer = match oracle.as_mut().unwrap().query(&input) {
            Ok(answer) => answer,
            Err(error) => {
                let stderr = oracle.take().unwrap().stop();
                return Err(error).with_context(|| format!("ignore-rule probe failed for {}: {}", path.display(), String::from_utf8_lossy(&stderr)));
            }
        };
        self.answers.borrow_mut().insert(key, answer);
        Ok(answer)
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

#[cfg(test)]
mod oracle_tests {
    use super::*;

    #[test]
    fn persistent_oracle_frames_negation_nonmatches_and_memoizes() {
        let rules = IgnoreRules::from_directory(tempfile::tempdir().unwrap());
        checked_git(rules.directory.path(), ["init", "--quiet", "--template="]).unwrap();
        fs::write(rules.directory.path().join(".gitignore"), "*.cache\n!keep.cache\nfolder/\n").unwrap();
        assert!(rules.is_ignored(Path::new("one.cache"), false).unwrap());
        let pid = rules.oracle.borrow().as_ref().unwrap().child.id();
        assert!(!rules.is_ignored(Path::new("keep.cache"), false).unwrap());
        assert!(!rules.is_ignored(Path::new("other"), true).unwrap());
        assert!(rules.is_ignored(Path::new("folder"), true).unwrap());
        assert!(rules.is_ignored(Path::new("one.cache"), false).unwrap());
        assert_eq!(rules.answers.borrow().len(), 4);
        assert_eq!(rules.oracle.borrow().as_ref().unwrap().child.id(), pid);
    }

    #[test]
    fn failed_oracle_is_joined_and_not_memoized_as_a_negative() {
        let rules = IgnoreRules::from_directory(tempfile::tempdir().unwrap());
        checked_git(rules.directory.path(), ["init", "--quiet", "--template="]).unwrap();
        *rules.oracle.borrow_mut() = Some(Oracle::start(rules.directory.path()).unwrap());
        rules.oracle.borrow_mut().as_mut().unwrap().child.kill().unwrap();
        assert!(rules.is_ignored_directory(Path::new("unknown")).is_err());
        assert!(rules.answers.borrow().is_empty());
        assert!(rules.oracle.borrow().is_none());
        assert!(!rules.is_ignored_directory(Path::new("unknown")).unwrap());
    }
}
