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
        Self::new_with_colocation(false)
    }
    fn new_with_colocation(colocated: bool) -> Option<Self> {
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
                "source_at='dev@origin'\n{extra}\n[freshen]\nremote='origin'\n{}",
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
            .args(["up", "worker", "--copy-ignored", "node_modules"])
            .args(extra)
            .output()
            .unwrap()
    }
    fn resume(&self) -> Output {
        self.jw()
            .args(["up", "worker", "--resume"])
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
        self.wrapper_at(condition, false)
    }
    fn wrapper_at(&self, condition: &str, before: bool) -> PathBuf {
        let real = std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|p| p.join("jj"))
            .find(|p| p.is_file())
            .unwrap();
        let bin = self.temp.path().join("wrapper-bin");
        fs::create_dir_all(&bin).unwrap();
        let marker = self.temp.path().join("wrapper fired");
        let failure = format!(
            "if [ ! -f {:?} ] && {condition}; then touch {:?}; exit 97; fi\n",
            marker, marker
        );
        let delegate = format!("{:?} \"$@\"\n", real);
        let body = format!(
            "#!/bin/sh\nset -eu\nnew=false; clear=false; fetch=false; sparse=false; listing=false; status=false\nfor arg do\n[ \"$arg\" != new ] || new=true\n[ \"$arg\" != --clear ] || clear=true\n[ \"$arg\" != fetch ] || fetch=true\n[ \"$arg\" != sparse ] || sparse=true\n[ \"$arg\" != list ] || listing=true\n[ \"$arg\" != status ] || status=true\ndone\n{}{}",
            if before { &failure } else { &delegate },
            if before { &delegate } else { &failure }
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
    fn receipt(&self) -> serde_json::Value {
        fs::read_dir(self.journal())
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with(".receipt.json")
            })
            .map(|p| serde_json::from_slice::<serde_json::Value>(&fs::read(p).unwrap()).unwrap())
            .find(|v| v["plans"].is_array())
            .unwrap()
    }
    fn downgrade_single_command_journal_to_v1(&self) -> PathBuf {
        let active = self.active();
        assert_eq!(active.len(), 1);
        let path = active[0].clone();
        let mut record: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(
            record["command_sequence"]["argv"].as_array().unwrap().len(),
            1
        );
        record["command"] = record["command_sequence"]["argv"][0].clone();
        record.as_object_mut().unwrap().remove("command_sequence");
        if let Some(proof) = record["verified_source"].as_object_mut() {
            proof.remove("selection_diagnostics");
        }
        record["schema_version"] = 1.into();
        // Reconstruct the pre-up wire contract, not a downgraded up/reset plan.
        record.as_object_mut().unwrap().remove("source_selection");
        record["request"]["up"] = false.into();
        record["request"]["freshen"] = true.into();
        record["request"]
            .as_object_mut()
            .unwrap()
            .remove("no_freshen_bookmark");
        fs::write(&path, serde_json::to_vec_pretty(&record).unwrap()).unwrap();
        path
    }
}

#[test]
fn up_fetches_once_preserves_target_work_and_advances_with_private_payload() {
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
    assert_shared_extents(&f.source.join("shared"), &f.target.join("shared"));
    fs::write(f.target.join("node_modules/package"), "private edit").unwrap();
    assert_eq!(
        fs::read_to_string(f.source.join("node_modules/package")).unwrap(),
        "installed\n"
    );
}

#[test]
fn up_target_advancement_preserves_target_history() {
    let Some(f) = Fixture::new() else {
        return;
    };
    fs::write(f.target.join("target-work"), "old feature\n").unwrap();
    let goal = f.advance();
    ok(f.freshen(&[]));
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
fn bare_up_at_primary_finishes_source_only_with_real_receipt() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let old_target = f.id(&f.target, "@");
    let goal = f.advance();
    let old_source = f.id(&f.source, "@");
    let preview = ok(f
        .jw()
        .current_dir(&f.source)
        .args(["up", "--dry-run"])
        .output()
        .unwrap());
    assert!(preview.contains("source-only up"));
    assert_eq!(f.id(&f.source, "@"), old_source);
    assert_eq!(f.id(&f.source, "dev@origin"), f.baseline);
    assert!(!f.log.exists());
    assert!(f.active().is_empty() && f.pending().is_empty());
    let output = ok(f.jw().current_dir(&f.source).args(["up"]).output().unwrap());
    assert!(output.contains("exact source goal"), "{output}");
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert_eq!(f.id(&f.target, "@"), old_target);
    assert!(!f.target.join("node_modules").exists());
    assert!(f.active().is_empty() && f.pending().is_empty());
    let receipt = f.receipt();
    assert_eq!(receipt["schema_version"], 3);
    assert_eq!(receipt["request"]["names"], serde_json::json!([]));
    assert_eq!(receipt["plans"], serde_json::json!([]));
    assert_eq!(receipt["goal"], goal);
}

