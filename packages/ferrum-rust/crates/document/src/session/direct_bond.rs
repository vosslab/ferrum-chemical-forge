use super::*;
use crate::direct_bond_mutation::DirectBondMutationCandidate;

fn same_point(first: DirectBondPoint2V1, second: DirectBondPoint2V1) -> bool {
    first.x() == second.x() && first.y() == second.y()
}

fn snap_point(
    start: DirectBondPoint2V1,
    raw: DirectBondPoint2V1,
    policy: DirectBondSnapPolicyV1,
) -> Result<DirectBondPoint2V1, DirectBondGestureErrorV1> {
    let mut dx = raw.x() - start.x();
    let mut dy = raw.y() - start.y();
    let mut length = dx.hypot(dy);
    if let Some(increment) = policy.angle_increment_degrees()
        && length > 0.0
    {
        let step = f64::from(increment).to_radians();
        let angle = dy.atan2(dx);
        let snapped = (angle / step).round() * step;
        dx = length * snapped.cos();
        dy = length * snapped.sin();
    }
    if let Some(fixed) = policy.fixed_length_pt() {
        if length == 0.0 {
            return Err(DirectBondGestureErrorV1::CollapsedEndpoint);
        }
        length = fixed;
        let scale = length / dx.hypot(dy);
        dx *= scale;
        dy *= scale;
    }
    if policy.hex_grid() {
        const GRID: f64 = 10.0;
        dx = (dx / GRID).round() * GRID;
        dy = (dy / GRID).round() * GRID;
    }
    DirectBondPoint2V1::new(start.x() + dx, start.y() + dy)
}

fn map_direct_bond_commit_error(error: DocumentSessionError) -> DirectBondCommitErrorV1 {
    match error {
        DocumentSessionError::RevisionConflict { .. } => DirectBondCommitErrorV1::StaleRevision,
        DocumentSessionError::RevisionExhausted => DirectBondCommitErrorV1::RevisionExhausted,
        DocumentSessionError::Operation(
            SessionOperationError::AtomIdentifierExhausted
            | SessionOperationError::BondIdentifierExhausted
            | SessionOperationError::GeneratedIdentifierAllocationFailed,
        ) => DirectBondCommitErrorV1::IdentityAllocationFailed,
        DocumentSessionError::Operation(SessionOperationError::HistoryResourceExhausted) => {
            DirectBondCommitErrorV1::ProvisionalTokenUnavailable
        }
        _ => DirectBondCommitErrorV1::CandidateApplicationFailed,
    }
}

/// A one-use, revision-bound prepared atom insertion.
///
/// The token is intentionally opaque. It originates from the exact current
/// document, can be committed only at its prepared revision, and is consumed only
/// after the fully validated candidate is accepted.
impl DocumentSession {
    /// Materialize one native, noninteractive direct-bond candidate without mutation.
    ///
    /// The caller supplies explicitly resolved durable atom IDs or finite
    /// new-atom points. Rust validates them against this session's fenced
    /// document and owns chemistry, identity allocation, history, and durable
    /// CDML mutation. This is not a pointer gesture API and is not exposed to
    /// Qt or PyO3.
    pub fn materialize_direct_bond_mutation(
        &self,
        fence: DocumentFenceV1,
        start: DirectBondEndpointIntent,
        end: DirectBondEndpointIntent,
        presentation: DocumentBondPresentationV1,
        new_atom_element: String,
        snap: DirectBondSnapPolicyV1,
    ) -> Result<DirectBondMutationCandidate, DirectBondAdmissionRefusalV1> {
        let gesture = self
            .begin_direct_bond_mutation(fence, start, presentation, new_atom_element, snap)
            .map_err(|error| match error {
                DirectBondGestureErrorV1::StaleRevision => {
                    DirectBondAdmissionRefusalV1::StaleRevision
                }
                DirectBondGestureErrorV1::StaleDigest => DirectBondAdmissionRefusalV1::StaleDigest,
                _ => DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission,
            })?;
        self.materialize_direct_bond_mutation_internal(&gesture, end)
    }

