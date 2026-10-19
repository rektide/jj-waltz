//! Native-JJ reseeding contracts. No mocked filesystem copies: successful runs
//! must use a filesystem supporting reflinks (set JW_TEST_REQUIRE_COW in CI).
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

struct Repo {
    temp: TempDir,
    root: PathBuf,
    config: PathBuf,
    baseline: String,
}

fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

impl Repo {
    fn new() -> Option<Self> {
        Self::new_with_colocation(false)
    }

    fn new_with_colocation(colocated: bool) -> Option<Self> {
        if !Command::new("jj").arg("--version").output().is_ok_and(|o| o.status.success()) {
            assert!(std::env::var_os("JW_TEST_REQUIRE_COW").is_none_or(|value| value.is_empty()), "jj is required");
            eprintln!("skipping reseed integration test: jj unavailable");
            return None;
        }
        // Keep fixtures on the checkout's filesystem, not an unrelated /tmp mount.
        let scratch = std::env::current_dir().unwrap().join(".test-agent/reseed-tests");
        fs::create_dir_all(&scratch).unwrap();
        let temp = tempfile::tempdir_in(scratch).unwrap();
        let root = temp.path().join("repo");
        let config = temp.path().join("config");
        fs::create_dir_all(&config).unwrap();
        let mut repo = Self {
            temp,
            root,
            config,
            baseline: String::new(),
        };
        repo.jj_at(
            repo.temp.path(),
            &[
                "git",
                "init",
                if colocated { "--colocate" } else { "--no-colocate" },
                repo.root.to_str().unwrap(),
            ],
        );
        fs::write(repo.root.join(".gitignore"), "cache/\n.env\n.jwreseed.local.toml\npackages/\n").unwrap();
        fs::write(repo.root.join("shared"), vec![b's'; 128 * 1024]).unwrap();
        fs::write(repo.root.join("modified"), "baseline\n").unwrap();
        fs::write(repo.root.join("deleted"), "delete me\n").unwrap();
        fs::write(repo.root.join("transition"), "regular\n").unwrap();
        fs::write(repo.root.join("directory-transition"), "regular\n").unwrap();
        fs::write(repo.root.join("mode"), "#!/bin/sh\nexit 0\n").unwrap();
        fs::write(repo.root.join("line\nbreak"), "newline path\n").unwrap();
        symlink("shared", repo.root.join("link")).unwrap();
        repo.baseline = repo.id(&repo.root, "@");
        Some(repo)
    }

    fn command(&self, program: &str, cwd: &Path) -> Command {
        let mut cmd = Command::new(program);
        cmd.current_dir(cwd)
            .env("XDG_CONFIG_HOME", &self.config)
            .env("JJ_CONFIG", "")
            .env("JJ_USER", "Reseed Test")
            .env("JJ_EMAIL", "reseed@example.invalid")
            .env("JJ_EDITOR", "true");
        if program == "jj" {
            cmd.args(["--config", "git.colocate=false"]);
        }
        cmd
    }

    fn jw(&self, cwd: &Path) -> Command {
        let mut cmd = self.command("jw", cwd);
        cmd = {
            let mut binary = Command::new(assert_cmd::cargo::cargo_bin!("jw"));
            binary.current_dir(cwd).envs(cmd.get_envs().filter_map(|(k, v)| v.map(|v| (k, v))));
            binary
        };
        cmd
    }

    fn jj_at(&self, cwd: &Path, args: &[&str]) -> String {
        success(self.command("jj", cwd).args(args).output().unwrap())
    }

    fn id(&self, cwd: &Path, rev: &str) -> String {
        self.jj_at(cwd, &["log", "--no-graph", "-r", rev, "-T", "commit_id"])
    }

    fn workspace(&self, name: &str) -> PathBuf {
        let path = self.temp.path().join(format!("workspace-{name}"));
        self.workspace_at(name, &path);
        path
    }

    fn workspace_at(&self, name: &str, path: &Path) {
        self.jj_at(&self.root, &["workspace", "add", "--name", name, "-r", &self.baseline, path.to_str().unwrap()]);
        // These filesystem fixtures model deliberately named work.
        // Dedicated bookmark-policy tests use their own unbookmarked fixtures.
        self.jj_at(
            path,
            &[
                "--ignore-working-copy",
                "bookmark",
                "create",
                "--",
                &serde_json::to_string(&format!("fixture-work-{name}")).unwrap(),
            ],
        );
    }

    fn cow(&self, source: &Path, target: &Path) -> bool {
        let probe = target.join(".reseed-cow-probe");
        match reflink_copy::reflink(source.join("shared"), &probe) {
            Ok(()) => {
                fs::remove_file(probe).unwrap();
                true
            }
            Err(error) => {
                assert!(
                    std::env::var_os("JW_TEST_REQUIRE_COW").is_none_or(|value| value.is_empty()),
                    "required reflink unavailable: {error}"
                );
                // Permission and unrelated IO failures must not silently skip.
                assert!(matches!(error.raw_os_error(), Some(18 | 22 | 38 | 95)), "unexpected reflink error: {error}");
                eprintln!("skipping real-CoW test: filesystem does not support reflinks: {error}");
                false
            }
        }
    }

    fn reseed(&self, names: &[&str], source: &Path, extra: &[&str]) -> Output {
        self.jw(&self.root)
            .arg("reseed")
            .args(names)
            .arg("--from")
            .arg(source)
            .arg("--source-at")
            .arg(&self.baseline)
            .args(extra)
            .output()
            .unwrap()
    }

    fn unchanged_failure(&self, target: &Path, output: Output, before: &str) {
        assert!(!output.status.success(), "unexpected success: {}", String::from_utf8_lossy(&output.stdout));
        assert_eq!(self.id(target, "@"), before);
        assert_eq!(fs::read(target.join("shared")).unwrap(), vec![b's'; 128 * 1024]);
        assert_eq!(self.jj_at(target, &["sparse", "list"]), ".");
    }

    fn lifecycle(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let config = PathBuf::from(self.jj_at(&self.root, &["config", "path", "--repo"]));
        let dir = config.parent().unwrap().join("jj-waltz/workspaces");
        let mut records: Vec<_> = fs::read_dir(dir)
            .into_iter()
            .flatten()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "json"))
            .map(|p| {
                let bytes = fs::read(&p).unwrap();
                (p, bytes)
            })
            .collect();
        records.sort_by(|a, b| a.0.cmp(&b.0));
        records
    }

    fn pending_records(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let config = PathBuf::from(self.jj_at(&self.root, &["--ignore-working-copy", "config", "path", "--repo"]));
        let dir = config.parent().unwrap().join("jj-waltz/reseed");
        let mut records: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.file_name().unwrap().to_string_lossy().ends_with(".pending.json"))
            .map(|path| {
                let bytes = fs::read(&path).unwrap();
                (path, bytes)
            })
            .collect();
        records.sort_by(|a, b| a.0.cmp(&b.0));
        records
    }

    fn prepare_links(&mut self, config: &str, extra_ignores: &str) {
        // Configure before adding sibling source/target changes so the prepared
        // source and frozen baseline include exactly the same tracked config.
        let mut ignores = fs::read_to_string(self.root.join(".gitignore")).unwrap();
        ignores.push_str(extra_ignores);
        fs::write(self.root.join(".gitignore"), ignores).unwrap();
        fs::write(self.root.join(".jwlinks.toml"), config).unwrap();
        self.baseline = self.id(&self.root, "@");
    }

    fn interrupt_reseed(&self, source: &Path, boundary: &str, extra: &[&str]) {
        self.interrupt_reseed_with_assertion(source, boundary, extra, Some(&self.baseline));
    }

    fn interrupt_reseed_with_assertion(&self, source: &Path, boundary: &str, extra: &[&str], source_at: Option<&str>) {
        let proxy = self.temp.path().join(format!("proxy-{boundary}"));
        fs::create_dir(&proxy).unwrap();
        // A mise shim can rediscover the injected proxy and recurse forever.
        // Resolve the native executable instead, preserving release-matrix PATH.
        let real_jj = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
            .map(|directory| directory.join("jj"))
            .filter_map(|path| fs::canonicalize(path).ok())
            .find(|path| path.is_file() && path.file_name().is_none_or(|name| name != "mise"))
            .expect("native jj executable on PATH");
        let marker = proxy.join("interrupted");
        // Fail only after native JJ succeeds: the operation exists but jw has
        // not received success and therefore may not have recorded its result.
        fs::write(
            proxy.join("jj"),
            r#"#!/bin/sh
hit=no
case "$JW_INTERRUPT_BOUNDARY: $* " in
  clear:*' sparse set '*)
    case " $* " in *' --clear '*) hit=yes;; esac;;
  new:*' new '*) hit=yes;;
  expand:*' sparse reset '*|expand:*' sparse set '*)
    case " $* " in *' --clear '*) ;; *) hit=yes;; esac;;
esac
if [ "$hit" = yes ]; then
  "$JW_REAL_JJ" "$@" || exit $?
  : > "$JW_INTERRUPT_MARKER"
  echo "injected lost-result failure after $JW_INTERRUPT_BOUNDARY" >&2
  exit 91
fi
exec "$JW_REAL_JJ" "$@"
"#,
        )
        .unwrap();
        fs::set_permissions(proxy.join("jj"), fs::Permissions::from_mode(0o755)).unwrap();
        let mut paths = vec![proxy];
        paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()));
        let mut command = self.jw(&self.root);
        command.args(["reseed", "feature", "--from"]).arg(source);
        if let Some(source_at) = source_at {
            command.args(["--source-at", source_at]);
        }
        let output = command
            .args(extra)
            .env("PATH", std::env::join_paths(paths).unwrap())
            .env("JW_REAL_JJ", real_jj)
            .env("JW_INTERRUPT_BOUNDARY", boundary)
            .env("JW_INTERRUPT_MARKER", &marker)
            .output()
            .unwrap();
        assert!(!output.status.success(), "fault must interrupt reseed");
        assert!(marker.is_file(), "fault must reach {boundary}: {}", String::from_utf8_lossy(&output.stderr));
        let message = String::from_utf8_lossy(&output.stdout);
        assert!(message.contains("covered by bookmark"), "{message}");
        assert!(message.contains("recover with `jj new ") && message.contains("`jj sparse reset`"), "{message}");
        assert!(message.contains("then re-run reseed"), "{message}");
        assert!(self.pending_records().is_empty(), "interruption must not write a journal");
    }
}

fn identity(path: &Path) -> (u64, u64) {
    let m = fs::symlink_metadata(path).unwrap();
    (m.dev(), m.ino())
}

fn frozen_id(repo: &Repo, root: &Path) -> String {
    repo.jj_at(root, &["--ignore-working-copy", "log", "--no-graph", "-r", "@", "-T", "commit_id"])
}

fn operation(repo: &Repo, root: &Path) -> String {
    repo.jj_at(root, &["--ignore-working-copy", "operation", "log", "--limit=1", "--no-graph", "-T", "id"])
}

fn group_config(repo: &Repo, body: &str) {
    fs::write(repo.root.join(".jwreseed.local.toml"), format!("source_at={:?}\n{body}\n", repo.baseline)).unwrap();
}

