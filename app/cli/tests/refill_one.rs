//! Native protocol regressions, migrated from the temporary N1 export to jw.
//! Child test processes isolate JJ configuration and argv tracing without any
//! process-global environment mutation or operator-workspace effects.
#![cfg(unix)]
use jj_waltz_native_jj::jj::{JjClient, commit_id_revset};
use jj_waltz_refill_engine::ReseedRequest;
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
        Self::with_colocation(false)
    }
    fn with_colocation(colocated: bool) -> Self {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/refill-one-tests");
        fs::create_dir_all(&scratch).unwrap();
        let temp = tempfile::tempdir_in(scratch.canonicalize().unwrap()).unwrap();
        let source = temp.path().join("source");
        let mut f = Self {
            temp,
            source,
            s: String::new(),
        };
        f.jj(
            f.temp.path(),
            &[
                "git",
                "init",
                if colocated { "--colocate" } else { "--no-colocate" },
                f.source.to_str().unwrap(),
            ],
        );
        fs::write(f.source.join(".gitignore"), "cache/\n.env\ntool\n.jwlinks.toml\n.jwreseed.local.toml\n").unwrap();
        fs::write(f.source.join("shared"), vec![b'x'; 128 * 1024]).unwrap();
        fs::write(f.source.join("modified"), "source\n").unwrap();
        fs::write(f.source.join("source-only"), "source only\n").unwrap();
        fs::create_dir_all(f.source.join("collapse")).unwrap();
        fs::write(f.source.join("collapse/old"), "source descendant\n").unwrap();
        fs::write(f.source.join("mode"), "#!/bin/sh\n").unwrap();
        fs::write(f.source.join("line\nbreak"), "literal path\n").unwrap();
        symlink("././shared", f.source.join("link")).unwrap();
        f.jj(&f.source, &["status"]);
        f.s = JjClient::new(&f.source).resolve_one("@").unwrap().commit_id;
        f
    }
    fn command(&self, program: &str, cwd: &Path) -> Command {
        let mut c = Command::new(program);
        c.current_dir(cwd)
            .env("JJ_CONFIG", "")
            .env("JJ_USER", "N1 Test")
            .env("JJ_EMAIL", "n1@example.invalid")
            .env("JJ_EDITOR", "true")
            .env("XDG_CONFIG_HOME", self.temp.path().join("config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env_remove("GIT_CONFIG_COUNT")
            .env_remove("GIT_CONFIG_PARAMETERS");
        c
    }
    fn jj(&self, cwd: &Path, args: &[&str]) -> String {
        success(self.command("jj", cwd).args(args).output().unwrap())
    }
    fn workspace(&self, name: &str, at: &str) -> PathBuf {
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
                &commit_id_revset(at).unwrap(),
                root.to_str().unwrap(),
            ],
        );
        root
    }
    fn desired(&self) -> String {
        let root = self.workspace("desired", &self.s);
        fs::write(root.join("modified"), "desired\n").unwrap();
        fs::remove_file(root.join("source-only")).unwrap();
        fs::remove_file(root.join("collapse/old")).unwrap();
        fs::remove_dir(root.join("collapse")).unwrap();
        fs::write(root.join("collapse"), "desired leaf\n").unwrap();
        fs::write(root.join("desired-only"), "desired only\n").unwrap();
        fs::set_permissions(root.join("mode"), fs::Permissions::from_mode(0o755)).unwrap();
        fs::remove_file(root.join("link")).unwrap();
        fs::write(root.join("link"), "now regular\n").unwrap();
        fs::remove_file(root.join("line\nbreak")).unwrap();
        symlink("./shared", root.join("line\nbreak")).unwrap();
        self.jj(&root, &["status"]);
        JjClient::new(&root).resolve_one("@").unwrap().commit_id
    }
    fn request(&self, at: Option<&str>) -> ReseedRequest {
        ReseedRequest {
            renewal_route: jj_waltz_refill_engine::RenewalRoute::ClearThenPopulate,
            names: vec!["target".into()],
            source: Some(self.source.clone()),
            at: at.map(str::to_owned),
            copy_ignored: Some(vec!["cache".into()]),
            ..Default::default()
        }
    }
    fn bookmark(&self, root: &Path) {
        self.jj(root, &["bookmark", "create", "saved-target"]);
    }
    fn no_autotrack(&self) {
        self.jj(&self.source, &["config", "set", "--repo", "snapshot.auto-track", "none()"]);
    }
    fn run(&self, request: ReseedRequest) -> (Value, Vec<Vec<String>>) {
        self.run_injected(request, None, "", false)
    }
    fn run_injected(&self, request: ReseedRequest, fail_root: Option<&Path>, fail_arg: &str, final_barrier: bool) -> (Value, Vec<Vec<String>>) {
        let bin = self.temp.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        let real = success(Command::new("sh").args(["-c", "command -v jj"]).output().unwrap());
        let script = format!(
            "#!/bin/sh\nprintf '%s\\t' \"$PWD\" \"$@\" >> \"$JW_N1_TRACE\"\nprintf '\\n' >> \"$JW_N1_TRACE\"\n\
if [ \"$PWD\" = \"$JW_FAIL_ROOT\" ]; then\n for arg in \"$@\"; do if [ \"$arg\" = \"$JW_FAIL_ARG\" ]; then echo 'public native sentinel' >&2; exit 73; fi; done\nfi\n\
if [ \"$JW_FINAL_BARRIER\" = 1 ]; then\n\
 if [ \"$PWD\" = \"$JW_FINAL_TARGET\" ]; then\n\
  case \" $* \" in *' diff '*) echo 0 > \"$JW_FINAL_COUNT\";; esac\n\
  case \" $* \" in *' workspace list '*) if [ -f \"$JW_FINAL_COUNT\" ]; then n=$(cat \"$JW_FINAL_COUNT\"); echo $((n+1)) > \"$JW_FINAL_COUNT\"; fi;; esac\n\
 fi\n\
 if [ \"$PWD\" = \"$JW_FINAL_SOURCE\" ] && [ -f \"$JW_FINAL_COUNT\" ] && [ \"$(cat \"$JW_FINAL_COUNT\")\" -ge 4 ]; then echo 'public final barrier sentinel' >&2; exit 74; fi\n\
fi\nexec '{}' \"$@\"\n",
            real.replace('\'', "'\\''")
        );
        fs::write(bin.join("jj"), script).unwrap();
        fs::set_permissions(bin.join("jj"), fs::Permissions::from_mode(0o755)).unwrap();
        let trace = self.temp.path().join("trace");
        let mut cmd = self.command(env!("CARGO_BIN_EXE_jw"), &self.source);
        cmd.args(["reseed", "--format=json"]).args(&request.names);
        if request.renewal_route != jj_waltz_refill_engine::RenewalRoute::Refresh {
            cmd.args([
                "--renewal-route",
                match request.renewal_route {
                    jj_waltz_refill_engine::command::RenewalRoute::Refresh => unreachable!(),
                    jj_waltz_refill_engine::command::RenewalRoute::ClearThenPopulate => "clear-then-populate",
                    jj_waltz_refill_engine::command::RenewalRoute::ReplaceAuthorized => "replace-authorized",
                },
            ]);
        }
        cmd.args(["--refresh-check", &request.refresh_check.to_string()]);
        if request.postcheck == jj_waltz_refill_engine::Postcheck::Mtime {
            cmd.arg("--postcheck=mtime");
        }
        for group in &request.groups {
            cmd.args(["--group", group]);
        }
        if request.dry_run {
            cmd.arg("--dry-run");
        }
        if request.prepare {
            cmd.arg("--prepare");
        }
        if request.allow_unbookmarked {
            cmd.arg("--allow-unbookmarked");
        }
        if let Some(source) = &request.source {
            cmd.arg("--from").arg(source);
        }
        if let Some(at) = &request.at {
            cmd.args(["--at", at]);
        }
        if let Some(at) = &request.source_at {
            cmd.args(["--source-at", at]);
        }
        if let Some(patterns) = &request.copy_ignored {
            for p in patterns {
                cmd.args(["--copy-ignored", p]);
            }
            if patterns.is_empty() {
                cmd.arg("--no-copy-ignored");
            }
        }
        if request.no_copy_ignored {
            cmd.arg("--no-copy-ignored");
        }
        if request.replace_content {
            cmd.arg("--replace-content");
        }
        cmd.env("JW_N1_TRACE", &trace)
            .env("JW_FAIL_ROOT", fail_root.unwrap_or(Path::new("")))
            .env("JW_FAIL_ARG", fail_arg)
            .env("JW_FINAL_BARRIER", if final_barrier { "1" } else { "0" })
            .env("JW_FINAL_SOURCE", &self.source)
            .env("JW_FINAL_TARGET", self.temp.path().join("target"))
            .env("JW_FINAL_COUNT", self.temp.path().join("final-count"))
            .env("PATH", format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()));
        let output = cmd.output().unwrap();
        let wire: Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|e| panic!("{e}: {} / {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)));
        assert_eq!(wire["schema_version"], 2);
        static SCHEMA: std::sync::OnceLock<jsonschema::Validator> = std::sync::OnceLock::new();
        let schema = SCHEMA
            .get_or_init(|| jsonschema::validator_for(&serde_json::from_str(include_str!("../../../docs/schema/refill-v2.schema.json")).unwrap()).unwrap());
        assert!(
            schema.is_valid(&wire),
            "schema rejected outcome: {:?}",
            schema.iter_errors(&wire).map(|e| e.to_string()).collect::<Vec<_>>()
        );
        assert_eq!(output.status.success(), wire["status"] == "complete" || wire["status"] == "planned");
        let row = &wire["receivers"][0];
        // Test-only aliases retain the physical assertions from the native N1
        // receipt; all values now come from the actual application envelope.
        let number = |v: &Value| v.as_str().map(|s| s.parse::<u128>().unwrap()).map_or(Value::Null, |n| json!(n));
        let result = json!({"ok": output.status.success(), "source": wire["source"]["source_commit_id"], "desired": wire["desired"]["commit_id"],
            "old": row["saved"]["commit_id"], "commit": row["child"]["commit_id"], "change": row["child"]["change_id"],
            "reflinks": number(&row["native"]["renewal_effects"]["files_reflinked"]), "copies": number(&row["native"]["renewal_effects"]["files_copied"]),
            "payload_reflinks": number(&row["ordinary"]["report"]["completed"]["files_reflinked"]), "ordinary": !row["ordinary"]["report"].is_null(),
            "links": number(&row["finish"]["links"]["linked"]), "error": String::from_utf8_lossy(&output.stderr), "wire": wire});
        let calls = fs::read_to_string(trace)
            .unwrap()
            .lines()
            .map(|line| line.trim_end_matches('\t').split('\t').map(str::to_owned).collect())
            .collect();
        (result, calls)
    }
}