#[test]
fn source_only_failure_resume_discovers_journal_before_snapshots_and_does_not_refetch() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let gate = f.temp.path().join("source-only gate");
    f.script(&format!("[ -f {:?} ] || exit 17\n", gate));
    let goal = f.advance();
    fails(
        f.jw().current_dir(&f.source).args(["up"]).output().unwrap(),
        "command failed",
    );
    let frozen = fs::read(&f.active()[0]).unwrap();
    let source = f.id(&f.source, "@");
    let output = ok(f
        .jw()
        .current_dir(&f.source)
        .args(["up", "--resume", "--dry-run"])
        .output()
        .unwrap());
    assert!(output.contains("commands completed 0/1"));
    assert_eq!(fs::read(&f.active()[0]).unwrap(), frozen);
    assert_eq!(f.id(&f.source, "@"), source);
    fs::write(&gate, "okay").unwrap();
    fs::write(f.remote.join("version"), "unfetched later\n").unwrap();
    f.git_commit();
    ok(f.jw()
        .current_dir(&f.source)
        .args(["up", "--resume"])
        .output()
        .unwrap());
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert_eq!(f.id(&f.source, "dev@origin"), goal);
    assert!(f.active().is_empty());
}

#[test]
fn source_only_verified_completion_resume_uses_empty_intent_and_receipt() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.advance();
    ok(f.jw().current_dir(&f.source).args(["up"]).output().unwrap());
    let receipt = f.receipt();
    let key = format!("source:{}", f.source.display());
    let digest = Sha256::digest(key.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let active = f
        .journal()
        .join(format!("workspace-{digest}.preparation.json"));
    fs::write(&active, serde_json::to_vec(&receipt).unwrap()).unwrap();
    let frozen = fs::read(&active).unwrap();
    let donor = f.id(&f.source, "@");
    let target = f.id(&f.target, "@");
    ok(f.jw()
        .current_dir(&f.source)
        .args(["up", "--resume", "--dry-run"])
        .output()
        .unwrap());
    assert_eq!(fs::read(&active).unwrap(), frozen);
    ok(f.jw()
        .current_dir(&f.source)
        .args(["up", "--resume"])
        .output()
        .unwrap());
    assert_eq!(f.id(&f.source, "@"), donor);
    assert_eq!(f.id(&f.target, "@"), target);
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
    assert!(f.active().is_empty() && f.pending().is_empty());
}

#[test]
fn missing_goal_pre_fetch_resume_preview_does_not_try_to_resolve_or_fetch_it() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.advance();
    ok(f.command("git", &f.remote)
        .args(["branch", "introduced-later"])
        .output()
        .unwrap());
    let absent = f.temp.path().join("offline remote");
    fs::rename(&f.remote, &absent).unwrap();
    fails(
        f.freshen(&["--source-at", "introduced-later@origin"]),
        "phase Fetching",
    );
    let active = f.active()[0].clone();
    let frozen = fs::read(&active).unwrap();
    let preview = ok(f
        .jw()
        .args(["up", "--resume", "--dry-run"])
        .output()
        .unwrap());
    assert!(preview.contains("post-fetch exact commit unknown"));
    assert!(preview.contains("introduced-later@origin"));
    assert_eq!(fs::read(&active).unwrap(), frozen);
    assert!(!f.log.exists());
    fs::rename(absent, &f.remote).unwrap();
    ok(f.resume());
}

#[test]
fn removed_flag_has_migration_guidance_and_legacy_v2_intent_is_never_upgraded_to_up() {
    let Some(f) = Fixture::new() else {
        return;
    };
    fails(
        f.jw().args(["reseed", "--freshen"]).output().unwrap(),
        "use `jw up`",
    );
    assert!(!f.log.exists());
    for explicit_base in [false, true] {
        let Some(f) = Fixture::new() else {
            return;
        };
        fs::write(f.target.join("feature-work"), "keep legacy work\n").unwrap();
        f.script("exit 17\n");
        let goal = f.advance();
        fails(f.freshen(&[]), "command failed");
        let old = f.id(&f.target, "@");
        let active = f.active()[0].clone();
        let mut legacy: serde_json::Value =
            serde_json::from_slice(&fs::read(&active).unwrap()).unwrap();
        legacy["schema_version"] = 2.into();
        legacy.as_object_mut().unwrap().remove("source_selection");
        legacy["request"]["up"] = false.into();
        legacy["request"]["freshen"] = true.into();
        legacy["request"]
            .as_object_mut()
            .unwrap()
            .remove("no_freshen_bookmark");
        if explicit_base {
            legacy["request"]["at"] = f.baseline.clone().into();
        }
        let frozen = serde_json::to_vec(&legacy).unwrap();
        fs::write(&active, &frozen).unwrap();
        fails(f.resume(), "legacy jw reseed");
        assert_eq!(fs::read(&active).unwrap(), frozen);
        f.script("mkdir -p node_modules\nprintf recovered > node_modules/package\n");
        ok(f.jw()
            .args(["reseed", "worker", "--resume"])
            .output()
            .unwrap());
        assert_eq!(f.id(&f.source, "@-"), goal);
        if explicit_base {
            assert_eq!(f.id(&f.target, "@-"), f.baseline);
            assert!(!f.target.join("feature-work").exists());
        } else {
            assert_eq!(f.id(&f.target, "@"), old);
            assert!(f.target.join("feature-work").exists());
        }
        assert!(f.receipt()["plans"][0].get("history").is_none());
    }
}