#[test]
fn builtin_all_from_worker_excludes_renamed_primary_and_refills_worker_named_default() {
    let Some(repo) = Repo::new() else { return };
    repo.jj_at(&repo.root, &["workspace", "rename", "primary"]);
    let z = repo.workspace("z");
    let default = repo.workspace("default");
    if !repo.cow(&repo.root, &default) {
        return;
    }
    fs::write(default.join("modified"), "worker named default\n").unwrap();
    let subdir = z.join("cache/subdir");
    fs::create_dir_all(&subdir).unwrap();
    let preview = success(repo.jw(&subdir).args(["reseed", "--group", "all", "--dry-run"]).output().unwrap());
    let selected = preview.lines().filter(|line| line.starts_with("SELECT ")).collect::<Vec<_>>();
    assert_eq!(selected.len(), 2, "{preview}");
    assert!(selected[0].starts_with("SELECT \"default\""), "{preview}");
    assert!(selected[1].starts_with("SELECT \"z\""), "{preview}");
    assert!(!preview.contains("SELECT \"primary\""), "{preview}");
    for name in ["default", "z"] {
        assert!(preview.contains(&format!("PLAN {name}: READY")), "{preview}");
    }
    assert!(preview.contains(&format!("SYNC to {}", repo.baseline)), "{preview}");
    let roots = [identity(&default), identity(&z)];
    let output = success(repo.jw(&subdir).args(["reseed", "--group", "all"]).output().unwrap());
    assert!(output.contains("OK default:"), "{output}");
    assert!(output.contains("OK z:"), "{output}");
    // Plain reseed syncs each workspace's content to the donor's current commit.
    assert_eq!(fs::read(default.join("modified")).unwrap(), b"baseline\n");
    assert_eq!([identity(&default), identity(&z)], roots);
    for target in [&default, &z] {
        assert_eq!(repo.id(target, "@-"), repo.baseline);
        assert_shared_extents(&repo.root.join("shared"), &target.join("shared"));
    }
    assert!(repo.pending_records().is_empty());
}

#[test]
fn builtin_all_excludes_effective_configured_or_cli_aliased_donor() {
    let Some(repo) = Repo::new() else { return };
    let configured = repo.workspace("configured");
    let explicit = repo.workspace("explicit");
    let target = repo.workspace("feature");
    if !repo.cow(&configured, &target) {
        return;
    }
    let alias = repo.temp.path().join("donor-alias");
    symlink(&explicit, &alias).unwrap();
    group_config(&repo, "source='../workspace-configured'");
    for cli_override in [false, true] {
        let mut command = repo.jw(&target);
        command.args(["reseed", "--group", "all", "--dry-run"]);
        if cli_override {
            command.args(["--from", "../donor-alias"]);
        }
        let preview = success(command.output().unwrap());
        let (excluded, included, donor) = if cli_override {
            ("explicit", "configured", &explicit)
        } else {
            ("configured", "explicit", &configured)
        };
        assert!(!preview.contains(&format!("SELECT {excluded:?}")), "{preview}");
        assert!(!preview.contains("SELECT \"default\""), "{preview}");
        assert!(preview.contains(&format!("PLAN {included}: READY")), "{preview}");
        assert!(preview.contains("PLAN feature: READY"), "{preview}");
        assert!(preview.contains(&format!("Dry-run source: {}", donor.display())), "{preview}");
    }
}

