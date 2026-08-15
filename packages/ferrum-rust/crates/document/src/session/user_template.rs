//! Immediate authenticated transaction ownership for native user templates.

use ferrum_geometry::Point2;

use crate::{
    DocumentUserTemplateErrorV1, DocumentUserTemplateInsertedMoleculeV1, DocumentUserTemplatePlanV1,
};

use super::{
    DocumentSession, DocumentSessionError, RevisionState, SessionDocumentObservationV1,
    SessionOperationError, SessionOperationResultV1,
};

/// Exact authoritative outcome of one accepted user-template insertion.
#[derive(Debug)]
pub struct DocumentUserTemplateResultV1 {
    operation: SessionOperationResultV1,
    inserted_molecule: DocumentUserTemplateInsertedMoleculeV1,
}

impl DocumentUserTemplateResultV1 {
    /// Return the complete post-insertion observation.
    #[must_use]
    pub fn operation_result(&self) -> &SessionOperationResultV1 {
        &self.operation
    }

    /// Consume the receipt and return its authoritative observation wrapper.
    #[must_use]
    pub fn into_operation_result(self) -> SessionOperationResultV1 {
        self.operation
    }

    /// Return the fresh durable molecule installed by this insertion.
    #[must_use]
    pub fn inserted_molecule(&self) -> &DocumentUserTemplateInsertedMoleculeV1 {
        &self.inserted_molecule
    }
}

impl DocumentSession {
    /// Place one admitted user template as one authenticated history transition.
    pub fn insert_document_user_template_v1(
        &mut self,
        expected_revision: u64,
        expected_digest: &[u8; 32],
        plan: &DocumentUserTemplatePlanV1,
        anchor: Point2,
    ) -> Result<DocumentUserTemplateResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let current = self.history.current();
        if current.digest() != expected_digest {
            return Err(DocumentUserTemplateErrorV1::DigestMismatch.into());
        }
        let (generated, tentative_generated_ids) = self
            .generated_ids
            .reserve_fragment_import(current.document().indexed(), plan.declared_id_count())?;
        let (candidate, inserted_molecule) =
            super::super::user_template_v1::compose_document_user_template_candidate_v1(
                current.document(),
                plan,
                &generated,
                anchor,
            )?;
        let revision = current
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let snapshot = candidate.snapshot(!self.saved_baseline.is_current(&candidate));
        let observation = SessionDocumentObservationV1::from_state(candidate.document(), snapshot)
            .map_err(DocumentSessionError::Projection)?;
        self.history
            .try_reserve_append()
            .map_err(|_| SessionOperationError::HistoryResourceExhausted)?;
        self.history.append_reserved(candidate);
        self.generated_ids = tentative_generated_ids;
        Ok(DocumentUserTemplateResultV1 {
            operation: SessionOperationResultV1::new(observation),
            inserted_molecule,
        })
    }
}
