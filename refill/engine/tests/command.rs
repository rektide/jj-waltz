//! Bounded native/producer evidence, isolated process environments and one donor
//! with at most two receivers. Not a contextual ordinary-transfer/cohort proof.
#![cfg(unix)]
use jj_waltz_native_jj::jj::commit_id_revset;
use jj_waltz_refill_engine::{PrepareRequest, ReseedRequest, command::*};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}
struct Fixture {
    temp: tempfile::TempDir,
    source: PathBuf,
    s: String,
}
impl Fixture {
    fn new() -> Self {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-command-tests");
        fs::create_dir_all(&scratch).unwrap();
        let temp = tempfile::tempdir_in(scratch.canonicalize().unwrap()).unwrap();
        let source = temp.path().join("source");
        let mut f = Self {
            temp,
            source,
            s: String::new(),
        };
        f.jj(f.temp.path(), &["git", "init", "--no-colocate", f.source.to_str().unwrap()]);
        fs::write(f.source.join(".gitignore"), "cache/\n.env\n.jwlinks.toml\n.jwreseed.local.toml\n").unwrap();
        fs::write(f.source.join("a"), "baseline\n").unwrap();
        fs::write(f.source.join("zz"), "last leaf\n").unwrap();
        f.jj(&f.source, &["status"]);
        f.s = f.jj(&f.source, &["--ignore-working-copy", "log", "--no-graph", "-r", "@", "-T", "commit_id"]);
        f
    }
    fn command(&self, program: impl AsRef<std::ffi::OsStr>, root: &Path) -> Command {
        let mut c = Command::new(program);
        c.current_dir(root)
            .env("JJ_CONFIG", "")
            .env("JJ_USER", "Command test")
            .env("JJ_EMAIL", "command@example.invalid")
            .env("JJ_EDITOR", "true")
            .env("XDG_CONFIG_HOME", self.temp.path().join("config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env_remove("GIT_CONFIG_COUNT")
            .env_remove("GIT_CONFIG_PARAMETERS");
        c
    }
    fn jj(&self, root: &Path, args: &[&str]) -> String {
        success(self.command("jj", root).args(args).output().unwrap())
    }
    fn receiver(&self, name: &str) -> PathBuf {
        let root = self.temp.path().join(name);
        self.jj(
            &self.source,
            &[
                "workspace",
                "add",
                "--no-colocate",
                "--name",
                name,
                "-r",
                &commit_id_revset(&self.s).unwrap(),
                root.to_str().unwrap(),
            ],
        );
        root
    }
    fn request(&self, names: &[&str]) -> ReseedRequest {
        ReseedRequest {
            names: names.iter().map(|n| n.to_string()).collect(),
            source: Some(self.source.clone()),
            copy_ignored: Some(vec![]),
            allow_unbookmarked: true,
            ..Default::default()
        }
    }
    fn publish(&self) {
        let remote = self.temp.path().join("remote");
        success(self.command("git", self.temp.path()).args(["init", "--bare"]).arg(&remote).output().unwrap());
        self.jj(&self.source, &["git", "remote", "add", "origin", remote.to_str().unwrap()]);
        self.jj(&self.source, &["describe", "-m", "published fixture baseline"]);
        self.jj(&self.source, &["bookmark", "create", "trunk"]);
        self.jj(&self.source, &["git", "push", "--bookmark", "trunk"]);
    }
    fn settings(&self, commands: Vec<Vec<&str>>) {
        fs::write(
            self.source.join(".jwreseed.local.toml"),
            format!("[freshen]\ncommands={}\n", serde_json::to_string(&commands).unwrap()),
        )
        .unwrap();
    }
    fn run(&self, mode: &str, request: &ReseedRequest, fail_clear: Option<&Path>) -> (Value, Vec<Vec<String>>) {
        let bin = self.temp.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        let real = success(Command::new("sh").args(["-c", "command -v jj"]).output().unwrap());
        let script = format!(
            "#!/bin/sh\nprintf '%s\\t' \"$PWD\" \"$@\" >> \"$JW_COMMAND_TRACE\"\nprintf '\\n' >> \"$JW_COMMAND_TRACE\"\nif [ \"$PWD\" = \"$JW_FAIL_CLEAR\" ]; then\n for arg in \"$@\"; do if [ \"$arg\" = --clear ]; then echo 'injected native clear failure' >&2; exit 73; fi; done\nfi\nexec '{}' \"$@\"\n",
            real.replace('\'', "'\\''")
        );
        fs::write(bin.join("jj"), script).unwrap();
        fs::set_permissions(bin.join("jj"), fs::Permissions::from_mode(0o755)).unwrap();
        let trace = self.temp.path().join("trace");
        let report = self.temp.path().join("report.json");
        let mut cmd = self.command(std::env::current_exe().unwrap(), &self.source);
        cmd.args(["--exact", "execute_command_child", "--ignored", "--nocapture"])
            .env("JW_COMMAND_MODE", mode)
            .env("JW_COMMAND_REQUEST", serde_json::to_string(request).unwrap())
            .env("JW_COMMAND_TRACE", &trace)
            .env("JW_COMMAND_REPORT", &report)
            .env("JW_FAIL_CLEAR", fail_clear.unwrap_or(Path::new("")))
            .env("PATH", format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()));
        success(cmd.output().unwrap());
        let report = serde_json::from_slice(&fs::read(report).unwrap()).unwrap();
        let calls = fs::read_to_string(trace)
            .unwrap()
            .lines()
            .map(|l| l.trim_end_matches('\t').split('\t').map(str::to_owned).collect())
            .collect();
        (report, calls)
    }
}

struct NoOrdinary;
impl OrdinaryTransfer for NoOrdinary {
    fn admit(&self, _receivers: usize, enabled: bool) -> anyhow::Result<()> {
        anyhow::ensure!(!enabled, "fixture has no ordinary adapter");
        Ok(())
    }
    fn transfer(&mut self, _: &OrdinaryCohort, _: &mut [OrdinaryOutcome]) -> Result<(), OrdinaryFailure> {
        panic!("empty policy must not enter transfer")
    }
}
fn report_value(report: &CommandReport) -> Value {
    json!({"source": report.source.as_ref().map(|s| json!({"commit": s.commit, "goal": s.upstream_goal, "evidence": format!("{:?}", s.evidence)})),
        "trees": report.capture.distinct_trees, "cache_reads": report.capture.donor_cache_reads,
        "receivers": report.receivers.iter().map(|r| json!({"name": r.name, "status": format!("{:?}", r.status), "phase": r.reached.to_string(), "old": r.old.as_ref().map(|o| &o.commit),
            "child": r.child.as_ref().map(|c| &c.commit_id), "change": r.child.as_ref().map(|c| &c.change_id), "native": format!("{:?}", r.native.state), "manifest": r.native.manifest_complete,
            "clones": r.native.installed.files_reflinked, "copies": r.native.installed.files_copied, "ordinary": format!("{:?}", r.ordinary.state), "finish": r.finish.complete,
            "links_created": r.finish.links.linked})).collect::<Vec<_>>()})
}
fn preparation_value(r: &PreparationReport) -> Value {
    json!({"saved": r.saved_commit, "goal": r.upstream_goal, "source": r.source_commit, "steps": r.steps.iter().map(|s| format!("{:?}", s.status)).collect::<Vec<_>>(),
        "final_started": r.final_conformance_started, "final_ok": r.final_conformance_succeeded})
}
#[test]
#[ignore = "isolated child only"]
fn execute_command_child() {
    let mode = std::env::var("JW_COMMAND_MODE").unwrap();
    let request: ReseedRequest = serde_json::from_str(&std::env::var("JW_COMMAND_REQUEST").unwrap()).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let mut diagnostic = |s: &str| eprintln!("{s}");
    let run = || -> anyhow::Result<Value> {
        if mode == "producer" || mode == "producer-preview" || mode == "handoff" {
            let source = ResolvedSource::resolve(
                &cwd,
                PrepareRequest {
                    source: request.source.clone(),
                    source_at: Some("trunk@origin".into()),
                    ..Default::default()
                },
            )?;
            if mode == "producer-preview" {
                let p = source.preview()?;
                return Ok(json!({"ok": true, "result": p.resulting_commit, "goal": p.prospective.unwrap().old_goal}));
            }
            let source = match prepare_source(&source, &mut diagnostic) {
                Ok(s) => s,
                Err(e) => return Ok(json!({"ok": false, "error": format!("{e:#}"), "preparation": preparation_value(&e.completed)})),
            };
            if mode == "producer" {
                return Ok(
                    json!({"ok": true, "source": source.report.commit, "goal": source.report.upstream_goal, "preparation": preparation_value(source.report.preparation.as_ref().unwrap())}),
                );
            }
            let command = ResolvedCommand::resolve(&cwd, request, RenewalRoute::ClearThenPopulate, &mut diagnostic)?;
            return Ok(match command.execute_prepared(source, &mut NoOrdinary, &mut diagnostic) {
                Ok(r) => json!({"ok": true, "report": report_value(&r)}),
                Err(e) => json!({"ok": false, "error": format!("{e:#}"), "report": report_value(&e.report)}),
            });
        }
        let command = ResolvedCommand::resolve(&cwd, request, RenewalRoute::ClearThenPopulate, &mut diagnostic)?;
        if mode == "preview" {
            let p = command.preview()?;
            return Ok(
                json!({"ok": true, "label": p.label, "source_result": p.source.resulting_commit, "assertion": p.source.assertion_pending,
                "desired": format!("{:?}", p.desired), "probed": p.clone_capability_probed, "inspected": p.obstructions_inspected}),
            );
        }
        Ok(match command.execute(&mut NoOrdinary, &mut diagnostic) {
            Ok(r) => json!({"ok": true, "report": report_value(&r)}),
            Err(e) => json!({"ok": false, "error": format!("{e:#}"), "phase": e.location.phase.to_string(), "receiver": e.location.receiver,
                "native_cause": e.source.downcast_ref::<jj_waltz_native_jj::jj::JjCommandError>().is_some(), "report": report_value(&e.report)}),
        })
    };
    let value = match run() {
        Ok(v) => v,
        Err(e) => json!({"ok": false, "error": format!("{e:#}")}),
    };
    fs::write(std::env::var_os("JW_COMMAND_REPORT").unwrap(), serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}
fn calls_named<'a>(calls: &'a [Vec<String>], name: &str) -> Vec<&'a Vec<String>> {
    calls.iter().filter(|c| c.iter().any(|s| s == name)).collect()
}