#[test]
fn builtin_all_keeps_bad_members_blocked_and_never_refills_ready_subset() {
    let Some(repo) = Repo::new() else { return };
    let ready = repo.workspace("ready");
    let missing = repo.workspace("missing");
    let sparse = repo.workspace("sparse");
    let colocated = repo.workspace("colocated");
    let unbookmarked = repo.workspace("unbookmarked");
    if !repo.cow(&repo.root, &ready) {
        return;
    }
    fs::rename(missing, repo.temp.path().join("moved-missing")).unwrap();
    repo.jj_at(&sparse, &["sparse", "set", "--clear", "--add", "shared"]);
    fs::write(colocated.join(".git"), "gitdir: /nonexistent\n").unwrap();
    repo.jj_at(&unbookmarked, &["new"]);
    fs::write(unbookmarked.join("modified"), "work without a covering bookmark\n").unwrap();
    let ready_shared = identity(&ready.join("shared"));
    let ready_commit = frozen_id(&repo, &ready);
    let output = repo
        .jw(&ready)
        .args(["reseed", "--group", "all", "--on-unmatched", "warn", "--dry-run"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PLAN ready: READY"), "{stdout}");
    for name in ["missing", "sparse", "colocated", "unbookmarked"] {
        assert!(stdout.contains(&format!("BLOCKED {name}:")), "{stdout}");
    }
    assert!(stdout.contains("unbookmarked work tip"), "{stdout}");
    assert!(!String::from_utf8_lossy(&output.stderr).contains("warning: skipped"));
    let output = repo.jw(&ready).args(["reseed", "--group", "all"]).output().unwrap();
    assert!(!output.status.success());
    assert_eq!(identity(&ready.join("shared")), ready_shared);
    assert_eq!(frozen_id(&repo, &ready), ready_commit);
    assert!(repo.pending_records().is_empty());
}

#[test]
fn builtin_all_rejects_local_collision_removed_resume_and_empty_membership() {
    let Some(repo) = Repo::new() else { return };
    let output = repo.jw(&repo.root).args(["reseed", "--group", "all", "--dry-run"]).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no workspaces selected"));
    group_config(&repo, "[groups]\nall=['default']");
    let output = repo.jw(&repo.root).args(["reseed", "--group", "all", "--dry-run"]).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("reserved"));
    let output = repo.jw(&repo.root).args(["reseed", "--group", "all", "--resume"]).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument '--resume'"));
}

#[test]
fn readiness_aggregates_early_and_late_blockers_and_real_run_never_refills_a_subset() {
    let Some(mut repo) = Repo::new() else { return };
    repo.prepare_links(
        "[[link]]\nsource='dependency-link'\ntarget='private/file'\nrequired=true\n",
        "dependency-link\nprivate/\n",
    );
    let ready = repo.workspace("ready");
    let sparse = repo.workspace("sparse");
    let missing = repo.workspace("missing");
    let ignored = repo.workspace("ignored-rules");
    let link = repo.workspace("missing-link");
    if !repo.cow(&repo.root, &ready) {
        return;
    }
    fs::create_dir(repo.root.join("cache")).unwrap();
    fs::write(repo.root.join("cache/fresh"), "prepared payload\n").unwrap();
    fs::create_dir(ready.join("cache")).unwrap();
    fs::write(ready.join("cache/old"), "private old payload\n").unwrap();
    fs::create_dir(ready.join("private")).unwrap();
    fs::write(ready.join("private/file"), "retained link target\n").unwrap();
    fs::write(ready.join("modified"), "unsnapshotted feature\n").unwrap();
    let ignores = fs::read_to_string(ignored.join(".gitignore")).unwrap().replace("cache/\n", "");
    fs::write(ignored.join(".gitignore"), ignores).unwrap();
    repo.jj_at(&sparse, &["sparse", "set", "--clear", "--add", "shared"]);
    let moved = repo.temp.path().join("moved-worker");
    fs::rename(&missing, &moved).unwrap();
    let shared = identity(&ready.join("shared"));

    let output = repo.reseed(
        &["ready", "sparse", "missing", "ignored-rules", "missing-link", "unknown"],
        &repo.root,
        &["--copy-ignored", "cache", "--dry-run"],
    );
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PLAN ready: READY"), "{stdout}");
    assert!(stdout.contains(&format!("SYNC to {}", repo.baseline)), "{stdout}");
    for name in ["sparse", "missing", "ignored-rules", "missing-link"] {
        assert!(stdout.contains(&format!("BLOCKED {name}:")), "{stdout}");
    }
    assert!(stdout.contains("BLOCKED \"unknown\":"), "{stdout}");
    assert!(stdout.contains("required link target will be absent"), "{stdout}");
    assert_eq!(identity(&ready.join("shared")), shared);
    assert_eq!(fs::read(ready.join("modified")).unwrap(), b"unsnapshotted feature\n");
    assert_eq!(fs::read(ready.join("cache/old")).unwrap(), b"private old payload\n");
    assert!(!ready.join("cache/fresh").exists());
    assert!(!ready.join("dependency-link").exists());
    assert!(repo.pending_records().is_empty());

    // Late planning failures must also prevent the already-valid first target
    // from entering execution; no journal, clearing, CoW replacement or links.
    let before = frozen_id(&repo, &ready);
    let output = repo.reseed(&["ready", "ignored-rules", "missing-link"], &repo.root, &["--copy-ignored", "cache"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    for text in ["ignored-rules", "missing-link", "no targets refilled"] {
        assert!(stderr.contains(text), "{stderr}");
    }
    assert_eq!(frozen_id(&repo, &ready), before);
    assert_eq!(identity(&ready.join("shared")), shared);
    assert_eq!(fs::read(ready.join("cache/old")).unwrap(), b"private old payload\n");
    assert!(!ready.join("cache/fresh").exists());
    assert!(!ready.join("dependency-link").exists());
    assert!(repo.pending_records().is_empty());
    assert!(link.is_dir());
}

#[test]
fn readiness_reports_one_sync_consequence_without_resetting_or_deleting_files() {
    let Some(repo) = Repo::new() else { return };
    let target = repo.workspace("feature");
    if !repo.cow(&repo.root, &target) {
        return;
    }
    let original = frozen_id(&repo, &target);
    fs::write(target.join("modified"), "keep this unfinished work\n").unwrap();
    fs::write(target.join("feature-only"), "new unfinished file\n").unwrap();
    fs::write(target.join(".env"), "PRIVATE=retained\n").unwrap();
    let shared = identity(&target.join("shared"));
    let plain = success(repo.reseed(&["feature"], &repo.root, &["--dry-run"]));
    assert!(plain.contains("JJ snapshots may record working-copy changes"), "{plain}");
    assert!(plain.contains("new working-copy updates snapshotted"), "{plain}");
    assert!(plain.contains("uncommitted working-copy changes present"), "{plain}");
    assert!(plain.contains(&format!("SYNC to {}", repo.baseline)), "{plain}");
    assert!(!plain.contains("RESET checkout"), "{plain}");
    assert!(!plain.contains("KEEP current feature tree"), "{plain}");
    let preserved = frozen_id(&repo, &target);
    assert_ne!(preserved, original, "preview deliberately preserves work by snapshotting");
    let overridden = success(repo.reseed(&["feature"], &repo.root, &["--at", &repo.baseline, "--replace-content", "--dry-run"]));
    assert!(overridden.contains(&format!("SYNC to {}", repo.baseline)), "{overridden}");
    assert!(overridden.contains(&format!("tracked work saved in JJ history at {preserved}")), "{overridden}");
    assert!(overridden.contains("no new working-copy updates recorded"), "{overridden}");
    assert_eq!(frozen_id(&repo, &target), preserved, "dry-run must not create the sync child");
    assert_eq!(identity(&target.join("shared")), shared);
    assert_eq!(fs::read(target.join("modified")).unwrap(), b"keep this unfinished work\n");
    assert_eq!(fs::read(target.join("feature-only")).unwrap(), b"new unfinished file\n");
    assert_eq!(fs::read(target.join(".env")).unwrap(), b"PRIVATE=retained\n");
    assert_eq!(repo.jj_at(&target, &["--ignore-working-copy", "sparse", "list"]), ".");
    assert!(repo.pending_records().is_empty());
}

#[test]
fn readiness_shared_source_failure_is_not_assessed_and_never_claims_ready() {
    let Some(repo) = Repo::new() else { return };
    let a = repo.workspace("a");
    let b = repo.workspace("b");
    // Do not let the donor snapshot rewrite the selected workers' ancestry.
    repo.jj_at(&repo.root, &["new", "-r", &repo.baseline]);
    fs::write(repo.root.join("modified"), "unprepared donor\n").unwrap();
    let before = [frozen_id(&repo, &a), frozen_id(&repo, &b)];
    for target in [&a, &b] {
        fs::write(target.join("modified"), "do not snapshot unchecked target\n").unwrap();
    }
    let output = repo.reseed(&["a", "b"], &repo.root, &["--dry-run"]);
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for name in ["a", "b"] {
        assert!(stdout.contains(&format!("BLOCKED {name}: not assessed: shared preflight failed")), "{stdout}");
    }
    assert!(stdout.contains("source tracked tree does not match source_at"), "{stdout}");
    assert!(!stdout.contains("READY"), "{stdout}");
    assert_eq!([frozen_id(&repo, &a), frozen_id(&repo, &b)], before);
    assert!(repo.pending_records().is_empty());
}

#[test]
fn readiness_sparse_empty_target_is_never_snapshotted_while_other_targets_are_checked() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let pending = repo.workspace("feature");
    let ready = repo.workspace("ready");
    if !repo.cow(&source, &pending) {
        return;
    }
    repo.interrupt_reseed(&source, "clear", &[]);
    fs::write(pending.join("shared"), "partial refill must not be snapshotted\n").unwrap();
    let before = frozen_id(&repo, &pending);
    let output = repo.reseed(&["feature", "ready"], &source, &["--dry-run"]);
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BLOCKED feature:"), "{stdout}");
    assert!(stdout.contains("workspace is sparse or mid-refill"), "{stdout}");
    assert!(stdout.contains("jj new <bookmark>") && stdout.contains("jj sparse reset"), "{stdout}");
    assert!(stdout.contains("PLAN ready: READY"), "{stdout}");
    assert_eq!(frozen_id(&repo, &pending), before);
    assert_eq!(fs::read(pending.join("shared")).unwrap(), b"partial refill must not be snapshotted\n");
    assert!(repo.pending_records().is_empty());
    assert_eq!(repo.jj_at(&pending, &["--ignore-working-copy", "sparse", "list"]), "");
    assert_eq!(repo.jj_at(&ready, &["--ignore-working-copy", "sparse", "list"]), ".");
}

fn skip_warning(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let lines = stderr.lines().filter(|line| line.starts_with("warning: skipped ")).collect::<Vec<_>>();
    assert_eq!(lines.len(), 1, "expected one aggregate warning: {stderr}");
    lines[0].to_owned()
}

#[test]
fn wildcard_group_skips_mixed_checkouts_in_one_line_and_uses_registered_names() {
    let Some(repo) = Repo::new() else { return };
    let a = repo.temp.path().join("opencode-a");
    let b = repo.temp.path().join("opencode-b");
    repo.workspace_at("worker-z", &a);
    repo.workspace_at("worker-a", &b);
    if !repo.cow(&repo.root, &a) {
        return;
    }
    let plain = repo.temp.path().join("opencode-notes");
    let git = repo.temp.path().join("opencode-git");
    let independent = repo.temp.path().join("opencode-independent");
    fs::create_dir(&plain).unwrap();
    success(repo.command("git", repo.temp.path()).args(["init", git.to_str().unwrap()]).output().unwrap());
    repo.jj_at(repo.temp.path(), &["git", "init", "--no-colocate", independent.to_str().unwrap()]);
    for root in [&plain, &git, &independent] {
        fs::write(root.join("sentinel"), "do not touch\n").unwrap();
    }
    let independent_op = operation(&repo, &independent);
    let git_head = fs::read(git.join(".git/HEAD")).unwrap();
    let identities = [&a, &b, &plain, &git, &independent].map(|p| identity(p));
    let subdir = a.join("cache/subdir");
    fs::create_dir_all(&subdir).unwrap();
    fs::write(a.join("modified"), "feature\n").unwrap();
    let before = repo.id(&a, "@");
    group_config(&repo, "[groups.opencode]\npath_globs=['~/opencode-*']\non_unmatched='warn'");
    let mut command = repo.jw(&subdir);
    command.args(["reseed", "--group", "opencode"]).env("HOME", repo.temp.path()).arg("--dry-run");
    let output = command.output().unwrap();
    let warning = skip_warning(&output);
    for name in ["opencode-notes", "opencode-git", "opencode-independent"] {
        assert!(warning.contains(name), "{warning}");
    }
    assert!(warning.starts_with("warning: skipped 3 "), "{warning}");
    let stdout = success(output);
    assert!(stdout.contains("PLAN worker-z:"), "{stdout}");
    assert!(stdout.contains("PLAN worker-a:"), "{stdout}");
    assert!(stdout.contains("SELECT \"worker-z\""), "{stdout}");
    assert!(stdout.contains("path_globs"), "{stdout}");
    assert_eq!(repo.id(&a, "@"), before, "dry-run must not sync");
    assert_eq!(fs::read_to_string(a.join("modified")).unwrap(), "feature\n");
    assert_eq!(operation(&repo, &independent), independent_op);
    assert_eq!(fs::read(git.join(".git/HEAD")).unwrap(), git_head);
    for root in [&plain, &git, &independent] {
        assert_eq!(fs::read_to_string(root.join("sentinel")).unwrap(), "do not touch\n");
    }
    assert_eq!([&a, &b, &plain, &git, &independent].map(|p| identity(p)), identities);
    let output = repo
        .jw(&subdir)
        .args(["reseed", "--group", "opencode"])
        .env("HOME", repo.temp.path())
        .output()
        .unwrap();
    let warning = skip_warning(&output);
    for name in ["opencode-notes", "opencode-git", "opencode-independent"] {
        assert!(warning.contains(name), "{warning}");
    }
    assert!(warning.starts_with("warning: skipped 3 "), "{warning}");
    let stdout = success(output);
    assert!(stdout.contains("OK worker-z:"), "{stdout}");
    assert!(stdout.contains("OK worker-a:"), "{stdout}");
    // The real run syncs both workers to the donor's current commit.
    assert_eq!(repo.id(&a, "@-"), repo.baseline);
    assert_eq!(fs::read_to_string(a.join("modified")).unwrap(), "baseline\n");
    assert_eq!(operation(&repo, &independent), independent_op);
    assert_eq!(fs::read(git.join(".git/HEAD")).unwrap(), git_head);
    for root in [&plain, &git, &independent] {
        assert_eq!(fs::read_to_string(root.join("sentinel")).unwrap(), "do not touch\n");
    }
    assert_eq!([&a, &b, &plain, &git, &independent].map(|p| identity(p)), identities);
    assert_shared_extents(&repo.root.join("shared"), &a.join("shared"));
    assert_shared_extents(&repo.root.join("shared"), &b.join("shared"));
}

#[test]
fn wildcard_group_defaults_and_override_fail_before_any_snapshot() {
    let Some(repo) = Repo::new() else { return };
    let target = repo.workspace("feature");
    fs::create_dir(repo.temp.path().join("workspace-unrelated")).unwrap();
    for policy in ["", "on_unmatched='warn'"] {
        group_config(&repo, &format!("[groups.w]\npath_globs=['~/workspace-*']\n{policy}"));
        let before = frozen_id(&repo, &target);
        let op = operation(&repo, &repo.root);
        fs::write(target.join("modified"), "unsnapshotted target\n").unwrap();
        fs::write(repo.root.join("modified"), "unsnapshotted donor\n").unwrap();
        let mut cmd = repo.jw(&target);
        cmd.args(["reseed", "--group", "w"]).env("HOME", repo.temp.path());
        if !policy.is_empty() {
            cmd.args(["--on-unmatched", "error"]);
        }
        let output = cmd.output().unwrap();
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("workspace-unrelated"), "{stderr}");
        assert!(!stderr.contains("warning: skipped"));
        assert_eq!(operation(&repo, &repo.root), op);
        assert_eq!(frozen_id(&repo, &target), before);
    }
}

#[test]
fn wildcard_group_warning_survives_later_source_failure() {
    let Some(repo) = Repo::new() else { return };
    let target = repo.workspace("feature");
    // Keep the donor's dirty snapshot from rewriting the target's ancestor.
    repo.jj_at(&repo.root, &["new"]);
    fs::create_dir(repo.temp.path().join("workspace-unrelated")).unwrap();
    group_config(&repo, "[groups.w]\npath_globs=['~/workspace-*']\non_unmatched='warn'");
    let before = frozen_id(&repo, &target);
    fs::write(target.join("modified"), "unsnapshotted feature\n").unwrap();
    fs::write(repo.root.join("modified"), "unprepared donor\n").unwrap();
    let output = repo
        .jw(&target)
        .args(["reseed", "--group", "w"])
        .env("HOME", repo.temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(skip_warning(&output).contains("workspace-unrelated"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("source tracked tree does not match"));
    assert_eq!(frozen_id(&repo, &target), before);
}

#[test]
fn wildcard_group_all_skipped_is_a_failure_and_positional_names_stay_strict() {
    let Some(repo) = Repo::new() else { return };
    let target = repo.workspace("feature");
    fs::create_dir(repo.temp.path().join("opencode-notes")).unwrap();
    group_config(
        &repo,
        "[groups.w]\npath_globs=['~/opencode-*']\non_unmatched='warn'\n[groups.names]\nname_globs=['feat*']",
    );
    let before = operation(&repo, &repo.root);
    let output = repo
        .jw(&target)
        .args(["reseed", "--group", "w"])
        .env("HOME", repo.temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(skip_warning(&output).contains("opencode-notes"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("no workspaces selected"));
    let output = repo
        .jw(&target)
        .args(["reseed", "typo", "--group", "names", "--group", "w", "--on-unmatched", "warn"])
        .env("HOME", repo.temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(skip_warning(&output).contains("opencode-notes"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("typo\" (unknown workspace)"));
    assert_eq!(operation(&repo, &repo.root), before);
}

#[test]
fn wildcard_group_name_and_path_selectors_deduplicate_across_legacy_groups() {
    let Some(repo) = Repo::new() else { return };
    let target = repo.workspace("feature");
    if !repo.cow(&repo.root, &target) {
        return;
    }
    group_config(
        &repo,
        "[groups]\nlegacy=['feature','missing']\n[groups.w]\nnames=['feature']\nname_globs=['feat*']\npath_globs=['~/workspace-feature']",
    );
    let output = repo
        .jw(&target)
        .args(["reseed", "feature", "--group", "w", "--group", "legacy", "--on-unmatched", "warn", "--dry-run"])
        .env("HOME", repo.temp.path())
        .output()
        .unwrap();
    assert!(skip_warning(&output).contains("missing"));
    let stdout = success(output);
    assert_eq!(stdout.lines().filter(|l| l.starts_with("PLAN feature:")).count(), 1);
    for origin in ["positional", "name_globs", "path_globs", "legacy"] {
        assert!(stdout.contains(origin), "{stdout}");
    }
}

#[test]
fn wildcard_group_warn_does_not_skip_registered_target_safety_failures() {
    for scenario in ["primary", "sparse", "missing", "identity", "colocated"] {
        let Some(repo) = Repo::new() else { return };
        let target = repo.workspace("feature");
        let pattern = if scenario == "primary" { "default" } else { "feat*" };
        match scenario {
            "sparse" => {
                repo.jj_at(&target, &["sparse", "set", "--clear", "--add", "shared"]);
            }
            "missing" => {
                fs::rename(&target, repo.temp.path().join("moved-target")).unwrap();
            }
            "identity" => {
                fs::write(target.join(".jj/repo"), repo.temp.path().join("not-repository").to_str().unwrap()).unwrap();
            }
            "colocated" => {
                fs::create_dir(target.join(".git")).unwrap();
            }
            _ => {}
        }
        group_config(&repo, &format!("[groups.w]\nname_globs=[{pattern:?}]\non_unmatched='warn'"));
        let op = operation(&repo, &repo.root);
        let output = repo.jw(&repo.root).args(["reseed", "--group", "w", "--dry-run"]).output().unwrap();
        assert!(!output.status.success(), "{scenario}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains("warning: skipped"), "{scenario}: {stderr}");
        assert_eq!(operation(&repo, &repo.root), op, "{scenario}: {stderr}");
    }
}

#[test]
fn primary_default_from_sibling_and_subdirectory_syncs_the_feature_variant() {
    let Some(repo) = Repo::new() else { return };
    let caller = repo.workspace("caller");
    let target = repo.workspace("feature");
    if !repo.cow(&repo.root, &target) {
        return;
    }
    fs::write(target.join("modified"), "feature\n").unwrap();
    fs::write(target.join("feature-only"), "keep\n").unwrap();
    let subdir = caller.join("cache/subdir");
    fs::create_dir_all(&subdir).unwrap();
    for cwd in [&caller, &subdir, &repo.root] {
        success(repo.jw(cwd).args(["reseed", "feature", "--source-at", &repo.baseline]).output().unwrap());
        // Plain reseed makes the workspace's content the donor's content.
        assert_eq!(repo.id(&target, "@-"), repo.baseline);
        assert_eq!(fs::read_to_string(target.join("modified")).unwrap(), "baseline\n");
        assert!(!target.join("feature-only").exists());
        assert_shared_extents(&repo.root.join("shared"), &target.join("shared"));
    }
}

#[test]
fn renamed_primary_is_default_donor_even_with_another_workspace_named_default() {
    let Some(repo) = Repo::new() else { return };
    repo.jj_at(&repo.root, &["workspace", "rename", "primary"]);
    let target = repo.workspace("feature");
    if !repo.cow(&repo.root, &target) {
        return;
    }
    for add_default in [false, true] {
        if add_default {
            let decoy = repo.workspace("default");
            fs::write(decoy.join("modified"), "not the donor\n").unwrap();
        }
        let output = success(repo.jw(&target).args(["reseed", "feature", "--source-at", &repo.baseline]).output().unwrap());
        assert!(output.contains(&format!("source: {}", repo.root.display())), "{output}");
    }
    let output = repo.jw(&target).args(["reseed", "primary", "--source-at", &repo.baseline]).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("primary/source"));
}

#[test]
fn configured_relative_source_and_explicit_override_win_over_primary() {
    let Some(repo) = Repo::new() else { return };
    let configured = repo.workspace("configured");
    let explicit = repo.workspace("explicit");
    let target = repo.workspace("feature");
    if !repo.cow(&configured, &target) {
        return;
    }
    fs::write(
        repo.root.join(".jwreseed.local.toml"),
        format!("source = '../workspace-configured'\nsource_at = '{}'\n", repo.baseline),
    )
    .unwrap();
    // Neither override may fall back to the dirty primary's nonmatching tree.
    fs::write(repo.root.join("modified"), "dirty primary\n").unwrap();
    let subdir = target.join("cache/subdir");
    fs::create_dir_all(&subdir).unwrap();
    let output = success(repo.jw(&subdir).args(["reseed", "feature", "--dry-run"]).output().unwrap());
    assert!(output.contains(&format!("source: {}", configured.display())), "{output}");
    let output = success(
        repo.jw(&subdir)
            .args(["reseed", "feature", "--from", "../../../workspace-explicit", "--dry-run"])
            .output()
            .unwrap(),
    );
    assert!(output.contains(&format!("source: {}", explicit.display())), "{output}");
    let before = repo.id(&target, "@");
    let output = repo.jw(&target).args(["reseed", "feature", "--from"]).arg(&target).output().unwrap();
    repo.unchanged_failure(&target, output, &before);
}

#[test]
fn source_assertions_keep_cli_over_local_precedence_and_reject_before_target_snapshot() {
    let Some(repo) = Repo::new() else { return };
    let target = repo.workspace("feature");
    if !repo.cow(&repo.root, &target) {
        return;
    }
    let before = repo.id(&target, "@");
    repo.jj_at(&repo.root, &["new", "-r", &repo.baseline]);
    fs::write(repo.root.join("modified"), "dirty primary\n").unwrap();
    fs::write(target.join("modified"), "unsnapshotted target\n").unwrap();
    group_config(&repo, "");
    for extra in [&[][..], &["--source-at", repo.baseline.as_str()][..]] {
        let output = repo.jw(&target).args(["reseed", "feature"]).args(extra).output().unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("source tracked tree does not match source_at"),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(frozen_id(&repo, &target), before);
    }
    assert_eq!(fs::read_to_string(target.join("modified")).unwrap(), "unsnapshotted target\n");
    assert_eq!(repo.jj_at(&target, &["--ignore-working-copy", "sparse", "list"]), ".");
    let output = success(repo.jw(&target).args(["reseed", "feature", "--source-at", "@"]).output().unwrap());
    assert!(output.contains(&frozen_id(&repo, &repo.root)), "{output}");
    // The plain default syncs the target onto the donor's current commit.
    assert_eq!(repo.id(&target, "@-"), frozen_id(&repo, &repo.root));
    assert_eq!(fs::read(target.join("modified")).unwrap(), b"dirty primary\n");
}

#[test]
fn bare_reseed_borrows_snapshotted_primary_and_syncs_current_registered_variant() {
    let Some(repo) = Repo::new() else { return };
    repo.jj_at(&repo.root, &["workspace", "rename", "primary"]);
    let target = repo.workspace("feature");
    let decoy = repo.workspace("default");
    if !repo.cow(&repo.root, &target) {
        return;
    }
    // Keep the mutable donor off the target's ancestry, as ordinary users must.
    repo.jj_at(&repo.root, &["new", &repo.baseline]);
    let old_source = frozen_id(&repo, &repo.root);
    fs::write(repo.root.join("modified"), "unsnapshotted donor\n").unwrap();
    fs::write(target.join("modified"), "feature work\n").unwrap();
    fs::write(decoy.join("modified"), "not the donor or target\n").unwrap();
    let decoy_id = frozen_id(&repo, &decoy);
    let identities = [identity(&target), identity(&target.join(".jj"))];
    let subdir = target.join("cache/subdirectory");
    fs::create_dir_all(&subdir).unwrap();
    let output = success(repo.jw(&subdir).arg("reseed").output().unwrap());
    assert!(output.contains("OK feature"), "{output}");
    assert!(!output.contains("OK default"), "{output}");
    assert_eq!(identities, [identity(&target), identity(&target.join(".jj"))]);
    assert_eq!(frozen_id(&repo, &decoy), decoy_id, "never snapshot another variant");
    assert_ne!(frozen_id(&repo, &repo.root), old_source);
    assert!(output.contains(&format!("source: {}", repo.root.display())), "{output}");
    assert!(output.contains(&frozen_id(&repo, &repo.root)), "{output}");
    // Bare reseed syncs the current variant onto the donor's snapshotted commit.
    assert_eq!(repo.id(&target, "@-"), frozen_id(&repo, &repo.root));
    assert_eq!(fs::read(target.join("modified")).unwrap(), b"unsnapshotted donor\n");
    assert_shared_extents(&repo.root.join("shared"), &target.join("shared"));
}

#[test]
fn explicit_selection_suppresses_current_and_preflight_failures_do_not_snapshot() {
    let Some(repo) = Repo::new() else { return };
    let target = repo.workspace("feature");
    let other = repo.workspace("other");
    repo.jj_at(&repo.root, &["new", &repo.baseline]);
    fs::write(repo.root.join("modified"), "unsnapshotted donor\n").unwrap();
    fs::write(target.join("modified"), "unsnapshotted current\n").unwrap();
    fs::write(
        repo.root.join(".jwreseed.local.toml"),
        "[groups]\nothers=['other']\n[groups.missing]\nname_globs=['absent-*']\non_unmatched='warn'\n",
    )
    .unwrap();
    let before = operation(&repo, &repo.root);
    for (cwd, args, message) in [
        (repo.root.as_path(), vec!["reseed"], "primary checkout requires literal target names"),
        (target.as_path(), vec!["reseed", "--group", "missing"], "no workspaces selected"),
        (
            target.as_path(),
            vec!["reseed", "--from", target.to_str().unwrap()],
            "primary/source or overlaps",
        ),
        (
            target.as_path(),
            vec!["reseed", "--freshen", "--dry-run"],
            "reseed --freshen was removed; use `jw prepare`",
        ),
        (target.as_path(), vec!["reseed", "@"], "unknown workspace"),
    ] {
        let output = repo.jw(cwd).args(args).output().unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(message),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(operation(&repo, &repo.root), before, "{message}");
    }
    if !repo.cow(&repo.root, &other) {
        return;
    }
    let current = frozen_id(&repo, &target);
    for args in [vec!["reseed", "other", "--dry-run"], vec!["reseed", "--group", "others", "--dry-run"]] {
        let output = success(repo.jw(&target).args(args).output().unwrap());
        assert!(output.contains("PLAN other"), "{output}");
        assert!(!output.contains("PLAN feature"), "{output}");
        assert_eq!(frozen_id(&repo, &target), current);
    }
}

#[test]
fn unregistered_or_moved_primary_fails_without_snapshotting_target() {
    for moved in [false, true] {
        let Some(repo) = Repo::new() else { return };
        let target = repo.workspace("feature");
        let before = repo.id(&target, "@");
        fs::write(target.join("modified"), "unsnapshotted feature\n").unwrap();
        if moved {
            let moved_root = repo.temp.path().join("moved-primary");
            fs::rename(&repo.root, &moved_root).unwrap();
            // Keep the repository accessible at its original identity, but move
            // the primary working copy away from its registered root.
            fs::create_dir_all(repo.root.join(".jj")).unwrap();
            fs::rename(moved_root.join(".jj/repo"), repo.root.join(".jj/repo")).unwrap();
        } else {
            repo.jj_at(&target, &["--ignore-working-copy", "workspace", "forget", "default"]);
        }
        let output = repo.jw(&target).args(["reseed", "feature", "--source-at", &repo.baseline]).output().unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("primary"),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            repo.jj_at(&target, &["--ignore-working-copy", "log", "--no-graph", "-r", "@", "-T", "commit_id"]),
            before
        );
        assert_eq!(fs::read_to_string(target.join("modified")).unwrap(), "unsnapshotted feature\n");
        assert_eq!(repo.jj_at(&target, &["--ignore-working-copy", "sparse", "list"]), ".");
    }
}

#[test]
fn source_advance_during_target_refresh_fails_before_sparse_mutation() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    let before = repo.id(&target, "@");
    let proxy = repo.temp.path().join("refresh-proxy");
    fs::create_dir(&proxy).unwrap();
    let real_jj = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|directory| directory.join("jj"))
        .filter_map(|path| fs::canonicalize(path).ok())
        .find(|path| path.is_file() && path.file_name().is_none_or(|name| name != "mise"))
        .expect("native jj executable on PATH");
    let marker = proxy.join("advanced");
    fs::write(
        proxy.join("jj"),
        r#"#!/bin/sh
case " $* " in
  *' status '*)
    if [ "$PWD" = "$JW_REFRESH_TARGET" ] && [ ! -e "$JW_REFRESH_MARKER" ]; then
      "$JW_REAL_JJ" "$@" || exit $?
      "$JW_REAL_JJ" -R "$JW_REFRESH_SOURCE" new --ignore-working-copy || exit $?
      : > "$JW_REFRESH_MARKER"
      exit 0
    fi;;
esac
exec "$JW_REAL_JJ" "$@"
"#,
    )
    .unwrap();
    fs::set_permissions(proxy.join("jj"), fs::Permissions::from_mode(0o755)).unwrap();
    let mut paths = vec![proxy];
    paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()));
    let output = repo
        .jw(&target)
        .args(["reseed", "feature", "--from"])
        .arg(&source)
        .args(["--source-at", &repo.baseline])
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env("JW_REAL_JJ", real_jj)
        .env("JW_REFRESH_TARGET", &target)
        .env("JW_REFRESH_SOURCE", &source)
        .env("JW_REFRESH_MARKER", &marker)
        .output()
        .unwrap();
    assert!(marker.is_file(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stderr).contains("source changed during target refresh"));
    repo.unchanged_failure(&target, output, &before);
    assert!(repo.pending_records().is_empty());
}

#[test]
fn plain_sync_replaces_feature_tree_types_modes_and_literal_paths() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    fs::write(target.join("modified"), "feature\n").unwrap();
    fs::remove_file(target.join("deleted")).unwrap();
    fs::remove_file(target.join("transition")).unwrap();
    symlink("modified", target.join("transition")).unwrap();
    fs::remove_file(target.join("directory-transition")).unwrap();
    fs::create_dir(target.join("directory-transition")).unwrap();
    fs::write(target.join("directory-transition/child"), "child\n").unwrap();
    fs::set_permissions(target.join("mode"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::rename(target.join("line\nbreak"), target.join("renamed\nfile")).unwrap();
    fs::write(target.join("added"), "feature only\n").unwrap();
    let root = identity(&target);
    let jj = identity(&target.join(".jj"));
    for _ in 0..2 {
        success(repo.reseed(&["feature"], &source, &[]));
        assert_eq!(identity(&target), root);
        assert_eq!(identity(&target.join(".jj")), jj);
        // The workspace's content is the donor's content, not the feature tree.
        assert_eq!(repo.id(&target, "@-"), frozen_id(&repo, &source));
        assert_eq!(fs::read_to_string(target.join("modified")).unwrap(), "baseline\n");
        assert_eq!(fs::read_to_string(target.join("deleted")).unwrap(), "delete me\n");
        assert_eq!(fs::read_to_string(target.join("transition")).unwrap(), "regular\n");
        assert_eq!(fs::read_to_string(target.join("directory-transition")).unwrap(), "regular\n");
        assert_eq!(fs::metadata(target.join("mode")).unwrap().mode() & 0o111, 0);
        assert_eq!(fs::read_link(target.join("link")).unwrap(), Path::new("shared"));
        assert_eq!(fs::read_to_string(target.join("line\nbreak")).unwrap(), "newline path\n");
        assert!(!target.join("renamed\nfile").exists());
        assert!(!target.join("added").exists());
        assert_ne!(identity(&source.join("shared")), identity(&target.join("shared")));
        assert_shared_extents(&source.join("shared"), &target.join("shared"));
    }
}

#[test]
fn symlink_bytes_and_patch_like_names_survive_sync() {
    let Some(mut repo) = Repo::new() else { return };
    let cases = [
        ("embedded-newline", "one\ntwo"),
        ("trailing-newline", "tail\n"),
        ("trailing-spaces", "tail  "),
        ("patch-prefix", "+diff --git a/x b/y\n+@@ -0,0 +1,1 @@\n"),
        ("symlink\n\"quote\" space", "quoted target\n"),
        ("header\n@@ -0,0 +1,1 @@\n", "hunk-looking\n target \n"),
    ];
    for (name, target_bytes) in cases {
        // Unchanged paths exercise source reads/cloning; sibling feature paths
        // exercise frozen target reads/materialization.
        symlink(target_bytes, repo.root.join(format!("unchanged-{name}"))).unwrap();
        symlink("shared", repo.root.join(name)).unwrap();
    }
    repo.baseline = repo.id(&repo.root, "@");
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    for (name, target_bytes) in cases {
        fs::remove_file(target.join(name)).unwrap();
        symlink(target_bytes, target.join(name)).unwrap();
    }
    let root = identity(&target);
    let jj = identity(&target.join(".jj"));
    success(repo.reseed(&["feature"], &source, &[]));
    assert_eq!(repo.id(&target, "@-"), frozen_id(&repo, &source));
    for (name, target_bytes) in cases {
        // The donor's link payload wins; equal paths arrive as reflinked copies.
        assert_eq!(fs::read_link(target.join(name)).unwrap(), Path::new("shared"));
        assert_eq!(fs::read_link(target.join(format!("unchanged-{name}"))).unwrap(), Path::new(target_bytes));
    }
    assert_eq!(identity(&target), root);
    assert_eq!(identity(&target.join(".jj")), jj);
    assert!(repo.pending_records().is_empty());
}

#[test]
fn reset_batch_across_two_sources_keeps_roots_bookmarks_and_lifecycle() {
    let Some(mut repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    success(repo.jw(&repo.root).args(["add", "worker-a", "worker-b"]).output().unwrap());
    let a = repo.root.with_extension("worker-a");
    let b = repo.root.with_extension("worker-b");
    if !repo.cow(&source, &a) || !repo.cow(&source, &b) {
        return;
    }
    fs::write(a.join("old-only"), "old\n").unwrap();
    repo.id(&a, "@");
    repo.jj_at(&a, &["bookmark", "create", "feature-bookmark"]);
    let lifecycle = repo.lifecycle();
    assert!(!lifecycle.is_empty(), "fixture must exercise lifecycle metadata");
    let identities = [identity(&a), identity(&a.join(".jj")), identity(&b), identity(&b.join(".jj"))];
    for pass in 0..2 {
        if pass == 1 {
            repo.jj_at(&source, &["new"]);
            fs::write(source.join("modified"), "new baseline\n").unwrap();
            fs::write(source.join("new-only"), "new\n").unwrap();
            repo.baseline = repo.id(&source, "@");
            // The new prepared base is intentional named work too; an arbitrary
            // donor snapshot is not a substitute for the bookmark guard.
            repo.jj_at(&source, &["bookmark", "create", "prepared-baseline", "-r", &repo.baseline]);
        }
        let bookmarks = repo.jj_at(&repo.root, &["bookmark", "list", "-T", "name ++ normal_target.commit_id()"]);
        success(repo.reseed(&["worker-a", "worker-b"], &source, &["--at", &repo.baseline]));
        let ids = [repo.id(&a, "@"), repo.id(&b, "@")];
        success(repo.reseed(&["worker-a", "worker-b"], &source, &["--at", &repo.baseline]));
        assert_ne!([repo.id(&a, "@"), repo.id(&b, "@")], ids, "explicit --at starts fresh children");
        for target in [&a, &b] {
            assert_eq!(repo.id(target, "@-"), repo.baseline);
            assert!(!target.join("old-only").exists());
            assert_eq!(fs::read(target.join("modified")).unwrap(), fs::read(source.join("modified")).unwrap());
            assert_shared_extents(&source.join("shared"), &target.join("shared"));
        }
        assert_eq!([identity(&a), identity(&a.join(".jj")), identity(&b), identity(&b.join(".jj"))], identities);
        assert_eq!(repo.lifecycle(), lifecycle);
        assert_eq!(
            repo.jj_at(&repo.root, &["bookmark", "list", "-T", "name ++ normal_target.commit_id()"]),
            bookmarks
        );
    }
}

#[test]
fn existing_workspace_can_validate_donor_tree_state_then_restore_a_different_desired_tree() {
    // Protocol evidence only: production reseed still uses sparse expansion.
    // Adoption describes copied S, not desired D; checkout/name is never copied.
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    repo.jj_at(&repo.root, &["new", &repo.baseline]);
    fs::write(repo.root.join("new-desired"), "desired only\n").unwrap();
    let desired = repo.id(&repo.root, "@");
    fs::write(source.join("modified"), "donor version\n").unwrap();
    fs::remove_file(source.join("deleted")).unwrap();
    fs::remove_file(source.join("directory-transition")).unwrap();
    fs::create_dir(source.join("directory-transition")).unwrap();
    fs::write(source.join("directory-transition/child"), "donor directory\n").unwrap();
    fs::set_permissions(source.join("mode"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(source.join("donor-only"), "remove on convergence\n").unwrap();
    let donor = repo.id(&source, "@");
    assert_ne!(donor, desired);
    let leaves = repo
        .jj_at(&source, &["--ignore-working-copy", "file", "list", "-T", r#"json(path) ++ "\n""#])
        .lines()
        .map(|line| PathBuf::from(serde_json::from_str::<String>(line).unwrap()))
        .collect::<Vec<_>>();
    fs::write(target.join("old-target-only"), "tracked target work\n").unwrap();
    let old = repo.id(&target, "@");
    repo.jj_at(&target, &["bookmark", "create", "saved-before-adoption", "-r", &old]);
    repo.jj_at(&target, &["config", "set", "--repo", "snapshot.auto-track", "none()"]);
    fs::write(target.join(".env"), "ignored local\n").unwrap();
    fs::write(target.join("unignored-local"), "unignored local\n").unwrap();
    fs::write(target.join(".jj/user-marker"), "target native metadata\n").unwrap();
    let root_identity = identity(&target);
    let jj_identity = identity(&target.join(".jj"));
    let repo_pointer = fs::read(target.join(".jj/repo")).unwrap();

    repo.jj_at(&target, &["--config=snapshot.auto-update-stale=false", "sparse", "set", "--clear"]);
    repo.jj_at(&target, &["--config=snapshot.auto-update-stale=false", "new", &desired]);
    let checkout = fs::read(target.join(".jj/working_copy/checkout")).unwrap();
    diff_dirs_sync::seed_leaves(&source, &target, &leaves, diff_dirs_sync::CopyPolicy::ReflinkRequired, diff_dirs::NoCheck).unwrap();
    let cloned_inode = identity(&target.join("shared"));
    jj_waltz::jj::copy_tree_state(&source, &target).unwrap();
    assert_eq!(fs::read(target.join(".jj/working_copy/checkout")).unwrap(), checkout);
    repo.jj_at(
        &target,
        &["--config=fsmonitor.backend=none", "--config=snapshot.auto-update-stale=false", "status"],
    );
    assert_eq!(repo.jj_at(&target, &["diff", "--from", &donor, "--to", "@", "--summary"]), "");
    // The native convergence diff is S -> D, not an empty-tree -> D refill.
    repo.jj_at(&target, &["--config=snapshot.auto-update-stale=false", "restore", "--from", &desired]);
    assert_eq!(repo.jj_at(&target, &["diff", "--from", &desired, "--to", "@", "--summary"]), "");
    assert_eq!(repo.id(&target, "@-"), desired);
    assert_eq!(repo.id(&target, "@"), repo.id(&repo.root, "feature@"));
    assert_eq!(repo.jj_at(&target, &["sparse", "list"]), ".");
    assert_eq!(fs::read(target.join("modified")).unwrap(), b"baseline\n");
    assert_eq!(fs::read(target.join("deleted")).unwrap(), b"delete me\n");
    assert_eq!(fs::read(target.join("directory-transition")).unwrap(), b"regular\n");
    assert_eq!(fs::read(target.join("new-desired")).unwrap(), b"desired only\n");
    assert_eq!(fs::metadata(target.join("mode")).unwrap().permissions().mode() & 0o111, 0);
    for absent in ["donor-only", "old-target-only"] {
        assert!(!target.join(absent).exists(), "{absent}");
    }
    assert_eq!(fs::read(target.join(".env")).unwrap(), b"ignored local\n");
    assert_eq!(fs::read(target.join("unignored-local")).unwrap(), b"unignored local\n");
    assert_eq!(fs::read(target.join(".jj/user-marker")).unwrap(), b"target native metadata\n");
    assert_eq!(fs::read(target.join(".jj/repo")).unwrap(), repo_pointer);
    assert_eq!(identity(&target), root_identity);
    assert_eq!(identity(&target.join(".jj")), jj_identity);
    assert_eq!(identity(&target.join("shared")), cloned_inode);
    assert_eq!(repo.id(&target, "saved-before-adoption"), old);
}

#[test]
fn visitor_preview_blocks_an_unrelated_routing_leaf_without_claiming_ready() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    repo.jj_at(&target, &["config", "set", "--repo", "snapshot.auto-track", "none()"]);
    fs::create_dir_all(source.join("packages/a/cache")).unwrap();
    fs::write(source.join("packages/a/cache/file"), "source\n").unwrap();
    fs::write(target.join("packages"), "target-owned leaf\n").unwrap();
    let before = repo.id(&target, "@");
    let output = repo.reseed(&["feature"], &source, &["--copy-ignored", "packages/a/cache", "--dry-run", "--format=json"]);
    assert!(!output.status.success());
    assert!(!output.stdout.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(!text.contains("READY"), "{text}");
    assert!(
        text.contains("real directory") || text.contains("non-directory") || text.contains("symlink"),
        "{text}"
    );
    assert_eq!(repo.id(&target, "@"), before);
    assert_eq!(fs::read(target.join("packages")).unwrap(), b"target-owned leaf\n");
    assert_eq!(repo.jj_at(&target, &["sparse", "list"]), ".");
}

#[test]
fn ignored_refresh_is_private_retains_unselected_content_and_replace_is_explicit() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    for root in [&source, &target] {
        fs::create_dir(root.join("cache")).unwrap();
    }
    fs::write(source.join("cache/fresh"), vec![b'c'; 128 * 1024]).unwrap();
    symlink("fresh", source.join("cache/link")).unwrap();
    fs::write(target.join("cache/obsolete"), "old").unwrap();
    fs::write(target.join(".env"), "secret").unwrap();
    let root_id = identity(&target);
    success(repo.reseed(&["feature"], &source, &["--copy-ignored", "cache"]));
    assert_eq!(repo.id(&target, "@-"), frozen_id(&repo, &source));
    assert!(!target.join("cache/obsolete").exists());
    assert_eq!(fs::read_to_string(target.join(".env")).unwrap(), "secret");
    assert_eq!(fs::read_link(target.join("cache/link")).unwrap(), Path::new("fresh"));
    assert_ne!(identity(&source.join("cache/fresh")), identity(&target.join("cache/fresh")));
    assert_shared_extents(&source.join("cache/fresh"), &target.join("cache/fresh"));
    fs::write(target.join("cache/fresh"), "target edit").unwrap();
    assert_eq!(fs::read(source.join("cache/fresh")).unwrap(), vec![b'c'; 128 * 1024]);
    fs::write(source.join("cache/fresh"), "source edit").unwrap();
    assert_eq!(fs::read_to_string(target.join("cache/fresh")).unwrap(), "target edit");
    success(repo.reseed(&["feature"], &source, &["--copy-ignored", "cache", "--replace-content"]));
    assert!(!target.join(".env").exists());
    assert_eq!(identity(&target), root_id);
    assert_eq!(fs::read_to_string(target.join("cache/fresh")).unwrap(), "source edit");
}

#[test]
fn selection_is_explicit_and_primary_source_and_unknown_targets_are_rejected() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    let before = repo.id(&target, "@");
    for names in [&[][..], &["default"][..], &["source"][..], &["missing"][..], &["feature", "missing"][..]] {
        repo.unchanged_failure(&target, repo.reseed(names, &source, &[]), &before);
    }
}

#[test]
fn dirty_and_sparse_source_fail_but_nonignored_source_matches_are_skipped() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    let before = repo.id(&target, "@");
    fs::write(source.join("modified"), "dirty\n").unwrap();
    repo.unchanged_failure(&target, repo.reseed(&["feature"], &source, &[]), &before);
    repo.jj_at(&source, &["restore", "--from", &repo.baseline]);
    repo.jj_at(&source, &["config", "set", "--repo", "snapshot.auto-track", "none()"]);
    fs::create_dir(source.join("not-ignored")).unwrap();
    fs::write(source.join("not-ignored/file"), "payload").unwrap();
    // Keep this untracked so source-at still matches; the payload is not ignored.
    let output = success(repo.reseed(&["feature"], &source, &["--copy-ignored", "not-ignored"]));
    assert!(output.contains("not ignored under frozen source rules"), "{output}");
    assert!(!target.join("not-ignored").exists());
    assert_eq!(repo.id(&target, "@-"), frozen_id(&repo, &source));
    fs::remove_dir_all(source.join("not-ignored")).unwrap();
    repo.jj_at(&source, &["sparse", "set", "--clear"]);
    repo.unchanged_failure(&target, repo.reseed(&["feature"], &source, &[]), &repo.id(&target, "@"));
}

#[test]
fn global_ignored_policy_skips_tracked_build_and_selects_eligible_descendants_once() {
    let Some(mut repo) = Repo::new() else {
        return;
    };
    fs::create_dir(repo.root.join("build")).unwrap();
    fs::write(repo.root.join("build/tracked"), "normal tracked build tree").unwrap();
    repo.baseline = repo.id(&repo.root, "@");
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    fs::create_dir_all(repo.config.join("jj-waltz")).unwrap();
    fs::write(
        repo.config.join("jj-waltz/config.toml"),
        "copy_ignored=['**/target', '**/node_modules', '**/dist/', '**/build/']\n",
    )
    .unwrap();
    for root in [
        "build/packages/a/node_modules",
        "packages/a/node_modules",
        "packages/a/dist",
        "packages/a/target",
    ] {
        fs::create_dir_all(source.join(root)).unwrap();
        fs::write(source.join(root).join("data"), vec![b'p'; 128 * 1024]).unwrap();
    }
    fs::create_dir_all(source.join("packages/a/node_modules/nested/target")).unwrap();
    fs::write(source.join("packages/a/node_modules/nested/target/data"), "nested payload").unwrap();
    fs::create_dir(target.join("cache")).unwrap();
    fs::write(target.join("cache/local-only"), "retain unselected data").unwrap();
    let output = repo.reseed(&["feature"], &source, &[]);
    assert!(String::from_utf8_lossy(&output.stderr).contains("Ignored-copy policy Global"));
    let output = success(output);
    assert!(
        output.contains("ignored roots [\"build/packages/a/node_modules\", \"packages/a/dist\", \"packages/a/node_modules\", \"packages/a/target\"]"),
        "{output}"
    );
    assert!(output.contains("not ignored under frozen source rules"), "{output}");
    assert_eq!(fs::read_to_string(target.join("build/tracked")).unwrap(), "normal tracked build tree");
    assert_eq!(fs::read_to_string(target.join("cache/local-only")).unwrap(), "retain unselected data");
    assert_eq!(
        fs::read_to_string(target.join("packages/a/node_modules/nested/target/data")).unwrap(),
        "nested payload"
    );
    let payload = Path::new("build/packages/a/node_modules/data");
    assert_ne!(identity(&source.join(payload)), identity(&target.join(payload)));
    assert_shared_extents(&source.join(payload), &target.join(payload));
    fs::write(target.join(payload), "private write").unwrap();
    assert_eq!(fs::read(source.join(payload)).unwrap(), vec![b'p'; 128 * 1024]);
}

#[test]
fn ignored_policy_precedence_preserves_explicit_empty_and_never_prunes_unselected_data() {
    let Some(repo) = Repo::new() else {
        return;
    };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    fs::create_dir_all(repo.config.join("jj-waltz")).unwrap();
    let global = repo.config.join("jj-waltz/config.toml");
    fs::write(&global, "copy_ignored=['cache/']\n").unwrap();
    for root in ["cache", "packages/a/node_modules"] {
        fs::create_dir_all(source.join(root)).unwrap();
        fs::write(source.join(root).join("data"), "source payload").unwrap();
    }
    let local = repo.root.join(".jwreseed.local.toml");
    fs::write(&local, "copy_ignored=[]\n").unwrap();
    let output = repo.reseed(&["feature"], &source, &[]);
    assert!(String::from_utf8_lossy(&output.stderr).contains("Ignored-copy policy Local: []"));
    assert!(success(output).contains("ignored roots []"));
    assert!(!target.join("cache").exists());
    fs::write(&local, "copy_ignored=['cache/']\n").unwrap();
    let output = repo.reseed(&["feature"], &source, &["--copy-ignored", "packages/*/node_modules/"]);
    assert!(String::from_utf8_lossy(&output.stderr).contains("Ignored-copy policy Cli"));
    success(output);
    assert!(!target.join("cache").exists());
    assert_eq!(fs::read_to_string(target.join("packages/a/node_modules/data")).unwrap(), "source payload");
    fs::write(target.join("packages/a/node_modules/data"), "keep private data").unwrap();
    success(repo.reseed(&["feature"], &source, &["--copy-ignored", "cache/", "--no-copy-ignored"]));
    assert!(!target.join("cache").exists());
    fs::write(&local, "").unwrap(); // Missing local selection inherits global.
    success(repo.reseed(&["feature"], &source, &[]));
    assert_eq!(fs::read_to_string(target.join("cache/data")).unwrap(), "source payload");
    assert_eq!(fs::read_to_string(target.join("packages/a/node_modules/data")).unwrap(), "keep private data");
    fs::write(&global, "copy_ignored=['missing/output/']\n").unwrap();
    let output = success(repo.reseed(&["feature"], &source, &[]));
    assert!(
        output.contains("ignored roots []") && output.contains("matched no source directories"),
        "{output}"
    );
    assert_eq!(fs::read_to_string(target.join("cache/data")).unwrap(), "source payload");
}

#[test]
fn donor_forced_tracked_payload_blocks_and_target_tracked_payload_is_replaced() {
    for role in ["source", "target"] {
        let Some(mut repo) = Repo::new() else {
            return;
        };
        if role == "source" {
            fs::write(repo.root.join(".gitignore"), ".env\n.jwreseed.local.toml\npackages/\n").unwrap();
            fs::create_dir(repo.root.join("cache")).unwrap();
            fs::write(repo.root.join("cache/forced-tracked"), "tracked before ignoring").unwrap();
            repo.id(&repo.root, "@");
            fs::write(repo.root.join(".gitignore"), "cache/\n.env\n.jwreseed.local.toml\npackages/\n").unwrap();
            repo.baseline = repo.id(&repo.root, "@");
        }
        let source = repo.workspace("source");
        let target = repo.workspace("feature");
        if role == "target" {
            fs::create_dir(source.join("cache")).unwrap();
            fs::write(source.join("cache/data"), "ignored source data").unwrap();
            fs::write(target.join(".gitignore"), ".env\n.jwreseed.local.toml\npackages/\n").unwrap();
            fs::create_dir(target.join("cache")).unwrap();
            fs::write(target.join("cache/forced-tracked"), "feature tracked data").unwrap();
            repo.id(&target, "@");
            fs::write(target.join(".gitignore"), "cache/\n.env\n.jwreseed.local.toml\npackages/\n").unwrap();
        }
        let before = repo.id(&target, "@");
        let output = repo.reseed(&["feature"], &source, &["--copy-ignored", "cache/"]);
        if role == "source" {
            // The donor's own tracked files inside a selected ignored root are
            // still a hard error: the payload and the desired tree conflict.
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("overlaps tracked"),
                "{role}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            repo.unchanged_failure(&target, output, &before);
            assert!(target.join("cache/forced-tracked").exists());
        } else {
            // Target-tracked content inside the root is ordinary old tracked
            // work: the sync deletes it and installs the donor's payload.
            success(output);
            assert_eq!(repo.id(&target, "@-"), frozen_id(&repo, &source));
            assert!(!target.join("cache/forced-tracked").exists());
            assert_eq!(fs::read_to_string(target.join("cache/data")).unwrap(), "ignored source data");
        }
    }
}

#[test]
fn copied_payload_rejects_nested_metadata_and_symlink_parent_escape() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    let before = repo.id(&target, "@");
    fs::create_dir_all(source.join("cache/nested/.jj")).unwrap();
    repo.unchanged_failure(&target, repo.reseed(&["feature"], &source, &["--copy-ignored", "cache", "--dry-run"]), &before);
    let output = repo.reseed(&["feature"], &source, &["--copy-ignored", "cache"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("administrative entry"), "{:?}", output);
    // Deep refusal is streaming: tracked refill already advanced JJ, but the
    // offending source remains untouched. Accepted ancestor creation can remain.
    assert_ne!(repo.id(&target, "@"), before);
    assert!(source.join("cache/nested/.jj").is_dir());
    assert!(!target.join("cache/nested/.jj").exists());
    assert_eq!(repo.jj_at(&target, &["sparse", "list"]), ".");
    fs::remove_dir_all(source.join("cache")).unwrap();
    fs::create_dir_all(source.join("packages/pkg/cache")).unwrap();
    fs::write(source.join("packages/pkg/cache/file"), "source").unwrap();
    let outside = repo.temp.path().join("outside");
    fs::create_dir_all(outside.join("pkg/cache")).unwrap();
    fs::write(outside.join("pkg/cache/sentinel"), "untouched").unwrap();
    symlink(&outside, target.join("packages")).unwrap();
    let before = repo.id(&target, "@");
    repo.unchanged_failure(&target, repo.reseed(&["feature"], &source, &["--copy-ignored", "packages/pkg/cache"]), &before);
    assert_eq!(fs::read_to_string(outside.join("pkg/cache/sentinel")).unwrap(), "untouched");
}

#[test]
fn sparse_target_is_rejected_without_changing_patterns_or_checkout() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    repo.jj_at(&target, &["sparse", "set", "--clear", "--add", "modified"]);
    let before = repo.id(&target, "@");
    let patterns = repo.jj_at(&target, &["sparse", "list"]);
    let root = identity(&target);
    let jj = identity(&target.join(".jj"));
    let output = repo.reseed(&["feature"], &source, &[]);
    assert!(!output.status.success(), "sparse targets are outside the initial contract");
    let message = String::from_utf8_lossy(&output.stderr);
    assert!(message.contains("jj new <bookmark>") && message.contains("jj sparse reset"), "{message}");
    assert_eq!(repo.id(&target, "@"), before);
    assert_eq!(repo.jj_at(&target, &["sparse", "list"]), patterns);
    assert_eq!(fs::read_to_string(target.join("modified")).unwrap(), "baseline\n");
    assert!(!target.join("shared").exists());
    assert_eq!(identity(&target), root);
    assert_eq!(identity(&target.join(".jj")), jj);
}

#[test]
fn reset_revision_must_ignore_selected_payload_not_just_source_revision() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    // Only the target revision changes its ignore rules. Source baseline and
    // source on-disk tree remain prepared and ignored-cache compatible.
    fs::write(target.join(".gitignore"), ".env\n.jwreseed.local.toml\npackages/\n").unwrap();
    let before = repo.id(&target, "@");
    fs::create_dir(source.join("cache")).unwrap();
    fs::write(source.join("cache/file"), "must stay ignored").unwrap();
    repo.unchanged_failure(
        &target,
        repo.reseed(&["feature"], &source, &["--at", &before, "--copy-ignored", "cache"]),
        &before,
    );
    assert!(!target.join("cache").exists());
}

#[test]
fn absolute_source_bound_ignored_symlink_is_rejected_or_explicitly_reported() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    fs::create_dir(source.join("cache")).unwrap();
    fs::write(source.join("cache/file"), "payload").unwrap();
    symlink(source.join("cache/file"), source.join("cache/source-bound")).unwrap();
    let before = repo.id(&target, "@");
    let output = repo.reseed(&["feature"], &source, &["--copy-ignored", "cache"]);
    let diagnostics = format!("{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        repo.unchanged_failure(&target, output, &before);
    } else {
        assert_eq!(repo.id(&target, "@-"), frozen_id(&repo, &source));
        assert_eq!(fs::read_link(target.join("cache/source-bound")).unwrap(), source.join("cache/file"));
    }
    assert!(
        diagnostics.to_lowercase().contains("absolute"),
        "diagnostics must explain the absolute-link hazard: {diagnostics}"
    );
    assert!(
        diagnostics.contains("source-bound"),
        "diagnostics must identify the offending link: {diagnostics}"
    );
}

#[test]
fn reset_refuses_to_overwrite_retained_ignored_secret_without_replace_content() {
    let Some(mut repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    fs::write(target.join(".env"), "target secret\n").unwrap();
    let before = repo.id(&target, "@");
    let root = identity(&target);
    let jj = identity(&target.join(".jj"));
    repo.jj_at(&source, &["new"]);
    fs::write(source.join(".gitignore"), "cache/\n.jwreseed.local.toml\npackages/\n").unwrap();
    fs::write(source.join(".env"), "new tracked environment\n").unwrap();
    repo.baseline = repo.id(&source, "@");
    repo.unchanged_failure(&target, repo.reseed(&["feature"], &source, &["--at", &repo.baseline]), &before);
    assert_eq!(fs::read_to_string(target.join(".env")).unwrap(), "target secret\n");
    assert_eq!(identity(&target), root);
    assert_eq!(identity(&target.join(".jj")), jj);
    success(repo.reseed(&["feature"], &source, &["--at", &repo.baseline, "--replace-content"]));
    assert_eq!(fs::read_to_string(target.join(".env")).unwrap(), "new tracked environment\n");
    assert_eq!(repo.id(&target, "@-"), repo.baseline);
    assert_eq!(identity(&target), root);
    assert_eq!(identity(&target.join(".jj")), jj);
}

#[test]
fn reset_refuses_retained_ignored_directory_obstructing_new_tracked_file() {
    let Some(mut repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    fs::write(target.join(".gitignore"), "cache/\n.env\n.jwreseed.local.toml\npackages/\nobstruction/\n").unwrap();
    let before = repo.id(&target, "@");
    fs::create_dir(target.join("obstruction")).unwrap();
    fs::write(target.join("obstruction/private"), "retained untracked content\n").unwrap();
    let root = identity(&target);
    let jj = identity(&target.join(".jj"));
    repo.jj_at(&source, &["new"]);
    fs::write(source.join("obstruction"), "new tracked file\n").unwrap();
    repo.baseline = repo.id(&source, "@");
    repo.unchanged_failure(&target, repo.reseed(&["feature"], &source, &["--at", &repo.baseline]), &before);
    assert_eq!(fs::read_to_string(target.join("obstruction/private")).unwrap(), "retained untracked content\n");
    success(repo.reseed(&["feature"], &source, &["--at", &repo.baseline, "--replace-content"]));
    assert_eq!(fs::read_to_string(target.join("obstruction")).unwrap(), "new tracked file\n");
    assert_eq!(repo.id(&target, "@-"), repo.baseline);
    assert_eq!(identity(&target), root);
    assert_eq!(identity(&target.join(".jj")), jj);
}

#[test]
fn required_private_self_target_is_preflight_rejected_when_replace_will_erase_it() {
    let Some(mut repo) = Repo::new() else { return };
    repo.prepare_links("[[link]]\nsource = \"private\"\ntarget = \"private\"\nrequired = true\n", "private/\n");
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    fs::create_dir(target.join("private")).unwrap();
    fs::write(target.join("private/secret"), "private retained content\n").unwrap();
    let before = repo.id(&target, "@");
    let identities = [identity(&target), identity(&target.join(".jj")), identity(&target.join("private"))];
    let output = repo.reseed(&["feature"], &source, &["--replace-content"]);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("required"),
        "must report required-link future-state failure: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    repo.unchanged_failure(&target, output, &before);
    assert_eq!(fs::read_to_string(target.join("private/secret")).unwrap(), "private retained content\n");
    assert_eq!(
        [identity(&target), identity(&target.join(".jj")), identity(&target.join("private"))],
        identities
    );
}

#[test]
fn required_internal_link_target_is_preflight_rejected_when_replace_will_erase_it() {
    let Some(mut repo) = Repo::new() else { return };
    repo.prepare_links(
        "[[link]]\nsource = \"dependency\"\ntarget = \"private/secret\"\nrequired = true\n",
        "private/\ndependency\n",
    );
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    fs::create_dir(target.join("private")).unwrap();
    fs::write(target.join("private/secret"), "private retained content\n").unwrap();
    let before = repo.id(&target, "@");
    let identities = [identity(&target), identity(&target.join(".jj")), identity(&target.join("private"))];
    let output = repo.reseed(&["feature"], &source, &["--replace-content"]);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("required"),
        "must report required-link future-state failure: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    repo.unchanged_failure(&target, output, &before);
    assert_eq!(fs::read_to_string(target.join("private/secret")).unwrap(), "private retained content\n");
    assert!(!target.join("dependency").exists());
    assert_eq!(
        [identity(&target), identity(&target.join(".jj")), identity(&target.join("private"))],
        identities
    );
}

#[test]
fn required_link_target_in_copied_ignored_root_works_with_and_without_replace() {
    for mode in ["retain", "replace"] {
        let Some(mut repo) = Repo::new() else { return };
        repo.prepare_links(
            "[[link]]\nsource = \"dependency\"\ntarget = \"cache/fresh\"\nrequired = true\n",
            // A directory-only dependency/ rule would NOT ignore the file
            // symlink created at dependency; the non-slash pattern is required.
            "dependency\n",
        );
        let source = repo.workspace("source");
        let target = repo.workspace("feature");
        if !repo.cow(&source, &target) {
            return;
        }
        fs::create_dir(source.join("cache")).unwrap();
        fs::write(source.join("cache/fresh"), vec![b'c'; 128 * 1024]).unwrap();
        fs::write(target.join(".env"), "unselected secret\n").unwrap();
        let root = identity(&target);
        let jj = identity(&target.join(".jj"));
        assert!(!target.join("cache").exists());
        let extra = if mode == "replace" {
            vec!["--copy-ignored", "cache", "--replace-content"]
        } else {
            vec!["--copy-ignored", "cache"]
        };
        success(repo.reseed(&["feature"], &source, &extra));
        assert!(
            fs::symlink_metadata(target.join("dependency")).unwrap().file_type().is_symlink(),
            "{mode}: must create the configured symlink"
        );
        assert_eq!(
            fs::canonicalize(target.join("dependency")).unwrap(),
            fs::canonicalize(target.join("cache/fresh")).unwrap(),
            "{mode}: required target must resolve inside copied payload"
        );
        assert_eq!(fs::read(target.join("dependency")).unwrap(), vec![b'c'; 128 * 1024]);
        assert_eq!(repo.id(&target, "@-"), frozen_id(&repo, &source), "{mode}: link must remain ignored");
        assert_eq!(identity(&target), root);
        assert_eq!(identity(&target.join(".jj")), jj);
        assert_eq!(target.join(".env").exists(), mode != "replace");
        assert_shared_extents(&source.join("cache/fresh"), &target.join("cache/fresh"));
    }
}

#[test]
fn local_groups_deduplicate_and_cli_can_disable_configured_ignored_copy() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    fs::create_dir(source.join("cache")).unwrap();
    fs::write(source.join("cache/file"), "source").unwrap();
    fs::create_dir(target.join("cache")).unwrap();
    fs::write(target.join("cache/file"), "target").unwrap();
    // Incremental comparison promises stat-key convergence, not hashing. Make
    // this deliberately different instead of depending on write-clock resolution.
    fs::File::open(target.join("cache/file"))
        .unwrap()
        .set_modified(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();
    let config = format!(
        "source = {:?}\nsource_at = {:?}\ncopy_ignored = [\"cache\"]\n[groups]\nworkers = [\"feature\", \"feature\"]\n",
        source.to_str().unwrap(),
        repo.baseline
    );
    fs::write(repo.root.join(".jwreseed.local.toml"), config).unwrap();
    // Config is read from the default root even when invoked in the target.
    success(
        repo.jw(&target)
            .args(["reseed", "feature", "--group", "workers", "--no-copy-ignored"])
            .output()
            .unwrap(),
    );
    assert_eq!(fs::read_to_string(target.join("cache/file")).unwrap(), "target");
    assert_eq!(repo.id(&target, "@-"), frozen_id(&repo, &source));
    success(repo.jw(&target).args(["reseed", "--group", "workers"]).output().unwrap());
    assert_eq!(fs::read_to_string(target.join("cache/file")).unwrap(), "source");
}

#[test]
fn dry_run_preserves_content_identity_and_revision() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    let before = repo.id(&target, "@");
    fs::write(target.join(".env"), "keep").unwrap();
    let identities = [identity(&target), identity(&target.join(".jj")), identity(&target.join("shared"))];
    success(repo.reseed(&["feature"], &source, &["--dry-run", "--replace-content", "--at", &repo.baseline]));
    assert_eq!(repo.id(&target, "@"), before);
    assert_eq!([identity(&target), identity(&target.join(".jj")), identity(&target.join("shared"))], identities);
    assert_eq!(fs::read_to_string(target.join(".env")).unwrap(), "keep");
}

#[test]
fn incremental_payload_preserves_clean_inodes_and_reconciles_sparse_changes() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    fs::create_dir_all(source.join("cache/nested")).unwrap();
    fs::write(source.join("cache/clean"), vec![b'c'; 128 * 1024]).unwrap();
    fs::write(source.join("cache/nested/change"), "initial").unwrap();
    symlink("clean", source.join("cache/link")).unwrap();
    success(repo.reseed(&["feature"], &source, &["--copy-ignored", "cache"]));
    let paths = ["cache", "cache/clean", "cache/nested", "cache/nested/change", "cache/link"];
    let observations = || {
        paths
            .iter()
            .map(|p| {
                let m = fs::symlink_metadata(target.join(p)).unwrap();
                (m.ino(), m.mtime(), m.mtime_nsec(), m.ctime(), m.ctime_nsec())
            })
            .collect::<Vec<_>>()
    };
    let before = observations();
    success(repo.reseed(&["feature"], &source, &["--copy-ignored", "cache"]));
    assert_eq!(observations(), before, "clean payload must not be replaced or chmodded");
    fs::write(source.join("cache/nested/change"), "new and longer").unwrap();
    fs::write(source.join("cache/new"), "added").unwrap();
    fs::remove_file(source.join("cache/link")).unwrap();
    fs::create_dir(source.join("cache/link")).unwrap();
    fs::write(source.join("cache/link/child"), "link becomes directory").unwrap();
    fs::create_dir_all(target.join("cache/stale/deep")).unwrap();
    fs::write(target.join("cache/stale/deep/file"), "remove").unwrap();
    success(repo.reseed(&["feature"], &source, &["--copy-ignored", "cache"]));
    assert_eq!(identity(&target.join("cache/clean")).1, before[1].0);
    assert_eq!(fs::read_to_string(target.join("cache/nested/change")).unwrap(), "new and longer");
    assert_eq!(fs::read_to_string(target.join("cache/new")).unwrap(), "added");
    assert_eq!(fs::read_to_string(target.join("cache/link/child")).unwrap(), "link becomes directory");
    assert!(!target.join("cache/stale").exists());
    assert_shared_extents(&source.join("cache/clean"), &target.join("cache/clean"));
}

#[test]
fn incremental_payload_batch_reuses_execution_across_roots_and_targets() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let targets = [repo.workspace("one"), repo.workspace("two")];
    if !repo.cow(&source, &targets[0]) {
        return;
    }
    for path in ["cache/tree", "packages/pkg/cache/tree"] {
        fs::create_dir_all(source.join(path)).unwrap();
        fs::write(source.join(path).join("data"), "initial").unwrap();
    }
    let extra = ["--copy-ignored", "cache", "--copy-ignored", "packages/pkg/cache"];
    success(repo.reseed(&["one", "two"], &source, &extra));
    let clean: Vec<_> = targets.iter().map(|target| identity(&target.join("cache/tree/data"))).collect();
    fs::remove_dir_all(source.join("packages/pkg/cache/tree")).unwrap();
    symlink("../relative", source.join("packages/pkg/cache/tree")).unwrap();
    success(repo.reseed(&["one", "two"], &source, &extra));
    for (target, clean) in targets.iter().zip(clean) {
        assert_eq!(identity(&target.join("cache/tree/data")), clean);
        assert_eq!(fs::read_link(target.join("packages/pkg/cache/tree")).unwrap(), Path::new("../relative"));
    }
}

