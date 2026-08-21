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
            return execute_document_molecule_interchange_import_envelope(&request_id, request);
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

fn execute_document_molecule_report<R: ChemistryRuntimeV1>(
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
    super::molecule_report_core_v1::execute_document_molecule_report_v1(
        &observation,
        request,
        runtime,
    )
}

fn execute_document_smarts_query<R: ChemistryRuntimeV1>(
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
    super::smarts_query_core_v1::execute_document_smarts_query_v1(&observation, request, runtime)
}

fn execute_document_molecule_interchange_import_envelope(
    request_id: &str,
    request: DocumentMoleculeInterchangeImportRequestV1,
) -> OperationProtocolEnvelopeV1 {
    if let Err(refusal) =
        crate::interchange_import_v1::InterchangeFormatRegistryV1::lookup_input_alias(
            &request.format_alias,
        )
    {
        return admit_cml_import_response_envelope(cml_import_error_envelope(request_id, refusal));
    }
    let prepared =
        match crate::cml_open_v1::prepare_cml_new_document_v1(request.cml_utf8.as_bytes()) {
            Ok(prepared) => prepared,
            Err(refusal) => {
                return admit_cml_import_response_envelope(cml_import_error_envelope(
                    request_id, refusal,
                ));
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
    if !cml_import_response_fits(&envelope) {
        return admit_cml_import_response_envelope(cml_import_error_envelope(
            request_id,
            crate::CmlImportRefusalV1::for_reason(
                crate::CmlImportRefusalReasonV1::ResponseBytesLimit,
            ),
        ));
    }
    if let Err(refusal) = prepared.commit_and_take_cdml() {
        return admit_cml_import_response_envelope(cml_import_error_envelope(request_id, refusal));
    }
    admit_cml_import_response_envelope(envelope)
}

fn cml_import_error_envelope(
    request_id: &str,
    refusal: crate::CmlImportRefusalV1,
) -> OperationProtocolEnvelopeV1 {
    vector_error_response(
        Some(request_id.to_owned()),
        Some(ProtocolOperationKindV1::DocumentMoleculeInterchangeImport),
        ExecutionFailureV1::cml_import_refusal(refusal),
    )
}

fn cml_import_response_fits(envelope: &OperationProtocolEnvelopeV1) -> bool {
    canonical_protocol_envelope_json_v1(envelope).is_ok_and(|bytes| {
        bytes.len() <= crate::interchange_import_v1::CML_IMPORT_RESPONSE_BUDGET_BYTES_V1
    })
}

fn admit_cml_import_response_envelope(
    envelope: OperationProtocolEnvelopeV1,
) -> OperationProtocolEnvelopeV1 {
    if cml_import_response_fits(&envelope) {
        return envelope;
    }
    let request_id = match envelope {
        OperationProtocolEnvelopeV1::Success(response) => response.request_id,
        OperationProtocolEnvelopeV1::Error(response) => response.request_id.unwrap_or_default(),
    };
    let limited = cml_import_error_envelope(
        &request_id,
        crate::CmlImportRefusalV1::for_reason(crate::CmlImportRefusalReasonV1::ResponseBytesLimit),
    );
    // The request identifier is already bounded before this operation is
    // dispatched, and this response contains only fixed protocol/CML enums.
    // Keep the second exact measurement here so the fallback is admitted by
    // this CML boundary rather than assumed to fit.
    assert!(
        cml_import_response_fits(&limited),
        "fixed CML response-limit refusal must fit the frozen response budget"
    );
    limited
}

fn reaction_session(
    document: String,
    expected_revision: u64,
    expected_digest_hex: String,
) -> Result<(RenderInteractionSessionV1, DocumentFenceV1, String), ExecutionFailureV1> {
    if expected_revision != 0 {
        return Err(ExecutionFailureV1::reaction_refusal(
            ReactionGestureErrorV1::StaleSnapshot,
        ));
    }
    let session = RenderInteractionSessionV1::new(admit_document(&document)?);
    let snapshot = session
        .snapshot()
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    let digest = parse_digest_hex(&expected_digest_hex)?;
    if snapshot.digest() != &digest {
        return Err(ExecutionFailureV1::reaction_refusal(
            ReactionGestureErrorV1::StaleSnapshot,
        ));
    }
    Ok((
        session,
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest()),
        hex_digest(snapshot.digest()),
    ))
}

fn execute_reaction_list(
    request: ReactionObservationRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let (session, fence, digest_hex) = reaction_session(
        request.document,
        request.expected_revision,
        request.expected_digest_hex,
    )?;
    let list = session
        .observe_reaction_list_v1(fence)
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    Ok(OperationProtocolOutcomeV1::ReactionList {
        input_revision: 0,
        next_input_expected_revision: 0,
        digest_hex,
        reactions: list.reactions().iter().map(reaction_summary).collect(),
    })
}

fn execute_reaction_observe(
    request: ReactionObserveRequestV1,
    select: bool,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let (session, fence, digest_hex) = reaction_session(
        request.document,
        request.expected_revision,
        request.expected_digest_hex,
    )?;
    let list = session
        .observe_reaction_list_v1(fence)
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    let reaction = list
        .reactions()
        .iter()
        .find(|value| value.reaction_id() == request.reaction_id)
        .ok_or_else(|| {
            ExecutionFailureV1::reaction_refusal(ReactionGestureErrorV1::InvalidRequest)
        })?;
    if select {
        let selection = session
            .select_reaction_v1(&list, reaction.reaction_id())
            .map_err(|_| {
                ExecutionFailureV1::reaction_refusal(ReactionGestureErrorV1::InvalidRequest)
            })?;
        session
            .validate_reaction_selection_v1(&selection)
            .map_err(|_| {
                ExecutionFailureV1::reaction_refusal(ReactionGestureErrorV1::SessionConflict)
            })?;
        Ok(OperationProtocolOutcomeV1::ReactionSelect {
            input_revision: 0,
            next_input_expected_revision: 0,
            digest_hex,
            reaction_id: selection.reaction_id().to_owned(),
            membership_digest: reaction.membership_digest().to_owned(),
        })
    } else {
        Ok(OperationProtocolOutcomeV1::ReactionObserve {
            input_revision: 0,
            next_input_expected_revision: 0,
            digest_hex,
            reaction: reaction_summary(reaction),
        })
    }
}

fn reaction_summary(value: &crate::ReactionObservationV1) -> ReactionObservationSummaryV1 {
    let bounds = |value: crate::RenderInteractionBoundsV1| ReactionBoundsSummaryV1 {
        left: value.left(),
        top: value.top(),
        right: value.right(),
        bottom: value.bottom(),
    };
    ReactionObservationSummaryV1 {
        reaction_id: value.reaction_id().to_owned(),
        source_order: value.source_order(),
        disposition: match value.disposition() {
            ReactionDefinitionDispositionV1::Strict => {
                ProtocolReactionDefinitionDispositionV1::Strict
            }
            ReactionDefinitionDispositionV1::DisplayOnly => {
                ProtocolReactionDefinitionDispositionV1::DisplayOnly
            }
        },
        diagnostics: value
            .diagnostics()
            .iter()
            .map(|value| format!("{value:?}").to_lowercase())
            .collect(),
        membership_digest: value.membership_digest().to_owned(),
        members: value
            .members()
            .iter()
            .map(|member| ReactionMemberSummaryV1 {
                identifier: member.identifier().to_owned(),
                role: member.role().local_name().to_owned(),
                role_ordinal: member.role_ordinal(),
                source_order: member.source_order(),
                bounds: member.bounds().map(bounds),
            })
            .collect(),
        union_bounds: value.union_bounds().map(bounds),
    }
}

fn execute_reaction_create(
    request: ReactionCreateRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    if request.expected_revision != 0 {
        return Err(ExecutionFailureV1::reaction_refusal(
            ReactionGestureErrorV1::StaleSnapshot,
        ));
    }
    let mut session = admit_document(&request.document)?;
    let digest = parse_digest_hex(&request.expected_digest_hex)?;
    let snapshot = session
        .snapshot()
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    if snapshot.digest() != &digest {
        return Err(ExecutionFailureV1::reaction_refusal(
            ReactionGestureErrorV1::StaleSnapshot,
        ));
    }
    let source_fence = DocumentFenceV1::new(snapshot.revision(), *snapshot.digest());
    let request = ferrum_document_render::ReactionCreateRequestV1::new(
        request.expected_revision,
        request.reactants,
        request.products,
        request.arrow,
        request.conditions,
        request.pluses,
    )
    .map_err(ExecutionFailureV1::reaction_refusal)?;
    let gesture = begin_api_reaction_gesture_v1(&session, source_fence, request)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let mut prepared = prepare_api_reaction_gesture_v1(&mut session, &gesture)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let accepted = commit_api_reaction_gesture_v1(&mut session, &mut prepared)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let snapshot = accepted.result().observation().snapshot();
    Ok(OperationProtocolOutcomeV1::ReactionCreate {
        document: snapshot.cdml().to_owned(),
        reaction_id: accepted.reaction_id().to_owned(),
        input_revision: 0,
        committed_revision: snapshot.revision(),
        next_input_expected_revision: 0,
        digest_hex: hex_digest(snapshot.digest()),
    })
}

fn select_lifecycle_reaction(
    session: &RenderInteractionSessionV1,
    fence: DocumentFenceV1,
    reaction_id: &str,
) -> Result<crate::ReactionSelectionV1, ExecutionFailureV1> {
    let list = session.observe_reaction_list_v1(fence).map_err(|_| {
        ExecutionFailureV1::reaction_refusal(ReactionGestureErrorV1::SessionConflict)
    })?;
    session
        .select_reaction_v1(&list, reaction_id)
        .map_err(|error| {
            ExecutionFailureV1::reaction_refusal(match error {
                crate::RenderInteractionErrorV1::ForeignSession => {
                    ReactionGestureErrorV1::ForeignSession
                }
                crate::RenderInteractionErrorV1::StaleRevision
                | crate::RenderInteractionErrorV1::StaleDigest => {
                    ReactionGestureErrorV1::StaleSnapshot
                }
                crate::RenderInteractionErrorV1::DisplayOnly => {
                    ReactionGestureErrorV1::LegacyDefinitionNotEditable
                }
                crate::RenderInteractionErrorV1::SelectionChanged => {
                    ReactionGestureErrorV1::MissingReaction
                }
                crate::RenderInteractionErrorV1::SessionConflict => {
                    ReactionGestureErrorV1::SessionConflict
                }
                _ => ReactionGestureErrorV1::RendererExclusion,
            })
        })
}

fn execute_reaction_patch(
    request: ReactionPatchMembershipRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let (mut session, fence, _) = reaction_session(
        request.document,
        request.expected_revision,
        request.expected_digest_hex,
    )?;
    let selection = select_lifecycle_reaction(&session, fence, &request.reaction_id)?;
    let patch = ReactionMembershipPatchRequestV1::new(
        request.expected_revision,
        request.reactants,
        request.products,
        request.arrow,
        request.conditions,
        request.pluses,
    )
    .map_err(ExecutionFailureV1::reaction_refusal)?;
    let gesture = begin_api_reaction_membership_patch_v1(&session, &selection, patch)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let mut prepared = prepare_api_reaction_lifecycle_v1(&mut session, &gesture)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let accepted = commit_api_reaction_lifecycle_v1(&mut session, &mut prepared)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let snapshot = accepted.result().observation().snapshot();
    Ok(OperationProtocolOutcomeV1::ReactionPatchMembership {
        document: snapshot.cdml().to_owned(),
        reaction_id: accepted.reaction_id().to_owned(),
        input_revision: 0,
        committed_revision: snapshot.revision(),
        next_input_expected_revision: 0,
        digest_hex: hex_digest(snapshot.digest()),
    })
}

fn execute_reaction_delete(
    request: ReactionObserveRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let (mut session, fence, _) = reaction_session(
        request.document,
        request.expected_revision,
        request.expected_digest_hex,
    )?;
    let selection = select_lifecycle_reaction(&session, fence, &request.reaction_id)?;
    let gesture = begin_api_reaction_definition_delete_v1(&session, &selection)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let mut prepared = prepare_api_reaction_lifecycle_v1(&mut session, &gesture)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let accepted = commit_api_reaction_lifecycle_v1(&mut session, &mut prepared)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let snapshot = accepted.result().observation().snapshot();
    Ok(OperationProtocolOutcomeV1::ReactionDeleteDefinition {
        document: snapshot.cdml().to_owned(),
        reaction_id: accepted.reaction_id().to_owned(),
        input_revision: 0,
        committed_revision: snapshot.revision(),
        next_input_expected_revision: 0,
        digest_hex: hex_digest(snapshot.digest()),
    })
}

fn execute_reaction_translate(
    request: ReactionTranslateRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let (mut session, fence, _) = reaction_session(
        request.document,
        request.expected_revision,
        request.expected_digest_hex,
    )?;
    let selection = select_lifecycle_reaction(&session, fence, &request.reaction_id)?;
    let snap = match request.snap {
        ProtocolReactionTranslationSnapV1::Free => RenderInteractionSnapV1::free(),
        ProtocolReactionTranslationSnapV1::ViewHexGrid => {
            RenderInteractionSnapV1::with_grid(RenderInteractionGridSnapPolicyV1::ViewHexGrid)
        }
    };
    let gesture = begin_api_reaction_translation_v1(
        &session,
        &selection,
        request.press_x,
        request.press_y,
        snap,
    )
    .map_err(ExecutionFailureV1::reaction_refusal)?;
    let preview = preview_api_reaction_translation_v1(
        &session,
        &gesture,
        request.pointer_x,
        request.pointer_y,
    )
    .map_err(ExecutionFailureV1::reaction_refusal)?;
    let mut prepared = prepare_api_reaction_translation_v1(&mut session, &gesture, &preview)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let accepted = commit_api_reaction_translation_v1(&mut session, &mut prepared)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let snapshot = accepted.result().observation().snapshot();
    Ok(OperationProtocolOutcomeV1::ReactionTranslate {
        document: snapshot.cdml().to_owned(),
        reaction_id: accepted.reaction_id().to_owned(),
        input_revision: 0,
        committed_revision: snapshot.revision(),
        next_input_expected_revision: 0,
        digest_hex: hex_digest(snapshot.digest()),
    })
}

fn execute_catalog_list(
    request: CatalogListRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let manifest = catalog_manifest_v1();
    let family = request.family.map(|value| match value {
        ProtocolCatalogFamilyV1::System => CatalogFamilyV1::System,
        ProtocolCatalogFamilyV1::Biomolecule => CatalogFamilyV1::Biomolecule,
    });
    let entries = search_catalog_v1(
        family,
        request.category.as_deref(),
        request.query.as_deref(),
    )
    .into_iter()
    .map(|entry| CatalogEntrySummaryV1 {
        id: entry.key().as_str().to_owned(),
        family: match entry.family() {
            CatalogFamilyV1::System => ProtocolCatalogFamilyV1::System,
            CatalogFamilyV1::Biomolecule => ProtocolCatalogFamilyV1::Biomolecule,
        },
        category: CatalogCategorySummaryV1 {
            id: entry.category().key().to_owned(),
            name: entry.category().label().to_owned(),
            order: entry.category().order(),
        },
        name: entry.label().to_owned(),
        provenance: CatalogProvenanceSummaryV1 {
            source_kind: entry.provenance().source_kind().to_owned(),
            source_id: entry.provenance().source_id().to_owned(),
            license_spdx: entry.provenance().license_spdx().to_owned(),
        },
    })
    .collect();
    Ok(OperationProtocolOutcomeV1::CatalogList {
        catalog_schema: manifest.schema().to_owned(),
        catalog_version: manifest.catalog_version().to_owned(),
        entries,
    })
}

fn execute_catalog_insert(
    request: CatalogInsertRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    if request.expected_revision != 0 {
        return Err(ExecutionFailureV1::catalog_refusal(
            CatalogPlacementErrorV2::StaleSnapshot,
        ));
    }
    let mut session = admit_document(&request.document)?;
    let digest = parse_digest_hex(&request.expected_digest_hex)?;
    let anchor = PresentationGesturePoint2V1::new(request.anchor_x, request.anchor_y)
        .map_err(|_| ExecutionFailureV1::catalog_refusal(CatalogPlacementErrorV2::InvalidPoint))?;
    let fence = DocumentFenceV1::new(request.expected_revision, digest);
    let gesture = begin_api_catalog_placement_v2(&session, fence, &request.catalog_id)
        .map_err(ExecutionFailureV1::catalog_refusal)?;
    let mut preview = preview_api_catalog_placement_v2(&mut session, &gesture, anchor)
        .map_err(ExecutionFailureV1::catalog_refusal)?;
    let mut prepared = prepare_api_catalog_placement_v2(&mut session, &gesture, &mut preview)
        .map_err(ExecutionFailureV1::catalog_refusal)?;
    let committed = commit_api_catalog_placement_v2(&mut session, &mut prepared)
        .map_err(ExecutionFailureV1::catalog_refusal)?;
    let snapshot = committed.result().observation().snapshot();
    Ok(OperationProtocolOutcomeV1::CatalogInsert {
        document: snapshot.cdml().to_owned(),
        identifier: committed.identifier().to_owned(),
        input_revision: 0,
        committed_revision: snapshot.revision(),
        next_input_expected_revision: 0,
        digest_hex: hex_digest(snapshot.digest()),
    })
}

fn execute_presentation_vector_create(
    request: PresentationVectorCreateRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    match request.appearance_policy {
        ProtocolPresentationVectorAppearancePolicyV1::EffectiveDrawingStandard => {}
    }
    if request.expected_revision != 0 {
        return Err(ExecutionFailureV1::vector_refusal(
            PresentationVectorGestureErrorV1::StaleSnapshot,
        ));
    }
    let mut session = admit_document(&request.document)?;
    let digest = parse_digest_hex(&request.expected_digest_hex)?;
    let kind = match request.vector_kind {
        ProtocolPresentationVectorKindV1::Line => PresentationVectorKindV1::Line,
        ProtocolPresentationVectorKindV1::Rectangle => PresentationVectorKindV1::Rectangle,
        ProtocolPresentationVectorKindV1::Square => PresentationVectorKindV1::Square,
        ProtocolPresentationVectorKindV1::Oval => PresentationVectorKindV1::Oval,
        ProtocolPresentationVectorKindV1::Circle => PresentationVectorKindV1::Circle,
    };
    let start =
        PresentationGesturePoint2V1::new(request.start_x, request.start_y).map_err(|_| {
            ExecutionFailureV1::document_invalid("vector start point must be finite".to_owned())
        })?;
    let end = PresentationGesturePoint2V1::new(request.end_x, request.end_y).map_err(|_| {
        ExecutionFailureV1::document_invalid("vector end point must be finite".to_owned())
    })?;
    let fence = DocumentFenceV1::new(request.expected_revision, digest);
    let gesture = begin_api_presentation_vector_gesture_v1(&session, fence, kind, start)
        .map_err(ExecutionFailureV1::vector_refusal)?;
    let preview = preview_api_presentation_vector_gesture_v1(&session, &gesture, end)
        .map_err(ExecutionFailureV1::vector_refusal)?;
    let mut prepared = prepare_api_presentation_vector_gesture_v1(&mut session, &gesture, &preview)
        .map_err(ExecutionFailureV1::vector_refusal)?;
    let committed = commit_api_presentation_vector_gesture_v1(&mut session, &mut prepared)
        .map_err(ExecutionFailureV1::vector_refusal)?;
    let renderer_observation =
        document_observation_from_accepted_operation_v1(committed.result().observation())
            .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    let plan = compose_document_render_plan_v1(&renderer_observation)
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    if plan
        .outcomes()
        .iter()
        .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(ExecutionFailureV1::internal(
            "renderer-preflighted vector commit produced an excluded root".to_owned(),
        ));
    }
    let immutable_renderer_observation = serde_json::to_value(renderer_observation.wire())
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    let snapshot = committed.result().observation().snapshot();
    Ok(OperationProtocolOutcomeV1::PresentationVectorCreate {
        document: snapshot.cdml().to_owned(),
        identifier: committed.root().presentation_id().as_str().to_owned(),
        input_revision: 0,
        committed_revision: snapshot.revision(),
        next_input_expected_revision: 0,
        digest_hex: hex_digest(snapshot.digest()),
        renderer_observation: immutable_renderer_observation,
    })
}