fn mutating_calls<'a>(calls: &'a [Vec<String>], root: &Path) -> Vec<&'a str> {
    calls
        .iter()
        .filter(|c| Path::new(&c[0]) == root && !c.iter().any(|s| s == "--ignore-working-copy"))
        .filter_map(|c| {
            c.iter()
                .find(|s| matches!(s.as_str(), "status" | "sparse" | "new" | "restore" | "diff"))
                .map(String::as_str)
        })
        .collect()
}

fn native_result(f: &Fixture, target: &Path, result: &Value, desired: &str) {
    assert_eq!(result["ok"], true, "{result:#}");
    assert_eq!(result["reflinks"], 7);
    assert_eq!(result["copies"], 0);
    let client = JjClient::new(target);
    let facts = client.commit_facts_at(&client.operation_id().unwrap(), "@").unwrap();
    assert!(facts.ordinary_empty_child());
    assert_eq!(facts.parents, [desired]);
    assert_eq!(facts.change_id, result["change"].as_str().unwrap());
    assert_eq!(f.jj(target, &["sparse", "list"]), ".");
    assert!(
        f.jj(
            target,
            &[
                "--config=snapshot.auto-track=none()",
                "diff",
                "--from",
                "@",
                "--to",
                &commit_id_revset(desired).unwrap(),
                "--summary"
            ]
        )
        .is_empty()
    );
    assert_ne!(
        fs::metadata(f.source.join("shared")).unwrap().ino(),
        fs::metadata(target.join("shared")).unwrap().ino()
    );
    assert_shared_extents(&f.source.join("shared"), &target.join("shared"));
    fs::write(target.join("shared"), "private write").unwrap();
    assert_eq!(fs::read(f.source.join("shared")).unwrap(), vec![b'x'; 128 * 1024]);
}

