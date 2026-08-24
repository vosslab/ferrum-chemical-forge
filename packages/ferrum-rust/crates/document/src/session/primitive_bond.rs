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
        let (atom_id, effects) =
            self.reserve_generated_ids_for_transition_v1(|ids, indexed| ids.reserve_atom(indexed))?;
        let pending = self.prepare_create_atom_candidate(
            expected_revision,
            &molecule_id,
            atom_id,
            element,
            position,
            effects,
        )?;
        Ok(pending)
    }

    fn prepare_create_atom_candidate(
        &mut self,
        expected_revision: u64,
        molecule_id: &PersistentId,
        atom_id: PersistentId,
        element: &str,
        position: Point3V1,
        effects: SessionTransitionEffectsV1,
    ) -> Result<PendingCreateAtom, DocumentSessionError> {
        let candidate = self
            .current_state_v1()
            .document()
            .with_insert_atom(molecule_id, &atom_id, element, position)
            .map_err(SessionOperationError::Candidate)?;
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
        Ok(PendingCreateAtom {
            identifier: atom_id,
            transition,
        })
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

    /// Accept one prepared atom insertion exactly once.
    pub fn commit_create_atom(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateAtom,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        self.commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(|refusal| map_transition_refusal(self, expected_revision, refusal))
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
                presentation,
            )
            .map_err(SessionOperationError::Candidate)?;
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
        Ok(PendingCreateBond {
            identifier: bond_id,
            transition,
        })
    }

    /// Accept one prepared bond insertion exactly once.
    pub fn commit_create_bond(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateBond,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        self.commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(|refusal| map_transition_refusal(self, expected_revision, refusal))
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
        let (identities, effects) =
            self.reserve_generated_ids_for_transition_v1(|ids, indexed| {
                ids.reserve_bonded_atom(indexed)
            })?;
        let candidate = self
            .current_state_v1()
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
        Ok(PendingCreateBondedAtom {
            atom_identifier: identities.atom,
            bond_identifier: identities.bond,
            transition,
        })
    }

    /// Accept one prepared atom-plus-bond insertion exactly once.
    pub fn commit_create_bonded_atom(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateBondedAtom,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        self.commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(|refusal| map_transition_refusal(self, expected_revision, refusal))
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

    #[cfg(test)]
    pub(crate) fn set_next_generated_bond_sequence_for_test(&mut self, sequence: Option<u64>) {
        self.generated_ids = self.generated_ids.with_bond_sequence(sequence);
    }

    #[cfg(test)]
    pub(crate) fn provisional_token_facts_for_test(&self) -> (u64, usize, usize) {
        self.current_document_v1()
            .indexed()
            .provisional_token_facts_for_test()
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
