//! Local remotes and argv installers exercise source-only preparation and the
//! persisted v1/v2/v3 recovery contract. No network or operator checkout is used.
#![cfg(unix)]
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

fn text(output: &Output) -> String {
    format!(
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}
fn ok(output: Output) -> String {
    assert!(output.status.success(), "{}", text(&output));
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn fails(output: Output, expected: &str) -> String {
    assert!(!output.status.success(), "{}", text(&output));
    let output = text(&output);
    assert!(output.contains(expected), "expected {expected:?}: {output}");
    output
}

struct Fixture {
    temp: TempDir,
    remote: PathBuf,
    source: PathBuf,
    target: PathBuf,
    config: PathBuf,
    installer: PathBuf,
    log: PathBuf,
    baseline: String,
}
impl Fixture {
    fn new() -> Option<Self> {
        Self::with_colocation(false)
    }
    fn with_colocation(colocated: bool) -> Option<Self> {
        if !Command::new("jj")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
        {
            assert!(
                std::env::var_os("JW_TEST_REQUIRE_COW").is_none(),
                "JJ required"
            );
            return None;
        }
        let scratch = std::env::current_dir()
            .unwrap()
            .join(".test-agent/prepare-tests");
        fs::create_dir_all(&scratch).unwrap();
        let temp = tempfile::tempdir_in(scratch).unwrap();
        let mut f = Self {
            remote: temp.path().join("local remote"),
            source: temp.path().join("donor with spaces"),
            target: temp.path().join("target"),
            config: temp.path().join("config"),
            installer: temp.path().join("fake installer"),
            log: temp.path().join("argv log"),
            baseline: String::new(),
            temp,
        };
        fs::create_dir_all(&f.config).unwrap();
        fs::create_dir(&f.remote).unwrap();
        ok(f.command("git", &f.remote)
            .args(["init", "--initial-branch=dev"])
            .output()
            .unwrap());
        fs::write(
            f.remote.join(".gitignore"),
            "node_modules/\n.jwreseed.local.toml\n.jwlinks.local.toml\nshared-link\n.env\n",
        )
        .unwrap();
        fs::write(f.remote.join("shared"), vec![b'x'; 128 * 1024]).unwrap();
        fs::write(f.remote.join("version"), "old\n").unwrap();
        fs::write(f.remote.join("lockfile"), "frozen\n").unwrap();
        f.git_commit();
        ok(f.command("jj", f.temp.path())
            .args([
                "git",
                "clone",
                if colocated {
                    "--colocate"
                } else {
                    "--no-colocate"
                },
            ])
            .arg(&f.remote)
            .arg(&f.source)
            .output()
            .unwrap());
        f.baseline = f.id(&f.source, "dev@origin");
        f.jj(
            &f.source,
            &[
                "workspace",
                "add",
                "--name",
                "worker",
                "-r",
                &f.baseline,
                f.target.to_str().unwrap(),
            ],
        );
        f.script("mkdir -p node_modules\nprintf 'installed\\n' > node_modules/package\n");
        f.policy(true, "");
        Some(f)
    }
    fn command(&self, program: &str, cwd: &Path) -> Command {
        let mut c = Command::new(program);
        c.current_dir(cwd)
            .env("XDG_CONFIG_HOME", &self.config)
            .env("JJ_CONFIG", "")
            .env("JJ_USER", "Prepare Test")
            .env("JJ_EMAIL", "prepare@example.invalid")
            .env("JJ_EDITOR", "true")
            .env("GIT_AUTHOR_NAME", "Prepare Test")
            .env("GIT_AUTHOR_EMAIL", "prepare@example.invalid")
            .env("GIT_COMMITTER_NAME", "Prepare Test")
            .env("GIT_COMMITTER_EMAIL", "prepare@example.invalid");
        if program == "jj" {
            c.args(["--config", "git.colocate=false"]);
        }
        c
    }
    fn jj(&self, cwd: &Path, args: &[&str]) -> String {
        ok(self.command("jj", cwd).args(args).output().unwrap())
    }
    fn id(&self, cwd: &Path, rev: &str) -> String {
        self.jj(
            cwd,
            &[
                "log",
                "--ignore-working-copy",
                "--no-graph",
                "-r",
                rev,
                "-T",
                "commit_id",
            ],
        )
    }
    fn git_commit(&self) {
        ok(self
            .command("git", &self.remote)
            .args(["add", "."])
            .output()
            .unwrap());
        ok(self
            .command("git", &self.remote)
            .args(["commit", "-m", "local fixture advance"])
            .output()
            .unwrap());
    }
    fn advance(&self) -> String {
        fs::write(self.remote.join("version"), "fresh\n").unwrap();
        self.git_commit();
        ok(self
            .command("git", &self.remote)
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap())
    }
    fn script(&self, body: &str) {
        fs::write(
            &self.installer,
            format!(
                "#!/bin/sh\nset -eu\npwd >> \"$1\"\nprintf '%s\\n' \"$2\" \"$3\" >> \"$1\"\n{body}"
            ),
        )
        .unwrap();
        fs::set_permissions(&self.installer, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fn argv(&self) -> Vec<String> {
        vec![
            self.installer.to_str().unwrap().into(),
            self.log.to_str().unwrap().into(),
            "argument with spaces".into(),
            "$(touch SHOULD_NOT_EXIST); *".into(),
        ]
    }
    fn policy(&self, command: bool, extra: &str) {
        fs::write(
            self.source.join(".jwreseed.local.toml"),
            format!(
                "source_at='dev@origin'\n{extra}\n[freshen]\nremote='origin'\n{}",
                if command {
                    format!("command={}\n", serde_json::to_string(&self.argv()).unwrap())
                } else {
                    String::new()
                }
            ),
        )
        .unwrap();
    }
    fn jw(&self) -> Command {
        let env = self.command("jw", &self.target);
        let mut c = Command::new(assert_cmd::cargo::cargo_bin!("jw"));
        c.current_dir(&self.target)
            .envs(env.get_envs().filter_map(|(k, v)| v.map(|v| (k, v))));
        c
    }
    fn prepare(&self, extra: &[&str]) -> Output {
        self.jw()
            .args(["prepare", "--copy-ignored", "node_modules"])
            .args(extra)
            .output()
            .unwrap()
    }
    fn resume(&self) -> Output {
        self.jw().args(["prepare", "--resume"]).output().unwrap()
    }
    fn journal(&self) -> PathBuf {
        PathBuf::from(self.jj(&self.source, &["config", "path", "--repo"]))
            .parent()
            .unwrap()
            .join("jj-waltz/reseed")
    }
    fn records(&self, suffix: &str) -> Vec<PathBuf> {
        fs::read_dir(self.journal())
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.file_name().unwrap().to_string_lossy().ends_with(suffix))
            .collect()
    }
    fn active(&self) -> PathBuf {
        let paths = self.records(".preparation.json");
        assert_eq!(paths.len(), 1);
        paths[0].clone()
    }
    fn receipt(&self) -> Value {
        self.records(".receipt.json")
            .into_iter()
            .map(|p| read_json(&p))
            .find(|v| v["plans"].is_array())
            .unwrap()
    }
    fn reactivate(&self, record: &Value) -> PathBuf {
        let digest = Sha256::digest(format!("source:{}", self.source.display()).as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let path = self
            .journal()
            .join(format!("workspace-{digest}.preparation.json"));
        write_json(&path, record);
        path
    }
    fn cow_supported(&self) -> bool {
        let probe = self.target.join("probe");
        match reflink_copy::reflink(self.source.join("shared"), &probe) {
            Ok(()) => {
                fs::remove_file(probe).unwrap();
                true
            }
            Err(e) => {
                assert!(
                    std::env::var_os("JW_TEST_REQUIRE_COW").is_none(),
                    "CoW required: {e}"
                );
                assert!(matches!(e.raw_os_error(), Some(18 | 22 | 38 | 95)), "{e}");
                false
            }
        }
    }
    fn wrapper(&self, condition: &str, before: bool) -> PathBuf {
        let real = std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|p| p.join("jj"))
            .find(|p| p.is_file())
            .unwrap();
        let bin = self.temp.path().join("wrapper-bin");
        fs::create_dir_all(&bin).unwrap();
        let marker = self.temp.path().join("wrapper fired");
        let failure =
            format!("if [ ! -f {marker:?} ] && {condition}; then touch {marker:?}; exit 97; fi\n");
        let delegate = format!("{real:?} \"$@\"\n");
        fs::write(bin.join("jj"), format!("#!/bin/sh\nset -eu\nnew=false; clear=false; fetch=false; diff=false\nfor arg do\n[ \"$arg\" != new ] || new=true\n[ \"$arg\" != --clear ] || clear=true\n[ \"$arg\" != fetch ] || fetch=true\n[ \"$arg\" != diff ] || diff=true\ndone\n{}{}", if before { &failure } else { &delegate }, if before { &delegate } else { &failure })).unwrap();
        fs::set_permissions(bin.join("jj"), fs::Permissions::from_mode(0o755)).unwrap();
        bin
    }
    fn with_wrapper(&self, c: &mut Command, bin: &Path) {
        c.env(
            "PATH",
            std::env::join_paths(
                std::iter::once(bin.to_path_buf())
                    .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
            )
            .unwrap(),
        );
    }
    /// Reconstruct the old wire contract from a stopped source preparation and
    /// explicit frozen worker facts, rather than keeping the deleted command alive.
    fn legacy(&self, version: u32, explicit_base: bool) -> PathBuf {
        self.jj(&self.target, &["status"]);
        let path = self.active();
        let mut record = read_json(&path);
        record["schema_version"] = version.into();
        record["request"].as_object_mut().unwrap().remove("prepare");
        record["request"]["names"] = json!(["worker"]);
        record["request"]["up"] = (version == 3).into();
        record["request"]["freshen"] = (version < 3).into();
        if explicit_base {
            record["request"]["at"] = self.baseline.clone().into();
        }
        if version < 3 {
            record.as_object_mut().unwrap().remove("source_selection");
        }
        if version == 1 {
            record["command"] = record["command_sequence"]["argv"][0].clone();
            record.as_object_mut().unwrap().remove("command_sequence");
        }
        record["targets"] = json!([{
            "name": "worker", "root": self.target, "root_identity": identity(&self.target),
            "jj_identity": identity(&self.target.join(".jj")), "commit": self.id(&self.target, "@"),
            "stamp": disk_stamp(&self.target),
        }]);
        record["plans"] = Value::Null;
        write_json(&path, &record);
        path
    }
}
fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}
fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}
fn identity(path: &Path) -> Value {
    let m = fs::metadata(path).unwrap();
    json!({"device": m.dev(), "inode": m.ino()})
}
// Explicit legacy fixture wire representation, including ordering and administrative exclusions.
fn disk_stamp(root: &Path) -> Value {
    fn visit(root: &Path, dir: &Path, result: &mut Vec<Value>) {
        let mut children = fs::read_dir(dir)
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        children.sort_by_key(|entry| entry.file_name());
        for entry in children {
            if dir == root
                && [".jj", ".git"]
                    .iter()
                    .any(|name| entry.file_name() == *name)
            {
                continue;
            }
            let path = entry.path();
            let m = fs::symlink_metadata(&path).unwrap();
            result.push(json!({"path": path.strip_prefix(root).unwrap(), "device": m.dev(), "inode": m.ino(), "size": m.len(), "mode": m.mode(), "mtime": m.mtime(), "mtime_ns": m.mtime_nsec(), "ctime": m.ctime(), "ctime_ns": m.ctime_nsec()}));
            if m.is_dir() {
                visit(root, &path, result);
            }
        }
    }
    let mut result = Vec::new();
    visit(root, root, &mut result);
    json!(result)
}

#[test]
fn prepare_from_primary_or_dirty_worker_never_selects_or_snapshots_workers() {
    for primary in [false, true] {
        let Some(f) = Fixture::new() else {
            return;
        };
        let old_target = f.id(&f.target, "@");
        fs::write(f.target.join("target-work"), "unsnapshotted feature\n").unwrap();
        fs::write(f.target.join("version"), "dirty worker\n").unwrap();
        let before = disk_stamp(&f.target);
        let goal = f.advance();
        let output = ok(f
            .jw()
            .current_dir(if primary { &f.source } else { &f.target })
            .args(["prepare", "--copy-ignored", "node_modules"])
            .output()
            .unwrap());
        assert!(output.contains(&goal), "{output}");
        assert!(!output.contains("OK worker"));
        assert_eq!(f.id(&f.target, "@"), old_target);
        assert_eq!(disk_stamp(&f.target), before);
        assert_eq!(f.id(&f.source, "@-"), goal);
        assert!(!f.target.join("node_modules").exists());
        let receipt = f.receipt();
        assert_eq!(receipt["schema_version"], 4);
        assert_eq!(receipt["request"]["names"], json!([]));
        assert_eq!(receipt["targets"], json!([]));
        assert_eq!(receipt["plans"], json!([]));
        assert_eq!(receipt["goal"], goal);
        assert_eq!(
            fs::read_to_string(&f.log)
                .unwrap()
                .lines()
                .collect::<Vec<_>>(),
            [
                f.source.to_str().unwrap(),
                "argument with spaces",
                "$(touch SHOULD_NOT_EXIST); *"
            ]
        );
        assert!(!f.source.join("SHOULD_NOT_EXIST").exists());
        assert!(f.records(".preparation.json").is_empty() && f.records(".pending.json").is_empty());
    }
}

#[test]
fn json_reports_frozen_commit_without_installer_stdout_contamination() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.script("echo installer-output\n");
    let goal = f.advance();
    let output = f.prepare(&["--format", "json"]);
    assert!(String::from_utf8_lossy(&output.stderr).contains("installer-output"));
    let report: Value = serde_json::from_str(&ok(output)).unwrap();
    assert_eq!(report["source_commit_id"], goal);
    assert_eq!(report["targets"], json!([]));
}

