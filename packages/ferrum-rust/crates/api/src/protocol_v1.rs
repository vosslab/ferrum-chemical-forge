//! Closed, stateless JSON operation protocol V1.
//!
//! This module owns the portable protocol boundary.  Each executor call takes
//! request-owned JSON text and returns response-owned DTOs; transient lower
//! layer sessions are dropped before the call returns.

use base64::Engine;
use ferrum_document::DocumentSession;
use schemars::{JsonSchema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    CdmlInspection, CdmlValidation, DocumentNativeArtifactProfileV1, RewriteCheck, inspect_cdml,
    load_document_utf8_bytes_with_budget, local_cdml_ingress_format_v1,
    prepare_document_native_artifact_v1, rewrite_cdml, validate_cdml, verify_cdml_rewrite,
};

/// Exact schema identifier accepted for V1 requests.
pub const OPERATION_PROTOCOL_REQUEST_SCHEMA_V1: &str = "ferrum-operation-request-v1";
/// Exact schema identifier emitted for V1 successful responses.
pub const OPERATION_PROTOCOL_RESPONSE_SCHEMA_V1: &str = "ferrum-operation-response-v1";
/// Exact schema identifier emitted for V1 protocol error responses.
pub const OPERATION_PROTOCOL_ERROR_SCHEMA_V1: &str = "ferrum-operation-error-v1";

// V1 returns one base64 completion, so its ceiling is the tightest accepted
// complete-artifact limit across the three existing native profiles.
const MAX_ARTIFACT_BYTES_V1: usize = smallest_artifact_limit(
    crate::LOCAL_SVG_COMPLETED_BYTES_V1,
    crate::LOCAL_PDF_COMPLETED_BYTES_V1,
    crate::LOCAL_PNG_ENCODED_BYTES_V1,
);
const MAX_ARTIFACT_BASE64_BYTES_V1: usize = base64_encoded_len(MAX_ARTIFACT_BYTES_V1)
    .expect("the accepted V1 artifact limit has a representable base64 expansion");

