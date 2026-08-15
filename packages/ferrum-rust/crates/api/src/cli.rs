use std::io::{Read, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::errors::CliError;
use crate::protocol_cli::{run_protocol, write_protocol_schema};

/// Ferrum command-line arguments.
#[derive(Debug, Parser)]
#[command(name = "ferrum", version, about = "Ferrum operation protocol tools")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Execute one frozen Ferrum operation-protocol V1 request.
    Protocol {
        #[command(subcommand)]
        command: ProtocolCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ProtocolCommand {
    /// Print the generated operation-protocol V1 schema.
    Schema,
    /// Execute one UTF-8 JSON operation-protocol V1 request.
    Run {
        /// Input JSON request path, or `-` for standard input.
        input: PathBuf,
        /// Explicit JSON response destination, published safely.
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
}

fn output_file_path(value: &str) -> Result<PathBuf, String> {
    if value == "-" {
        return Err(
            "--output must name a file destination; omit it for standard output".to_owned(),
        );
    }
    Ok(PathBuf::from(value))
}

/// Execute accepted CLI arguments with caller-owned standard streams.
pub fn run(
    cli: Cli,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), CliError> {
    match cli.command {
        Command::Protocol { command } => match command {
            ProtocolCommand::Schema => Ok(write_protocol_schema(stdout)?),
            ProtocolCommand::Run { input, output } => Ok(run_protocol(
                &input,
                output.as_deref(),
                stdin,
                stdout,
                stderr,
            )?),
        },
    }
}
