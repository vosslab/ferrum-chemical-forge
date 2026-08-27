//! Fenced durable live adapters for selected-atom chemistry actions.

use ferrum_document::{
    DocumentAtomOxidationObservationRequestV1, DocumentAtomOxidationObservationV1,
    DocumentAtomOxidationResultV1, DocumentAtomOxidationUnavailableReasonV1,
    DocumentMoleculeHydrogenMaterializationRequestV1,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use super::binding::PyDocumentSession;
use super::document_error_binding::{document_object_id, document_result};
use super::session_operation_result_binding::PySessionOperationResultV1;

/// Frozen read-only observation returned by one fenced live selected-atom request.
#[pyclass(frozen, name = "_LiveAtomOxidationObservationV1")]
pub(crate) struct PyLiveAtomOxidationObservationV1 {
    #[pyo3(get)]
    source_revision: u64,
    #[pyo3(get)]
    source_digest_hex: String,
    #[pyo3(get)]
    molecule_object_id: String,
    #[pyo3(get)]
    atom_object_id: String,
    #[pyo3(get)]
    document_root_order: usize,
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    oxidation_number: Option<i16>,
    #[pyo3(get)]
    unavailable_reason: Option<String>,
}

#[pymethods]
impl PyDocumentSession {
    /// Observe oxidation through one live durable molecule/atom address and exact fence.
    fn observe_live_atom_oxidation_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        molecule_object_id: String,
        atom_object_id: String,
    ) -> PyResult<PyLiveAtomOxidationObservationV1> {
        let snapshot = require_fence(py, &self.session, expected_revision, &expected_digest_hex)?;
        let molecule_id = document_object_id(py, molecule_object_id)?;
        let atom_id = document_object_id(py, atom_object_id)?;
        let observation = document_result(py, self.session.observe(expected_revision))?;
        let document_root_order = observation
            .projection()
            .molecules()
            .iter()
            .position(|molecule| molecule.document_object_id() == &molecule_id)
            .ok_or_else(|| {
                PyValueError::new_err(
                    "selected molecule is absent from the current live projection",
                )
            })?;
        let request = DocumentAtomOxidationObservationRequestV1::new(
            expected_revision,
            *snapshot.digest(),
            molecule_id.clone(),
            atom_id.clone(),
        );
        let (status, oxidation_number, unavailable_reason) = match self
            .session
            .observe_atom_oxidation_v1(&request)
            .map_err(|error| PyValueError::new_err(error.to_string()))?
        {
            DocumentAtomOxidationResultV1::Observation(
                DocumentAtomOxidationObservationV1::Accepted { oxidation_number },
            ) => ("accepted".to_owned(), Some(oxidation_number), None),
            DocumentAtomOxidationResultV1::Observation(
                DocumentAtomOxidationObservationV1::Unavailable { reason },
            ) => (
                "unavailable".to_owned(),
                None,
                Some(unavailable_reason(reason).to_owned()),
            ),
            DocumentAtomOxidationResultV1::ResourceLimit { .. } => (
                "unavailable".to_owned(),
                None,
                Some("resource_limit".to_owned()),
            ),
        };
        Ok(PyLiveAtomOxidationObservationV1 {
            source_revision: expected_revision,
            source_digest_hex: expected_digest_hex,
            molecule_object_id: molecule_id.as_str().to_owned(),
            atom_object_id: atom_id.as_str().to_owned(),
            document_root_order,
            status,
            oxidation_number,
            unavailable_reason,
        })
    }

    /// Materialize hydrogen atoms through one live durable molecule/atom address and exact fence.
    fn materialize_live_molecule_hydrogens_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        molecule_object_id: String,
        anchor_atom_object_id: String,
    ) -> PyResult<PySessionOperationResultV1> {
        let snapshot = require_fence(py, &self.session, expected_revision, &expected_digest_hex)?;
        let molecule_id = document_object_id(py, molecule_object_id)?;
        let anchor_atom_id = document_object_id(py, anchor_atom_object_id)?;
        let request = DocumentMoleculeHydrogenMaterializationRequestV1::new(
            expected_revision,
            *snapshot.digest(),
            molecule_id,
            anchor_atom_id,
        );
        document_result(
            py,
            self.session
                .materialize_molecule_hydrogens_v1(expected_revision, request),
        )
        .map(Into::into)
    }
}

fn require_fence(
    py: Python<'_>,
    session: &ferrum_document::DocumentSession,
    expected_revision: u64,
    expected_digest_hex: &str,
) -> PyResult<ferrum_document::DocumentSnapshot> {
    let expected_digest = parse_digest(expected_digest_hex)?;
    let snapshot = document_result(py, session.snapshot())?;
    if snapshot.revision() != expected_revision {
        return document_result(
            py,
            Err(ferrum_document::DocumentSessionError::RevisionConflict {
                expected: expected_revision,
                actual: snapshot.revision(),
            }),
        );
    }
    if snapshot.digest() != &expected_digest {
        return Err(PyValueError::new_err(
            "live document digest does not match the current document",
        ));
    }
    Ok(snapshot)
}

fn parse_digest(value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(PyValueError::new_err(
            "expected digest must be exactly 64 hexadecimal characters",
        ));
    }
    let mut digest = [0; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        digest[index] = (hex_value(pair[0]) << 4) | hex_value(pair[1]);
    }
    Ok(digest)
}

const fn hex_value(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => 0,
    }
}

const fn unavailable_reason(reason: DocumentAtomOxidationUnavailableReasonV1) -> &'static str {
    match reason {
        DocumentAtomOxidationUnavailableReasonV1::ElementOutsideProfile => {
            "element_outside_profile"
        }
        DocumentAtomOxidationUnavailableReasonV1::FormalChargeUnavailable => {
            "formal_charge_unavailable"
        }
        DocumentAtomOxidationUnavailableReasonV1::HydrogenTopologyUnsupported => {
            "hydrogen_topology_unsupported"
        }
        DocumentAtomOxidationUnavailableReasonV1::AromaticityUnsupported => {
            "aromaticity_unsupported"
        }
        DocumentAtomOxidationUnavailableReasonV1::RadicalUnsupported => "radical_unsupported",
        DocumentAtomOxidationUnavailableReasonV1::BondOrderUnavailable => "bond_order_unavailable",
        DocumentAtomOxidationUnavailableReasonV1::BondOrderUnsupported => "bond_order_unsupported",
        DocumentAtomOxidationUnavailableReasonV1::NonAtomVertexUnsupported => {
            "non_atom_vertex_unsupported"
        }
        DocumentAtomOxidationUnavailableReasonV1::CoordinationOrDelocalizationUnsupported => {
            "coordination_or_delocalization_unsupported"
        }
        DocumentAtomOxidationUnavailableReasonV1::ComponentInvariantFailed => {
            "component_invariant_failed"
        }
        DocumentAtomOxidationUnavailableReasonV1::ArithmeticOverflow => "arithmetic_overflow",
    }
}