#[test]
fn explicit_clear_s_native_transfer_links_and_real_call_ledger() {
    let f = Fixture::with_colocation(true);
    let target = f.workspace("target", &f.s);
    fs::write(target.join("modified"), "old receiving edits\n").unwrap();
    f.bookmark(&target);
    fs::create_dir_all(f.source.join("cache")).unwrap();
    fs::write(f.source.join("cache/tool"), "payload\n").unwrap();
    fs::create_dir_all(target.join("cache/stale/deep")).unwrap();
    fs::write(target.join("cache/stale/deep/file"), "remove\n").unwrap();
    fs::write(target.join(".env"), "local secret\n").unwrap();
    fs::write(
        f.source.join(".jwlinks.toml"),
        "[[link]]\nsource='tool'\ntarget='cache/tool'\nrequired=true\n[[link]]\nsource='cache'\ntarget='cache'\nrequired=true\n",
    )
    .unwrap();
    let root_id = fs::metadata(&target).unwrap().ino();
    let jj_id = fs::metadata(target.join(".jj")).unwrap().ino();
    let source_checkout = fs::read(f.source.join(".jj/working_copy/checkout")).unwrap();
    let source_state = fs::read(f.source.join(".jj/working_copy/tree_state")).unwrap();
    let (result, calls) = f.run(f.request(None));
    assert_eq!(result["payload_reflinks"], 1, "{result:#}");
    assert_eq!(result["links"], 1);
    assert!(!target.join("cache/stale").exists());
    assert_eq!(fs::read(target.join(".env")).unwrap(), b"local secret\n");
    assert_eq!(fs::read(target.join("tool")).unwrap(), b"payload\n");
    assert_eq!(fs::metadata(&target).unwrap().ino(), root_id);
    assert_eq!(fs::metadata(target.join(".jj")).unwrap().ino(), jj_id);
    assert!(fs::symlink_metadata(target.join(".git")).is_err());
    assert_eq!(fs::read_link(target.join("link")).unwrap().as_os_str(), Path::new("././shared").as_os_str());
    assert_eq!(fs::read(f.source.join(".jj/working_copy/checkout")).unwrap(), source_checkout);
    assert_eq!(fs::read(f.source.join(".jj/working_copy/tree_state")).unwrap(), source_state);
    assert_eq!(JjClient::new(&f.source).resolve_one("@").unwrap().commit_id, f.s);
    assert_eq!(mutating_calls(&calls, &f.source), Vec::<&str>::new(), "no default donor snapshot");
    assert_eq!(mutating_calls(&calls, &target), ["status", "sparse", "new", "diff"]);
    let enumerations = calls.iter().filter(|c| c.windows(2).any(|p| p == ["file", "list"])).count();
    assert_eq!(enumerations, 2, "S=D shared; distinct O listed once");
    assert_eq!(
        calls.iter().filter(|c| c.windows(2).any(|p| p == ["file", "show"])).count(),
        1,
        "one pinned ignore extraction; shared source/desired rules"
    );
    assert!(!calls.iter().any(|c| c.iter().any(|arg| arg == "check-ignore")));
    let old = result["old"].as_str().unwrap();
    assert_eq!(
        f.jj(
            &f.source,
            &["--ignore-working-copy", "file", "show", "-r", &commit_id_revset(old).unwrap(), "modified"]
        ),
        "old receiving edits"
    );
    native_result(&f, &target, &result, &f.s);
}

#[test]
fn explicit_d_direct_restore_covers_shapes_modes_and_source_only_removal() {
    let f = Fixture::new();
    let d = f.desired();
    let target = f.workspace("target", &f.s);
    f.bookmark(&target);
    let (result, calls) = f.run(ReseedRequest {
        no_copy_ignored: true,
        ..f.request(Some(&d))
    });
    assert_eq!(result["ok"], true, "{result:#}");
    assert_eq!(mutating_calls(&calls, &f.source), Vec::<&str>::new());
    assert_eq!(mutating_calls(&calls, &target), ["status", "sparse", "new", "restore", "diff"]);
    assert_eq!(
        calls.iter().filter(|c| c.windows(2).any(|p| p == ["file", "list"])).count(),
        2,
        "empty O shares S membership rather than enumerating again"
    );
    assert_eq!(result["ordinary"], false);
    assert!(!target.join("source-only").exists());
    assert_eq!(fs::read(target.join("desired-only")).unwrap(), b"desired only\n");
    assert_eq!(fs::read(target.join("collapse")).unwrap(), b"desired leaf\n");
    assert_eq!(fs::read(target.join("link")).unwrap(), b"now regular\n");
    assert_eq!(
        fs::read_link(target.join("line\nbreak")).unwrap().as_os_str(),
        Path::new("./shared").as_os_str()
    );
    assert_eq!(fs::metadata(target.join("mode")).unwrap().permissions().mode() & 0o111, 0o111);
    native_result(&f, &target, &result, &d);
}

#[test]
fn byte_and_stat_equal_s_only_local_is_refused_before_state_adoption() {
    let f = Fixture::new();
    let d = f.desired();
    let target = f.workspace("target", &d);
    f.bookmark(&target);
    f.no_autotrack();
    let from = f.source.join("source-only");
    let to = target.join("source-only");
    fs::copy(&from, &to).unwrap();
    fs::File::options()
        .write(true)
        .open(&to)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(fs::metadata(&from).unwrap().modified().unwrap()))
        .unwrap();
    assert_eq!(fs::metadata(&from).unwrap().modified().unwrap(), fs::metadata(&to).unwrap().modified().unwrap());
    let (result, _) = f.run(f.request(Some(&d)));
    assert_eq!(result["ok"], false);
    assert!(result["error"].as_str().unwrap().contains("full-S tracked satisfaction"), "{result:#}");
    assert_eq!(fs::read(to).unwrap(), b"source only\n");
    assert_eq!(f.jj(&target, &["--ignore-working-copy", "sparse", "list"]), "");
}

