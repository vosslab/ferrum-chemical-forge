//! Protocol execution and its direct behavioral tests.

use base64::Engine;
use ferrum_catalog_placement::{
    CatalogPlacementCategoryV1, CatalogPlacementErrorV1, CatalogPlacementRecoveryV1,
    resolve_catalog_molecule_placement_v1,
};
use ferrum_chemistry::UnavailableChemEngine;
use ferrum_document::{
    AdmittedSessionTransitionRefusalV1, CdmlError, DocumentFenceV1, DocumentObjectIdV1,
    DocumentSession, DocumentSessionError, InterchangeCodecErrorV1, InterchangeFormatV1,
    MoleculeCoordinateBatchUpdateV1, PresentationGesturePoint2V1, SessionOperation,
    SessionOperationOutcomeV1, SessionOperationV1, TransitionAuthorizationV1, TypedClass,
    TypedDocument, build_molecule_coordinate_update_v1, decode_interchange_v1,
    encode_interchange_v1, load_document_utf8_bytes_with_budget, local_cdml_ingress_format_v1,
    rewrite_cdml, validate_cdml, verify_cdml_rewrite,
};
use ferrum_document::{
    DocumentNativeArtifactErrorV1, DocumentNativeArtifactProfileV1,
    prepare_document_native_artifact_v1,
};
use ferrum_domain::{CatalogFamilyV1, catalog_manifest_v1, search_catalog_v1};
use serde::Deserialize;

use super::dto::*;
use super::runtime::{ChemistryRuntimeErrorV1, ChemistryRuntimeV1, NoChemistryRuntimeV1};
#[cfg(test)]
use super::schema::generated_operation_protocol_schema_v1;
use crate::{
    PresentationVectorGestureCategoryV1, PresentationVectorKindV1, ReactionDefinitionDispositionV1,
    ReactionGestureCategoryV1, ReactionGestureErrorV1, ReactionGestureRecoveryV1,
    ReactionMembershipPatchRequestV1, RenderInteractionGridSnapPolicyV1,
    RenderInteractionSessionV1, RenderInteractionSnapV1, begin_api_reaction_definition_delete_v1,
    begin_api_reaction_gesture_v1, begin_api_reaction_membership_patch_v1,
    begin_api_reaction_translation_v1, resolve_api_reaction_gesture_v1,
    resolve_api_reaction_lifecycle_v1, resolve_api_reaction_translation_v1,
};

#[path = "execution_chemistry.rs"]
mod execution_chemistry;
#[path = "execution_document.rs"]
mod execution_document;
#[path = "execution_failure.rs"]
mod execution_failure;
#[path = "execution_placement.rs"]
mod execution_placement;
#[path = "execution_presentation_author.rs"]
mod execution_presentation_author;
use execution_presentation_author::execute_presentation_author;
#[path = "execution_reaction.rs"]
mod execution_reaction;
#[cfg(test)]
#[path = "execution_tests.rs"]
mod execution_tests;

use super::document_hydrogen_materialization_v1::execute_document_molecule_hydrogen_materialize;
use execution_chemistry::*;
pub(crate) use execution_document::admit_document;
use execution_document::*;
pub(super) use execution_failure::ExecutionFailureV1;
pub(crate) use execution_placement::hex_digest;
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
    // The public constant keeps its established name; its value is the shared
    // operation response budget for every operation admitted below.
    execute_operation_with_runtime_and_shared_response_budget_v1(
        request_json,
        runtime,
        DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1,
    )
}

#[cfg(test)]
pub(crate) fn execute_operation_with_runtime_and_smarts_response_limit_for_test<
    R: ChemistryRuntimeV1,
>(
    request_json: &str,
    runtime: &R,
    response_limit: usize,
) -> Result<OperationProtocolEnvelopeV1, OperationProtocolInputErrorV1> {
    execute_operation_with_runtime_and_shared_response_budget_v1(
        request_json,
        runtime,
        response_limit,
    )
}

fn execute_operation_with_runtime_and_shared_response_budget_v1<R: ChemistryRuntimeV1>(
    request_json: &str,
    runtime: &R,
    shared_response_budget: usize,
) -> Result<OperationProtocolEnvelopeV1, OperationProtocolInputErrorV1> {
    let request = match admit_operation_request_v1(request_json)? {
        OperationProtocolAdmissionV1::Response(response) => return Ok(response),
        OperationProtocolAdmissionV1::Request(request) => request,
    };
    let operation_kind = request.operation.kind();
    let envelope = execute_admitted_operation(request.request_id, request.operation, runtime);
    Ok(admit_shared_response_budget_v1(
        envelope,
        operation_kind,
        shared_response_budget,
    ))
}

/// Result of the shared V1 envelope-admission stage.
pub(crate) enum OperationProtocolAdmissionV1 {
    /// One complete, schema-admitted operation ready for an execution context.
    Request(OperationProtocolRequestV1),
    /// One typed envelope refusal produced before an operation exists.
    Response(OperationProtocolEnvelopeV1),
}

