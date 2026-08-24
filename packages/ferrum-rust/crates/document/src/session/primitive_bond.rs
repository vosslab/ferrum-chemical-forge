use super::admitted_transition_v1::{ChangedTransitionCommitV1, SessionOperationOutcomeStagingV1};
use super::*;
use crate::{
    AuthoringCapabilityClaimV1,
    session_operation::{CreateAtomV1, CreateBondV1},
};

impl DocumentSession {
    pub(crate) fn prepare_create_atom_transition_v1(
        &mut self,
        expected_revision: u64,
        request: CreateAtomV1,
        authorization_claim: AuthoringCapabilityClaimV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let molecule_id = self.resolve_molecule_id(request.molecule())?;
        let (atom_id, effects) =
            self.reserve_generated_ids_for_transition_v1(|ids, indexed| ids.reserve_atom(indexed))?;
        let candidate = self
            .current_state_v1()
            .document()
            .with_insert_atom(
                &molecule_id,
                &atom_id,
                request.element(),
                request.position(),
            )
            .map_err(SessionOperationError::Candidate)?;
        let revision = self
            .current_state_v1()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        self.prepare_changed_session_transition_with_commit_v1(
            expected_revision,
            self.current_digest_v1(),
            candidate,
            effects,
            ChangedTransitionCommitV1::Append,
            SessionOperationOutcomeStagingV1::AtomCreatedV1(atom_id),
            Some(authorization_claim),
        )
    }

    fn resolve_molecule_id(
        &self,
        molecule_object_id: &DocumentObjectIdV1,
    ) -> Result<PersistentId, SessionOperationError> {
        let object_id = molecule_object_id.as_str().to_owned();
        let record = self
            .current_state_v1()
            .document()
            .resolve_document_object_id(molecule_object_id)
            .ok_or_else(|| SessionOperationError::UnknownDocumentObject(object_id.clone()))?;
        if record.class() != TypedClass::Molecule {
            return Err(SessionOperationError::InvalidCreateAtomTarget(object_id));
        }
        let source_id = record
            .attribute("id")
            .ok_or_else(|| SessionOperationError::InvalidCreateAtomTarget(object_id.clone()))?;
        PersistentId::new(source_id.to_owned())
            .map_err(|_| SessionOperationError::InvalidCreateAtomTarget(object_id))
    }

    #[cfg(test)]
    pub(crate) fn set_next_generated_atom_sequence_for_test(&mut self, sequence: Option<u64>) {
        self.generated_ids = self.generated_ids.with_atom_sequence(sequence);
    }

    pub(crate) fn prepare_create_bond_transition_v1(
        &mut self,
        expected_revision: u64,
        request: CreateBondV1,
        authorization_claim: AuthoringCapabilityClaimV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if request.start_atom() == request.end_atom() {
            return Err(SessionOperationError::CreateBondSelfLoop(
                request.start_atom().as_str().to_owned(),
            )
            .into());
        }
        let (start_molecule, start_atom) = self.resolve_bond_atom(request.start_atom())?;
        let (end_molecule, end_atom) = self.resolve_bond_atom(request.end_atom())?;
        if start_molecule != end_molecule {
            return Err(SessionOperationError::CreateBondAcrossMolecules.into());
        }
        self.reject_existing_bond(&start_molecule, &start_atom, &end_atom)?;
        let (bond_id, effects) =
            self.reserve_generated_ids_for_transition_v1(|ids, indexed| ids.reserve_bond(indexed))?;
        let candidate = self
            .current_state_v1()
            .document()
            .with_insert_bond(
                &start_molecule,
                &bond_id,
                &start_atom,
                &end_atom,
                request.presentation(),
            )
            .map_err(SessionOperationError::Candidate)?;
        let revision = self
            .current_state_v1()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        self.prepare_changed_session_transition_with_commit_v1(
            expected_revision,
            self.current_digest_v1(),
            candidate,
            effects,
            ChangedTransitionCommitV1::Append,
            SessionOperationOutcomeStagingV1::BondCreatedV1(bond_id),
            Some(authorization_claim),
        )
    }

    pub(super) fn resolve_bond_atom(
        &self,
        object_id: &DocumentObjectIdV1,
    ) -> Result<(PersistentId, PersistentId), SessionOperationError> {
        let object_key = object_id.as_str().to_owned();
        let document = self.current_document_v1();
        let target = document
            .resolve_document_object_id(object_id)
            .ok_or_else(|| SessionOperationError::UnknownDocumentObject(object_key.clone()))?;
        if target.class() != TypedClass::Atom {
            return Err(SessionOperationError::InvalidCreateBondTarget(object_key));
        }
        let atom_id = target
            .attribute("id")
            .and_then(|value| PersistentId::new(value.to_owned()).ok())
            .ok_or_else(|| SessionOperationError::InvalidCreateBondTarget(object_key.clone()))?;
        for molecule_child in document.root().typed_children() {
            let molecule = molecule_child.record();
            if molecule.class() != TypedClass::Molecule {
                continue;
            }
            let contains_target = molecule.typed_children().iter().any(|child| {
                child.record().path() == target.path() && child.record().class() == TypedClass::Atom
            });
            if !contains_target {
                continue;
            }
            let molecule_id = molecule
                .attribute("id")
                .and_then(|value| PersistentId::new(value.to_owned()).ok())
                .ok_or(SessionOperationError::InvalidCreateBondTarget(object_key))?;
            return Ok((molecule_id, atom_id));
        }
        Err(SessionOperationError::InvalidCreateBondTarget(object_key))
    }

    pub(super) fn reject_existing_bond(
        &self,
        molecule_id: &PersistentId,
        start_atom_id: &PersistentId,
        end_atom_id: &PersistentId,
    ) -> Result<(), SessionOperationError> {
        let document = self.current_document_v1();
        let molecule = document
            .root()
            .children_of(TypedClass::Molecule)
            .find(|record| record.attribute("id") == Some(molecule_id.as_str()))
            .ok_or_else(|| {
                SessionOperationError::InvalidCreateBondTarget(molecule_id.to_string())
            })?;
        let duplicate = molecule.children_of(TypedClass::Bond).any(|bond| {
            let start = bond.attribute("start");
            let end = bond.attribute("end");
            (start == Some(start_atom_id.as_str()) && end == Some(end_atom_id.as_str()))
                || (start == Some(end_atom_id.as_str()) && end == Some(start_atom_id.as_str()))
        });
        if duplicate {
            return Err(SessionOperationError::CreateBondDuplicate {
                start: start_atom_id.as_str().to_owned(),
                end: end_atom_id.as_str().to_owned(),
            });
        }
        Ok(())
    }
}