#[test]
fn bare_variant_up_pins_native_non_origin_line_and_fetches_only_that_bookmark() {
    let Some(f) = Fixture::new() else {
        return;
    };
    fs::remove_file(f.source.join(".jwreseed.local.toml")).unwrap();
    f.jj(&f.source, &["git", "remote", "rename", "origin", "native"]);
    let goal = f.advance();
    ok(f.command("git", &f.remote)
        .args(["branch", "unselected"])
        .output()
        .unwrap());
    let output = ok(f.jw().args(["up"]).output().unwrap());
    assert!(output.contains("OK worker"));
    assert_eq!(f.id(&f.target, "@-"), goal);
    assert_eq!(f.id(&f.source, "dev@native"), goal);
    assert!(
        f.jj(&f.source, &["bookmark", "list", "--all-remotes"])
            .lines()
            .all(|l| !l.contains("unselected"))
    );
    assert_eq!(f.receipt()["source_selection"]["kind"], "Tracked");
}

#[test]
fn bare_variant_up_uses_untracked_exact_remote_without_creating_tracking_state() {
    let Some(f) = Fixture::new() else {
        return;
    };
    fs::remove_file(f.source.join(".jwreseed.local.toml")).unwrap();
    f.jj(&f.source, &["bookmark", "untrack", "dev@origin"]);
    let goal = f.advance();
    ok(f.jw().args(["up"]).output().unwrap());
    assert_eq!(f.id(&f.target, "@-"), goal);
    let receipt = f.receipt();
    assert_eq!(receipt["source_selection"]["kind"], "ExactRemote");
    assert_eq!(receipt["source_selection"]["tracked"], false);
}

#[test]
fn source_local_work_blocks_even_above_an_empty_scratch() {
    for committed in [false, true] {
        let Some(f) = Fixture::new() else {
            return;
        };
        fs::write(f.source.join("local-source"), "unpublished\n").unwrap();
        if committed {
            f.jj(&f.source, &["describe", "-m", "unpublished source"]);
            f.jj(&f.source, &["new"]);
        }
        let target = f.id(&f.target, "@");
        f.advance();
        fails(f.freshen(&[]), "source contains unpublished work");
        assert_eq!(f.id(&f.source, "dev@origin"), f.baseline);
        assert_eq!(f.id(&f.target, "@"), target);
        assert!(!f.log.exists());
        assert!(f.active().is_empty());
    }
}

#[test]
fn up_freezes_optout_and_preserves_feature_bookmarks_without_moving_them() {
    for disabled in [false, true] {
        let Some(f) = Fixture::new() else {
            return;
        };
        fs::write(f.target.join("target-work"), "unpublished\n").unwrap();
        f.jj(&f.target, &["bookmark", "create", "feature"]);
        let work = f.id(&f.target, "@");
        let goal = f.advance();
        ok(f.freshen(if disabled {
            &["--no-freshen-bookmark"]
        } else {
            &[]
        }));
        assert_eq!(f.id(&f.source, "feature"), work);
        assert_eq!(f.id(&f.target, "@-"), goal);
        let receipt = f.receipt();
        assert_eq!(receipt["request"]["no_freshen_bookmark"], disabled);
        let history = &receipt["plans"][0]["history"];
        assert_eq!(history["work_tip"], work);
        assert_eq!(history["published"], false);
        if disabled {
            assert!(history["bookmark"].is_null());
        } else {
            let name = history["bookmark"]["name"].as_str().unwrap();
            assert!(name.starts_with("freshen-"));
            assert_eq!(f.id(&f.source, name), work);
        }
    }
}

#[test]
fn published_scratch_needs_no_bookmark_but_meaningful_empty_gets_a_new_change() {
    for described in [false, true] {
        let Some(f) = Fixture::new() else {
            return;
        };
        f.policy(false, "");
        if described {
            f.jj(&f.target, &["describe", "-m", "meaningful empty"]);
        }
        let old = f.id(&f.target, "@");
        ok(f.jw().args(["up"]).output().unwrap());
        assert_ne!(f.id(&f.target, "@"), old);
        assert_eq!(f.id(&f.target, "@-"), f.baseline);
        assert_eq!(
            f.jj(
                &f.target,
                &["log", "--no-graph", "-r", "@", "-T", "description"]
            ),
            ""
        );
        let history = f.receipt()["plans"][0]["history"].clone();
        assert_eq!(history["published"], !described);
        assert_eq!(history["bookmark"].is_object(), described);
    }
}