    fn materialize_direct_bond_mutation_internal(
        &self,
        gesture: &DirectBondGestureV2,
        end: DirectBondEndpointIntent,
    ) -> Result<DirectBondMutationCandidate, DirectBondAdmissionRefusalV1> {
        let _admission = self.admit_direct_bond_candidate_v2(gesture, end.clone())?;
        let source = self
            .snapshot()
            .map_err(|_| DirectBondAdmissionRefusalV1::StaleRevision)?
            .cdml()
            .to_owned();
        let mut candidate_session = DocumentSession::load(&source)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        let candidate_snapshot = candidate_session
            .snapshot()
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        let candidate_gesture = candidate_session
            .begin_direct_bond_mutation(
                DocumentFenceV1::new(candidate_snapshot.revision(), *candidate_snapshot.digest()),
                gesture.start.clone(),
                gesture.presentation,
                gesture.new_atom_element.clone(),
                gesture.snap,
            )
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        let candidate_admission = candidate_session
            .admit_direct_bond_candidate_v2(&candidate_gesture, end)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        let committed = candidate_session
            .commit_direct_bond_admission_v2(&candidate_admission)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        let candidate_snapshot = candidate_session
            .snapshot()
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        Ok(DirectBondMutationCandidate::new(
            self.authoring_capability_issuer.issue(),
            gesture.fence,
            _admission.overlay().start(),
            _admission.overlay().end(),
            candidate_snapshot.cdml().to_owned(),
            *candidate_snapshot.digest(),
            committed.bond().clone(),
            committed.end_atom().clone(),
            committed.second_created_atom().cloned(),
            committed.created_new_atom(),
            committed.created_new_molecule(),
        ))
    }

    pub fn commit_direct_bond_mutation(
        &mut self,
        candidate: &DirectBondMutationCandidate,
    ) -> Result<SessionOperationResultV1, DirectBondCommitErrorV1> {
        let claim = candidate
            .capability
            .claim_for_commit(&self.authoring_capability_issuer)
            .map_err(|error| match error {
                crate::AuthoringCapabilityAccessErrorV1::ForeignSession => {
                    DirectBondCommitErrorV1::ForeignSession
                }
                crate::AuthoringCapabilityAccessErrorV1::Replayed => {
                    DirectBondCommitErrorV1::ReplayedReceipt
                }
            })?;
        let verified = DocumentSession::load(candidate.candidate_cdml_for_render_bridge())
            .map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?;
        if *verified
            .snapshot()
            .map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?
            .digest()
            != candidate.candidate_digest_for_render_bridge()
        {
            return Err(DirectBondCommitErrorV1::CandidateApplicationFailed);
        }
        let result = self
            .commit_complete_cdml_transaction_v1(
                candidate.source_fence_for_render_bridge(),
                candidate.candidate_cdml_for_render_bridge(),
            )
            .map_err(map_direct_bond_commit_error)?;
        claim.consume();
        Ok(result)
    }

    fn begin_direct_bond_mutation(
        &self,
        fence: DocumentFenceV1,
        start: DirectBondEndpointIntent,
        presentation: DocumentBondPresentationV1,
        new_atom_element: String,
        snap: DirectBondSnapPolicyV1,
    ) -> Result<DirectBondGestureV2, DirectBondGestureErrorV1> {
        self.require_fence(fence)?;
        Ok(DirectBondGestureV2 {
            fence,
            start,
            presentation,
            new_atom_element,
            snap,
        })
    }

