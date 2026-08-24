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
    use ferrum_document::{MoleculeInsertionAtomV1, MoleculeInsertionV1, Point3V1};
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
            .prepare_admitted_molecule_insertion_v1(revision, &insertion)
            .expect("ordinary skeleton preparation");
        session
            .commit_admitted_molecule_insertion_v1(revision, &mut prepared)
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
    fn python_live_operation_receipt_exposes_an_installable_result_only_after_change() {
        Python::initialize();
        Python::attach(|py| {
            let exported = Py::new(py, PyDocumentSession::from_session(session()))
                .expect("Python document session");
            assert!(
                exported
                    .bind(py)
                    .hasattr("apply_live_document_operation_v1")
                    .expect("live operation method lookup")
            );
            assert!(
                !exported
                    .bind(py)
                    .hasattr("_apply_live_document_operation_v1")
                    .expect("retired live operation method lookup")
            );
            let mut live = PyDocumentSession::from_session(session());
            let applied_request = request(&live.session);
            let receipt = live
                .apply_live_document_operation_v1(py, &applied_request)
                .expect("accepted live operation");
            let receipt = Py::new(py, receipt).expect("frozen Python receipt");
            assert!(
                !receipt
                    .bind(py)
                    .getattr("mutation_result")
                    .expect("receipt mutation result")
                    .is_none()
            );
            let no_op_request = request(&live.session);
            let no_op = live
                .apply_live_document_operation_v1(py, &no_op_request)
                .expect("accepted no-op operation");
            let no_op = Py::new(py, no_op).expect("frozen Python receipt");
            assert!(
                no_op
                    .bind(py)
                    .getattr("mutation_result")
                    .expect("no-op mutation result")
                    .is_none()
            );
            let source_digest = receipt
                .bind(py)
                .getattr("source_digest")
                .expect("receipt source digest")
                .extract::<Vec<u8>>()
                .expect("binary source digest");
            assert_eq!(source_digest.len(), 32);
        });
    }
}
