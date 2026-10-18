use crate::{context::Context, instrumentation};
use clap::ValueEnum;
use diff_dirs_perf_runner::{
    Result,
    artifacts::{self, Control},
    execution::{Sample, Workload},
    fixture::{self, Fixture},
    scenario::{self, Scenario},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Prepare,
    Preview,
    Reseed,
    All,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Timing {
    Native,
    Instrumented,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub scenario: Scenario,
    pub phase: Phase,
    pub timing: Timing,
    pub destinations: usize,
    pub files: usize,
    pub symlinks: usize,
    pub feature_files: usize,
    pub payload_files: usize,
    pub payload_bytes: usize,
    pub require_clean_label: Option<String>,
}

pub struct Reseed {
    output: PathBuf,
    config: Config,
    jj: PathBuf,
    git: PathBuf,
    own: PathBuf,
    fixture: Option<Fixture>,
    starting: Option<Vec<Option<String>>>,
}

impl Reseed {
    pub fn new(output: &Path, config: Config, jj: PathBuf, git: PathBuf) -> Result<Self> {
        if config.destinations == 0 || config.files == 0 || config.payload_files == 0 || config.payload_bytes == 0 || config.feature_files > config.files {
            return Err("positive dimensions required; requested feature count must not exceed tracked regular files".into());
        }
        if config.phase == Phase::All && config.scenario != Scenario::Init {
            return Err("all phases models first fill; use --scenario init".into());
        }
        Ok(Self {
            output: output.into(),
            config,
            jj,
            git,
            own: std::env::current_exe()?,
            fixture: None,
            starting: None,
        })
    }
}

impl Workload for Reseed {
    fn initialize(&mut self) -> Result<Value> {
        self.output = fs::canonicalize(&self.output)?;
        let own = self.output.join("adapter");
        fs::copy(&self.own, &own)?;
        fs::set_permissions(&own, fs::Permissions::from_mode(0o555))?;
        self.own = own;
        let seed = self.output.join("payload-seed");
        fs::create_dir(&seed)?;
        for index in 0..self.config.payload_files {
            let directory = seed.join(format!("package-{:05}", index / 100));
            fs::create_dir_all(&directory)?;
            fs::write(directory.join(format!("file-{index:05}")), vec![(index % 251) as u8; self.config.payload_bytes])?;
        }
        std::os::unix::fs::symlink("package-00000/file-00000", seed.join("link"))?;
        let base = self.output.join("payload-fixture");
        fixture::setup(&base, Some(&seed))?;
        let fixture: Fixture = serde_json::from_slice(&fs::read(base.join("fixture.json"))?)?;
        self.fixture = Some(fixture);
        Ok(
            json!({"config":self.config,"scope":"native JW process incl startup; instrumented mode includes Rust exec wrappers",
            "environment":{"jj":instrumentation::identity(&self.jj)?,"git":instrumentation::identity(&self.git)?,"adapter":instrumentation::identity(&self.own)?},
            "feature_contract":"reseed creates an empty child of explicit donor goal; feature bookmarks preserve old work, not its current materialization",
            "source_identity":diff_dirs_perf_runner::source_identity()}),
        )
    }
    fn sample(&mut self, control: &Control, stem: &Path, sample: &mut Sample) -> Result<()> {
        let setup_start = Instant::now();
        let config = &self.config;
        let fixture = self.fixture.as_ref().ok_or("uninitialized payload fixture")?;
        let case = scenario::prepare(fixture, &stem.with_extension("case"), config.scenario, config.destinations)?;
        if let Some(expected) = &self.starting {
            if expected != &case.starting_equivalence {
                return Err("unmatched JW payload starting state".into());
            }
        } else {
            self.starting = Some(case.starting_equivalence.clone());
        }
        let mut ctx = Context::new(&case.root, &self.jj, &self.git, &self.own)?;
        let source = case.root.join("repo");
        fs::create_dir(&source)?;
        fs::create_dir(source.join("files"))?;
        for index in 0..config.files {
            fs::write(source.join(format!("files/file-{index:05}")), tracked(index))?;
        }
        if config.symlinks > 0 {
            fs::create_dir(source.join("links"))?;
            for index in 0..config.symlinks {
                std::os::unix::fs::symlink(
                    format!("../files/file-{:05}", index % config.files),
                    source.join(format!("links/link-{index:05}")),
                )?;
            }
        }
        fs::write(source.join(".gitignore"), "node_modules/\ndist/\n.jwreseed.local.toml\n.perf-owned-native\n")?;
        fs::write(source.join(".perf-owned-native"), source.display().to_string())?;
        fixture::clone_tree(&case.source(), &source.join("node_modules"))?;
        let install = serde_json::to_string(&vec![self.own.display().to_string(), "install".into(), source.display().to_string()])?;
        fs::write(
            source.join(".jwreseed.local.toml"),
            format!("source_at = \"dev@origin\"\ncopy_ignored = [\"node_modules\", \"**/node_modules\", \"**/dist\"]\n[freshen]\ncommand = {install}\n"),
        )?;
        // Preparation requires an explicit remote-tracking source goal. This remote is
        // NEW and local/owned; never fetch or clone from a live repository or network.
        let remote = case.root.join("remote");
        fs::rename(&source, &remote)?;
        let git = ctx.git.clone();
        ctx.run(
            &git,
            &["init".into(), "--quiet".into(), "--initial-branch=dev".into(), remote.display().to_string()],
            &case.root,
            None,
        )?;
        ctx.run(&git, &["add".into(), ".".into()], &remote, None)?;
        ctx.run(
            &git,
            &["commit".into(), "--quiet".into(), "-m".into(), "owned fixture source".into()],
            &remote,
            None,
        )?;
        ctx.jj(
            &case.root,
            &[
                "git",
                "clone",
                "--no-colocate",
                remote.to_str().ok_or("non-UTF8 local remote")?,
                source.to_str().ok_or("non-UTF8 owned source")?,
            ],
        )?;
        fixture::clone_tree(&remote.join("node_modules"), &source.join("node_modules"))?;
        for ignored in [".jwreseed.local.toml", ".perf-owned-native"] {
            fs::copy(remote.join(ignored), source.join(ignored))?;
        }
        // Root proof BEFORE workspace add: never attach fixtures to an enclosing checkout.
        ctx.root_check(&source)?;
        // Keep the cloned WC undescribed/empty: prepare intentionally refuses an
        // unpublished described child, even when its tracked tree equals the remote.
        let mut goal = ctx.jj(&source, &["log", "--no-graph", "-r", "@", "-T", "commit_id"])?;
        ctx.jj(&source, &["bookmark", "create", "performance-source", "-r", &goal])?;
        let repository = fs::canonicalize(source.join(".jj/repo"))?;
        let mut workers = Vec::new();
        let mut old_commits = Vec::new();
        let mut payload_before = Vec::new();
        for index in 0..config.destinations {
            let name = format!("worker-{index}");
            let worker = case.root.join(&name);
            ctx.jj(
                &source,
                &[
                    "workspace",
                    "add",
                    "--name",
                    &name,
                    "-r",
                    &goal,
                    worker.to_str().ok_or("non-UTF8 owned worker")?,
                ],
            )?;
            ctx.root_check(&worker)?;
            let repo_link = fs::read_to_string(worker.join(".jj/repo"))?;
            if fs::canonicalize(worker.join(".jj").join(repo_link.trim()))? != repository {
                return Err("worker belongs to the wrong JJ repository".into());
            }
            if case.target(index).exists() {
                fixture::clone_tree(&case.target(index), &worker.join("node_modules"))?;
            }
            if config.feature_files > 0 {
                for file in 0..config.feature_files {
                    OpenOptions::new()
                        .append(true)
                        .open(worker.join(format!("files/file-{file:05}")))?
                        .write_all(format!("feature {index}\n").as_bytes())?;
                }
                ctx.jj(&worker, &["describe", "-m", &format!("owned feature {index}")])?;
                ctx.jj(&worker, &["bookmark", "create", &format!("performance-feature-{index}"), "-r", "@"])?;
            }
            let actual_features = ctx.jj(&worker, &["diff", "--from", &goal, "--to", "@", "--summary"])?;
            let expected_features = (0..config.feature_files)
                .map(|file| format!("M files/file-{file:05}"))
                .collect::<Vec<_>>()
                .join("\n");
            if actual_features != expected_features {
                return Err("JJ tree does not contain the exact requested feature-file delta".into());
            }
            old_commits.push(ctx.jj(&worker, &["log", "--no-graph", "-r", "@", "-T", "commit_id"])?);
            let before = case.root.join(format!("inodes-before-{index}.jsonl"));
            write_inodes(&worker.join("node_modules"), &before)?;
            payload_before.push(if worker.join("node_modules").exists() {
                Some(fixture::tree(&worker.join("node_modules"), true, true)?)
            } else {
                None
            });
            workers.push((name, worker));
        }
        let setup_ns = setup_start.elapsed().as_nanos();
        sample.preparation_ns += setup_ns;
        sample.details["setup"] =
            json!({"case":case,"old_commits":old_commits,"goal":goal,"feature_files_per_target":config.feature_files,"environment":ctx.identities()?});
        let mut phases = serde_json::Map::new();
        let mut measured = 0;
        let mut process_total = 0;
        let mut verification_ns = 0;
        if config.phase == Phase::Prepare || config.phase == Phase::All {
            let (text, result) = phase(
                &mut ctx,
                control,
                &source,
                "prepare",
                vec!["prepare".into(), "--format=json".into()],
                config.timing,
            )?;
            measured += result["process"]["elapsed_ns"].as_u64().ok_or("missing preparation elapsed time")? as u128;
            process_total = measured;
            goal = serde_json::from_str::<Value>(&text)?["source_commit_id"]
                .as_str()
                .ok_or("prepare omitted immutable source ID")?
                .to_owned();
            phases.insert("prepare".into(), result);
            sample.details["phases"] = json!(&phases);
            sample.process_ns = measured;
            let start = Instant::now();
            if fs::read_to_string(source.join("node_modules/__install_count"))? != "1" {
                return Err("owned installer did not run exactly once".into());
            }
            verify_unchanged(&mut ctx, &workers, &old_commits, &payload_before)?;
            verification_ns += start.elapsed().as_nanos();
        }
        let mut args = vec!["reseed".into()];
        args.extend(workers.iter().map(|(name, _)| name.clone()));
        args.extend([
            "--from".into(),
            source.display().to_string(),
            "--source-at".into(),
            goal.clone(),
            "--at".into(),
            goal.clone(),
        ]);
        sample.argv = args.clone();
        if config.phase == Phase::Preview || config.phase == Phase::All {
            let mut preview = args.clone();
            preview.push("--dry-run".into());
            let (_, result) = phase(&mut ctx, control, &source, "preview", preview, config.timing)?;
            measured += result["process"]["elapsed_ns"].as_u64().ok_or("missing preview elapsed time")? as u128;
            process_total = measured;
            phases.insert("preview".into(), result);
            sample.details["phases"] = json!(&phases);
            sample.process_ns = measured;
            let start = Instant::now();
            verify_unchanged(&mut ctx, &workers, &old_commits, &payload_before)?;
            verification_ns += start.elapsed().as_nanos();
        }
        if config.phase == Phase::Reseed || config.phase == Phase::All {
            let (_, result) = phase(&mut ctx, control, &source, "reseed", args, config.timing)?;
            measured += result["process"]["elapsed_ns"].as_u64().ok_or("missing reseed elapsed time")? as u128;
            process_total = measured;
            phases.insert("reseed".into(), result);
            sample.details["phases"] = json!(&phases);
            sample.process_ns = measured;
            let verify = Instant::now();
            let expected = fixture::payload(&source.join("node_modules"))?;
            let mut rewrites = Vec::new();
            for (index, (_, worker)) in workers.iter().enumerate() {
                ctx.root_check(worker)?;
                if fixture::payload(&worker.join("node_modules"))? != expected {
                    return Err("reseed payload bytes/link/mode/file-mtime mismatch".into());
                }
                // The BEFORE source and retained logs explicitly specify this empty-child-of-goal contract.
                if !ctx.jj(worker, &["diff", "--from", &goal, "--to", "@", "--summary"])?.is_empty() {
                    return Err("tracked final tree differs from requested donor goal".into());
                }
                if ctx.jj(worker, &["--ignore-working-copy", "log", "--no-graph", "-r", "@-", "-T", "commit_id"])? != goal {
                    return Err("final working copy is not a child of the exact goal".into());
                }
                if config.feature_files > 0 {
                    let bookmark = format!("performance-feature-{index}");
                    if ctx.jj(worker, &["--ignore-working-copy", "log", "--no-graph", "-r", &bookmark, "-T", "commit_id"])? != old_commits[index] {
                        return Err("old feature history was not preserved".into());
                    }
                    for file in 0..config.feature_files {
                        let data = ctx.jj(worker, &["file", "show", "-r", &bookmark, &format!("files/file-{file:05}")])?;
                        if data != format!("{}feature {index}", String::from_utf8(tracked(file))?) {
                            return Err("feature history content mismatch".into());
                        }
                    }
                }
                let changes = inode_changes(&worker.join("node_modules"), &case.root.join(format!("inodes-before-{index}.jsonl")))?;
                rewrites.push(changes.clone());
                sample.details["payload_changes"] = json!(&rewrites);
                if config.scenario == Scenario::Clean
                    && config.phase == Phase::Reseed
                    && config.require_clean_label.as_deref() == Some(&control.label)
                    && (changes["inode_changes"] != 0 || changes["mtime_changes"] != 0 || changes["added"] != 0 || changes["removed"] != 0)
                {
                    return Err("incremental candidate violated clean payload identity invariant".into());
                }
            }
            sample.details["payload_changes"] = json!(rewrites);
            verification_ns += verify.elapsed().as_nanos();
        }
        if config.phase == Phase::All && fs::read_to_string(source.join("node_modules/__install_count"))? != "1" {
            return Err("preview/reseed reran the installer".into());
        }
        sample.details["phases"] = Value::Object(phases);
        sample.details["goal"] = json!(goal);
        sample.native_ns = Some(measured);
        sample.process_ns = process_total;
        sample.verification_ns = verification_ns;
        sample.exit_code = Some(0);
        Ok(())
    }
    fn finish(&mut self) -> Result<Value> {
        let fixture = self.fixture.as_ref().ok_or("uninitialized fixture")?;
        fixture::verify(&fixture.root, fixture, true)?;
        Ok(json!({"payload_fixture_unchanged":true}))
    }
}

fn tracked(index: usize) -> Vec<u8> {
    format!("tracked deterministic file {index}\n").into_bytes()
}

pub fn install(root: &Path) -> Result<()> {
    if fs::canonicalize(root)?.display().to_string() != fs::read_to_string(root.join(".perf-owned-native"))? {
        return Err("installer requires its original owned fixture root".into());
    }
    let count = root.join("node_modules/__install_count");
    let n = match fs::read_to_string(&count) {
        Ok(value) => value.parse::<u64>()?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
        Err(e) => return Err(e.into()),
    };
    fs::write(count, (n + 1).to_string())?;
    Ok(())
}

fn phase(ctx: &mut Context, control: &Control, source: &Path, name: &str, args: Vec<String>, timing: Timing) -> Result<(String, Value)> {
    artifacts::verify(control)?;
    let trace = ctx.root.join(format!("{name}.processes.jsonl"));
    if timing == Timing::Instrumented {
        OpenOptions::new().write(true).create_new(true).open(&trace)?;
    }
    let (text, process) = ctx.run(&control.binary, &args, source, if timing == Timing::Instrumented { Some(&trace) } else { None })?;
    let counts = if timing == Timing::Instrumented {
        instrumentation::counts(&trace)?
    } else {
        Value::Null
    };
    Ok((text, json!({"argv":args,"process":process,"counts":counts,"timing":timing})))
}

fn verify_unchanged(ctx: &mut Context, workers: &[(String, PathBuf)], old: &[String], payload: &[Option<String>]) -> Result<()> {
    for (index, (_, worker)) in workers.iter().enumerate() {
        ctx.root_check(worker)?;
        if ctx.jj(worker, &["log", "--no-graph", "-r", "@", "-T", "commit_id"])? != old[index] {
            return Err("prepare/preview changed a worker commit".into());
        }
        let actual = if worker.join("node_modules").exists() {
            Some(fixture::tree(&worker.join("node_modules"), true, true)?)
        } else {
            None
        };
        if actual != payload[index] {
            return Err("prepare/preview mutated worker payload".into());
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Inode {
    path: String,
    ino: u64,
    mtime: (i64, i64),
}

fn regular(root: &Path, path: &Path, visit: &mut impl FnMut(Inode) -> Result<()>) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(path)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let kind = entry.file_type()?;
        if kind.is_dir() {
            regular(root, &entry.path(), visit)?;
        } else if kind.is_file() {
            let m = entry.metadata()?;
            visit(Inode {
                path: entry.path().strip_prefix(root)?.to_str().ok_or("non-UTF8 payload")?.into(),
                ino: m.ino(),
                mtime: (m.mtime(), m.mtime_nsec()),
            })?;
        }
    }
    Ok(())
}

fn write_inodes(root: &Path, output: &Path) -> Result<()> {
    let mut out = OpenOptions::new().write(true).create_new(true).open(output)?;
    regular(root, root, &mut |row| {
        serde_json::to_writer(&mut out, &row)?;
        out.write_all(b"\n")?;
        Ok(())
    })
}

fn inode_changes(root: &Path, before: &Path) -> Result<Value> {
    let mut rows = BufReader::new(File::open(before)?)
        .lines()
        .map(|line| -> Result<Inode> { Ok(serde_json::from_str(&line?)?) })
        .peekable();
    let (mut added, mut removed, mut inode, mut mtime, mut common) = (0u64, 0u64, 0u64, 0u64, 0u64);
    regular(root, root, &mut |now| {
        while rows.peek().is_some_and(|row| {
            row.as_ref()
                .is_ok_and(|old| Path::new(&old.path).components().cmp(Path::new(&now.path).components()).is_lt())
        }) {
            rows.next().transpose()?;
            removed += 1;
        }
        if rows.peek().is_some_and(|row| row.as_ref().is_err()) {
            rows.next().transpose()?;
        }
        if rows.peek().is_some_and(|row| row.as_ref().is_ok_and(|old| old.path == now.path)) {
            let old = rows.next().transpose()?.ok_or("inode stream ended")?;
            common += 1;
            if old.ino != now.ino {
                inode += 1;
            }
            if old.mtime != now.mtime {
                mtime += 1;
            }
        } else {
            added += 1;
        }
        Ok(())
    })?;
    for row in rows {
        row?;
        removed += 1;
    }
    Ok(json!({"common_files":common,"inode_changes":inode,"mtime_changes":mtime,"added":added,"removed":removed}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streaming_inode_merge_handles_directory_prefix_order() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-perf-unit");
        fs::create_dir_all(&root).unwrap();
        let own = tempfile::tempdir_in(root).unwrap();
        let payload = own.path().join("payload");
        fs::create_dir(&payload).unwrap();
        fs::create_dir(payload.join("a")).unwrap();
        fs::write(payload.join("a/child"), b"data").unwrap();
        fs::write(payload.join("a.ext"), b"data").unwrap();
        let before = own.path().join("before.jsonl");
        write_inodes(&payload, &before).unwrap();
        let same = inode_changes(&payload, &before).unwrap();
        assert_eq!(same["common_files"], 2);
        assert_eq!(same["inode_changes"], 0);
        assert_eq!(same["added"], 0);
        assert_eq!(same["removed"], 0);
        fs::rename(payload.join("a/child"), payload.join("added")).unwrap();
        let changes = inode_changes(&payload, &before).unwrap();
        assert_eq!(changes["common_files"], 1);
        assert_eq!(changes["added"], 1);
        assert_eq!(changes["removed"], 1);
    }
    #[test]
    fn excessive_feature_request_is_rejected_before_fixture_creation() {
        let config = Config {
            scenario: Scenario::Clean,
            phase: Phase::Reseed,
            timing: Timing::Native,
            destinations: 2,
            files: 1,
            symlinks: 0,
            feature_files: 2,
            payload_files: 2,
            payload_bytes: 16,
            require_clean_label: None,
        };
        assert!(Reseed::new(Path::new("unused"), config, "/native/jj".into(), "/native/git".into()).is_err());
    }
}