#[test]
fn explicit_group_suppresses_current_and_from_keeps_caller_role() {
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
    let goal = f.advance();
    ok(f.jw().args(["up", "--from"]).arg(&donor).output().unwrap());
    assert_eq!(f.id(&f.target, "@-"), goal);
    assert_eq!(f.id(&donor, "@-"), goal);
    assert_eq!(f.id(&f.source, "@-"), f.baseline);
    f.policy(false, "[groups]\nselected=['donor']\n");
    let target = f.id(&f.target, "@");
    ok(f.jw().args(["up", "--group", "selected"]).output().unwrap());
    assert_eq!(f.id(&f.target, "@"), target);
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
fn failed_installer_requires_full_selection_until_plans_freeze_and_resume_pins_goal() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let gate = f.temp.path().join("allow success");
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
    f.script(&format!("mkdir -p node_modules\nprintf partial > node_modules/package\n[ -f {:?} ] || exit 17\nprintf complete > node_modules/package\n", gate));
    let goal = f.advance();
    let before = f.id(&f.target, "@");
    fails(
        f.freshen(&["second"]),
        "ignored installation may be incomplete",
    );
    assert_eq!(f.id(&f.target, "@"), before);
    assert_eq!(f.active().len(), 1);
    let active = fs::read(&f.active()[0]).unwrap();
    let batch: serde_json::Value = serde_json::from_slice(&active).unwrap();
    assert!(batch["plans"].is_null());
    let source = f.id(&f.source, "@");
    for args in [
        vec!["up", "--resume"],
        vec!["up", "second", "worker", "--resume"],
    ] {
        fails(
            f.jw().args(args).output().unwrap(),
            "all recorded literal target names in original order",
        );
        assert_eq!(f.id(&f.source, "@"), source);
        assert_eq!(f.id(&f.target, "@"), before);
        assert_eq!(fs::read(&f.active()[0]).unwrap(), active);
        assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
    }
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
    ok(f.jw()
        .args(["up", "worker", "second", "--resume"])
        .output()
        .unwrap());
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
fn invalid_command_policy_remote_and_selection_fail_before_fetch() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let old = f.id(&f.source, "dev@origin");
    f.advance();
    for (policy, expected) in [
        ("[freshen]\ncommand=[]\n", "nonempty argv"),
        (
            "[freshen]\ncommand=[]\ncommands=[]\n",
            "cannot both be present",
        ),
        ("[freshen]\n[prepare]\n", "cannot both be present"),
    ] {
        fs::write(
            f.source.join(".jwreseed.local.toml"),
            format!("source_at='dev@origin'\n{policy}"),
        )
        .unwrap();
        fails(f.freshen(&[]), expected);
        assert_eq!(f.id(&f.source, "dev@origin"), old);
    }
    f.policy(true, "");
    fails(
        f.jw().args(["up", "missing"]).output().unwrap(),
        "unknown workspace",
    );
    f.jj(&f.source, &["git", "remote", "remove", "origin"]);
    fails(f.freshen(&[]), "unconfigured remote");
    assert!(!f.log.exists());
}

#[test]
fn tracked_only_freshen_without_command_reports_honestly() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.policy(false, "");
    let goal = f.advance();
    let output = ok(f.jw().args(["up", "worker"]).output().unwrap());
    assert!(output.contains("tracked-source preparation only"));
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert!(!f.log.exists());
}

