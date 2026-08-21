//! One-use session ownership for planned direct-molecule structural deletion.

use super::{
    DocumentSession, DocumentSessionError, GeneratedIdSequences, PersistentId, ProvisionalToken,
    RevisionState, SessionDocumentObservationV1, SessionOperationError, SessionOperationResultV1,
};
use crate::StructureDeletionReceiptV1;

/// A revision-bound structural deletion candidate with session-owned split IDs.
pub struct PendingDeleteStructureV1 {
    revision: u64,
    token: ProvisionalToken,
    receipt: StructureDeletionReceiptV1,
    candidate: Option<RevisionState>,
    operation: Option<SessionOperationResultV1>,
    tentative_generated_ids: GeneratedIdSequences,
}

impl PendingDeleteStructureV1 {
    /// Return the planned source-order deletion facts before or after commit.
    #[must_use]
    pub fn receipt(&self) -> &StructureDeletionReceiptV1 {
        &self.receipt
    }
}

impl DocumentSession {
    /// Prepare direct structural deletion and reserve only required split roots.
    pub fn prepare_delete_structure_v1(
        &mut self,
        expected_revision: u64,
        molecule_id: String,
        atom_ids: Vec<String>,
        bond_ids: Vec<String>,
    ) -> Result<PendingDeleteStructureV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let molecule =
            PersistentId::new(molecule_id).map_err(|_| SessionOperationError::UnknownMolecule)?;
        let atoms = atom_ids
            .into_iter()
            .map(PersistentId::new)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| {
                SessionOperationError::UnknownAtom("invalid structural atom".to_owned())
            })?;
        let bonds = bond_ids
            .into_iter()
            .map(PersistentId::new)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| {
                SessionOperationError::UnknownBond("invalid structural bond".to_owned())
            })?;
        let plan = self
            .history
            .current()
            .document()
            .prepare_delete_structure(molecule, atoms, bonds)
            .map_err(SessionOperationError::Candidate)?;
        let (later_ids, tentative_generated_ids) = self.generated_ids.reserve_molecule_roots(
            self.history.current().document().indexed(),
            plan.additional_molecule_count(),
        )?;
        let (document, receipt) = self
            .history
            .current()
            .document()
            .commit_delete_structure(&plan, &later_ids)
            .map_err(SessionOperationError::Candidate)?;
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate =
            RevisionState::from_document(revision, document).map_err(DocumentSessionError::Load)?;
        let snapshot = candidate.snapshot(!self.saved_baseline.is_current(&candidate));
        let observation = SessionDocumentObservationV1::from_state(candidate.document(), snapshot)
            .map_err(DocumentSessionError::Projection)?;
        let token =
            super::prepared::issue_prepared_token(self.history.current_mut().document_mut())?;
        Ok(PendingDeleteStructureV1 {
            revision: expected_revision,
            token,
            receipt,
            candidate: Some(candidate),
            operation: Some(SessionOperationResultV1::new(observation)),
            tentative_generated_ids,
        })
    }

    /// Commit a prepared structural deletion exactly once.
    pub fn commit_delete_structure_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingDeleteStructureV1,
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
        self.generated_ids = pending.tentative_generated_ids;
        Ok(operation)
    }
}
