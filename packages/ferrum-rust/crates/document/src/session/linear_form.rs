//! Revision-bound linear-form conversion over the generic admitted transition.

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentObjectIdV1, DocumentSession, DocumentSessionError,
    PersistentId, PreparedSessionTransitionV1, RevisionState, SessionOperationError,
    SessionOperationResultV1, SessionTransitionEffectsV1,
};
use crate::linear_form_convert_v1::{LinearFormCandidateV1, LinearFormDocumentErrorV1};

/// A one-use native linear-form conversion with its durable fragment identity.
pub struct PendingLinearFormConvertV1 {
    revision: u64,
    fragment_id: PersistentId,
    transition: PreparedSessionTransitionV1,
}

/// The complete outcome of classifying one native linear-form request.
pub enum PreparedLinearFormConvertResultV1 {
    NoChange(Box<SessionOperationResultV1>),
    Pending(Box<PendingLinearFormConvertV1>),
}

impl std::fmt::Debug for PreparedLinearFormConvertResultV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoChange(_) => formatter.write_str("PreparedLinearFormConvertResultV1::NoChange"),
            Self::Pending(pending) => formatter.debug_tuple("Pending").field(pending).finish(),
        }
    }
}

impl std::fmt::Debug for PendingLinearFormConvertV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingLinearFormConvertV1")
            .field("revision", &self.revision)
            .field("fragment_id", &self.fragment_id)
            .field("is_resolved", &self.transition.is_consumed_v1())
            .finish()
    }
}

impl PendingLinearFormConvertV1 {
    #[must_use]
    pub fn fragment_id(&self) -> &PersistentId {
        &self.fragment_id
    }
}

impl DocumentSession {
    /// Classify and prepare one exact selected-atom native linear-form conversion.
    pub fn prepare_convert_linear_form_v1(
        &mut self,
        expected_revision: u64,
        molecule_object_id: &DocumentObjectIdV1,
        selected_atoms: &[PersistentId],
    ) -> Result<PreparedLinearFormConvertResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if selected_atoms.is_empty() {
            return Err(SessionOperationError::EmptyLinearFormSelection.into());
        }
        let classified = self
            .current_state_v1()
            .document()
            .prepare_linear_form_convert_v1(molecule_object_id, selected_atoms)
            .map_err(map_linear_form_error)?;
        let (classified, effects) = match classified {
            LinearFormCandidateV1::NeedFragmentId => {
                let (fragment_id, effects) =
                    self.reserve_generated_ids_for_transition_v1(|ids, indexed| {
                        let (fragment_id, next) = ids.reserve_fragment(indexed)?;
                        Ok((fragment_id, next))
                    })?;
                let candidate = self
                    .current_state_v1()
                    .document()
                    .apply_linear_form_convert_v1(molecule_object_id, selected_atoms, &fragment_id)
                    .map_err(map_linear_form_error)?;
                (candidate, effects)
            }
            other => (other, SessionTransitionEffectsV1::none()),
        };
        let LinearFormCandidateV1::Repair {
            candidate,
            fragment_id,
        } = classified
        else {
            return self
                .operation_result()
                .map(Box::new)
                .map(PreparedLinearFormConvertResultV1::NoChange);
        };
        let (source_revision, source_digest, revision) = {
            let current = self.current_state_v1();
            (
                current.revision(),
                *current.digest(),
                current.next_revision(),
            )
        };
        let revision = revision.ok_or(DocumentSessionError::RevisionExhausted)?;
        let state = RevisionState::from_document(revision, *candidate)
            .map_err(DocumentSessionError::Load)?;
        let transition = self.prepare_changed_session_transition_v1(
            source_revision,
            source_digest,
            state,
            effects,
        )?;
        Ok(PreparedLinearFormConvertResultV1::Pending(Box::new(
            PendingLinearFormConvertV1 {
                revision: expected_revision,
                fragment_id,
                transition,
            },
        )))
    }

    /// Commit a prepared native linear-form conversion exactly once.
    pub fn commit_convert_linear_form_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingLinearFormConvertV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        if pending.transition.is_consumed_v1() {
            return Err(DocumentSessionError::PreparedOperationConsumed);
        }
        if pending.revision != expected_revision {
            return Err(DocumentSessionError::RevisionConflict {
                expected: pending.revision,
                actual: expected_revision,
            });
        }
        self.commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(|refusal| map_transition_refusal(self, pending.revision, refusal))
    }
}

fn map_linear_form_error(error: LinearFormDocumentErrorV1) -> DocumentSessionError {
    match error {
        LinearFormDocumentErrorV1::Plan(error) => {
            SessionOperationError::LinearFormPlan(error).into()
        }
        LinearFormDocumentErrorV1::Document(error) => {
            SessionOperationError::Candidate(error).into()
        }
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
