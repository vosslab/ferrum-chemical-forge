//! Revision-bound complete-molecule creation owned by the document session.

use super::{
    DetachedRegularRingInsertionV1, DocumentSession, DocumentSessionError, MoleculeInsertionV1,
    PendingCreateMolecule, PersistentId, RevisionState, SessionOperationError,
    SessionOperationResultV1, TypedDocument,
};

impl DocumentSession {
    pub(crate) fn prepare_create_molecule_v1(
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
    pub(crate) fn prepare_create_regular_ring_v1(
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
        let transition = self.prepare_changed_session_transition_v1(
            expected_revision,
            self.current_digest_v1(),
            candidate,
            effects,
        )?;
        Ok(PendingCreateMolecule {
            molecule_identifier: identities.molecule,
            atom_identifiers: identities.atoms,
            bond_identifiers: identities.bonds,
            transition,
        })
    }

    pub(crate) fn commit_create_molecule(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateMolecule,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        self.commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(|refusal| map_transition_refusal(self, expected_revision, refusal))
    }
}

fn map_transition_refusal(
    session: &DocumentSession,
    expected_revision: u64,
    refusal: super::AdmittedSessionTransitionRefusalV1,
) -> DocumentSessionError {
    match refusal {
        super::AdmittedSessionTransitionRefusalV1::ForeignSession => {
            DocumentSessionError::PreparedOperationForeignSession
        }
        super::AdmittedSessionTransitionRefusalV1::Replayed
        | super::AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            DocumentSessionError::PreparedOperationConsumed
        }
        super::AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            DocumentSessionError::RevisionConflict {
                expected: expected_revision,
                actual: session.current_revision_v1(),
            }
        }
        super::AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            DocumentSessionError::RendererAdmission
        }
        super::AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
            SessionOperationError::HistoryResourceExhausted.into()
        }
    }
}
