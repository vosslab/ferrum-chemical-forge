use std::io;
use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let cli = ferrum_api::Cli::parse();
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    match ferrum_api::run(cli, &mut stdin, &mut stdout, &mut stderr) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            drop(stderr);
            eprintln!("ferrum: {error}");
            ExitCode::FAILURE
        }
    }
}
