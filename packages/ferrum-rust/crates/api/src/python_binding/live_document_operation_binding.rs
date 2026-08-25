//! Private Python transport for registered live-document operations.

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use super::binding::{PyDocumentSession, PySessionOperationResultV1};

/// Frozen result of one generic live-document operation.
#[pyclass(frozen, name = "_LiveDocumentOperationReceiptV1")]
pub(crate) struct PyLiveDocumentOperationReceiptV1 {
    #[pyo3(get)]
    response_json: String,
    #[pyo3(get)]
    mutation_result: Option<PySessionOperationResultV1>,
    #[pyo3(get)]
    source_revision: u64,
    #[pyo3(get)]
    source_digest: Py<PyBytes>,
}

#[pymethods]
impl PyDocumentSession {
    /// Execute one registered operation against this exact live session.
    ///
    /// The request CDML is a fence witness: it must be byte-for-byte equal to
    /// the current snapshot and can never replace the owned session document.
    fn apply_live_document_operation_v1(
        &mut self,
        py: Python<'_>,
        request_json: &str,
    ) -> PyResult<PyLiveDocumentOperationReceiptV1> {
        let receipt =
            crate::protocol::live_document_operation_v1::execute_live_document_operation_v1(
                &mut self.session,
                request_json,
            )
            .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))?;
        Ok(PyLiveDocumentOperationReceiptV1 {
            response_json: receipt.response_json().to_owned(),
            mutation_result: receipt.mutation_result().cloned().map(Into::into),
            source_revision: receipt.source_revision(),
            source_digest: PyBytes::new(py, receipt.source_digest()).unbind(),
        })
    }
}

#[cfg(test)]
mod tests {
    use ferrum_document::{
        MoleculeInsertionAtomV1, MoleculeInsertionV1, Point3V1, SessionOperation,
        SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
    };
    use pyo3::types::PyAnyMethods;
    use serde_json::json;

    use super::*;

    fn session() -> ferrum_document::DocumentSession {
        let mut session =
            ferrum_document::DocumentSession::create_empty_document_v1().expect("empty document");
        let oxygen = MoleculeInsertionAtomV1::new(
            "O",
            Point3V1::new(0.0, 0.0, 0.0).expect("finite coordinate"),
            None,
            None,
            None,
        )
        .expect("oxygen atom");
        let insertion = MoleculeInsertionV1::new(vec![oxygen], Vec::new()).expect("molecule");
        let revision = session.snapshot().expect("snapshot").revision();
        let mut prepared = session
            .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
                revision,
                SessionOperation::V1(SessionOperationV1::InsertMoleculeV1(insertion.into())),
                TransitionAuthorizationV1::none(),
            ))
            .expect("ordinary skeleton preparation");
        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("ordinary skeleton installation");
        session
    }

    fn request(session: &ferrum_document::DocumentSession) -> String {
        let snapshot = session.snapshot().expect("snapshot");
        let observation = session.observe(snapshot.revision()).expect("observation");
        let molecule = &observation.projection().molecules()[0];
        let digest = snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        json!({
            "schema": crate::OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "live-binding-materialization-test",
            "operation": {
                "kind": "document.molecule.hydrogen.materialize.v1",
                "document": {
                    "cdml": snapshot.cdml(),
                    "expected_revision": snapshot.revision(),
                    "expected_digest_hex": digest,
                },
                "molecule_id": molecule.id().expect("durable molecule").as_str(),
                "anchor_atom_id": molecule.atoms()[0].id().expect("durable atom").as_str(),
            },
        })
        .to_string()
    }

    #[test]
    fn python_live_operation_receipt_preserves_generic_hydrogen_outcome_facts() {
        Python::initialize();
        Python::attach(|py| {
            let mut live = PyDocumentSession::from_session(session());
            let applied_request = request(&live.session);
            let receipt = live
                .apply_live_document_operation_v1(py, &applied_request)
                .expect("accepted live operation");
            let receipt = Py::new(py, receipt).expect("frozen Python receipt");
            let applied = receipt
                .bind(py)
                .getattr("mutation_result")
                .expect("applied mutation result");
            let applied_outcome = applied.getattr("outcome").expect("applied outcome");
            let applied_materialization = applied_outcome
                .getattr("molecule_hydrogens_materialized")
                .expect("applied hydrogen materialization outcome");
            let anchor_atom_identifier = applied_materialization
                .getattr("anchor_atom_identifier")
                .expect("applied anchor identifier")
                .extract::<String>()
                .expect("string anchor identifier");
            assert!(
                applied_materialization
                    .getattr("changed")
                    .expect("applied changed fact")
                    .extract::<bool>()
                    .expect("boolean changed fact")
            );
            assert!(
                applied_materialization
                    .getattr("added_hydrogen_count")
                    .expect("applied hydrogen count")
                    .extract::<usize>()
                    .expect("nonnegative applied hydrogen count")
                    > 0
            );
            let no_op_request = request(&live.session);
            let no_op = live
                .apply_live_document_operation_v1(py, &no_op_request)
                .expect("accepted no-op operation");
            let no_op = Py::new(py, no_op).expect("frozen Python receipt");
            let no_op = no_op
                .bind(py)
                .getattr("mutation_result")
                .expect("no-change mutation result");
            let no_op_outcome = no_op.getattr("outcome").expect("no-change outcome");
            let no_op_materialization = no_op_outcome
                .getattr("molecule_hydrogens_materialized")
                .expect("no-change hydrogen materialization outcome");
            assert!(
                !no_op_materialization
                    .getattr("changed")
                    .expect("no-change changed fact")
                    .extract::<bool>()
                    .expect("boolean no-change fact")
            );
            assert_eq!(
                no_op_materialization
                    .getattr("added_hydrogen_count")
                    .expect("no-change hydrogen count")
                    .extract::<usize>()
                    .expect("nonnegative no-change hydrogen count"),
                0
            );
            assert_eq!(
                no_op_materialization
                    .getattr("anchor_atom_identifier")
                    .expect("no-change anchor identifier")
                    .extract::<String>()
                    .expect("string no-change anchor identifier"),
                anchor_atom_identifier
            );
        });
    }
}