#[test]
fn streaming_guard_covers_equal_and_one_sided_payload_interiors() {
    for kind in ["source-only", "target-only", "equal", "fifo"] {
        let Some(repo) = Repo::new() else { return };
        let source = repo.workspace("source");
        let target = repo.workspace("feature");
        if !repo.cow(&source, &target) {
            return;
        }
        fs::create_dir_all(source.join("cache")).unwrap();
        fs::create_dir_all(target.join("cache")).unwrap();
        let mut offenders = Vec::new();
        if kind == "source-only" || kind == "equal" {
            offenders.push(source.join("cache/opaque/.git"));
        }
        if kind == "target-only" || kind == "equal" {
            offenders.push(target.join("cache/opaque/.git"));
        }
        for path in &offenders {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "administration must remain untouched").unwrap();
        }
        if kind == "fifo" {
            success(Command::new("mkfifo").arg(target.join("cache/fifo")).output().unwrap());
        }
        let before = repo.id(&target, "@");
        let preview = repo.reseed(&["feature"], &source, &["--copy-ignored", "cache", "--dry-run"]);
        assert!(!preview.status.success(), "{kind}");
        assert!(!String::from_utf8_lossy(&preview.stdout).contains("READY;"), "{kind}");
        assert_eq!(repo.id(&target, "@"), before);
        let output = repo.reseed(&["feature"], &source, &["--copy-ignored", "cache"]);
        assert!(!output.status.success(), "{kind}");
        for path in &offenders {
            assert_eq!(fs::read_to_string(path).unwrap(), "administration must remain untouched");
        }
        if kind == "fifo" {
            use std::os::unix::fs::FileTypeExt;
            assert!(fs::symlink_metadata(target.join("cache/fifo")).unwrap().file_type().is_fifo());
        }
    }
}