#[test]
fn vector_native_stage_reuses_trees_and_cache_and_retains_forward_failure() {
    let f = Fixture::new();
    let a = f.receiver("a");
    let b = f.receiver("b");
    fs::write(a.join(".env"), "private").unwrap();
    let source_state = fs::read(f.source.join(".jj/working_copy/tree_state")).unwrap();
    let b_state = fs::read(b.join(".jj/working_copy/tree_state")).unwrap();
    let old_a = f.jj(&a, &["--ignore-working-copy", "log", "--no-graph", "-r", "@", "-T", "commit_id"]);
    let (value, calls) = f.run("execute", &f.request(&["a", "b"]), Some(&b));
    assert_eq!(value["ok"], false, "{value:#}");
    assert_eq!(value["receiver"], "b");
    assert_eq!(value["native_cause"], true);
    let report = &value["report"];
    assert_eq!(report["trees"], 1);
    assert_eq!(report["cache_reads"], 1);
    assert_eq!(report["receivers"][0]["native"], "Established");
    assert_eq!(report["receivers"][0]["status"], "Partial");
    assert_eq!(report["receivers"][0]["finish"], false);
    assert_eq!(report["receivers"][1]["native"], "InProgress");
    assert_eq!(report["receivers"][1]["status"], "Failed");
    assert_eq!(report["receivers"][0]["clones"], 3);
    assert_eq!(report["receivers"][1]["clones"], 0);
    assert_eq!(fs::read(b.join(".jj/working_copy/tree_state")).unwrap(), b_state);
    assert_eq!(fs::read(f.source.join(".jj/working_copy/tree_state")).unwrap(), source_state);
    assert_eq!(fs::read(a.join(".env")).unwrap(), b"private");
    assert_eq!(
        f.jj(&a, &["--ignore-working-copy", "file", "show", "-r", &commit_id_revset(&old_a).unwrap(), "a"]),
        "baseline"
    );
    assert_eq!(calls_named(&calls, "list").iter().filter(|c| c.iter().any(|s| s == "file")).count(), 1);
    let (value, _) = f.run("execute", &f.request(&["a", "b"]), None);
    assert_eq!(value["ok"], true, "{value:#}");
    for (i, root) in [&a, &b].into_iter().enumerate() {
        assert_eq!(value["report"]["receivers"][i]["status"], "Complete");
        assert_eq!(value["report"]["receivers"][i]["clones"], 3, "clean refill renews too");
        assert_ne!(fs::metadata(root.join("a")).unwrap().ino(), fs::metadata(f.source.join("a")).unwrap().ino());
        fs::write(root.join("a"), "private write").unwrap();
        assert_eq!(fs::read(f.source.join("a")).unwrap(), b"baseline\n");
    }
}

