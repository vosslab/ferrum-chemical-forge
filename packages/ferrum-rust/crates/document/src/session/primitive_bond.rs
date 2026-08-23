use super::*;

impl DocumentSession {
    pub fn prepare_create_atom_v1(
        &mut self,
        expected_revision: u64,
        molecule_object_id: &DocumentObjectIdV1,
        element: &str,
        position: Point3V1,
    ) -> Result<PendingCreateAtom, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let molecule_id = self.resolve_molecule_id(molecule_object_id)?;
        let (atom_id, generated_ids) = self
            .generated_ids
            .reserve_atom(self.history.current().document().indexed())?;
        let pending = self.prepare_create_atom_candidate(
            expected_revision,
            &molecule_id,
            atom_id,
            element,
            position,
        )?;
        self.generated_ids = generated_ids;
        Ok(pending)
    }

    fn prepare_create_atom_candidate(
        &mut self,
        expected_revision: u64,
        molecule_id: &PersistentId,
        atom_id: PersistentId,
        element: &str,
        position: Point3V1,
    ) -> Result<PendingCreateAtom, DocumentSessionError> {
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_atom(molecule_id, &atom_id, element, position)
            .map_err(SessionOperationError::Candidate)?;
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
        Ok(PendingCreateAtom {
            revision: expected_revision,
            token,
            identifier: atom_id,
            candidate: Some(candidate),
        })
    }

    fn resolve_molecule_id(
        &self,
        molecule_object_id: &DocumentObjectIdV1,
    ) -> Result<PersistentId, SessionOperationError> {
        let object_id = molecule_object_id.as_str().to_owned();
        let record = self
            .history
            .current()
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

    /// Accept one prepared atom insertion exactly once.
    pub fn commit_create_atom(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateAtom,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.commit_prepared_candidate(
            expected_revision,
            pending.revision,
            &pending.token,
            &mut pending.candidate,
        )
    }

    /// Prepare one molecule-local bond insertion at the current revision.
    ///
    /// Endpoint selectors must name two distinct durable atoms under the same
    /// durable molecule. The session allocates the bond identity and validates the
    /// complete detached candidate before issuing its document-local token.
    pub fn prepare_create_bond_v2(
        &mut self,
        expected_revision: u64,
        start_atom_object_id: &DocumentObjectIdV1,
        end_atom_object_id: &DocumentObjectIdV1,
        presentation: DocumentBondPresentationV1,
    ) -> Result<PendingCreateBond, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if start_atom_object_id == end_atom_object_id {
            return Err(SessionOperationError::CreateBondSelfLoop(
                start_atom_object_id.as_str().to_owned(),
            )
            .into());
        }
        let (start_molecule, start_atom) = self.resolve_bond_atom(start_atom_object_id)?;
        let (end_molecule, end_atom) = self.resolve_bond_atom(end_atom_object_id)?;
        if start_molecule != end_molecule {
            return Err(SessionOperationError::CreateBondAcrossMolecules.into());
        }
        self.reject_existing_bond(&start_molecule, &start_atom, &end_atom)?;
        let (bond_id, generated_ids) = self
            .generated_ids
            .reserve_bond(self.history.current().document().indexed())?;
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_bond(
                &start_molecule,
                &bond_id,
                &start_atom,
                &end_atom,
                presentation,
            )
            .map_err(SessionOperationError::Candidate)?;
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
        self.generated_ids = generated_ids;
        Ok(PendingCreateBond {
            revision: expected_revision,
            token,
            identifier: bond_id,
            candidate: Some(candidate),
        })
    }

    /// Accept one prepared bond insertion exactly once.
    pub fn commit_create_bond(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateBond,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.commit_prepared_candidate(
            expected_revision,
            pending.revision,
            &pending.token,
            &mut pending.candidate,
        )
    }

    /// Prepare one atom and its bond to an existing durable atom as one edit.
    ///
    /// Rust resolves the start atom and its containing molecule, allocates both
    /// durable identities, and validates the complete projected candidate before
    /// issuing a one-use token. No intermediate free-standing atom can become
    /// visible or enter history.
    pub fn prepare_create_bonded_atom_v2(
        &mut self,
        expected_revision: u64,
        start_atom_object_id: &DocumentObjectIdV1,
        element: &str,
        position: Point3V1,
        presentation: DocumentBondPresentationV1,
    ) -> Result<PendingCreateBondedAtom, DocumentSessionError> {
        self.prepare_create_bonded_atom_oriented_v2(
            expected_revision,
            start_atom_object_id,
            element,
            position,
            presentation,
            false,
        )
    }

    /// Prepare one atom-plus-bond insertion with explicit authored endpoint order.
    ///
    /// A directed presentation uses this order as its CDML tip-to-base direction.
    /// Ordinary callers use `prepare_create_bonded_atom_v2`, which retains the
    /// established existing-atom-to-new-atom order.
    pub(crate) fn prepare_create_bonded_atom_oriented_v2(
        &mut self,
        expected_revision: u64,
        start_atom_object_id: &DocumentObjectIdV1,
        element: &str,
        position: Point3V1,
        presentation: DocumentBondPresentationV1,
        new_atom_is_start: bool,
    ) -> Result<PendingCreateBondedAtom, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let (molecule_id, start_atom_id) = self.resolve_bond_atom(start_atom_object_id)?;
        let (identities, generated_ids) = self
            .generated_ids
            .reserve_bonded_atom(self.history.current().document().indexed())?;
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_bonded_atom(
                &molecule_id,
                &start_atom_id,
                BondedAtomInsertion::new(
                    &identities.atom,
                    &identities.bond,
                    element,
                    position,
                    presentation,
                    new_atom_is_start,
                ),
            )
            .map_err(SessionOperationError::Candidate)?;
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
        self.generated_ids = generated_ids;
        Ok(PendingCreateBondedAtom {
            revision: expected_revision,
            token,
            atom_identifier: identities.atom,
            bond_identifier: identities.bond,
            candidate: Some(candidate),
        })
    }

    /// Accept one prepared atom-plus-bond insertion exactly once.
    pub fn commit_create_bonded_atom(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateBondedAtom,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.commit_prepared_candidate(
            expected_revision,
            pending.revision,
            &pending.token,
            &mut pending.candidate,
        )
    }

    pub(super) fn resolve_bond_atom(
        &self,
        object_id: &DocumentObjectIdV1,
    ) -> Result<(PersistentId, PersistentId), SessionOperationError> {
        let object_key = object_id.as_str().to_owned();
        let document = self.history.current().document();
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
        let document = self.history.current().document();
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

    #[cfg(test)]
    pub(crate) fn set_next_generated_bond_sequence_for_test(&mut self, sequence: Option<u64>) {
        self.generated_ids = self.generated_ids.with_bond_sequence(sequence);
    }

    #[cfg(test)]
    pub(crate) fn provisional_token_facts_for_test(&self) -> (u64, usize, usize) {
        self.history
            .current()
            .document()
            .indexed()
            .provisional_token_facts_for_test()
    }

    pub(super) fn commit_prepared_candidate(
        &mut self,
        expected_revision: u64,
        prepared_revision: u64,
        token: &ProvisionalToken,
        candidate: &mut Option<RevisionState>,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if candidate.is_none() {
            return Err(DocumentSessionError::PreparedOperationConsumed);
        }
        if prepared_revision != expected_revision {
            return Err(DocumentSessionError::RevisionConflict {
                expected: prepared_revision,
                actual: expected_revision,
            });
        }
        self.history
            .current()
            .document()
            .verify_provisional_token(token)
            .map_err(prepared::map_prepared_token_error)?;
        self.history
            .current_mut()
            .document_mut()
            .consume_provisional_token(token)
            .map_err(SessionOperationError::Candidate)?;
        let state = candidate
            .take()
            .expect("the candidate presence check established this invariant");
        self.history.append(state);
        self.operation_result()
    }
}
