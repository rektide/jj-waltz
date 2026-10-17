//! Bookmark policy uses real native JJ refs/ancestry and disposable CoW workspaces.
#![cfg(unix)]
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Fixture {
    _temp: tempfile::TempDir,
    source: PathBuf,
    a: PathBuf,
    b: PathBuf,
    config: PathBuf,
    base: String,
}

fn ok(output: Output) -> String {
    assert!(output.status.success(), "{}", text(&output));
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn text(output: &Output) -> String {
    format!("{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr))
}

impl Fixture {
    fn new() -> Option<Self> {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join(".test-agent/bookmark-tests");
        fs::create_dir_all(&scratch).unwrap();
        let temp = tempfile::tempdir_in(scratch).unwrap();
        let mut f = Self {
            source: temp.path().join("source"),
            a: temp.path().join("a"),
            b: temp.path().join("b"),
            config: temp.path().join("config"),
            base: String::new(),
            _temp: temp,
        };
        fs::create_dir(&f.config).unwrap();
        f.jj(f._temp.path(), &["git", "init", "--no-colocate", f.source.to_str().unwrap()]);
        fs::write(f.source.join("file"), b"baseline\n").unwrap();
        fs::write(f.source.join(".gitignore"), b"cache/\n.env\n.jwreseed.local.toml\n").unwrap();
        f.jj(&f.source, &["status"]);
        f.base = f.id(&f.source, "@");
        f.workspace("a", &f.a, &f.base);
        f.workspace("b", &f.b, &f.base);
        let probe = f._temp.path().join("cow-probe");
        if let Err(error) = reflink_copy::reflink(f.source.join("file"), &probe) {
            assert!(std::env::var_os("JW_TEST_REQUIRE_COW").is_none(), "real CoW required: {error}");
            return None;
        }
        Some(f)
    }
    fn command(&self, program: &str, root: &Path) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(root)
            .env("JJ_CONFIG", "")
            .env("XDG_CONFIG_HOME", &self.config)
            .env("JJ_USER", "Bookmark Guard Test")
            .env("JJ_EMAIL", "bookmark@example.invalid")
            .env("JJ_EDITOR", "true");
        command
    }
    fn jj(&self, root: &Path, args: &[&str]) -> String {
        ok(self.command("jj", root).args(args).output().unwrap())
    }
    fn id(&self, root: &Path, rev: &str) -> String {
        self.jj(root, &["--ignore-working-copy", "log", "--no-graph", "-r", rev, "-T", "commit_id"])
    }
    fn workspace(&self, name: &str, root: &Path, rev: &str) {
        self.jj(&self.source, &["workspace", "add", "--name", name, "-r", rev, root.to_str().unwrap()]);
    }
    fn bookmark(&self, root: &Path, name: &str, rev: &str) {
        self.jj(root, &["--ignore-working-copy", "bookmark", "create", "-r", rev, "--", name]);
    }
    fn reseed(&self, names: &[&str], args: &[&str]) -> Output {
        self.command(assert_cmd::cargo::cargo_bin!("jw").to_str().unwrap(), &self.source)
            .arg("reseed")
            .args(names)
            .args(["--source-at", &self.base])
            .args(args)
            .output()
            .unwrap()
    }
    fn records(&self, suffix: &str) -> Vec<serde_json::Value> {
        let config = PathBuf::from(self.jj(&self.source, &["--ignore-working-copy", "config", "path", "--repo"]));
        fs::read_dir(config.parent().unwrap().join("jj-waltz/reseed"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.file_name().unwrap().to_string_lossy().ends_with(suffix))
            .map(|path| serde_json::from_slice(&fs::read(path).unwrap()).unwrap())
            .collect()
    }
}

#[test]
fn default_guard_blocks_unbookmarked_work_without_refilling_a_valid_peer() {
    let Some(f) = Fixture::new() else { return };
    f.bookmark(&f.source, "old-base", &f.base);
    fs::write(f.a.join("file"), b"unfinished work\n").unwrap();
    let peer_inode = fs::metadata(f.b.join("file")).unwrap().ino();
    for overridden in [false, true] {
        for dry_run in [true, false] {
            let mut args = Vec::new();
            if overridden {
                args.extend(["--at", f.base.as_str()]);
            }
            if dry_run {
                args.push("--dry-run");
            }
            let output = f.reseed(&["b", "a"], &args);
            assert!(!output.status.success(), "{}", text(&output));
            let text = text(&output);
            assert!(text.contains("unbookmarked work tip"), "{text}");
            assert!(text.contains("--allow-unbookmarked"), "{text}");
            if dry_run {
                assert!(text.contains("BLOCKED a:"), "{text}");
                assert!(text.contains("PLAN b: READY"), "{text}");
                assert!(text.contains("old-base"), "{text}");
            }
            assert_eq!(fs::read(f.a.join("file")).unwrap(), b"unfinished work\n");
            assert_eq!(fs::metadata(f.b.join("file")).unwrap().ino(), peer_inode);
            assert_eq!(f.jj(&f.b, &["--ignore-working-copy", "sparse", "list"]), ".");
            assert!(f.records(".pending.json").is_empty());
        }
    }
}

#[test]
fn existing_named_change_follows_normal_dirty_snapshots_and_guard_never_creates_bookmarks() {
    let Some(f) = Fixture::new() else { return };
    f.bookmark(&f.a, "feature", "@");
    fs::write(f.a.join("file"), b"named unfinished work\n").unwrap();
    let preview = ok(f.reseed(&["a"], &["--dry-run"]));
    assert!(preview.contains("bookmark \"feature\" covers work tip"), "{preview}");
    let preserved = f.id(&f.a, "@");
    assert_eq!(f.id(&f.a, "feature"), preserved);
    // The plain default syncs the workspace onto the donor's current commit;
    // the bookmarked work itself stays in JJ history at the preserved tip.
    ok(f.reseed(&["a"], &[]));
    assert_eq!(f.id(&f.a, "@-"), f.base, "sync child of the donor's base");
    assert_eq!(f.id(&f.a, "feature"), preserved);
    assert_eq!(fs::read(f.a.join("file")).unwrap(), b"baseline\n");
    ok(f.reseed(&["a"], &["--at", &f.base]));
    assert_eq!(fs::read(f.a.join("file")).unwrap(), b"baseline\n");
    assert_eq!(f.id(&f.a, "feature"), preserved);
    assert_eq!(
        f.jj(&f.source, &["--ignore-working-copy", "bookmark", "list", "-T", "name ++ \"\\n\""]),
        "feature"
    );
    assert!(f.records(".receipt.json").is_empty());
}

#[test]
fn descendant_refs_cover_scratch_parents_but_described_empty_changes_and_merges_count_themselves() {
    let Some(f) = Fixture::new() else { return };
    fs::write(f.a.join("file"), b"feature\n").unwrap();
    f.jj(&f.a, &["status"]);
    let work = f.id(&f.a, "@");
    let descendant = f._temp.path().join("descendant");
    f.workspace("descendant", &descendant, &work);
    f.bookmark(&descendant, "descendant", "@");
    let preview = ok(f.reseed(&["a"], &["--dry-run"]));
    assert!(preview.contains(&format!("bookmark \"descendant\" covers work tip {work}")), "{preview}");
    f.jj(&f.a, &["new"]);
    let scratch = f.id(&f.a, "@");
    assert_ne!(scratch, work);
    let preview = ok(f.reseed(&["a"], &["--dry-run"]));
    assert!(preview.contains(&format!("covers work tip {work}")), "{preview}");
    ok(f.reseed(&["a"], &[]));
    f.jj(&f.a, &["describe", "-m", "intentional empty checkpoint"]);
    let checkpoint = f.id(&f.a, "@");
    let blocked = f.reseed(&["a"], &["--dry-run"]);
    assert!(!blocked.status.success());
    assert!(text(&blocked).contains(&format!("unbookmarked work tip {checkpoint}")));
    f.bookmark(&f.a, "checkpoint", "@");
    ok(f.reseed(&["a"], &["--dry-run"]));
    f.jj(&f.b, &["describe", "-m", "other empty lineage"]);
    let other = f.id(&f.b, "@");
    f.jj(&f.a, &["new", &checkpoint, &other]);
    let merge = f.id(&f.a, "@");
    let blocked = f.reseed(&["a"], &["--dry-run"]);
    assert!(!blocked.status.success());
    assert!(text(&blocked).contains(&format!("unbookmarked work tip {merge}")));
    f.bookmark(&f.a, "merge", "@");
    ok(f.reseed(&["a"], &["--dry-run"]));
}

#[test]
fn explicit_opt_out_never_bypasses_other_safety_checks() {
    let Some(f) = Fixture::new() else { return };
    fs::write(f.a.join("file"), b"deliberately unnamed work\n").unwrap();
    for reset in [false, true] {
        let mut args = vec!["--allow-unbookmarked"];
        if reset {
            args.extend(["--at", f.base.as_str()]);
        }
        let mut preview_args = args.clone();
        preview_args.push("--dry-run");
        let preview = ok(f.reseed(&["a"], &preview_args));
        assert!(preview.contains("bookmark requirement explicitly bypassed"), "{preview}");
        ok(f.reseed(&["a"], &args));
    }
    assert!(f.records(".receipt.json").is_empty());
    f.jj(&f.a, &["sparse", "set", "--clear", "--add", "file"]);
    for args in [&["--allow-unbookmarked"][..], &["--allow-unbookmarked", "--dry-run"][..]] {
        let output = f.reseed(&["a"], args);
        assert!(!output.status.success());
        assert!(text(&output).contains("full sparse patterns"));
        assert!(text(&output).contains("jj sparse reset"));
    }
    let output = f.reseed(&["a"], &["--resume", "--allow-unbookmarked"]);
    assert!(!output.status.success());
    assert!(text(&output).contains("unexpected argument '--resume'"));
}

#[test]
fn actual_remote_bookmark_counts_without_a_local_ref_and_removal_stops_coverage() {
    let Some(f) = Fixture::new() else { return };
    fs::write(f.a.join("file"), b"remote-preserved work\n").unwrap();
    f.jj(&f.a, &["status"]);
    let work = f.id(&f.a, "@");
    let backend = f.jj(&f.source, &["--ignore-working-copy", "git", "root"]);
    // Seed an owned bare remote from JJ's actual Git objects, without any push.
    ok(f.command("git", &f.source)
        .args(["--git-dir", &backend, "update-ref", "refs/heads/saved", &work])
        .output()
        .unwrap());
    let remote = f._temp.path().join("remote.git");
    ok(f.command("git", &f.source).args(["clone", "--bare", &backend]).arg(&remote).output().unwrap());
    ok(f.command("git", &f.source)
        .args(["--git-dir", &backend, "update-ref", "-d", "refs/heads/saved"])
        .output()
        .unwrap());
    f.jj(&f.source, &["git", "remote", "add", "backup", remote.to_str().unwrap()]);
    f.jj(&f.source, &["git", "fetch", "--remote", "backup", "--branch", "saved"]);
    assert_eq!(f.jj(&f.source, &["--ignore-working-copy", "bookmark", "list", "-T", "name ++ \"\\n\""]), "");
    let preview = ok(f.reseed(&["a"], &["--dry-run"]));
    assert!(preview.contains(&format!("bookmark \"saved@backup\" covers work tip {work}")), "{preview}");
    ok(f.reseed(&["a"], &[]));
    f.jj(&f.source, &["git", "remote", "remove", "backup"]);
    let blocked = f.reseed(&["a"], &["--dry-run"]);
    assert!(!blocked.status.success());
    assert!(text(&blocked).contains("unbookmarked work tip"));
}
