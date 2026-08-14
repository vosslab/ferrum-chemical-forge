use std::io;

use ferrum_document::artifact_publication_v1::ArtifactPublicationErrorV1;
use ferrum_document::{
    CdsvgExtractionError, CoreProjectionError, DocumentSessionError, TypedDocumentError,
    XmlSerializationError,
};
use thiserror::Error;

use crate::canonical_smiles::CanonicalSmilesError;
use crate::inchi_codec::{InchiExportError, InchiInspectionError};
use crate::molblock_export::MolblockExportError;
use crate::molblock_inspection::MolblockInspectionError;
use crate::molecule_coordinate_cli::MoleculeCoordinateCliError;
use crate::render_observation_cli::RenderObservationCliError;
use crate::sdf_export::SdfExportError;
use crate::sdf_inspection::SdfInspectionError;
use crate::smarts_export::SmartsExportError;
use crate::smiles_inspection::SmilesInspectionError;
use crate::{
    DocumentIngressErrorV1, DocumentPdfArtifactErrorV1, DocumentPngArtifactErrorV1,
    DocumentSvgArtifactErrorV1,
};

/// A CDML library operation failed.
#[derive(Debug, Error)]
pub enum CdmlError {
    /// CDML could not be parsed or retained in Ferrum's typed document model.
    #[error(transparent)]
    Document(#[from] TypedDocumentError),
    /// Typed CDML could not produce a valid core molecule model.
    #[error(transparent)]
    Projection(#[from] CoreProjectionError),
    /// The retained document tree could not be serialized.
    #[error(transparent)]
    Serialization(#[from] XmlSerializationError),
    /// A parsed XML tree could not be snapshotted for preservation validation.
    #[error("cannot snapshot retained XML structure: {0}")]
    StructuralSnapshot(#[source] xot::ParseError),
    /// Serializing then reparsing changed a Ferrum-owned structural observation.
    #[error("structural preservation check failed after serialization")]
    StructuralPreservation,
}

/// A CD-SVG extraction operation failed.
#[derive(Debug, Error)]
pub enum CdsvgError {
    /// The SVG wrapper did not contain exactly one valid canonical CDML payload.
    #[error(transparent)]
    Extraction(#[from] CdsvgExtractionError),
    /// The extracted CDML payload could not be structurally serialized.
    #[error(transparent)]
    Serialization(#[from] XmlSerializationError),
    /// The extracted CDML did not satisfy Ferrum's rewrite verification transaction.
    #[error(transparent)]
    Preservation(#[from] CdmlError),
}

/// A command-line operation failed after its arguments were accepted.
#[derive(Debug, Error)]
pub enum CliError {
    /// The requested input could not be read as UTF-8 text.
    #[error("could not read {input}: {source}")]
    Read {
        /// User-facing input label.
        input: String,
        /// Underlying I/O failure.
        #[source]
        source: io::Error,
    },
    /// A local file or bounded standard-input document was not admitted.
    #[error("could not admit document input: {0}")]
    DocumentIngress(#[from] DocumentIngressErrorV1),
    /// An admitted document could not produce one complete native SVG artifact.
    #[error("could not render complete SVG: {0}")]
    DocumentSvgArtifact(#[from] DocumentSvgArtifactErrorV1),
    /// An admitted document could not produce one complete native vector PDF artifact.
    #[error("could not render complete PDF: {0}")]
    DocumentPdfArtifact(#[from] DocumentPdfArtifactErrorV1),
    /// An admitted document could not produce one complete native raster PNG artifact.
    #[error("could not render complete PNG: {0}")]
    DocumentPngArtifact(#[from] DocumentPngArtifactErrorV1),
    /// Safe generic artifact publication rejected or could not finish the output.
    #[error("could not publish rendered artifact: {0}")]
    ArtifactPublication(#[from] ArtifactPublicationErrorV1),
    /// CDML processing failed.
    #[error("could not process {input}: {source}")]
    Cdml {
        /// User-facing input label.
        input: String,
        /// Typed CDML failure.
        #[source]
        source: CdmlError,
    },
    /// CD-SVG extraction failed.
    #[error("could not extract CDML from {input}: {source}")]
    Cdsvg {
        /// User-facing input label.
        input: String,
        /// CD-SVG extraction failure.
        #[source]
        source: CdsvgError,
    },
    /// The loaded CDML could not produce a complete Ferrum render observation.
    #[error("could not render-observe {input}: {source}")]
    RenderObservation {
        /// User-facing input label.
        input: String,
        /// Render-observation failure.
        #[source]
        source: RenderObservationCliError,
    },
    /// One existing CDML molecule could not be regenerated through the named adapter.
    #[error("could not generate coordinates for {input}: {source}")]
    MoleculeCoordinates {
        /// User-facing CDML input label.
        input: String,
        /// Typed document, adapter, chemistry, or placement failure.
        #[source]
        source: MoleculeCoordinateCliError,
    },
    /// The requested SMILES value could not be inspected through the named adapter.
    #[error("could not inspect SMILES: {0}")]
    SmilesInspection(#[from] SmilesInspectionError),
    /// The requested SMILES value could not be serialized canonically.
    #[error("could not canonicalize SMILES: {0}")]
    CanonicalSmiles(#[from] CanonicalSmilesError),
    /// The requested SMILES molecule could not be exported as InChI.
    #[error("could not export InChI: {0}")]
    InchiExport(#[from] InchiExportError),
    /// The requested InChI could not be inspected through the named adapter.
    #[error("could not inspect InChI: {0}")]
    InchiInspection(#[from] InchiInspectionError),
    /// The requested SMILES molecule could not be exported as SMARTS.
    #[error("could not export SMARTS: {0}")]
    SmartsExport(#[from] SmartsExportError),
    /// The requested SMILES molecule could not be exported as a molblock.
    #[error("could not export molblock: {0}")]
    MolblockExport(#[from] MolblockExportError),
    /// The requested SMILES molecule could not be exported as one SDF record.
    #[error("could not export SDF: {0}")]
    SdfExport(#[from] SdfExportError),
    /// The requested SDF input could not be inspected through the named adapter.
    #[error("could not inspect SDF from {input}: {source}")]
    SdfInspection {
        /// User-facing input label.
        input: String,
        /// Typed adapter or chemistry failure.
        #[source]
        source: SdfInspectionError,
    },
    /// The requested molblock could not be inspected through the named adapter.
    #[error("could not inspect molblock from {input}: {source}")]
    MolblockInspection {
        /// User-facing input label.
        input: String,
        /// Typed adapter or chemistry failure.
        #[source]
        source: MolblockInspectionError,
    },
    /// A versioned JSON report could not be encoded.
    #[error("could not encode Ferrum JSON report: {0}")]
    Json(#[from] serde_json::Error),
    /// The requested output could not be written.
    #[error("could not write {output}: {source}")]
    Write {
        /// User-facing output label.
        output: String,
        /// Underlying I/O failure.
        #[source]
        source: io::Error,
    },
    /// The document-owned safe publisher rejected or could not complete the requested output.
    #[error("could not write {output}: {source}")]
    Publish {
        /// User-facing output label.
        output: String,
        /// Descriptor-relative publication failure.
        #[source]
        source: DocumentSessionError,
    },
}
