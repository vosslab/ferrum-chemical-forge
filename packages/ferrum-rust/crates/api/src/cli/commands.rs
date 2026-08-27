use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use ferrum_document::InterchangeFormatV1;

use crate::InterchangeCapabilityResolverV1;
use crate::protocol::ProtocolOperationKindV1;

/// Ferrum command-line arguments.
#[derive(Debug, Parser)]
#[command(name = "ferrum", version, about = "Ferrum chemical document tools")]
pub struct Cli {
    #[command(subcommand)]
    pub(super) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Materialize one selected molecule's hydrogens through the frozen protocol route.
    #[command(name = "document-molecule-hydrogen-materialize")]
    DocumentMoleculeHydrogenMaterialize {
        /// Complete operation-protocol JSON request path, or `-` for standard input.
        #[arg(long)]
        request: PathBuf,
    },
    /// Observe one selected atom's oxidation state through the frozen protocol route.
    #[command(name = "document-atom-oxidation-observe")]
    DocumentAtomOxidationObserve {
        /// Complete operation-protocol JSON request path, or `-` for standard input.
        #[arg(long)]
        request: PathBuf,
    },
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
        /// Source syntax; otherwise inferred from .smi, .inchi, .mol, .sdf, .cdml, .cml, or .cdxml.
        #[arg(long = "from", value_parser = parse_interchange_input_format)]
        input_format: Option<InterchangeInputFormat>,
        /// Target syntax resolved by Ferrum's conversion-output registry.
        #[arg(long = "to")]
        output_format: String,
        /// Emit the complete operation-protocol envelope.
        #[arg(long)]
        json: bool,
    },
    #[command(after_help = "Example:\n  ferrum inspect-graph molecule.cml --from cml --json")]
    InspectGraph {
        input: PathBuf,
        #[arg(long = "from")]
        input_format: String,
        #[arg(long)]
        json: bool,
    },
    /// List Ferrum's declared molecular interchange capabilities.
    #[command(after_help = "Example:\n  ferrum formats --json")]
    Formats {
        /// Emit the versioned interchange-capability response as JSON.
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
/// Values are presentation types; API-owned capability resolution chooses them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InterchangeInputFormat {
    Native(InterchangeFormat),
    CmlSimpleMolecule,
    CdxmlSimpleMolecule,
}

impl From<InterchangeInputFormat> for InterchangeFormatV1 {
    fn from(value: InterchangeInputFormat) -> Self {
        match value {
            InterchangeInputFormat::Native(format) => format.into(),
            InterchangeInputFormat::CmlSimpleMolecule => Self::CmlSimpleMolecule,
            InterchangeInputFormat::CdxmlSimpleMolecule => Self::CdxmlSimpleMolecule,
        }
    }
}

/// Parse a convert input format through the API-owned capability resolver.
pub(crate) fn parse_interchange_input_format(
    value: &str,
) -> Result<InterchangeInputFormat, String> {
    InterchangeCapabilityResolverV1::lookup_input_alias(value)
        .map(|descriptor| {
            interchange_input_format_from_protocol_format(descriptor.protocol_format())
        })
        .map_err(|error| format!("unsupported interchange input format: {error:?}"))
}