#[test]
fn preparation_commands_leave_new_nonignored_outputs_untracked() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.script("printf output > new-build-output\n");
    let goal = f.advance();
    ok(f.prepare(&[]));
    assert_eq!(
        fs::read_to_string(f.source.join("new-build-output")).unwrap(),
        "output"
    );
    assert_eq!(f.id(&f.source, "@-"), goal);
    let files = f.jj(
        &f.source,
        &["--ignore-working-copy", "file", "list", "-r", "@"],
    );
    assert!(!files.contains("new-build-output"));
    assert!(!f.target.join("new-build-output").exists());
}

#[test]
fn dry_run_selects_native_upstream_without_fetch_install_or_worker_snapshot() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.advance();
    fs::write(f.target.join("unsnapshotted"), "work").unwrap();
    let worker = f.id(&f.target, "@");
    let source = f.id(&f.source, "@");
    let output = ok(f.prepare(&["--dry-run"]));
    assert!(
        output.contains("source-only prepare")
            && output.contains("post-fetch exact commit unknown")
    );
    assert_eq!(f.id(&f.source, "@"), source);
    assert_eq!(f.id(&f.target, "@"), worker);
    assert_eq!(f.id(&f.source, "dev@origin"), f.baseline);
    assert!(!f.log.exists());
    assert!(f.records(".preparation.json").is_empty());
}

