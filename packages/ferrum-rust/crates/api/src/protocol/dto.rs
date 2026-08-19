//! Closed, stateless JSON operation protocol V1.
//!
//! This module owns only the portable request/response data contract.

use ferrum_document::{CdmlInspection, CdmlValidation, InterchangeFormatV1, RewriteCheck};
use ferrum_render::{
    LOCAL_PDF_COMPLETED_BYTES_V1, LOCAL_PNG_ENCODED_BYTES_V1, LOCAL_SVG_COMPLETED_BYTES_V1,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Exact schema identifier accepted for V1 requests.
pub const OPERATION_PROTOCOL_REQUEST_SCHEMA_V1: &str = "ferrum-operation-request-v1";
/// Exact schema identifier emitted for V1 successful responses.
pub const OPERATION_PROTOCOL_RESPONSE_SCHEMA_V1: &str = "ferrum-operation-response-v1";
/// Exact schema identifier emitted for V1 protocol error responses.
pub const OPERATION_PROTOCOL_ERROR_SCHEMA_V1: &str = "ferrum-operation-error-v1";

// V1 returns one base64 completion, so its ceiling is the tightest accepted
// complete-artifact limit across the three existing native profiles.
pub(super) const MAX_ARTIFACT_BYTES_V1: usize = smallest_artifact_limit(
    LOCAL_SVG_COMPLETED_BYTES_V1,
    LOCAL_PDF_COMPLETED_BYTES_V1,
    LOCAL_PNG_ENCODED_BYTES_V1,
);
pub(super) const MAX_ARTIFACT_BASE64_BYTES_V1: usize = base64_encoded_len(MAX_ARTIFACT_BYTES_V1)
    .expect("the accepted V1 artifact limit has a representable base64 expansion");

// A JSON string can encode every ASCII CDML byte as a six-byte `\\u00XX`
// escape.  The fixed allowance covers V1's closed framing and its opaque request
// ID; it is intentionally separate from the document-source policy.
const OPERATION_PROTOCOL_V1_FRAMING_AND_REQUEST_ID_UTF8_BYTES: usize = 4 * 1024;
/// Maximum UTF-8 length of the opaque request identifier admitted by V1.
///
/// This independent cap keeps every echoed success or admitted-error envelope
/// bounded even when the surrounding transport budget has spare capacity.
pub const MAX_REQUEST_ID_UTF8_BYTES_V1: usize = 2 * 1024;
/// Maximum UTF-8 request text accepted before any JSON allocation or parsing.
///
/// This is `LOCAL_CDML_SOURCE_UTF8_BYTES_V1 * 6` for worst-case valid JSON
/// escaping of an accepted CDML source, plus the named V1 framing/request-ID
/// allowance above. It is a transport allocation boundary, not a document or
/// corpus validity rule.
pub const OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1: usize = protocol_request_limit(
    ferrum_document::LOCAL_CDML_SOURCE_UTF8_BYTES_V1,
    OPERATION_PROTOCOL_V1_FRAMING_AND_REQUEST_ID_UTF8_BYTES,
)
.expect("the accepted local CDML limit has a representable V1 request expansion");

/// One closed V1 request after its exact schema identifier has been admitted.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationProtocolRequestV1 {
    /// Exact V1 request schema identifier.
    pub schema: ProtocolRequestSchemaV1,
    /// Caller-owned opaque text echoed unchanged after envelope admission.
    pub request_id: String,
    /// The one fully typed operation to execute.
    pub operation: OperationProtocolOperationV1,
}

/// Closed V1 request schema identifiers.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub enum ProtocolRequestSchemaV1 {
    /// `ferrum-operation-request-v1`.
    #[serde(rename = "ferrum-operation-request-v1")]
    V1,
}

/// The closed protocol operations admitted by V1.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum OperationProtocolOperationV1 {
    /// Inspect one bounded CDML document.
    #[serde(rename = "document.inspect")]
    Inspect(DocumentInspectRequestV1),
    /// Validate one bounded CDML document at the selected semantic level.
    #[serde(rename = "document.validate")]
    Validate(DocumentValidateRequestV1),
    /// Structurally rewrite one bounded CDML document.
    #[serde(rename = "document.rewrite")]
    Rewrite(DocumentRewriteRequestV1),
    /// Render one complete bounded CDML document to an in-memory artifact.
    #[serde(rename = "document.render_artifact")]
    RenderArtifact(DocumentRenderArtifactRequestV1),
    /// Convert bounded molecular interchange through an injected chemistry runtime.
    #[serde(rename = "chemistry.convert")]
    ChemistryConvert(ChemistryConvertRequestV1),
    /// Regenerate all direct typed molecule coordinates as one document transition.
    #[serde(rename = "document.generate_coordinates")]
    GenerateCoordinates(DocumentGenerateCoordinatesRequestV1),
}

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