/// Map one API-resolved protocol format to CLI presentation.
pub(crate) fn interchange_input_format_from_protocol_format(
    format: InterchangeFormatV1,
) -> InterchangeInputFormat {
    match format {
        InterchangeFormatV1::CmlSimpleMolecule => InterchangeInputFormat::CmlSimpleMolecule,
        InterchangeFormatV1::CdxmlSimpleMolecule => InterchangeInputFormat::CdxmlSimpleMolecule,
        InterchangeFormatV1::Smiles => InterchangeInputFormat::Native(InterchangeFormat::Smiles),
        InterchangeFormatV1::InchiStandard => {
            InterchangeInputFormat::Native(InterchangeFormat::InchiStandard)
        }
        InterchangeFormatV1::InchiFixedHydrogen => {
            InterchangeInputFormat::Native(InterchangeFormat::InchiFixedHydrogen)
        }
        InterchangeFormatV1::MolblockV2000 => {
            InterchangeInputFormat::Native(InterchangeFormat::MolblockV2000)
        }
        InterchangeFormatV1::MolblockV3000 => {
            InterchangeInputFormat::Native(InterchangeFormat::MolblockV3000)
        }
        InterchangeFormatV1::SdfV2000 => {
            InterchangeInputFormat::Native(InterchangeFormat::SdfV2000)
        }
        InterchangeFormatV1::SdfV3000 => {
            InterchangeInputFormat::Native(InterchangeFormat::SdfV3000)
        }
        InterchangeFormatV1::Cdml => InterchangeInputFormat::Native(InterchangeFormat::Cdml),
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

#[cfg(test)]
mod tests {
    use super::{InterchangeInputFormat, parse_interchange_input_format};

    #[test]
    fn cdxml_alias_resolves_to_the_cdxml_input_identity() {
        assert_eq!(
            parse_interchange_input_format("cdxml"),
            Ok(InterchangeInputFormat::CdxmlSimpleMolecule)
        );
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
    /// Report one selected molecule through the frozen molecule-report route.
    #[command(name = "document.molecule.report.v1")]
    DocumentMoleculeReport {
        input: PathBuf,
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    /// Check selected direct roots through the frozen structure-diagnostics route.
    #[command(name = "document.molecule.diagnostics.v1")]
    DocumentMoleculeDiagnostics {
        input: PathBuf,
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    /// Materialize one typed compact group through the frozen protocol route.
    #[command(name = "document.compact-group.materialize.v1")]
    DocumentCompactGroupMaterialize {
        input: PathBuf,
        #[arg(short, long, value_parser = output_file_path)]
        output: Option<PathBuf>,
    },
    /// Attach one typed compact group through the frozen protocol route.
    #[command(name = "document.compact-group.attach.v1")]
    DocumentCompactGroupAttach {
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
}

impl NamedDocumentCommand {
    /// Consume one named route into its only admitted operation kind and transport paths.
    ///
    /// The named command is an operation-specific entry point, not an alias for the
    /// generic protocol runner. Keeping this exhaustive adapter beside the CLI
    /// vocabulary makes every new named route declare its protocol contract.
    pub(crate) fn into_protocol_request(
        self,
    ) -> (ProtocolOperationKindV1, PathBuf, Option<PathBuf>) {
        match self {
            Self::DocumentMoleculeReport { input, output } => (
                ProtocolOperationKindV1::DocumentMoleculeReport,
                input,
                output,
            ),
            Self::DocumentMoleculeDiagnostics { input, output } => (
                ProtocolOperationKindV1::DocumentMoleculeDiagnostics,
                input,
                output,
            ),
            Self::DocumentCompactGroupMaterialize { input, output } => (
                ProtocolOperationKindV1::DocumentCompactGroupMaterialize,
                input,
                output,
            ),
            Self::DocumentCompactGroupAttach { input, output } => (
                ProtocolOperationKindV1::DocumentCompactGroupAttach,
                input,
                output,
            ),
            Self::DocumentMoleculeSmartsQuery { input, output } => {
                (ProtocolOperationKindV1::DocumentSmartsQuery, input, output)
            }
            Self::DocumentMoleculeInterchangeImport { input, output } => (
                ProtocolOperationKindV1::DocumentMoleculeInterchangeImport,
                input,
                output,
            ),
            Self::CatalogList { input, output } => {
                (ProtocolOperationKindV1::CatalogList, input, output)
            }
            Self::CatalogInsert { input, output } => {
                (ProtocolOperationKindV1::CatalogInsert, input, output)
            }
            Self::PresentationAuthor { input, output } => {
                (ProtocolOperationKindV1::PresentationAuthor, input, output)
            }
            Self::ReactionCreate { input, output } => {
                (ProtocolOperationKindV1::ReactionCreate, input, output)
            }
            Self::ReactionList { input, output } => {
                (ProtocolOperationKindV1::ReactionList, input, output)
            }
            Self::ReactionObserve { input, output } => {
                (ProtocolOperationKindV1::ReactionObserve, input, output)
            }
            Self::ReactionSelect { input, output } => {
                (ProtocolOperationKindV1::ReactionSelect, input, output)
            }
            Self::ReactionPatchMembership { input, output } => (
                ProtocolOperationKindV1::ReactionPatchMembership,
                input,
                output,
            ),
            Self::ReactionDeleteDefinition { input, output } => (
                ProtocolOperationKindV1::ReactionDeleteDefinition,
                input,
                output,
            ),
        }
    }
}

fn output_file_path(value: &str) -> Result<PathBuf, String> {
    if value == "-" {
        return Err(
            "--output must name a file destination; omit it for standard output".to_owned(),
        );
    }
    Ok(PathBuf::from(value))
}
