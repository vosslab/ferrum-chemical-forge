//! Protocol execution and its direct behavioral tests.

use base64::Engine;
use ferrum_chemistry::UnavailableChemEngine;
use ferrum_document::{
    CdmlError, DocumentObjectIdV1, DocumentSession, InterchangeCodecErrorV1, InterchangeFormatV1,
    MoleculeCoordinateBatchUpdateV1, SessionOperation, TypedClass, TypedDocument,
    build_molecule_coordinate_update_v1, decode_interchange_v1, encode_interchange_v1,
    inspect_cdml, load_document_utf8_bytes_with_budget, local_cdml_ingress_format_v1, rewrite_cdml,
    validate_cdml, verify_cdml_rewrite,
};
use ferrum_render::{
    DocumentNativeArtifactErrorV1, DocumentNativeArtifactProfileV1,
    prepare_document_native_artifact_v1,
};
use serde::Deserialize;

use super::dto::*;
use super::runtime::{ChemistryRuntimeErrorV1, ChemistryRuntimeV1, NoChemistryRuntimeV1};
#[cfg(test)]
use super::schema::generated_operation_protocol_schema_v1;

/// Execute one request-owned JSON operation without retaining session or path state.
pub fn execute_operation_v1(
    request_json: &str,
) -> Result<OperationProtocolEnvelopeV1, OperationProtocolInputErrorV1> {
    execute_operation_with_runtime_v1(request_json, &NoChemistryRuntimeV1)
}

