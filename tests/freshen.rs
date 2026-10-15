//! Local Git remotes and fake argv installers only; real CoW is required when
//! JW_TEST_REQUIRE_COW is set. Never accesses an operator checkout or network.
#![cfg(unix)]
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
        let temp = tempfile::tempdir().unwrap();
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
            .args(["git", "clone", "--no-colocate"])
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
        let probe = f.target.join("probe");
        if let Err(e) = reflink_copy::reflink(f.source.join("shared"), &probe) {
            assert!(
                std::env::var_os("JW_TEST_REQUIRE_COW").is_none(),
                "CoW required: {e}"
            );
            assert!(matches!(e.raw_os_error(), Some(18 | 22 | 38 | 95)), "{e}");
            return None;
        }
        fs::remove_file(probe).unwrap();
        f.script("mkdir -p node_modules\nprintf 'installed\\n' > node_modules/package\n");
        f.policy(true, "");
        Some(f)
    }
    fn command(&self, program: &str, cwd: &Path) -> Command {
        let mut c = Command::new(program);
        c.current_dir(cwd)
            .env("XDG_CONFIG_HOME", &self.config)
            .env("JJ_CONFIG", "")
            .env("JJ_USER", "Freshen Test")
            .env("JJ_EMAIL", "freshen@example.invalid")
            .env("JJ_EDITOR", "true")
            .env("GIT_AUTHOR_NAME", "Freshen Test")
            .env("GIT_AUTHOR_EMAIL", "freshen@example.invalid")
            .env("GIT_COMMITTER_NAME", "Freshen Test")
            .env("GIT_COMMITTER_EMAIL", "freshen@example.invalid");
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
        // The executable explicitly uses its own shebang; jw does not insert a shell.
        let script = format!(
            "#!/bin/sh\nset -eu\npwd >> \"$1\"\nprintf '%s\\n' \"$2\" \"$3\" >> \"$1\"\n{body}"
        );
        fs::write(&self.installer, script).unwrap();
        fs::set_permissions(&self.installer, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fn policy(&self, command: bool, extra: &str) {
        let argv = serde_json::to_string(&vec![
            self.installer.to_str().unwrap(),
            self.log.to_str().unwrap(),
            "argument with spaces",
            "$(touch SHOULD_NOT_EXIST); *",
        ])
        .unwrap();
        fs::write(
            self.source.join(".jwreseed.local.toml"),
            format!(
                "source_at='dev@origin'\n{extra}\n[prepare]\nremote='origin'\n{}",
                if command {
                    format!("command={argv}\n")
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
    fn freshen(&self, extra: &[&str]) -> Output {
        self.jw()
            .args([
                "reseed",
                "worker",
                "--freshen",
                "--copy-ignored",
                "node_modules",
            ])
            .args(extra)
            .output()
            .unwrap()
    }
    fn resume(&self) -> Output {
        self.jw()
            .args(["reseed", "worker", "--resume"])
            .output()
            .unwrap()
    }
    fn journal(&self) -> PathBuf {
        let p = PathBuf::from(self.jj(&self.source, &["config", "path", "--repo"]));
        p.parent().unwrap().join("jj-waltz/reseed")
    }
    fn active(&self) -> Vec<PathBuf> {
        fs::read_dir(self.journal())
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with(".preparation.json")
            })
            .collect()
    }
    fn inode(&self, p: &Path) -> (u64, u64) {
        let m = fs::metadata(p).unwrap();
        (m.dev(), m.ino())
    }
    fn wrapper(&self, condition: &str) -> PathBuf {
        let real = std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|p| p.join("jj"))
            .find(|p| p.is_file())
            .unwrap();
        let bin = self.temp.path().join("wrapper-bin");
        fs::create_dir_all(&bin).unwrap();
        let marker = self.temp.path().join("wrapper fired");
        let body = format!(
            "#!/bin/sh\nset -eu\nnew=false; clear=false\nfor arg do\n[ \"$arg\" != new ] || new=true\n[ \"$arg\" != --clear ] || clear=true\ndone\n{:?} \"$@\"\nif [ ! -f {:?} ] && {condition}; then touch {:?}; exit 97; fi\n",
            real, marker, marker
        );
        fs::write(bin.join("jj"), body).unwrap();
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
    fn pending(&self) -> Vec<PathBuf> {
        fs::read_dir(self.journal())
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with(".pending.json")
            })
            .collect()
    }
}

#[test]
fn freshen_fetches_once_preserves_work_and_keeps_target_with_private_payload() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let roots = [
        &f.source,
        &f.source.join(".jj"),
        &f.target,
        &f.target.join(".jj"),
    ];
    let identities = roots.map(|p| f.inode(p));
    fs::write(f.source.join("source-work"), "preserve source work\n").unwrap();
    fs::write(f.target.join("target-work"), "keep feature\n").unwrap();
    let goal = f.advance();
    let output = ok(f.freshen(&[]));
    assert!(output.contains("exact source goal"), "{output}");
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert_eq!(
        fs::read_to_string(f.source.join("version")).unwrap(),
        "fresh\n"
    );
    assert_eq!(
        fs::read_to_string(f.target.join("version")).unwrap(),
        "old\n"
    );
    assert_eq!(
        fs::read_to_string(f.target.join("target-work")).unwrap(),
        "keep feature\n"
    );
    assert!(
        f.jj(
            &f.source,
            &[
                "log",
                "--no-graph",
                "-r",
                "all()",
                "-T",
                "diff.files().map(|f| f.path()).join('\\n')"
            ]
        )
        .contains("source-work")
    );
    assert_eq!(identities, roots.map(|p| f.inode(p)));
    let log = fs::read_to_string(&f.log).unwrap();
    assert_eq!(
        log.lines().collect::<Vec<_>>(),
        vec![
            f.source.to_str().unwrap(),
            "argument with spaces",
            "$(touch SHOULD_NOT_EXIST); *"
        ]
    );
    assert!(!f.source.join("SHOULD_NOT_EXIST").exists());
    assert!(f.active().is_empty());
    assert_ne!(
        f.inode(&f.source.join("node_modules/package")),
        f.inode(&f.target.join("node_modules/package"))
    );
    fs::write(f.target.join("node_modules/package"), "private edit").unwrap();
    assert_eq!(
        fs::read_to_string(f.source.join("node_modules/package")).unwrap(),
        "installed\n"
    );
}