#[test]
fn native_non_origin_and_untracked_exact_lines_are_selected_without_extra_fetches() {
    for tracked in [true, false] {
        let Some(f) = Fixture::new() else {
            return;
        };
        fs::remove_file(f.source.join(".jwreseed.local.toml")).unwrap();
        f.jj(&f.source, &["git", "remote", "rename", "origin", "native"]);
        if !tracked {
            f.jj(&f.source, &["bookmark", "untrack", "dev@native"]);
        }
        let goal = f.advance();
        let target = f.id(&f.target, "@");
        ok(f.command("git", &f.remote)
            .args(["branch", "unselected"])
            .output()
            .unwrap());
        ok(f.prepare(&[]));
        assert_eq!(f.id(&f.source, "dev@native"), goal);
        assert_eq!(f.id(&f.target, "@"), target);
        assert!(
            !f.jj(&f.source, &["bookmark", "list", "--all-remotes"])
                .contains("unselected")
        );
        assert_eq!(
            f.receipt()["source_selection"]["kind"],
            if tracked { "Tracked" } else { "ExactRemote" }
        );
    }
}

#[test]
fn source_unpublished_work_blocks_before_fetch_even_above_empty_scratch() {
    for committed in [false, true] {
        let Some(f) = Fixture::new() else {
            return;
        };
        fs::write(f.source.join("local-source"), "unpublished\n").unwrap();
        if committed {
            f.jj(&f.source, &["describe", "-m", "unpublished source"]);
            f.jj(&f.source, &["new"]);
        }
        f.advance();
        fails(f.prepare(&[]), "source contains unpublished work");
        assert_eq!(f.id(&f.source, "dev@origin"), f.baseline);
        assert!(!f.log.exists());
    }
}