#[test]
fn incremental_payload_replaces_selected_leaf_roots_without_following_links() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    if !repo.cow(&source, &repo.root) {
        return;
    }
    fs::create_dir(source.join("cache")).unwrap();
    fs::write(source.join("cache/data"), "payload").unwrap();
    let outside = repo.temp.path().join("outside-leaf");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("sentinel"), "untouched").unwrap();
    for kind in ["absent", "file", "link"] {
        let target = repo.workspace(kind);
        if kind == "file" {
            fs::write(target.join("cache"), "old root").unwrap();
        }
        if kind == "link" {
            symlink(&outside, target.join("cache")).unwrap();
        }
        // cache/ only ignores directories; use an explicit all-kind rule to
        // keep existing selected file/link roots outside target snapshots.
        repo.jj_at(&target, &["config", "set", "--repo", "snapshot.auto-track", "none()"]);
        success(repo.reseed(&[kind], &source, &["--copy-ignored", "cache"]));
        assert!(fs::symlink_metadata(target.join("cache")).unwrap().is_dir());
        assert_eq!(fs::read_to_string(target.join("cache/data")).unwrap(), "payload");
        assert_eq!(fs::read_to_string(outside.join("sentinel")).unwrap(), "untouched");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn reseed_front_guard_rejects_real_directory_and_file_bind_mounts() {
    const CHILD: &str = "JW_MOUNT_GUARD_TEST_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let probe = Command::new("unshare").args(["--user", "--map-root-user", "--mount", "true"]).output().unwrap();
        if !probe.status.success() {
            assert!(std::env::var_os("JW_TEST_REQUIRE_MOUNTS").is_none(), "mount namespace required: {:?}", probe);
            eprintln!("skipping mount namespace unavailable");
            return;
        }
        let output = Command::new("unshare")
            .args(["--user", "--map-root-user", "--mount", "--propagation", "private"])
            .arg(std::env::current_exe().unwrap())
            .args(["--exact", "reseed_front_guard_rejects_real_directory_and_file_bind_mounts", "--nocapture"])
            .env(CHILD, "1")
            .output()
            .unwrap();
        success(output);
        return;
    }
    for (side, directory, replace) in [
        ("source", true, false),
        ("target", true, false),
        ("target", false, false),
        ("target", false, true),
    ] {
        let Some(repo) = Repo::new() else { return };
        let source = repo.workspace("source");
        let target = repo.workspace("feature");
        let donor = repo.temp.path().join("mount-donor");
        let operated = if side == "source" { &source } else { &target };
        fs::create_dir(source.join("cache")).unwrap();
        if directory {
            fs::create_dir(&donor).unwrap();
            fs::write(donor.join("sentinel"), "untouched").unwrap();
            fs::create_dir_all(operated.join("cache")).unwrap();
        } else {
            fs::write(&donor, "untouched").unwrap();
            fs::create_dir_all(operated.join("cache")).unwrap();
            fs::write(operated.join("cache/mounted-file"), "covered").unwrap();
        }
        let mountpoint = operated.join(if directory { "cache" } else { "cache/mounted-file" });
        success(Command::new("mount").arg("--bind").arg(&donor).arg(&mountpoint).output().unwrap());
        let before = repo.id(&target, "@");
        let extra = if replace {
            vec!["--copy-ignored", "cache", "--replace-content"]
        } else {
            vec!["--copy-ignored", "cache"]
        };
        let output = repo.reseed(&["feature"], &source, &extra);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("mounted filesystem boundary"), "{:?}", output);
        assert_eq!(repo.id(&target, "@"), before);
        assert_eq!(
            fs::read_to_string(if directory { donor.join("sentinel") } else { donor.clone() }).unwrap(),
            "untouched"
        );
        success(Command::new("umount").arg(&mountpoint).output().unwrap());
    }
}