#[test]
fn matching_d_only_local_is_not_silently_adopted_or_deleted() {
    let f = Fixture::new();
    let d = f.desired();
    let target = f.workspace("target", &f.s);
    f.bookmark(&target);
    f.no_autotrack();
    fs::write(target.join("desired-only"), "desired only\n").unwrap();
    let (result, _) = f.run(f.request(Some(&d)));
    assert_eq!(result["ok"], false);
    assert!(result["error"].as_str().unwrap().contains("D-addition namespace admission"), "{result:#}");
    assert_eq!(fs::read(target.join("desired-only")).unwrap(), b"desired only\n");
}

#[test]
fn collapse_with_retained_child_does_not_turn_skipped_native_checkout_into_success() {
    let f = Fixture::new();
    let d = f.desired();
    let target = f.workspace("target", &f.s);
    f.bookmark(&target);
    f.no_autotrack();
    fs::write(target.join("collapse/local"), "retain\n").unwrap();
    let (result, _) = f.run(f.request(Some(&d)));
    assert_eq!(result["ok"], false);
    assert!(result["error"].as_str().unwrap().contains("final native diff"), "{result:#}");
    assert_eq!(fs::read(target.join("collapse/local")).unwrap(), b"retain\n");
}

#[test]
fn explicit_assertion_is_one_live_donor_check_and_does_not_choose_d() {
    let f = Fixture::new();
    let target = f.workspace("target", &f.s);
    f.bookmark(&target);
    // Create a different source coordinate with the SAME tree as the assertion.
    f.jj(&f.source, &["new", &commit_id_revset(&f.s).unwrap()]);
    let source = JjClient::new(&f.source).resolve_one("@").unwrap().commit_id;
    let (result, calls) = f.run(ReseedRequest {
        source_at: Some(f.s.clone()),
        no_copy_ignored: true,
        ..f.request(None)
    });
    assert_eq!(result["ok"], true, "{result:#}");
    assert_eq!(result["desired"], source);
    assert_eq!(mutating_calls(&calls, &f.source), ["diff"]);
    // A requested assertion detects an unprepared live tracked edit and fails
    // before clearing, though its snapshot can record the rejected source edit.
    fs::write(f.source.join("modified"), "unprepared\n").unwrap();
    let (result, calls) = f.run(ReseedRequest {
        source_at: Some(f.s.clone()),
        no_copy_ignored: true,
        ..f.request(None)
    });
    assert_eq!(result["ok"], false);
    assert!(result["error"].as_str().unwrap().contains("explicit source_at"));
    // Trace is appended by run(), so inspect just its last donor call.
    assert_eq!(mutating_calls(&calls, &f.source), ["diff", "diff"]);
}

#[test]
fn known_sparse_profile_refuses_before_receiving_preservation_or_clear() {
    let f = Fixture::new();
    let target = f.workspace("target", &f.s);
    f.jj(&target, &["sparse", "set", "--clear"]);
    let (result, calls) = f.run(f.request(None));
    assert_eq!(result["ok"], false);
    assert!(result["error"].as_str().unwrap().contains("full sparse"));
    assert!(mutating_calls(&calls, &target).is_empty());
}

#[test]
fn empty_policy_retains_ordinary_names_and_only_explicit_replace_content_clears_them() {
    let f = Fixture::new();
    let target = f.workspace("target", &f.s);
    f.bookmark(&target);
    fs::create_dir_all(f.source.join("cache")).unwrap();
    fs::write(f.source.join("cache/tool"), "donor").unwrap();
    fs::create_dir_all(target.join("cache")).unwrap();
    fs::write(target.join("cache/local"), "local").unwrap();
    fs::write(target.join(".env"), "secret").unwrap();
    let request = ReseedRequest {
        copy_ignored: Some(vec![]),
        ..f.request(None)
    };
    let (result, _) = f.run(request.clone());
    assert_eq!(result["ok"], true, "{result:#}");
    assert_eq!(result["ordinary"], false);
    assert_eq!(fs::read(target.join("cache/local")).unwrap(), b"local");
    assert_eq!(fs::read(target.join(".env")).unwrap(), b"secret");
    assert!(!target.join("cache/tool").exists());
    let (result, _) = f.run(ReseedRequest {
        replace_content: true,
        ..request
    });
    assert_eq!(result["ok"], true, "{result:#}");
    assert!(!target.join("cache").exists());
    assert!(!target.join(".env").exists());
    assert!(target.join(".jj").is_dir());
}

#[test]
fn loaded_link_finish_failure_leaves_native_and_selected_payload_effects() {
    let f = Fixture::new();
    let target = f.workspace("target", &f.s);
    f.bookmark(&target);
    fs::create_dir_all(f.source.join("cache")).unwrap();
    fs::write(f.source.join("cache/tool"), "payload").unwrap();
    fs::write(
        f.source.join(".jwlinks.toml"),
        "[[link]]\nsource='tool'\ntarget='cache/missing'\nrequired=true\n",
    )
    .unwrap();
    let (result, _) = f.run(f.request(None));
    assert_eq!(result["ok"], false);
    let error = result["error"].as_str().unwrap();
    assert!(
        error.contains("phase loaded link finish") && error.contains("required link target is missing"),
        "{error}"
    );
    assert!(
        error.contains("ordinary files_reflinked: 1"),
        "known ordinary work must be retained in failure context: {error}"
    );
    assert_eq!(fs::read(target.join("cache/tool")).unwrap(), b"payload");
    assert_eq!(f.jj(&target, &["--ignore-working-copy", "sparse", "list"]), ".");
}

