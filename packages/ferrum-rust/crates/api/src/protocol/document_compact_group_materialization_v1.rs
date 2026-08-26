//! Public adapter for one fenced generic compact-group materialization.

use ferrum_document::{
    AdmittedSessionTransitionRefusalV1, DocumentCompactGroupMaterializationRefusalV1,
    DocumentCompactGroupMaterializationRequestV1 as DocumentRequest, DocumentObjectIdV1,
    DocumentSessionError, SessionOperation, SessionOperationError, SessionOperationOutcomeV1,
    SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
};

use super::execution::{ExecutionFailureV1, admit_document, hex_digest};
use super::*;

const MATERIALIZATION_SCHEMA_V1: &str = "ferrum-document-compact-group-materialization-v1";

pub(super) fn execute_document_compact_group_materialize(
    request: DocumentCompactGroupMaterializationRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let mut session = admit_document(&request.document.cdml)?;
    execute_document_compact_group_materialize_on_session(&mut session, request)
        .map(|(outcome, _)| outcome)
}

pub(crate) fn execute_document_compact_group_materialize_on_session(
    session: &mut ferrum_document::DocumentSession,
    request: DocumentCompactGroupMaterializationRequestV1,
) -> Result<
    (
        OperationProtocolOutcomeV1,
        Option<ferrum_document::SessionOperationResultV1>,
    ),
    ExecutionFailureV1,
> {
    let source_molecule_id = request.molecule_id.clone();
    let source_compact_group_id = request.compact_group_id.clone();
    let molecule_id = parse_object_id(&request.molecule_id, "molecule_id")?;
    let compact_group_id = parse_object_id(&request.compact_group_id, "compact_group_id")?;
    let expected_digest = parse_digest(&request.document.expected_digest_hex)?;
    let source_revision = request.document.expected_revision;
    let source_digest_hex = request.document.expected_digest_hex.clone();
    let result = execute_document_compact_group_materialize_transition_on_session(
        session,
        DocumentRequest::new(
            source_revision,
            expected_digest,
            molecule_id,
            compact_group_id,
        ),
    )?;
    let SessionOperationOutcomeV1::CompactGroupMaterializedV1(materialization) = result.outcome()
    else {
        return Err(ExecutionFailureV1::internal(
            "generic compact-group transition returned an unexpected outcome".to_owned(),
        ));
    };
    let snapshot = session
        .snapshot()
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    Ok((
        compact_group_materialization_outcome(
            source_revision,
            source_digest_hex,
            source_molecule_id,
            source_compact_group_id,
            materialization.focus_atom_id().as_str().to_owned(),
            snapshot.cdml().to_owned(),
            0,
            hex_digest(snapshot.digest()),
        ),
        Some(result),
    ))
}

pub(crate) fn execute_document_compact_group_materialize_transition_on_session(
    session: &mut ferrum_document::DocumentSession,
    request: DocumentRequest,
) -> Result<ferrum_document::SessionOperationResultV1, ExecutionFailureV1> {
    let transition = SessionOperationTransitionRequestV1::new(
        request.expected_revision(),
        SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(request)),
        TransitionAuthorizationV1::None,
    );
    match session.prepare_session_operation_transition_v1(transition) {
        Ok(mut prepared) => session
            .commit_session_operation_transition_v1(&mut prepared)
            .map_err(map_commit_refusal),
        Err(error) => Err(map_prepare_error(error)),
    }
}

pub(crate) fn compact_group_materialization_outcome(
    source_revision: u64,
    source_digest_hex: String,
    molecule_id: String,
    compact_group_id: String,
    replacement_focus_atom_id: String,
    document: String,
    document_revision: u64,
    document_digest_hex: String,
) -> OperationProtocolOutcomeV1 {
    OperationProtocolOutcomeV1::DocumentCompactGroupMaterialize {
        materialization: DocumentCompactGroupMaterializationResultV1 {
            schema: MATERIALIZATION_SCHEMA_V1.to_owned(),
            source_revision,
            source_digest_hex,
            molecule_id,
            compact_group_id,
            replacement_focus_atom_id,
            document,
            document_fence: DocumentRequestFenceV1 {
                expected_revision: document_revision,
                expected_digest_hex: document_digest_hex,
            },
        },
    }
}

pub(crate) fn map_prepare_error(error: DocumentSessionError) -> ExecutionFailureV1 {
    match error {
        DocumentSessionError::RevisionConflict { .. } => compact_refusal(
            ProtocolCompactGroupMaterializationCategoryV1::StaleDocumentFence,
            ProtocolCompactGroupMaterializationRecoveryV1::RefreshAndRetry,
        ),
        DocumentSessionError::Operation(SessionOperationError::CompactGroupMaterialization(
            refusal,
        )) => map_document_refusal(refusal),
        DocumentSessionError::RendererAdmission => compact_refusal(
            ProtocolCompactGroupMaterializationCategoryV1::RendererPreparationRefusal,
            ProtocolCompactGroupMaterializationRecoveryV1::DocumentUnchanged,
        ),
        _ => compact_refusal(
            ProtocolCompactGroupMaterializationCategoryV1::SessionConflictOrConsumedPreparation,
            ProtocolCompactGroupMaterializationRecoveryV1::RefreshAndRetry,
        ),
    }
}

