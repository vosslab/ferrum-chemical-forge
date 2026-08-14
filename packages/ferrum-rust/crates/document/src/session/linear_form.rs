//! Revision-bound transaction ownership for native linear-form conversion.

use super::{
    DocumentObjectIdV1, DocumentSession, DocumentSessionError, GeneratedIdSequences, PersistentId,
    ProvisionalToken, RevisionState, SessionDocumentObservationV1, SessionOperationError,
    SessionOperationResultV1,
};
use crate::linear_form_convert_v1::{LinearFormCandidateV1, LinearFormDocumentErrorV1};

/// A one-use, revision-bound native linear-form candidate.
///
/// The receipt is deliberately not `Clone`: its detached revision, opaque token,
/// precomputed result, and optional tentative fragment sequence have one owner.
pub struct PendingLinearFormConvertV1 {
    revision: u64,
    token: ProvisionalToken,
    fragment_id: PersistentId,
    candidate: Option<RevisionState>,
    operation: Option<SessionOperationResultV1>,
    tentative_generated_ids: Option<GeneratedIdSequences>,
}

/// The complete outcome of classifying one native linear-form request.
pub enum PreparedLinearFormConvertResultV1 {
    /// The authoritative current state already has the canonical conversion.
    NoChange(Box<SessionOperationResultV1>),
    /// A detached changed state awaits its one authenticated commit.
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
            .field("is_resolved", &self.candidate.is_none())
            .finish()
    }
}

impl PendingLinearFormConvertV1 {
    /// Return the exact generated-record ID that the committed candidate owns.
    #[must_use]
    pub fn fragment_id(&self) -> &PersistentId {
        &self.fragment_id
    }
}

impl DocumentSession {
    /// Classify and prepare one exact selected-atom native linear-form conversion.
    ///
    /// A no-change answer is returned immediately from the authoritative current
    /// state. Changed candidates own their token until successful commit or drop.
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
            .history
            .current()
            .document()
            .prepare_linear_form_convert_v1(molecule_object_id, selected_atoms)
            .map_err(map_linear_form_error)?;
        let (classified, tentative_generated_ids) = match classified {
            LinearFormCandidateV1::NeedFragmentId => {
                let (fragment_id, sequences) = self
                    .generated_ids
                    .reserve_fragment(self.history.current().document().indexed())?;
                let candidate = self
                    .history
                    .current()
                    .document()
                    .apply_linear_form_convert_v1(molecule_object_id, selected_atoms, &fragment_id)
                    .map_err(map_linear_form_error)?;
                (candidate, Some(sequences))
            }
            other => (other, None),
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
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, *candidate)
            .map_err(DocumentSessionError::Load)?;
        let snapshot = candidate.snapshot(!self.saved_baseline.is_current(&candidate));
        let observation = SessionDocumentObservationV1::from_state(candidate.document(), snapshot)
            .map_err(DocumentSessionError::Projection)?;
        let operation = SessionOperationResultV1::new(observation);
        self.history
            .try_reserve_append()
            .map_err(|_| SessionOperationError::HistoryResourceExhausted)?;
        let token = self
            .history
            .current_mut()
            .document_mut()
            .try_issue_provisional_token()
            .map_err(SessionOperationError::Candidate)?;
        Ok(PreparedLinearFormConvertResultV1::Pending(Box::new(
            PendingLinearFormConvertV1 {
                revision: expected_revision,
                token,
                fragment_id,
                candidate: Some(candidate),
                operation: Some(operation),
                tentative_generated_ids,
            },
        )))
    }

    /// Commit a prepared native linear-form conversion exactly once.
    pub fn commit_convert_linear_form_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingLinearFormConvertV1,
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
