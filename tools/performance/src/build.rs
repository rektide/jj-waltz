//! Two explicit application build variants; current source inputs must match pinned commits.
use clap::ValueEnum;
use diff_dirs_perf_runner::{Result, artifacts, fixture, process, write_json};
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command, time::Duration};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Variant {
    Serial,
    PayloadParallel,
}

/// Real source inputs of an application build: the virtual workspace's pins
/// (root manifest, lockfile, toolchain file that selects the compiler) plus
/// every production crate the `jj-waltz` binary compiles from. The dev-only
/// tools/performance crate is not a binary input, and target/ or scratch
/// directories are never selected.
pub fn application_paths() -> &'static [&'static str] {
    &[
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "native/jj/Cargo.toml",
        "native/jj/src",
        "workspace/state/Cargo.toml",
        "workspace/state/src",
        "workspace/lifecycle/Cargo.toml",
        "workspace/lifecycle/src",
        "transfer/workspace/Cargo.toml",
        "transfer/workspace/src",
        "refill/engine/Cargo.toml",
        "refill/engine/src",
        "app/cli/Cargo.toml",
        "app/cli/src",
    ]
}

/// Real source inputs of the library build: the diff-dirs workspace pins plus
/// the exact path-dependency closure of `diff-dirs-tree-api` and
/// `diff-dirs-transfer-sync` (observation -> inspection -> comparison -> api,
/// and the transfer sync kernel). Other diff-dirs packages (apps, examples,
/// perf tooling, the retired root src/) are not inputs to a JW build.
pub fn library_paths() -> &'static [&'static str] {
    &[
        "Cargo.toml",
        "Cargo.lock",
        "tree/observe/Cargo.toml",
        "tree/observe/src",
        "tree/inspect/Cargo.toml",
        "tree/inspect/src",
        "tree/compare/Cargo.toml",
        "tree/compare/src",
        "tree/api/Cargo.toml",
        "tree/api/src",
        "transfer/sync/Cargo.toml",
        "transfer/sync/src",
    ]
}

fn command(mut command: Command, stem: &Path) -> Result<String> {
    let result = process::run(&mut command, stem, Duration::from_secs(1800))?;
    if result.code != Some(0) || result.timed_out {
        return Err(format!("build/input check failed; see {}.stderr", stem.display()).into());
    }
    Ok(fs::read_to_string(stem.with_extension("stdout"))?.trim_end().into())
}
fn pin(repo: &Path, requested: Option<&str>, jj: &Path, stem: &Path) -> Result<String> {
    let revision = if let Some(revision) = requested {
        revision.to_owned()
    } else {
        let mut c = Command::new(jj);
        c.args(["--ignore-working-copy", "--repository"])
            .arg(repo)
            .args(["log", "--no-graph", "-r", "@-", "-T", "commit_id"]);
        command(c, stem)?
    };
    if ![40, 64].contains(&revision.len()) || !revision.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("build requires a full immutable source commit".into());
    }
    Ok(revision)
}

/// Hash one selected input: directories as streaming tree fingerprints, files
/// as SHA-256. The same list feeds the clean-tree guard and the provenance
/// sidecar, so provenance cannot claim inputs the guard never checked.
fn hash_inputs(repo: &Path, paths: &[&str]) -> Result<serde_json::Value> {
    let mut hashes = serde_json::Map::new();
    for path in paths {
        let full = repo.join(path);
        hashes.insert(
            (*path).into(),
            serde_json::json!(if full.is_dir() {
                fixture::tree(&full, true, false)?
            } else {
                artifacts::sha256(&full)?
            }),
        );
    }
    Ok(hashes.into())
}

fn inputs(repo: &Path, revision: &str, library: bool, stem: &Path) -> Result<serde_json::Value> {
    let paths = if library { library_paths() } else { application_paths() };
    let mut git = Command::new("/usr/bin/git");
    git.arg("-C").arg(repo).args(["diff", "--exit-code", revision, "--"]).args(paths);
    command(git, stem)?;
    hash_inputs(repo, paths)
}

