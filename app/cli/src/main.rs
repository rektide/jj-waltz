use std::process::ExitCode;

mod cli;
mod manager;
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
