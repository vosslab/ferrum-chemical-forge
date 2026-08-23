use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use ferrum_document::InterchangeFormatV1;

use crate::interchange_import_v1::{InterchangeDecoderKeyV1, InterchangeFormatRegistryV1};

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
        /// Source syntax; otherwise inferred from .smi, .inchi, .mol, .sdf, .cdml, or .cml.
        #[arg(long = "from", value_parser = parse_interchange_input_format)]
        input_format: Option<InterchangeInputFormat>,
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum DocumentInputFormat {
    Cdml,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum DocumentOutputFormat {
    Cdml,
}

/// Closed molecular interchange syntax vocabulary used by `ferrum convert`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
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

/// Closed molecular interchange input vocabulary used by `ferrum convert`.
///
/// CML stays input-only: its accepted aliases are resolved from the API-owned
/// interchange registry and its records flow through the same lowering bridge
/// as document import.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InterchangeInputFormat {
    Native(InterchangeFormat),
    CmlSimpleMolecule,
}

impl From<InterchangeInputFormat> for InterchangeFormatV1 {
    fn from(value: InterchangeInputFormat) -> Self {
        match value {
            InterchangeInputFormat::Native(format) => format.into(),
            InterchangeInputFormat::CmlSimpleMolecule => Self::CmlSimpleMolecule,
        }
    }
}

/// Parse a convert input format, joining CML aliases through the canonical registry.
pub(crate) fn parse_interchange_input_format(
    value: &str,
) -> Result<InterchangeInputFormat, String> {
    if let Ok(descriptor) = InterchangeFormatRegistryV1::lookup_input_alias(value) {
        return Ok(interchange_input_format_from_descriptor(descriptor));
    }
    InterchangeFormat::from_str(value, false)
        .map(InterchangeInputFormat::Native)
        .map_err(|error| error.to_string())
}

/// Map an API-owned interchange descriptor to its closed convert input profile.
pub(crate) fn interchange_input_format_from_descriptor(
    descriptor: &crate::interchange_import_v1::InterchangeFormatDescriptorV1,
) -> InterchangeInputFormat {
    match descriptor.decoder() {
        InterchangeDecoderKeyV1::CmlSimpleMolecule => InterchangeInputFormat::CmlSimpleMolecule,
        InterchangeDecoderKeyV1::Sdf => InterchangeInputFormat::Native(InterchangeFormat::SdfV2000),
    }
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
    /// Export selected direct-root molecules as one atomic multi-record SDF file.
    #[command(
        name = "export-sdf",
        after_help = "Example:\n  ferrum document export-sdf --input drawing.cdml --molecule-id root-a --molecule-id root-b --version v3000 --output selected.sdf"
    )]
    ExportSdf {
        /// Input CDML document.
        #[arg(long)]
        input: PathBuf,
        /// Authored CDML direct-molecule ID to include. Repeat for each root.
        #[arg(long = "molecule-id", required = true)]
        molecule_ids: Vec<String>,
        /// SDF Molfile record syntax.
        #[arg(long, value_enum)]
        version: SdfVersion,
        /// Required SDF destination, published atomically after complete export.
        #[arg(long, value_parser = output_file_path)]
        output: PathBuf,
    },
    /// Execute one named document mutation command.
    Command {
        #[command(subcommand)]
        command: NamedDocumentCommand,
    },
}

/// Closed SDF record syntax accepted by document export.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum SdfVersion {
    V2000,
    V3000,
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
    /// Author one closed presentation family through the stateless protocol.
    #[command(name = "presentation.author.v1")]
    PresentationAuthor {
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

fn output_file_path(value: &str) -> Result<PathBuf, String> {
    if value == "-" {
        return Err(
            "--output must name a file destination; omit it for standard output".to_owned(),
        );
    }
    Ok(PathBuf::from(value))
}