#[test]
fn command_sequence_resumes_at_failed_step_without_replaying_completed_commands() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let gate = f.temp.path().join("second step allowed");
    f.script(&format!(
        "mkdir -p node_modules\ncase \"$2\" in\nfirst) printf first > node_modules/first ;;\nsecond) printf partial > node_modules/package; [ -f {:?} ] || exit 17; printf complete > node_modules/package ;;\nthird) printf third > node_modules/third ;;\nesac\n", gate
    ));
    let commands = ["first", "second", "third"].map(|step| {
        vec![
            f.installer.to_str().unwrap().to_owned(),
            f.log.to_str().unwrap().to_owned(),
            step.to_owned(),
            "$(touch SHOULD_NOT_EXIST); *".to_owned(),
        ]
    });
    fs::write(
        f.source.join(".jwreseed.local.toml"),
        format!(
            "source_at='dev@origin'\n[freshen]\nremote='origin'\ncommands={}\n",
            serde_json::to_string(&commands).unwrap()
        ),
    )
    .unwrap();
    let goal = f.advance();
    let target = f.id(&f.target, "@");
    fails(f.freshen(&[]), "step 2");
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(&f.active()[0]).unwrap()).unwrap();
    assert_eq!(record["schema_version"], 3);
    assert_eq!(record["command_sequence"]["completed"], 1);
    assert_eq!(record["command_sequence"]["in_flight"], true);
    assert_eq!(
        record["command_sequence"]["argv"],
        serde_json::to_value(&commands).unwrap()
    );
    assert_eq!(f.id(&f.target, "@"), target);
    assert!(!f.target.join("node_modules").exists());
    assert!(!f.source.join("node_modules/third").exists());
    assert_eq!(
        fs::read_to_string(f.source.join("node_modules/package")).unwrap(),
        "partial"
    );
    let before_preview = fs::read(&f.active()[0]).unwrap();
    let preview = ok(f
        .jw()
        .args(["up", "worker", "--resume", "--dry-run"])
        .output()
        .unwrap());
    assert!(preview.contains("commands completed 1/3"), "{preview}");
    assert_eq!(fs::read(&f.active()[0]).unwrap(), before_preview);
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 6);
    fs::write(&gate, "ok").unwrap();
    fs::write(f.remote.join("version"), "newer than frozen goal").unwrap();
    f.git_commit();
    let resumed = f.resume();
    assert!(text(&resumed).contains("not exactly-once"));
    ok(resumed);
    assert_eq!(f.id(&f.source, "@-"), goal);
    let log = fs::read_to_string(&f.log).unwrap();
    assert_eq!(
        log.lines().skip(1).step_by(3).collect::<Vec<_>>(),
        ["first", "second", "second", "third"]
    );
    assert!(
        log.lines()
            .step_by(3)
            .all(|cwd| cwd == f.source.to_str().unwrap())
    );
    assert!(!f.source.join("SHOULD_NOT_EXIST").exists());
    assert_eq!(
        fs::read_to_string(f.target.join("node_modules/package")).unwrap(),
        "complete"
    );
    assert_eq!(
        fs::read_to_string(f.target.join("node_modules/third")).unwrap(),
        "third"
    );
    assert!(f.active().is_empty());
}

#[test]
fn unacknowledged_successful_command_is_explicitly_retried_after_interruption() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let marker = f.temp.path().join("command side effect happened");
    f.script(&format!(
        "mkdir -p node_modules\nprintf ready > node_modules/package\ntouch {:?}\n",
        marker
    ));
    f.advance();
    let bin = f.wrapper_at(&format!("[ -f {:?} ] && $status", marker), true);
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    fails(
        command
            .args(["up", "worker", "--copy-ignored", "node_modules"])
            .output()
            .unwrap(),
        "source preparation stopped",
    );
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(&f.active()[0]).unwrap()).unwrap();
    assert_eq!(record["command_sequence"]["completed"], 0);
    assert_eq!(record["command_sequence"]["in_flight"], true);
    assert!(marker.exists());
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
    let resumed = f.resume();
    assert!(text(&resumed).contains("prior attempt may have performed external side effects"));
    ok(resumed);
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 6);
    assert!(f.active().is_empty());
}

#[test]
fn no_command_freshen_borrows_existing_ignored_payload_without_claiming_a_build() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.policy(false, "");
    fs::create_dir_all(f.config.join("jj-waltz")).unwrap();
    fs::write(
        f.config.join("jj-waltz/config.toml"),
        "copy_ignored=['**/node_modules', '**/build/']\n",
    )
    .unwrap();
    fs::create_dir(f.source.join("node_modules")).unwrap();
    let payload = vec![b'e'; 128 * 1024];
    fs::write(f.source.join("node_modules/package"), &payload).unwrap();
    let goal = f.advance();
    let output = ok(f.jw().args(["up", "worker"]).output().unwrap());
    assert!(
        output.contains("existing data, not freshly built"),
        "{output}"
    );
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert!(!f.log.exists());
    assert_eq!(
        fs::read(f.target.join("node_modules/package")).unwrap(),
        payload
    );
    assert_ne!(
        f.inode(&f.source.join("node_modules/package")),
        f.inode(&f.target.join("node_modules/package"))
    );
    assert_shared_extents(
        &f.source.join("node_modules/package"),
        &f.target.join("node_modules/package"),
    );
}

#[test]
fn legacy_prepare_v1_pending_command_migrates_only_on_real_resume() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let policy = f.source.join(".jwreseed.local.toml");
    fs::write(
        &policy,
        fs::read_to_string(&policy)
            .unwrap()
            .replace("[freshen]", "[prepare]"),
    )
    .unwrap();
    let gate = f.temp.path().join("legacy command allowed");
    f.script(&format!("mkdir -p node_modules\nprintf partial > node_modules/package\n[ -f {:?} ] || exit 17\nprintf legacy > node_modules/package\n", gate));
    f.advance();
    let failed = f.freshen(&[]);
    assert!(text(&failed).contains("[prepare] is a deprecated local alias for [freshen]"));
    fails(failed, "command failed");
    let path = f.downgrade_single_command_journal_to_v1();
    let frozen = fs::read(&path).unwrap();
    let preview = ok(f
        .jw()
        .args(["reseed", "worker", "--resume", "--dry-run"])
        .output()
        .unwrap());
    assert!(
        preview.contains("v1 single-command journal loaded compatibly"),
        "{preview}"
    );
    assert!(preview.contains("in-flight command may retry"), "{preview}");
    assert_eq!(fs::read(&path).unwrap(), frozen);
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
    fs::write(&gate, "ok").unwrap();
    // Stop after the command result was acknowledged, before target plans freeze.
    fs::write(
        f.source.join(".jwlinks.local.toml"),
        "[[link]]\nsource='shared-link'\ntarget='missing'\nrequired=true\n",
    )
    .unwrap();
    fails(
        f.jw()
            .args(["reseed", "worker", "--resume"])
            .output()
            .unwrap(),
        "required link target will be absent",
    );
    let upgraded: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(upgraded["schema_version"], 2);
    assert!(upgraded.get("command").is_none());
    assert_eq!(upgraded["command_sequence"]["completed"], 1);
    assert_eq!(upgraded["command_sequence"]["in_flight"], false);
    fs::write(
        f.source.join(".jwlinks.local.toml"),
        "[[link]]\nsource='shared-link'\ntarget='shared'\nrequired=true\n",
    )
    .unwrap();
    ok(f.jw()
        .args(["reseed", "worker", "--resume"])
        .output()
        .unwrap());
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 6);
}

