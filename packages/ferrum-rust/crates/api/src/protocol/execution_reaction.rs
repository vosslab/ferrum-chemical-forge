//! Reaction operation execution.

use super::*;

pub(super) fn reaction_session(
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

pub(super) fn execute_reaction_list(
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

pub(super) fn execute_reaction_observe(
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

pub(super) fn reaction_summary(
    value: &crate::ReactionObservationV1,
) -> ReactionObservationSummaryV1 {
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

pub(super) fn execute_reaction_create(
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
    let request = resolve_api_reaction_gesture_v1(&session, gesture)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let mut prepared = session
        .prepare_session_operation_transition_v1(request)
        .map_err(|_| {
            ExecutionFailureV1::reaction_refusal(ReactionGestureErrorV1::SessionConflict)
        })?;
    let result = session
        .commit_session_operation_transition_v1(&mut prepared)
        .map_err(|_| {
            ExecutionFailureV1::reaction_refusal(ReactionGestureErrorV1::SessionConflict)
        })?;
    let SessionOperationOutcomeV1::ReactionCreatedV1(outcome) = result.outcome() else {
        return Err(ExecutionFailureV1::reaction_refusal(
            ReactionGestureErrorV1::SessionConflict,
        ));
    };
    let snapshot = result.observation().snapshot();
    Ok(OperationProtocolOutcomeV1::ReactionCreate {
        document: snapshot.cdml().to_owned(),
        reaction_id: outcome.reaction_id().to_owned(),
        input_revision: 0,
        committed_revision: snapshot.revision(),
        next_input_expected_revision: 0,
        digest_hex: hex_digest(snapshot.digest()),
    })
}

pub(super) fn select_lifecycle_reaction(
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

pub(super) fn execute_reaction_patch(
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
    let request = resolve_api_reaction_lifecycle_v1(&session, gesture)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let mut prepared = session
        .prepare_session_operation_transition_v1(request)
        .map_err(|_| {
            ExecutionFailureV1::reaction_refusal(ReactionGestureErrorV1::SessionConflict)
        })?;
    let result = session
        .commit_session_operation_transition_v1(&mut prepared)
        .map_err(|_| {
            ExecutionFailureV1::reaction_refusal(ReactionGestureErrorV1::SessionConflict)
        })?;
    let SessionOperationOutcomeV1::ReactionMembershipReplacedV1(outcome) = result.outcome() else {
        return Err(ExecutionFailureV1::reaction_refusal(
            ReactionGestureErrorV1::SessionConflict,
        ));
    };
    let snapshot = result.observation().snapshot();
    Ok(OperationProtocolOutcomeV1::ReactionPatchMembership {
        document: snapshot.cdml().to_owned(),
        reaction_id: outcome.reaction_id().to_owned(),
        input_revision: 0,
        committed_revision: snapshot.revision(),
        next_input_expected_revision: 0,
        digest_hex: hex_digest(snapshot.digest()),
    })
}

pub(super) fn execute_reaction_delete(
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
    let request = resolve_api_reaction_lifecycle_v1(&session, gesture)
        .map_err(ExecutionFailureV1::reaction_refusal)?;
    let mut prepared = session
        .prepare_session_operation_transition_v1(request)
        .map_err(|_| {
            ExecutionFailureV1::reaction_refusal(ReactionGestureErrorV1::SessionConflict)
        })?;
    let result = session
        .commit_session_operation_transition_v1(&mut prepared)
        .map_err(|_| {
            ExecutionFailureV1::reaction_refusal(ReactionGestureErrorV1::SessionConflict)
        })?;
    let SessionOperationOutcomeV1::ReactionDefinitionDeletedV1(outcome) = result.outcome() else {
        return Err(ExecutionFailureV1::reaction_refusal(
            ReactionGestureErrorV1::SessionConflict,
        ));
    };
    let snapshot = result.observation().snapshot();
    Ok(OperationProtocolOutcomeV1::ReactionDeleteDefinition {
        document: snapshot.cdml().to_owned(),
        reaction_id: outcome.reaction_id().to_owned(),
        input_revision: 0,
        committed_revision: snapshot.revision(),
        next_input_expected_revision: 0,
        digest_hex: hex_digest(snapshot.digest()),
    })
}

pub(super) fn execute_reaction_translate(
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
    let transition_request = resolve_api_reaction_translation_v1(
        &session,
        gesture,
        request.pointer_x,
        request.pointer_y,
    )
    .map_err(ExecutionFailureV1::reaction_refusal)?;
    let mut prepared = session
        .prepare_session_operation_transition_v1(transition_request)
        .map_err(|_| {
            ExecutionFailureV1::reaction_refusal(ReactionGestureErrorV1::SessionConflict)
        })?;
    let result = session
        .commit_session_operation_transition_v1(&mut prepared)
        .map_err(|_| {
            ExecutionFailureV1::reaction_refusal(ReactionGestureErrorV1::SessionConflict)
        })?;
    let SessionOperationOutcomeV1::Standard = result.outcome() else {
        return Err(ExecutionFailureV1::reaction_refusal(
            ReactionGestureErrorV1::SessionConflict,
        ));
    };
    let snapshot = result.observation().snapshot();
    Ok(OperationProtocolOutcomeV1::ReactionTranslate {
        document: snapshot.cdml().to_owned(),
        reaction_id: request.reaction_id,
        input_revision: 0,
        committed_revision: snapshot.revision(),
        next_input_expected_revision: 0,
        digest_hex: hex_digest(snapshot.digest()),
    })
}
