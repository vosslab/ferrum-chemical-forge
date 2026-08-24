//! Public adapter for one renderer-safe fenced hydrogen materialization.

use ferrum_document::{
    DocumentMoleculeHydrogenMaterializationRefusalV1 as DocumentRefusal,
    DocumentMoleculeHydrogenMaterializationRequestV1 as DocumentRequest, DocumentObjectIdV1,
    DocumentSessionError, SessionOperation, SessionOperationError, SessionOperationOutcomeV1,
    SessionOperationResultV1, SessionOperationTransitionRequestV1, SessionOperationV1,
    TransitionAuthorizationV1,
};

use super::execution::{ExecutionFailureV1, admit_document, hex_digest};
use super::*;

const MATERIALIZATION_SCHEMA_V1: &str = "ferrum-document-molecule-hydrogen-materialization-v1";

pub(super) fn execute_document_molecule_hydrogen_materialize(
    request: DocumentMoleculeHydrogenMaterializationRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let mut session = admit_document(&request.document.cdml)?;
    execute_document_molecule_hydrogen_materialize_on_session(&mut session, request)
        .map(|(outcome, _)| outcome)
}

pub(crate) fn execute_document_molecule_hydrogen_materialize_on_session(
    session: &mut ferrum_document::DocumentSession,
    request: DocumentMoleculeHydrogenMaterializationRequestV1,
) -> Result<(OperationProtocolOutcomeV1, Option<SessionOperationResultV1>), ExecutionFailureV1> {
    let molecule_id = parse_object_id(&request.molecule_id, "molecule_id")?;
    let anchor_atom_id = parse_object_id(&request.anchor_atom_id, "anchor_atom_id")?;
    let expected_digest = parse_digest(&request.document.expected_digest_hex)?;
    let source_revision = request.document.expected_revision;
    let source_digest_hex = request.document.expected_digest_hex.clone();
    let materialization_request = DocumentRequest::new(
        request.document.expected_revision,
        expected_digest,
        molecule_id,
        anchor_atom_id,
    );
    let transition = SessionOperationTransitionRequestV1::new(
        request.document.expected_revision,
        SessionOperation::V1(SessionOperationV1::MaterializeMoleculeHydrogensV1(
            materialization_request,
        )),
        TransitionAuthorizationV1::None,
    );
    let (outcome, operation_result) = match session
        .prepare_session_operation_transition_v1(transition)
    {
        Ok(mut prepared) => match session.commit_session_operation_transition_v1(&mut prepared) {
            Ok(result) => {
                let SessionOperationOutcomeV1::MoleculeHydrogensMaterializedV1(materialization) =
                    result.outcome()
                else {
                    return Err(ExecutionFailureV1::internal(
                        "generic hydrogen transition returned an unexpected outcome".to_owned(),
                    ));
                };
                (
                    committed_outcome(session, materialization.clone())?,
                    Some(result),
                )
            }
            Err(error) => {
                return Err(ExecutionFailureV1::internal(format!(
                    "generic hydrogen transition could not commit: {error:?}"
                )));
            }
        },
        Err(DocumentSessionError::Operation(SessionOperationError::HydrogenMaterialization(
            refusal,
        ))) => (unavailable_or_refusal(refusal)?, None),
        Err(DocumentSessionError::RendererAdmission) => (
            unavailable_or_refusal(DocumentRefusal::RendererAdmission)?,
            None,
        ),
        Err(error) => {
            return Err(ExecutionFailureV1::internal(format!(
                "generic hydrogen transition could not prepare: {error}"
            )));
        }
    };
    Ok((
        OperationProtocolOutcomeV1::DocumentMoleculeHydrogenMaterialize {
            materialization: DocumentMoleculeHydrogenMaterializationResultV1 {
                schema: MATERIALIZATION_SCHEMA_V1.to_owned(),
                // The authenticated request fence describes the source across
                // both request-owned and live-session execution. The live
                // session's monotonic history revision remains available only
                // through its separate mutation receipt.
                source_revision,
                source_digest_hex,
                molecule_id: request.molecule_id,
                anchor_atom_id: request.anchor_atom_id,
                outcome,
            },
        },
        operation_result,
    ))
}

