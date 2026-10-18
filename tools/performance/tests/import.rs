use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn run(mut command: Command) -> std::process::Output {
    let out = command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("JJ_USER", "Fixture")
        .env("JJ_EMAIL", "fixture@example.invalid")
        .env("JJ_CONFIG", "")
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    out
}
#[test]
#[ignore = "requires preserved native BEFORE; explicit retained import/campaign smoke"]
fn native_import_preserves_forced_membership_nested_roots_and_links_without_jj_mutation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-import-tests");
    fs::create_dir_all(&root).unwrap();
    let owned = tempfile::tempdir_in(root).unwrap().keep();
    println!("retained native import proof: {}", owned.display());
    let remote = owned.join("remote");
    fs::create_dir(&remote).unwrap();
    fs::write(remote.join(".gitignore"), "node_modules/\ndist/\n*.log\n.env\n.cache/\n").unwrap();
    fs::write(remote.join("tracked.txt"), b"tracked\n").unwrap();
    fs::write(remote.join("kept.log"), [0u8, 255, 2]).unwrap();
    fs::create_dir_all(remote.join("packages/b")).unwrap();
    fs::write(remote.join("packages/b/.gitignore"), "!dist/\n!dist/**\n").unwrap();
    let mut c = Command::new("/usr/bin/git");
    c.args(["init", "--quiet", "--initial-branch=dev"]).arg(&remote);
    run(c);
    let mut c = Command::new("/usr/bin/git");
    c.current_dir(&remote)
        .args(["add", "--force", ".gitignore", "tracked.txt", "kept.log", "packages/b/.gitignore"]);
    run(c);
    let mut c = Command::new("/usr/bin/git");
    c.current_dir(&remote).args(["commit", "--quiet", "-m", "owned fixture"]);
    run(c);
    let donor = owned.join("donor");
    let mut c = Command::new("/usr/local/bin/jj");
    c.args(["git", "clone", "--no-colocate"]).arg(&remote).arg(&donor);
    run(c);
    for path in [
        "node_modules/a",
        "packages/a/node_modules/b",
        "packages/a/dist/chunk",
        "dist/root",
        "packages/b/dist/unignored",
    ] {
        fs::create_dir_all(donor.join(path)).unwrap();
        fs::write(donor.join(path).join("data"), b"payload").unwrap();
    }
    fs::write(donor.join(".env"), b"secret-not-selected").unwrap();
    fs::create_dir(donor.join(".cache")).unwrap();
    fs::write(donor.join(".cache/junk"), b"not-selected").unwrap();
    std::os::unix::fs::symlink("data", donor.join("node_modules/a/link")).unwrap();
    let operation = || {
        let mut c = Command::new("/usr/local/bin/jj");
        c.args(["--ignore-working-copy", "-R"])
            .arg(&donor)
            .args(["op", "log", "--no-graph", "--limit", "1", "-T", "id"]);
        run(c).stdout
    };
    let before = operation();
    let import = owned.join("import");
    let mut c = Command::new(env!("CARGO_BIN_EXE_jw-perf"));
    c.arg("snapshot").arg(&donor).arg(&import);
    run(c);
    assert_eq!(before, operation());
    let fixture = import.join("fixture");
    let record: serde_json::Value = serde_json::from_slice(&fs::read(fixture.join("snapshot.json")).unwrap()).unwrap();
    let roots = record["payload_roots"].as_array().unwrap();
    assert_eq!(roots.len(), 4);
    assert!(roots.contains(&serde_json::json!("packages/a/dist")));
    let base = fixture.join("source");
    assert_eq!(fs::read(base.join("kept.log")).unwrap(), [0u8, 255, 2]);
    assert!(!base.join(".jj").exists());
    assert!(!base.join(".git").exists());
    assert!(!base.join(".env").exists());
    assert!(!base.join(".cache").exists());
    assert!(!base.join("packages/b/dist").exists());
    assert_eq!(fs::read_link(base.join("node_modules/a/link")).unwrap(), PathBuf::from("data"));
    let baseline = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/inline-policy/jw-before");
    let provenance = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/inline-policy/baseline.json");
    assert!(baseline.is_file(), "native proof requires preserved BEFORE; see tooling README");
    let campaign = owned.join("campaign");
    let mut c = Command::new(env!("CARGO_BIN_EXE_jw-perf"));
    c.arg("run")
        .arg(&campaign)
        .arg("--fixture")
        .arg(&fixture)
        .arg("--baseline")
        .arg(baseline)
        .arg("--baseline-provenance")
        .arg(provenance)
        .args(["--destinations", "2", "--feature-files", "2", "--scenario", "sparse", "--blocks", "2"]);
    run(c);
    let result: serde_json::Value = serde_json::from_slice(&fs::read(campaign.join("summary.json")).unwrap()).unwrap();
    assert_eq!(result["passed"], true);
}
