use std::process::ExitCode;

mod cli;
mod manager;
#[cfg_attr(not(unix), path = "refill_report_unsupported.rs")]
mod refill_report;
mod shell;

fn main() -> ExitCode {
    match cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("jw: {err:#}");
            ExitCode::FAILURE
        }
    }
}
