//! Protocol execution and its direct behavioral tests.

use base64::Engine;
use ferrum_chemistry::UnavailableChemEngine;
use ferrum_document::{
    CdmlError, DocumentFenceV1, DocumentObjectIdV1, DocumentSession, InterchangeCodecErrorV1,
    InterchangeFormatV1, MoleculeCoordinateBatchUpdateV1, PresentationGesturePoint2V1,
    SessionOperation, TypedClass, TypedDocument, build_molecule_coordinate_update_v1,
    decode_interchange_v1, encode_interchange_v1, inspect_cdml,
    load_document_utf8_bytes_with_budget, local_cdml_ingress_format_v1, rewrite_cdml,
    validate_cdml, verify_cdml_rewrite,
};
use ferrum_domain::{CatalogFamilyV1, catalog_manifest_v1, search_catalog_v1};
use ferrum_render::{
    DocumentNativeArtifactErrorV1, DocumentNativeArtifactProfileV1, DocumentRenderOutcomeV1,
    compose_document_render_plan_v1, document_observation_from_accepted_operation_v1,
    prepare_document_native_artifact_v1,
};
use serde::Deserialize;

use super::dto::*;
use super::runtime::{ChemistryRuntimeErrorV1, ChemistryRuntimeV1, NoChemistryRuntimeV1};
#[cfg(test)]
use super::schema::generated_operation_protocol_schema_v1;
use crate::{
    CatalogPlacementCategoryV2, CatalogPlacementErrorV2, CatalogPlacementRecoveryV2,
    PresentationVectorGestureCategoryV1, PresentationVectorGestureErrorV1,
    PresentationVectorGestureRecoveryV1, PresentationVectorKindV1, ReactionDefinitionDispositionV1,
    ReactionGestureCategoryV1, ReactionGestureErrorV1, ReactionGestureRecoveryV1,
    ReactionMembershipPatchRequestV1, RenderInteractionGridSnapPolicyV1,
    RenderInteractionSessionV1, RenderInteractionSnapV1, begin_api_catalog_placement_v2,
    begin_api_presentation_vector_gesture_v1, begin_api_reaction_definition_delete_v1,
    begin_api_reaction_gesture_v1, begin_api_reaction_membership_patch_v1,
    begin_api_reaction_translation_v1, commit_api_catalog_placement_v2,
    commit_api_presentation_vector_gesture_v1, commit_api_reaction_gesture_v1,
    commit_api_reaction_lifecycle_v1, commit_api_reaction_translation_v1,
    prepare_api_catalog_placement_v2, prepare_api_presentation_vector_gesture_v1,
    prepare_api_reaction_gesture_v1, prepare_api_reaction_lifecycle_v1,
    prepare_api_reaction_translation_v1, preview_api_catalog_placement_v2,
    preview_api_presentation_vector_gesture_v1, preview_api_reaction_translation_v1,
};

#[path = "execution_chemistry.rs"]
mod execution_chemistry;
#[path = "execution_document.rs"]
mod execution_document;
#[path = "execution_failure.rs"]
mod execution_failure;
#[path = "execution_placement.rs"]
mod execution_placement;
#[path = "execution_reaction.rs"]
mod execution_reaction;
#[cfg(test)]
#[path = "execution_tests.rs"]
mod execution_tests;

use execution_chemistry::*;
use execution_document::*;
pub(super) use execution_failure::ExecutionFailureV1;
use execution_placement::*;
use execution_reaction::*;

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
pub(crate) fn execute_operation_with_runtime_v1<R: ChemistryRuntimeV1>(
    request_json: &str,
    runtime: &R,
) -> Result<OperationProtocolEnvelopeV1, OperationProtocolInputErrorV1> {
    execute_operation_with_runtime_and_smarts_response_limit_v1(
        request_json,
        runtime,
        DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1,
    )
}

#[cfg(any(test, feature = "response-size-e2e-harness"))]
pub(crate) fn execute_operation_with_runtime_and_smarts_response_limit_for_test<
    R: ChemistryRuntimeV1,
>(
    request_json: &str,
    runtime: &R,
    response_limit: usize,
) -> Result<OperationProtocolEnvelopeV1, OperationProtocolInputErrorV1> {
    execute_operation_with_runtime_and_smarts_response_limit_v1(
        request_json,
        runtime,
        response_limit,
    )
}

