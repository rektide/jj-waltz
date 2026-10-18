mod context;
mod diagnostics;
mod instrumentation;
mod workload;

use clap::{Parser, Subcommand};
use diff_dirs_perf_runner::{
    Result,
    artifacts::{self, Control},
    execution,
    scenario::Scenario,
};
use serde_json::json;
use std::{fs, path::PathBuf};
use workload::{Config, Phase, Timing};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    action: Action,
}
#[derive(Subcommand)]
enum Action {
    Run(Box<RunArgs>),
    Check {
        root: PathBuf,
        #[arg(long)]
        max_jj_calls: Vec<String>,
        #[arg(long)]
        max_file_show_calls: Vec<String>,
        #[arg(long)]
        max_git_calls: Vec<String>,
    },
    Compare {
        before: PathBuf,
        after: PathBuf,
        #[arg(long)]
        scaling: bool,
        #[arg(long)]
        max_jj_growth: Option<u64>,
        #[arg(long)]
        max_file_show_growth: Option<u64>,
        #[arg(long)]
        max_git_growth: Option<u64>,
    },
    CopyBaseline {
        source: PathBuf,
        output: PathBuf,
    },
    /// Fixture-only Rust installer, invoked by owned preparation config.
    Install {
        root: PathBuf,
    },
}

#[derive(clap::Args)]
struct RunArgs {
    output: PathBuf,
    #[arg(long)]
    baseline: PathBuf,
    #[arg(long)]
    baseline_provenance: PathBuf,
    #[arg(long, requires = "candidate_provenance")]
    candidate: Option<PathBuf>,
    #[arg(long, requires = "candidate")]
    candidate_provenance: Option<PathBuf>,
    #[arg(long, default_value = "/usr/local/bin/jj")]
    jj: PathBuf,
    #[arg(long, default_value = "/usr/bin/git")]
    git: PathBuf,
    #[arg(long, value_enum, default_value = "clean")]
    scenario: Scenario,
    #[arg(long, value_enum, default_value = "reseed")]
    phase: Phase,
    #[arg(long, value_enum, default_value = "native")]
    timing: Timing,
    #[arg(long, default_value_t = 2)]
    destinations: usize,
    #[arg(long, default_value_t = 4)]
    files: usize,
    #[arg(long, default_value_t = 1)]
    symlinks: usize,
    #[arg(long, default_value_t = 0)]
    feature_files: usize,
    #[arg(long, default_value_t = 4)]
    payload_files: usize,
    #[arg(long, default_value_t = 4096)]
    payload_bytes: usize,
    #[arg(long, default_value_t = 2)]
    blocks: usize,
    #[arg(long, default_value_t = 1701)]
    seed: u64,
    /// Only explicitly named incremental candidates must have zero clean payload rewrites.
    #[arg(long)]
    require_clean_label: Option<String>,
}

fn import(label: &str, binary: PathBuf, provenance: PathBuf) -> Result<Control> {
    let binary = instrumentation::native(&binary)?;
    let provenance = fs::canonicalize(provenance)?;
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&provenance)?)?;
    let sha = artifacts::sha256(&binary)?;
    let expected = value
        .get("sha256")
        .or_else(|| value.get("binary_sha256"))
        .and_then(|v| v.as_str())
        .ok_or("provenance needs the original binary sha256")?;
    if expected != sha {
        return Err("JW executable differs from build provenance".into());
    }
    Ok(Control {
        label: label.into(),
        binary,
        sha256: sha,
        mode: "none".into(),
        provenance: value,
        provenance_sha256: artifacts::sha256(&provenance)?,
        provenance_path: provenance,
    })
}

fn main() -> Result<()> {
    if instrumentation::wrapper()? {
        return Ok(());
    }
    match Cli::parse().action {
        Action::Install { root } => workload::install(&root)?,
        Action::CopyBaseline { source, output } => diagnostics::copy_baseline(&source, &output)?,
        Action::Check {
            root,
            max_jj_calls,
            max_file_show_calls,
            max_git_calls,
        } => {
            let mut budgets = Vec::new();
            for (metric, specs) in [
                ("jj_calls", max_jj_calls),
                ("file_show_calls", max_file_show_calls),
                ("git_calls", max_git_calls),
            ] {
                budgets.extend(specs.into_iter().map(|spec| (metric.to_string(), spec)));
            }
            diagnostics::check(&root, &budgets)?;
        }
        Action::Compare {
            before,
            after,
            scaling,
            max_jj_growth,
            max_file_show_growth,
            max_git_growth,
        } => {
            let growth = [
                ("jj_calls", max_jj_growth),
                ("file_show_calls", max_file_show_growth),
                ("git_calls", max_git_growth),
            ]
            .into_iter()
            .filter_map(|(name, value)| value.map(|v| (name.to_string(), v)))
            .collect::<Vec<_>>();
            diagnostics::compare(&before, &after, scaling, &growth)?;
        }
        Action::Run(args) => {
            let RunArgs {
                output,
                baseline,
                baseline_provenance,
                candidate,
                candidate_provenance,
                jj,
                git,
                scenario,
                phase,
                timing,
                destinations,
                files,
                symlinks,
                feature_files,
                payload_files,
                payload_bytes,
                blocks,
                seed,
                require_clean_label,
            } = *args;
            let before = import("baseline", baseline, baseline_provenance)?;
            let after = if let (Some(binary), Some(provenance)) = (candidate, candidate_provenance) {
                import("candidate", binary, provenance)?
            } else {
                let mut same = before.clone();
                same.label = "candidate".into();
                same
            };
            let config = Config {
                scenario,
                phase,
                timing,
                destinations,
                files,
                symlinks,
                feature_files,
                payload_files,
                payload_bytes,
                require_clean_label,
            };
            let mut work = workload::Reseed::new(&output, config, instrumentation::native(&jj)?, instrumentation::native(&git)?)?;
            if !execution::balanced(&output, &[before, after], blocks, seed, &mut work)? {
                return Err("native JW campaign failed; all attempts retained".into());
            }
            println!(
                "{}",
                json!({"result":output.join("summary.json"),"claims":"plumbing only; no speedup inferred"})
            );
        }
    }
    Ok(())
}