#[test]
fn special_workspace_names_are_literal_not_switch_tokens() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("^");
    if !repo.cow(&source, &target) {
        return;
    }
    success(repo.reseed(&["^"], &source, &[]));
    assert_eq!(repo.id(&target, "@-"), frozen_id(&repo, &source));
    assert_shared_extents(&source.join("shared"), &target.join("shared"));
}

#[test]
fn colocated_source_is_supported_but_colocated_primary_target_is_not() {
    // Init's --colocate works across the supported JJ matrix; the newer
    // `git colocation enable` conversion command is not required here.
    let Some(repo) = Repo::new_with_colocation(true) else {
        return;
    };
    assert!(repo.root.join(".git").exists());
    let target = repo.workspace("feature");
    if !repo.cow(&repo.root, &target) {
        return;
    }
    success(repo.reseed(&["feature"], &repo.root, &[]));
    assert!(!target.join(".git").exists());
    let before = repo.id(&target, "@");
    repo.unchanged_failure(&target, repo.reseed(&["default"], &repo.root, &[]), &before);
}

#[test]
fn interrupted_refill_requires_out_of_band_bookmark_recovery() {
    for boundary in ["clear", "new"] {
        let Some(repo) = Repo::new() else { return };
        let source = repo.workspace("source");
        let target = repo.workspace("feature");
        if !repo.cow(&source, &target) {
            return;
        }
        fs::write(target.join("modified"), "feature\n").unwrap();
        let before = repo.id(&target, "@");
        repo.jj_at(&target, &["bookmark", "create", "feature-bookmark"]);
        let bookmarks = repo.jj_at(
            &repo.root,
            &["--ignore-working-copy", "bookmark", "list", "-T", "name ++ normal_target.commit_id()"],
        );
        let root = identity(&target);
        let jj = identity(&target.join(".jj"));
        repo.interrupt_reseed(&source, boundary, &["--at", &repo.baseline]);
        let child = repo.jj_at(&target, &["--ignore-working-copy", "log", "--no-graph", "-r", "@", "-T", "commit_id"]);
        if boundary == "new" {
            assert_ne!(child, before, "fault must follow a successful fresh jj new");
        }
        assert_eq!(repo.jj_at(&target, &["--ignore-working-copy", "sparse", "list"]), "");
        // Material left behind by an interrupted writer must not become history.
        fs::write(target.join("shared"), vec![b's'; 128 * 1024]).unwrap();
        let output = repo.reseed(&["feature"], &source, &[]);
        assert!(!output.status.success());
        let message = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        assert!(message.contains("jj new <bookmark>") && message.contains("jj sparse reset"), "{message}");
        assert_eq!(frozen_id(&repo, &target), child, "refusal must not snapshot partial bytes");
        assert_eq!(fs::read(target.join("shared")).unwrap(), vec![b's'; 128 * 1024]);
        // The operator, not jw, restores a complete checkout before retrying.
        repo.jj_at(&target, &["new", "feature-bookmark"]);
        repo.jj_at(&target, &["sparse", "reset"]);
        success(repo.reseed(&["feature"], &source, &[]));
        assert_eq!(fs::read_to_string(target.join("modified")).unwrap(), "baseline\n");
        assert_eq!(
            repo.jj_at(
                &repo.root,
                &["--ignore-working-copy", "bookmark", "list", "-T", "name ++ normal_target.commit_id()"]
            ),
            bookmarks
        );
        assert_eq!(identity(&target), root);
        assert_eq!(identity(&target.join(".jj")), jj);
        assert!(repo.pending_records().is_empty());
        assert_shared_extents(&source.join("shared"), &target.join("shared"));
    }
}

// FIEMAP verifies shared physical extents, not merely equal bytes or distinct
// inodes. Linux is the currently supported native CoW acceptance environment.
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
        let result = unsafe { libc::ioctl(file.as_raw_fd(), 0xc020660b as libc::c_ulong, &mut map) };
        assert_eq!(result, 0, "FIEMAP {path:?}: {}", std::io::Error::last_os_error());
        assert_eq!(map.mapped, 1, "expected an extent for {path:?}");
        map.extent
    }
    let source = extent(source);
    let target = extent(target);
    assert_eq!(source.physical, target.physical, "reseed must actually share extents");
    assert_ne!(source.flags & 0x2000, 0, "source extent must be shared: {source:?}");
    assert_ne!(target.flags & 0x2000, 0, "target extent must be shared: {target:?}");
}

#[cfg(not(target_os = "linux"))]
fn assert_shared_extents(_source: &Path, _target: &Path) {}
