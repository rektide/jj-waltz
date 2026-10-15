//! Native creation contracts: real private CoW payloads, policy ownership, and rollback.
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
    base: String,
}

fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn failure(output: Output, message: &str) {
    assert!(
        !output.status.success(),
        "unexpected success: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(message),
        "expected {message:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

impl Repo {
    fn new() -> Option<Self> {
        let scratch = std::env::current_dir()
            .unwrap()
            .join(".test-agent/creation-ignored");
        fs::create_dir_all(&scratch).unwrap();
        Self::in_directory(&scratch)
    }

    fn in_directory(directory: &Path) -> Option<Self> {
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
        let temp = tempfile::tempdir_in(directory).unwrap();
        let root = temp.path().join("repo");
        let config = temp.path().join("config");
        fs::create_dir_all(config.join("jj-waltz")).unwrap();
        let mut repo = Self {
            temp,
            root,
            config,
            base: String::new(),
        };
        repo.jj_at(
            repo.temp.path(),
            &["git", "init", "--no-colocate", repo.root.to_str().unwrap()],
        );
        fs::write(
            repo.root.join(".gitignore"),
            "cache/\nnode_modules/\ntarget/\n.jwreseed.local.toml\n.jwlinks.local.toml\n.env\n",
        )
        .unwrap();
        fs::write(repo.root.join("tracked"), vec![b't'; 128 * 1024]).unwrap();
        repo.base = repo.id(&repo.root);
        Some(repo)
    }

    fn command(&self, program: &Path, cwd: &Path) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(cwd)
            .env("XDG_CONFIG_HOME", &self.config)
            .env("JJ_CONFIG", "")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("JJ_USER", "Creation Test")
            .env("JJ_EMAIL", "creation@example.invalid")
            .env("JJ_EDITOR", "true");
        command
    }

    fn jw(&self, cwd: &Path, args: &[&str]) -> Output {
        self.command(Path::new(assert_cmd::cargo::cargo_bin!("jw")), cwd)
            .args(args)
            .output()
            .unwrap()
    }

    fn jj_at(&self, cwd: &Path, args: &[&str]) -> String {
        success(
            self.command(Path::new("jj"), cwd)
                .args(args)
                .output()
                .unwrap(),
        )
        .trim()
        .to_owned()
    }

    fn id(&self, cwd: &Path) -> String {
        self.jj_at(cwd, &["log", "--no-graph", "-r", "@", "-T", "commit_id"])
    }

    fn operation_id(&self) -> String {
        self.jj_at(
            &self.root,
            &[
                "--at-operation",
                "@",
                "--ignore-working-copy",
                "operation",
                "log",
                "--limit=1",
                "--no-graph",
                "-T",
                "id",
            ],
        )
    }

    fn global(&self, text: &str) {
        fs::write(self.config.join("jj-waltz/config.toml"), text).unwrap();
    }

    fn payload(&self, root: &Path, relative: &str) {
        fs::create_dir_all(root.join(relative)).unwrap();
        fs::write(root.join(relative).join("data"), vec![b'p'; 128 * 1024]).unwrap();
    }

    fn cow(&self) -> bool {
        let probe = self.temp.path().join("probe");
        match reflink_copy::reflink(self.root.join("tracked"), &probe) {
            Ok(()) => {
                fs::remove_file(probe).unwrap();
                true
            }
            Err(error) => {
                assert!(
                    std::env::var_os("JW_TEST_REQUIRE_COW").is_none(),
                    "real CoW required: {error}"
                );
                assert!(
                    matches!(error.raw_os_error(), Some(18 | 22 | 38 | 95)),
                    "{error}"
                );
                false
            }
        }
    }

    fn absent(&self, name: &str, path: &Path) {
        assert!(
            !path.exists(),
            "owned failed destination retained: {path:?}"
        );
        let names = self.jj_at(&self.root, &["workspace", "list", "-T", "name ++ \"\\n\""]);
        assert!(!names.lines().any(|n| n == name), "{names}");
        assert!(
            !self
                .jj_at(&self.root, &["bookmark", "list", "-T", "name ++ \"\\n\""])
                .lines()
                .any(|n| n == name)
        );
        let records = self.root.join(".jj/repo/jj-waltz/workspaces");
        if records.is_dir() {
            for record in fs::read_dir(records).unwrap() {
                let value: serde_json::Value =
                    serde_json::from_slice(&fs::read(record.unwrap().path()).unwrap()).unwrap();
                assert_ne!(value["metadata"]["workspace_name"].as_str(), Some(name));
            }
        }
    }
}

#[test]
fn nested_payload_destination_is_rejected_before_creating_any_workspace() {
    let Some(repo) = Repo::new() else { return };
    repo.payload(&repo.root, "cache");
    repo.global("copy_ignored=['cache/']\n");
    fs::create_dir(repo.root.join("slots")).unwrap();
    let alias = repo.temp.path().join("source-alias");
    symlink(&repo.root, &alias).unwrap();
    let operation = repo.operation_id();
    for (command, name, destination) in [
        ("add", "nested", repo.root.join("nested")),
        ("s", "aliased", alias.join("slots/aliased")),
        ("add", "recursive", repo.root.join("cache/recursive")),
    ] {
        let args = [
            command,
            name,
            "--path",
            destination.to_str().unwrap(),
            "--from",
            repo.root.to_str().unwrap(),
            "--no-cow",
            "--bookmark",
            name,
        ];
        failure(
            repo.jw(repo.temp.path(), &args),
            "cannot copy ignored payload into a workspace nested within its source",
        );
        assert_eq!(repo.operation_id(), operation);
        repo.absent(name, &destination);
    }
    assert_eq!(
        fs::read(repo.root.join("cache/data")).unwrap(),
        vec![b'p'; 128 * 1024]
    );
}

#[test]
fn nested_creation_still_works_when_payload_copying_is_disabled_or_matches_nothing() {
    for disabled in [true, false] {
        let Some(repo) = Repo::new() else { return };
        repo.payload(&repo.root, "cache");
        repo.global(if disabled {
            "copy_ignored=['cache/']\n"
        } else {
            "copy_ignored=['missing/']\n"
        });
        let destination = repo.root.join("nested");
        let mut args = vec![
            if disabled { "add" } else { "s" },
            "nested",
            "--path",
            destination.to_str().unwrap(),
            "--no-cow",
        ];
        if disabled {
            args.push("--no-copy-ignored");
        }
        success(repo.jw(&repo.root, &args));
        assert_eq!(
            fs::read(destination.join("tracked")).unwrap(),
            vec![b't'; 128 * 1024]
        );
        assert!(!destination.join("cache").exists());
        assert!(repo.root.join("cache/data").is_file());
    }
}

#[test]
fn creation_source_assertion_accepts_equal_trees_without_selecting_the_target_parent() {
    for command in ["add", "s"] {
        let Some(repo) = Repo::new() else { return };
        repo.jj_at(&repo.root, &["new", &repo.base]);
        let source = repo.id(&repo.root);
        assert_ne!(source, repo.base);
        fs::write(
            repo.root.join(".jwreseed.local.toml"),
            "source='/unusable'\nsource_at='absent-ref'\n[freshen]\ncommand=['never-run']\n",
        )
        .unwrap();
        for explicit_at in [false, true] {
            let name = if explicit_at { "exact" } else { "current" };
            let destination = repo.temp.path().join(format!("chosen-{name}"));
            let mut args = vec![
                command,
                "--from",
                repo.root.to_str().unwrap(),
                "--path",
                destination.to_str().unwrap(),
                "--source-at",
                &repo.base,
                "--no-cow",
            ];
            if explicit_at {
                args.extend(["--at", "root()"]);
            }
            success(repo.jw(repo.temp.path(), &args));
            let parent = repo.jj_at(
                &destination,
                &["log", "--no-graph", "-r", "parents(@)", "-T", "commit_id"],
            );
            if explicit_at {
                assert_eq!(
                    parent,
                    repo.jj_at(
                        &repo.root,
                        &["log", "--no-graph", "-r", "root()", "-T", "commit_id"]
                    )
                );
                assert!(!destination.join("tracked").exists());
            } else {
                assert_eq!(
                    parent, source,
                    "assertion must not replace source @ as parent"
                );
                assert!(destination.join("tracked").is_file());
            }
        }
    }
}

#[test]
fn creation_source_assertion_snapshots_dirty_donor_even_with_explicit_target_at() {
    for command in ["add", "s"] {
        let Some(repo) = Repo::new() else { return };
        fs::write(repo.root.join("tracked"), "unsnapshotted donor\n").unwrap();
        for (assertion, message) in [
            (
                repo.base.as_str(),
                "source tracked tree does not match --source-at",
            ),
            ("all()", "must resolve to exactly one revision"),
        ] {
            failure(
                repo.jw(
                    &repo.root,
                    &[
                        command,
                        "rejected",
                        "--source-at",
                        assertion,
                        "--at",
                        &repo.base,
                        "--no-cow",
                        "--bookmark",
                        "rejected",
                    ],
                ),
                message,
            );
            repo.absent("rejected", &repo.temp.path().join("repo.rejected"));
        }
        assert_ne!(
            repo.id(&repo.root),
            repo.base,
            "assertion must snapshot first"
        );
        success(repo.jw(
            &repo.root,
            &[
                command,
                "accepted",
                "--source-at",
                "@",
                "--at",
                &repo.base,
                "--no-cow",
            ],
        ));
        assert_eq!(
            fs::read(repo.temp.path().join("repo.accepted/tracked")).unwrap(),
            vec![b't'; 128 * 1024],
            "explicit target --at stays independent of the asserted dirty donor"
        );
        assert_eq!(
            fs::read(repo.root.join("tracked")).unwrap(),
            b"unsnapshotted donor\n"
        );
    }
}

#[test]
fn global_selection_is_private_nested_and_independent_of_tracked_checkout_mode() {
    let Some(repo) = Repo::new() else { return };
    if !repo.cow() {
        return;
    }
    repo.global("copy_ignored=['**/node_modules/', '**/target', '**/build/']\n");
    fs::create_dir_all(repo.root.join("build/packages/a")).unwrap();
    fs::write(repo.root.join("build/tracked"), "not ignored\n").unwrap();
    repo.payload(&repo.root, "build/packages/a/node_modules/deep/target");
    repo.payload(&repo.root, "packages/b/target");
    symlink(
        "deep/target/data",
        repo.root.join("build/packages/a/node_modules/link"),
    )
    .unwrap();
    fs::write(repo.root.join(".env"), "not selected").unwrap();
    let result = repo.jw(&repo.root, &["add", "private", "--no-cow"]);
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("not ignored under frozen source rules")
    );
    let output = success(result);
    assert!(output.contains("private"));
    let target = repo.temp.path().join("repo.private");
    assert_eq!(
        fs::read(target.join("build/tracked")).unwrap(),
        b"not ignored\n"
    );
    assert!(!target.join(".env").exists());
    let relative = "build/packages/a/node_modules/deep/target/data";
    let donor = repo.root.join(relative);
    let copy = target.join(relative);
    assert_ne!(
        fs::metadata(&donor).unwrap().ino(),
        fs::metadata(&copy).unwrap().ino()
    );
    assert_shared_extents(&donor, &copy);
    assert!(
        !fs::symlink_metadata(target.join("build/packages/a/node_modules"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read_link(target.join("build/packages/a/node_modules/link")).unwrap(),
        Path::new("deep/target/data")
    );
    fs::write(&copy, "private edit").unwrap();
    assert_eq!(fs::read(&donor).unwrap(), vec![b'p'; 128 * 1024]);
    fs::write(&donor, "donor edit").unwrap();
    assert_eq!(fs::read(&copy).unwrap(), b"private edit");
}

#[test]
fn primary_local_policy_does_not_select_a_donor_or_trust_a_default_named_workspace() {
    let Some(repo) = Repo::new() else { return };
    if !repo.cow() {
        return;
    }
    repo.jj_at(&repo.root, &["workspace", "rename", "primary"]);
    let donor = repo.temp.path().join("unrelated-default");
    repo.jj_at(
        &repo.root,
        &[
            "workspace",
            "add",
            "--name",
            "default",
            "--revision",
            &repo.base,
            donor.to_str().unwrap(),
        ],
    );
    repo.global("copy_ignored=['**/target/']\n");
    fs::write(repo.root.join(".jwreseed.local.toml"), "copy_ignored=['cache/']\nsource='/unusable'\nsource_at='absent-ref'\n[freshen]\ncommand=['never-run']\n").unwrap();
    fs::write(donor.join(".jwreseed.local.toml"), "copy_ignored=[]\n").unwrap();
    repo.payload(&donor, "cache");
    repo.payload(&donor, "target");
    fs::write(donor.join("cache/data"), vec![b'd'; 128 * 1024]).unwrap();
    fs::write(donor.join("tracked"), "dirty donor tracked bytes\n").unwrap();
    let chosen = repo.temp.path().join("chosen-location");
    success(repo.jw(
        &repo.root,
        &[
            "s",
            "chosen",
            "--path",
            chosen.to_str().unwrap(),
            "--from",
            donor.to_str().unwrap(),
            "--source-at",
            "@",
            "--no-links",
        ],
    ));
    assert_eq!(
        fs::read(chosen.join("tracked")).unwrap(),
        b"dirty donor tracked bytes\n"
    );
    assert_eq!(
        fs::read(chosen.join("cache/data")).unwrap(),
        vec![b'd'; 128 * 1024]
    );
    assert!(!chosen.join("target").exists());
    success(repo.jw(
        &donor,
        &["add", "cli", "--copy-ignored", "target/", "--no-links"],
    ));
    assert!(repo.temp.path().join("repo.cli/target/data").exists());
    assert!(!repo.temp.path().join("repo.cli/cache").exists());
    success(repo.jw(
        &donor,
        &[
            "s",
            "disabled",
            "--copy-ignored",
            "cache",
            "--no-copy-ignored",
            "--no-links",
        ],
    ));
    assert!(!repo.temp.path().join("repo.disabled/cache").exists());
    fs::write(repo.root.join(".jwreseed.local.toml"), "copy_ignored=[]\n").unwrap();
    success(repo.jw(&donor, &["add", "empty", "--no-links"]));
    assert!(!repo.temp.path().join("repo.empty/target").exists());
    assert!(!repo.temp.path().join("repo.empty/cache").exists());
}

#[test]
fn existing_switch_does_not_refill_or_even_compile_creation_policy() {
    let Some(repo) = Repo::new() else { return };
    success(repo.jw(&repo.root, &["add", "existing", "--no-copy-ignored"]));
    let target = repo.temp.path().join("repo.existing");
    repo.payload(&target, "cache");
    fs::write(target.join("cache/data"), "target private state").unwrap();
    repo.global("copy_ignored=['../invalid']\n");
    fs::write(repo.root.join(".jwreseed.local.toml"), "not valid toml").unwrap();
    let before = repo.id(&target);
    success(repo.jw(
        &repo.root,
        &[
            "s",
            "existing",
            "--copy-ignored",
            "also/[invalid]",
            "--source-at",
            "absent-ref",
            "--no-links",
        ],
    ));
    assert_eq!(
        fs::read(target.join("cache/data")).unwrap(),
        b"target private state"
    );
    assert_eq!(repo.id(&target), before);
}

#[test]
fn exact_target_base_must_ignore_payload_and_have_no_tracked_overlap() {
    let Some(repo) = Repo::new() else { return };
    repo.jj_at(&repo.root, &["new", &repo.base]);
    fs::write(repo.root.join(".gitignore"), ".jwreseed.local.toml\n").unwrap();
    let nonignored = repo.id(&repo.root);
    repo.payload(&repo.root, "cache");
    let tracked = repo.id(&repo.root);
    repo.jj_at(&repo.root, &["new", &repo.base]);
    repo.payload(&repo.root, "cache");
    for (name, base, message) in [
        (
            "nonignored",
            nonignored,
            "not ignored in the desired target tree",
        ),
        ("tracked", tracked, "overlaps the desired target tree"),
    ] {
        failure(
            repo.jw(
                &repo.root,
                &[
                    "add",
                    name,
                    "--at",
                    &base,
                    "--copy-ignored",
                    "cache",
                    "--no-cow",
                ],
            ),
            message,
        );
        repo.absent(name, &repo.temp.path().join(format!("repo.{name}")));
        assert!(repo.root.join("cache/data").is_file());
    }
    if repo.cow() {
        fs::write(repo.root.join("tracked"), "donor-only dirty state\n").unwrap();
        success(repo.jw(
            &repo.root,
            &[
                "add",
                "exact",
                "--at",
                &repo.base,
                "--source-at",
                "@",
                "--copy-ignored",
                "cache",
            ],
        ));
        let target = repo.temp.path().join("repo.exact");
        assert_eq!(
            repo.jj_at(
                &target,
                &["log", "--no-graph", "-r", "parents(@)", "-T", "commit_id"]
            ),
            repo.base
        );
        assert!(target.join("cache/data").is_file());
        assert_eq!(
            fs::read(target.join("tracked")).unwrap(),
            vec![b't'; 128 * 1024]
        );
        assert_eq!(
            fs::read(repo.root.join("tracked")).unwrap(),
            b"donor-only dirty state\n"
        );
    }
}

#[test]
fn links_cannot_substitute_or_enter_private_payload_and_local_self_override_works() {
    let Some(repo) = Repo::new() else { return };
    if !repo.cow() {
        return;
    }
    repo.payload(&repo.root, "cache");
    let unrelated = repo.temp.path().join("unrelated");
    fs::create_dir(&unrelated).unwrap();
    fs::write(unrelated.join("keep"), "keep").unwrap();
    for source in ["cache", "cache/new-child"] {
        fs::write(
            repo.root.join(".jwlinks.toml"),
            format!(
                "[[link]]\nsource={source:?}\ntarget={:?}\n",
                repo.root.join("cache")
            ),
        )
        .unwrap();
        failure(
            repo.jw(&repo.root, &["add", "bad-link", "--copy-ignored", "cache"]),
            "overlaps private ignored payload",
        );
        repo.absent("bad-link", &repo.temp.path().join("repo.bad-link"));
    }
    fs::write(
        repo.root.join(".jwlinks.toml"),
        format!(
            "[[link]]\nsource='cache'\ntarget={:?}\n",
            repo.root.join("cache")
        ),
    )
    .unwrap();
    fs::write(
        repo.root.join(".jwlinks.local.toml"),
        "[[link]]\nsource='cache'\ntarget='cache'\nrequired=true\n",
    )
    .unwrap();
    success(repo.jw(
        &repo.root,
        &["add", "private-link", "--copy-ignored", "cache"],
    ));
    let target = repo.temp.path().join("repo.private-link");
    assert!(
        !fs::symlink_metadata(target.join("cache"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_shared_extents(&repo.root.join("cache/data"), &target.join("cache/data"));
    assert_eq!(fs::read(unrelated.join("keep")).unwrap(), b"keep");
}

#[test]
fn post_payload_failure_rolls_back_readonly_owned_content_only() {
    let Some(repo) = Repo::new() else { return };
    if !repo.cow() {
        return;
    }
    repo.payload(&repo.root, "cache/readonly");
    fs::set_permissions(
        repo.root.join("cache/readonly"),
        fs::Permissions::from_mode(0o555),
    )
    .unwrap();
    fs::write(
        repo.root.join(".jwlinks.toml"),
        "[[link]]\nsource='.env'\ntarget='../missing-env'\nrequired=true\n",
    )
    .unwrap();
    failure(
        repo.jw(
            &repo.root,
            &[
                "add",
                "rollback",
                "--bookmark",
                "rollback",
                "--copy-ignored",
                "cache",
            ],
        ),
        "required link target is missing",
    );
    repo.absent("rollback", &repo.temp.path().join("repo.rollback"));
    assert!(repo.root.join("cache/readonly/data").is_file());
    assert_eq!(
        fs::metadata(repo.root.join("cache/readonly"))
            .unwrap()
            .mode()
            & 0o777,
        0o555
    );
    fs::set_permissions(
        repo.root.join("cache/readonly"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn missing_payload_needs_no_clone_support_but_real_payload_has_no_fallback() {
    let approved = Path::new("/tmp/opencode");
    if !approved.is_dir() {
        return;
    }
    let Some(repo) = Repo::in_directory(approved) else {
        return;
    };
    // This fixture intentionally needs a non-reflinking filesystem (tmpfs here).
    let probe = repo.temp.path().join("probe");
    if reflink_copy::reflink(repo.root.join("tracked"), &probe).is_ok() {
        return;
    }
    repo.global("copy_ignored=['missing/nested/']\n");
    success(repo.jw(&repo.root, &["add", "missing", "--no-cow"]));
    repo.payload(&repo.root, "cache");
    failure(
        repo.jw(
            &repo.root,
            &["s", "unsupported", "--no-cow", "--copy-ignored", "cache"],
        ),
        "real reflinks are required",
    );
    repo.absent("unsupported", &repo.temp.path().join("repo.unsupported"));
    assert!(repo.root.join("cache/data").is_file());
    assert!(repo.temp.path().join("repo.missing/tracked").is_file());
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
        "payload must actually share extents"
    );
    assert_ne!(source.flags & 0x2000, 0, "{source:?}");
    assert_ne!(target.flags & 0x2000, 0, "{target:?}");
}
#[cfg(not(target_os = "linux"))]
fn assert_shared_extents(_source: &Path, _target: &Path) {}
