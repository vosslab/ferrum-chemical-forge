//! Request-owned document operation execution.

use super::*;

pub(super) fn execute_document_molecule_report<R: ChemistryRuntimeV1>(
    request: DocumentMoleculeReportRequestV1,
    runtime: &R,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    if request.expected_revision != 0 {
        return Err(ExecutionFailureV1::document_invalid(
            "expected_revision must be zero for a request-owned document".to_owned(),
        ));
    }
    let session = admit_document(&request.document)?;
    let observation = session.observe(0).map_err(|_| {
        ExecutionFailureV1::document_invalid("document observation was refused".to_owned())
    })?;
    super::super::molecule_report_core_v1::execute_document_molecule_report_v1(
        &observation,
        request,
        runtime,
    )
}

pub(super) fn execute_document_smarts_query<R: ChemistryRuntimeV1>(
    request: DocumentSmartsQueryRequestV1,
    runtime: &R,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    if request.document.expected_revision != 0 {
        return Err(ExecutionFailureV1::document_invalid(
            "expected_revision must be zero for a request-owned document".to_owned(),
        ));
    }
    let session = admit_document(&request.document.cdml)?;
    let observation = session.observe(0).map_err(|_| {
        ExecutionFailureV1::document_invalid("document observation was refused".to_owned())
    })?;
    super::super::smarts_query_core_v1::execute_document_smarts_query_v1(
        &observation,
        request,
        runtime,
    )
}

pub(super) fn execute_document_molecule_interchange_import_envelope(
    request_id: &str,
    request: DocumentMoleculeInterchangeImportRequestV1,
    runtime: &impl ChemistryRuntimeV1,
) -> OperationProtocolEnvelopeV1 {
    let descriptor =
        match crate::interchange_import_v1::InterchangeFormatRegistryV1::lookup_input_alias(
            &request.format_alias,
        ) {
            Ok(descriptor) => descriptor,
            Err(refusal) => {
                return admit_interchange_import_response_envelope(
                    request_id,
                    None,
                    interchange_import_error_envelope(request_id, refusal),
                );
            }
        };
    let source = crate::document_interchange_import_v1::admit_interchange_source_v1(
        descriptor,
        crate::document_interchange_import_v1::InterchangeSourceInputV1::RequestText(
            request.source_utf8.as_bytes(),
        ),
    );
    let source = match source {
        Ok(source) => source,
        Err(refusal) => {
            return admit_interchange_import_response_envelope(
                request_id,
                Some(descriptor),
                interchange_import_error_envelope(request_id, refusal),
            );
        }
    };
    let provenance = DocumentInterchangeProvenanceV1 {
        format_id: descriptor.format_id().to_owned(),
        profile_id: descriptor.profile_id().to_owned(),
        source_kind: source.source_kind(),
    };
    let preparation = crate::document_interchange_import_v1::prepare_interchange_new_document_v1(
        descriptor, &source, runtime, provenance,
    );
    let prepared = match preparation {
        Ok(prepared) => prepared,
        Err(refusal) => {
            return admit_interchange_import_response_envelope(
                request_id,
                Some(descriptor),
                interchange_import_error_envelope(request_id, refusal),
            );
        }
    };
    let outcome = OperationProtocolOutcomeV1::DocumentMoleculeInterchangeImport {
        summary: prepared.summary().clone(),
    };
    let envelope = OperationProtocolEnvelopeV1::Success(OperationProtocolResponseV1 {
        schema: ProtocolResponseSchemaV1::V1,
        request_id: request_id.to_owned(),
        outcome: outcome.clone(),
    });
    if !interchange_import_response_fits(descriptor, &envelope) {
        return admit_interchange_import_response_envelope(
            request_id,
            Some(descriptor),
            interchange_import_error_envelope(
                request_id,
                crate::InterchangeImportRefusalV1::for_reason(
                    crate::InterchangeImportRefusalReasonV1::ResponseBytesLimit,
                ),
            ),
        );
    }
    if let Err(refusal) = prepared.commit_and_take_session() {
        return admit_interchange_import_response_envelope(
            request_id,
            Some(descriptor),
            interchange_import_error_envelope(request_id, refusal),
        );
    }
    admit_interchange_import_response_envelope(request_id, Some(descriptor), envelope)
}

pub(super) fn interchange_import_error_envelope(
    request_id: &str,
    refusal: crate::InterchangeImportRefusalV1,
) -> OperationProtocolEnvelopeV1 {
    vector_error_response(
        Some(request_id.to_owned()),
        Some(ProtocolOperationKindV1::DocumentMoleculeInterchangeImport),
        ExecutionFailureV1::interchange_import_refusal(refusal),
    )
}

pub(super) fn interchange_import_response_fits(
    descriptor: &crate::InterchangeFormatDescriptorV1,
    envelope: &OperationProtocolEnvelopeV1,
) -> bool {
    let limit = descriptor.limits().max_response_bytes();
    canonical_protocol_envelope_json_v1(envelope).is_ok_and(|bytes| bytes.len() <= limit)
}

pub(super) fn admit_interchange_import_response_envelope(
    request_id: &str,
    descriptor: Option<&crate::InterchangeFormatDescriptorV1>,
    envelope: OperationProtocolEnvelopeV1,
) -> OperationProtocolEnvelopeV1 {
    if descriptor.is_none_or(|descriptor| interchange_import_response_fits(descriptor, &envelope)) {
        return envelope;
    }
    let limited = interchange_import_error_envelope(
        request_id,
        crate::InterchangeImportRefusalV1::for_reason(
            crate::InterchangeImportRefusalReasonV1::ResponseBytesLimit,
        ),
    );
    // The request identifier is already bounded before this operation is
    // dispatched, and this response contains only fixed protocol/interchange
    // enums. Keep the second exact measurement at this interchange boundary.
    assert!(interchange_import_response_fits(
        descriptor.expect("checked above"),
        &limited
    ));
    limited
}

pub(super) fn execute_document_operation<T>(
    source: &str,
    operation: impl FnOnce(&str) -> Result<T, CdmlError>,
) -> Result<T, ExecutionFailureV1> {
    admit_document(source)?;
    operation(source).map_err(|error| ExecutionFailureV1::document_invalid(error.to_string()))
}

pub(super) fn execute_render_artifact(
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

pub(super) fn admit_document(source: &str) -> Result<DocumentSession, ExecutionFailureV1> {
    load_document_utf8_bytes_with_budget(source.as_bytes(), local_cdml_ingress_format_v1())
        .map_err(|error| ExecutionFailureV1::document_admission(error.to_string()))
}

pub(super) fn map_render_error(error: DocumentNativeArtifactErrorV1) -> ExecutionFailureV1 {
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

pub(super) fn media_type(format: ProtocolArtifactFormatV1) -> &'static str {
    match format {
        ProtocolArtifactFormatV1::Svg => "image/svg+xml",
        ProtocolArtifactFormatV1::Pdf => "application/pdf",
        ProtocolArtifactFormatV1::PngOnePixelPerPointTransparent => "image/png",
    }
}