#[test]
fn public_schema_both_routes_n1_n2_and_omitted_default() {
    use jj_waltz_refill_engine::command::RenewalRoute;
    for route in [RenewalRoute::ClearThenPopulate, RenewalRoute::ReplaceAuthorized] {
        for names in [vec!["target"], vec!["target", "peer"]] {
            let f = Fixture::new();
            for name in &names {
                f.workspace(name, &f.s);
            }
            let request = ReseedRequest {
                renewal_route: route,
                names: names.iter().map(|n| (*n).into()).collect(),
                no_copy_ignored: true,
                allow_unbookmarked: true,
                ..f.request(None)
            };
            let (result, _) = f.run(request);
            let wire = &result["wire"];
            assert_eq!(wire["status"], "complete", "{result:#}");
            assert_eq!(wire["source"]["evidence"], "assumed_prepared");
            assert_eq!(wire["source"]["source_commit_id"], f.s);
            assert_eq!(wire["desired"]["commit_id"], f.s);
            assert_eq!(wire["capture"]["desired_manifests"], "1");
            assert_eq!(
                wire["capture"]["owned_manifests"],
                if route == RenewalRoute::ReplaceAuthorized { "1" } else { "0" }
            );
            for row in wire["receivers"].as_array().unwrap() {
                assert_eq!(row["status"], "complete");
                assert_eq!(row["native"]["manifest"]["desired"], "8");
                assert_eq!(row["native"]["manifest"]["installed"], "8");
                assert_eq!(row["native"]["manifest"]["complete"], true);
                assert_eq!(row["native"]["source_state_adopted"], true);
                assert_eq!(row["native"]["renewal_effects"]["files_reflinked"], "7");
                assert_eq!(row["native"]["renewal_effects"]["files_copied"], "0");
                assert_eq!(row["ordinary"]["state"], "disabled");
                assert!(row["ordinary"]["report"].is_null());
                assert!(row["ordinary"]["application_ms"].is_null());
                assert_eq!(row["child"]["parent_commit_ids"], json!([f.s]));
                assert_eq!(row["finish"]["state"], "complete");
            }
            assert!(wire["source_diagnostics"]["native"].is_object());
            assert!(wire["source_diagnostics"]["ordinary"].is_null());
        }
    }
    let f = Fixture::new();
    f.workspace("target", &f.s);
    let wire: Value = serde_json::from_str(&success(
        f.command(env!("CARGO_BIN_EXE_jw"), &f.source)
            .args(["reseed", "target", "--allow-unbookmarked", "--no-copy-ignored", "--format=json"])
            .output()
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(wire["renewal_route"], "refresh");
    assert_eq!(wire["receivers"][0]["renewal_route"], "refresh");
    assert_eq!(wire["refresh_check"], "mtime,size");
    assert_eq!(wire["postcheck"], "none");
}

fn refresh_request(f: &Fixture) -> ReseedRequest {
    ReseedRequest {
        renewal_route: jj_waltz_refill_engine::RenewalRoute::Refresh,
        allow_unbookmarked: true,
        ..f.request(None)
    }
}

fn seed(f: &Fixture, request: &ReseedRequest) {
    let (result, _) = f.run(ReseedRequest {
        renewal_route: jj_waltz_refill_engine::RenewalRoute::ReplaceAuthorized,
        ..request.clone()
    });
    assert_eq!(result["ok"], true, "{result:#}");
}

fn exact_native(f: &Fixture, target: &Path, result: &Value, desired: &str) {
    assert_eq!(result["ok"], true, "{result:#}");
    assert_eq!(result["wire"]["receivers"][0]["native"]["source_state_adopted"], true);
    let client = JjClient::new(target);
    let facts = client.commit_facts_at(&client.operation_id().unwrap(), "@").unwrap();
    assert!(facts.ordinary_empty_child());
    assert_eq!(facts.parents, [desired]);
    assert_eq!(facts.change_id, result["change"].as_str().unwrap());
    assert!(
        f.jj(
            target,
            &[
                "--config=snapshot.auto-track=none()",
                "diff",
                "--from",
                "@",
                "--to",
                &commit_id_revset(desired).unwrap(),
                "--summary"
            ]
        )
        .is_empty()
    );
}

#[test]
fn default_seeded_refresh_retains_older_inodes_ctimes_and_adopts_native_state_without_clone_effects() {
    let f = Fixture::new();
    let target = f.workspace("target", &f.s);
    fs::create_dir_all(f.source.join("cache")).unwrap();
    fs::write(f.source.join("cache/file"), "ordinary").unwrap();
    let request = refresh_request(&f);
    seed(&f, &request);
    let paths = [
        ".gitignore",
        "shared",
        "modified",
        "source-only",
        "collapse/old",
        "mode",
        "line\nbreak",
        "link",
        "cache/file",
    ];
    fs::set_permissions(target.join("cache/file"), fs::Permissions::from_mode(0o740)).unwrap();
    let before = paths.map(|p| fs::symlink_metadata(target.join(p)).unwrap());
    // No mandatory privacy audit: this seeded ordinary file intentionally has a
    // second name outside Q. Refresh leaves both names and the inode untouched.
    fs::hard_link(target.join("cache/file"), target.join(".env")).unwrap();
    let before_cache = fs::metadata(target.join("cache/file")).unwrap();
    // Source ctime is newer than the retained receiving inode's ctime, without
    // changing mtime, bytes or native executable meaning. Adoption must still
    // converge exactly; it cannot require donor/receiver inode or ctime equality.
    std::thread::sleep(std::time::Duration::from_millis(5));
    let source_mode = fs::metadata(f.source.join("mode")).unwrap().permissions();
    fs::set_permissions(f.source.join("mode"), source_mode).unwrap();
    let source_ctime = fs::metadata(f.source.join("mode")).unwrap();
    assert!((before[5].ctime(), before[5].ctime_nsec()) < (source_ctime.ctime(), source_ctime.ctime_nsec()));
    for _ in 0..2 {
        let (result, calls) = f.run(request.clone()); // route omitted: actual CLI default
        let w = &result["wire"];
        exact_native(&f, &target, &result, &f.s);
        assert_eq!(w["renewal_route"], "refresh");
        assert_eq!(w["refresh_check"], "mtime,size");
        assert_eq!(w["postcheck"], "none");
        let native = &w["receivers"][0]["native"];
        assert_eq!(native["manifest"]["installed"], "0");
        assert_eq!(native["manifest"]["retained"], "8");
        assert_eq!(native["manifest"]["complete"], true);
        assert_eq!(native["renewal_effects"]["files_reflinked"], "0");
        assert_eq!(native["retained_permissions_set"], "0");
        assert_eq!(w["receivers"][0]["ordinary"]["report"]["completed"]["files_reflinked"], "0");
        for (p, old) in paths.iter().zip(&before) {
            let now = fs::symlink_metadata(target.join(p)).unwrap();
            assert_eq!((now.ino(), now.modified().unwrap()), (old.ino(), old.modified().unwrap()), "{p}");
            if *p != "cache/file" {
                assert_eq!((now.ctime(), now.ctime_nsec()), (old.ctime(), old.ctime_nsec()), "no metadata write: {p}");
            }
        }
        let now = fs::metadata(target.join("cache/file")).unwrap();
        assert_eq!(now.mode() & 0o7777, 0o740, "ordinary executable state is not normalized");
        assert_eq!((now.ctime(), now.ctime_nsec()), (before_cache.ctime(), before_cache.ctime_nsec()));
        assert_eq!(fs::metadata(target.join(".env")).unwrap().ino(), now.ino());
        assert!(!calls.iter().any(|c| c.iter().any(|a| a == "--clear")));
        assert!(native["steps"].as_array().unwrap().iter().any(|s| s["phase"] == "reparent"));
        assert!(!native["steps"].as_array().unwrap().iter().any(|s| s["phase"] == "restore"));
    }
}

#[test]
fn refresh_native_executable_correction_is_in_place_preserves_other_bits_and_is_not_an_install() {
    for executable in [false, true] {
        let mut f = Fixture::new();
        if executable {
            fs::set_permissions(f.source.join("mode"), fs::Permissions::from_mode(0o755)).unwrap();
            f.jj(&f.source, &["status"]);
            f.s = JjClient::new(&f.source).resolve_one("@").unwrap().commit_id;
        }
        let target = f.workspace("target", &f.s);
        let request = refresh_request(&f);
        seed(&f, &request);
        fs::set_permissions(target.join("mode"), fs::Permissions::from_mode(if executable { 0o2640 } else { 0o2751 })).unwrap();
        let before = fs::metadata(target.join("mode")).unwrap();
        let (result, _) = f.run(request.clone());
        exact_native(&f, &target, &result, &f.s);
        let after = fs::metadata(target.join("mode")).unwrap();
        assert_eq!((after.ino(), after.modified().unwrap()), (before.ino(), before.modified().unwrap()));
        assert_eq!(after.mode() & 0o7777, if executable { 0o2751 } else { 0o2640 });
        let native = &result["wire"]["receivers"][0]["native"];
        assert_eq!(native["renewal_effects"]["files_reflinked"], "0");
        assert_eq!(native["retained_permissions_set"], "1");
        assert_eq!(native["manifest"]["retained"], "8");
        if executable {
            fs::set_permissions(target.join("mode"), fs::Permissions::from_mode(0o2641)).unwrap();
        }
        let before_noop = fs::metadata(target.join("mode")).unwrap();
        let (again, _) = f.run(request);
        exact_native(&f, &target, &again, &f.s);
        assert_eq!(again["wire"]["receivers"][0]["native"]["retained_permissions_set"], "0");
        let after_noop = fs::metadata(target.join("mode")).unwrap();
        assert_eq!((after_noop.ctime(), after_noop.ctime_nsec()), (before_noop.ctime(), before_noop.ctime_nsec()));
    }
}

#[test]
fn refresh_changed_missing_obsolete_types_links_and_exact_distinct_desired_tree() {
    let f = Fixture::new();
    let d = f.desired();
    let target = f.workspace("target", &f.s);
    let request = refresh_request(&f);
    seed(&f, &request);
    let shared = fs::metadata(target.join("shared")).unwrap();
    fs::write(target.join("modified"), "changed receiving contents").unwrap();
    fs::remove_file(target.join("source-only")).unwrap();
    fs::remove_file(target.join("collapse/old")).unwrap();
    fs::remove_dir(target.join("collapse")).unwrap();
    fs::write(target.join("collapse"), "receiving leaf").unwrap();
    fs::write(target.join("obsolete"), "owned old-only").unwrap();
    fs::remove_file(target.join("link")).unwrap();
    symlink("./shared", target.join("link")).unwrap();
    let (result, _) = f.run(request.clone());
    exact_native(&f, &target, &result, &f.s);
    let native = &result["wire"]["receivers"][0]["native"];
    assert_eq!(native["manifest"]["installed"], "4");
    assert_eq!(native["manifest"]["retained"], "4");
    assert_eq!(native["manifest"]["owned_obsolete"], "2");
    assert_eq!(native["manifest"]["owned_retired"], "2");
    assert_eq!(native["renewal_effects"]["files_reflinked"], "3");
    assert_eq!(native["renewal_effects"]["symlinks_created"], "1");
    assert!(!target.join("obsolete").exists());
    assert_eq!(fs::read(target.join("collapse/old")).unwrap(), b"source descendant\n");
    assert_eq!(fs::read_link(target.join("link")).unwrap().as_os_str(), Path::new("././shared").as_os_str());
    assert_eq!(fs::metadata(target.join("shared")).unwrap().ino(), shared.ino());
    let (distinct, _) = f.run(ReseedRequest {
        at: Some(d.clone()),
        ..request
    });
    exact_native(&f, &target, &distinct, &d);
    assert_eq!(distinct["wire"]["receivers"][0]["native"]["manifest"]["retained"], "8");
    assert_eq!(distinct["reflinks"], 0);
    assert_eq!(fs::read(target.join("collapse")).unwrap(), b"desired leaf\n");
    assert!(!target.join("source-only").exists());
}

#[test]
fn refresh_check_selection_postcheck_and_explicit_replace_content_keep_their_independent_meaning() {
    use jj_waltz_refill_engine::{Postcheck, RenewalRoute};
    let f = Fixture::new();
    let target = f.workspace("target", &f.s);
    fs::create_dir_all(f.source.join("cache")).unwrap();
    fs::write(f.source.join("cache/file"), "ordinary").unwrap();
    let request = refresh_request(&f);
    let (preview, _) = f.run(ReseedRequest {
        dry_run: true,
        refresh_check: "mode".parse().unwrap(),
        postcheck: Postcheck::Mtime,
        ..request.clone()
    });
    assert_eq!(preview["wire"]["status"], "planned");
    assert_eq!(preview["wire"]["renewal_route"], "refresh");
    assert_eq!(preview["wire"]["refresh_check"], "mode");
    assert_eq!(preview["wire"]["postcheck"], "mtime");
    assert!(preview["wire"]["receivers"][0]["native"]["manifest"].is_null());
    assert!(preview["wire"]["receivers"][0]["native"]["retained_permissions_set"].is_null());
    seed(&f, &request);
    for (checks, renew) in [("size", false), ("mtime", true), ("mtime,size,mode", true)] {
        for path in ["mode", "cache/file"] {
            if checks.ends_with("mode") {
                fs::set_permissions(target.join(path), fs::Permissions::from_mode(0o740)).unwrap();
            } else {
                fs::File::options()
                    .write(true)
                    .open(target.join(path))
                    .unwrap()
                    .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(10))
                    .unwrap();
            }
        }
        let before = ["mode", "cache/file"].map(|p| fs::metadata(target.join(p)).unwrap().ino());
        let (result, _) = f.run(ReseedRequest {
            refresh_check: checks.parse().unwrap(),
            postcheck: Postcheck::Mtime,
            ..request.clone()
        });
        exact_native(&f, &target, &result, &f.s);
        assert_eq!(result["wire"]["refresh_check"], checks);
        assert_eq!(result["wire"]["postcheck"], "mtime");
        for (p, old) in ["mode", "cache/file"].iter().zip(before) {
            assert_eq!(fs::metadata(target.join(p)).unwrap().ino() != old, renew, "{checks}/{p}");
        }
        assert_eq!(result["reflinks"], if renew { 1 } else { 0 });
        assert_eq!(result["payload_reflinks"], if renew { 1 } else { 0 });
        assert_eq!(result["wire"]["receivers"][0]["native"]["retained_permissions_set"], "0");
    }
    for route in [RenewalRoute::ClearThenPopulate, RenewalRoute::ReplaceAuthorized] {
        fs::File::options()
            .write(true)
            .open(target.join("cache/file"))
            .unwrap()
            .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(10))
            .unwrap();
        let ordinary_inode = fs::metadata(target.join("cache/file")).unwrap().ino();
        let (result, _) = f.run(ReseedRequest {
            renewal_route: route,
            refresh_check: "size".parse().unwrap(),
            postcheck: Postcheck::Mtime,
            ..request.clone()
        });
        exact_native(&f, &target, &result, &f.s);
        assert_eq!(result["reflinks"], 7, "old routes remain unconditional tracked installation");
        assert_eq!(result["payload_reflinks"], 0, "old routes still use selected ordinary checks");
        assert_eq!(fs::metadata(target.join("cache/file")).unwrap().ino(), ordinary_inode);
    }
    for route in [RenewalRoute::Refresh, RenewalRoute::ClearThenPopulate, RenewalRoute::ReplaceAuthorized] {
        fs::write(target.join(".env"), "explicitly erase").unwrap();
        let (result, _) = f.run(ReseedRequest {
            renewal_route: route,
            replace_content: true,
            postcheck: Postcheck::Mtime,
            ..request.clone()
        });
        exact_native(&f, &target, &result, &f.s);
        let native = &result["wire"]["receivers"][0]["native"];
        assert_eq!(native["manifest"]["installed"], "8");
        assert_eq!(native["manifest"]["retained"], "0");
        assert_eq!(result["reflinks"], 7);
        assert!(!target.join(".env").exists());
        assert_eq!(result["payload_reflinks"], 1);
    }
}

