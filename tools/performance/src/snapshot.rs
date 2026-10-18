//! Read-only donor inventory + selected filesystem copy, never a live JJ/JW snapshot.
use diff_dirs_perf_runner::{Result, fixture, process, write_json};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    time::Duration,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub donor: PathBuf,
    #[serde(default)]
    pub inventory_operation: String,
    pub inventory_commit: String,
    pub tracked: Vec<PathBuf>,
    pub payload_roots: Vec<PathBuf>,
    pub contract: String,
}

pub fn safe(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path.components().any(|p| !matches!(p, Component::Normal(_)))
        || path.components().any(|p| p.as_os_str() == ".git" || p.as_os_str() == ".jj")
    {
        return Err(format!("unsafe donor-relative path {}", path.display()).into());
    }
    Ok(())
}

fn discover(root: &Path, path: &Path, selected: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(path)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        if entry.file_name() == ".git" || entry.file_name() == ".jj" {
            continue;
        }
        let relative = entry.path().strip_prefix(root)?.to_owned();
        let kind = entry.file_type()?;
        // Native ** consumes zero components too (so a root dist is eligible).
        let chosen = entry.file_name() == "node_modules" || entry.file_name() == "dist";
        if chosen {
            if !kind.is_dir() {
                return Err(format!("selected payload root must be a real directory: {}", relative.display()).into());
            }
            selected.push(relative);
        }
        if kind.is_dir() {
            discover(root, &entry.path(), selected)?;
        }
    }
    Ok(())
}

pub fn import(donor: &Path, output: &Path, jj: &Path) -> Result<()> {
    let _lock = diff_dirs_perf_runner::host_lock()?;
    let donor = fs::canonicalize(donor)?;
    let output = fixture::fresh_path(output)?;
    if output.starts_with(&donor) {
        return Err("import output must be outside live donor".into());
    }
    fs::create_dir(&output)?;
    let query = |args: &[String], stem: &str| -> Result<String> {
        let mut cmd = Command::new(jj);
        cmd.args(["--ignore-working-copy", "--repository"]).arg(&donor).args(args).current_dir(&donor);
        let result = process::run(&mut cmd, &output.join(stem), Duration::from_secs(180))?;
        if result.code != Some(0) || result.timed_out {
            return Err("read-only donor inventory failed".into());
        }
        Ok(fs::read_to_string(output.join(format!("{stem}.stdout")))?.trim_end().to_string())
    };
    if query(&["root".into()], "root")? != donor.display().to_string() {
        return Err("donor is not its explicit JJ workspace root".into());
    }
    let operation = query(
        &[
            "op".into(),
            "log".into(),
            "--no-graph".into(),
            "--limit".into(),
            "1".into(),
            "-T".into(),
            "id".into(),
        ],
        "operation",
    )?;
    let commit = query(
        &[
            "--at-operation".into(),
            operation.clone(),
            "log".into(),
            "--no-graph".into(),
            "-r".into(),
            "@".into(),
            "-T".into(),
            "commit_id".into(),
        ],
        "commit",
    )?;
    query(
        &[
            "--at-operation".into(),
            operation.clone(),
            "file".into(),
            "list".into(),
            "-r".into(),
            commit.clone(),
        ],
        "tracked",
    )?;
    let list = fs::read_to_string(output.join("tracked.stdout"))?;
    let mut tracked: Vec<PathBuf> = list.lines().map(PathBuf::from).collect();
    tracked.sort();
    tracked.dedup();
    let mut candidates = Vec::new();
    discover(&donor, &donor, &mut candidates)?;
    candidates.sort();
    if tracked.is_empty() || candidates.is_empty() {
        return Err("import needs tracked files and selected payload candidates".into());
    }
    for path in tracked.iter().chain(&candidates) {
        safe(path)?;
        if !fs::canonicalize(donor.join(path).parent().ok_or("missing parent")?)?.starts_with(&donor) {
            return Err("donor path follows an escaping parent symlink".into());
        }
    }
    for path in &tracked {
        let kind = fs::symlink_metadata(donor.join(path))?.file_type();
        if !kind.is_file() && !kind.is_symlink() {
            return Err("tracked directories/submodules are not an ordinary tracked-file fixture".into());
        }
        if path == Path::new(".jwreseed.local.toml") || path == Path::new(".perf-owned-native") {
            return Err("tracked fixture-configuration collision; refusing to execute/overwrite donor actions".into());
        }
    }
    let seed = output.join("seed");
    fs::create_dir(&seed)?;
    copy(&donor, &seed, &tracked, &output.join("tracked-copy"))?;
    // Match the copied tracked .gitignore bytes in an isolated index. Never use a
    // live Git index or silently treat merely named node_modules/dist as eligible.
    let gitdir = output.join("probe.git");
    let git = || {
        let mut c = Command::new("/usr/bin/git");
        c.env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .arg("--git-dir")
            .arg(&gitdir)
            .arg("--work-tree")
            .arg(&seed)
            .current_dir(&seed);
        c
    };
    let checked = |mut c: Command, stem: &str, input: Option<&Path>| -> Result<i32> {
        let p = process::run_input(&mut c, &output.join(stem), Duration::from_secs(180), input)?;
        p.code
            .filter(|code| !p.timed_out && (*code == 0 || *code == 1))
            .ok_or_else(|| "isolated ignore eligibility probe failed".into())
    };
    let mut c = git();
    c.arg("init").arg("--quiet");
    if checked(c, "probe-init", None)? != 0 {
        return Err("isolated index init failed".into());
    }
    let listfile = output.join("tracked.nul");
    write_nul(&listfile, &tracked, false)?;
    let mut c = git();
    c.args(["add", "--force", "--pathspec-from-file"]).arg(&listfile).arg("--pathspec-file-nul");
    if checked(c, "probe-add", None)? != 0 {
        return Err("isolated tracked index failed".into());
    }
    let queries = output.join("candidates.nul");
    write_nul(&queries, &candidates, true)?;
    let mut c = git();
    c.args(["check-ignore", "--stdin", "-z"]);
    checked(c, "probe-ignored", Some(&queries))?;
    let data = fs::read(output.join("probe-ignored.stdout"))?;
    let mut ignored = data
        .split(|b| *b == 0)
        .filter(|b| !b.is_empty())
        .map(|b| Ok(PathBuf::from(std::str::from_utf8(b)?.trim_end_matches('/'))))
        .collect::<Result<Vec<_>>>()?;
    ignored.sort_by_key(|p| (p.components().count(), p.clone()));
    let mut roots = Vec::<PathBuf>::new();
    for path in ignored {
        if roots.iter().any(|root| path.starts_with(root)) {
            continue;
        }
        if tracked.iter().any(|file| file.starts_with(&path)) {
            return Err(format!("ignored root {} contains tracked work; source payload contract forbids overlap", path.display()).into());
        }
        fixture::tree(&donor.join(&path), false, false)?;
        roots.push(path);
    }
    roots.sort_by(|a, b| a.components().cmp(b.components()));
    if roots.is_empty() {
        return Err("no eligible ignored payload roots under tracked rules".into());
    }
    copy(&donor, &seed, &roots, &output.join("payload-copy"))?;
    let base = output.join("fixture");
    fixture::setup(&base, Some(&seed))?;
    let snapshot = Snapshot {
        donor,
        inventory_operation: operation,
        inventory_commit: commit,
        tracked,
        payload_roots: roots,
        contract: "read-only, non-quiesced disk copy; commit pins tracked PATH inventory, not live file bytes; no admin state or donor actions copied".into(),
    };
    write_json(&base.join("snapshot.json"), &snapshot)?;
    println!("{}", base.display());
    Ok(())
}