#[test]
fn legacy_prepare_v1_verified_command_and_payload_scope_are_not_replayed_or_reselected() {
    for frozen_patterns in [
        serde_json::json!([]),
        serde_json::json!(["**/node_modules/"]),
    ] {
        let Some(f) = Fixture::new() else {
            return;
        };
        let policy = f.source.join(".jwreseed.local.toml");
        fs::write(
            &policy,
            fs::read_to_string(&policy)
                .unwrap()
                .replace("[freshen]", "[prepare]"),
        )
        .unwrap();
        fs::write(
            f.source.join(".jwlinks.local.toml"),
            "[[link]]\nsource='shared-link'\ntarget='missing'\nrequired=true\n",
        )
        .unwrap();
        f.advance();
        fails(
            f.jw().args(["up", "worker"]).output().unwrap(),
            "required link target will be absent",
        );
        let path = f.downgrade_single_command_journal_to_v1();
        let mut v1: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(v1["phase"], "Verified");
        assert_eq!(v1["request"]["copy_ignored"], serde_json::json!([]));
        assert_eq!(v1["verified_source"]["ignored"], serde_json::json!([]));
        // The former matcher could verify an empty selection for a trailing-slash
        // wildcard even though the directory existed. Freeze that recorded scope,
        // rather than adopting an additional payload under today's fixed matcher.
        v1["request"]["copy_ignored"] = frozen_patterns;
        fs::write(&path, serde_json::to_vec_pretty(&v1).unwrap()).unwrap();
        // A legacy recorded [] is a frozen empty selection, not permission to pick
        // up newly introduced global defaults during recovery.
        fs::create_dir_all(f.config.join("jj-waltz")).unwrap();
        fs::write(
            f.config.join("jj-waltz/config.toml"),
            "copy_ignored=['**/node_modules']\n",
        )
        .unwrap();
        fs::write(
            f.source.join(".jwlinks.local.toml"),
            "[[link]]\nsource='shared-link'\ntarget='shared'\nrequired=true\n",
        )
        .unwrap();
        ok(f.jw()
            .args(["reseed", "worker", "--resume"])
            .output()
            .unwrap());
        assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
        assert!(!f.target.join("node_modules").exists());
        assert!(f.active().is_empty());
    }
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
        f.jw().args(["up", "--group", "empty"]).output().unwrap(),
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
    let goal = f.advance();
    let bin = f.wrapper("[ \"$new\" = true ]");
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    fails(
        command
            .args(["up", "worker", "--copy-ignored", "node_modules"])
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
    fs::write(f.target.join("worker-work"), "unpublished worker\n").unwrap();
    fs::write(second.join("second-work"), "unpublished second\n").unwrap();
    let goal = f.advance();
    let bin = f.wrapper(&format!(
        "[ \"$clear\" = true ] && [ \"$PWD\" = {:?} ]",
        f.target
    ));
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    let output = fails(
        command
            .args(["up", "worker", "second", "--copy-ignored", "node_modules"])
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
    let bookmarks = batch["plans"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            let b = &p["history"]["bookmark"];
            let name = b["name"].as_str().unwrap().to_owned();
            let tip = b["tip"].as_str().unwrap().to_owned();
            assert_eq!(
                f.id(&f.source, &name),
                tip,
                "bookmarked before target clearing"
            );
            (name, tip)
        })
        .collect::<Vec<_>>();
    assert_ne!(bookmarks[0].0, bookmarks[1].0);
    fs::write(f.remote.join("version"), "newer remote\n").unwrap();
    f.git_commit();
    let partial = f.id(&f.target, "@");
    let completed = f.id(&second, "@");
    let source = f.id(&f.source, "@");
    let pending = fs::read(&f.pending()[0]).unwrap();
    let recorded = fs::read(&f.active()[0]).unwrap();
    // A frozen subset may use its completion receipt without touching the
    // unselected, half-filled worker or replaying shared source preparation.
    let output = ok(f.jw().args(["up", "second", "--resume"]).output().unwrap());
    assert!(output.contains("OK second"));
    assert!(!output.contains("OK worker"));
    assert_eq!(f.id(&f.target, "@"), partial);
    assert!(!f.target.join("shared").exists());
    assert_eq!(fs::read(&f.pending()[0]).unwrap(), pending);
    assert_eq!(fs::read(&f.active()[0]).unwrap(), recorded);

    // Bare resume means this current frozen target only, not the whole batch.
    let output = ok(f.jw().args(["up", "--resume"]).output().unwrap());
    assert!(output.contains("OK worker"));
    assert!(!output.contains("OK second"));
    assert_eq!(f.id(&second, "@"), completed);
    assert_eq!(f.id(&f.source, "@"), source);
    assert_eq!(f.id(&f.source, "dev@origin"), goal, "must not refetch");
    assert_eq!(f.id(&f.target, "@-"), goal, "use the frozen target base");
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
    assert!(f.active().is_empty());
    assert!(f.pending().is_empty());
    for (name, tip) in bookmarks {
        assert_eq!(f.id(&f.source, &name), tip);
    }
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
        f.jw().args(["up", "second"]).output().unwrap(),
        "borrowed by unfinished target worker",
    );
    assert_eq!(f.id(&f.source, "@"), source);
    assert!(!f.log.exists());
    ok(f.jw()
        .args(["reseed", "worker", "--resume"])
        .output()
        .unwrap());
}

