//! Public adapter for one fenced document atom oxidation observation.

use ferrum_document::{
    DocumentAtomOxidationObservationRequestV1,
    DocumentAtomOxidationObservationV1 as DocumentObservation, DocumentAtomOxidationResultV1,
    DocumentAtomOxidationUnavailableReasonV1 as DocumentUnavailable, DocumentObjectIdV1,
};

use super::execution::{ExecutionFailureV1, admit_document, hex_digest};
use super::*;

const OBSERVATION_SCHEMA_V1: &str = "ferrum-document-atom-oxidation-observation-v1";
const OXIDATION_CONVENTION_V1: &str = "formal-electron-assignment-hcno-v1";

pub(super) fn execute_document_atom_oxidation_observe(
    request: DocumentAtomOxidationObserveRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let molecule_id = DocumentObjectIdV1::parse(request.molecule_id.clone())
        .map_err(|_| invalid_request("molecule_id is not a durable document object identifier"))?;
    let atom_id = DocumentObjectIdV1::parse(request.atom_id.clone())
        .map_err(|_| invalid_request("atom_id is not a durable document object identifier"))?;
    let expected_digest = parse_digest(&request.document.expected_digest_hex)?;
    let session = admit_document(&request.document.cdml)?;
    let snapshot = session
        .snapshot()
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    let root_order = session
        .observe(0)
        .map_err(|error| ExecutionFailureV1::document_invalid(error.to_string()))?
        .projection()
        .molecules()
        .iter()
        .position(|molecule| molecule.id() == Some(&molecule_id))
        .and_then(|position| u32::try_from(position).ok());
    let session_request = DocumentAtomOxidationObservationRequestV1::new(
        request.document.expected_revision,
        expected_digest,
        molecule_id,
        atom_id,
    );
    let result = session
        .observe_atom_oxidation_v1(&session_request)
        .map_err(ExecutionFailureV1::oxidation_refusal)?;
    let document_root_order = root_order.ok_or_else(|| {
        ExecutionFailureV1::oxidation_refusal(
            ferrum_document::DocumentAtomOxidationRefusalV1::UnknownDirectMolecule,
        )
    })?;
    let outcome = match result {
        DocumentAtomOxidationResultV1::Observation(DocumentObservation::Accepted {
            oxidation_number,
        }) => DocumentAtomOxidationObservationOutcomeV1::Accepted { oxidation_number },
        DocumentAtomOxidationResultV1::Observation(DocumentObservation::Unavailable { reason }) => {
            DocumentAtomOxidationObservationOutcomeV1::Unavailable {
                unavailable_reason: unavailable_reason(reason),
            }
        }
        DocumentAtomOxidationResultV1::ResourceLimit {
            resource,
            maximum: _,
            actual: _,
        } => {
            return Err(ExecutionFailureV1::oxidation_resource_limit(resource));
        }
    };
    Ok(OperationProtocolOutcomeV1::DocumentAtomOxidationObserve {
        observation: DocumentAtomOxidationObservationV1 {
            schema: OBSERVATION_SCHEMA_V1.to_owned(),
            source_revision: snapshot.revision(),
            source_digest_hex: hex_digest(snapshot.digest()),
            molecule_id: request.molecule_id,
            atom_id: request.atom_id,
            document_root_order,
            convention: OXIDATION_CONVENTION_V1.to_owned(),
            outcome,
        },
    })
}

fn invalid_request(message: &str) -> ExecutionFailureV1 {
    ExecutionFailureV1::invalid_request(message)
}

fn parse_digest(value: &str) -> Result<[u8; 32], ExecutionFailureV1> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(invalid_request(
            "expected_digest_hex must be a lowercase SHA-256 digest",
        ));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = hexadecimal_nibble(pair[0])
            .ok_or_else(|| invalid_request("expected_digest_hex is invalid"))?;
        let low = hexadecimal_nibble(pair[1])
            .ok_or_else(|| invalid_request("expected_digest_hex is invalid"))?;
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

const fn unavailable_reason(
    reason: DocumentUnavailable,
) -> DocumentAtomOxidationUnavailableReasonV1 {
    match reason {
        DocumentUnavailable::ElementOutsideProfile => {
            DocumentAtomOxidationUnavailableReasonV1::ElementOutsideProfile
        }
        DocumentUnavailable::FormalChargeUnavailable => {
            DocumentAtomOxidationUnavailableReasonV1::FormalChargeUnavailable
        }
        DocumentUnavailable::HydrogenTopologyUnsupported => {
            DocumentAtomOxidationUnavailableReasonV1::HydrogenTopologyUnsupported
        }
        DocumentUnavailable::AromaticityUnsupported => {
            DocumentAtomOxidationUnavailableReasonV1::AromaticityUnsupported
        }
        DocumentUnavailable::RadicalUnsupported => {
            DocumentAtomOxidationUnavailableReasonV1::RadicalUnsupported
        }
        DocumentUnavailable::BondOrderUnavailable => {
            DocumentAtomOxidationUnavailableReasonV1::BondOrderUnavailable
        }
        DocumentUnavailable::BondOrderUnsupported => {
            DocumentAtomOxidationUnavailableReasonV1::BondOrderUnsupported
        }
        DocumentUnavailable::NonAtomVertexUnsupported => {
            DocumentAtomOxidationUnavailableReasonV1::NonAtomVertexUnsupported
        }
        DocumentUnavailable::CoordinationOrDelocalizationUnsupported => {
            DocumentAtomOxidationUnavailableReasonV1::CoordinationOrDelocalizationUnsupported
        }
        DocumentUnavailable::ComponentInvariantFailed => {
            DocumentAtomOxidationUnavailableReasonV1::ComponentInvariantFailed
        }
        DocumentUnavailable::ArithmeticOverflow => {
            DocumentAtomOxidationUnavailableReasonV1::ArithmeticOverflow
        }
    }
}
