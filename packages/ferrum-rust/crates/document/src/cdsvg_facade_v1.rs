//! CD-SVG extraction and verification for the Rust library boundary.

use crate::{CdmlError, CdsvgExtractionError, XmlSerializationError, extract_cdml_from_svg};
use thiserror::Error;

use crate::rewrite_cdml;

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

/// Extract the one canonical CDML payload from decoded CD-SVG and verify it for publication.
///
/// The returned owned XML has completed Ferrum's structural parse, serialize, and reparse
/// transaction. Callers can therefore pass it directly to an atomic output boundary.
pub fn extract_cdsvg(source: &str) -> Result<String, CdsvgError> {
    let extracted = extract_cdml_from_svg(source)?;
    let serialized = extracted.to_xml()?;
    rewrite_cdml(&serialized).map_err(CdsvgError::from)
}