#[test]
fn incomplete_manifest_returns_actual_progress_and_never_adopts_source_cache() {
    let f = Fixture::new();
    let a = f.receiver("a");
    fs::remove_file(f.source.join("zz")).unwrap();
    let (value, _) = f.run("execute", &f.request(&["a"]), None);
    assert_eq!(value["ok"], false, "{value:#}");
    assert_eq!(value["phase"], "strict full-S install");
    assert_eq!(value["report"]["source"]["evidence"], "AssumedPrepared");
    assert_eq!(value["report"]["receivers"][0]["clones"], 2);
    assert_eq!(value["report"]["receivers"][0]["manifest"], false);
    assert_eq!(f.jj(&a, &["--at-operation", "@", "--ignore-working-copy", "sparse", "list"]), "");
    assert_eq!(fs::read(a.join("a")).unwrap(), b"baseline\n");
    assert!(!a.join("zz").exists());
}

#[test]
fn producer_checks_only_final_conformance_and_hands_actual_s_to_native_command_once() {
    let f = Fixture::new();
    let a = f.receiver("a");
    f.publish();
    f.settings(vec![
        vec!["sh", "-c", "printf 'intermediate\\n' > a"],
        vec!["sh", "-c", "printf 'baseline\\n' > a; printf built > .env"],
    ]);
    let (value, calls) = f.run(
        "handoff",
        &ReseedRequest {
            prepare: true,
            ..f.request(&["a"])
        },
        None,
    );
    assert_eq!(value["ok"], true, "{value:#}");
    let source = &value["report"]["source"];
    assert_ne!(source["commit"], source["goal"]);
    assert_eq!(source["evidence"], "Produced");
    assert_eq!(fs::read(f.source.join("a")).unwrap(), b"baseline\n");
    assert_eq!(fs::read(f.source.join(".env")).unwrap(), b"built");
    assert_eq!(fs::read(a.join("a")).unwrap(), b"baseline\n");
    let donor = calls.iter().filter(|c| Path::new(&c[0]) == f.source).cloned().collect::<Vec<_>>();
    assert_eq!(calls_named(&donor, "fetch").len(), 1);
    assert_eq!(calls_named(&donor, "status").len(), 1);
    assert_eq!(
        donor
            .iter()
            .filter(|c| c.iter().any(|s| s == "diff") && !c.iter().any(|s| s == "--ignore-working-copy"))
            .count(),
        1
    );
    assert!(!calls_named(&donor, "list").iter().any(|c| c.iter().any(|s| s == "file")) || value["report"]["trees"] == 1);
}

