use ferrum_document::{
    CdsvgExtractionError, CoreProjectionError, TypedDocumentError, XmlSerializationError,
};
use thiserror::Error;

use crate::protocol_cli::ProtocolCliError;

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

/// A CD-SVG library extraction operation failed.
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
    /// Frozen operation-protocol CLI input, emission, or publication failed.
    #[error(transparent)]
    Protocol(#[from] ProtocolCliError),
}