fn committed_outcome(
    session: &ferrum_document::DocumentSession,
    result: ferrum_document::DocumentMoleculeHydrogenMaterializationResultV1,
) -> Result<DocumentMoleculeHydrogenMaterializationOutcomeV1, ExecutionFailureV1> {
    let snapshot = session
        .snapshot()
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    let added_hydrogen_count = u32::try_from(result.added_hydrogen_count())
        .map_err(|_| ExecutionFailureV1::internal("hydrogen count exceeds V1 range".to_owned()))?;
    let document_fence = DocumentRequestFenceV1 {
        // The returned CDML is a fresh request-owned document. Re-admission
        // starts its stateless session at revision zero rather than inheriting
        // the transient mutation session's commit revision.
        expected_revision: 0,
        expected_digest_hex: hex_digest(snapshot.digest()),
    };
    let document = snapshot.cdml().to_owned();
    if result.changed() {
        Ok(DocumentMoleculeHydrogenMaterializationOutcomeV1::Applied {
            added_hydrogen_count,
            document,
            document_fence,
        })
    } else {
        Ok(DocumentMoleculeHydrogenMaterializationOutcomeV1::NoOp {
            added_hydrogen_count,
            document,
            document_fence,
        })
    }
}

fn unavailable_or_refusal(
    refusal: DocumentRefusal,
) -> Result<DocumentMoleculeHydrogenMaterializationOutcomeV1, ExecutionFailureV1> {
    let unavailable_reason = match refusal {
        DocumentRefusal::StaleObservation
        | DocumentRefusal::DigestMismatch
        | DocumentRefusal::UnknownDirectMolecule
        | DocumentRefusal::UnknownAnchorAtom
        | DocumentRefusal::AnchorNotInSelectedRoot => {
            return Err(ExecutionFailureV1::hydrogen_materialization_refusal(
                refusal,
            ));
        }
        DocumentRefusal::ElementOutsideProfile => {
            DocumentMoleculeHydrogenMaterializationUnavailableReasonV1::ElementOutsideProfile
        }
        DocumentRefusal::NonzeroFormalCharge => {
            DocumentMoleculeHydrogenMaterializationUnavailableReasonV1::NonzeroFormalCharge
        }
        DocumentRefusal::NonzeroExplicitHydrogens => {
            DocumentMoleculeHydrogenMaterializationUnavailableReasonV1::NonzeroExplicitHydrogens
        }
        DocumentRefusal::UnsupportedBondOrRadical => {
            DocumentMoleculeHydrogenMaterializationUnavailableReasonV1::UnsupportedBondOrRadical
        }
        DocumentRefusal::ExistingHydrogenTopology => {
            DocumentMoleculeHydrogenMaterializationUnavailableReasonV1::ExistingHydrogenTopology
        }
        DocumentRefusal::ValenceExceeded => {
            DocumentMoleculeHydrogenMaterializationUnavailableReasonV1::ValenceExceeded
        }
        DocumentRefusal::UnsupportedDocument => {
            DocumentMoleculeHydrogenMaterializationUnavailableReasonV1::UnsupportedDocument
        }
        DocumentRefusal::ResourceLimit => {
            DocumentMoleculeHydrogenMaterializationUnavailableReasonV1::ResourceLimit
        }
        DocumentRefusal::UnrenderableCandidate => {
            DocumentMoleculeHydrogenMaterializationUnavailableReasonV1::UnrenderableCandidate
        }
        DocumentRefusal::RendererAdmission => {
            DocumentMoleculeHydrogenMaterializationUnavailableReasonV1::RenderPreparation
        }
        DocumentRefusal::OxidationPostcondition => {
            DocumentMoleculeHydrogenMaterializationUnavailableReasonV1::OxidationPostcondition
        }
    };
    Ok(DocumentMoleculeHydrogenMaterializationOutcomeV1::Unavailable { unavailable_reason })
}

fn parse_object_id(value: &str, field: &str) -> Result<DocumentObjectIdV1, ExecutionFailureV1> {
    DocumentObjectIdV1::parse(value.to_owned()).map_err(|_| {
        ExecutionFailureV1::invalid_request(format!(
            "{field} is not a durable document object identifier"
        ))
    })
}

fn parse_digest(value: &str) -> Result<[u8; 32], ExecutionFailureV1> {
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