pub fn build(
    repository: &Path,
    output: &Path,
    revision: Option<&str>,
    library: &Path,
    library_revision: Option<&str>,
    variant: Variant,
    jj: &Path,
) -> Result<()> {
    let _lock = diff_dirs_perf_runner::host_lock()?;
    let repository = fs::canonicalize(repository)?;
    let library = fs::canonicalize(library)?;
    let output = fixture::fresh_path(output)?;
    fs::create_dir(&output)?;
    let revision = pin(&repository, revision, jj, &output.join("source-commit"))?;
    let library_revision = pin(&library, library_revision, jj, &output.join("library-commit"))?;
    let source_inputs = inputs(&repository, &revision, false, &output.join("source-before"))?;
    let library_inputs = inputs(&library, &library_revision, true, &output.join("library-before"))?;
    let mut rustc = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    rustc.current_dir(&repository).args(["--version", "--verbose"]);
    let compiler = command(rustc, &output.join("compiler"))?;
    let features = match variant {
        Variant::Serial => Vec::new(),
        Variant::PayloadParallel => vec!["payload-parallel"],
    };
    let mut cargo = Command::new("cargo");
    cargo
        .current_dir(&repository)
        .args(["build", "--locked", "--release", "-p", "jj-waltz", "--target-dir"])
        .arg(output.join("target"));
    if !features.is_empty() {
        cargo.args(["--features", &features.join(",")]);
    }
    let argv = format!("{cargo:?}");
    command(cargo, &output.join("cargo-build"))?;
    if inputs(&repository, &revision, false, &output.join("source-after"))? != source_inputs
        || inputs(&library, &library_revision, true, &output.join("library-after"))? != library_inputs
    {
        return Err("source changed during application build".into());
    }
    let binary = output.join("jw");
    fs::copy(output.join("target/release/jw"), &binary)?;
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o555))?;
    let environment: std::collections::BTreeMap<_, _> = std::env::vars()
        .filter(|(name, _)| {
            [
                "RUSTC",
                "RUSTFLAGS",
                "CARGO_ENCODED_RUSTFLAGS",
                "RUSTUP_TOOLCHAIN",
                "RUSTC_BOOTSTRAP",
                "RUSTC_WRAPPER",
                "RUSTC_WORKSPACE_WRAPPER",
            ]
            .contains(&name.as_str())
                || name.starts_with("CARGO_PROFILE_RELEASE_")
        })
        .collect();
    write_json(
        &output.join("provenance.json"),
        &serde_json::json!({"source_commit":revision,"repository":repository,"source_inputs":source_inputs,
        "library_commit":library_revision,"library":library,"library_inputs":library_inputs,"rustc":compiler,"profile":"release (unchanged source profile)",
        "features":features,"build_command":argv,"build_environment":environment,"sha256":artifacts::sha256(&binary)?,
        "driver_sha256":artifacts::sha256(&std::env::current_exe()?)?,"driver_source":diff_dirs_perf_runner::source_identity()}),
    )?;
    println!("{}", output.join("provenance.json").display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// Every selected input must exist in the real workspaces: a stale list
    /// would silently narrow provenance instead of failing loudly.
    #[test]
    fn selected_inputs_exist_for_application_and_library_workspaces() {
        for path in application_paths() {
            assert!(
                repository().join(path).exists(),
                "application input {path} is missing from the jj-waltz workspace"
            );
        }
        assert!(application_paths().contains(&"rust-toolchain.toml"), "toolchain selection is a build input");

        let library = repository().join("../diff-dirs");
        for path in library_paths() {
            assert!(library.join(path).exists(), "library input {path} is missing from the diff-dirs workspace");
        }
    }

    /// Provenance must react to changes inside selected inputs and stay
    /// invariant to unselected files (build artifacts, scratch, design notes).
    #[test]
    fn input_hashes_track_selected_source_and_ignore_unselected_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
        fs::write(root.join("Cargo.lock"), "version = 3\n").unwrap();
        fs::create_dir_all(root.join("app/cli/src")).unwrap();
        fs::write(root.join("app/cli/Cargo.toml"), "[package]\n").unwrap();
        fs::write(root.join("app/cli/src/main.rs"), "fn main() {}\n").unwrap();
        let selected = ["Cargo.toml", "Cargo.lock", "app/cli/Cargo.toml", "app/cli/src"];

        let before = hash_inputs(root, &selected).unwrap();

        // Unselected noise: build output and scratch at unselected locations.
        fs::create_dir_all(root.join("target/debug")).unwrap();
        fs::write(root.join("target/debug/artifact"), "junk").unwrap();
        fs::create_dir_all(root.join(".test-agent")).unwrap();
        fs::write(root.join(".test-agent/scratch"), "junk").unwrap();
        fs::create_dir_all(root.join("design")).unwrap();
        fs::write(root.join("design/note.md"), "junk").unwrap();
        assert_eq!(hash_inputs(root, &selected).unwrap(), before);

        // A selected manifest change is detected.
        fs::write(root.join("Cargo.toml"), "[workspace]\n# changed\n").unwrap();
        let after_manifest = hash_inputs(root, &selected).unwrap();
        assert_ne!(after_manifest["Cargo.toml"], before["Cargo.toml"]);

        // A selected source-tree change is detected.
        fs::write(root.join("app/cli/src/main.rs"), "fn main() { changed() }\n").unwrap();
        let after_source = hash_inputs(root, &selected).unwrap();
        assert_ne!(after_source["app/cli/src"], before["app/cli/src"]);
    }
}