#[test]
fn freshen_target_reset_is_independent_and_preserves_target_history() {
    let Some(f) = Fixture::new() else {
        return;
    };
    fs::write(f.target.join("target-work"), "old feature\n").unwrap();
    let goal = f.advance();
    ok(f.freshen(&["--at", "dev@origin"]));
    assert_eq!(f.id(&f.target, "@-"), goal);
    assert_eq!(
        fs::read_to_string(f.target.join("version")).unwrap(),
        "fresh\n"
    );
    assert!(!f.target.join("target-work").exists());
    assert!(
        f.jj(
            &f.source,
            &[
                "log",
                "--no-graph",
                "-r",
                "all()",
                "-T",
                "diff.files().map(|f| f.path()).join('\\n')"
            ]
        )
        .contains("target-work")
    );
}

#[test]
fn freshen_dry_run_unknown_remote_goal_performs_no_preparation_actions() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.advance();
    ok(f.command("git", &f.remote)
        .args(["branch", "introduced-later"])
        .output()
        .unwrap());
    let source = f.id(&f.source, "@");
    let output = ok(f.freshen(&["--source-at", "introduced-later@origin", "--dry-run"]));
    assert!(
        output.contains("prospective post-fetch goal")
            && output.contains("post-fetch exact commit unknown"),
        "{output}"
    );
    assert_eq!(f.id(&f.source, "@"), source);
    assert!(!f.log.exists());
    assert!(f.active().is_empty());
    assert!(
        !f.command("jj", &f.source)
            .args([
                "--ignore-working-copy",
                "log",
                "-r",
                "introduced-later@origin"
            ])
            .output()
            .unwrap()
            .status
            .success()
    );
    ok(f.freshen(&["--source-at", "introduced-later@origin"]));
}

#[test]
fn failed_installer_blocks_borrowing_and_resume_pins_goal_without_fetch() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let gate = f.temp.path().join("allow success");
    f.script(&format!("mkdir -p node_modules\nprintf partial > node_modules/package\n[ -f {:?} ] || exit 17\nprintf complete > node_modules/package\n", gate));
    let goal = f.advance();
    let before = f.id(&f.target, "@");
    fails(f.freshen(&[]), "ignored installation may be incomplete");
    assert_eq!(f.id(&f.target, "@"), before);
    assert_eq!(f.active().len(), 1);
    fails(
        f.jw()
            .args(["reseed", "worker", "--source-at", "dev@origin"])
            .output()
            .unwrap(),
        "active preparation batch",
    );
    fs::write(&gate, "ok").unwrap();
    fs::write(f.remote.join("version"), "even newer\n").unwrap();
    f.git_commit();
    ok(f.resume());
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert_eq!(
        fs::read_to_string(f.target.join("node_modules/package")).unwrap(),
        "complete"
    );
    assert!(f.active().is_empty());
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 6);
}

