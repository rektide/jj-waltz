//! Frozen native ignore inputs. Streaming refill compiles immutable in-process
//! gix-ignore searches from pinned .gitignore files and backend/global excludes.
//! The legacy creation/reseed adapter retains its persistent Git oracle over an
//! isolated frozen tree; tests also use that oracle for differential evidence.
use crate::tracked::{Entry, file_bytes};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::rc::Rc;
use std::sync::Arc;

use jj_waltz_native_jj::jj::JjClient;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Excludes {
    global: Vec<u8>,
    info: Vec<u8>,
}

/// Command-owned cache. Equal pinned coordinates reuse extracted rule bytes;
/// equal coordinates AND effective excludes share the sandbox and native oracle.
#[derive(Default)]
pub struct FrozenRules {
    files: BTreeMap<String, BTreeMap<PathBuf, Vec<u8>>>,
    rules: BTreeMap<(String, Excludes), Rc<IgnoreRules>>,
    equivalent: BTreeMap<String, String>,
}
impl FrozenRules {
    /// Caller supplies a JJ whole-tracked-tree equality proof, not merely equal
    /// revisions or matching .gitignore path names. Excludes remain independent.
    pub fn same_tree(&mut self, commit: &str, equivalent: &str) {
        if commit != equivalent {
            self.equivalent.insert(commit.to_owned(), equivalent.to_owned());
        }
    }
    pub fn get(&mut self, client: &JjClient, operation: &str, commit: &str, entries: &[Entry]) -> Result<Rc<IgnoreRules>> {
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

/// Immutable Git-compatible matching over captured control-file bytes. No live
/// ignore files, subprocesses, answer cache or filesystem calls during matching.
#[derive(Debug)]
pub struct FrozenIgnore {
    base: gix_ignore::Search,
    directories: BTreeMap<PathBuf, gix_ignore::Search>,
    identity: [u8; 32],
}

/// Serializable active rule chain and sticky ancestor exclusion. No live paths
/// or matcher pointers: indexes are interpreted only against frozen input bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IgnoreState {
    chain: Vec<PathBuf>,
    ignored: bool,
}
impl IgnoreState {
    pub fn ignored(&self) -> bool {
        self.ignored
    }
}
impl Default for FrozenIgnore {
    fn default() -> Self {
        Self::build(&BTreeMap::new(), &Excludes { global: vec![], info: vec![] }).expect("empty ignore inputs")
    }
}

impl FrozenIgnore {
    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub fn root(&self) -> IgnoreState {
        IgnoreState {
            chain: self.directories.contains_key(Path::new("")).then(PathBuf::new).into_iter().collect(),
            ignored: false,
        }
    }
    /// Advance exactly one real source-directory component. Ignored parents
    /// cannot be reopened, including by control files inside the ignored parent.
    pub fn descend(&self, state: &IgnoreState, path: &Path) -> IgnoreState {
        let mut next = state.clone();
        if !next.ignored {
            let case = gix_ignore::glob::pattern::Case::Sensitive;
            let matched = state.chain.iter().rev().find_map(|parent| {
                let relative = path.strip_prefix(parent).expect("active ignore ancestor");
                self.directories[parent].pattern_matching_relative_path(relative.as_os_str().as_encoded_bytes().into(), Some(true), case)
            });
            next.ignored = matched
                .or_else(|| {
                    self.base
                        .pattern_matching_relative_path(path.as_os_str().as_encoded_bytes().into(), Some(true), case)
                })
                .is_some_and(|m| !m.pattern.is_negative());
            if !next.ignored && self.directories.contains_key(path) {
                next.chain.push(path.to_owned());
            }
        }
        next
    }
    #[cfg(test)]
    pub(crate) fn test_rules(bytes: &[u8]) -> Self {
        Self::build(
            &BTreeMap::from([(PathBuf::from(".gitignore"), bytes.to_vec())]),
            &Excludes { global: vec![], info: vec![] },
        )
        .unwrap()
    }
    fn build(files: &BTreeMap<PathBuf, Vec<u8>>, excludes: &Excludes) -> Result<Self> {
        let mut hash = Sha256::new();
        for bytes in [&excludes.global, &excludes.info] {
            hash.update((bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
        }
        for (path, bytes) in files {
            let path = path.as_os_str().as_encoded_bytes();
            hash.update((path.len() as u64).to_le_bytes());
            hash.update(path);
            hash.update((bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
        }
        let identity = hash.finalize().into();
        let parse = gix_ignore::search::Ignore { support_precious: false };
        let mut base = gix_ignore::Search::default();
        base.add_patterns_buffer(&excludes.global, "global-excludes", None, parse)?;
        base.add_patterns_buffer(&excludes.info, "info-exclude", None, parse)?;
        let directories = files
            .iter()
            .map(|(path, bytes)| {
                let mut matcher = gix_ignore::Search::default();
                // Strip the prefix ourselves, as JJ does. This prevents filesystem
                // path interpretation from changing repository-relative semantics.
                matcher.add_patterns_buffer(bytes, path, None, parse)?;
                Ok((path.parent().unwrap_or(Path::new("")).to_path_buf(), matcher))
            })
            .collect::<Result<_>>()?;
        Ok(Self { base, directories, identity })
    }

    /// Native membership is independent of ignoredness. Callers must still
    /// exclude forced-tracked paths. Ancestor exclusion cannot be reopened by a
    /// deeper negation (including a negation in that ignored directory).
    pub fn ignored(&self, path: &Path, is_directory: bool) -> bool {
        let mut ancestors = path.ancestors().filter(|p| !p.as_os_str().is_empty()).collect::<Vec<_>>();
        ancestors.reverse();
        ancestors.into_iter().any(|p| self.exact(p, p != path || is_directory))
    }

    fn exact(&self, path: &Path, is_directory: bool) -> bool {
        let case = gix_ignore::glob::pattern::Case::Sensitive;
        for parent in path.parent().unwrap_or(Path::new("")).ancestors() {
            if let Some(matcher) = self.directories.get(parent) {
                let relative = path.strip_prefix(parent).expect("ancestor");
                if let Some(matched) = matcher.pattern_matching_relative_path(relative.as_os_str().as_encoded_bytes().into(), Some(is_directory), case) {
                    return !matched.pattern.is_negative();
                }
            }
        }
        self.base
            .pattern_matching_relative_path(path.as_os_str().as_encoded_bytes().into(), Some(is_directory), case)
            .is_some_and(|matched| !matched.pattern.is_negative())
    }
}

/// Batch-owned extraction cache for the streaming consumer. Membership is
/// supplied by the native phase, never enumerated again to capture ignore rules.
#[derive(Default)]
pub struct FrozenIgnores {
    files: BTreeMap<String, BTreeMap<PathBuf, Vec<u8>>>,
    excludes: BTreeMap<PathBuf, Excludes>,
    rules: BTreeMap<(String, Excludes), Arc<FrozenIgnore>>,
    equivalent: BTreeMap<String, String>,
}
impl FrozenIgnores {
    /// Caller supplies native whole-tree equality (for example an empty
    /// single-parent change), not merely equal ignore filenames or stat keys.
    pub fn same_tree(&mut self, commit: &str, coordinate: &str) {
        self.equivalent.insert(commit.to_owned(), coordinate.to_owned());
    }
    pub fn get(&mut self, client: &JjClient, operation: &str, commit: &str, entries: &[Entry]) -> Result<Arc<FrozenIgnore>> {
        if !self.excludes.contains_key(client.cwd()) {
            self.excludes.insert(client.cwd().to_path_buf(), Excludes::capture(client, operation)?);
        }
        let coordinate = self.equivalent.get(commit).map(String::as_str).unwrap_or(commit);
        let key = (coordinate.to_owned(), self.excludes[client.cwd()].clone());
        if let Some(rules) = self.rules.get(&key) {
            return Ok(Arc::clone(rules));
        }
        if !self.files.contains_key(coordinate) {
            self.files.insert(coordinate.to_owned(), ignore_files(client, operation, commit, entries)?);
        }
        let rules = Arc::new(FrozenIgnore::build(&self.files[coordinate], &key.1)?);
        self.rules.insert(key, Arc::clone(&rules));
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
pub struct IgnoreRules {
    pub directory: tempfile::TempDir,
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
    pub fn new(client: &JjClient, operation: &str, commit: &str, entries: &[Entry]) -> Result<Self> {
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
    pub fn from_directory(directory: tempfile::TempDir) -> Self {
        Self {
            directory,
            oracle: RefCell::new(None),
            answers: RefCell::new(BTreeMap::new()),
        }
    }
    pub fn require_ignored(&self, path: &Path) -> Result<()> {
        self.check(path, true)
    }
    pub fn require_ignored_file(&self, path: &Path) -> Result<()> {
        self.check(path, false)
    }
    fn check(&self, path: &Path, directory: bool) -> Result<()> {
        if !self.is_ignored(path, directory)? {
            bail!("selected path is not ignored under the frozen desired rules: {}", path.display());
        }
        Ok(())
    }
    pub fn is_ignored_directory(&self, path: &Path) -> Result<bool> {
        self.is_ignored(path, true)
    }
    fn is_ignored(&self, path: &Path, directory: bool) -> Result<bool> {
        crate::tracked::relative(path)?;
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
pub fn checked_git<I, S>(root: &Path, args: I) -> Result<()>
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

#[cfg(test)]
mod oracle_tests {
    use super::*;

    #[test]
    fn immutable_matcher_agrees_with_git_on_bounded_precedence_and_ancestor_corpus() {
        fn assert_sync<T: Send + Sync>() {}
        assert_sync::<FrozenIgnore>();
        let files = BTreeMap::from([
            (
                PathBuf::from(".gitignore"),
                b"cache/\n*.tmp\n!keep.tmp\n/rooted\n**/build/\nblocked/\n!blocked/open\n\\#literal\n\\!bang\nspace\\ \n$literal\n!override\n".to_vec(),
            ),
            (PathBuf::from("pkg/.gitignore"), b"!local.tmp\n/deep\noverride\n".to_vec()),
            (PathBuf::from("blocked/.gitignore"), b"!open\n!nested/\n".to_vec()),
            (PathBuf::from("pkg/sub/.gitignore"), b"!override\n".to_vec()),
        ]);
        let excludes = Excludes {
            global: b"*.global\noverride\n*.precedence\n".to_vec(),
            info: b"!info.global\n!yes.precedence\n".to_vec(),
        };
        let pure = FrozenIgnore::build(&files, &excludes).unwrap();
        let oracle = IgnoreRules::build(&files, &excludes).unwrap();
        let paths = [
            "cache",
            "cache/file",
            "cache/deep/keep.tmp",
            "file.tmp",
            "keep.tmp",
            "rooted",
            "pkg/rooted",
            "pkg/build",
            "pkg/build/a",
            "blocked",
            "blocked/open",
            "blocked/nested/file",
            "pkg/local.tmp",
            "pkg/deep",
            "pkg/deep/a",
            "pkg/sub/deep",
            "#literal",
            "!bang",
            "space ",
            "space",
            "$literal",
            "x.global",
            "info.global",
            "yes.precedence",
            "no.precedence",
            "override",
            "pkg/override",
            "pkg/sub/override",
            "normal",
            "λ/build",
            "λ/local.tmp",
        ];
        for path in paths {
            for directory in [false, true] {
                // Git also stats real directories in its oracle sandbox. Do
                // not ask it to pretend an existing directory is a file.
                if !directory && oracle.directory.path().join(path).is_dir() {
                    continue;
                }
                assert_eq!(
                    pure.ignored(Path::new(path), directory),
                    oracle.is_ignored(Path::new(path), directory).unwrap(),
                    "{path:?}, dir={directory}"
                );
            }
        }
        // Matching does not consult live files after compilation.
        fs::write(oracle.directory.path().join(".gitignore"), "normal\n").unwrap();
        assert!(!pure.ignored(Path::new("normal"), true));
    }

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
