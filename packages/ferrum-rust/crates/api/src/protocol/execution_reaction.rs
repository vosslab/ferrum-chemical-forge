//! Durable reaction-command protocol execution.

use super::*;

pub(super) fn reaction_session(
    document: String,
    expected_revision: u64,
    expected_digest_hex: String,
) -> Result<(RenderInteractionSessionV1, DocumentFenceV1, String), ExecutionFailureV1> {
    let session = RenderInteractionSessionV1::new(admit_document(&document)?);
    let snapshot = session
        .snapshot()
        .map_err(|_| ExecutionFailureV1::reaction_invalid_request())?;
    let digest = parse_digest_hex(&expected_digest_hex)?;
    if snapshot.revision() != expected_revision || snapshot.digest() != &digest {
        return Err(ExecutionFailureV1::reaction_transition_refusal(
            ferrum_document::AdmittedSessionTransitionRefusalV1::StaleSnapshot,
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
    let list = std::ops::Deref::deref(&session)
        .observe_reaction_list_v1()
        .map_err(ExecutionFailureV1::reaction_selection_refusal)?;
    Ok(OperationProtocolOutcomeV1::ReactionList {
        input_revision: fence.revision(),
        next_input_expected_revision: fence.revision(),
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
    let object_id = DocumentObjectIdV1::parse(request.reaction_document_object_id)
        .map_err(|_| ExecutionFailureV1::reaction_invalid_request())?;
    let observation = session
        .observe_reaction_members_v1(&object_id)
        .map_err(ExecutionFailureV1::reaction_selection_refusal)?;
    if select {
        session
            .select_reaction_members_v1(&observation)
            .map_err(ExecutionFailureV1::reaction_selection_refusal)?;
        Ok(OperationProtocolOutcomeV1::ReactionSelect {
            input_revision: fence.revision(),
            next_input_expected_revision: fence.revision(),
            digest_hex,
            reaction_document_object_id: object_id.as_str().to_owned(),
        })
    } else {
        let list = std::ops::Deref::deref(&session)
            .observe_reaction_list_v1()
            .map_err(ExecutionFailureV1::reaction_selection_refusal)?;
        let reaction = list
            .reactions()
            .iter()
            .find(|value| value.reaction_object_id() == &object_id)
            .ok_or_else(ExecutionFailureV1::reaction_invalid_request)?;
        Ok(OperationProtocolOutcomeV1::ReactionObserve {
            input_revision: fence.revision(),
            next_input_expected_revision: fence.revision(),
            digest_hex,
            reaction: reaction_summary(reaction),
        })
    }
}

pub(super) fn reaction_summary(
    value: &ferrum_document::DocumentReactionListReactionV1,
) -> ReactionObservationSummaryV1 {
    ReactionObservationSummaryV1 {
        reaction_document_object_id: value.reaction_object_id().as_str().to_owned(),
        disposition: match value.disposition() {
            ferrum_document::DocumentReactionListDispositionV1::Strict => {
                ProtocolReactionDefinitionDispositionV1::Strict
            }
            ferrum_document::DocumentReactionListDispositionV1::DisplayOnly => {
                ProtocolReactionDefinitionDispositionV1::DisplayOnly
            }
        },
        diagnostics: value
            .diagnostics()
            .iter()
            .map(|diagnostic| format!("{diagnostic:?}").to_lowercase())
            .collect(),
        members: value
            .members()
            .iter()
            .map(|member| ReactionMemberSummaryV1 {
                document_object_id: member.object_id().as_str().to_owned(),
                role: member.role().local_name().to_owned(),
                role_ordinal: member.role_ordinal(),
                document_paint_order: member.document_paint_order(),
            })
            .collect(),
    }
}

pub(super) fn execute_reaction_create(
    request: ReactionCreateRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let (mut session, fence, _) = reaction_session(
        request.document,
        request.expected_revision,
        request.expected_digest_hex,
    )?;
    let targets = reaction_targets(
        request.reactant_document_object_ids,
        request.arrow_document_object_id,
        request.product_document_object_ids,
        request.reagent_document_object_ids,
        request.plus_document_object_ids,
    )?;
    let command = session
        .begin_create_reaction_v1(session.issue_authoring_capability_v1(), fence, targets)
        .map_err(ExecutionFailureV1::reaction_authoring_command_refusal)?;
    let transition = session
        .resolve_create_reaction_command_v1(command)
        .map_err(ExecutionFailureV1::reaction_authoring_command_refusal)?;
    let result = prepare_and_commit(&mut session, transition)?;
    let SessionOperationOutcomeV1::ReactionCreatedV1(outcome) = result.outcome() else {
        return Err(ExecutionFailureV1::reaction_invalid_request());
    };
    let reaction_document_object_id = outcome.reaction_document_object_id().as_str().to_owned();
    let snapshot = result.observation().snapshot();
    Ok(OperationProtocolOutcomeV1::ReactionCreate {
        document: snapshot.cdml().to_owned(),
        reaction_document_object_id,
        input_revision: fence.revision(),
        committed_revision: snapshot.revision(),
        next_input_expected_revision: snapshot.revision(),
        digest_hex: hex_digest(snapshot.digest()),
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
    let selection = reaction_selection(&session, request.reaction_document_object_id)?;
    let targets = reaction_targets(
        request.reactant_document_object_ids,
        request.arrow_document_object_id,
        request.product_document_object_ids,
        request.reagent_document_object_ids,
        request.plus_document_object_ids,
    )?;
    let capability = session.issue_authoring_capability_v1();
    let command = session
        .begin_replace_reaction_members_v1(capability, selection, targets)
        .map_err(ExecutionFailureV1::reaction_authoring_command_refusal)?;
    let transition = session
        .resolve_replace_reaction_members_command_v1(command)
        .map_err(ExecutionFailureV1::reaction_authoring_command_refusal)?;
    let result = prepare_and_commit(&mut session, transition)?;
    let SessionOperationOutcomeV1::ReactionMembershipReplacedV1(outcome) = result.outcome() else {
        return Err(ExecutionFailureV1::reaction_invalid_request());
    };
    let reaction_document_object_id = outcome.reaction_document_object_id().as_str().to_owned();
    let snapshot = result.observation().snapshot();
    Ok(OperationProtocolOutcomeV1::ReactionPatchMembership {
        document: snapshot.cdml().to_owned(),
        reaction_document_object_id,
        input_revision: fence.revision(),
        committed_revision: snapshot.revision(),
        next_input_expected_revision: snapshot.revision(),
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
    let selection = reaction_selection(&session, request.reaction_document_object_id)?;
    let capability = session.issue_authoring_capability_v1();
    let command = session
        .begin_delete_reaction_v1(capability, selection)
        .map_err(ExecutionFailureV1::reaction_authoring_command_refusal)?;
    let transition = session
        .resolve_delete_reaction_command_v1(command)
        .map_err(ExecutionFailureV1::reaction_authoring_command_refusal)?;
    let result = prepare_and_commit(&mut session, transition)?;
    let SessionOperationOutcomeV1::ReactionDefinitionDeletedV1(outcome) = result.outcome() else {
        return Err(ExecutionFailureV1::reaction_invalid_request());
    };
    let reaction_document_object_id = outcome.reaction_document_object_id().as_str().to_owned();
    let snapshot = result.observation().snapshot();
    Ok(OperationProtocolOutcomeV1::ReactionDeleteDefinition {
        document: snapshot.cdml().to_owned(),
        reaction_document_object_id,
        input_revision: fence.revision(),
        committed_revision: snapshot.revision(),
        next_input_expected_revision: snapshot.revision(),
        digest_hex: hex_digest(snapshot.digest()),
    })
}

fn reaction_targets(
    reactants: Vec<String>,
    arrow: String,
    products: Vec<String>,
    reagents: Vec<String>,
    pluses: Vec<String>,
) -> Result<ferrum_document::DocumentReactionMemberTargetsV1, ExecutionFailureV1> {
    let parse = |value| {
        DocumentObjectIdV1::parse(value).map_err(|_| ExecutionFailureV1::reaction_invalid_request())
    };
    let parse_many = |values: Vec<String>| {
        values
            .into_iter()
            .map(&parse)
            .collect::<Result<Vec<_>, _>>()
    };
    ferrum_document::DocumentReactionMemberTargetsV1::new(
        parse_many(reactants)?,
        parse(arrow)?,
        parse_many(products)?,
        parse_many(reagents)?,
        parse_many(pluses)?,
    )
    .map_err(ExecutionFailureV1::reaction_operation_refusal)
}

fn reaction_selection(
    session: &RenderInteractionSessionV1,
    selector: String,
) -> Result<ferrum_document::DocumentReactionMemberSelectionV1, ExecutionFailureV1> {
    let object_id = DocumentObjectIdV1::parse(selector)
        .map_err(|_| ExecutionFailureV1::reaction_invalid_request())?;
    let observation = std::ops::Deref::deref(session)
        .observe_reaction_members_v1(&object_id)
        .map_err(ExecutionFailureV1::reaction_selection_refusal)?;
    session
        .select_reaction_members_v1(&observation)
        .map_err(ExecutionFailureV1::reaction_selection_refusal)
}

fn prepare_and_commit(
    session: &mut RenderInteractionSessionV1,
    request: ferrum_document::SessionOperationTransitionRequestV1,
) -> Result<ferrum_document::SessionOperationResultV1, ExecutionFailureV1> {
    let mut prepared = session
        .prepare_session_operation_transition_v1(request)
        .map_err(|_| ExecutionFailureV1::reaction_invalid_request())?;
    session
        .commit_session_operation_transition_v1(&mut prepared)
        .map_err(|_| ExecutionFailureV1::reaction_invalid_request())
}