/// Parse and admit the common V1 envelope before selecting an execution context.
pub(crate) fn admit_operation_request_v1(
    request_json: &str,
) -> Result<OperationProtocolAdmissionV1, OperationProtocolInputErrorV1> {
    ensure_request_json_budget(request_json, OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1)?;
    let value = serde_json::from_str::<serde_json::Value>(request_json)?;
    let wire = match serde_json::from_value::<WireRequestEnvelopeV1>(value) {
        Ok(wire) => wire,
        Err(error) => {
            return Ok(OperationProtocolAdmissionV1::Response(error_response(
                None,
                None,
                OperationProtocolErrorCategoryV1::InvalidRequest,
                error,
            )));
        }
    };
    if wire.request_id.len() > MAX_REQUEST_ID_UTF8_BYTES_V1 {
        return Ok(OperationProtocolAdmissionV1::Response(error_response(
            None,
            None,
            OperationProtocolErrorCategoryV1::ResourceLimit,
            format!("request identifier exceeds the {MAX_REQUEST_ID_UTF8_BYTES_V1}-byte V1 limit"),
        )));
    }
    if wire.schema != OPERATION_PROTOCOL_REQUEST_SCHEMA_V1 {
        return Ok(OperationProtocolAdmissionV1::Response(error_response(
            Some(wire.request_id),
            None,
            OperationProtocolErrorCategoryV1::UnsupportedProtocolVersion,
            "unsupported protocol schema identifier",
        )));
    }
    let operation = match serde_json::from_value::<OperationProtocolOperationV1>(wire.operation) {
        Ok(operation) => operation,
        Err(error) => {
            return Ok(OperationProtocolAdmissionV1::Response(error_response(
                Some(wire.request_id),
                None,
                OperationProtocolErrorCategoryV1::InvalidRequest,
                error,
            )));
        }
    };
    Ok(OperationProtocolAdmissionV1::Request(
        OperationProtocolRequestV1 {
            schema: ProtocolRequestSchemaV1::V1,
            request_id: wire.request_id,
            operation,
        },
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
            execute_document_inspect(&request.document)
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
        OperationProtocolOperationV1::PresentationAuthor(request) => {
            execute_presentation_author(request)
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
        OperationProtocolOperationV1::DocumentAtomOxidationObserve(request) => {
            execute_document_atom_oxidation_observe(request)
        }
        OperationProtocolOperationV1::DocumentMoleculeHydrogenMaterialize(request) => {
            execute_document_molecule_hydrogen_materialize(request)
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
        Err(error) => operation_error_response(Some(request_id), Some(kind), error),
    };
    envelope
}

pub(crate) fn admit_shared_response_budget_v1(
    envelope: OperationProtocolEnvelopeV1,
    operation: ProtocolOperationKindV1,
    shared_response_budget: usize,
) -> OperationProtocolEnvelopeV1 {
    if !uses_shared_response_budget_v1(operation) {
        return envelope;
    }
    if canonical_protocol_envelope_json_v1(&envelope)
        .is_ok_and(|bytes| bytes.len() <= shared_response_budget)
    {
        return envelope;
    }
    let request_id = match envelope {
        OperationProtocolEnvelopeV1::Success(response) => Some(response.request_id),
        OperationProtocolEnvelopeV1::Error(response) => response.request_id,
    };
    response_size_exceeded_error(request_id, operation)
}

/// Closed V1 admission policy for operations whose result volume is bounded.
///
/// The current shared budget covers molecule reports, SMARTS result enumeration, oxidation
/// observation, and explicit-hydrogen materialization. New operations must opt in here deliberately rather than
/// inheriting a bound from a similarly shaped response.
const fn uses_shared_response_budget_v1(operation: ProtocolOperationKindV1) -> bool {
    matches!(
        operation,
        ProtocolOperationKindV1::DocumentMoleculeReport
            | ProtocolOperationKindV1::DocumentSmartsQuery
            | ProtocolOperationKindV1::DocumentAtomOxidationObserve
            | ProtocolOperationKindV1::DocumentMoleculeHydrogenMaterialize
    )
}

pub(crate) fn canonical_protocol_envelope_json_v1(
    envelope: &OperationProtocolEnvelopeV1,
) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(envelope)
}

fn response_size_exceeded_error(
    request_id: Option<String>,
    operation: ProtocolOperationKindV1,
) -> OperationProtocolEnvelopeV1 {
    OperationProtocolEnvelopeV1::Error(OperationProtocolErrorResponseV1 {
        schema: ProtocolErrorSchemaV1::V1,
        request_id,
        error: OperationProtocolErrorV1 {
            category: OperationProtocolErrorCategoryV1::ResourceLimit,
            operation: Some(operation),
            message: "response_size_exceeded".to_owned(),
            resource_limit: Some(ProtocolResourceLimitRefusalV1 {
                reason: ProtocolResourceLimitReasonV1::ResponseSizeExceeded,
                recovery: ProtocolResourceLimitRecoveryV1::ReduceRequestedResult,
            }),
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        },
    })
}

pub(crate) fn ensure_request_json_budget(
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
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        },
    })
}

pub(crate) fn operation_error_response(
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
            resource_limit: failure.resource_limit,
            presentation_author_refusal: failure.presentation_author_refusal,
            catalog_placement_refusal: failure.catalog_placement_refusal,
            reaction_refusal: failure.reaction_refusal,
        },
    })
}