#[test]
fn frozen_handoff_rejects_foreign_pending_without_overwriting_it() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.advance();
    let bin = f.wrapper("[ \"$clear\" = true ]");
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    fails(
        command.args(["up", "worker"]).output().unwrap(),
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
    // Restoring this batch's actual pending record permits recorded recovery.
    ok(f.resume());
    assert!(f.active().is_empty());
}

#[test]
fn missing_handoff_record_uses_atomic_intent_not_an_unrelated_old_receipt() {
    let Some(f) = Fixture::new() else {
        return;
    };
    ok(f.jw().args(["reseed", "worker"]).output().unwrap());
    f.advance();
    let bin = f.wrapper_at(&format!("[ \"$sparse\" = true ] && [ \"$listing\" = true ] && [ \"$PWD\" = {:?} ] && [ -f {:?} ]", f.target, f.source.join("node_modules/package")), true);
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    fails(
        command
            .args(["up", "worker", "--copy-ignored", "node_modules"])
            .output()
            .unwrap(),
        "PENDING worker",
    );
    assert!(f.target.join("shared").is_file());
    let before = f.id(&f.target, "@");
    let batch: serde_json::Value =
        serde_json::from_slice(&fs::read(&f.active()[0]).unwrap()).unwrap();
    assert_eq!(batch["plans"][0]["phase"], "Prepared");
    // Simulate crash after atomic intent but before target handoff. No target
    // filesystem mutation occurred, so the original Prepared stamp still holds.
    fs::remove_file(&f.pending()[0]).unwrap();
    ok(f.resume());
    assert!(f.active().is_empty());
    assert_ne!(f.id(&f.target, "@"), before);
    assert_eq!(f.id(&f.target, "@-"), f.id(&f.source, "@-"));
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
    assert_eq!(
        fs::read_to_string(f.target.join("node_modules/package")).unwrap(),
        "installed\n"
    );
}

#[test]
fn lost_successful_fetch_before_goal_freeze_refetches_only_unfrozen_intent() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.advance();
    let bin = f.wrapper("[ \"$fetch\" = true ]");
    let mut command = f.jw();
    f.with_wrapper(&mut command, &bin);
    fails(
        command
            .args(["up", "worker", "--copy-ignored", "node_modules"])
            .output()
            .unwrap(),
        "phase Fetching",
    );
    let batch: serde_json::Value =
        serde_json::from_slice(&fs::read(&f.active()[0]).unwrap()).unwrap();
    assert!(batch["goal"].is_null());
    assert!(!f.log.exists());
    fs::write(f.remote.join("version"), "latest before goal freeze\n").unwrap();
    f.git_commit();
    let latest = ok(f
        .command("git", &f.remote)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap());
    ok(f.resume());
    assert_eq!(f.id(&f.source, "@-"), latest);
    assert_eq!(fs::read_to_string(&f.log).unwrap().lines().count(), 3);
}

#[test]
fn stale_target_refuses_freshen_before_fetch_or_installer() {
    let Some(f) = Fixture::new() else {
        return;
    };
    f.advance();
    f.jj(
        &f.source,
        &["restore", "--into", "worker@", "--from", "root()"],
    );
    fails(f.freshen(&[]), "stale");
    assert_eq!(f.id(&f.source, "dev@origin"), f.baseline);
    assert!(!f.log.exists());
    assert!(f.active().is_empty());
}

