use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use ferrum_document::InterchangeFormatV1;

/// Ferrum command-line arguments.
#[derive(Debug, Parser)]
#[command(name = "ferrum", version, about = "Ferrum chemical document tools")]
pub struct Cli {
    #[command(subcommand)]
    pub(super) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
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
    /// Render one closed direct-glycosidic Haworth structural SMILES as SVG.
    #[command(after_help = "Example:\n  ferrum haworth 'O1CCCCC1OCC2CCCCC2O' -o haworth.svg")]
    Haworth {
        /// Structural SMILES text, or `-` for standard input.
        smiles: String,
        /// SVG destination, or omit for standard output.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Open one declared interchange source as a newly created Ferrum document.
    #[command(
        after_help = "Example:\n  ferrum open molecule.sdf --format sdf --output molecule.cdml"
    )]
    Open {
        /// Declared interchange input path, or `-` for standard input.
        input: PathBuf,
        /// Input format. Required for standard input; registry suffixes are inferred for named files.
        #[arg(long)]
        format: Option<String>,
        /// New CDML file destination.
        #[arg(short, long, value_parser = output_file_path)]
        output: PathBuf,
        /// Emit the complete fixed interchange-open response envelope.
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
pub(crate) enum DocumentInputFormat {
    Cdml,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum DocumentOutputFormat {
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
pub(crate) enum ProtocolCommand {
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
pub(crate) enum DocumentCommand {
    /// Execute one named document mutation command.
    Command {
        #[command(subcommand)]
        command: NamedDocumentCommand,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum NamedDocumentCommand {
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
pub(crate) enum EngineCommand {
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
