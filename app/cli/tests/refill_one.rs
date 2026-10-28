//! Native first-slice integration, through the callable replacement engine.
//! Child test processes isolate JJ configuration and argv tracing without any
//! process-global environment mutation or operator-workspace effects.
#![cfg(unix)]
use jj_waltz_native_jj::jj::{JjClient, commit_id_revset};
use jj_waltz_refill_engine::{ReseedRequest, refill_one_from};
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
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/refill-one-tests");
        fs::create_dir_all(&scratch).unwrap();
        let temp = tempfile::tempdir_in(scratch.canonicalize().unwrap()).unwrap();
        let source = temp.path().join("source");
        let mut f = Self {
            temp,
            source,
            s: String::new(),
        };
        f.jj(f.temp.path(), &["git", "init", "--no-colocate", f.source.to_str().unwrap()]);
        fs::write(f.source.join(".gitignore"), "cache/\n.env\ntool\n.jwlinks.toml\n.jwreseed.local.toml\n").unwrap();
        fs::write(f.source.join("shared"), vec![b'x'; 128 * 1024]).unwrap();
        fs::write(f.source.join("modified"), "source\n").unwrap();
        fs::write(f.source.join("source-only"), "source only\n").unwrap();
        fs::create_dir_all(f.source.join("collapse")).unwrap();
        fs::write(f.source.join("collapse/old"), "source descendant\n").unwrap();
        fs::write(f.source.join("mode"), "#!/bin/sh\n").unwrap();
        fs::write(f.source.join("line\nbreak"), "literal path\n").unwrap();
        symlink("shared", f.source.join("link")).unwrap();
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
            &["workspace", "add", "--name", name, "-r", &commit_id_revset(at).unwrap(), root.to_str().unwrap()],
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
        self.jj(&root, &["status"]);
        JjClient::new(&root).resolve_one("@").unwrap().commit_id
    }
    fn request(&self, at: Option<&str>) -> ReseedRequest {
        ReseedRequest {
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
        let bin = self.temp.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        let real = success(Command::new("sh").args(["-c", "command -v jj"]).output().unwrap());
        let script = format!(
            "#!/bin/sh\nprintf '%s\\t' \"$PWD\" \"$@\" >> \"$JW_N1_TRACE\"\nprintf '\\n' >> \"$JW_N1_TRACE\"\nexec '{}' \"$@\"\n",
            real.replace('\'', "'\\''")
        );
        fs::write(bin.join("jj"), script).unwrap();
        fs::set_permissions(bin.join("jj"), fs::Permissions::from_mode(0o755)).unwrap();
        let trace = self.temp.path().join("trace");
        let report = self.temp.path().join("report.json");
        let mut cmd = self.command(std::env::current_exe().unwrap().to_str().unwrap(), &self.source);
        cmd.args(["--exact", "execute_refill_child", "--ignored", "--nocapture"])
            .env("JW_N1_REQUEST", serde_json::to_string(&request).unwrap())
            .env("JW_N1_TRACE", &trace)
            .env("JW_N1_REPORT", &report)
            .env("PATH", format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()));
        success(cmd.output().unwrap());
        let result = serde_json::from_slice(&fs::read(report).unwrap()).unwrap();
        let calls = fs::read_to_string(trace)
            .unwrap()
            .lines()
            .map(|line| line.trim_end_matches('\t').split('\t').map(str::to_owned).collect())
            .collect();
        (result, calls)
    }
}

#[test]
#[ignore = "isolated engine subprocess, invoked by the fixture"]
fn execute_refill_child() {
    let request: ReseedRequest = serde_json::from_str(&std::env::var("JW_N1_REQUEST").unwrap()).unwrap();
    let result = match refill_one_from(&std::env::current_dir().unwrap(), request, |s| eprintln!("{s}")) {
        Ok(r) => {
            json!({"ok": true, "source": r.source_commit, "desired": r.desired_commit, "old": r.old_commit, "commit": r.final_commit, "change": r.change_id,
            "reflinks": r.native_install.files_reflinked, "copies": r.native_install.files_copied,
            "payload_reflinks": r.ordinary.as_ref().map(|o| o.totals.completed.files_reflinked), "ordinary": r.ordinary.is_some(), "links": r.links.linked})
        }
        Err(e) => json!({"ok": false, "error": format!("{e:#}")}),
    };
    fs::write(std::env::var_os("JW_N1_REPORT").unwrap(), serde_json::to_vec_pretty(&result).unwrap()).unwrap();
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
fn default_s_native_transfer_links_and_real_call_ledger() {
    let f = Fixture::new();
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
    assert!(result["error"].as_str().unwrap().contains("strict full-S install"), "{result:#}");
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
        error.contains("files_reflinked: 1"),
        "known ordinary work must be retained in failure context: {error}"
    );
    assert_eq!(fs::read(target.join("cache/tool")).unwrap(), b"payload");
    assert_eq!(f.jj(&target, &["--ignore-working-copy", "sparse", "list"]), ".");
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
