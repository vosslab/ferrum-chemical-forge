//! CD-SVG extraction and verification for the command-line boundary.

use ferrum_document::extract_cdml_from_svg;

use crate::cdml::rewrite_cdml;
use crate::errors::CdsvgError;

/// Extract the one canonical CDML payload from decoded CD-SVG and verify it for publication.
///
/// The returned owned XML has completed Ferrum's structural parse, serialize, and reparse
/// transaction. Callers can therefore pass it directly to an atomic output boundary.
pub fn extract_cdsvg(source: &str) -> Result<String, CdsvgError> {
    let extracted = extract_cdml_from_svg(source)?;
    let serialized = extracted.to_xml()?;
    rewrite_cdml(&serialized).map_err(CdsvgError::from)
}
