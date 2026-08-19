use std::io::{Read, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use ferrum_document::InterchangeFormatV1;

use crate::cli::protocol::{run_protocol, write_protocol_schema};
use crate::cli::verbs::{convert, coords, inspect, render, rewrite, validate};
use crate::transport::errors::CliError;

pub(crate) mod engine_bundle;
pub(crate) mod protocol;
pub(crate) mod verbs;

/// Ferrum command-line arguments.
#[derive(Debug, Parser)]
#[command(name = "ferrum", version, about = "Ferrum chemical document tools")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inspect one CDML document and print its semantic report.
    #[command(after_help = "Example:\n  ferrum inspect drawing.cdml")]
    Inspect {
        /// Input CDML path, or `-` for standard input.
        document: PathBuf,
        /// Explicit input format.
        #[arg(long = "from", value_enum, default_value_t = DocumentInputFormat::Cdml)]
        input_format: DocumentInputFormat,
        /// Emit the complete operation-protocol envelope.
        #[arg(long)]
        json: bool,
    },
    /// Validate one CDML document.
    #[command(after_help = "Example:\n  ferrum validate drawing.cdml --level typed")]
    Validate {
        /// Input CDML path, or `-` for standard input.
        document: PathBuf,
        /// Explicit input format.
        #[arg(long = "from", value_enum, default_value_t = DocumentInputFormat::Cdml)]
        input_format: DocumentInputFormat,
        /// Structural or typed validation.
        #[arg(long, value_enum, default_value_t = ValidationLevel::Typed)]
        level: ValidationLevel,
        /// Emit the complete operation-protocol envelope.
        #[arg(long)]
        json: bool,
    },
    /// Rewrite one CDML document structurally.
    #[command(after_help = "Example:\n  ferrum rewrite drawing.cdml -o cleaned.cdml")]
    Rewrite {
        /// Input CDML path, or `-` for standard input.
        document: PathBuf,
        /// Output CDML path, or `-` for standard output.
        #[arg(short, long, conflicts_with = "json")]
        output: Option<PathBuf>,
        /// Explicit input format.
        #[arg(long = "from", value_enum, default_value_t = DocumentInputFormat::Cdml)]
        input_format: DocumentInputFormat,
        /// Explicit output format.
        #[arg(long = "to", value_enum, default_value_t = DocumentOutputFormat::Cdml)]
        output_format: DocumentOutputFormat,
        /// Emit the complete operation-protocol envelope.
        #[arg(long)]
        json: bool,
    },
    /// Render one complete CDML document as SVG, PDF, or PNG.
    #[command(after_help = "Example:\n  ferrum render drawing.cdml -o drawing.svg")]
    Render {
        /// Input CDML path, or `-` for standard input.
        document: PathBuf,
        /// Artifact destination, or `-` for standard output.
        #[arg(short, long, conflicts_with = "json")]
        output: Option<PathBuf>,
        /// Explicit input format.
        #[arg(long = "from", value_enum, default_value_t = DocumentInputFormat::Cdml)]
        input_format: DocumentInputFormat,
        /// Artifact format; otherwise inferred from the output extension.
        #[arg(long = "to", value_enum)]
        output_format: Option<ArtifactOutputFormat>,
        /// Emit the complete operation-protocol envelope.
        #[arg(long)]
        json: bool,
    },
    /// Convert one molecular interchange source through Ferrum's native chemistry engine.
    #[command(after_help = "Example:\n  ferrum convert aspirin.smi --to sdf_v2000 -o aspirin.sdf")]
    Convert {
        /// Input molecular interchange path, or `-` for standard input.
        input: PathBuf,
        /// Output path, or `-` for standard output.
        #[arg(short, long, conflicts_with = "json")]
        output: Option<PathBuf>,
        /// Source syntax; otherwise inferred from .smi, .inchi, .mol, .sdf, or .cdml.
        #[arg(long = "from", value_enum)]
        input_format: Option<InterchangeFormat>,
        /// Target syntax using one exact closed protocol format name.
        #[arg(long = "to", value_enum)]
        output_format: InterchangeFormat,
        /// Emit the complete operation-protocol envelope.
        #[arg(long)]
        json: bool,
    },
    /// Regenerate all direct molecule coordinates in one CDML document.
    #[command(after_help = "Example:\n  ferrum coords drawing.cdml -o laid-out.cdml")]
    Coords {
        /// Input CDML path, or `-` for standard input.
        document: PathBuf,
        /// Output CDML path, or `-` for standard output.
        #[arg(short, long, conflicts_with = "json")]
        output: Option<PathBuf>,
        /// Emit the complete operation-protocol envelope.
        #[arg(long)]
        json: bool,
    },
    /// Execute one frozen Ferrum operation-protocol V1 request.
    Protocol {
        #[command(subcommand)]
        command: ProtocolCommand,
    },
    /// Install or inspect the explicitly provisioned native chemistry engine bundle.
    Engine {
        #[command(subcommand)]
        command: EngineCommand,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DocumentInputFormat {
    Cdml,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DocumentOutputFormat {
    Cdml,
}

/// Closed molecular interchange syntax vocabulary used by `ferrum convert`.
#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "snake_case")]
pub(crate) enum InterchangeFormat {
    Smiles,
    InchiStandard,
    #[value(name = "inchi_fixed_h")]
    InchiFixedHydrogen,
    MolblockV2000,
    MolblockV3000,
    SdfV2000,
    SdfV3000,
    Cdml,
}

impl From<InterchangeFormat> for InterchangeFormatV1 {
    fn from(value: InterchangeFormat) -> Self {
        match value {
            InterchangeFormat::Smiles => Self::Smiles,
            InterchangeFormat::InchiStandard => Self::InchiStandard,
            InterchangeFormat::InchiFixedHydrogen => Self::InchiFixedHydrogen,
            InterchangeFormat::MolblockV2000 => Self::MolblockV2000,
            InterchangeFormat::MolblockV3000 => Self::MolblockV3000,
            InterchangeFormat::SdfV2000 => Self::SdfV2000,
            InterchangeFormat::SdfV3000 => Self::SdfV3000,
            InterchangeFormat::Cdml => Self::Cdml,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum ValidationLevel {
    Structural,
    Typed,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum ArtifactOutputFormat {
    Svg,
    Pdf,
    Png,
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

#[derive(Debug, Subcommand)]
enum EngineCommand {
    /// Validate and install one explicit Ferrum engine bundle directory.
    #[command(
        after_help = "The bundle location is used only for this install; Ferrum never searches for adapters."
    )]
    Install { bundle: PathBuf },
    /// Report whether the fixed application-data root has a valid active bundle.
    Status,
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
        Command::Inspect {
            document,
            input_format: _,
            json,
        } => Ok(inspect::run(&document, json, stdin, stdout, stderr)?),
        Command::Validate {
            document,
            input_format: _,
            level,
            json,
        } => Ok(validate::run(
            &document, level, json, stdin, stdout, stderr,
        )?),
        Command::Rewrite {
            document,
            output,
            input_format: _,
            output_format: _,
            json,
        } => Ok(rewrite::run(
            &document,
            output.as_deref(),
            json,
            stdin,
            stdout,
            stderr,
        )?),
        Command::Render {
            document,
            output,
            input_format: _,
            output_format,
            json,
        } => Ok(render::run(
            &document,
            output.as_deref(),
            output_format,
            json,
            stdin,
            stdout,
            stderr,
        )?),
        Command::Convert {
            input,
            output,
            input_format,
            output_format,
            json,
        } => Ok(convert::run(
            convert::ConvertOptions {
                input,
                output,
                input_format,
                output_format,
                json,
            },
            stdin,
            stdout,
            stderr,
        )?),
        Command::Coords {
            document,
            output,
            json,
        } => Ok(coords::run(
            &document,
            output.as_deref(),
            json,
            stdin,
            stdout,
            stderr,
        )?),
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
        Command::Engine { command } => match command {
            EngineCommand::Install { bundle } => Ok(engine_bundle::install_bundle(&bundle)?),
            EngineCommand::Status => Ok(engine_bundle::write_status(stdout)?),
        },
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::{Cli, run};

    const CDML: &str = "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom></molecule></cdml>";

    fn run_from_stdin(arguments: &[&str]) -> (Vec<u8>, Vec<u8>) {
        let cli = Cli::try_parse_from(arguments).expect("verb arguments should parse");
        let mut stdin = CDML.as_bytes();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        run(cli, &mut stdin, &mut stdout, &mut stderr).expect("verb should complete");
        (stdout, stderr)
    }

    #[test]
    fn inspect_reads_cdml_from_standard_input() {
        let (stdout, stderr) = run_from_stdin(&["ferrum", "inspect", "-"]);
        let report: serde_json::Value =
            serde_json::from_slice(&stdout).expect("inspection report should be JSON");
        assert_eq!(report["schema"], "ferrum-cdml-inspection-v1");
        assert!(stderr.is_empty());
    }

    #[test]
    fn validate_exposes_the_typed_protocol_operation() {
        let (stdout, stderr) = run_from_stdin(&["ferrum", "validate", "-", "--level", "typed"]);
        let report: serde_json::Value =
            serde_json::from_slice(&stdout).expect("validation report should be JSON");
        assert_eq!(report["schema"], "ferrum-cdml-validation-v1");
        assert!(stderr.is_empty());
    }

    #[test]
    fn rewrite_emits_cdml_to_standard_output() {
        let (stdout, stderr) = run_from_stdin(&["ferrum", "rewrite", "-"]);
        let document = String::from_utf8(stdout).expect("rewritten CDML should be UTF-8");
        assert!(document.starts_with("<cdml"));
        assert!(stderr.is_empty());
    }

    #[test]
    fn render_emits_the_selected_artifact_to_standard_output() {
        let (stdout, stderr) = run_from_stdin(&["ferrum", "render", "-", "--to", "svg"]);
        assert!(stdout.starts_with(b"<svg"));
        assert!(stderr.is_empty());
    }

    #[test]
    fn engine_verbs_complete_through_the_protocol_envelope() {
        for arguments in [
            vec![
                "ferrum", "convert", "-", "--from", "smiles", "--to", "smiles", "--json",
            ],
            vec!["ferrum", "coords", "-", "--json"],
        ] {
            let cli = Cli::try_parse_from(arguments).expect("engine verb arguments should parse");
            let mut stdin = CDML.as_bytes();
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            run(cli, &mut stdin, &mut stdout, &mut stderr)
                .expect("typed refusal should complete the CLI operation");
            let envelope: serde_json::Value =
                serde_json::from_slice(&stdout).expect("engine verb should return an envelope");
            assert!(
                envelope["schema"].is_string(),
                "the protocol envelope must retain its schema discriminator"
            );
            assert!(stderr.is_empty());
        }
    }

    #[test]
    fn every_human_verb_help_includes_a_worked_example() {
        let command = Cli::command();
        for (verb, example) in [
            ("inspect", "ferrum inspect drawing.cdml"),
            ("validate", "ferrum validate drawing.cdml --level typed"),
            ("rewrite", "ferrum rewrite drawing.cdml -o cleaned.cdml"),
            ("render", "ferrum render drawing.cdml -o drawing.svg"),
            (
                "convert",
                "ferrum convert aspirin.smi --to sdf_v2000 -o aspirin.sdf",
            ),
            ("coords", "ferrum coords drawing.cdml -o laid-out.cdml"),
        ] {
            let help = command
                .find_subcommand(verb)
                .expect("human verb should exist")
                .clone()
                .render_long_help()
                .to_string();
            assert!(
                help.contains(example),
                "{verb} help should teach one example"
            );
        }
    }
}