#[test]
fn tracked_preparer_edits_fail_before_clear_and_require_restoration() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let gate = f.temp.path().join("allow success");
    f.script(&format!("mkdir -p node_modules\nif [ ! -f {:?} ]; then printf changed > lockfile; fi\nprintf ready > node_modules/package\n", gate));
    let goal = f.advance();
    let before = f.id(&f.target, "@");
    fails(f.freshen(&[]), "preparer changed tracked tree");
    fails(f.resume(), "restore the recorded source child");
    assert_eq!(f.id(&f.target, "@"), before);
    fs::write(gate, "ok").unwrap();
    f.jj(&f.source, &["restore", "--from", &goal]);
    ok(f.resume());
}

#[test]
fn failed_fetch_retains_source_and_targets_and_can_resume() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let goal = f.advance();
    let unavailable = f.temp.path().join("remote temporarily unavailable");
    fs::rename(&f.remote, &unavailable).unwrap();
    let before = f.id(&f.source, "@");
    fails(f.freshen(&[]), "source progress was not rolled back");
    assert_eq!(f.id(&f.source, "@"), before);
    assert!(!f.log.exists());
    fs::rename(unavailable, &f.remote).unwrap();
    ok(f.resume());
    assert_eq!(f.id(&f.source, "@-"), goal);
}

#[test]
fn preparation_settings_do_not_activate_normal_offline_reseed() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let unavailable = f.temp.path().join("unavailable");
    fs::rename(&f.remote, unavailable).unwrap();
    ok(f.jw().args(["reseed", "worker"]).output().unwrap());
    assert!(!f.log.exists());
    assert_eq!(f.id(&f.source, "@-"), f.baseline);
}

#[test]
fn missing_command_and_remote_and_invalid_selection_fail_before_fetch() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let old = f.id(&f.source, "dev@origin");
    f.advance();
    f.policy(false, "");
    fails(f.freshen(&[]), "requires prepare.command");
    assert_eq!(f.id(&f.source, "dev@origin"), old);
    f.policy(true, "");
    fails(
        f.jw()
            .args(["reseed", "missing", "--freshen"])
            .output()
            .unwrap(),
        "unknown workspace",
    );
    f.jj(&f.source, &["git", "remote", "remove", "origin"]);
    fails(f.freshen(&[]), "not configured");
    assert!(!f.log.exists());
}

#[test]
fn tracked_only_freshen_without_command_reports_honestly() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.policy(false, "");
    let goal = f.advance();
    let output = ok(f
        .jw()
        .args(["reseed", "worker", "--freshen"])
        .output()
        .unwrap());
    assert!(output.contains("tracked-source preparation only"));
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert!(!f.log.exists());
}

#[test]
fn changed_target_or_trusted_config_stops_preparation_resume() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.script("exit 19\n");
    f.advance();
    fails(f.freshen(&[]), "command failed");
    fs::write(f.target.join("version"), "concurrent target edit\n").unwrap();
    fails(f.resume(), "changed or became stale");
    assert_eq!(
        fs::read_to_string(f.target.join("version")).unwrap(),
        "concurrent target edit\n"
    );
    f.policy(true, "# policy changed\n");
    fails(f.resume(), "configuration changed");
}

#[test]
fn all_skipped_group_selection_does_not_freshen() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.policy(
        true,
        "[groups.empty]\nname_globs=['missing-*']\non_unmatched='warn'\n",
    );
    let before = f.id(&f.source, "dev@origin");
    f.advance();
    fails(
        f.jw()
            .args(["reseed", "--group", "empty", "--freshen"])
            .output()
            .unwrap(),
        "no workspaces selected",
    );
    assert_eq!(f.id(&f.source, "dev@origin"), before);
    assert!(!f.log.exists());
}

#[test]
fn lost_successful_source_new_result_resumes_without_another_new() {
    let Some(f) = Fixture::new() else {
        return;
    };
    fs::write(f.source.join("prior-work"), "keep this\n").unwrap();
    let goal = f.advance();
    let bin = f.wrapper("[ \"$new\" = true ]");
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    fails(
        command
            .args([
                "reseed",
                "worker",
                "--freshen",
                "--copy-ignored",
                "node_modules",
            ])
            .output()
            .unwrap(),
        "phase Selecting",
    );
    let selected = f.id(&f.source, "@");
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert!(!f.log.exists());
    ok(f.resume());
    assert_eq!(f.id(&f.source, "@"), selected);
    assert_eq!(fs::read_to_string(f.log).unwrap().lines().count(), 3);
}