// A JSON string can encode every ASCII CDML byte as a six-byte `\\u00XX`
// escape.  The fixed allowance covers V1's closed framing and its opaque request
// ID; it is intentionally separate from the document-source policy.
const OPERATION_PROTOCOL_V1_FRAMING_AND_REQUEST_ID_UTF8_BYTES: usize = 4 * 1024;
/// Maximum UTF-8 request text accepted before any JSON allocation or parsing.
///
/// This is `LOCAL_CDML_SOURCE_UTF8_BYTES_V1 * 6` for worst-case valid JSON
/// escaping of an accepted CDML source, plus the named V1 framing/request-ID
/// allowance above. It is a transport allocation boundary, not a document or
/// corpus validity rule.
pub const OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1: usize = protocol_request_limit(
    crate::LOCAL_CDML_SOURCE_UTF8_BYTES_V1,
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

/// The four protocol operations admitted by V1.
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

/// Execute one request-owned JSON operation without retaining session or path state.
///
/// An over-budget request or invalid JSON has no response envelope, so this
/// returns [`OperationProtocolInputErrorV1`]. Every admitted, syntactically
/// decodable JSON value returns a success or typed error envelope.
pub fn execute_operation_v1(
    request_json: &str,
) -> Result<OperationProtocolEnvelopeV1, OperationProtocolInputErrorV1> {
    ensure_request_json_budget(request_json, OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1)?;
    let value = serde_json::from_str::<serde_json::Value>(request_json)?;
    let wire = match serde_json::from_value::<WireRequestEnvelopeV1>(value) {
        Ok(wire) => wire,
        Err(error) => {
            return Ok(error_response(
                None,
                None,
                OperationProtocolErrorCategoryV1::InvalidRequest,
                error,
            ));
        }
    };
    if wire.schema != OPERATION_PROTOCOL_REQUEST_SCHEMA_V1 {
        return Ok(error_response(
            Some(wire.request_id),
            None,
            OperationProtocolErrorCategoryV1::UnsupportedProtocolVersion,
            "unsupported protocol schema identifier",
        ));
    }
    let operation = match serde_json::from_value::<OperationProtocolOperationV1>(wire.operation) {
        Ok(operation) => operation,
        Err(error) => {
            return Ok(error_response(
                Some(wire.request_id),
                None,
                OperationProtocolErrorCategoryV1::InvalidRequest,
                error,
            ));
        }
    };
    let request = OperationProtocolRequestV1 {
        schema: ProtocolRequestSchemaV1::V1,
        request_id: wire.request_id,
        operation,
    };
    Ok(execute_admitted_operation(
        request.request_id,
        request.operation,
    ))
}

/// Produce the generated V1 schema document from the authoritative Rust DTOs.
#[must_use]
pub fn generated_operation_protocol_schema_v1() -> serde_json::Value {
    let mut generator = SchemaGenerator::default();
    let request = generator.subschema_for::<OperationProtocolRequestV1>();
    let success_response = generator.subschema_for::<OperationProtocolResponseV1>();
    let error_response = generator.subschema_for::<OperationProtocolErrorResponseV1>();
    let mut root = generator.into_root_schema_for::<OperationProtocolEnvelopeV1>();
    root.insert(
        "title".to_owned(),
        serde_json::Value::String("Ferrum operation protocol V1".to_owned()),
    );
    root.insert(
        "x-ferrum-roots".to_owned(),
        serde_json::json!({
            "request": request,
            "success_response": success_response,
            "error_response": error_response,
        }),
    );
    root.into()
}

/// Return the checked-in schema generated by [`generated_operation_protocol_schema_v1`].
#[must_use]
pub const fn operation_protocol_schema_v1() -> &'static str {
    include_str!("../protocol/ferrum-operation-v1.schema.json")
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRequestEnvelopeV1 {
    schema: String,
    request_id: String,
    operation: serde_json::Value,
}

fn execute_admitted_operation(
    request_id: String,
    operation: OperationProtocolOperationV1,
) -> OperationProtocolEnvelopeV1 {
    let kind = operation.kind();
    let result = match operation {
        OperationProtocolOperationV1::Inspect(request) => {
            execute_document_operation(&request.document, |document| {
                inspect_cdml(document).map(|report| OperationProtocolOutcomeV1::Inspect { report })
            })
        }
        OperationProtocolOperationV1::Validate(request) => {
            execute_document_operation(&request.document, |document| {
                validate_cdml(document, request.level == ProtocolValidationLevelV1::Typed).map(
                    |report| OperationProtocolOutcomeV1::Validate {
                        level: request.level,
                        report,
                    },
                )
            })
        }
        OperationProtocolOperationV1::Rewrite(request) => {
            execute_document_operation(&request.document, |document| {
                let rewritten = rewrite_cdml(document)?;
                let report = verify_cdml_rewrite(document)?;
                Ok(OperationProtocolOutcomeV1::Rewrite {
                    document: rewritten,
                    report,
                })
            })
        }
        OperationProtocolOperationV1::RenderArtifact(request) => {
            execute_render_artifact(&request.document, request.format)
        }
    };
    match result {
        Ok(outcome) => OperationProtocolEnvelopeV1::Success(OperationProtocolResponseV1 {
            schema: ProtocolResponseSchemaV1::V1,
            request_id,
            outcome,
        }),
        Err(error) => error_response(Some(request_id), Some(kind), error.category, error.message),
    }
}

fn execute_document_operation<T>(
    source: &str,
    operation: impl FnOnce(&str) -> Result<T, crate::CdmlError>,
) -> Result<T, ExecutionFailureV1> {
    admit_document(source)?;
    operation(source).map_err(|error| ExecutionFailureV1::document_invalid(error.to_string()))
}

fn execute_render_artifact(
    source: &str,
    format: ProtocolArtifactFormatV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let session = admit_document(source)?;
    let snapshot = session
        .snapshot()
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    let observation = session
        .observe(snapshot.revision())
        .map_err(|error| ExecutionFailureV1::document_invalid(error.to_string()))?;
    let profile = match format {
        ProtocolArtifactFormatV1::Svg => DocumentNativeArtifactProfileV1::Svg,
        ProtocolArtifactFormatV1::Pdf => DocumentNativeArtifactProfileV1::Pdf,
        ProtocolArtifactFormatV1::PngOnePixelPerPointTransparent => {
            DocumentNativeArtifactProfileV1::PngOnePixelPerPointTransparent
        }
    };
    let artifact = prepare_document_native_artifact_v1(
        &observation,
        snapshot.revision(),
        *snapshot.digest(),
        profile,
    )
    .map_err(map_render_error)?;
    let bytes = artifact.bytes();
    let encoded_len = base64_encoded_len(bytes.len()).ok_or_else(|| {
        ExecutionFailureV1::resource_limit("artifact base64 length is unrepresentable")
    })?;
    if encoded_len > MAX_ARTIFACT_BASE64_BYTES_V1 {
        return Err(ExecutionFailureV1::resource_limit(
            "artifact base64 completion exceeds the derived V1 response limit",
        ));
    }
    Ok(OperationProtocolOutcomeV1::RenderArtifact {
        format,
        media_type: media_type(format).to_owned(),
        artifact_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
    })
}

fn admit_document(source: &str) -> Result<DocumentSession, ExecutionFailureV1> {
    load_document_utf8_bytes_with_budget(source.as_bytes(), local_cdml_ingress_format_v1())
        .map_err(|error| ExecutionFailureV1::document_admission(error.to_string()))
}

fn map_render_error(error: crate::DocumentNativeArtifactErrorV1) -> ExecutionFailureV1 {
    match error {
        crate::DocumentNativeArtifactErrorV1::ExcludedRoots
        | crate::DocumentNativeArtifactErrorV1::PageDimension { .. } => {
            ExecutionFailureV1::render_unsupported(error.to_string())
        }
        crate::DocumentNativeArtifactErrorV1::Svg(ref source)
            if matches!(
                source,
                ferrum_render::SvgRenderError::OutputBudgetExceeded { .. }
                    | ferrum_render::SvgRenderError::ResourceExhausted
            ) =>
        {
            ExecutionFailureV1::resource_limit(error.to_string())
        }
        crate::DocumentNativeArtifactErrorV1::Pdf(ref source)
            if matches!(
                source,
                ferrum_render::PdfRenderError::OutputBudgetExceeded { .. }
                    | ferrum_render::PdfRenderError::ComplexityLimitExceeded { .. }
                    | ferrum_render::PdfRenderError::ComplexityCountOverflow { .. }
                    | ferrum_render::PdfRenderError::ResourceExhausted
            ) =>
        {
            ExecutionFailureV1::resource_limit(error.to_string())
        }
        crate::DocumentNativeArtifactErrorV1::Png(ref source)
            if matches!(
                source,
                ferrum_render::PngRenderError::EncodedOutputLimit { .. }
                    | ferrum_render::PngRenderError::RasterAllocationLimit { .. }
            ) =>
        {
            ExecutionFailureV1::resource_limit(error.to_string())
        }
        _ => ExecutionFailureV1::render_failed(error.to_string()),
    }
}

fn media_type(format: ProtocolArtifactFormatV1) -> &'static str {
    match format {
        ProtocolArtifactFormatV1::Svg => "image/svg+xml",
        ProtocolArtifactFormatV1::Pdf => "application/pdf",
        ProtocolArtifactFormatV1::PngOnePixelPerPointTransparent => "image/png",
    }
}

