//! Native local remotes and retry-tolerant argv installers exercise source-only preparation.
#![cfg(unix)]
use serde_json::{Value, json};
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
        f.jj(
            &f.target,
            &[
                "--ignore-working-copy",
                "bookmark",
                "create",
                "fixture-worker-work",
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
}
fn identity(path: &Path) -> Value {
    let m = fs::metadata(path).unwrap();
    json!({"device": m.dev(), "inode": m.ino()})
}
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
            result.push(json!({"path": path.strip_prefix(root).unwrap(), "inode": m.ino(), "size": m.len(), "mode": m.mode(), "mtime": m.mtime(), "mtime_ns": m.mtime_nsec(), "ctime": m.ctime(), "ctime_ns": m.ctime_nsec()}));
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
        let Some(f) = Fixture::new() else { return };
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
    }
}

#[test]
fn json_reports_exact_commit_without_installer_stdout_contamination() {
    let Some(f) = Fixture::new() else { return };
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
    let Some(f) = Fixture::new() else { return };
    f.script("printf output > new-build-output\n");
    let goal = f.advance();
    ok(f.prepare(&[]));
    assert_eq!(
        fs::read_to_string(f.source.join("new-build-output")).unwrap(),
        "output"
    );
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert!(
        !f.jj(
            &f.source,
            &["--ignore-working-copy", "file", "list", "-r", "@"]
        )
        .contains("new-build-output")
    );
    assert!(!f.target.join("new-build-output").exists());
}

#[test]
fn dry_run_selects_native_upstream_without_fetch_install_or_worker_snapshot() {
    let Some(f) = Fixture::new() else { return };
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
}

#[test]
fn native_non_origin_and_untracked_exact_lines_are_selected_without_extra_fetches() {
    for tracked in [true, false] {
        let Some(f) = Fixture::new() else { return };
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
    }
}

#[test]
fn source_unpublished_work_blocks_before_fetch_even_above_empty_scratch() {
    for committed in [false, true] {
        let Some(f) = Fixture::new() else { return };
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
fn command_sequence_runs_again_from_the_start_after_failure() {
    let Some(f) = Fixture::new() else { return };
    let gate = f.temp.path().join("second allowed");
    f.script(&format!(
        "mkdir -p node_modules\nif [ \"$2\" = second ]; then [ -f {gate:?} ] || exit 17; fi\nprintf '%s' \"$2\" > node_modules/\"$2\"\n"
    ));
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
    let failure = fails(f.prepare(&[]), "step 2");
    assert!(failure.contains("fix the source and run prepare again"));
    fs::write(&gate, "ok").unwrap();
    ok(f.prepare(&[]));
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert_eq!(
        fs::read_to_string(&f.log)
            .unwrap()
            .lines()
            .skip(1)
            .step_by(3)
            .collect::<Vec<_>>(),
        ["first", "second", "first", "second", "third"]
    );
    assert!(!f.target.join("node_modules").exists());
}

#[test]
fn tracked_installer_edits_require_deliberate_restoration_before_retry() {
    let Some(f) = Fixture::new() else { return };
    f.script("printf changed > lockfile\n");
    let goal = f.advance();
    fails(f.prepare(&[]), "preparer changed tracked tree");
    assert_eq!(fs::read(f.source.join("lockfile")).unwrap(), b"changed");
    f.jj(&f.source, &["restore", "--from", &goal]);
    f.script("true\n");
    f.policy(true, "# policy may change between attempts");
    ok(f.prepare(&[]));
}

#[test]
fn configured_or_explicit_donor_is_prepared_from_worker_subdirectory() {
    let Some(f) = Fixture::new() else { return };
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
    let Some(f) = Fixture::new() else { return };
    let goal = f.advance();
    f.jj(
        &f.source,
        &["restore", "--into", "worker@", "--from", "root()"],
    );
    let worker = f.id(&f.target, "@");
    let before = disk_stamp(&f.target);
    ok(f.prepare(&[]));
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert_eq!(f.id(&f.target, "@"), worker);
    assert_eq!(disk_stamp(&f.target), before);
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
fn invalid_policy_and_removed_commands_fail_before_fetch() {
    let Some(f) = Fixture::new() else { return };
    f.advance();
    fails(
        f.jw().args(["up"]).output().unwrap(),
        "unrecognized subcommand",
    );
    fails(
        f.jw().args(["reseed", "--freshen"]).output().unwrap(),
        "use `jw prepare`",
    );
    for command in ["prepare", "reseed"] {
        fails(
            f.jw().args([command, "--resume"]).output().unwrap(),
            "unexpected argument '--resume'",
        );
    }
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
    let Some(f) = Fixture::new() else { return };
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
    assert!(!f.log.exists());
    assert!(!f.target.join("node_modules").exists());
}

#[test]
fn reseed_after_prepare_keeps_feature_tree_and_copies_private_payload() {
    let Some(f) = Fixture::new() else { return };
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
    let Some(f) = Fixture::new() else { return };
    if !f.cow_supported() {
        return;
    }
    fs::rename(&f.remote, f.temp.path().join("offline")).unwrap();
    ok(f.jw().args(["reseed", "worker"]).output().unwrap());
    assert!(!f.log.exists());
}

#[test]
fn leftover_journals_are_ignored_without_migration_or_pruning() {
    let Some(f) = Fixture::new() else { return };
    let repo_config = PathBuf::from(f.jj(&f.source, &["config", "path", "--repo"]));
    let directory = repo_config.parent().unwrap().join("jj-waltz/reseed");
    // Initialize the authoritative metadata manifest before adding old files;
    // manifestless nonempty stores must still fail closed, never be repaired.
    ok(f.prepare(&["--dry-run"]));
    fs::create_dir_all(&directory).unwrap();
    let paths = [
        "old.preparation.json",
        "old.pending.json",
        "old.receipt.json",
    ]
    .map(|name| directory.join(name));
    for path in &paths {
        fs::write(path, b"obsolete invalid JSON\n").unwrap();
    }
    f.advance();
    ok(f.prepare(&[]));
    if f.cow_supported() {
        ok(f.jw().args(["reseed", "worker"]).output().unwrap());
    }
    for path in paths {
        assert_eq!(fs::read(path).unwrap(), b"obsolete invalid JSON\n");
    }
    let mut names = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    names.sort();
    assert_eq!(
        names,
        [
            "old.pending.json",
            "old.preparation.json",
            "old.receipt.json",
            "operation.lock"
        ]
    );
}
