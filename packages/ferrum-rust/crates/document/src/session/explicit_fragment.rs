//! One-use session containment for explicit fragment annotation creation.

use super::{
    DocumentObjectIdV1, DocumentSession, DocumentSessionError, GeneratedIdSequences, PersistentId,
    ProvisionalToken, RevisionState, SessionDocumentObservationV1, SessionOperationError,
    SessionOperationResultV1,
};
use crate::{DocumentExplicitFragmentRecordV1, explicit_fragment_v1::ExplicitFragmentCandidateV1};

/// A non-cloneable, revision-bound explicit fragment candidate.
pub struct PendingCreateExplicitFragmentV1 {
    revision: u64,
    token: ProvisionalToken,
    record: DocumentExplicitFragmentRecordV1,
    candidate: Option<RevisionState>,
    operation: Option<SessionOperationResultV1>,
    tentative_generated_ids: Option<GeneratedIdSequences>,
}

impl PendingCreateExplicitFragmentV1 {
    /// Return scalar facts that will be committed if this one-use receipt is accepted.
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
            .field("is_resolved", &self.candidate.is_none())
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
            .history
            .current()
            .document()
            .prepare_explicit_fragment_v1(molecule_id, name, selected_atom_ids, selected_bond_ids)
            .map_err(SessionOperationError::from)?;
        let (fragment_id, generated_ids) = self
            .generated_ids
            .reserve_fragment(self.history.current().document().indexed())?;
        let record = plan.record(fragment_id);
        let candidate = self
            .history
            .current()
            .document()
            .apply_explicit_fragment_v1(&plan, record.fragment_id())
            .map_err(SessionOperationError::from)?;
        let revision = self
            .history
            .current()
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
        let token = self
            .history
            .current_mut()
            .document_mut()
            .try_issue_provisional_token()
            .map_err(SessionOperationError::Candidate)?;
        Ok(PendingCreateExplicitFragmentV1 {
            revision: expected_revision,
            token,
            record,
            candidate: Some(candidate),
            operation: Some(SessionOperationResultV1::new(observation)),
            tentative_generated_ids: Some(generated_ids),
        })
    }

    /// Commit one prepared explicit fragment exactly once.
    pub fn commit_create_explicit_fragment_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateExplicitFragmentV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if pending.candidate.is_none() {
            return Err(DocumentSessionError::PreparedOperationConsumed);
        }
        if pending.revision != expected_revision {
            return Err(DocumentSessionError::RevisionConflict {
                expected: pending.revision,
                actual: expected_revision,
            });
        }
        self.history
            .current()
            .document()
            .verify_provisional_token(&pending.token)
            .map_err(super::prepared::map_prepared_token_error)?;
        self.history
            .try_reserve_append()
            .map_err(|_| SessionOperationError::HistoryResourceExhausted)?;
        let (Some(state), Some(operation)) = (pending.candidate.take(), pending.operation.take())
        else {
            return Err(DocumentSessionError::PreparedOperationConsumed);
        };
        if let Err(error) = self
            .history
            .current_mut()
            .document_mut()
            .consume_provisional_token(&pending.token)
        {
            pending.candidate = Some(state);
            pending.operation = Some(operation);
            return Err(SessionOperationError::Candidate(error).into());
        }
        self.history.append_reserved(state);
        if let Some(generated_ids) = pending.tentative_generated_ids.take() {
            self.generated_ids = generated_ids;
        }
        Ok(operation)
    }
}
