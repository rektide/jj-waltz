//! Two explicit application build variants; current source inputs must match pinned commits.
use clap::ValueEnum;
use diff_dirs_perf_runner::{Result, artifacts, fixture, process, write_json};
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command, time::Duration};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Variant {
    Serial,
    PayloadParallel,
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
fn inputs(repo: &Path, revision: &str, library: bool, stem: &Path) -> Result<serde_json::Value> {
    let paths = if library {
        vec!["Cargo.toml", "Cargo.lock", "src", "sync/Cargo.toml", "sync/src"]
    } else {
        vec!["Cargo.toml", "Cargo.lock", "src"]
    };
    let mut git = Command::new("/usr/bin/git");
    git.arg("-C").arg(repo).args(["diff", "--exit-code", revision, "--"]).args(&paths);
    command(git, stem)?;
    let mut hashes = serde_json::Map::new();
    for path in paths {
        let full = repo.join(path);
        hashes.insert(
            path.into(),
            serde_json::json!(if full.is_dir() {
                fixture::tree(&full, true, false)?
            } else {
                artifacts::sha256(&full)?
            }),
        );
    }
    Ok(hashes.into())
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
    let mut rustc = Command::new(std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into()));
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