#[test]
fn preparation_failure_resumes_from_any_workspace_without_refetch_or_worker_checks() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let gate = f.temp.path().join("allow success");
    f.script(&format!("[ -f {gate:?} ] || exit 17\n"));
    let goal = f.advance();
    let failed = fails(f.prepare(&[]), "command failed");
    assert!(failed.contains("use `jw prepare --resume`"), "{failed}");
    let path = f.active();
    let frozen = fs::read(&path).unwrap();
    fs::write(f.target.join("version"), "concurrent target edit\n").unwrap();
    let stamp = disk_stamp(&f.target);
    let preview = ok(f
        .jw()
        .current_dir(&f.source)
        .args(["prepare", "--resume", "--dry-run"])
        .output()
        .unwrap());
    assert!(preview.contains("commands completed 0/1"));
    assert_eq!(fs::read(&path).unwrap(), frozen);
    let blocked = fails(
        f.jw().args(["reseed", "worker"]).output().unwrap(),
        "active preparation batch",
    );
    assert!(blocked.contains("use `jw prepare --resume`"), "{blocked}");
    let retry = fails(f.prepare(&[]), "active preparation batch");
    assert!(retry.contains("use `jw prepare --resume`"), "{retry}");
    fs::write(&gate, "ok").unwrap();
    fs::write(f.remote.join("version"), "later\n").unwrap();
    f.git_commit();
    ok(f.resume());
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert_eq!(f.id(&f.source, "dev@origin"), goal);
    assert_eq!(disk_stamp(&f.target), stamp);
    assert!(f.records(".preparation.json").is_empty());
}