#[test]
fn frozen_batch_recovery_uses_receipts_and_never_reprepares_or_snapshots_partial_targets() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let second = f.temp.path().join("second");
    f.jj(
        &f.source,
        &[
            "workspace",
            "add",
            "--name",
            "second",
            "-r",
            &f.baseline,
            second.to_str().unwrap(),
        ],
    );
    f.advance();
    let bin = f.wrapper(&format!(
        "[ \"$clear\" = true ] && [ \"$PWD\" = {:?} ]",
        f.target
    ));
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    let output = fails(
        command
            .args([
                "reseed",
                "worker",
                "second",
                "--freshen",
                "--at",
                "dev@origin",
                "--copy-ignored",
                "node_modules",
            ])
            .output()
            .unwrap(),
        "PENDING worker",
    );
    assert!(output.contains("OK second"));
    assert_eq!(f.pending().len(), 1);
    assert!(!f.target.join("shared").exists());
    assert_eq!(f.active().len(), 1);
    let batch: serde_json::Value =
        serde_json::from_slice(&fs::read(&f.active()[0]).unwrap()).unwrap();
    assert_eq!(batch["plans"].as_array().unwrap().len(), 2);
    fs::write(f.remote.join("version"), "newer remote\n").unwrap();
    f.git_commit();
    ok(f.jw()
        .args(["reseed", "second", "worker", "--resume"])
        .output()
        .unwrap());
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
    assert!(f.active().is_empty());
    assert!(f.pending().is_empty());
    assert_eq!(
        fs::read_to_string(f.target.join("version")).unwrap(),
        "fresh\n"
    );
}

#[test]
fn ordinary_legacy_pending_donor_borrow_blocks_new_freshen_even_if_unselected() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let second = f.temp.path().join("second");
    f.jj(
        &f.source,
        &[
            "workspace",
            "add",
            "--name",
            "second",
            "-r",
            &f.baseline,
            second.to_str().unwrap(),
        ],
    );
    let bin = f.wrapper("[ \"$clear\" = true ]");
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    fails(
        command.args(["reseed", "worker"]).output().unwrap(),
        "PENDING worker",
    );
    let p = &f.pending()[0];
    let mut legacy: serde_json::Value = serde_json::from_slice(&fs::read(p).unwrap()).unwrap();
    legacy.as_object_mut().unwrap().remove("preparation_id");
    fs::write(p, serde_json::to_vec(&legacy).unwrap()).unwrap();
    let source = f.id(&f.source, "@");
    f.advance();
    fails(
        f.jw()
            .args(["reseed", "second", "--freshen"])
            .output()
            .unwrap(),
        "borrowed by unfinished target worker",
    );
    assert_eq!(f.id(&f.source, "@"), source);
    assert!(!f.log.exists());
    ok(f.resume());
}

#[test]
fn frozen_handoff_recreates_only_missing_intent_and_rejects_foreign_pending() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.advance();
    let bin = f.wrapper("[ \"$clear\" = true ]");
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    fails(
        command
            .args(["reseed", "worker", "--freshen"])
            .output()
            .unwrap(),
        "PENDING worker",
    );
    let p = f.pending()[0].clone();
    let saved = fs::read(&p).unwrap();
    let mut foreign: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    foreign["preparation_id"] = "foreign".into();
    let bad = serde_json::to_vec(&foreign).unwrap();
    fs::write(&p, &bad).unwrap();
    fails(f.resume(), "foreign or changed pending plan");
    assert_eq!(fs::read(&p).unwrap(), bad);
    fs::write(&p, saved).unwrap();
    // Missing record can be recreated only in its originally Prepared state:
    // reconstruct the complete original tree through this recorded recovery,
    // then simulate the crash between atomic intent and first target handoff.
    ok(f.resume());
    assert!(f.active().is_empty());
}

