//! Native-JJ reseeding contracts. No mocked filesystem copies: successful runs
//! must use a filesystem supporting reflinks (set JW_TEST_REQUIRE_COW in CI).
#![cfg(unix)]

use assert_cmd::prelude::*;
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
        if !Command::new("jj")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
        {
            assert!(
                std::env::var_os("JW_TEST_REQUIRE_COW").is_none(),
                "jj is required"
            );
            eprintln!("skipping reseed integration test: jj unavailable");
            return None;
        }
        // Keep fixtures on the checkout's filesystem, not an unrelated /tmp mount.
        let temp = tempfile::tempdir_in(std::env::current_dir().unwrap()).unwrap();
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
            &["git", "init", "--no-colocate", repo.root.to_str().unwrap()],
        );
        fs::write(
            repo.root.join(".gitignore"),
            "cache/\n.env\n.jwreseed.local.toml\npackages/\n",
        )
        .unwrap();
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
        cmd
    }

    fn jw(&self, cwd: &Path) -> Command {
        let mut cmd = self.command("jw", cwd);
        cmd = {
            let mut binary = Command::cargo_bin("jw").unwrap();
            binary
                .current_dir(cwd)
                .envs(cmd.get_envs().filter_map(|(k, v)| v.map(|v| (k, v))));
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
        self.jj_at(
            &self.root,
            &[
                "workspace",
                "add",
                "--name",
                name,
                "-r",
                &self.baseline,
                path.to_str().unwrap(),
            ],
        );
        path
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
                    std::env::var_os("JW_TEST_REQUIRE_COW").is_none(),
                    "required reflink unavailable: {error}"
                );
                // Permission and unrelated IO failures must not silently skip.
                assert!(
                    matches!(error.raw_os_error(), Some(18 | 22 | 38 | 95)),
                    "unexpected reflink error: {error}"
                );
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
        assert!(
            !output.status.success(),
            "unexpected success: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert_eq!(self.id(target, "@"), before);
        assert_eq!(
            fs::read(target.join("shared")).unwrap(),
            vec![b's'; 128 * 1024]
        );
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
}

fn identity(path: &Path) -> (u64, u64) {
    let m = fs::symlink_metadata(path).unwrap();
    (m.dev(), m.ino())
}

#[test]
fn keep_preserves_feature_tree_types_modes_and_literal_paths() {
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
    let before = repo.id(&target, "@");
    let root = identity(&target);
    let jj = identity(&target.join(".jj"));
    for _ in 0..2 {
        success(repo.reseed(&["feature"], &source, &[]));
        assert_eq!(repo.id(&target, "@"), before);
        assert_eq!(identity(&target), root);
        assert_eq!(identity(&target.join(".jj")), jj);
        assert_eq!(
            fs::read_to_string(target.join("modified")).unwrap(),
            "feature\n"
        );
        assert!(!target.join("deleted").exists());
        assert_eq!(
            fs::read_link(target.join("transition")).unwrap(),
            Path::new("modified")
        );
        assert_eq!(
            fs::read_to_string(target.join("directory-transition/child")).unwrap(),
            "child\n"
        );
        assert_ne!(fs::metadata(target.join("mode")).unwrap().mode() & 0o111, 0);
        assert_eq!(
            fs::read_link(target.join("link")).unwrap(),
            Path::new("shared")
        );
        assert!(!target.join("line\nbreak").exists());
        assert!(target.join("renamed\nfile").is_file());
        assert_eq!(
            fs::read_to_string(target.join("added")).unwrap(),
            "feature only\n"
        );
        assert_ne!(
            identity(&source.join("shared")),
            identity(&target.join("shared"))
        );
        assert_shared_extents(&source.join("shared"), &target.join("shared"));
    }
}

#[test]
fn reset_batch_across_two_sources_keeps_roots_bookmarks_and_lifecycle() {
    let Some(mut repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    success(
        repo.jw(&repo.root)
            .args(["add", "worker-a", "worker-b"])
            .output()
            .unwrap(),
    );
    let a = repo.root.with_extension("worker-a");
    let b = repo.root.with_extension("worker-b");
    if !repo.cow(&source, &a) || !repo.cow(&source, &b) {
        return;
    }
    fs::write(a.join("old-only"), "old\n").unwrap();
    repo.id(&a, "@");
    repo.jj_at(&a, &["bookmark", "create", "feature-bookmark"]);
    let bookmarks = repo.jj_at(&repo.root, &["bookmark", "list", "-T", "name ++ commit_id"]);
    let lifecycle = repo.lifecycle();
    assert!(
        !lifecycle.is_empty(),
        "fixture must exercise lifecycle metadata"
    );
    let identities = [
        identity(&a),
        identity(&a.join(".jj")),
        identity(&b),
        identity(&b.join(".jj")),
    ];
    for pass in 0..2 {
        if pass == 1 {
            repo.jj_at(&source, &["new"]);
            fs::write(source.join("modified"), "new baseline\n").unwrap();
            fs::write(source.join("new-only"), "new\n").unwrap();
            repo.baseline = repo.id(&source, "@");
        }
        success(repo.reseed(
            &["worker-a", "worker-b"],
            &source,
            &["--at", &repo.baseline],
        ));
        let ids = [repo.id(&a, "@"), repo.id(&b, "@")];
        success(repo.reseed(
            &["worker-a", "worker-b"],
            &source,
            &["--at", &repo.baseline],
        ));
        assert_eq!([repo.id(&a, "@"), repo.id(&b, "@")], ids);
        for target in [&a, &b] {
            assert_eq!(repo.id(target, "@-"), repo.baseline);
            assert!(!target.join("old-only").exists());
            assert_eq!(
                fs::read(target.join("modified")).unwrap(),
                fs::read(source.join("modified")).unwrap()
            );
            assert_shared_extents(&source.join("shared"), &target.join("shared"));
        }
        assert_eq!(
            [
                identity(&a),
                identity(&a.join(".jj")),
                identity(&b),
                identity(&b.join(".jj"))
            ],
            identities
        );
        assert_eq!(repo.lifecycle(), lifecycle);
        assert_eq!(
            repo.jj_at(&repo.root, &["bookmark", "list", "-T", "name ++ commit_id"]),
            bookmarks
        );
    }
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
    let before = repo.id(&target, "@");
    let root_id = identity(&target);
    success(repo.reseed(&["feature"], &source, &["--copy-ignored", "cache"]));
    assert_eq!(repo.id(&target, "@"), before);
    assert!(!target.join("cache/obsolete").exists());
    assert_eq!(fs::read_to_string(target.join(".env")).unwrap(), "secret");
    assert_eq!(
        fs::read_link(target.join("cache/link")).unwrap(),
        Path::new("fresh")
    );
    assert_ne!(
        identity(&source.join("cache/fresh")),
        identity(&target.join("cache/fresh"))
    );
    assert_shared_extents(&source.join("cache/fresh"), &target.join("cache/fresh"));
    fs::write(target.join("cache/fresh"), "target edit").unwrap();
    assert_eq!(
        fs::read(source.join("cache/fresh")).unwrap(),
        vec![b'c'; 128 * 1024]
    );
    fs::write(source.join("cache/fresh"), "source edit").unwrap();
    assert_eq!(
        fs::read_to_string(target.join("cache/fresh")).unwrap(),
        "target edit"
    );
    success(repo.reseed(
        &["feature"],
        &source,
        &["--copy-ignored", "cache", "--replace-content"],
    ));
    assert!(!target.join(".env").exists());
    assert_eq!(identity(&target), root_id);
    assert_eq!(
        fs::read_to_string(target.join("cache/fresh")).unwrap(),
        "source edit"
    );
}

#[test]
fn selection_is_explicit_and_primary_source_and_unknown_targets_are_rejected() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    let before = repo.id(&target, "@");
    for names in [
        &[][..],
        &["default"][..],
        &["source"][..],
        &["missing"][..],
        &["feature", "missing"][..],
    ] {
        repo.unchanged_failure(&target, repo.reseed(names, &source, &[]), &before);
    }
}

#[test]
fn dirty_or_sparse_source_and_nonignored_payload_fail_before_target_mutation() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    let before = repo.id(&target, "@");
    fs::write(source.join("modified"), "dirty\n").unwrap();
    repo.unchanged_failure(&target, repo.reseed(&["feature"], &source, &[]), &before);
    repo.jj_at(&source, &["restore", "--from", &repo.baseline]);
    repo.jj_at(
        &source,
        &["config", "set", "--repo", "snapshot.auto-track", "none()"],
    );
    fs::create_dir(source.join("not-ignored")).unwrap();
    fs::write(source.join("not-ignored/file"), "payload").unwrap();
    // Keep this untracked so source-at still matches; the payload is not ignored.
    repo.unchanged_failure(
        &target,
        repo.reseed(&["feature"], &source, &["--copy-ignored", "not-ignored"]),
        &before,
    );
    fs::remove_dir_all(source.join("not-ignored")).unwrap();
    repo.jj_at(&source, &["sparse", "set", "--clear"]);
    repo.unchanged_failure(&target, repo.reseed(&["feature"], &source, &[]), &before);
}

