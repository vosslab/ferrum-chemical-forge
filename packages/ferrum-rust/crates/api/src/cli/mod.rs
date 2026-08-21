use std::io::{Read, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use ferrum_document::InterchangeFormatV1;

use crate::cli::protocol::{run_protocol, write_protocol_schema};
use crate::cli::verbs::{convert, coords, inspect, open, render, rewrite, validate};
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
    /// Open one CML source as a newly created Ferrum document.
    #[command(
        after_help = "Example:\n  ferrum open molecule.cml --format cml --output molecule.cdml"
    )]
    Open {
        /// CML input path, or `-` for standard input.
        input: PathBuf,
        /// Input format. Required for standard input; .cml is inferred for named files.
        #[arg(long, value_enum)]
        format: Option<CmlOpenInputFormat>,
        /// New CDML file destination.
        #[arg(short, long, value_parser = output_file_path)]
        output: PathBuf,
        /// Emit the complete fixed CML-open response envelope.
        #[arg(long)]
        json: bool,
    },
    /// Execute one frozen Ferrum operation-protocol V1 request.
    Protocol {
        #[command(subcommand)]
        command: ProtocolCommand,
    },
    /// Execute one named, versioned document command through the frozen protocol envelope.
    Document {
        #[command(subcommand)]
        command: DocumentCommand,
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

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CmlOpenInputFormat {
    Cml,
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
enum DocumentCommand {
    /// Execute one named document mutation command.
    Command {
        #[command(subcommand)]
        command: NamedDocumentCommand,
    },
}

#[derive(Debug, Subcommand)]
enum NamedDocumentCommand {
    /// Produce a Rust-owned molecule report through the frozen protocol route.
    #[command(name = "document.molecule.report.v1")]
    DocumentMoleculeReport {
        input: PathBuf,
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    /// Query direct document molecules through the bounded SMARTS protocol route.
    #[command(name = "document.molecule.smarts.query.v1")]
    DocumentMoleculeSmartsQuery {
        input: PathBuf,
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    /// Import CML into a new Rust-owned document through the frozen protocol route.
    #[command(name = "document.molecule.interchange.import.v1")]
    DocumentMoleculeInterchangeImport {
        input: PathBuf,
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    /// List immutable Ferrum-authored template catalog summary facts.
    #[command(name = "catalog.list.v1")]
    CatalogList {
        /// Complete operation-protocol JSON request path, or `-` for standard input.
        input: PathBuf,
        /// Explicit JSON response destination, published safely.
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    /// Insert one catalog template through Rust's renderer-preflighted gesture.
    #[command(name = "catalog.insert.v1")]
    CatalogInsert {
        /// Complete operation-protocol JSON request path, or `-` for standard input.
        input: PathBuf,
        /// Explicit JSON response destination, published safely.
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    /// Create a standard-resolved direct-root presentation vector.
    #[command(name = "presentation.vector.create.v1")]
    PresentationVectorCreate {
        /// Complete operation-protocol JSON request path, or `-` for standard input.
        input: PathBuf,
        /// Explicit JSON response destination, published safely.
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    /// Create one durable reaction aggregate from direct-root selectors.
    #[command(name = "reaction.create.v1")]
    ReactionCreate {
        input: PathBuf,
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    #[command(name = "reaction.list.v1")]
    ReactionList {
        input: PathBuf,
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    #[command(name = "reaction.observe.v1")]
    ReactionObserve {
        input: PathBuf,
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    #[command(name = "reaction.select.v1")]
    ReactionSelect {
        input: PathBuf,
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    /// Replace all members of one selected strict reaction through Rust's lifecycle bridge.
    #[command(name = "reaction.patch-membership.v1")]
    ReactionPatchMembership {
        input: PathBuf,
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    /// Remove only one selected strict reaction definition through Rust's lifecycle bridge.
    #[command(name = "reaction.delete-definition.v1")]
    ReactionDeleteDefinition {
        input: PathBuf,
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    /// Translate one selected strict reaction through Rust's renderer-preflighted gesture.
    #[command(name = "reaction.translate.v1")]
    ReactionTranslate {
        input: PathBuf,
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
        Command::Open {
            input,
            format,
            output,
            json,
        } => {
            if !cml_open_format_is_declared_or_inferred(&input, format) {
                return Err(crate::cli::verbs::VerbCliError::MissingInterchangeInputFormat.into());
            }
            Ok(open::run(&input, &output, json, stdin, stdout, stderr)?)
        }
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
        Command::Document { command } => match command {
            DocumentCommand::Command { command } => match command {
                NamedDocumentCommand::CatalogList { input, output }
                | NamedDocumentCommand::CatalogInsert { input, output }
                | NamedDocumentCommand::PresentationVectorCreate { input, output }
                | NamedDocumentCommand::DocumentMoleculeReport { input, output }
                | NamedDocumentCommand::DocumentMoleculeSmartsQuery { input, output }
                | NamedDocumentCommand::DocumentMoleculeInterchangeImport { input, output } => Ok(
                    run_protocol(&input, output.as_deref(), stdin, stdout, stderr)?,
                ),
                NamedDocumentCommand::ReactionCreate { input, output } => Ok(run_protocol(
                    &input,
                    output.as_deref(),
                    stdin,
                    stdout,
                    stderr,
                )?),
                NamedDocumentCommand::ReactionList { input, output }
                | NamedDocumentCommand::ReactionObserve { input, output }
                | NamedDocumentCommand::ReactionSelect { input, output }
                | NamedDocumentCommand::ReactionPatchMembership { input, output }
                | NamedDocumentCommand::ReactionDeleteDefinition { input, output }
                | NamedDocumentCommand::ReactionTranslate { input, output } => Ok(run_protocol(
                    &input,
                    output.as_deref(),
                    stdin,
                    stdout,
                    stderr,
                )?),
            },
        },
        Command::Engine { command } => match command {
            EngineCommand::Install { bundle } => Ok(engine_bundle::install_bundle(&bundle)?),
            EngineCommand::Status => Ok(engine_bundle::write_status(stdout)?),
        },
    }
}

fn cml_open_format_is_declared_or_inferred(
    input: &std::path::Path,
    format: Option<CmlOpenInputFormat>,
) -> bool {
    format.is_some()
        || input
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|extension| extension.eq_ignore_ascii_case("cml"))
}

/// Execute the parsed named SMARTS CLI route with a controlled typed runtime.
///
/// This test-only seam deliberately accepts the same parsed [`Cli`] that the
/// production entry point receives. It is restricted to the named SMARTS
/// command so a controlled runtime can never become a production fallback.
#[cfg(test)]
fn run_with_runtime_for_test<R: crate::protocol::runtime::ChemistryRuntimeV1>(
    cli: Cli,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    runtime: &R,
) -> Result<(), CliError> {
    run_with_runtime_and_smarts_response_limit_for_test(
        cli,
        stdin,
        stdout,
        stderr,
        runtime,
        crate::protocol::DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1,
    )
}

#[cfg(test)]
fn run_with_runtime_and_smarts_response_limit_for_test<
    R: crate::protocol::runtime::ChemistryRuntimeV1,
>(
    cli: Cli,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    runtime: &R,
    response_limit: usize,
) -> Result<(), CliError> {
    match cli.command {
        Command::Document {
            command:
                DocumentCommand::Command {
                    command: NamedDocumentCommand::DocumentMoleculeSmartsQuery { input, output },
                },
        } => {
            assert!(
                output.is_none(),
                "controlled named SMARTS test does not publish files"
            );
            let _ = stderr;
            if response_limit == crate::protocol::DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1 {
                Ok(protocol::run_protocol_with_runtime_for_test(
                    &input, stdin, stdout, runtime,
                )?)
            } else {
                Ok(
                    protocol::run_protocol_with_runtime_and_smarts_response_limit_for_test(
                        &input,
                        stdin,
                        stdout,
                        runtime,
                        response_limit,
                    )?,
                )
            }
        }
        _ => panic!("controlled runtime is restricted to the named SMARTS CLI route"),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use clap::{CommandFactory, Parser};
    use ferrum_chemistry::{
        ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, SmartsMatchOptions,
        SmartsMatchResult, SmilesMolecule,
    };

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
    fn named_smarts_query_command_routes_one_complete_protocol_envelope() {
        let session = ferrum_document::DocumentSession::load(CDML).expect("fixture loads");
        let snapshot = session.snapshot().expect("fixture snapshots");
        let digest = snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let request = serde_json::json!({
            "schema": "ferrum-operation-request-v1",
            "request_id": "named-smarts-query",
            "operation": {
                "kind": "document.molecule.smarts.query.v1",
                "document": {"cdml": CDML, "expected_revision": 0, "expected_digest_hex": digest},
                "query": {"kind": "smarts", "value": "[#6]"},
                "limits": {"max_matches_per_molecule": 1, "max_total_matches": 1},
            },
        });
        let cli = Cli::try_parse_from([
            "ferrum",
            "document",
            "command",
            "document.molecule.smarts.query.v1",
            "-",
        ])
        .expect("named SMARTS command parses");
        let input = request.to_string();
        let mut stdin = input.as_bytes();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        run(cli, &mut stdin, &mut stdout, &mut stderr).expect("command returns an envelope");
        let envelope: serde_json::Value = serde_json::from_slice(&stdout).expect("JSON envelope");
        assert_eq!(envelope["request_id"], "named-smarts-query");
        assert!(
            envelope["outcome"]["kind"] == "document.molecule.smarts.query.v1"
                || envelope["error"]["category"] == "chemistry_unavailable"
        );
        assert!(stderr.is_empty());
    }

    struct SelectedSmartsEngine {
        queries: RefCell<Vec<String>>,
    }

    impl ChemEngine for SelectedSmartsEngine {
        fn smiles_to_molecule(&self, _: &str) -> Result<SmilesMolecule, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "smiles",
            })
        }

        fn generate_2d_coordinates(&self, _: &MolGraph) -> Result<Coordinates, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "coordinates",
            })
        }

        fn smarts_match(
            &self,
            query: &str,
            target: &MolGraph,
            options: SmartsMatchOptions,
        ) -> Result<SmartsMatchResult, ChemistryError> {
            self.queries.borrow_mut().push(query.to_owned());
            SmartsMatchResult::try_from_rows(target, options, vec![vec![0]], true).map_err(|_| {
                ChemistryError::SmartsMatchUnavailable {
                    reason: ferrum_chemistry::SmartsMatchUnavailableReason::MalformedNativeResponse,
                }
            })
        }

        fn molecule_to_smarts(&self, _: &MolGraph) -> Result<String, ChemistryError> {
            Ok("selected-fixture-smarts".to_owned())
        }

        fn kekulize(&self, _: &MolGraph, _: KekulizeOptions) -> Result<MolGraph, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "kekulize",
            })
        }
    }

    struct SelectedSmartsRuntime(SelectedSmartsEngine);

    impl crate::protocol::runtime::ChemistryRuntimeV1 for SelectedSmartsRuntime {
        fn with_engine<T>(
            &self,
            operation: impl FnOnce(
                &dyn ChemEngine,
            )
                -> Result<T, crate::protocol::runtime::ChemistryRuntimeErrorV1>,
        ) -> Result<T, crate::protocol::runtime::ChemistryRuntimeErrorV1> {
            operation(&self.0)
        }
    }

    #[test]
    fn named_smarts_protocol_lowers_selected_molecules_and_emits_bounded_facts() {
        let session = ferrum_document::DocumentSession::load(CDML).expect("fixture loads");
        let snapshot = session.snapshot().expect("fixture snapshots");
        let selected = session
            .observe(0)
            .expect("fixture observation")
            .projection()
            .molecules()[0]
            .id()
            .expect("fixture molecule has a durable identity")
            .as_str()
            .to_owned();
        assert_ne!(
            selected, "m",
            "the named protocol must receive the durable direct-root selector"
        );
        let digest = snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let request = serde_json::json!({
            "schema": "ferrum-operation-request-v1",
            "request_id": "named-selected-smarts-query",
            "operation": {
                "kind": "document.molecule.smarts.query.v1",
                "document": {"cdml": CDML, "expected_revision": 0, "expected_digest_hex": digest},
                "query": {"kind": "selected_molecule", "molecule_id": selected},
                "limits": {"max_matches_per_molecule": 1, "max_total_matches": 1},
            },
        });
        let runtime = SelectedSmartsRuntime(SelectedSmartsEngine {
            queries: RefCell::new(Vec::new()),
        });
        let input = request.to_string();
        let mut stdin = input.as_bytes();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let cli = Cli::try_parse_from([
            "ferrum",
            "document",
            "command",
            "document.molecule.smarts.query.v1",
            "-",
        ])
        .expect("selected SMARTS named command parses");
        super::run_with_runtime_for_test(cli, &mut stdin, &mut stdout, &mut stderr, &runtime)
            .expect("named CLI command completes with controlled chemistry");
        let envelope: serde_json::Value = serde_json::from_slice(&stdout).expect("JSON envelope");
        assert_eq!(envelope["request_id"], "named-selected-smarts-query");
        assert_eq!(
            envelope["outcome"]["kind"], "document.molecule.smarts.query.v1",
            "controlled selected SMARTS protocol refused: {envelope:?}"
        );
        assert_eq!(
            envelope["outcome"]["query"],
            serde_json::json!({
                "schema": "ferrum-document-molecule-smarts-query-v1",
                "traversal": {"kind": "complete"},
                "molecules": [{
                    "source_order": 0,
                    "match_count": 1,
                    "completeness": "truncated",
                }],
            })
        );
        assert_eq!(
            runtime.0.queries.borrow().as_slice(),
            ["selected-fixture-smarts"]
        );
        let serialized = String::from_utf8(stdout).expect("CLI response is UTF-8 JSON");
        for forbidden in [
            "selected-fixture-smarts",
            selected.as_str(),
            CDML,
            "record_id",
            "receipt",
            "adapter",
            "rows",
            "graph",
            "position",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "named CLI response leaked private SMARTS state: {forbidden}"
            );
        }
        assert!(stderr.is_empty());
    }

    #[test]
    fn named_smarts_query_response_admission_is_exact_and_redacted_for_raw_and_selected_forms() {
        let session = ferrum_document::DocumentSession::load(CDML).expect("fixture loads");
        let snapshot = session.snapshot().expect("fixture snapshots");
        let digest = snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        for query in [
            serde_json::json!({"kind": "smarts", "value": "FERRUM_PRIVATE_RAW_SMARTS"}),
            serde_json::json!({"kind": "selected_molecule", "molecule_id": "m"}),
        ] {
            let request = serde_json::json!({
                "schema": "ferrum-operation-request-v1",
                "request_id": "response-admission-correlation",
                "operation": {
                    "kind": "document.molecule.smarts.query.v1",
                    "document": {"cdml": CDML, "expected_revision": 0, "expected_digest_hex": digest},
                    "query": query,
                    "limits": {"max_matches_per_molecule": 1, "max_total_matches": 1},
                },
            });
            let input = request.to_string();
            let cli = || {
                Cli::try_parse_from([
                    "ferrum",
                    "document",
                    "command",
                    "document.molecule.smarts.query.v1",
                    "-",
                ])
                .expect("named SMARTS command parses")
            };
            let runtime = || {
                SelectedSmartsRuntime(SelectedSmartsEngine {
                    queries: RefCell::new(Vec::new()),
                })
            };

            let mut complete_stdin = input.as_bytes();
            let mut complete_stdout = Vec::new();
            let mut complete_stderr = Vec::new();
            let complete_runtime = runtime();
            super::run_with_runtime_and_smarts_response_limit_for_test(
                cli(),
                &mut complete_stdin,
                &mut complete_stdout,
                &mut complete_stderr,
                &complete_runtime,
                usize::MAX,
            )
            .expect("unbounded controlled named SMARTS response completes");
            assert!(complete_stderr.is_empty());
            let canonical_len = complete_stdout.len().checked_sub(1).expect("newline only");
            assert_eq!(complete_stdout.last(), Some(&b'\n'));

            let mut boundary_stdin = input.as_bytes();
            let mut boundary_stdout = Vec::new();
            let mut boundary_stderr = Vec::new();
            let boundary_runtime = runtime();
            super::run_with_runtime_and_smarts_response_limit_for_test(
                cli(),
                &mut boundary_stdin,
                &mut boundary_stdout,
                &mut boundary_stderr,
                &boundary_runtime,
                canonical_len,
            )
            .expect("exact canonical JSON boundary is admitted");
            assert_eq!(boundary_stdout, complete_stdout);
            assert!(boundary_stderr.is_empty());

            let mut over_stdin = input.as_bytes();
            let mut over_stdout = Vec::new();
            let mut over_stderr = Vec::new();
            let over_runtime = runtime();
            super::run_with_runtime_and_smarts_response_limit_for_test(
                cli(),
                &mut over_stdin,
                &mut over_stdout,
                &mut over_stderr,
                &over_runtime,
                canonical_len - 1,
            )
            .expect("over-limit response becomes protocol data");
            assert!(over_stderr.is_empty());
            let rendered = String::from_utf8(over_stdout).expect("UTF-8 JSON response");
            let refusal: serde_json::Value = serde_json::from_str(&rendered).expect("JSON refusal");
            assert_eq!(refusal["request_id"], "response-admission-correlation");
            assert_eq!(refusal["error"]["category"], "resource_limit");
            assert_eq!(
                refusal["error"]["resource_limit_reason"],
                "response_size_exceeded"
            );
            assert_eq!(refusal["error"]["message"], "response_size_exceeded");
            assert_eq!(
                refusal["error"]["operation"],
                "document.molecule.smarts.query.v1"
            );
            for forbidden in [
                "FERRUM_PRIVATE_RAW_SMARTS",
                "selected-fixture-smarts",
                CDML,
                "molecules",
                "rows",
                "receipt",
                "record_id",
            ] {
                assert!(
                    !rendered.contains(forbidden),
                    "oversized named SMARTS response leaked {forbidden}"
                );
            }
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