#[test]
fn public_ordinary_retention_is_not_reconstructed_from_deviation() {
    let f = Fixture::new();
    let target = f.workspace("target", &f.s);
    fs::create_dir_all(f.source.join("cache/routing/selected")).unwrap();
    fs::write(f.source.join("cache/routing/selected/good"), "selected").unwrap();
    fs::write(f.source.join(".env"), "equal retained secret").unwrap();
    fs::copy(f.source.join(".env"), target.join(".env")).unwrap();
    fs::File::options()
        .write(true)
        .open(target.join(".env"))
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(fs::metadata(f.source.join(".env")).unwrap().modified().unwrap()))
        .unwrap();
    let (result, _) = f.run(ReseedRequest {
        allow_unbookmarked: true,
        copy_ignored: Some(vec!["cache/routing/selected".into()]),
        ..f.request(None)
    });
    assert_eq!(result["wire"]["status"], "complete");
    let ordinary = &result["wire"]["receivers"][0]["ordinary"]["report"];
    assert_ne!(ordinary["retained"], ordinary["deviated"], "{ordinary:#}");
    assert_eq!(ordinary["completed"]["files_reflinked"], "1");
    assert_eq!(fs::read(target.join(".env")).unwrap(), b"equal retained secret");
}

#[test]
fn public_partial_native_failure_keeps_known_peers_and_last_child_facts_without_hint_queries() {
    use jj_waltz_refill_engine::command::RenewalRoute;
    for route in [RenewalRoute::ClearThenPopulate, RenewalRoute::ReplaceAuthorized] {
        let f = Fixture::new();
        f.workspace("target", &f.s);
        let peer = f.workspace("peer", &f.s);
        let request = ReseedRequest {
            renewal_route: route,
            names: vec!["target".into(), "peer".into()],
            no_copy_ignored: true,
            allow_unbookmarked: true,
            ..f.request(None)
        };
        let arg = if route == RenewalRoute::ReplaceAuthorized { "rebase" } else { "--clear" };
        let (result, calls) = f.run_injected(request, Some(&peer), arg, false);
        let w = &result["wire"];
        assert_eq!(w["status"], "failed");
        assert_eq!(w["error"]["receiver"], "peer");
        assert!(
            w["error"]["message"].as_str().unwrap().contains("public native sentinel")
                || w["error"]["causes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|c| c.as_str().unwrap().contains("public native sentinel")),
            "{result:#}"
        );
        assert_eq!(w["receivers"][0]["status"], "partial");
        assert_eq!(w["receivers"][0]["native"]["state"], "established");
        assert_eq!(w["receivers"][1]["status"], "failed");
        assert!(w["receivers"][0]["finish"]["links"].is_null());
        if route == RenewalRoute::ReplaceAuthorized {
            assert!(w["receivers"][1]["child"]["commit_id"].is_string());
            assert_eq!(w["receivers"][1]["native"]["source_state_adopted"], true);
        }
        let last = calls.last().unwrap();
        assert_eq!(Path::new(&last[0]), peer);
        assert!(last.iter().any(|a| a == arg), "no post-error discovery: {last:?}");
    }
}

#[test]
fn public_final_barrier_failure_can_leave_every_receiver_complete() {
    let f = Fixture::new();
    f.workspace("target", &f.s);
    let (result, calls) = f.run_injected(
        ReseedRequest {
            no_copy_ignored: true,
            allow_unbookmarked: true,
            ..f.request(None)
        },
        None,
        "",
        true,
    );
    let w = &result["wire"];
    assert_eq!(w["status"], "failed", "{result:#}");
    assert_eq!(w["error"]["phase"], "finish");
    assert!(w["error"]["receiver"].is_null());
    assert_eq!(w["receivers"][0]["status"], "complete");
    assert!(result["error"].as_str().unwrap().contains("public final barrier sentinel"));
    assert_eq!(Path::new(&calls.last().unwrap()[0]), f.source);
}

#[test]
fn public_finish_failure_keeps_earlier_complete_receiver_and_actual_link_counts() {
    use jj_waltz_refill_engine::command::RenewalRoute;
    for route in [RenewalRoute::ClearThenPopulate, RenewalRoute::ReplaceAuthorized] {
        let f = Fixture::new();
        let target = f.workspace("target", &f.s);
        f.workspace("peer", &f.s);
        fs::write(target.join(".env"), "retained first-lane target").unwrap();
        fs::create_dir_all(f.source.join("cache")).unwrap();
        fs::write(f.source.join("cache/tool"), "ordinary work").unwrap();
        fs::write(f.source.join(".jwlinks.toml"), "[[link]]\nsource='tool'\ntarget='.env'\nrequired=true\n").unwrap();
        let (result, _) = f.run(ReseedRequest {
            renewal_route: route,
            names: vec!["target".into(), "peer".into()],
            allow_unbookmarked: true,
            ..f.request(None)
        });
        let w = &result["wire"];
        assert_eq!(w["status"], "failed", "{result:#}");
        assert_eq!(w["error"]["phase"], "links");
        assert_eq!(w["error"]["receiver"], "peer");
        assert_eq!(w["receivers"][0]["status"], "complete");
        assert_eq!(w["receivers"][0]["finish"]["links"]["linked"], "1");
        assert_eq!(w["receivers"][1]["status"], "failed");
        assert_eq!(w["receivers"][1]["finish"]["state"], "partial");
        assert_eq!(w["receivers"][1]["finish"]["links"]["linked"], "0");
        for row in w["receivers"].as_array().unwrap() {
            assert_eq!(row["native"]["state"], "established");
            assert_eq!(row["ordinary"]["report"]["completed"]["files_reflinked"], "1");
        }
    }
}

#[test]
fn public_early_resolution_error_has_zero_rows_and_preserves_requested_names_and_path_bytes() {
    use std::os::unix::ffi::OsStringExt;
    let f = Fixture::new();
    let missing = f.temp.path().join(std::ffi::OsString::from_vec(b"missing-\xff".to_vec()));
    let (result, calls) = f.run(ReseedRequest {
        source: Some(missing.clone()),
        ..f.request(None)
    });
    let w = &result["wire"];
    assert_eq!(w["status"], "failed");
    assert_eq!(w["selection"]["names"], json!(["target"]));
    assert_eq!(w["selection"]["resolved"], json!([]));
    assert_eq!(w["receivers"], json!([]));
    assert!(w["source"].is_null());
    assert!(w["capture"].is_null());
    assert!(w["timings"]["engine_ms"].is_null());
    assert!(w["error"]["path"]["unix_bytes_hex"].as_str().unwrap().ends_with("6d697373696e672dff"));
    assert!(!calls.iter().any(|c| c.iter().any(|a| matches!(a.as_str(), "status" | "diff" | "sparse"))));
    let output = f
        .command(env!("CARGO_BIN_EXE_jw"), &f.source)
        .args(["prepare", "--format=json", "--from"])
        .arg(missing)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let w: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(w["command"], "prepare");
    assert_eq!(w["receivers"], json!([]));
    assert!(w["source"].is_null());
    assert!(w["renewal_route"].is_null());
}

#[cfg(target_os = "linux")]
#[test]
fn public_ordinary_failure_has_returned_work_and_lossless_provider_path() {
    use jj_waltz_refill_engine::command::RenewalRoute;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    for route in [RenewalRoute::ClearThenPopulate, RenewalRoute::ReplaceAuthorized] {
        let f = Fixture::new();
        f.workspace("target", &f.s);
        fs::create_dir_all(f.source.join("cache/a")).unwrap();
        let bad_directory = f.source.join("cache").join(std::ffi::OsString::from_vec(b"z-\xff".to_vec()));
        fs::create_dir_all(&bad_directory).unwrap();
        fs::write(f.source.join("cache/a/good"), "completed before failure").unwrap();
        let bad = bad_directory.join("bad");
        let c = std::ffi::CString::new(bad.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
        let (result, _) = f.run(ReseedRequest {
            renewal_route: route,
            allow_unbookmarked: true,
            ..f.request(None)
        });
        let w = &result["wire"];
        assert_eq!(w["status"], "failed", "{result:#}");
        assert_eq!(w["error"]["phase"], "ordinary");
        assert!(w["error"]["path"]["unix_bytes_hex"].as_str().unwrap().ends_with("7a2dff"), "{result:#}");
        assert_eq!(w["receivers"][0]["native"]["manifest"]["complete"], true);
        assert_eq!(w["receivers"][0]["ordinary"]["report"]["completed"]["files_reflinked"], "1");
        assert!(w["source_diagnostics"]["ordinary"].is_object());
        assert!(w["receivers"][0]["finish"]["links"].is_null());
    }
}

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
            ..Default::default()
        };
        assert_eq!(unsafe { libc::ioctl(file.as_raw_fd(), 0xc020660b as libc::c_ulong, &mut map) }, 0);
        assert_eq!(map.mapped, 1);
        map.extent
    }
    let (source, target) = (extent(source), extent(target));
    assert_eq!(source.physical, target.physical);
    assert_ne!(source.flags & 0x2000, 0);
    assert_ne!(target.flags & 0x2000, 0);
}
#[cfg(not(target_os = "linux"))]
fn assert_shared_extents(_: &Path, _: &Path) {}