#[test]
fn unknown_prefetch_goal_can_preview_and_resume_while_remote_is_offline() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.advance();
    ok(f.command("git", &f.remote)
        .args(["branch", "introduced-later"])
        .output()
        .unwrap());
    let offline = f.temp.path().join("offline");
    fs::rename(&f.remote, &offline).unwrap();
    fails(
        f.prepare(&["--source-at", "introduced-later@origin"]),
        "phase Fetching",
    );
    let path = f.active();
    let before = fs::read(&path).unwrap();
    let output = ok(f
        .jw()
        .args(["prepare", "--resume", "--dry-run"])
        .output()
        .unwrap());
    assert!(
        output.contains("post-fetch exact commit unknown")
            && output.contains("introduced-later@origin")
    );
    assert_eq!(fs::read(&path).unwrap(), before);
    assert!(!f.log.exists());
    fs::rename(offline, &f.remote).unwrap();
    ok(f.resume());
}

#[test]
fn verified_completion_resume_retains_proof_and_does_not_replay_commands() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.advance();
    ok(f.prepare(&[]));
    let receipt = f.receipt();
    let path = f.reactivate(&receipt);
    let frozen = fs::read(&path).unwrap();
    let source = f.id(&f.source, "@");
    ok(f.jw()
        .args(["prepare", "--resume", "--dry-run"])
        .output()
        .unwrap());
    assert_eq!(fs::read(&path).unwrap(), frozen);
    ok(f.resume());
    assert_eq!(f.id(&f.source, "@"), source);
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
    assert!(f.records(".preparation.json").is_empty());
}

#[test]
fn v4_journal_rejects_any_target_intent_before_mutation() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.script("exit 17\n");
    f.advance();
    fails(f.prepare(&[]), "command failed");
    let path = f.active();
    let original = read_json(&path);
    let source = f.id(&f.source, "@");
    for key in ["names", "groups", "at", "replace_content", "up"] {
        let mut corrupt = original.clone();
        corrupt["request"][key] = match key {
            "names" | "groups" => json!(["worker"]),
            "at" => json!(f.baseline),
            _ => json!(true),
        };
        write_json(&path, &corrupt);
        fails(f.resume(), "target intent");
        assert_eq!(f.id(&f.source, "@"), source);
    }
    write_json(&path, &original);
}

#[test]
fn command_sequence_retries_only_unacknowledged_step_and_preserves_goal() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let gate = f.temp.path().join("second allowed");
    f.script(&format!("mkdir -p node_modules\nif [ \"$2\" = second ]; then [ -f {gate:?} ] || exit 17; fi\nprintf '%s' \"$2\" > node_modules/\"$2\"\n"));
    let commands = ["first", "second", "third"].map(|step| {
        let mut argv = f.argv();
        argv[2] = step.into();
        argv
    });
    fs::write(
        f.source.join(".jwreseed.local.toml"),
        format!(
            "source_at='dev@origin'\n[freshen]\ncommands={}\n",
            serde_json::to_string(&commands).unwrap()
        ),
    )
    .unwrap();
    let goal = f.advance();
    fails(f.prepare(&[]), "step 2");
    let record = read_json(&f.active());
    assert_eq!(record["command_sequence"]["completed"], 1);
    assert_eq!(record["command_sequence"]["in_flight"], true);
    let preview = ok(f
        .jw()
        .args(["prepare", "--resume", "--dry-run"])
        .output()
        .unwrap());
    assert!(preview.contains("commands completed 1/3"));
    fs::write(&gate, "ok").unwrap();
    fs::write(f.remote.join("version"), "later").unwrap();
    f.git_commit();
    let output = f.resume();
    assert!(text(&output).contains("not exactly-once"));
    ok(output);
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert_eq!(
        fs::read_to_string(&f.log)
            .unwrap()
            .lines()
            .skip(1)
            .step_by(3)
            .collect::<Vec<_>>(),
        ["first", "second", "second", "third"]
    );
    assert!(!f.target.join("node_modules").exists());
}

