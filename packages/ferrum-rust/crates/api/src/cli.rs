use std::io::{Read, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::cdml::{inspect_cdml, rewrite_cdml, validate_cdml, verify_cdml_rewrite};
use crate::cdsvg::extract_cdsvg;
use crate::errors::CliError;
use crate::streams::{read_input, write_report, write_rewrite};

/// Ferrum command-line arguments.
#[derive(Debug, Parser)]
#[command(name = "ferrum", version, about = "Ferrum chemical document tools")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inspect, validate, or structurally rewrite a CDML document.
    Cdml {
        #[command(subcommand)]
        command: CdmlCommand,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ReportFormat {
    /// Versioned machine-readable JSON, the default for pipeline commands.
    Json,
    /// Deterministic human-readable output; it is not a parsing contract.
    Text,
}

#[derive(Debug, Subcommand)]
enum CdmlCommand {
    /// Print a core-projected CDML summary.
    Inspect {
        /// Input CDML path, or `-` for standard input.
        input: PathBuf,
        /// Report representation. JSON is the stable preview contract.
        #[arg(long, value_enum, default_value_t = ReportFormat::Json)]
        format: ReportFormat,
    },
    /// Validate retained CDML structure, optionally requiring core molecule facts.
    Validate {
        /// Input CDML path, or `-` for standard input.
        input: PathBuf,
        /// Also require Ferrum's current typed/core molecule projection.
        #[arg(long)]
        typed: bool,
        /// Report representation. JSON is the stable preview contract.
        #[arg(long, value_enum, default_value_t = ReportFormat::Json)]
        format: ReportFormat,
    },
    /// Parse and structurally re-emit a CDML document, or check that preservation.
    Rewrite {
        /// Input CDML path, or `-` for standard input.
        input: PathBuf,
        /// Output CDML path, or `-` for standard output.
        #[arg(
            short,
            long,
            required_unless_present = "check",
            conflicts_with = "check"
        )]
        output: Option<PathBuf>,
        /// Verify the serialize/reparse structural contract without writing output.
        #[arg(long)]
        check: bool,
    },
    /// Extract the canonical CDML payload from decoded CD-SVG for atomic publication.
    ExtractCdsvg {
        /// Input CD-SVG path, or `-` for standard input.
        input: PathBuf,
        /// Output CDML path, or `-` for standard output.
        #[arg(short, long)]
        output: PathBuf,
    },
}

pub(crate) fn run(cli: Cli, stdin: &mut dyn Read, stdout: &mut dyn Write) -> Result<(), CliError> {
    match cli.command {
        Command::Cdml { command } => match command {
            CdmlCommand::Inspect { input, format } => {
                let (source, label) = read_input(&input, stdin)?;
                let report = inspect_cdml(&source).map_err(|source| CliError::Cdml {
                    input: label,
                    source,
                })?;
                write_report(render_inspection(&report, format)?.as_bytes(), stdout)
            }
            CdmlCommand::Validate {
                input,
                typed,
                format,
            } => {
                let (source, label) = read_input(&input, stdin)?;
                let report = validate_cdml(&source, typed).map_err(|source| CliError::Cdml {
                    input: label,
                    source,
                })?;
                write_report(render_validation(&report, format)?.as_bytes(), stdout)
            }
            CdmlCommand::Rewrite {
                input,
                output,
                check,
            } => {
                let (source, label) = read_input(&input, stdin)?;
                if check {
                    let report = verify_cdml_rewrite(&source).map_err(|source| CliError::Cdml {
                        input: label,
                        source,
                    })?;
                    write_report(&json_line(&report)?, stdout)
                } else {
                    let rewritten = rewrite_cdml(&source).map_err(|source| CliError::Cdml {
                        input: label,
                        source,
                    })?;
                    let output = output.expect("clap requires output when --check is absent");
                    write_rewrite(&output, &rewritten, stdout)
                }
            }
            CdmlCommand::ExtractCdsvg { input, output } => {
                let (source, label) = read_input(&input, stdin)?;
                let extracted = extract_cdsvg(&source).map_err(|source| CliError::Cdsvg {
                    input: label,
                    source,
                })?;
                write_rewrite(&output, &extracted, stdout)
            }
        },
    }
}

fn render_inspection(
    report: &crate::CdmlInspection,
    format: ReportFormat,
) -> Result<String, CliError> {
    match format {
        ReportFormat::Json => String::from_utf8(json_line(report)?).map_err(|error| {
            CliError::Json(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                error,
            )))
        }),
        ReportFormat::Text => Ok(format!(
            "CDML inspection ({})\npersistent IDs: {}\ntop-level records: {}\ndiagnostics: {}\nmolecules: {}\n",
            report.schema,
            report.persistent_id_count,
            report.top_level_record_count,
            report.diagnostic_count,
            report.molecules.len(),
        )),
    }
}

fn render_validation(
    report: &crate::CdmlValidation,
    format: ReportFormat,
) -> Result<String, CliError> {
    match format {
        ReportFormat::Json => String::from_utf8(json_line(report)?).map_err(|error| {
            CliError::Json(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                error,
            )))
        }),
        ReportFormat::Text => Ok(format!(
            "CDML validation ({})\nvalid: {}\nlevel: {}\npersistent IDs: {}\ntop-level records: {}\ndiagnostics: {}\n",
            report.schema,
            report.valid,
            report.level,
            report.persistent_id_count,
            report.top_level_record_count,
            report.diagnostic_count,
        )),
    }
}

fn json_line<T: serde::Serialize>(report: &T) -> Result<Vec<u8>, CliError> {
    let mut json = serde_json::to_vec(report)?;
    json.push(b'\n');
    Ok(json)
}