fn write_nul(path: &Path, paths: &[PathBuf], directory: bool) -> Result<()> {
    let mut bytes = Vec::new();
    for p in paths {
        bytes.extend_from_slice(p.to_str().ok_or("non-UTF8 donor path")?.as_bytes());
        if directory {
            bytes.push(b'/');
        }
        bytes.push(0);
    }
    fs::write(path, bytes)?;
    Ok(())
}
fn copy(donor: &Path, seed: &Path, paths: &[PathBuf], stem: &Path) -> Result<()> {
    let mut cp = Command::new("/usr/bin/cp");
    cp.current_dir(donor)
        .args(["-a", "--reflink=always", "--parents", "--target-directory"])
        .arg(seed)
        .arg("--")
        .args(paths);
    let p = process::run(&mut cp, stem, Duration::from_secs(1800))?;
    if p.code != Some(0) || p.timed_out {
        return Err("private reflink import failed; partial output retained".into());
    }
    Ok(())
}

#[test]
fn nested_roots_are_distinct_and_never_follow_links() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-perf-unit");
    fs::create_dir_all(&root).unwrap();
    let own = tempfile::tempdir_in(root).unwrap();
    for path in ["node_modules/a", "packages/a/node_modules/b", "packages/a/dist/chunk", ".jj/admin"] {
        fs::create_dir_all(own.path().join(path)).unwrap();
    }
    let mut roots = Vec::new();
    discover(own.path(), own.path(), &mut roots).unwrap();
    assert_eq!(roots.len(), 3);
    assert!(roots.contains(&PathBuf::from("packages/a/dist")));
    assert!(safe(Path::new("../escape")).is_err());
    assert!(safe(Path::new("x/.jj/repo")).is_err());
}