#[test]
fn tracked_installer_edits_require_restoration_and_changed_policy_is_rejected() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let gate = f.temp.path().join("allow success");
    f.script(&format!(
        "if [ ! -f {gate:?} ]; then printf changed > lockfile; fi\n"
    ));
    let goal = f.advance();
    fails(f.prepare(&[]), "preparer changed tracked tree");
    fails(f.resume(), "restore the recorded source child");
    let policy = fs::read(f.source.join(".jwreseed.local.toml")).unwrap();
    f.policy(true, "# changed");
    fails(f.resume(), "configuration changed");
    fs::write(f.source.join(".jwreseed.local.toml"), policy).unwrap();
    fs::write(gate, "ok").unwrap();
    f.jj(&f.source, &["restore", "--from", &goal]);
    ok(f.resume());
}

#[test]
fn lost_successful_source_new_resumes_without_another_new() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let goal = f.advance();
    let bin = f.wrapper("$new", false);
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    fails(
        command.args(["prepare"]).output().unwrap(),
        "phase Selecting",
    );
    let selected = f.id(&f.source, "@");
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert!(!f.log.exists());
    ok(f.resume());
    assert_eq!(f.id(&f.source, "@"), selected);
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
}

#[test]
fn unacknowledged_fetch_refetches_but_acknowledged_command_does_not_replay() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.advance();
    let bin = f.wrapper("$fetch", false);
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    fails(
        command.args(["prepare"]).output().unwrap(),
        "phase Fetching",
    );
    assert!(read_json(&f.active())["goal"].is_null());
    fs::write(f.remote.join("version"), "latest\n").unwrap();
    f.git_commit();
    let latest = ok(f
        .command("git", &f.remote)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap());
    ok(f.resume());
    assert_eq!(f.id(&f.source, "@-"), latest);
}

#[test]
fn unacknowledged_successful_command_is_explicitly_retried() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let marker = f.temp.path().join("side effect");
    f.script(&format!("touch {marker:?}\n"));
    f.advance();
    let bin = f.wrapper(&format!("[ -f {marker:?} ] && $diff"), true);
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    fails(
        command.args(["prepare"]).output().unwrap(),
        "source preparation stopped",
    );
    assert_eq!(read_json(&f.active())["command_sequence"]["completed"], 0);
    let output = f.resume();
    assert!(text(&output).contains("prior attempt may have performed external side effects"));
    ok(output);
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 6);
}

#[test]
fn configured_or_explicit_donor_is_prepared_from_worker_subdirectory() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let donor = f.temp.path().join("other donor");
    f.jj(
        &f.source,
        &[
            "workspace",
            "add",
            "--name",
            "donor",
            "-r",
            &f.baseline,
            donor.to_str().unwrap(),
        ],
    );
    f.policy(true, &format!("source={donor:?}\n"));
    let primary = f.id(&f.source, "@");
    let worker = f.id(&f.target, "@");
    let goal = f.advance();
    fs::create_dir(f.target.join("subdirectory")).unwrap();
    ok(f.jw()
        .current_dir(f.target.join("subdirectory"))
        .args(["prepare"])
        .output()
        .unwrap());
    assert_eq!(f.id(&donor, "@-"), goal);
    assert_eq!(f.id(&f.source, "@"), primary);
    assert_eq!(f.id(&f.target, "@"), worker);
    let other = f.id(&donor, "@");
    ok(f.jw()
        .args(["prepare", "--from"])
        .arg(&f.source)
        .output()
        .unwrap());
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert_eq!(f.id(&donor, "@"), other);
    assert_eq!(f.id(&f.target, "@"), worker);
}

#[test]
fn stale_worker_does_not_block_source_preparation() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let goal = f.advance();
    f.jj(
        &f.source,
        &["restore", "--into", "worker@", "--from", "root()"],
    );
    let worker = f.id(&f.target, "@");
    let stamp = disk_stamp(&f.target);
    ok(f.prepare(&[]));
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert_eq!(f.id(&f.target, "@"), worker);
    assert_eq!(disk_stamp(&f.target), stamp);
}

