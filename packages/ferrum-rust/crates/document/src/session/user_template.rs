//! User-template placement through the generic admitted transition boundary.

use ferrum_geometry::Point2;

use crate::{
    DocumentUserTemplateErrorV1, DocumentUserTemplateInsertedMoleculeV1, DocumentUserTemplatePlanV1,
};

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentSession, DocumentSessionError, RevisionState,
    SessionOperationError, SessionOperationResultV1,
};

/// Exact authoritative outcome of one accepted user-template insertion.
#[derive(Debug)]
pub struct DocumentUserTemplateResultV1 {
    operation: SessionOperationResultV1,
    inserted_molecule: DocumentUserTemplateInsertedMoleculeV1,
}

impl DocumentUserTemplateResultV1 {
    #[must_use]
    pub fn operation_result(&self) -> &SessionOperationResultV1 {
        &self.operation
    }
    #[must_use]
    pub fn into_operation_result(self) -> SessionOperationResultV1 {
        self.operation
    }
    #[must_use]
    pub fn inserted_molecule(&self) -> &DocumentUserTemplateInsertedMoleculeV1 {
        &self.inserted_molecule
    }
}

impl DocumentSession {
    /// Place one immutable user template through renderer admission.
    pub fn insert_document_user_template_v1(
        &mut self,
        expected_revision: u64,
        expected_digest: &[u8; 32],
        plan: &DocumentUserTemplatePlanV1,
        anchor: Point2,
    ) -> Result<DocumentUserTemplateResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if self.current_state_v1().digest() != expected_digest {
            return Err(DocumentUserTemplateErrorV1::DigestMismatch.into());
        }
        let (generated, effects, source_revision, source_digest, revision) = {
            let (generated, effects) =
                self.reserve_generated_ids_for_transition_v1(|ids, indexed| {
                    ids.reserve_fragment_import(indexed, plan.declared_id_count())
                })?;
            let current = self.current_state_v1();
            (
                generated,
                effects,
                current.revision(),
                *current.digest(),
                current.next_revision(),
            )
        };
        let revision = revision.ok_or(DocumentSessionError::RevisionExhausted)?;
        let (candidate, inserted_molecule) =
            super::super::user_template_v1::compose_document_user_template_candidate_v1(
                self.current_state_v1().document(),
                plan,
                &generated,
                anchor,
            )?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let mut transition = self.prepare_changed_session_transition_v1(
            source_revision,
            source_digest,
            state,
            effects,
        )?;
        let operation = self
            .commit_session_operation_transition_v1(&mut transition)
            .map_err(|refusal| map_transition_refusal(self, expected_revision, refusal))?;
        Ok(DocumentUserTemplateResultV1 {
            operation,
            inserted_molecule,
        })
    }
}

fn map_transition_refusal(
    session: &DocumentSession,
    expected_revision: u64,
    refusal: AdmittedSessionTransitionRefusalV1,
) -> DocumentSessionError {
    match refusal {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            DocumentSessionError::PreparedOperationForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Replayed
        | AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            DocumentSessionError::PreparedOperationConsumed
        }
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            DocumentSessionError::RevisionConflict {
                expected: expected_revision,
                actual: session.current_revision_v1(),
            }
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            DocumentSessionError::RendererAdmission
        }
        AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
            SessionOperationError::HistoryResourceExhausted.into()
        }
    }
}
