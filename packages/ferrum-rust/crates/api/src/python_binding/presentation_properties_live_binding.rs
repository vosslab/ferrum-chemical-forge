//! Fenced durable-root adapters for direct presentation property mutations.

use ferrum_document::{DocumentObjectIdV1, DocumentSessionError, TopLevelRootKindV1};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::document_error_binding::{document_object_id, document_result};
use super::document_session_binding::{PyDocumentSession, require_live_fence};
use super::session_operation_result_binding::PySessionOperationResultV1;

#[pymethods]
impl PyDocumentSession {
    /// Apply a fenced durable Plus-property patch to one current direct root.
    fn set_plus_properties_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        plus_object_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<PySessionOperationResultV1> {
        let plus_id = presentation_root_document_object_id(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            plus_object_id,
            TopLevelRootKindV1::Plus,
        )?;
        let operation = super::plus_properties_binding::set_plus_properties(py, plus_id, changes)?;
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }

    /// Apply a fenced durable Arrow-property patch to one current direct root.
    fn set_arrow_properties_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        arrow_object_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<PySessionOperationResultV1> {
        let arrow_id = presentation_root_document_object_id(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            arrow_object_id,
            TopLevelRootKindV1::Arrow,
        )?;
        let operation =
            super::arrow_properties_binding::set_arrow_properties(py, arrow_id, changes)?;
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }

    /// Apply a fenced durable Text-property patch to one current direct root.
    fn set_text_properties_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        text_object_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<PySessionOperationResultV1> {
        let text_id = presentation_root_document_object_id(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            text_object_id,
            TopLevelRootKindV1::Text,
        )?;
        let operation = super::text_properties_binding::set_text_properties(py, text_id, changes)?;
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }
}

fn presentation_root_document_object_id(
    py: Python<'_>,
    session: &ferrum_document_render::RenderInteractionSessionV1,
    expected_revision: u64,
    expected_digest_hex: &str,
    object_id: String,
    kind: TopLevelRootKindV1,
) -> PyResult<DocumentObjectIdV1> {
    require_live_fence(py, session, expected_revision, expected_digest_hex)?;
    let object_id = document_object_id(py, object_id)?;
    document_result(
        py,
        session
            .lower_live_top_level_roots_v1(&[(object_id.clone(), kind)])
            .map_err(DocumentSessionError::Operation),
    )?;
    Ok(object_id)
}

#[cfg(test)]
mod tests {
    use ferrum_document::DocumentFenceV1;

    use super::*;

    const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><plus id=\"plus\"><point x=\"0\" y=\"0\"/></plus><arrow id=\"arrow\" type=\"normal\"><point x=\"0\" y=\"0\"/><point x=\"10\" y=\"0\"/></arrow><text id=\"text\"><point x=\"0\" y=\"0\"/><ftext>text</ftext></text></cdml>";

    fn plus_root_document_object_id(
        live: &PyDocumentSession,
        snapshot: &ferrum_document::DocumentSnapshot,
    ) -> String {
        live.session
            .observe_render_interaction_v1(DocumentFenceV1::new(
                snapshot.revision(),
                *snapshot.digest(),
            ))
            .expect("current render interaction observation")
            .roots()
            .iter()
            .find(|root| root.kind() == TopLevelRootKindV1::Plus)
            .expect("render observation exposes Plus root")
            .document_object_id()
            .as_str()
            .to_owned()
    }

    fn digest(session: &ferrum_document_render::RenderInteractionSessionV1) -> String {
        session
            .snapshot()
            .expect("current snapshot")
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    #[test]
    fn durable_presentation_property_lowering_requires_current_fenced_root_kind() {
        Python::initialize();
        Python::attach(|py| {
            let live = PyDocumentSession::from_session(
                ferrum_document::DocumentSession::load(SOURCE).expect("document"),
            );
            let before = live.session.snapshot().expect("before mutation");
            let current_digest = digest(&live.session);
            let plus_document_object_id = plus_root_document_object_id(&live, &before);
            assert_eq!(
                presentation_root_document_object_id(
                    py,
                    &live.session,
                    before.revision(),
                    &current_digest,
                    plus_document_object_id.clone(),
                    TopLevelRootKindV1::Plus,
                )
                .expect("current Plus lowers")
                .as_str(),
                plus_document_object_id.clone(),
            );
            assert!(
                presentation_root_document_object_id(
                    py,
                    &live.session,
                    before.revision(),
                    &current_digest,
                    plus_document_object_id.clone(),
                    TopLevelRootKindV1::Arrow,
                )
                .is_err()
            );
            assert!(
                presentation_root_document_object_id(
                    py,
                    &live.session,
                    before.revision() + 1,
                    &current_digest,
                    plus_document_object_id,
                    TopLevelRootKindV1::Plus,
                )
                .is_err()
            );
            assert_eq!(live.session.snapshot().expect("after lowering"), before);
        });
    }
}