const fn base64_encoded_len(byte_len: usize) -> Option<usize> {
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

fn ensure_request_json_budget(
    request_json: &str,
    limit: usize,
) -> Result<(), OperationProtocolInputErrorV1> {
    let observed = request_json.len();
    if observed > limit {
        return Err(OperationProtocolInputErrorV1::ResourceLimit { limit, observed });
    }
    Ok(())
}

fn error_response(
    request_id: Option<String>,
    operation: Option<ProtocolOperationKindV1>,
    category: OperationProtocolErrorCategoryV1,
    message: impl ToString,
) -> OperationProtocolEnvelopeV1 {
    OperationProtocolEnvelopeV1::Error(OperationProtocolErrorResponseV1 {
        schema: ProtocolErrorSchemaV1::V1,
        request_id,
        error: OperationProtocolErrorV1 {
            category,
            operation,
            message: message.to_string(),
        },
    })
}

#[derive(Debug)]
struct ExecutionFailureV1 {
    category: OperationProtocolErrorCategoryV1,
    message: String,
}

impl ExecutionFailureV1 {
    fn document_admission(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::DocumentAdmissionFailed,
            message,
        }
    }

    fn document_invalid(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::DocumentInvalid,
            message,
        }
    }

    fn render_unsupported(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::RenderUnsupported,
            message,
        }
    }

    fn render_failed(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::RenderFailed,
            message,
        }
    }

    fn resource_limit(message: impl Into<String>) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ResourceLimit,
            message: message.into(),
        }
    }

    fn internal(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::InternalFailure,
            message,
        }
    }
}

