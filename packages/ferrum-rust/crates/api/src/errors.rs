use std::io;

use ferrum_document::{
    CdsvgExtractionError, CoreProjectionError, DocumentSessionError, TypedDocumentError,
    XmlSerializationError,
};
use thiserror::Error;

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
