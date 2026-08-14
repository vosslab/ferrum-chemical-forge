use std::io::{Read, Write};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Parser, Subcommand, ValueEnum};
use ferrum_chemistry::{InchiMode, MOLBLOCK_MAX_INPUT_BYTES, MolblockVersion, SDF_MAX_INPUT_BYTES};
use ferrum_render::{PngBackgroundV1, Rgb24};

use crate::canonical_smiles::canonical_smiles_from_smiles;
use crate::cdml::{inspect_cdml, rewrite_cdml, validate_cdml, verify_cdml_rewrite};
use crate::cdsvg::extract_cdsvg;
use crate::document_render_cli::{
    PdfCliRenderPolicyV1, PngCliRenderPolicyV1, render_pdf, render_png, render_svg,
};
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
use crate::{
    LOCAL_PDF_COMPLETED_BYTES_V1, LOCAL_PDF_DRAW_PATH_COMMANDS_V1, LOCAL_PDF_PLAN_ITEMS_V1,
    LOCAL_PNG_ENCODED_BYTES_V1, LOCAL_PNG_RAW_RGBA_BYTES_V1, LOCAL_SVG_COMPLETED_BYTES_V1,
};

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
    /// Render an admitted CDML document through one native artifact backend.
    Render {
        #[command(subcommand)]
        command: CdmlRenderCommand,
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
enum CdmlRenderCommand {
    /// Render one complete document as structurally validated SVG.
    Svg {
        /// Input uncompressed CDML path, or `-` for bounded standard input.
        input: PathBuf,
        /// Output SVG path, or `-` for standard output.
        #[arg(short, long)]
        output: PathBuf,
        /// Maximum completed SVG bytes returned by the native backend.
        #[arg(long, default_value_t = LOCAL_SVG_COMPLETED_BYTES_V1)]
        max_output_bytes: usize,
    },
    /// Render one complete document as a one-page native vector PDF.
    Pdf {
        /// Input uncompressed CDML path, or `-` for bounded standard input.
        input: PathBuf,
        /// Output PDF path, or `-` for standard output.
        #[arg(short, long)]
        output: PathBuf,
        /// Maximum completed PDF bytes returned by the native backend.
        #[arg(long, default_value_t = LOCAL_PDF_COMPLETED_BYTES_V1)]
        max_output_bytes: usize,
        /// Maximum counted plan traversal items admitted before PDF allocation.
        #[arg(long, default_value_t = LOCAL_PDF_PLAN_ITEMS_V1)]
        max_plan_items: usize,
        /// Maximum lowered vector path commands admitted before PDF allocation.
        #[arg(long, default_value_t = LOCAL_PDF_DRAW_PATH_COMMANDS_V1)]
        max_path_commands: usize,
    },
    /// Render one complete document as a caller-sized native raster PNG.
    Png {
        /// Input uncompressed CDML path, or `-` for bounded standard input.
        input: PathBuf,
        /// Output PNG path, or `-` for standard output.
        #[arg(short, long)]
        output: PathBuf,
        /// Exact nonzero output width in device pixels.
        #[arg(long)]
        width: NonZeroU32,
        /// Exact nonzero output height in device pixels.
        #[arg(long)]
        height: NonZeroU32,
        /// `transparent` or a six-digit RGB canvas color without `#`.
        #[arg(long, default_value = "ffffff")]
        background: PngBackgroundArgument,
        /// Maximum raw RGBA bytes admitted before pixmap allocation.
        #[arg(long, default_value_t = LOCAL_PNG_RAW_RGBA_BYTES_V1)]
        max_raw_bytes: usize,
        /// Maximum completed encoded PNG bytes.
        #[arg(long, default_value_t = LOCAL_PNG_ENCODED_BYTES_V1)]
        max_output_bytes: usize,
    },
}

#[derive(Clone, Debug)]
enum PngBackgroundArgument {
    Transparent,
    Opaque(Rgb24),
}

impl PngBackgroundArgument {
    fn into_render(self) -> PngBackgroundV1 {
        match self {
            Self::Transparent => PngBackgroundV1::Transparent,
            Self::Opaque(color) => PngBackgroundV1::Opaque(color),
        }
    }
}

impl FromStr for PngBackgroundArgument {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "transparent" {
            return Ok(Self::Transparent);
        }
        let color = value.to_ascii_lowercase();
        Rgb24::new(color).map(Self::Opaque).map_err(|_| {
            "PNG background must be `transparent` or six hexadecimal digits without `#`".to_owned()
        })
    }
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
    /// Parse and re-emit one canonical-isomeric SMILES value.
    Canonicalize {
        /// Absolute regular ABI-4 adapter library path.
        #[arg(long)]
        adapter: PathBuf,
        /// SMILES input whose complete graph will be serialized canonically.
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
pub fn run(
    cli: Cli,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), CliError> {
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
            CdmlCommand::Render { command } => match command {
                CdmlRenderCommand::Svg {
                    input,
                    output,
                    max_output_bytes,
                } => render_svg(&input, &output, max_output_bytes, stdin, stdout, stderr),
                CdmlRenderCommand::Pdf {
                    input,
                    output,
                    max_output_bytes,
                    max_plan_items,
                    max_path_commands,
                } => render_pdf(
                    &input,
                    &output,
                    PdfCliRenderPolicyV1 {
                        max_output_bytes,
                        max_plan_items,
                        max_path_commands,
                    },
                    stdin,
                    stdout,
                    stderr,
                ),
                CdmlRenderCommand::Png {
                    input,
                    output,
                    width,
                    height,
                    background,
                    max_raw_bytes,
                    max_output_bytes,
                } => render_png(
                    &input,
                    &output,
                    PngCliRenderPolicyV1 {
                        width,
                        height,
                        background: background.into_render(),
                        max_raw_rgba_bytes: max_raw_bytes,
                        max_output_bytes,
                    },
                    stdin,
                    stdout,
                    stderr,
                ),
            },
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
            SmilesCommand::Canonicalize { adapter, smiles } => {
                let canonical = canonical_smiles_from_smiles(&adapter, &smiles)?;
                write_report(format!("{canonical}\n").as_bytes(), stdout)
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