/// One successful V1 response.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationProtocolResponseV1 {
    /// Exact V1 success schema identifier.
    pub schema: ProtocolResponseSchemaV1,
    /// Unchanged caller-owned request identifier.
    pub request_id: String,
    /// The typed operation result.
    pub outcome: OperationProtocolOutcomeV1,
}

/// Closed V1 success schema identifiers.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
pub enum ProtocolResponseSchemaV1 {
    /// `ferrum-operation-response-v1`.
    #[serde(rename = "ferrum-operation-response-v1")]
    V1,
}

/// Tagged successful results for the V1 operation set.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum OperationProtocolOutcomeV1 {
    /// Semantic CDML inspection facts.
    #[serde(rename = "document.inspect")]
    Inspect { report: CdmlInspection },
    /// Validation facts plus the caller-selected protocol level.
    #[serde(rename = "document.validate")]
    Validate {
        /// The selected protocol level, retained even when lower report labels differ.
        level: ProtocolValidationLevelV1,
        /// Existing Ferrum validation report.
        report: CdmlValidation,
    },
    /// Rewritten CDML plus its structural preservation report.
    #[serde(rename = "document.rewrite")]
    Rewrite {
        /// Structurally re-emitted CDML, not a byte-preservation promise.
        document: String,
        /// Existing structural rewrite-check report.
        report: RewriteCheck,
    },
    /// One complete native artifact encoded as standard base64.
    #[serde(rename = "document.render_artifact")]
    RenderArtifact {
        /// The requested closed artifact format.
        format: ProtocolArtifactFormatV1,
        /// The media type associated with `format`.
        media_type: String,
        /// Complete artifact bytes encoded with RFC 4648 standard base64.
        artifact_base64: String,
    },
    /// Converted owned molecular interchange text.
    #[serde(rename = "chemistry.convert")]
    ChemistryConvert {
        /// Closed target syntax.
        format: InterchangeFormatV1,
        /// Complete converted text.
        text: String,
        /// Number of preserved molecular records.
        record_count: usize,
    },
    /// Structural CDML after atomic coordinate regeneration.
    #[serde(rename = "document.generate_coordinates")]
    GenerateCoordinates {
        /// Structural CDML snapshot after one shared coordinate batch commit.
        document: String,
        /// Number of direct typed molecular roots regenerated.
        regenerated_molecule_count: usize,
    },
}

/// A typed domain refusal returned as protocol data.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationProtocolErrorResponseV1 {
    /// Exact V1 error schema identifier.
    pub schema: ProtocolErrorSchemaV1,
    /// Present only after the request envelope admitted this opaque value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Stable discriminator and diagnostic detail.
    pub error: OperationProtocolErrorV1,
}

/// Closed V1 error schema identifiers.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
pub enum ProtocolErrorSchemaV1 {
    /// `ferrum-operation-error-v1`.
    #[serde(rename = "ferrum-operation-error-v1")]
    V1,
}

/// Stable protocol error facts; clients must not discriminate on `message`.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationProtocolErrorV1 {
    /// Stable machine-readable failure category.
    pub category: OperationProtocolErrorCategoryV1,
    /// Decoded operation kind when one was available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<ProtocolOperationKindV1>,
    /// Human-readable diagnostic detail.
    pub message: String,
}