#[test]
fn colocated_source_preserves_administrative_directory_identity() {
    let Some(f) = Fixture::with_colocation(true) else {
        return;
    };
    let roots = [&f.source, &f.source.join(".jj"), &f.source.join(".git")];
    let before = roots.map(|p| identity(p));
    let goal = f.advance();
    ok(f.prepare(&[]));
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert_eq!(roots.map(|p| identity(p)), before);
}

#[test]
fn invalid_policy_and_removed_command_fail_before_fetch() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.advance();
    fails(
        f.jw().args(["up"]).output().unwrap(),
        "unrecognized subcommand",
    );
    fails(
        f.jw().args(["reseed", "--freshen"]).output().unwrap(),
        "use `jw prepare`",
    );
    for (policy, expected) in [
        ("[freshen]\ncommand=[]", "nonempty argv"),
        (
            "[freshen]\ncommand=[]\ncommands=[]",
            "cannot both be present",
        ),
        ("[freshen]\n[prepare]", "cannot both be present"),
    ] {
        fs::write(f.source.join(".jwreseed.local.toml"), policy).unwrap();
        fails(f.prepare(&[]), expected);
    }
    assert_eq!(f.id(&f.source, "dev@origin"), f.baseline);
    assert!(!f.log.exists());
}

#[test]
fn ignored_selection_is_proved_and_no_command_does_not_claim_a_build() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.policy(false, "copy_ignored=['node_modules']");
    fs::create_dir(f.source.join("node_modules")).unwrap();
    fs::write(f.source.join("node_modules/package"), "existing").unwrap();
    f.advance();
    let output = ok(f
        .jw()
        .args(["prepare", "--no-copy-ignored"])
        .output()
        .unwrap());
    assert!(output.contains("existing data, not freshly built"));
    assert_eq!(f.receipt()["verified_source"]["ignored"], json!([]));
    assert!(!f.log.exists());
    assert!(!f.target.join("node_modules").exists());
}

#[test]
fn reseed_after_prepare_keeps_feature_tree_and_copies_private_payload() {
    let Some(f) = Fixture::new() else {
        return;
    };
    if !f.cow_supported() {
        return;
    }
    fs::write(f.target.join("feature"), "keep feature\n").unwrap();
    f.jj(&f.target, &["status"]);
    let worker = f.id(&f.target, "@");
    f.advance();
    ok(f.prepare(&[]));
    ok(f.jw()
        .args(["reseed", "worker", "--copy-ignored", "node_modules"])
        .output()
        .unwrap());
    assert_eq!(f.id(&f.target, "@"), worker);
    assert_eq!(
        fs::read_to_string(f.target.join("version")).unwrap(),
        "old\n"
    );
    assert_eq!(
        fs::read_to_string(f.target.join("feature")).unwrap(),
        "keep feature\n"
    );
    assert_ne!(
        identity(&f.source.join("node_modules/package")),
        identity(&f.target.join("node_modules/package"))
    );
    fs::write(f.target.join("node_modules/package"), "private edit").unwrap();
    assert_eq!(
        fs::read_to_string(f.source.join("node_modules/package")).unwrap(),
        "installed\n"
    );
}

#[test]
fn configured_preparation_never_activates_offline_reseed() {
    let Some(f) = Fixture::new() else {
        return;
    };
    if !f.cow_supported() {
        return;
    }
    fs::rename(&f.remote, f.temp.path().join("offline")).unwrap();
    ok(f.jw().args(["reseed", "worker"]).output().unwrap());
    assert!(!f.log.exists());
}

