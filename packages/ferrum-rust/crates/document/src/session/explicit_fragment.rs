//! Explicit fragment creation over the generic admitted transition boundary.

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentObjectIdV1, DocumentSession, DocumentSessionError,
    PersistentId, PreparedSessionTransitionV1, RevisionState, SessionOperationError,
};
use crate::{DocumentExplicitFragmentRecordV1, explicit_fragment_v1::ExplicitFragmentCandidateV1};

/// A non-cloneable explicit-fragment receipt retaining only domain metadata.
pub struct PendingCreateExplicitFragmentV1 {
    revision: u64,
    record: DocumentExplicitFragmentRecordV1,
    transition: PreparedSessionTransitionV1,
}

impl PendingCreateExplicitFragmentV1 {
    #[must_use]
    pub fn record(&self) -> &DocumentExplicitFragmentRecordV1 {
        &self.record
    }
}

impl std::fmt::Debug for PendingCreateExplicitFragmentV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingCreateExplicitFragmentV1")
            .field("revision", &self.revision)
            .field("fragment_id", self.record.fragment_id())
            .field("is_resolved", &self.transition.is_consumed_v1())
            .finish()
    }
}

impl DocumentSession {
    /// Prepare one explicit-only molecule-local fragment record without mutation.
    pub fn prepare_create_explicit_fragment_v1(
        &mut self,
        expected_revision: u64,
        molecule_id: &DocumentObjectIdV1,
        name: &str,
        selected_atom_ids: &[PersistentId],
        selected_bond_ids: &[PersistentId],
    ) -> Result<PendingCreateExplicitFragmentV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let plan: ExplicitFragmentCandidateV1 = self
            .current_state_v1()
            .document()
            .prepare_explicit_fragment_v1(molecule_id, name, selected_atom_ids, selected_bond_ids)
            .map_err(SessionOperationError::from)?;
        let (fragment_id, effects) =
            self.reserve_generated_ids_for_transition_v1(|ids, indexed| {
                let (fragment_id, next) = ids.reserve_fragment(indexed)?;
                Ok((fragment_id, next))
            })?;
        let record = plan.record(fragment_id);
        let candidate = self
            .current_state_v1()
            .document()
            .apply_explicit_fragment_v1(&plan, record.fragment_id())
            .map_err(SessionOperationError::from)?;
        let (source_revision, source_digest, revision) = {
            let current = self.current_state_v1();
            (
                current.revision(),
                *current.digest(),
                current.next_revision(),
            )
        };
        let revision = revision.ok_or(DocumentSessionError::RevisionExhausted)?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let transition = self.prepare_changed_session_transition_v1(
            source_revision,
            source_digest,
            state,
            effects,
        )?;
        Ok(PendingCreateExplicitFragmentV1 {
            revision: expected_revision,
            record,
            transition,
        })
    }

    /// Commit one prepared explicit fragment exactly once.
    pub fn commit_create_explicit_fragment_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateExplicitFragmentV1,
    ) -> Result<super::SessionOperationResultV1, DocumentSessionError> {
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