/// Stable V1 error categories.
#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
pub enum OperationProtocolErrorCategoryV1 {
    /// The JSON envelope or closed operation payload was invalid.
    #[serde(rename = "invalid_request")]
    InvalidRequest,
    /// The envelope named a schema identifier not supported by V1.
    #[serde(rename = "unsupported_protocol_version")]
    UnsupportedProtocolVersion,
    /// Bounded uncompressed CDML admission failed.
    #[serde(rename = "document_admission_failed")]
    DocumentAdmissionFailed,
    /// A semantically required document operation could not proceed.
    #[serde(rename = "document_invalid")]
    DocumentInvalid,
    /// The document cannot produce a complete artifact for the requested profile.
    #[serde(rename = "render_unsupported")]
    RenderUnsupported,
    /// A native artifact backend could not finish rendering.
    #[serde(rename = "render_failed")]
    RenderFailed,
    /// The caller supplied no usable out-of-band chemistry capability.
    #[serde(rename = "chemistry_unavailable")]
    ChemistryUnavailable,
    /// Molecular interchange was malformed, bounded out, or could not complete.
    #[serde(rename = "conversion_failed")]
    ConversionFailed,
    /// The requested interchange target cannot exactly represent the input records.
    #[serde(rename = "conversion_unsupported")]
    ConversionUnsupported,
    /// Coordinate generation could not complete for the whole document snapshot.
    #[serde(rename = "coordinate_generation_failed")]
    CoordinateGenerationFailed,
    /// Existing resource policy or derived base64 completion limits refused work.
    #[serde(rename = "resource_limit")]
    ResourceLimit,
    /// An unexpected owned-value executor failure occurred.
    #[serde(rename = "internal_failure")]
    InternalFailure,
}

/// Stable names for decoded operation kinds in error envelopes.
#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
pub enum ProtocolOperationKindV1 {
    /// `document.inspect`.
    #[serde(rename = "document.inspect")]
    Inspect,
    /// `document.validate`.
    #[serde(rename = "document.validate")]
    Validate,
    /// `document.rewrite`.
    #[serde(rename = "document.rewrite")]
    Rewrite,
    /// `document.render_artifact`.
    #[serde(rename = "document.render_artifact")]
    RenderArtifact,
    /// `chemistry.convert`.
    #[serde(rename = "chemistry.convert")]
    ChemistryConvert,
    /// `document.generate_coordinates`.
    #[serde(rename = "document.generate_coordinates")]
    GenerateCoordinates,
}

/// A completed operation response or typed protocol refusal.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum OperationProtocolEnvelopeV1 {
    /// Successful response.
    Success(OperationProtocolResponseV1),
    /// Typed refusal.
    Error(OperationProtocolErrorResponseV1),
}

/// Failure before a JSON response envelope can exist.
#[derive(Debug, Error)]
pub enum OperationProtocolInputErrorV1 {
    /// Request text exceeded the derived V1 ingress budget before JSON parsing.
    #[error(
        "operation request exceeds the {limit}-byte V1 ingress limit ({observed} bytes observed)"
    )]
    ResourceLimit {
        /// Derived V1 request transport budget.
        limit: usize,
        /// Exact UTF-8 request length observed before parsing.
        observed: usize,
    },
    /// Input was not valid JSON text.
    #[error("operation input is not valid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
}

pub(super) const fn base64_encoded_len(byte_len: usize) -> Option<usize> {
    match byte_len.checked_add(2) {
        Some(rounded) => match rounded.checked_div(3) {
            Some(groups) => groups.checked_mul(4),
            None => None,
        },
        None => None,
    }
}

const fn smallest_artifact_limit(first: usize, second: usize, third: usize) -> usize {
    let smaller = if first < second { first } else { second };
    if smaller < third { smaller } else { third }
}

const fn protocol_request_limit(
    document_source_bytes: usize,
    framing_and_request_id_bytes: usize,
) -> Option<usize> {
    match document_source_bytes.checked_mul(6) {
        Some(escaped_document_bytes) => {
            escaped_document_bytes.checked_add(framing_and_request_id_bytes)
        }
        None => None,
    }
}

impl OperationProtocolOperationV1 {
    pub(super) const fn kind(&self) -> ProtocolOperationKindV1 {
        match self {
            Self::Inspect(_) => ProtocolOperationKindV1::Inspect,
            Self::Validate(_) => ProtocolOperationKindV1::Validate,
            Self::Rewrite(_) => ProtocolOperationKindV1::Rewrite,
            Self::RenderArtifact(_) => ProtocolOperationKindV1::RenderArtifact,
            Self::ChemistryConvert(_) => ProtocolOperationKindV1::ChemistryConvert,
            Self::GenerateCoordinates(_) => ProtocolOperationKindV1::GenerateCoordinates,
        }
    }
}