#[test]
fn producer_failure_stops_later_argv_and_final_divergence_is_distinct() {
    let f = Fixture::new();
    f.publish();
    f.settings(vec![vec!["sh", "-c", "printf intermediate > a; exit 23"], vec!["sh", "-c", "touch .env"]]);
    let (value, calls) = f.run("producer", &f.request(&[]), None);
    assert_eq!(value["ok"], false, "{value:#}");
    assert_eq!(value["preparation"]["steps"], json!(["Started", "Selected"]));
    assert_eq!(value["preparation"]["final_started"], false);
    assert!(!f.source.join(".env").exists());
    assert!(!calls.iter().any(|c| c.iter().any(|s| s == "diff")));
    // Restore the tracked file explicitly; no retry/rollback is supplied by JW.
    fs::write(f.source.join("a"), "baseline\n").unwrap();
    f.settings(vec![vec!["sh", "-c", "printf divergent > a"]]);
    let (value, _) = f.run("producer", &f.request(&[]), None);
    assert_eq!(value["ok"], false, "{value:#}");
    assert_eq!(value["preparation"]["steps"], json!(["Succeeded"]));
    assert_eq!(value["preparation"]["final_started"], true);
    assert_eq!(value["preparation"]["final_ok"], false);
    assert_eq!(fs::read(f.source.join("a")).unwrap(), b"divergent");

    f.jj(&f.source, &["new", "trunk@origin"]);
    f.settings(vec![vec!["jj", "new", "@"], vec!["sh", "-c", "touch .env"]]);
    let (value, _) = f.run("producer", &f.request(&[]), None);
    assert_eq!(value["ok"], false, "{value:#}");
    assert!(value["error"].as_str().unwrap().contains("selected revision changed"));
    assert_eq!(value["preparation"]["steps"], json!(["Succeeded", "Selected"]));
    assert_eq!(value["preparation"]["final_started"], false);
    assert!(!f.source.join(".env").exists());
}