fn map_commit_refusal(refusal: AdmittedSessionTransitionRefusalV1) -> ExecutionFailureV1 {
    let (category, recovery) = match refusal {
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => (
            ProtocolCompactGroupMaterializationCategoryV1::StaleDocumentFence,
            ProtocolCompactGroupMaterializationRecoveryV1::RefreshAndRetry,
        ),
        AdmittedSessionTransitionRefusalV1::RendererAdmission => (
            ProtocolCompactGroupMaterializationCategoryV1::RendererPreparationRefusal,
            ProtocolCompactGroupMaterializationRecoveryV1::DocumentUnchanged,
        ),
        AdmittedSessionTransitionRefusalV1::ForeignSession
        | AdmittedSessionTransitionRefusalV1::Consumed
        | AdmittedSessionTransitionRefusalV1::ProvisionalCapability => (
            ProtocolCompactGroupMaterializationCategoryV1::SessionConflictOrConsumedPreparation,
            ProtocolCompactGroupMaterializationRecoveryV1::RefreshAndRetry,
        ),
    };
    compact_refusal(category, recovery)
}

fn map_document_refusal(
    refusal: DocumentCompactGroupMaterializationRefusalV1,
) -> ExecutionFailureV1 {
    let (category, recovery) = match refusal {
        DocumentCompactGroupMaterializationRefusalV1::StaleObservation
        | DocumentCompactGroupMaterializationRefusalV1::DigestMismatch => (
            ProtocolCompactGroupMaterializationCategoryV1::StaleDocumentFence,
            ProtocolCompactGroupMaterializationRecoveryV1::RefreshAndRetry,
        ),
        DocumentCompactGroupMaterializationRefusalV1::InvalidTarget => (
            ProtocolCompactGroupMaterializationCategoryV1::UnknownOrForeignTarget,
            ProtocolCompactGroupMaterializationRecoveryV1::CorrectTarget,
        ),
        DocumentCompactGroupMaterializationRefusalV1::UnsupportedRecipe
        | DocumentCompactGroupMaterializationRefusalV1::InvalidTopology
        | DocumentCompactGroupMaterializationRefusalV1::InvalidCandidate => (
            ProtocolCompactGroupMaterializationCategoryV1::IneligibleTarget,
            ProtocolCompactGroupMaterializationRecoveryV1::ChooseEligibleTarget,
        ),
        DocumentCompactGroupMaterializationRefusalV1::RendererAdmission => (
            ProtocolCompactGroupMaterializationCategoryV1::RendererPreparationRefusal,
            ProtocolCompactGroupMaterializationRecoveryV1::DocumentUnchanged,
        ),
    };
    compact_refusal(category, recovery)
}

pub(crate) fn compact_refusal(
    category: ProtocolCompactGroupMaterializationCategoryV1,
    recovery: ProtocolCompactGroupMaterializationRecoveryV1,
) -> ExecutionFailureV1 {
    ExecutionFailureV1::compact_group_materialization_refusal(
        CompactGroupMaterializationRefusalV1 { category, recovery },
    )
}

fn parse_object_id(value: &str, field: &str) -> Result<DocumentObjectIdV1, ExecutionFailureV1> {
    DocumentObjectIdV1::parse(value).map_err(|_| {
        ExecutionFailureV1::invalid_request(format!(
            "{field} is not a durable document object identifier"
        ))
    })
}

pub(crate) fn parse_digest(value: &str) -> Result<[u8; 32], ExecutionFailureV1> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(ExecutionFailureV1::invalid_request(
            "expected_digest_hex must be a lowercase SHA-256 digest",
        ));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = hexadecimal_nibble(pair[0])
            .ok_or_else(|| ExecutionFailureV1::invalid_request("expected_digest_hex is invalid"))?;
        let low = hexadecimal_nibble(pair[1])
            .ok_or_else(|| ExecutionFailureV1::invalid_request("expected_digest_hex is invalid"))?;
        digest[index] = (high << 4) | low;
    }
    Ok(digest)
}

const fn hexadecimal_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use ferrum_document::DocumentObjectIdV1;

    use super::parse_object_id;

    #[test]
    fn compact_materialization_selector_requires_a_durable_document_object_id() {
        let expected = DocumentObjectIdV1::from_entropy_bytes([0x4a; 16]);

        assert_eq!(
            parse_object_id(expected.as_str(), "molecule_id").expect("durable selector parses"),
            expected
        );
        assert!(parse_object_id("molecule-source-id", "molecule_id").is_err());
    }
}
