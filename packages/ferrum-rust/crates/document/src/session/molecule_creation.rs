//! Revision-bound complete-molecule creation owned by the document session.

use super::{
    DetachedRegularRingInsertionV1, DocumentSession, DocumentSessionError, MoleculeInsertionV1,
    PendingCreateMolecule, PersistentId, RevisionState, SessionDocumentObservationV1,
    SessionOperationError, SessionOperationResultV1, TypedDocument, prepared,
};

impl DocumentSession {
    pub fn prepare_create_molecule_v1(
        &mut self,
        expected_revision: u64,
        molecule: &MoleculeInsertionV1,
    ) -> Result<PendingCreateMolecule, DocumentSessionError> {
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

    /// Prepare one complete detached regular ring. Rust owns its geometry and IDs.
    pub fn prepare_create_regular_ring_v1(
        &mut self,
        expected_revision: u64,
        request: DetachedRegularRingInsertionV1,
    ) -> Result<PendingCreateMolecule, DocumentSessionError> {
        let molecule = request.molecule().map_err(|error| {
            DocumentSessionError::Operation(SessionOperationError::InvalidRegularRingInsertion(
                error.to_string(),
            ))
        })?;
        self.prepare_create_molecule_v1(expected_revision, &molecule)
    }

    fn prepare_complete_molecule_candidate<F>(
        &mut self,
        expected_revision: u64,
        atom_count: usize,
        bond_count: usize,
        writer: F,
    ) -> Result<PendingCreateMolecule, DocumentSessionError>
    where
        F: FnOnce(
            &TypedDocument,
            &PersistentId,
            &[PersistentId],
            &[PersistentId],
        ) -> Result<TypedDocument, SessionOperationError>,
    {
        self.require_current(expected_revision)?;
        let (identities, generated_ids) = self.generated_ids.reserve_molecule(
            self.history.current().document().indexed(),
            atom_count,
            bond_count,
        )?;
        let candidate = writer(
            self.history.current().document(),
            &identities.molecule,
            &identities.atoms,
            &identities.bonds,
        )
        .map_err(DocumentSessionError::Operation)?;
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let candidate_snapshot = candidate.snapshot(!self.saved_baseline.is_current(&candidate));
        SessionDocumentObservationV1::from_state(candidate.document(), candidate_snapshot)
            .map_err(DocumentSessionError::Projection)?;
        let token = prepared::issue_prepared_token(self.history.current_mut().document_mut())?;
        Ok(PendingCreateMolecule {
            revision: expected_revision,
            token,
            molecule_identifier: identities.molecule,
            atom_identifiers: identities.atoms,
            bond_identifiers: identities.bonds,
            candidate: Some(candidate),
            tentative_generated_ids: generated_ids,
        })
    }

    pub fn commit_create_molecule(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateMolecule,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        let result = self.commit_prepared_candidate(
            expected_revision,
            pending.revision,
            &pending.token,
            &mut pending.candidate,
        )?;
        self.generated_ids = pending.tentative_generated_ids;
        Ok(result)
    }
}