#[test]
fn freshen_mutates_only_configured_or_explicit_donor_from_sibling_subdirectory() {
    let Some(f) = Fixture::new() else {
        return;
    };
    let donor = f.temp.path().join("configured donor");
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
    f.policy(true, &format!("source={:?}\n", donor));
    let primary_before = f.id(&f.source, "@");
    let goal = f.advance();
    fs::create_dir_all(f.target.join("subdirectory")).unwrap();
    ok(f.jw()
        .current_dir(f.target.join("subdirectory"))
        .args(["up", "worker", "--copy-ignored", "node_modules"])
        .output()
        .unwrap());
    assert_eq!(f.id(&donor, "@-"), goal);
    assert_eq!(f.id(&f.source, "@"), primary_before);
    assert_eq!(
        fs::read_to_string(&f.log).unwrap().lines().next().unwrap(),
        donor.to_str().unwrap()
    );
    let donor_before = f.id(&donor, "@");
    ok(f.jw()
        .args(["up", "worker", "--from"])
        .arg(&f.source)
        .args(["--copy-ignored", "node_modules"])
        .output()
        .unwrap());
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert_eq!(f.id(&donor, "@"), donor_before);
    assert_eq!(
        fs::read_to_string(&f.log).unwrap().lines().nth(3).unwrap(),
        f.source.to_str().unwrap()
    );
}

#[test]
fn ordinary_colocated_primary_can_be_freshened_without_root_replacement() {
    let Some(f) = Fixture::new_with_colocation(true) else {
        return;
    };
    let root = f.inode(&f.source);
    let jj = f.inode(&f.source.join(".jj"));
    let git = f.inode(&f.source.join(".git"));
    let goal = f.advance();
    ok(f.freshen(&[]));
    assert_eq!(f.id(&f.source, "@-"), goal);
    assert_eq!(f.inode(&f.source), root);
    assert_eq!(f.inode(&f.source.join(".jj")), jj);
    assert_eq!(f.inode(&f.source.join(".git")), git);
}

// Match the existing reseed acceptance proof: equal bytes and distinct inodes
// alone do not demonstrate actual physical CoW sharing.
#[cfg(target_os = "linux")]
fn assert_shared_extents(source: &Path, target: &Path) {
    use std::os::fd::AsRawFd;
    #[repr(C)]
    #[derive(Default, Debug)]
    struct Extent {
        logical: u64,
        physical: u64,
        length: u64,
        reserved: [u64; 2],
        flags: u32,
        reserved2: [u32; 3],
    }
    #[repr(C)]
    #[derive(Default)]
    struct Map {
        start: u64,
        length: u64,
        flags: u32,
        mapped: u32,
        count: u32,
        reserved: u32,
        extent: Extent,
    }
    fn extent(path: &Path) -> Extent {
        let file = fs::File::open(path).unwrap();
        let mut map = Map {
            length: u64::MAX,
            flags: 1,
            count: 1,
            ..Map::default()
        };
        assert_eq!(
            unsafe { libc::ioctl(file.as_raw_fd(), 0xc020660b as libc::c_ulong, &mut map) },
            0,
            "FIEMAP {path:?}: {}",
            std::io::Error::last_os_error()
        );
        assert_eq!(map.mapped, 1);
        map.extent
    }
    let source = extent(source);
    let target = extent(target);
    assert_eq!(
        source.physical, target.physical,
        "freshen must actually share extents"
    );
    assert_ne!(source.flags & 0x2000, 0, "{source:?}");
    assert_ne!(target.flags & 0x2000, 0, "{target:?}");
}
#[cfg(not(target_os = "linux"))]
fn assert_shared_extents(_source: &Path, _target: &Path) {}

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
    let original_verified = batch["verified_source"]["wc_commit"]
        .as_str()
        .unwrap()
        .to_owned();
    let original_change = batch["verified_source"]["change_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let goal = batch["goal"].as_str().unwrap();
    fs::write(
        f.source.join("lockfile"),
        "tracked edit after verification\n",
    )
    .unwrap();
    f.jj(&f.source, &["status"]);
    f.jj(&f.source, &["restore", "--from", goal]);
    let restored = f.id(&f.source, "@");
    assert_ne!(restored, original_verified);
    assert_eq!(
        f.jj(
            &f.source,
            &[
                "log",
                "--ignore-working-copy",
                "--no-graph",
                "-r",
                "@",
                "-T",
                "change_id"
            ]
        ),
        original_change
    );
    ok(f.resume());
    let batch_receipt = fs::read_dir(f.journal())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(".receipt.json")
        })
        .map(|p| serde_json::from_slice::<serde_json::Value>(&fs::read(p).unwrap()).unwrap())
        .find(|v| v["plans"].is_array())
        .unwrap();
    assert_eq!(
        batch_receipt["verified_source"]["wc_commit"],
        original_verified
    );
    assert_eq!(batch_receipt["accepted_verified_wc_commit"], restored);
    assert_eq!(fs::read_to_string(f.log).unwrap().lines().count(), 3);
}