    fn admit_direct_bond_candidate_v2(
        &self,
        gesture: &DirectBondGestureV2,
        end: DirectBondEndpointIntent,
    ) -> Result<DirectBondAdmissionV2, DirectBondAdmissionRefusalV1> {
        self.require_direct_bond_admission_fence(gesture.fence)?;
        let endpoint_point = |intent: &DirectBondEndpointIntent| -> Result<DirectBondPoint2V1, DirectBondAdmissionRefusalV1> { match intent { DirectBondEndpointIntent::ExistingAtom { atom } => self.direct_atom_point(atom).ok_or(DirectBondAdmissionRefusalV1::UnknownEndAtom), DirectBondEndpointIntent::NewAtomAt { raw_point } => Ok(*raw_point) } };
        let raw_start = endpoint_point(&gesture.start)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnknownStartAtom)?;
        let raw_end = endpoint_point(&end)?;
        if matches!(
            (&gesture.start, &end),
            (
                DirectBondEndpointIntent::ExistingAtom { atom: start },
                DirectBondEndpointIntent::ExistingAtom { atom: finish },
            ) if start == finish
        ) {
            return Err(DirectBondAdmissionRefusalV1::SelfLoop);
        }
        let (start_point, end_point) = match (&gesture.start, &end) {
            (
                DirectBondEndpointIntent::ExistingAtom { .. },
                DirectBondEndpointIntent::NewAtomAt { raw_point },
            ) => (
                raw_start,
                snap_point(raw_start, *raw_point, gesture.snap)
                    .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?,
            ),
            (
                DirectBondEndpointIntent::NewAtomAt { raw_point },
                DirectBondEndpointIntent::ExistingAtom { .. },
            ) => (
                snap_point(raw_end, *raw_point, gesture.snap)
                    .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?,
                raw_end,
            ),
            (
                DirectBondEndpointIntent::NewAtomAt { .. },
                DirectBondEndpointIntent::NewAtomAt { raw_point },
            ) => (
                raw_start,
                snap_point(raw_start, *raw_point, gesture.snap)
                    .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?,
            ),
            _ => (raw_start, raw_end),
        };
        if same_point(start_point, end_point) {
            return Err(DirectBondAdmissionRefusalV1::CollapsedEndpoint);
        }
        let candidate = match (&gesture.start, &end) {
            (
                DirectBondEndpointIntent::ExistingAtom { atom: start },
                DirectBondEndpointIntent::ExistingAtom { atom: finish },
            ) => {
                let (molecule, start_id) = self
                    .resolve_bond_atom(start)
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnknownStartAtom)?;
                let (end_molecule, end_id) = self
                    .resolve_bond_atom(finish)
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnknownEndAtom)?;
                if molecule != end_molecule {
                    return Err(DirectBondAdmissionRefusalV1::CrossMolecule);
                }
                self.reject_existing_bond_for_object_ids(start, finish)
                    .map_err(|_| DirectBondAdmissionRefusalV1::DuplicateBond)?;
                self.admit_direct_bond_existing_chemistry(
                    &molecule,
                    &start_id,
                    &end_id,
                    gesture.presentation,
                )?;
                DirectBondAdmittedCandidateV2::ExistingExisting {
                    start: start.clone(),
                    end: finish.clone(),
                    presentation: gesture.presentation,
                }
            }
            (
                DirectBondEndpointIntent::ExistingAtom { atom },
                DirectBondEndpointIntent::NewAtomAt { .. },
            ) => {
                let (molecule, existing_id) = self
                    .resolve_bond_atom(atom)
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnknownStartAtom)?;
                self.admit_direct_bond_new_chemistry(
                    &molecule,
                    &existing_id,
                    &gesture.new_atom_element,
                    end_point,
                    gesture.presentation,
                    false,
                )?;
                DirectBondAdmittedCandidateV2::ExistingNew {
                    existing: atom.clone(),
                    new_point: end_point,
                    element: gesture.new_atom_element.clone(),
                    presentation: gesture.presentation,
                    new_atom_is_start: false,
                }
            }
            (
                DirectBondEndpointIntent::NewAtomAt { .. },
                DirectBondEndpointIntent::ExistingAtom { atom },
            ) => {
                let (molecule, existing_id) = self
                    .resolve_bond_atom(atom)
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnknownEndAtom)?;
                self.admit_direct_bond_new_chemistry(
                    &molecule,
                    &existing_id,
                    &gesture.new_atom_element,
                    start_point,
                    gesture.presentation,
                    true,
                )?;
                DirectBondAdmittedCandidateV2::NewExisting {
                    new_point: start_point,
                    existing: atom.clone(),
                    element: gesture.new_atom_element.clone(),
                    presentation: gesture.presentation,
                    new_atom_is_start: true,
                }
            }
            (
                DirectBondEndpointIntent::NewAtomAt { .. },
                DirectBondEndpointIntent::NewAtomAt { .. },
            ) => {
                self.admit_direct_bond_new_molecule_chemistry(
                    &gesture.new_atom_element,
                    start_point,
                    end_point,
                    gesture.presentation,
                )?;
                DirectBondAdmittedCandidateV2::NewNew {
                    start: start_point,
                    end: end_point,
                    element: gesture.new_atom_element.clone(),
                    presentation: gesture.presentation,
                }
            }
        };
        Ok(direct_bond_mutation::admission(
            gesture,
            candidate,
            start_point,
            end_point,
        ))
    }

    fn commit_direct_bond_admission_v2(
        &mut self,
        admission: &DirectBondAdmissionV2,
    ) -> Result<CommittedDirectBondGestureV2, DirectBondCommitErrorV1> {
        self.require_direct_bond_commit_fence(admission.fence)?;
        match &admission.candidate {
            DirectBondAdmittedCandidateV2::ExistingExisting {
                start,
                end,
                presentation,
            } => {
                let (_, end_id) = self
                    .resolve_bond_atom(end)
                    .map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?;
                let mut pending = self
                    .prepare_create_bond_v2(admission.fence.revision(), start, end, *presentation)
                    .map_err(map_direct_bond_commit_error)?;
                let bond = pending.identifier().clone();
                let result = self
                    .commit_create_bond(admission.fence.revision(), &mut pending)
                    .map_err(map_direct_bond_commit_error)?;
                Ok(CommittedDirectBondGestureV2::new(
                    bond, end_id, None, false, false, result,
                ))
            }
            DirectBondAdmittedCandidateV2::ExistingNew {
                existing,
                new_point,
                element,
                presentation,
                new_atom_is_start,
            } => {
                let position = Point3V1::new(new_point.x(), new_point.y(), 0.0)
                    .map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?;
                let mut pending = self
                    .prepare_create_bonded_atom_oriented_v2(
                        admission.fence.revision(),
                        existing,
                        element,
                        position,
                        *presentation,
                        *new_atom_is_start,
                    )
                    .map_err(map_direct_bond_commit_error)?;
                let atom = pending.atom_identifier().clone();
                let bond = pending.bond_identifier().clone();
                let result = self
                    .commit_create_bonded_atom(admission.fence.revision(), &mut pending)
                    .map_err(map_direct_bond_commit_error)?;
                Ok(CommittedDirectBondGestureV2::new(
                    bond, atom, None, true, false, result,
                ))
            }
            DirectBondAdmittedCandidateV2::NewExisting {
                new_point,
                existing,
                element,
                presentation,
                new_atom_is_start,
            } => {
                let (_, end_id) = self
                    .resolve_bond_atom(existing)
                    .map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?;
                let position = Point3V1::new(new_point.x(), new_point.y(), 0.0)
                    .map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?;
                let mut pending = self
                    .prepare_create_bonded_atom_oriented_v2(
                        admission.fence.revision(),
                        existing,
                        element,
                        position,
                        *presentation,
                        *new_atom_is_start,
                    )
                    .map_err(map_direct_bond_commit_error)?;
                let bond = pending.bond_identifier().clone();
                let result = self
                    .commit_create_bonded_atom(admission.fence.revision(), &mut pending)
                    .map_err(map_direct_bond_commit_error)?;
                Ok(CommittedDirectBondGestureV2::new(
                    bond, end_id, None, true, false, result,
                ))
            }
            DirectBondAdmittedCandidateV2::NewNew {
                start,
                end,
                element,
                presentation,
            } => {
                let make_atom = |point: DirectBondPoint2V1| -> Result<MoleculeInsertionAtomV1, DirectBondCommitErrorV1> { MoleculeInsertionAtomV1::new(element, Point3V1::new(point.x(), point.y(), 0.0).map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?, None, None, None).map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed) };
                let molecule = MoleculeInsertionV1::new(
                    vec![make_atom(*start)?, make_atom(*end)?],
                    vec![MoleculeInsertionBondV1::new_with_presentation(
                        0,
                        1,
                        *presentation,
                    )],
                )
                .map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?;
                let mut pending = self
                    .prepare_create_molecule_v1(admission.fence.revision(), &molecule)
                    .map_err(map_direct_bond_commit_error)?;
                let atoms = pending.atom_identifiers().to_vec();
                let bond = pending.bond_identifiers()[0].clone();
                let result = self
                    .commit_create_molecule(admission.fence.revision(), &mut pending)
                    .map_err(map_direct_bond_commit_error)?;
                Ok(CommittedDirectBondGestureV2::new(
                    bond,
                    atoms[1].clone(),
                    Some(atoms[0].clone()),
                    true,
                    true,
                    result,
                ))
            }
        }
    }
    fn require_direct_bond_admission_fence(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<(), DirectBondAdmissionRefusalV1> {
        if self.history.current().revision() != fence.revision() {
            return Err(DirectBondAdmissionRefusalV1::StaleRevision);
        }
        if *self.history.current().digest() != fence.digest() {
            return Err(DirectBondAdmissionRefusalV1::StaleDigest);
        }
        Ok(())
    }

    fn require_direct_bond_commit_fence(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<(), DirectBondCommitErrorV1> {
        if self.history.current().revision() != fence.revision() {
            return Err(DirectBondCommitErrorV1::StaleRevision);
        }
        if *self.history.current().digest() != fence.digest() {
            return Err(DirectBondCommitErrorV1::StaleDigest);
        }
        Ok(())
    }

    fn admit_direct_bond_existing_chemistry(
        &self,
        molecule_id: &PersistentId,
        start_atom_id: &PersistentId,
        end_atom_id: &PersistentId,
        presentation: DocumentBondPresentationV1,
    ) -> Result<(), DirectBondAdmissionRefusalV1> {
        let bond_id = PersistentId::new("ferrum-direct-bond-admission-bond")
            .expect("static direct-bond admission identifier is nonblank");
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_bond(
                molecule_id,
                &bond_id,
                start_atom_id,
                end_atom_id,
                presentation,
            )
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        self.require_admitted_direct_bond_capacity(&candidate, molecule_id)
    }

    fn admit_direct_bond_new_chemistry(
        &self,
        molecule_id: &PersistentId,
        start_atom_id: &PersistentId,
        element: &str,
        point: DirectBondPoint2V1,
        presentation: DocumentBondPresentationV1,
        new_atom_is_start: bool,
    ) -> Result<(), DirectBondAdmissionRefusalV1> {
        let atom_id = PersistentId::new("ferrum-direct-bond-admission-atom")
            .expect("static direct-bond admission identifier is nonblank");
        let bond_id = PersistentId::new("ferrum-direct-bond-admission-bond")
            .expect("static direct-bond admission identifier is nonblank");
        let position = Point3V1::new(point.x(), point.y(), 0.0)
            .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?;
        let insertion = BondedAtomInsertion::new(
            &atom_id,
            &bond_id,
            element,
            position,
            presentation,
            new_atom_is_start,
        );
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_bonded_atom(molecule_id, start_atom_id, insertion)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        self.require_admitted_direct_bond_capacity(&candidate, molecule_id)
    }

    fn admit_direct_bond_new_molecule_chemistry(
        &self,
        element: &str,
        start: DirectBondPoint2V1,
        end: DirectBondPoint2V1,
        presentation: DocumentBondPresentationV1,
    ) -> Result<(), DirectBondAdmissionRefusalV1> {
        let molecule_id = PersistentId::new("ferrum-direct-bond-admission-molecule")
            .expect("static direct-bond admission identifier is nonblank");
        let atom_ids = [
            PersistentId::new("ferrum-direct-bond-admission-start-atom")
                .expect("static direct-bond admission identifier is nonblank"),
            PersistentId::new("ferrum-direct-bond-admission-end-atom")
                .expect("static direct-bond admission identifier is nonblank"),
        ];
        let bond_ids = [PersistentId::new("ferrum-direct-bond-admission-bond")
            .expect("static direct-bond admission identifier is nonblank")];
        let make_atom = |point: DirectBondPoint2V1| -> Result<MoleculeInsertionAtomV1, DirectBondAdmissionRefusalV1> { MoleculeInsertionAtomV1::new(element, Point3V1::new(point.x(), point.y(), 0.0).map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?, None, None, None).map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission) };
        let molecule = MoleculeInsertionV1::new(
            vec![make_atom(start)?, make_atom(end)?],
            vec![MoleculeInsertionBondV1::new_with_presentation(
                0,
                1,
                presentation,
            )],
        )
        .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_molecule(&molecule_id, &atom_ids, &bond_ids, &molecule)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        self.require_admitted_direct_bond_capacity(&candidate, &molecule_id)
    }

    fn require_admitted_direct_bond_capacity(
        &self,
        candidate: &TypedDocument,
        molecule_id: &PersistentId,
    ) -> Result<(), DirectBondAdmissionRefusalV1> {
        let molecule_object =
            DocumentObjectIdV1::from_class_source("cdml/molecule", molecule_id.as_str());
        let molecule = candidate
            .core_molecule(&molecule_object)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?
            .ok_or(DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        match crate::chemistry::evaluate_document_molecule_neutral_capacity_v1(&molecule) {
            Ok(DocumentBondCapacityOutcomeV1::WithinCapacity { .. }) => Ok(()),
            Ok(DocumentBondCapacityOutcomeV1::ExceedsCapacity { .. }) => {
                Err(DirectBondAdmissionRefusalV1::ExceedsChemistryCapacity)
            }
            Ok(DocumentBondCapacityOutcomeV1::NotChecked { .. }) | Err(_) => {
                Err(DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)
            }
        }
    }
}

#[cfg(test)]
mod direct_bond_v2_tests {
    use super::*;
    use crate::DocumentBondOrderV1;

    const BLANK_SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"/>";

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    fn point(x: f64, y: f64) -> DirectBondEndpointIntent {
        DirectBondEndpointIntent::NewAtomAt {
            raw_point: DirectBondPoint2V1::new(x, y).expect("finite point"),
        }
    }

    #[test]
    fn v2_blank_new_new_commit_has_one_undoable_history_transition() {
        let mut session = DocumentSession::load(BLANK_SOURCE).expect("blank session loads");
        let blank_snapshot = session.snapshot().expect("blank snapshot");
        assert!(!session.can_undo());
        assert!(!session.can_redo());

        let gesture = session
            .begin_direct_bond_mutation(
                fence(&session),
                point(0.0, 0.0),
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("gesture begins");
        let admission = session
            .admit_direct_bond_candidate_v2(&gesture, point(20.0, 0.0))
            .expect("blank candidate admits");
        session
            .commit_direct_bond_admission_v2(&admission)
            .expect("blank candidate commits");
        let committed_snapshot = session.snapshot().expect("committed snapshot");
        assert_ne!(committed_snapshot.cdml(), blank_snapshot.cdml());
        assert!(session.can_undo());
        assert!(!session.can_redo());

        session
            .undo(committed_snapshot.revision())
            .expect("commit undoes");
        assert_eq!(
            session.snapshot().expect("undone snapshot").cdml(),
            blank_snapshot.cdml()
        );
        assert!(!session.can_undo());
        assert!(session.can_redo());

        let undone_snapshot = session.snapshot().expect("undone snapshot");
        session
            .redo(undone_snapshot.revision())
            .expect("commit redoes");
        assert_eq!(
            session.snapshot().expect("redone snapshot").cdml(),
            committed_snapshot.cdml()
        );
        assert!(session.can_undo());
        assert!(!session.can_redo());
    }

    #[test]
    fn v2_rejected_blank_candidate_does_not_consume_commit_identity_or_history_state() {
        let mut rejected = DocumentSession::load(BLANK_SOURCE).expect("blank session loads");
        let snapshot = rejected.snapshot().expect("snapshot before refusal");
        let token_before = rejected.provisional_token_facts_for_test();
        let rejected_gesture = rejected
            .begin_direct_bond_mutation(
                fence(&rejected),
                point(0.0, 0.0),
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "Xx".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("gesture begins");
        assert_eq!(
            rejected.admit_direct_bond_candidate_v2(&rejected_gesture, point(20.0, 0.0)),
            Err(DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)
        );
        assert_eq!(
            rejected.snapshot().expect("snapshot after refusal"),
            snapshot
        );
        assert_eq!(rejected.provisional_token_facts_for_test(), token_before);

        let valid_gesture = rejected
            .begin_direct_bond_mutation(
                fence(&rejected),
                point(0.0, 0.0),
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("valid gesture begins");
        let valid_admission = rejected
            .admit_direct_bond_candidate_v2(&valid_gesture, point(20.0, 0.0))
            .expect("valid candidate admits");
        let rejected_receipt = rejected
            .commit_direct_bond_admission_v2(&valid_admission)
            .expect("valid candidate commits");

        let mut control = DocumentSession::load(BLANK_SOURCE).expect("control session loads");
        let control_gesture = control
            .begin_direct_bond_mutation(
                fence(&control),
                point(0.0, 0.0),
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("control gesture begins");
        let control_admission = control
            .admit_direct_bond_candidate_v2(&control_gesture, point(20.0, 0.0))
            .expect("control candidate admits");
        let control_receipt = control
            .commit_direct_bond_admission_v2(&control_admission)
            .expect("control candidate commits");
        assert_eq!(rejected_receipt.bond(), control_receipt.bond());
        assert_eq!(rejected_receipt.end_atom(), control_receipt.end_atom());
        assert_eq!(
            rejected_receipt.second_created_atom(),
            control_receipt.second_created_atom()
        );
        assert_eq!(rejected_receipt.result(), control_receipt.result());
    }

    #[test]
    fn v2_admitted_blank_candidate_drop_does_not_consume_commit_identity_or_history_state() {
        let mut dropped = DocumentSession::load(BLANK_SOURCE).expect("blank session loads");
        let snapshot = dropped.snapshot().expect("snapshot before admission");
        let token_before = dropped.provisional_token_facts_for_test();
        let dropped_gesture = dropped
            .begin_direct_bond_mutation(
                fence(&dropped),
                point(0.0, 0.0),
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("gesture begins");
        let dropped_admission = dropped
            .admit_direct_bond_candidate_v2(&dropped_gesture, point(20.0, 0.0))
            .expect("candidate admits");
        drop(dropped_admission);
        assert_eq!(dropped.snapshot().expect("snapshot after drop"), snapshot);
        assert_eq!(dropped.provisional_token_facts_for_test(), token_before);

        let valid_gesture = dropped
            .begin_direct_bond_mutation(
                fence(&dropped),
                point(0.0, 0.0),
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("valid gesture begins");
        let valid_admission = dropped
            .admit_direct_bond_candidate_v2(&valid_gesture, point(20.0, 0.0))
            .expect("valid candidate admits");
        let dropped_receipt = dropped
            .commit_direct_bond_admission_v2(&valid_admission)
            .expect("valid candidate commits");

        let mut control = DocumentSession::load(BLANK_SOURCE).expect("control session loads");
        let control_gesture = control
            .begin_direct_bond_mutation(
                fence(&control),
                point(0.0, 0.0),
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("control gesture begins");
        let control_admission = control
            .admit_direct_bond_candidate_v2(&control_gesture, point(20.0, 0.0))
            .expect("control candidate admits");
        let control_receipt = control
            .commit_direct_bond_admission_v2(&control_admission)
            .expect("control candidate commits");
        assert_eq!(dropped_receipt.bond(), control_receipt.bond());
        assert_eq!(dropped_receipt.end_atom(), control_receipt.end_atom());
        assert_eq!(
            dropped_receipt.second_created_atom(),
            control_receipt.second_created_atom()
        );
        assert_eq!(dropped_receipt.result(), control_receipt.result());
    }
}
