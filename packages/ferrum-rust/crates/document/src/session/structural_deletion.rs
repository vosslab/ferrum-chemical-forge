//! One-use session ownership for planned direct-molecule structural deletion.

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentSession, DocumentSessionError, PersistentId,
    PreparedSessionTransitionV1, RevisionState, SessionOperationError, SessionOperationResultV1,
};
use crate::StructureDeletionReceiptV1;

/// A revision-bound structural deletion candidate with session-owned split IDs.
pub struct PendingDeleteStructureV1 {
    receipt: StructureDeletionReceiptV1,
    transition: PreparedSessionTransitionV1,
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
            .current_state_v1()
            .document()
            .prepare_delete_structure(molecule, atoms, bonds)
            .map_err(SessionOperationError::Candidate)?;
        let (later_ids, effects) =
            self.reserve_generated_ids_for_transition_v1(|ids, indexed| {
                ids.reserve_molecule_roots(indexed, plan.additional_molecule_count())
            })?;
        let (document, receipt) = self
            .current_state_v1()
            .document()
            .commit_delete_structure(&plan, &later_ids)
            .map_err(SessionOperationError::Candidate)?;
        let revision = self
            .current_state_v1()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate =
            RevisionState::from_document(revision, document).map_err(DocumentSessionError::Load)?;
        let transition = self.prepare_changed_session_transition_v1(
            expected_revision,
            self.current_digest_v1(),
            candidate,
            effects,
        )?;
        Ok(PendingDeleteStructureV1 {
            receipt,
            transition,
        })
    }

    /// Commit a prepared structural deletion exactly once.
    pub fn commit_delete_structure_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingDeleteStructureV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        self.commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(|refusal| map_transition_refusal(self, expected_revision, refusal))
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