impl OperationProtocolOperationV1 {
    const fn kind(&self) -> ProtocolOperationKindV1 {
        match self {
            Self::Inspect(_) => ProtocolOperationKindV1::Inspect,
            Self::Validate(_) => ProtocolOperationKindV1::Validate,
            Self::Rewrite(_) => ProtocolOperationKindV1::Rewrite,
            Self::RenderArtifact(_) => ProtocolOperationKindV1::RenderArtifact,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CDML: &str = "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom></molecule></cdml>";

    #[test]
    fn inspect_echoes_the_admitted_opaque_request_id() {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "opaque: request id",
            "operation": {"kind": "document.inspect", "document": CDML},
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("inspection should succeed");
        };
        assert_eq!(response.request_id, "opaque: request id");
        assert!(matches!(
            response.outcome,
            OperationProtocolOutcomeV1::Inspect { .. }
        ));
    }

    #[test]
    fn unknown_schema_and_kind_are_closed_before_document_execution() {
        let version = serde_json::json!({
            "schema": "ferrum-operation-request-v2",
            "request_id": "v2",
            "operation": {"kind": "document.inspect", "document": "not CDML"},
        });
        let kind = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "future",
            "operation": {"kind": "document.future", "document": "not CDML"},
        });
        for (request, category) in [
            (
                version,
                OperationProtocolErrorCategoryV1::UnsupportedProtocolVersion,
            ),
            (kind, OperationProtocolErrorCategoryV1::InvalidRequest),
        ] {
            let response = execute_operation_v1(&request.to_string()).expect("JSON input");
            let OperationProtocolEnvelopeV1::Error(response) = response else {
                panic!("unknown schema or operation must be refused");
            };
            assert_eq!(response.error.category, category);
        }
    }

    #[test]
    fn rewrite_result_has_a_structural_rewrite_check() {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "rewrite",
            "operation": {"kind": "document.rewrite", "document": CDML},
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("rewrite should succeed");
        };
        let OperationProtocolOutcomeV1::Rewrite { document, report } = response.outcome else {
            panic!("rewrite outcome expected");
        };
        assert!(report.valid);
        assert!(document.contains("cdml"));
    }

    #[test]
    fn artifact_result_declares_complete_svg_media_type() {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "svg",
            "operation": {"kind": "document.render_artifact", "document": CDML, "format": "svg"},
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("SVG should succeed");
        };
        let OperationProtocolOutcomeV1::RenderArtifact {
            media_type,
            artifact_base64,
            ..
        } = response.outcome
        else {
            panic!("artifact outcome expected");
        };
        assert_eq!(media_type, "image/svg+xml");
        let artifact = base64::engine::general_purpose::STANDARD
            .decode(artifact_base64)
            .expect("base64 artifact");
        assert!(artifact.starts_with(b"<svg"));
    }

    #[test]
    fn request_ingress_limit_rejects_before_json_parsing() {
        let error = ensure_request_json_budget("012345", 5).expect_err("limit refusal");
        assert!(matches!(
            error,
            OperationProtocolInputErrorV1::ResourceLimit {
                limit: 5,
                observed: 6,
            }
        ));
    }
}