/// Execute one request-owned JSON operation with an out-of-band chemistry capability.
///
/// The protocol stays portable because the capability and any adapter location
/// never become protocol values. Existing non-chemistry operations intentionally
/// have identical behavior with every runtime.
pub fn execute_operation_with_runtime_v1<R: ChemistryRuntimeV1>(
    request_json: &str,
    runtime: &R,
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
    if wire.request_id.len() > MAX_REQUEST_ID_UTF8_BYTES_V1 {
        return Ok(error_response(
            None,
            None,
            OperationProtocolErrorCategoryV1::ResourceLimit,
            format!("request identifier exceeds the {MAX_REQUEST_ID_UTF8_BYTES_V1}-byte V1 limit"),
        ));
    }
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
        runtime,
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRequestEnvelopeV1 {
    schema: String,
    request_id: String,
    operation: serde_json::Value,
}

fn execute_admitted_operation<R: ChemistryRuntimeV1>(
    request_id: String,
    operation: OperationProtocolOperationV1,
    runtime: &R,
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
        OperationProtocolOperationV1::ChemistryConvert(request) => {
            execute_chemistry_convert(request, runtime)
        }
        OperationProtocolOperationV1::GenerateCoordinates(request) => {
            execute_generate_coordinates(&request.document, runtime)
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

fn execute_chemistry_convert<R: ChemistryRuntimeV1>(
    request: ChemistryConvertRequestV1,
    runtime: &R,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    if request.input.format == InterchangeFormatV1::Cdml
        && request.output_format == InterchangeFormatV1::Cdml
    {
        return execute_cdml_to_cdml_conversion(request);
    }
    runtime
        .with_engine(|engine| {
            let records =
                match decode_interchange_v1(engine, request.input.format, &request.input.text) {
                    Ok(records) => records,
                    Err(error) => return Ok(Err(map_conversion_error(error))),
                };
            let record_count = records.len();
            let text = match encode_interchange_v1(engine, request.output_format, &records) {
                Ok(text) => text,
                Err(error) => return Ok(Err(map_conversion_error(error))),
            };
            Ok(Ok(OperationProtocolOutcomeV1::ChemistryConvert {
                format: request.output_format,
                text,
                record_count,
            }))
        })
        .map_err(map_runtime_conversion_error)?
}

/// Complete CDML-to-CDML projection without acquiring a chemistry runtime.
///
/// The document interchange codec preserves its explicit refusal behavior for
/// nonrepresentable presentation or opaque content; the unavailable engine is
/// never reached for this closed same-format conversion.
fn execute_cdml_to_cdml_conversion(
    request: ChemistryConvertRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let engine = UnavailableChemEngine;
    let records = decode_interchange_v1(&engine, request.input.format, &request.input.text)
        .map_err(map_conversion_error)?;
    let record_count = records.len();
    let text = encode_interchange_v1(&engine, request.output_format, &records)
        .map_err(map_conversion_error)?;
    Ok(OperationProtocolOutcomeV1::ChemistryConvert {
        format: request.output_format,
        text,
        record_count,
    })
}

fn execute_generate_coordinates<R: ChemistryRuntimeV1>(
    source: &str,
    runtime: &R,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let mut session = admit_document(source)?;
    let snapshot = session
        .snapshot()
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    let observation = session
        .observe(snapshot.revision())
        .map_err(|error| ExecutionFailureV1::document_invalid(error.to_string()))?;
    let typed = TypedDocument::parse(snapshot.cdml())
        .map_err(|error| ExecutionFailureV1::document_invalid(error.to_string()))?;
    let projection = typed
        .core_projection()
        .map_err(|error| ExecutionFailureV1::coordinate(error.to_string()))?;
    let ids = projection
        .molecules()
        .iter()
        .map(molecule_document_id)
        .collect::<Result<Vec<_>, _>>()?;
    if ids.is_empty() {
        return Err(ExecutionFailureV1::coordinate(
            "coordinate generation requires at least one durable typed molecule".to_owned(),
        ));
    }
    let regenerated_molecule_count = ids.len();
    let batch = runtime
        .with_engine(|engine| {
            let updates = ids
                .iter()
                .map(|id| build_molecule_coordinate_update_v1(engine, &observation, id))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| ExecutionFailureV1::coordinate(error.to_string()));
            let updates = match updates {
                Ok(updates) => updates,
                Err(error) => return Ok(Err(error)),
            };
            Ok(MoleculeCoordinateBatchUpdateV1::new(
                snapshot.revision(),
                *snapshot.digest(),
                updates,
            )
            .map_err(|error| ExecutionFailureV1::coordinate(error.to_string())))
        })
        .map_err(map_runtime_error)??;
    session
        .submit(
            snapshot.revision(),
            SessionOperation::V1(
                ferrum_document::SessionOperationV1::SetMoleculeAtomPositionsBatch {
                    update: batch,
                },
            ),
        )
        .map_err(|error| ExecutionFailureV1::coordinate(error.to_string()))?;
    let document = session
        .snapshot()
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?
        .cdml()
        .to_owned();
    Ok(OperationProtocolOutcomeV1::GenerateCoordinates {
        document,
        regenerated_molecule_count,
    })
}

fn molecule_document_id(
    molecule: &ferrum_core::Molecule,
) -> Result<DocumentObjectIdV1, ExecutionFailureV1> {
    let source = molecule.source_id().ok_or_else(|| {
        ExecutionFailureV1::coordinate(
            "typed molecule lacks a durable source identifier".to_owned(),
        )
    })?;
    let encoded = source
        .as_str()
        .bytes()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let class = TypedClass::Molecule
        .name()
        .bytes()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    DocumentObjectIdV1::parse(format!(
        "ferrum-document-object-v1/{class}/source/{encoded}"
    ))
    .map_err(|error| ExecutionFailureV1::coordinate(error.to_string()))
}

fn map_runtime_error(error: ChemistryRuntimeErrorV1) -> ExecutionFailureV1 {
    match error {
        ChemistryRuntimeErrorV1::Unavailable => {
            ExecutionFailureV1::chemistry_unavailable(error.to_string())
        }
        ChemistryRuntimeErrorV1::Chemistry(error) => {
            ExecutionFailureV1::coordinate(error.to_string())
        }
    }
}

fn map_runtime_conversion_error(error: ChemistryRuntimeErrorV1) -> ExecutionFailureV1 {
    match error {
        ChemistryRuntimeErrorV1::Unavailable => {
            ExecutionFailureV1::chemistry_unavailable(error.to_string())
        }
        ChemistryRuntimeErrorV1::Chemistry(error) => {
            ExecutionFailureV1::conversion_failed(error.to_string())
        }
    }
}

fn map_conversion_error(error: InterchangeCodecErrorV1) -> ExecutionFailureV1 {
    match error {
        InterchangeCodecErrorV1::MultiRecordUnsupported { .. }
        | InterchangeCodecErrorV1::NonMolecularCdml
        | InterchangeCodecErrorV1::CdmlCoordinatesRequired { .. }
        | InterchangeCodecErrorV1::CdmlUnsupportedBond { .. }
        | InterchangeCodecErrorV1::CdmlSdfPropertiesUnsupported { .. } => {
            ExecutionFailureV1::conversion_unsupported(error.to_string())
        }
        InterchangeCodecErrorV1::InputTooLarge { .. }
        | InterchangeCodecErrorV1::OutputTooLarge { .. } => {
            ExecutionFailureV1::resource_limit(error.to_string())
        }
        _ => ExecutionFailureV1::conversion_failed(error.to_string()),
    }
}

fn execute_document_operation<T>(
    source: &str,
    operation: impl FnOnce(&str) -> Result<T, CdmlError>,
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

fn map_render_error(error: DocumentNativeArtifactErrorV1) -> ExecutionFailureV1 {
    match error {
        DocumentNativeArtifactErrorV1::ExcludedRoots
        | DocumentNativeArtifactErrorV1::PageDimension { .. } => {
            ExecutionFailureV1::render_unsupported(error.to_string())
        }
        DocumentNativeArtifactErrorV1::Svg(ref source)
            if matches!(
                source,
                ferrum_render::SvgRenderError::OutputBudgetExceeded { .. }
                    | ferrum_render::SvgRenderError::ResourceExhausted
            ) =>
        {
            ExecutionFailureV1::resource_limit(error.to_string())
        }
        DocumentNativeArtifactErrorV1::Pdf(ref source)
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
        DocumentNativeArtifactErrorV1::Png(ref source)
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

    fn chemistry_unavailable(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ChemistryUnavailable,
            message,
        }
    }

    fn conversion_failed(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ConversionFailed,
            message,
        }
    }

    fn conversion_unsupported(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ConversionUnsupported,
            message,
        }
    }

    fn coordinate(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::CoordinateGenerationFailed,
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

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_chemistry::{
        ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, Point2, SmilesMolecule,
    };

    struct CoordinateOnlyEngine;

    impl ChemEngine for CoordinateOnlyEngine {
        fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "smiles_to_molecule",
            })
        }

        fn generate_2d_coordinates(
            &self,
            molecule: &MolGraph,
        ) -> Result<Coordinates, ChemistryError> {
            let points = molecule
                .atoms()
                .iter()
                .map(|_| Point2::new(0.0, 0.0))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| ChemistryError::CoordinateGenerationFailed {
                    reason: error.to_string(),
                })?;
            Ok(Coordinates::new(points))
        }

        fn kekulize(
            &self,
            molecule: &MolGraph,
            _options: KekulizeOptions,
        ) -> Result<MolGraph, ChemistryError> {
            Ok(molecule.clone())
        }
    }

    struct CoordinateOnlyRuntime;

    impl ChemistryRuntimeV1 for CoordinateOnlyRuntime {
        fn with_engine<T>(
            &self,
            operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
        ) -> Result<T, ChemistryRuntimeErrorV1> {
            operation(&CoordinateOnlyEngine)
        }
    }

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

    #[test]
    fn request_identifier_exact_boundary_is_echoed_in_the_response() {
        let request_id = "r".repeat(MAX_REQUEST_ID_UTF8_BYTES_V1);
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": request_id,
            "operation": {"kind": "document.inspect", "document": CDML},
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("boundary identifier must be admitted");
        };
        assert_eq!(response.request_id.len(), MAX_REQUEST_ID_UTF8_BYTES_V1);
    }

    #[test]
    fn oversized_request_identifier_is_not_echoed_in_error_response() {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "r".repeat(MAX_REQUEST_ID_UTF8_BYTES_V1 + 1),
            "operation": {"kind": "document.inspect", "document": CDML},
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Error(response) = response else {
            panic!("oversized identifier must be refused");
        };
        assert_eq!(
            response.error.category,
            OperationProtocolErrorCategoryV1::ResourceLimit
        );
        assert_eq!(response.request_id, None);
    }

    #[test]
    fn chemistry_operations_refuse_without_leaking_runtime_details() {
        let requests = [
            serde_json::json!({
                "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
                "request_id": "convert-no-runtime",
                "operation": {
                    "kind": "chemistry.convert",
                    "input": {"format": "smiles", "text": "CCO"},
                    "output_format": "inchi_standard",
                },
            }),
            serde_json::json!({
                "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
                "request_id": "coords-no-runtime",
                "operation": {"kind": "document.generate_coordinates", "document": CDML},
            }),
        ];
        for request in requests {
            let response = execute_operation_v1(&request.to_string()).expect("JSON input");
            let OperationProtocolEnvelopeV1::Error(response) = response else {
                panic!("missing runtime must be a typed refusal");
            };
            assert_eq!(
                response.error.category,
                OperationProtocolErrorCategoryV1::ChemistryUnavailable
            );
            assert!(!response.error.message.contains('/'));
        }
    }

    #[test]
    fn cdml_to_cdml_conversion_completes_without_a_runtime() {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "cdml-no-runtime",
            "operation": {
                "kind": "chemistry.convert",
                "input": {"format": "cdml", "text": CDML},
                "output_format": "cdml",
            },
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("pure CDML conversion must not acquire a runtime: {response:?}");
        };
        let OperationProtocolOutcomeV1::ChemistryConvert { record_count, .. } = response.outcome
        else {
            panic!("CDML conversion outcome expected");
        };
        assert_eq!(record_count, 1);
    }

    #[test]
    fn convert_refuses_opaque_nested_cdml_instead_of_rebuilding_it_without_data() {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "opaque-cdml",
            "operation": {
                "kind": "chemistry.convert",
                "input": {
                    "format": "cdml",
                    "text": "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\" vendor=\"kept\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>"
                },
                "output_format": "cdml"
            }
        });
        let response =
            execute_operation_with_runtime_v1(&request.to_string(), &CoordinateOnlyRuntime)
                .expect("JSON input");
        let OperationProtocolEnvelopeV1::Error(response) = response else {
            panic!("opaque CDML must be refused rather than projected");
        };
        assert_eq!(
            response.error.category,
            OperationProtocolErrorCategoryV1::ConversionUnsupported
        );
    }

    #[test]
    fn schema_includes_the_additive_runtime_backed_operations() {
        let schema = generated_operation_protocol_schema_v1().to_string();
        assert!(schema.contains("chemistry.convert"));
        assert!(schema.contains("document.generate_coordinates"));
        assert!(schema.contains("conversion_unsupported"));
        assert!(schema.contains("coordinate_generation_failed"));
    }

    #[test]
    fn coordinate_generation_uses_one_injected_engine_capability() {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "coords-runtime",
            "operation": {"kind": "document.generate_coordinates", "document": CDML},
        });
        let response =
            execute_operation_with_runtime_v1(&request.to_string(), &CoordinateOnlyRuntime)
                .expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("runtime-backed coordinate generation should succeed: {response:?}");
        };
        let OperationProtocolOutcomeV1::GenerateCoordinates {
            document,
            regenerated_molecule_count,
        } = response.outcome
        else {
            panic!("coordinate outcome expected");
        };
        assert_eq!(regenerated_molecule_count, 1);
        assert!(document.contains("<cdml"));
    }

    #[test]
    fn coordinate_generation_commits_all_direct_molecules_as_one_outcome() {
        let document = "<cdml><molecule id=\"first\"><atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom></molecule><molecule id=\"second\"><atom id=\"b\" name=\"O\"><point x=\"30\" y=\"40\"/></atom></molecule></cdml>";
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "coords-two-molecules",
            "operation": {"kind": "document.generate_coordinates", "document": document},
        });
        let response =
            execute_operation_with_runtime_v1(&request.to_string(), &CoordinateOnlyRuntime)
                .expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("both molecules must commit as one coordinate outcome");
        };
        let OperationProtocolOutcomeV1::GenerateCoordinates {
            regenerated_molecule_count,
            ..
        } = response.outcome
        else {
            panic!("coordinate outcome expected");
        };
        assert_eq!(regenerated_molecule_count, 2);
    }

    #[test]
    fn coordinate_generation_refuses_invalid_later_molecule_without_outcome() {
        let document = "<cdml><molecule id=\"first\"><atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom></molecule><molecule id=\"second\"><atom id=\"b\"><point x=\"30\" y=\"40\"/></atom></molecule></cdml>";
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "coords-invalid-later",
            "operation": {"kind": "document.generate_coordinates", "document": document},
        });
        let response =
            execute_operation_with_runtime_v1(&request.to_string(), &CoordinateOnlyRuntime)
                .expect("JSON input");
        let OperationProtocolEnvelopeV1::Error(response) = response else {
            panic!("invalid later molecule must reject the complete batch");
        };
        assert_eq!(
            response.error.category,
            OperationProtocolErrorCategoryV1::CoordinateGenerationFailed
        );
    }
}
