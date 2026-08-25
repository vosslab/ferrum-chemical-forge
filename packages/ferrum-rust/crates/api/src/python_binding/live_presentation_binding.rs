//! Fenced durable presentation mutation adapters for the live Python session.

use ferrum_document::{
    PresentationRootDeletionSetV1, PresentationRootDeletionV1, PresentationStackReorderV1,
    SessionOperation, SessionOperationV1,
};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::binding::{PyDocumentSession, PySessionOperationResultV1, operation_validation_error};
use super::document_error_binding::document_result;
use super::document_session_binding::require_live_fence;

#[pymethods]
impl PyDocumentSession {
    /// Delete exact durable presentation roots under one current live fence.
    fn apply_live_presentation_deletion_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        targets: &Bound<'_, PyTuple>,
    ) -> PyResult<PySessionOperationResultV1> {
        require_live_fence(py, &self.session, expected_revision, &expected_digest_hex)?;
        let targets = super::presentation_stack_binding::live_targets(
            py,
            targets,
            "live presentation deletion",
        )?;
        let targets = document_result(
            py,
            self.session
                .lower_live_presentation_roots_v1(&targets)
                .map_err(ferrum_document::DocumentSessionError::Operation),
        )?;
        let deletions = targets
            .into_iter()
            .map(|target| {
                PresentationRootDeletionV1::new(target.document_object_id().clone(), target.kind())
            })
            .collect();
        let deletions = PresentationRootDeletionSetV1::new(deletions)
            .map_err(|error| operation_validation_error(py, error.to_string()))?;
        let operation =
            SessionOperation::V1(SessionOperationV1::DeletePresentationRoots { deletions });
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }

    /// Reorder exact durable presentation roots under one current live fence.
    fn apply_live_presentation_reorder_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        order: PyRef<'_, super::presentation_stack_binding::PyDocumentPresentationStackOrderV1>,
        targets: &Bound<'_, PyTuple>,
    ) -> PyResult<PySessionOperationResultV1> {
        require_live_fence(py, &self.session, expected_revision, &expected_digest_hex)?;
        let targets = super::presentation_stack_binding::live_targets(
            py,
            targets,
            "live presentation reorder",
        )?;
        let targets = document_result(
            py,
            self.session
                .lower_live_presentation_roots_v1(&targets)
                .map_err(ferrum_document::DocumentSessionError::Operation),
        )?;
        let reorder = PresentationStackReorderV1::new((*order).into(), targets)
            .map_err(|error| operation_validation_error(py, error.to_string()))?;
        let operation =
            SessionOperation::V1(SessionOperationV1::ReorderPresentationRoots { reorder });
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }
}