fn bytes_under(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut result = Vec::new();
    fn visit(root: &Path, at: &Path, result: &mut Vec<(PathBuf, Vec<u8>)>) {
        let mut children = fs::read_dir(at).unwrap().map(Result::unwrap).collect::<Vec<_>>();
        children.sort_by_key(|e| e.file_name());
        for child in children {
            let p = child.path();
            let m = fs::symlink_metadata(&p).unwrap();
            if m.is_dir() {
                result.push((p.strip_prefix(root).unwrap().to_path_buf(), vec![]));
                visit(root, &p, result);
            } else if m.is_file() {
                result.push((p.strip_prefix(root).unwrap().to_path_buf(), fs::read(p).unwrap()));
            } else {
                result.push((
                    p.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read_link(p).unwrap().as_os_str().as_encoded_bytes().to_vec(),
                ));
            }
        }
    }
    visit(root, root, &mut result);
    result
}

#[test]
fn preview_is_planned_and_read_only_including_prepare_and_pending_assertion() {
    let f = Fixture::new();
    let a = f.receiver("a");
    fs::write(a.join("a"), "unsaved target\n").unwrap();
    fs::write(f.source.join("a"), "unsaved donor\n").unwrap();
    let before_source = bytes_under(&f.source);
    let before_a = bytes_under(&a);
    let config = f.temp.path().join("config");
    let before_config = config.exists().then(|| bytes_under(&config));
    let request = ReseedRequest {
        dry_run: true,
        source_at: Some(f.s.clone()),
        ..f.request(&["a"])
    };
    let (value, calls) = f.run("preview", &request, None);
    assert_eq!(value["ok"], true, "{value:#}");
    assert!(value["label"].as_str().unwrap().starts_with("PLANNED"));
    assert_eq!(value["source_result"], Value::Null);
    assert_eq!(value["assertion"], true);
    assert_eq!(value["probed"], false);
    assert_eq!(value["inspected"], false);
    assert_eq!(before_source, bytes_under(&f.source));
    assert_eq!(before_a, bytes_under(&a));
    assert_eq!(before_config, config.exists().then(|| bytes_under(&config)));
    for name in ["status", "diff", "fetch", "new", "show"] {
        assert!(calls_named(&calls, name).is_empty(), "{name}: {calls:#?}");
    }
    assert!(
        !calls
            .iter()
            .any(|c| c.windows(2).any(|p| p == ["file", "list"]) || c.windows(2).any(|p| p == ["config", "path"]))
    );
    // Prepare preview uses recorded source facts; it does not snapshot these
    // unsaved edits or run a builder, even when its source selection is blocked.
    fs::write(f.source.join("a"), "baseline\n").unwrap();
    f.publish();
    f.settings(vec![vec!["sh", "-c", "touch .env"]]);
    let before = bytes_under(&f.source);
    let config_before = bytes_under(&f.temp.path().join("config"));
    let (value, calls) = f.run(
        "preview",
        &ReseedRequest {
            dry_run: true,
            prepare: true,
            ..f.request(&["a"])
        },
        None,
    );
    assert_eq!(value["ok"], true, "{value:#}");
    assert_eq!(value["source_result"], Value::Null);
    assert_eq!(value["desired"], "FuturePreparedSource");
    assert_eq!(before, bytes_under(&f.source));
    assert_eq!(config_before, bytes_under(&f.temp.path().join("config")));
    assert!(!f.source.join(".env").exists());
    assert!(calls_named(&calls, "fetch").is_empty());
    assert!(!calls.iter().any(|c| c.iter().any(|s| s == "status" || s == "--clear")));
    let (value, _) = f.run(
        "preview",
        &ReseedRequest {
            dry_run: true,
            prepare: true,
            at: Some("post-fetch-only-ref".into()),
            ..f.request(&["a"])
        },
        None,
    );
    assert_eq!(value["ok"], true, "{value:#}");
    assert_eq!(value["desired"], "PostPreparationRevset(\"post-fetch-only-ref\")");
    assert_eq!(before, bytes_under(&f.source));
}