#[test]
fn baseline_owned_links_change_normally_and_done_handoff_keeps_runtime_diagnostics_separate() {
    let Some(f) = Fixture::new() else {
        return;
    };
    fs::write(
        f.remote.join(".jwlinks.toml"),
        format!(
            "[[link]]\nsource='shared-link'\ntarget={:?}\nrequired=true\n",
            f.source.join("shared")
        ),
    )
    .unwrap();
    f.advance();
    let output = ok(f.freshen(&[]));
    assert!(output.contains("links: 1 created"), "{output}");
    assert_eq!(
        fs::read_link(f.target.join("shared-link")).unwrap(),
        f.source.join("shared")
    );
    let records = fs::read_dir(f.journal())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(".receipt.json")
        })
        .collect::<Vec<_>>();
    let batch_path = records
        .iter()
        .find(|p| {
            let value: serde_json::Value = serde_json::from_slice(&fs::read(p).unwrap()).unwrap();
            value["plans"].is_array()
        })
        .unwrap();
    let batch_bytes = fs::read(batch_path).unwrap();
    let key = format!("source:{}", f.source.display());
    let digest = Sha256::digest(key.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let active_path = f
        .journal()
        .join(format!("workspace-{digest}.preparation.json"));
    fs::write(active_path, batch_bytes).unwrap();
    let target_receipt = records
        .iter()
        .find(|p| {
            let value: serde_json::Value = serde_json::from_slice(&fs::read(p).unwrap()).unwrap();
            value["name"] == "worker"
        })
        .unwrap();
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(target_receipt).unwrap()).unwrap();
    assert_eq!(receipt["phase"], "Done");
    assert_eq!(
        receipt["execution_diagnostics"].as_array().unwrap().len(),
        1
    );
    // Simulate interruption after Done write-ahead, before receipt completion.
    let pending = target_receipt.with_file_name(
        target_receipt
            .file_name()
            .unwrap()
            .to_string_lossy()
            .replace(".receipt.json", ".pending.json"),
    );
    fs::write(&pending, fs::read(target_receipt).unwrap()).unwrap();
    fs::remove_file(target_receipt).unwrap();
    ok(f.resume());
    assert!(f.pending().is_empty());
    assert!(f.active().is_empty());
    assert_eq!(fs::read_to_string(f.log).unwrap().lines().count(), 3);
}

#[test]
fn resume_revalidates_donor_external_ownership_before_installer() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.script("exit 17\n");
    f.advance();
    fails(f.freshen(&[]), "command failed");
    let batch_before = fs::read(&f.active()[0]).unwrap();
    let repo_config = PathBuf::from(f.jj(&f.source, &["config", "path", "--repo"]));
    // Owner metadata is private API; create the documented schema fixture in
    // this disposable store, using its stable native workspace-key encoding.
    let record = serde_json::json!({"schema_version": 3, "metadata": {
        "workspace_name": "default", "created_at_unix_ms": 0,
        "creation_operation_id": "", "creation_base_commit_id": "",
        "associated_bookmark": null, "intended_remote": null,
        "external_owner": { "checkout_root": f.source, "git_dir": f.source.join(".git/worktrees/external"), "common_dir": f.source.join(".git") }
    }});
    let mut hash: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
    for byte in b"workspace\0default" {
        hash = (hash ^ u128::from(*byte)).wrapping_mul(0x0000_0000_0100_0000_0000_0000_0000_013b);
    }
    let directory = repo_config.parent().unwrap().join("jj-waltz/workspaces");
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join(format!("workspace-{hash:032x}.json")),
        serde_json::to_vec(&record).unwrap(),
    )
    .unwrap();
    fails(f.resume(), "external/app-owned");
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
    assert_eq!(fs::read(&f.active()[0]).unwrap(), batch_before);
}

#[test]
fn verified_unfrozen_preparation_refuses_changed_dependencies_and_allows_exact_restoration() {
    let Some(f) = Fixture::new() else {
        return;
    };
    // This target-planning failure occurs after verified preparation, before
    // atomic target intent. Its local link policy is deliberately repairable.
    fs::write(
        f.source.join(".jwlinks.local.toml"),
        "[[link]]\nsource='shared-link'\ntarget='temporarily-missing'\nrequired=true\n",
    )
    .unwrap();
    f.advance();
    fails(f.freshen(&[]), "required link target will be absent");
    assert!(f.pending().is_empty());
    let batch: serde_json::Value =
        serde_json::from_slice(&fs::read(&f.active()[0]).unwrap()).unwrap();
    assert_eq!(batch["phase"], "Verified");
    assert!(batch["plans"].is_null());
    assert!(batch["verified_source"]["payload_digest"].is_string());
    fs::write(f.source.join("node_modules/package"), "corrupted payload").unwrap();
    // Repair link planning without changing the frozen preparation argv policy.
    fs::write(
        f.source.join(".jwlinks.local.toml"),
        "[[link]]\nsource='shared-link'\ntarget='shared'\nrequired=true\n",
    )
    .unwrap();
    fails(f.resume(), "verified preparation source/payload changed");
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
    fs::write(f.source.join("node_modules/package"), "installed\n").unwrap();
    ok(f.resume());
    assert_eq!(fs::read_to_string(f.log).unwrap().lines().count(), 3);
}