fn execute_operation_with_runtime_and_smarts_response_limit_v1<R: ChemistryRuntimeV1>(
    request_json: &str,
    runtime: &R,
    smarts_response_limit: usize,
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
    let envelope = execute_admitted_operation(request.request_id, request.operation, runtime);
    Ok(admit_smarts_response_envelope_v1(
        envelope,
        smarts_response_limit,
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
        OperationProtocolOperationV1::PresentationVectorCreate(request) => {
            execute_presentation_vector_create(request)
        }
        OperationProtocolOperationV1::CatalogList(request) => execute_catalog_list(request),
        OperationProtocolOperationV1::CatalogInsert(request) => execute_catalog_insert(request),
        OperationProtocolOperationV1::ReactionCreate(request) => execute_reaction_create(request),
        OperationProtocolOperationV1::ReactionList(request) => execute_reaction_list(request),
        OperationProtocolOperationV1::ReactionObserve(request) => {
            execute_reaction_observe(request, false)
        }
        OperationProtocolOperationV1::ReactionSelect(request) => {
            execute_reaction_observe(request, true)
        }
        OperationProtocolOperationV1::ReactionPatchMembership(request) => {
            execute_reaction_patch(request)
        }
        OperationProtocolOperationV1::ReactionDeleteDefinition(request) => {
            execute_reaction_delete(request)
        }
        OperationProtocolOperationV1::ReactionTranslate(request) => {
            execute_reaction_translate(request)
        }
        OperationProtocolOperationV1::DocumentMoleculeReport(request) => {
            execute_document_molecule_report(request, runtime)
        }
        OperationProtocolOperationV1::DocumentSmartsQuery(request) => {
            execute_document_smarts_query(request, runtime)
        }
        OperationProtocolOperationV1::DocumentMoleculeInterchangeImport(request) => {
            return execute_document_molecule_interchange_import_envelope(
                &request_id,
                request,
                runtime,
            );
        }
    };
    let envelope = match result {
        Ok(outcome) => OperationProtocolEnvelopeV1::Success(OperationProtocolResponseV1 {
            schema: ProtocolResponseSchemaV1::V1,
            request_id,
            outcome,
        }),
        Err(error) => vector_error_response(Some(request_id), Some(kind), error),
    };
    if matches!(kind, ProtocolOperationKindV1::DocumentSmartsQuery) {
        admit_smarts_response_envelope_v1(envelope, DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1)
    } else {
        envelope
    }
}

fn admit_smarts_response_envelope_v1(
    envelope: OperationProtocolEnvelopeV1,
    response_limit: usize,
) -> OperationProtocolEnvelopeV1 {
    let is_smarts_query = matches!(
        &envelope,
        OperationProtocolEnvelopeV1::Success(OperationProtocolResponseV1 {
            outcome: OperationProtocolOutcomeV1::DocumentSmartsQuery { .. },
            ..
        }) | OperationProtocolEnvelopeV1::Error(OperationProtocolErrorResponseV1 {
            error: OperationProtocolErrorV1 {
                operation: Some(ProtocolOperationKindV1::DocumentSmartsQuery),
                ..
            },
            ..
        })
    );
    if !is_smarts_query {
        return envelope;
    }
    if canonical_protocol_envelope_json_v1(&envelope)
        .is_ok_and(|bytes| bytes.len() <= response_limit)
    {
        return envelope;
    }
    let request_id = match envelope {
        OperationProtocolEnvelopeV1::Success(response) => Some(response.request_id),
        OperationProtocolEnvelopeV1::Error(response) => response.request_id,
    };
    response_size_exceeded_error(request_id)
}

pub(crate) fn canonical_protocol_envelope_json_v1(
    envelope: &OperationProtocolEnvelopeV1,
) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(envelope)
}

fn response_size_exceeded_error(request_id: Option<String>) -> OperationProtocolEnvelopeV1 {
    OperationProtocolEnvelopeV1::Error(OperationProtocolErrorResponseV1 {
        schema: ProtocolErrorSchemaV1::V1,
        request_id,
        error: OperationProtocolErrorV1 {
            category: OperationProtocolErrorCategoryV1::ResourceLimit,
            operation: Some(ProtocolOperationKindV1::DocumentSmartsQuery),
            message: "response_size_exceeded".to_owned(),
            resource_limit_reason: Some(ProtocolResourceLimitReasonV1::ResponseSizeExceeded),
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        },
    })
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
            resource_limit_reason: None,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        },
    })
}

fn vector_error_response(
    request_id: Option<String>,
    operation: Option<ProtocolOperationKindV1>,
    failure: ExecutionFailureV1,
) -> OperationProtocolEnvelopeV1 {
    OperationProtocolEnvelopeV1::Error(OperationProtocolErrorResponseV1 {
        schema: ProtocolErrorSchemaV1::V1,
        request_id,
        error: OperationProtocolErrorV1 {
            category: failure.category,
            operation,
            message: failure.message,
            resource_limit_reason: None,
            presentation_vector_refusal: failure.presentation_vector_refusal,
            catalog_placement_refusal: failure.catalog_placement_refusal,
            reaction_refusal: failure.reaction_refusal,
        },
    })
}