#[test]
fn source_only_resolution_does_not_load_receivers_links_or_payload() {
    let f = Fixture::new();
    let a = f.receiver("a");
    f.publish();
    fs::write(f.source.join(".jwlinks.toml"), "deliberately invalid link configuration").unwrap();
    fs::create_dir_all(f.source.join("cache")).unwrap();
    let _socket = std::os::unix::net::UnixListener::bind(f.source.join("cache/socket")).unwrap();
    f.jj(&a, &["sparse", "set", "--clear"]);
    let before = fs::read(a.join(".jj/working_copy/tree_state")).unwrap();
    let (value, calls) = f.run("producer", &f.request(&[]), None);
    assert_eq!(value["ok"], true, "{value:#}");
    assert_eq!(fs::read(a.join(".jj/working_copy/tree_state")).unwrap(), before);
    assert!(!calls.iter().any(|c| Path::new(&c[0]) == a));
    assert!(!calls.iter().any(|c| c.windows(3).any(|p| p == ["root", "--name", "a"])));
    assert!(
        !calls
            .iter()
            .any(|c| c.windows(2).any(|p| p == ["file", "list"]) || c.windows(2).any(|p| p == ["file", "show"]))
    );
}

#[test]
fn divergent_operation_preview_refuses_without_reconciliation() {
    let f = Fixture::new();
    f.receiver("a");
    let op = f.jj(
        &f.source,
        &[
            "--at-operation",
            "@",
            "--ignore-working-copy",
            "op",
            "log",
            "--no-graph",
            "--limit",
            "1",
            "-T",
            "id",
        ],
    );
    f.jj(&f.source, &["--at-operation", &op, "describe", "-m", "left"]);
    f.jj(&f.source, &["--at-operation", &op, "describe", "-m", "right"]);
    let before = bytes_under(&f.source);
    let (value, _) = f.run(
        "preview",
        &ReseedRequest {
            dry_run: true,
            ..f.request(&["a"])
        },
        None,
    );
    assert_eq!(value["ok"], false, "{value:#}");
    assert_eq!(before, bytes_under(&f.source), "even divergent operation reconciliation must not write");
}

