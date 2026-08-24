//! DTOs for general bounded document transformations and rendering.

use ferrum_document::InterchangeFormatV1;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Request payload for `chemistry.convert`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChemistryConvertRequestV1 {
    /// Closed source interchange syntax and bounded owned text.
    pub input: ChemistryConvertInputV1,
    /// Closed target interchange syntax.
    pub output_format: InterchangeFormatV1,
}

/// Owned chemistry conversion input.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChemistryConvertInputV1 {
    /// Closed source interchange syntax.
    pub format: InterchangeFormatV1,
    /// Bounded source text; never a filesystem path or native handle.
    pub text: String,
}

/// Request payload for `document.generate_coordinates`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentGenerateCoordinatesRequestV1 {
    /// Uncompressed CDML text admitted under Ferrum's existing V1 profile.
    pub document: String,
}

/// Request payload for `document.inspect`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentInspectRequestV1 {
    /// Uncompressed CDML text admitted under Ferrum's existing V1 profile.
    pub document: String,
}

/// Immutable request fence derived from one admitted document snapshot.
///
/// The caller keeps the original request-owned document and may submit these
/// values unchanged to a subsequent document mutation.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentRequestFenceV1 {
    /// Revision of the admitted snapshot.
    pub expected_revision: u64,
    /// Lowercase hexadecimal SHA-256 digest of the admitted snapshot.
    pub expected_digest_hex: String,
}

/// Request payload for `document.validate`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentValidateRequestV1 {
    /// Uncompressed CDML text admitted under Ferrum's existing V1 profile.
    pub document: String,
    /// The requested structural or typed validation level.
    pub level: ProtocolValidationLevelV1,
}

/// Closed validation levels exposed by V1.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub enum ProtocolValidationLevelV1 {
    /// Retain and structurally validate CDML.
    #[serde(rename = "structural")]
    Structural,
    /// Require the existing typed Ferrum core projection.
    #[serde(rename = "typed")]
    Typed,
}

/// Request payload for `document.rewrite`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentRewriteRequestV1 {
    /// Uncompressed CDML text admitted under Ferrum's existing V1 profile.
    pub document: String,
}

/// Request payload for `document.render_artifact`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentRenderArtifactRequestV1 {
    /// Uncompressed CDML text admitted under Ferrum's existing V1 profile.
    pub document: String,
    /// The complete artifact format to render.
    pub format: ProtocolArtifactFormatV1,
}

/// Closed complete-document artifact formats exposed by V1.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub enum ProtocolArtifactFormatV1 {
    /// Complete SVG.
    #[serde(rename = "svg")]
    Svg,
    /// Complete vector PDF.
    #[serde(rename = "pdf")]
    Pdf,
    /// Transparent PNG at one pixel per Rust page point.
    #[serde(rename = "png_one_pixel_per_point_transparent")]
    PngOnePixelPerPointTransparent,
}
