//! Revision-bound complete-molecule creation owned by the document session.

use super::{
    DocumentSession, DocumentSessionError, MoleculeInsertionV1, PersistentId,
    PreparedSessionTransitionV1, RevisionState, SessionOperationError, TypedDocument,
};

impl DocumentSession {
    pub(in crate::session) fn prepare_insert_molecule_transition_v1(
        &mut self,
        expected_revision: u64,
        molecule: &MoleculeInsertionV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.prepare_complete_molecule_candidate(
            expected_revision,
            molecule.atoms().len(),
            molecule.bonds().len(),
            |document, molecule_id, atom_ids, bond_ids| {
                document
                    .with_insert_molecule(molecule_id, atom_ids, bond_ids, molecule)
                    .map_err(SessionOperationError::Candidate)
            },
        )
    }

    fn prepare_complete_molecule_candidate<F>(
        &mut self,
        expected_revision: u64,
        atom_count: usize,
        bond_count: usize,
        writer: F,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError>
    where
        F: FnOnce(
            &TypedDocument,
            &PersistentId,
            &[PersistentId],
            &[PersistentId],
        ) -> Result<TypedDocument, SessionOperationError>,
    {
        self.require_current(expected_revision)?;
        let (identities, effects) =
            self.reserve_generated_ids_for_transition_v1(|ids, indexed| {
                ids.reserve_molecule(indexed, atom_count, bond_count)
            })?;
        let candidate = writer(
            self.current_document_v1(),
            &identities.molecule,
            &identities.atoms,
            &identities.bonds,
        )
        .map_err(DocumentSessionError::Operation)?;
        let revision = self
            .current_state_v1()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        self.prepare_changed_session_transition_with_molecule_insertion_outcome_v1(
            expected_revision,
            self.current_digest_v1(),
            candidate,
            effects,
            identities.molecule,
            identities.atoms,
            identities.bonds,
        )
    }
}