fn parse_digest_hex(value: &str) -> Result<[u8; 32], ExecutionFailureV1> {
    if value.len() != 64 {
        return Err(ExecutionFailureV1::document_invalid(
            "expected_digest_hex must contain 64 lowercase or uppercase hexadecimal characters"
                .to_owned(),
        ));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let text = std::str::from_utf8(pair).expect("hex input is ASCII-sized");
        digest[index] = u8::from_str_radix(text, 16).map_err(|_| {
            ExecutionFailureV1::document_invalid(
                "expected_digest_hex must contain hexadecimal characters".to_owned(),
            )
        })?;
    }
    Ok(digest)
}

fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
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
    // A runtime error occurs before coordinate generation can establish any
    // user-actionable chemistry result. In particular, a native loader error
    // can contain the trusted adapter path, so it must never become protocol
    // data. Semantic coordinate failures are mapped inside `with_engine`.
    let _ = error;
    ExecutionFailureV1::chemistry_runtime_unavailable()
}

fn map_runtime_conversion_error(error: ChemistryRuntimeErrorV1) -> ExecutionFailureV1 {
    // Keep the trusted runtime capability and its native diagnostics entirely
    // inside this execution boundary. Semantic codec refusals are mapped by
    // `map_conversion_error` before the runtime result is returned.
    let _ = error;
    ExecutionFailureV1::chemistry_runtime_unavailable()
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

#[derive(Debug)]
pub(crate) struct ExecutionFailureV1 {
    category: OperationProtocolErrorCategoryV1,
    message: String,
    presentation_vector_refusal: Option<PresentationVectorRefusalV1>,
    catalog_placement_refusal: Option<CatalogPlacementRefusalV1>,
    reaction_refusal: Option<ReactionRefusalV1>,
}

impl ExecutionFailureV1 {
    fn cml_import_refusal(refusal: crate::CmlImportRefusalV1) -> Self {
        let category = match refusal.category() {
            crate::CmlImportRefusalCategoryV1::ConversionFailed => {
                OperationProtocolErrorCategoryV1::ConversionFailed
            }
            crate::CmlImportRefusalCategoryV1::ConversionUnsupported => {
                OperationProtocolErrorCategoryV1::ConversionUnsupported
            }
            crate::CmlImportRefusalCategoryV1::ResourceLimit => {
                OperationProtocolErrorCategoryV1::ResourceLimit
            }
            crate::CmlImportRefusalCategoryV1::DocumentAdmissionFailed
            | crate::CmlImportRefusalCategoryV1::StaleDocument => {
                OperationProtocolErrorCategoryV1::DocumentAdmissionFailed
            }
            crate::CmlImportRefusalCategoryV1::ChemistryUnavailable => {
                OperationProtocolErrorCategoryV1::ChemistryUnavailable
            }
        };
        Self {
            category,
            message: format!("cml_import_refused:{:?}", refusal.reason()),
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }
    fn document_admission(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::DocumentAdmissionFailed,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn document_invalid(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::DocumentInvalid,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    fn render_unsupported(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::RenderUnsupported,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    fn render_failed(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::RenderFailed,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn chemistry_unavailable(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ChemistryUnavailable,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    fn chemistry_runtime_unavailable() -> Self {
        Self::chemistry_unavailable("Ferrum chemistry runtime is unavailable".to_owned())
    }

    fn conversion_failed(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ConversionFailed,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    fn conversion_unsupported(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ConversionUnsupported,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    fn coordinate(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::CoordinateGenerationFailed,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn resource_limit(message: impl Into<String>) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ResourceLimit,
            message: message.into(),
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn internal(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::InternalFailure,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    fn vector_refusal(error: PresentationVectorGestureErrorV1) -> Self {
        Self {
            category: match error.category() {
                PresentationVectorGestureCategoryV1::RenderPreparation => {
                    OperationProtocolErrorCategoryV1::RenderFailed
                }
                PresentationVectorGestureCategoryV1::ResourceExhausted => {
                    OperationProtocolErrorCategoryV1::ResourceLimit
                }
                _ => OperationProtocolErrorCategoryV1::DocumentInvalid,
            },
            message: error.to_string(),
            presentation_vector_refusal: Some(PresentationVectorRefusalV1 {
                category: vector_category(error.category()),
                recovery: vector_recovery(error.recovery()),
            }),
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    fn catalog_refusal(error: CatalogPlacementErrorV2) -> Self {
        Self {
            category: match error.category() {
                CatalogPlacementCategoryV2::RenderPreparation => {
                    OperationProtocolErrorCategoryV1::RenderFailed
                }
                _ => OperationProtocolErrorCategoryV1::DocumentInvalid,
            },
            message: error.to_string(),
            presentation_vector_refusal: None,
            catalog_placement_refusal: Some(CatalogPlacementRefusalV1 {
                category: catalog_category(error.category()),
                recovery: catalog_recovery(error.recovery()),
            }),
            reaction_refusal: None,
        }
    }

    fn reaction_refusal(error: ReactionGestureErrorV1) -> Self {
        Self {
            category: match error.category() {
                ReactionGestureCategoryV1::UnrenderableDocument
                | ReactionGestureCategoryV1::RenderPreparation => {
                    OperationProtocolErrorCategoryV1::RenderFailed
                }
                _ => OperationProtocolErrorCategoryV1::DocumentInvalid,
            },
            message: error.to_string(),
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: Some(ReactionRefusalV1 {
                category: reaction_category(error.category()),
                recovery: reaction_recovery(error.recovery()),
            }),
        }
    }
}

fn reaction_category(value: ReactionGestureCategoryV1) -> ProtocolReactionRefusalCategoryV1 {
    match value {
        ReactionGestureCategoryV1::StaleSnapshot => {
            ProtocolReactionRefusalCategoryV1::StaleSnapshot
        }
        ReactionGestureCategoryV1::ForeignSession => {
            ProtocolReactionRefusalCategoryV1::ForeignSession
        }
        ReactionGestureCategoryV1::ReplayedGesture => {
            ProtocolReactionRefusalCategoryV1::ReplayedGesture
        }
        ReactionGestureCategoryV1::InvalidRequest => {
            ProtocolReactionRefusalCategoryV1::InvalidRequest
        }
        ReactionGestureCategoryV1::MissingTarget => {
            ProtocolReactionRefusalCategoryV1::MissingTarget
        }
        ReactionGestureCategoryV1::WrongTargetKind => {
            ProtocolReactionRefusalCategoryV1::WrongTargetKind
        }
        ReactionGestureCategoryV1::DuplicateTarget => {
            ProtocolReactionRefusalCategoryV1::DuplicateTarget
        }
        ReactionGestureCategoryV1::CrossReactionReuse => {
            ProtocolReactionRefusalCategoryV1::CrossReactionReuse
        }
        ReactionGestureCategoryV1::UnrenderableDocument => {
            ProtocolReactionRefusalCategoryV1::UnrenderableDocument
        }
        ReactionGestureCategoryV1::RenderPreparation => {
            ProtocolReactionRefusalCategoryV1::RenderPreparation
        }
        ReactionGestureCategoryV1::SessionConflict => {
            ProtocolReactionRefusalCategoryV1::SessionConflict
        }
        ReactionGestureCategoryV1::MissingReaction => {
            ProtocolReactionRefusalCategoryV1::MissingReaction
        }
        ReactionGestureCategoryV1::LegacyDefinitionNotEditable => {
            ProtocolReactionRefusalCategoryV1::LegacyDefinitionNotEditable
        }
        ReactionGestureCategoryV1::MembershipChanged => {
            ProtocolReactionRefusalCategoryV1::MembershipChanged
        }
        ReactionGestureCategoryV1::RendererExclusion => {
            ProtocolReactionRefusalCategoryV1::RendererExclusion
        }
        _ => unreachable!("a new reaction category requires protocol mapping"),
    }
}

fn reaction_recovery(value: ReactionGestureRecoveryV1) -> ProtocolReactionRefusalRecoveryV1 {
    match value {
        ReactionGestureRecoveryV1::RefreshAndRestart => {
            ProtocolReactionRefusalRecoveryV1::RefreshAndRestart
        }
        ReactionGestureRecoveryV1::CorrectSelectors => {
            ProtocolReactionRefusalRecoveryV1::CorrectSelectors
        }
        ReactionGestureRecoveryV1::ChooseRenderableMembers => {
            ProtocolReactionRefusalRecoveryV1::ChooseRenderableMembers
        }
        ReactionGestureRecoveryV1::RepairLegacyDefinition => {
            ProtocolReactionRefusalRecoveryV1::RepairLegacyDefinition
        }
        _ => unreachable!("a new reaction recovery requires protocol mapping"),
    }
}

fn catalog_category(value: CatalogPlacementCategoryV2) -> ProtocolCatalogPlacementCategoryV1 {
    match value {
        CatalogPlacementCategoryV2::UnknownKey => ProtocolCatalogPlacementCategoryV1::UnknownKey,
        CatalogPlacementCategoryV2::StaleSnapshot => {
            ProtocolCatalogPlacementCategoryV1::StaleSnapshot
        }
        CatalogPlacementCategoryV2::ForeignSession => {
            ProtocolCatalogPlacementCategoryV1::ForeignSession
        }
        CatalogPlacementCategoryV2::MismatchedPreview => {
            ProtocolCatalogPlacementCategoryV1::MismatchedPreview
        }
        CatalogPlacementCategoryV2::ReplayedGesture => {
            ProtocolCatalogPlacementCategoryV1::ReplayedGesture
        }
        CatalogPlacementCategoryV2::InvalidPoint => {
            ProtocolCatalogPlacementCategoryV1::InvalidPoint
        }
        CatalogPlacementCategoryV2::RenderPreparation => {
            ProtocolCatalogPlacementCategoryV1::RenderPreparation
        }
        CatalogPlacementCategoryV2::SessionConflict => {
            ProtocolCatalogPlacementCategoryV1::SessionConflict
        }
    }
}

fn catalog_recovery(value: CatalogPlacementRecoveryV2) -> ProtocolCatalogPlacementRecoveryV1 {
    match value {
        CatalogPlacementRecoveryV2::ChooseCatalogEntry => {
            ProtocolCatalogPlacementRecoveryV1::ChooseCatalogEntry
        }
        CatalogPlacementRecoveryV2::RefreshAndRestart => {
            ProtocolCatalogPlacementRecoveryV1::RefreshAndRestart
        }
        CatalogPlacementRecoveryV2::DocumentUnchanged => {
            ProtocolCatalogPlacementRecoveryV1::DocumentUnchanged
        }
    }
}

fn vector_category(
    value: PresentationVectorGestureCategoryV1,
) -> ProtocolPresentationVectorGestureCategoryV1 {
    match value {
        PresentationVectorGestureCategoryV1::StaleSnapshot => {
            ProtocolPresentationVectorGestureCategoryV1::StaleSnapshot
        }
        PresentationVectorGestureCategoryV1::ForeignSession => {
            ProtocolPresentationVectorGestureCategoryV1::ForeignSession
        }
        PresentationVectorGestureCategoryV1::MismatchedPreview => {
            ProtocolPresentationVectorGestureCategoryV1::MismatchedPreview
        }
        PresentationVectorGestureCategoryV1::ReplayedGesture => {
            ProtocolPresentationVectorGestureCategoryV1::ReplayedGesture
        }
        PresentationVectorGestureCategoryV1::InvalidPoint => {
            ProtocolPresentationVectorGestureCategoryV1::InvalidPoint
        }
        PresentationVectorGestureCategoryV1::DegenerateGeometry => {
            ProtocolPresentationVectorGestureCategoryV1::DegenerateGeometry
        }
        PresentationVectorGestureCategoryV1::UnsupportedKind => {
            ProtocolPresentationVectorGestureCategoryV1::UnsupportedKind
        }
        PresentationVectorGestureCategoryV1::UnrenderableStandard => {
            ProtocolPresentationVectorGestureCategoryV1::UnrenderableStandard
        }
        PresentationVectorGestureCategoryV1::RenderPreparation => {
            ProtocolPresentationVectorGestureCategoryV1::RenderPreparation
        }
        PresentationVectorGestureCategoryV1::SessionConflict => {
            ProtocolPresentationVectorGestureCategoryV1::SessionConflict
        }
        PresentationVectorGestureCategoryV1::ResourceExhausted => {
            ProtocolPresentationVectorGestureCategoryV1::ResourceExhausted
        }
        _ => unreachable!("new vector category requires protocol mapping"),
    }
}

fn vector_recovery(
    value: PresentationVectorGestureRecoveryV1,
) -> ProtocolPresentationVectorGestureRecoveryV1 {
    match value {
        PresentationVectorGestureRecoveryV1::DocumentUnchanged => {
            ProtocolPresentationVectorGestureRecoveryV1::DocumentUnchanged
        }
        PresentationVectorGestureRecoveryV1::RefreshAndRestart => {
            ProtocolPresentationVectorGestureRecoveryV1::RefreshAndRestart
        }
        PresentationVectorGestureRecoveryV1::ChangeGeometry => {
            ProtocolPresentationVectorGestureRecoveryV1::ChangeGeometry
        }
        PresentationVectorGestureRecoveryV1::ChooseSupportedAppearance => {
            ProtocolPresentationVectorGestureRecoveryV1::ChooseSupportedAppearance
        }
        PresentationVectorGestureRecoveryV1::ReduceRequest => {
            ProtocolPresentationVectorGestureRecoveryV1::ReduceRequest
        }
        _ => unreachable!("new vector recovery requires protocol mapping"),
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

    struct HostileRuntime;

    impl ChemistryRuntimeV1 for HostileRuntime {
        fn with_engine<T>(
            &self,
            _operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
        ) -> Result<T, ChemistryRuntimeErrorV1> {
            Err(ChemistryRuntimeErrorV1::Chemistry(
                ChemistryError::NativeBoundary {
                    reason: HOSTILE_RUNTIME_DETAIL.to_owned(),
                },
            ))
        }
    }

    const CDML: &str = "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom></molecule></cdml>";
    const HOSTILE_RUNTIME_DETAIL: &str = "/private/ferrum/.dylibs/libferrum_chem.dylib: private_native_adapter dlopen native loader text";

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
    fn reaction_create_protocol_is_canonical_and_rejects_stale_digest() {
        const REACTION_SOURCE: &str = "<cdml><molecule id=\"left\"><atom id=\"left-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"product\"><atom id=\"product-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><arrow id=\"arrow\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow></cdml>";
        let session = admit_document(REACTION_SOURCE).expect("fixture admits");
        let digest = hex_digest(session.snapshot().expect("snapshot").digest());
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "reaction",
            "operation": {"kind": "reaction.create.v1", "document": REACTION_SOURCE,
                "expected_revision": 0, "expected_digest_hex": digest,
                "reactants": ["left"], "products": ["product"], "arrow": "arrow", "conditions": [], "pluses": []}
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("reaction should create");
        };
        let OperationProtocolOutcomeV1::ReactionCreate {
            document,
            reaction_id,
            committed_revision,
            ..
        } = response.outcome
        else {
            panic!("reaction outcome expected");
        };
        assert_eq!(reaction_id, "rxn-1");
        assert_eq!(committed_revision, 1);
        assert!(document.contains("<reaction id=\"rxn-1\""));
        let mut stale = request;
        stale["operation"]["expected_digest_hex"] = serde_json::json!("00".repeat(32));
        let response = execute_operation_v1(&stale.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Error(response) = response else {
            panic!("stale digest must refuse");
        };
        assert_eq!(
            response.error.category,
            OperationProtocolErrorCategoryV1::DocumentInvalid
        );
        assert!(response.error.reaction_refusal.is_some());
    }

    #[test]
    fn reaction_observation_protocol_lists_observes_and_selects_strict_membership() {
        const SOURCE: &str = "<cdml><molecule id=\"left\"><atom id=\"left-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"right\"><atom id=\"right-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><arrow id=\"a\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow><reaction id=\"r\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction></cdml>";
        let digest = hex_digest(
            admit_document(SOURCE)
                .expect("load")
                .snapshot()
                .expect("snapshot")
                .digest(),
        );
        let list = serde_json::json!({ "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "reaction-list", "operation": { "kind": "reaction.list.v1", "document": SOURCE, "expected_revision": 0, "expected_digest_hex": digest } });
        let response = execute_operation_v1(&list.to_string()).expect("request");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("list succeeds");
        };
        let OperationProtocolOutcomeV1::ReactionList { reactions, .. } = response.outcome else {
            panic!("list outcome");
        };
        assert_eq!(reactions.len(), 1);
        assert_eq!(reactions[0].reaction_id, "r");
        assert_eq!(reactions[0].members.len(), 3);
        let observe = serde_json::json!({ "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "reaction-observe", "operation": { "kind": "reaction.observe.v1", "document": SOURCE, "expected_revision": 0, "expected_digest_hex": digest, "reaction_id": "r" } });
        let response = execute_operation_v1(&observe.to_string()).expect("request");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("observe succeeds");
        };
        assert!(matches!(
            response.outcome,
            OperationProtocolOutcomeV1::ReactionObserve { .. }
        ));
        let select = serde_json::json!({ "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "reaction-select", "operation": { "kind": "reaction.select.v1", "document": SOURCE, "expected_revision": 0, "expected_digest_hex": digest, "reaction_id": "r" } });
        let response = execute_operation_v1(&select.to_string()).expect("request");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("select succeeds");
        };
        assert!(
            matches!(response.outcome, OperationProtocolOutcomeV1::ReactionSelect { reaction_id, .. } if reaction_id == "r")
        );
    }

    #[test]
    fn reaction_lifecycle_protocol_replaces_members_and_deletes_only_definition() {
        const SOURCE: &str = "<cdml><molecule id=\"left\"><atom id=\"left-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"right\"><atom id=\"right-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><molecule id=\"third\"><atom id=\"third-a\" name=\"N\"><point x=\"140\" y=\"0\"/></atom></molecule><arrow id=\"a\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow><reaction id=\"r\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction></cdml>";
        let digest = hex_digest(
            admit_document(SOURCE)
                .expect("load")
                .snapshot()
                .expect("snapshot")
                .digest(),
        );
        let patch = serde_json::json!({ "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "patch", "operation": { "kind": "reaction.patch-membership.v1", "document": SOURCE, "expected_revision": 0, "expected_digest_hex": digest, "reaction_id": "r", "reactants": ["left"], "products": ["third"], "arrow": "a", "conditions": [], "pluses": [] } });
        let response = execute_operation_v1(&patch.to_string()).expect("request");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("patch succeeds");
        };
        let OperationProtocolOutcomeV1::ReactionPatchMembership {
            document,
            committed_revision,
            ..
        } = response.outcome
        else {
            panic!("patch outcome");
        };
        assert_eq!(committed_revision, 1);
        assert!(document.contains("product idref=\"third\""));
        let patched_digest = hex_digest(
            admit_document(&document)
                .expect("load")
                .snapshot()
                .expect("snapshot")
                .digest(),
        );
        let delete = serde_json::json!({ "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "delete", "operation": { "kind": "reaction.delete-definition.v1", "document": document, "expected_revision": 0, "expected_digest_hex": patched_digest, "reaction_id": "r" } });
        let response = execute_operation_v1(&delete.to_string()).expect("request");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("delete succeeds");
        };
        let OperationProtocolOutcomeV1::ReactionDeleteDefinition { document, .. } =
            response.outcome
        else {
            panic!("delete outcome");
        };
        assert!(!document.contains("<reaction"));
        assert!(document.contains("molecule id=\"left\""));
    }

    #[test]
    fn reaction_lifecycle_protocol_preserves_missing_and_legacy_refusal_categories() {
        const SOURCE: &str = "<cdml><molecule id=\"left\"><atom id=\"left-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"right\"><atom id=\"right-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><arrow id=\"a\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow><reaction id=\"r\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction></cdml>";
        let digest = hex_digest(
            admit_document(SOURCE)
                .expect("load")
                .snapshot()
                .expect("snapshot")
                .digest(),
        );
        let missing = serde_json::json!({ "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "missing", "operation": { "kind": "reaction.delete-definition.v1", "document": SOURCE, "expected_revision": 0, "expected_digest_hex": digest, "reaction_id": "missing" } });
        let OperationProtocolEnvelopeV1::Error(response) =
            execute_operation_v1(&missing.to_string()).expect("request")
        else {
            panic!("missing reaction must refuse");
        };
        assert!(matches!(
            response.error.reaction_refusal,
            Some(ReactionRefusalV1 {
                category: ProtocolReactionRefusalCategoryV1::MissingReaction,
                recovery: ProtocolReactionRefusalRecoveryV1::RefreshAndRestart
            })
        ));

        let legacy = SOURCE.replace(
            "<reaction id=\"r\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction>",
            "<reaction id=\"r\"><reactant idref=\"left\"/></reaction>",
        );
        let legacy_digest = hex_digest(
            admit_document(&legacy)
                .expect("legacy load")
                .snapshot()
                .expect("legacy snapshot")
                .digest(),
        );
        let request = serde_json::json!({ "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "legacy", "operation": { "kind": "reaction.delete-definition.v1", "document": legacy, "expected_revision": 0, "expected_digest_hex": legacy_digest, "reaction_id": "r" } });
        let OperationProtocolEnvelopeV1::Error(response) =
            execute_operation_v1(&request.to_string()).expect("request")
        else {
            panic!("legacy reaction must refuse");
        };
        assert!(matches!(
            response.error.reaction_refusal,
            Some(ReactionRefusalV1 {
                category: ProtocolReactionRefusalCategoryV1::LegacyDefinitionNotEditable,
                recovery: ProtocolReactionRefusalRecoveryV1::RepairLegacyDefinition
            })
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
    fn hostile_runtime_failures_are_redacted_for_all_runtime_backed_operations() {
        let requests = [
            serde_json::json!({
                "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
                "request_id": "hostile-convert",
                "operation": {
                    "kind": "chemistry.convert",
                    "input": {"format": "smiles", "text": "CCO"},
                    "output_format": "inchi_standard",
                },
            }),
            serde_json::json!({
                "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
                "request_id": "hostile-coordinates",
                "operation": {"kind": "document.generate_coordinates", "document": CDML},
            }),
        ];
        for request in requests {
            let response = execute_operation_with_runtime_v1(&request.to_string(), &HostileRuntime)
                .expect("request decodes");
            let serialized = serde_json::to_string(&response).expect("response serializes");
            let value: serde_json::Value =
                serde_json::from_str(&serialized).expect("response JSON");
            assert_eq!(value["request_id"], request["request_id"]);
            assert_eq!(value["error"]["category"], "chemistry_unavailable");
            assert_eq!(
                value["error"]["message"],
                "Ferrum chemistry runtime is unavailable"
            );
            for private_detail in [
                HOSTILE_RUNTIME_DETAIL,
                ".dylibs",
                "libferrum_chem",
                "private_native_adapter",
                "dlopen",
            ] {
                assert!(!serialized.contains(private_detail));
            }
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