#[test]
fn copied_payload_rejects_nested_metadata_and_symlink_parent_escape() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    let before = repo.id(&target, "@");
    fs::create_dir_all(source.join("cache/nested/.jj")).unwrap();
    repo.unchanged_failure(
        &target,
        repo.reseed(
            &["feature"],
            &source,
            &["--copy-ignored", "cache", "--replace-content"],
        ),
        &before,
    );
    fs::remove_dir_all(source.join("cache")).unwrap();
    fs::create_dir_all(source.join("packages/pkg/cache")).unwrap();
    fs::write(source.join("packages/pkg/cache/file"), "source").unwrap();
    let outside = repo.temp.path().join("outside");
    fs::create_dir_all(outside.join("pkg/cache")).unwrap();
    fs::write(outside.join("pkg/cache/sentinel"), "untouched").unwrap();
    symlink(&outside, target.join("packages")).unwrap();
    repo.unchanged_failure(
        &target,
        repo.reseed(
            &["feature"],
            &source,
            &["--copy-ignored", "packages/pkg/cache"],
        ),
        &before,
    );
    assert_eq!(
        fs::read_to_string(outside.join("pkg/cache/sentinel")).unwrap(),
        "untouched"
    );
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
    let config = format!(
        "source = {:?}\nsource_at = {:?}\ncopy_ignored = [\"cache\"]\n[groups]\nworkers = [\"feature\", \"feature\"]\n",
        source.to_str().unwrap(),
        repo.baseline
    );
    fs::write(repo.root.join(".jwreseed.local.toml"), config).unwrap();
    let before = repo.id(&target, "@");
    // Config is read from the default root even when invoked in the target.
    success(
        repo.jw(&target)
            .args([
                "reseed",
                "feature",
                "--group",
                "workers",
                "--no-copy-ignored",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(
        fs::read_to_string(target.join("cache/file")).unwrap(),
        "target"
    );
    assert_eq!(repo.id(&target, "@"), before);
    success(
        repo.jw(&target)
            .args(["reseed", "--group", "workers"])
            .output()
            .unwrap(),
    );
    assert_eq!(
        fs::read_to_string(target.join("cache/file")).unwrap(),
        "source"
    );
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
    let identities = [
        identity(&target),
        identity(&target.join(".jj")),
        identity(&target.join("shared")),
    ];
    success(repo.reseed(
        &["feature"],
        &source,
        &["--dry-run", "--replace-content", "--at", &repo.baseline],
    ));
    assert_eq!(repo.id(&target, "@"), before);
    assert_eq!(
        [
            identity(&target),
            identity(&target.join(".jj")),
            identity(&target.join("shared"))
        ],
        identities
    );
    assert_eq!(fs::read_to_string(target.join(".env")).unwrap(), "keep");
}

#[test]
fn special_workspace_names_are_literal_not_switch_tokens() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("^");
    if !repo.cow(&source, &target) {
        return;
    }
    let before = repo.id(&target, "@");
    success(repo.reseed(&["^"], &source, &[]));
    assert_eq!(repo.id(&target, "@"), before);
    assert_shared_extents(&source.join("shared"), &target.join("shared"));
}

#[test]
fn colocated_source_is_supported_but_colocated_primary_target_is_not() {
    let Some(repo) = Repo::new() else { return };
    repo.jj_at(&repo.root, &["git", "colocation", "enable"]);
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
fn resume_after_sparse_expansion_does_not_snapshot_partial_prefill() {
    let Some(repo) = Repo::new() else { return };
    let source = repo.workspace("source");
    let target = repo.workspace("feature");
    if !repo.cow(&source, &target) {
        return;
    }
    fs::write(target.join("modified"), "feature\n").unwrap();
    let before = repo.id(&target, "@");
    let root = identity(&target);
    let jj = identity(&target.join(".jj"));
    let proxy = repo.temp.path().join("proxy");
    fs::create_dir(&proxy).unwrap();
    let real_jj = success(
        Command::new("sh")
            .args(["-c", "command -v jj"])
            .output()
            .unwrap(),
    );
    let marker = proxy.join("expanded");
    // Execute expansion successfully, then report a lost subprocess result.
    // This exercises the dangerous boundary before the next target snapshot.
    fs::write(proxy.join("jj"), "#!/bin/sh\ncase \" $* \" in\n  *' sparse reset '*|*' sparse set '* )\n    case \" $* \" in *' --clear '*) exec \"$JW_REAL_JJ\" \"$@\";; esac\n    \"$JW_REAL_JJ\" \"$@\" || exit $?\n    : > \"$JW_EXPANSION_MARKER\"\n    echo 'injected failure after sparse expansion' >&2\n    exit 91;;\nesac\nexec \"$JW_REAL_JJ\" \"$@\"\n").unwrap();
    fs::set_permissions(proxy.join("jj"), fs::Permissions::from_mode(0o755)).unwrap();
    let mut paths = vec![proxy];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let output = repo
        .jw(&repo.root)
        .args(["reseed", "feature", "--from"])
        .arg(&source)
        .args(["--source-at", &repo.baseline])
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env("JW_REAL_JJ", real_jj)
        .env("JW_EXPANSION_MARKER", &marker)
        .output()
        .unwrap();
    assert!(!output.status.success(), "fault must interrupt reseed");
    assert!(
        marker.is_file(),
        "fault must actually reach sparse expansion: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(identity(&target), root);
    assert_eq!(identity(&target.join(".jj")), jj);
    // Model an incomplete/invalid prefill at the expanded-but-unsnapshotted
    // boundary. Do not run an ordinary JJ command before recovery.
    fs::write(target.join("shared"), "partial prefill\n").unwrap();
    success(
        repo.jw(&repo.root)
            .args(["reseed", "--resume", "feature"])
            .output()
            .unwrap(),
    );
    assert_eq!(repo.id(&target, "@"), before);
    assert_eq!(
        fs::read(target.join("shared")).unwrap(),
        vec![b's'; 128 * 1024]
    );
    assert_eq!(
        fs::read_to_string(target.join("modified")).unwrap(),
        "feature\n"
    );
    assert_eq!(identity(&target), root);
    assert_eq!(identity(&target.join(".jj")), jj);
    assert_shared_extents(&source.join("shared"), &target.join("shared"));
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
        let result =
            unsafe { libc::ioctl(file.as_raw_fd(), 0xc020660b as libc::c_ulong, &mut map) };
        assert_eq!(
            result,
            0,
            "FIEMAP {path:?}: {}",
            std::io::Error::last_os_error()
        );
        assert_eq!(map.mapped, 1, "expected an extent for {path:?}");
        map.extent
    }
    let source = extent(source);
    let target = extent(target);
    assert_eq!(
        source.physical, target.physical,
        "reseed must actually share extents"
    );
    assert_ne!(
        source.flags & 0x2000,
        0,
        "source extent must be shared: {source:?}"
    );
    assert_ne!(
        target.flags & 0x2000,
        0,
        "target extent must be shared: {target:?}"
    );
}

#[cfg(not(target_os = "linux"))]
fn assert_shared_extents(_source: &Path, _target: &Path) {}