#[test]
fn v1_v2_v3_recover_only_through_reseed_preserving_recorded_target_intent() {
    for (version, explicit_base) in [(1, false), (2, false), (2, true), (3, false)] {
        let Some(f) = Fixture::new() else {
            return;
        };
        if !f.cow_supported() {
            return;
        }
        fs::write(f.target.join("feature"), "legacy feature\n").unwrap();
        f.script("exit 17\n");
        let goal = f.advance();
        fails(f.prepare(&[]), "command failed");
        let path = f.legacy(version, explicit_base);
        let frozen = fs::read(&path).unwrap();
        let worker = f.id(&f.target, "@");
        fails(f.resume(), "legacy batches require `jw reseed --resume`");
        assert_eq!(fs::read(&path).unwrap(), frozen);
        let preview = ok(f
            .jw()
            .args(["reseed", "worker", "--resume", "--dry-run"])
            .output()
            .unwrap());
        if version == 1 {
            assert!(preview.contains("v1 single-command journal loaded compatibly"));
        }
        assert_eq!(fs::read(&path).unwrap(), frozen);
        f.script("mkdir -p node_modules\nprintf legacy > node_modules/package\n");
        ok(f.jw()
            .args(["reseed", "worker", "--resume"])
            .output()
            .unwrap());
        assert_eq!(f.id(&f.source, "@-"), goal);
        if version == 3 {
            assert_eq!(f.id(&f.target, "@-"), goal);
            assert_eq!(f.receipt()["plans"][0]["history"]["work_tip"], worker);
        } else if explicit_base {
            assert_eq!(f.id(&f.target, "@-"), f.baseline);
        } else {
            assert_eq!(f.id(&f.target, "@"), worker);
            assert!(f.target.join("feature").exists());
        }
        assert_eq!(
            fs::read_to_string(f.target.join("node_modules/package")).unwrap(),
            "legacy"
        );
        assert!(f.records(".preparation.json").is_empty());
    }
}

#[test]
fn legacy_v3_partial_target_resume_uses_frozen_plan_and_preservation_bookmark() {
    let Some(f) = Fixture::new() else {
        return;
    };
    if !f.cow_supported() {
        return;
    }
    fs::write(f.target.join("feature"), "legacy feature").unwrap();
    f.script("exit 17\n");
    let goal = f.advance();
    fails(f.prepare(&[]), "command failed");
    f.legacy(3, false);
    f.script("mkdir -p node_modules\nprintf ready > node_modules/package\n");
    let bin = f.wrapper("$clear", false);
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    fails(
        command
            .args(["reseed", "worker", "--resume"])
            .output()
            .unwrap(),
        "PENDING worker",
    );
    let batch = read_json(&f.active());
    let bookmark = &batch["plans"][0]["history"]["bookmark"];
    assert_eq!(
        f.id(&f.source, bookmark["name"].as_str().unwrap()),
        bookmark["tip"].as_str().unwrap()
    );
    let pending = f.records(".pending.json")[0].clone();
    let bytes = fs::read(&pending).unwrap();
    fails(f.resume(), "legacy batches require `jw reseed --resume`");
    assert_eq!(fs::read(&pending).unwrap(), bytes);
    fs::write(f.remote.join("version"), "later").unwrap();
    f.git_commit();
    ok(f.jw().args(["reseed", "--resume"]).output().unwrap());
    assert_eq!(f.id(&f.target, "@-"), goal);
    assert_eq!(f.id(&f.source, "dev@origin"), goal);
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 6);
    assert!(f.records(".preparation.json").is_empty() && f.records(".pending.json").is_empty());
}

#[test]
fn legacy_source_only_v3_batch_is_recoverable_from_primary_via_reseed() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.script("exit 17\n");
    f.advance();
    fails(f.prepare(&[]), "command failed");
    let path = f.active();
    let mut record = read_json(&path);
    record["schema_version"] = 3.into();
    record["request"].as_object_mut().unwrap().remove("prepare");
    record["request"]["up"] = true.into();
    write_json(&path, &record);
    fails(f.resume(), "legacy batches require `jw reseed --resume`");
    f.script("true\n");
    ok(f.jw()
        .current_dir(&f.source)
        .args(["reseed", "--resume"])
        .output()
        .unwrap());
    assert!(f.records(".preparation.json").is_empty());
}

#[test]
fn unfinished_target_borrow_blocks_prepare_even_from_other_workspace() {
    let Some(f) = Fixture::new() else {
        return;
    };
    if !f.cow_supported() {
        return;
    }
    let bin = f.wrapper("$clear", false);
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    fails(
        command.args(["reseed", "worker"]).output().unwrap(),
        "PENDING worker",
    );
    f.advance();
    fails(f.prepare(&[]), "borrowed by unfinished target worker");
    assert!(!f.log.exists());
    ok(f.jw()
        .args(["reseed", "worker", "--resume"])
        .output()
        .unwrap());
}

#[test]
fn verified_payload_change_requires_exact_restoration_before_completion() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.advance();
    ok(f.prepare(&[]));
    let receipt = f.receipt();
    f.reactivate(&receipt);
    fs::write(f.source.join("node_modules/package"), "changed").unwrap();
    fails(f.resume(), "verified preparation source/payload changed");
    fs::write(f.source.join("node_modules/package"), "installed\n").unwrap();
    ok(f.resume());
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
}
