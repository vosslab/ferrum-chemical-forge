use std::io::{Read, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use ferrum_chemistry::{InchiMode, MOLBLOCK_MAX_INPUT_BYTES, MolblockVersion, SDF_MAX_INPUT_BYTES};

use crate::cdml::{inspect_cdml, rewrite_cdml, validate_cdml, verify_cdml_rewrite};
use crate::cdsvg::extract_cdsvg;
use crate::errors::CliError;
use crate::inchi_codec::{inchi_from_smiles, inspect_inchi};
use crate::molblock_export::molblock_from_smiles;
use crate::molblock_inspection::inspect_molblock;
use crate::molecule_coordinate_cli::generate_molecule_coordinates_cdml;
use crate::render_observation_cli::render_observation_json;
use crate::sdf_export::sdf_from_smiles;
use crate::sdf_inspection::inspect_sdf;
use crate::smarts_export::smarts_from_smiles;
use crate::smiles_inspection::inspect_smiles;
use crate::streams::{read_input, read_input_bounded, write_report, write_rewrite};

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
    /// Inspect one SMILES value through an explicitly named native adapter.
    Smiles {
        #[command(subcommand)]
        command: SmilesCommand,
    },
    /// Inspect bounded SDF through an explicitly named native adapter.
    Sdf {
        #[command(subcommand)]
        command: SdfCommand,
    },
    /// Inspect one bounded V2000 or V3000 molblock.
    Molblock {
        #[command(subcommand)]
        command: MolblockCommand,
    },
    /// Inspect one bounded InChI through an explicitly named native adapter.
    Inchi {
        #[command(subcommand)]
        command: InchiCommand,
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
    /// Produce one complete verified render-observation JSON object for a CDML document.
    RenderObservation {
        /// Input CDML path, or `-` for standard input.
        input: PathBuf,
    },
    /// Regenerate one existing durable molecule through an explicitly named adapter.
    GenerateCoordinates {
        /// Absolute regular ABI-4 adapter library path.
        #[arg(long)]
        adapter: PathBuf,
        /// Exact authored molecule `id` value; Ferrum never guesses a target.
        #[arg(long)]
        molecule_id: String,
        /// Input CDML path, or `-` for standard input.
        input: PathBuf,
        /// Output CDML path, or `-` for standard output.
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum SmilesCommand {
    /// Inspect one SMILES value with the explicitly named ABI-4 adapter library.
    Inspect {
        /// Absolute regular ABI-4 adapter library path.
        #[arg(long)]
        adapter: PathBuf,
        /// SMILES input to parse and inspect.
        smiles: String,
    },
    /// Export one parsed SMILES molecule using the selected adapter's SMARTS writer.
    ToSmarts {
        /// Absolute regular ABI-4 adapter library path.
        #[arg(long)]
        adapter: PathBuf,
        /// SMILES input whose complete graph will be exported.
        smiles: String,
    },
    /// Export one parsed SMILES molecule as explicit V2000 or V3000 molfile text.
    ToMolblock {
        /// Absolute regular ABI-4 adapter library path.
        #[arg(long)]
        adapter: PathBuf,
        /// Required molfile syntax; Ferrum never silently promotes V2000.
        #[arg(long, value_enum)]
        format: MolblockFormat,
        /// SMILES input whose complete graph and coordinates will be exported.
        smiles: String,
    },
    /// Export one parsed SMILES molecule as one ordered SDF record.
    ToSdf {
        /// Absolute regular ABI-4 adapter library path.
        #[arg(long)]
        adapter: PathBuf,
        /// Required molfile syntax inside the SDF record.
        #[arg(long, value_enum)]
        format: MolblockFormat,
        /// Optional first-line record title.
        #[arg(long, default_value = "")]
        title: String,
        /// Ordered text property in NAME=VALUE form; may be repeated.
        #[arg(long = "property")]
        properties: Vec<String>,
        /// SMILES input whose complete graph and coordinates will be exported.
        smiles: String,
    },
    /// Export one parsed SMILES molecule as standard or fixed-hydrogen InChI.
    ToInchi {
        /// Absolute regular ABI-4 adapter library path.
        #[arg(long)]
        adapter: PathBuf,
        /// Request non-standard fixed-hydrogen output instead of standard InChI.
        #[arg(long)]
        fixed_hydrogen: bool,
        /// SMILES input whose complete graph will be exported.
        smiles: String,
    },
}

#[derive(Debug, Subcommand)]
enum SdfCommand {
    /// Inspect complete ordered records with the explicitly named ABI-4 adapter.
    Inspect {
        /// Absolute regular ABI-4 adapter library path.
        #[arg(long)]
        adapter: PathBuf,
        /// Input SDF path, or `-` for standard input.
        input: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum MolblockCommand {
    /// Inspect one molecule with the explicitly named ABI-4 adapter.
    Inspect {
        /// Absolute regular ABI-4 adapter library path.
        #[arg(long)]
        adapter: PathBuf,
        /// Input molblock path, or `-` for standard input.
        input: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum InchiCommand {
    /// Inspect one InChI and derive its official InChIKey.
    Inspect {
        /// Absolute regular ABI-4 adapter library path.
        #[arg(long)]
        adapter: PathBuf,
        /// InChI line to parse and inspect.
        inchi: String,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum MolblockFormat {
    V2000,
    V3000,
}

/// Execute accepted CLI arguments with caller-owned standard streams.
pub fn run(cli: Cli, stdin: &mut dyn Read, stdout: &mut dyn Write) -> Result<(), CliError> {
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
            CdmlCommand::RenderObservation { input } => {
                let (source, label) = read_input(&input, stdin)?;
                let report = render_observation_json(&source).map_err(|source| {
                    CliError::RenderObservation {
                        input: label,
                        source,
                    }
                })?;
                write_report(format!("{report}\n").as_bytes(), stdout)
            }
            CdmlCommand::GenerateCoordinates {
                adapter,
                molecule_id,
                input,
                output,
            } => {
                let (source, label) = read_input(&input, stdin)?;
                let generated = generate_molecule_coordinates_cdml(&adapter, &source, &molecule_id)
                    .map_err(|source| CliError::MoleculeCoordinates {
                        input: label,
                        source,
                    })?;
                write_rewrite(&output, &generated, stdout)
            }
        },
        Command::Smiles { command } => match command {
            SmilesCommand::Inspect { adapter, smiles } => {
                let report =
                    inspect_smiles(&adapter, &smiles).map_err(CliError::SmilesInspection)?;
                write_report(&json_line(&report)?, stdout)
            }
            SmilesCommand::ToSmarts { adapter, smiles } => {
                let smarts = smarts_from_smiles(&adapter, &smiles)?;
                write_report(format!("{smarts}\n").as_bytes(), stdout)
            }
            SmilesCommand::ToMolblock {
                adapter,
                format,
                smiles,
            } => {
                let version = match format {
                    MolblockFormat::V2000 => MolblockVersion::V2000,
                    MolblockFormat::V3000 => MolblockVersion::V3000,
                };
                let molblock = molblock_from_smiles(&adapter, &smiles, version)?;
                write_report(molblock.as_bytes(), stdout)
            }
            SmilesCommand::ToSdf {
                adapter,
                format,
                title,
                properties,
                smiles,
            } => {
                let version = match format {
                    MolblockFormat::V2000 => MolblockVersion::V2000,
                    MolblockFormat::V3000 => MolblockVersion::V3000,
                };
                let sdf = sdf_from_smiles(&adapter, &smiles, &title, &properties, version)?;
                write_report(sdf.as_bytes(), stdout)
            }
            SmilesCommand::ToInchi {
                adapter,
                fixed_hydrogen,
                smiles,
            } => {
                let mode = if fixed_hydrogen {
                    InchiMode::FixedHydrogen
                } else {
                    InchiMode::Standard
                };
                let inchi = inchi_from_smiles(&adapter, &smiles, mode)?;
                write_report(format!("{inchi}\n").as_bytes(), stdout)
            }
        },
        Command::Sdf { command } => match command {
            SdfCommand::Inspect { adapter, input } => {
                let (source, label) = read_input_bounded(&input, stdin, SDF_MAX_INPUT_BYTES)?;
                let report =
                    inspect_sdf(&adapter, &source).map_err(|source| CliError::SdfInspection {
                        input: label,
                        source,
                    })?;
                write_report(&json_line(&report)?, stdout)
            }
        },
        Command::Molblock { command } => match command {
            MolblockCommand::Inspect { adapter, input } => {
                let (source, label) = read_input_bounded(&input, stdin, MOLBLOCK_MAX_INPUT_BYTES)?;
                let report = inspect_molblock(&adapter, &source).map_err(|source| {
                    CliError::MolblockInspection {
                        input: label,
                        source,
                    }
                })?;
                write_report(&json_line(&report)?, stdout)
            }
        },
        Command::Inchi { command } => match command {
            InchiCommand::Inspect { adapter, inchi } => {
                let report = inspect_inchi(&adapter, &inchi)?;
                write_report(&json_line(&report)?, stdout)
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