#[test]
fn preservation_barrier_refuses_a_coordinate_rewritten_by_later_snapshot() {
    let f = Fixture::new();
    let b = f.receiver("b");
    let a = f.receiver("a");
    f.jj(&a, &["new", "b@"]);
    let old_a = f.jj(
        &a,
        &[
            "--at-operation",
            "@",
            "--ignore-working-copy",
            "log",
            "--no-graph",
            "-r",
            "@",
            "-T",
            "commit_id",
        ],
    );
    fs::write(b.join("a"), "unsaved ancestor change\n").unwrap();
    let (value, calls) = f.run("execute", &f.request(&["a", "b"]), None);
    assert_eq!(value["ok"], false, "{value:#}");
    assert!(
        value["error"].as_str().unwrap().contains("saved receiver O moved during preservation"),
        "{value:#}"
    );
    assert_eq!(value["report"]["receivers"][0]["old"], old_a);
    assert!(!calls.iter().any(|c| c.iter().any(|s| s == "--clear")));
    assert_eq!(value["report"]["receivers"][0]["clones"], 0);
}

#[test]
fn preview_refuses_native_secure_config_migration_without_creating_metadata() {
    let f = Fixture::new();
    f.receiver("a");
    let legacy = f.source.join(".jj/repo/config.toml");
    assert!(!f.source.join(".jj/repo/config-id").exists());
    fs::write(&legacy, "[user]\nname='legacy'\n").unwrap();
    let before = bytes_under(&f.source);
    let (value, calls) = f.run(
        "preview",
        &ReseedRequest {
            dry_run: true,
            ..f.request(&["a"])
        },
        None,
    );
    assert_eq!(value["ok"], false, "{value:#}");
    assert!(value["error"].as_str().unwrap().contains("would require migration"));
    assert_eq!(before, bytes_under(&f.source));
    assert!(!f.source.join(".jj/repo/config-id").exists());
    assert!(!calls.iter().any(|c| c.iter().any(|s| s == "operation" || s == "workspace")));
}

#[test]
fn preservation_barrier_refuses_donor_rewrite_without_implicit_resnapshot() {
    let f = Fixture::new();
    let a = f.receiver("a");
    f.jj(&f.source, &["new", "a@"]);
    let s = f.jj(
        &f.source,
        &[
            "--at-operation",
            "@",
            "--ignore-working-copy",
            "log",
            "--no-graph",
            "-r",
            "@",
            "-T",
            "commit_id",
        ],
    );
    fs::write(a.join("a"), "unsaved ancestor change\n").unwrap();
    let (value, calls) = f.run("execute", &f.request(&["a"]), None);
    assert_eq!(value["ok"], false, "{value:#}");
    assert!(
        value["error"].as_str().unwrap().contains("donor coordinate changed during receiving snapshots"),
        "{value:#}"
    );
    assert_eq!(value["report"]["source"]["commit"], s);
    assert!(!calls.iter().any(|c| c.iter().any(|s| s == "--clear")));
    assert!(
        !calls
            .iter()
            .any(|c| Path::new(&c[0]) == f.source && !c.iter().any(|s| s == "--ignore-working-copy")),
        "no live donor resnapshot/repin"
    );
}

#[test]
fn later_finish_failure_keeps_earlier_complete_receiver_and_actual_link_count() {
    let f = Fixture::new();
    let a = f.receiver("a");
    let b = f.receiver("b");
    fs::create_dir(a.join("cache")).unwrap();
    fs::write(a.join("cache/tool"), "local tool\n").unwrap();
    fs::write(f.source.join(".jwlinks.toml"), "[[link]]\nsource='.env'\ntarget='cache/tool'\nrequired=true\n").unwrap();
    let (value, _) = f.run("execute", &f.request(&["a", "b"]), None);
    assert_eq!(value["ok"], false, "{value:#}");
    assert_eq!(value["receiver"], "b");
    assert_eq!(value["phase"], "loaded link finish");
    assert_eq!(value["report"]["receivers"][0]["status"], "Complete");
    assert_eq!(value["report"]["receivers"][0]["links_created"], 1);
    assert_eq!(value["report"]["receivers"][1]["status"], "Failed");
    assert_eq!(value["report"]["receivers"][1]["native"], "Established");
    assert_eq!(value["report"]["receivers"][1]["links_created"], 0);
    assert!(fs::symlink_metadata(a.join(".env")).unwrap().is_symlink());
    assert_eq!(fs::read(a.join(".env")).unwrap(), b"local tool\n");
    assert!(!b.join(".env").exists());
}
