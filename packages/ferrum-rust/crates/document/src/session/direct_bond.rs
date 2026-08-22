use super::*;

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
    pub fn begin_direct_bond_gesture_v2(
        &self,
        fence: DocumentFenceV1,
        start: DirectBondEndpointIntentV2,
        presentation: DocumentBondPresentationV1,
        new_atom_element: String,
        snap: DirectBondSnapPolicyV1,
    ) -> Result<DirectBondGestureV2, DirectBondGestureErrorV1> {
        self.require_fence(fence)?;
        if !matches!(presentation, DocumentBondPresentationV1::Normal(_)) {
            return Err(DirectBondGestureErrorV1::UnsupportedPresentation);
        }
        Ok(DirectBondGestureV2 {
            capability: self.direct_bond_origin.issue_gesture(),
            fence,
            start,
            presentation,
            new_atom_element,
            snap,
        })
    }

    pub fn admit_direct_bond_candidate_v2(
        &self,
        gesture: &DirectBondGestureV2,
        end: DirectBondEndpointIntentV2,
    ) -> Result<DirectBondAdmissionV2, DirectBondAdmissionRefusalV1> {
        if !gesture.capability.belongs_to(self.direct_bond_origin) {
            return Err(DirectBondAdmissionRefusalV1::ForeignSession);
        }
        self.require_direct_bond_admission_fence(gesture.fence)?;
        if !matches!(gesture.presentation, DocumentBondPresentationV1::Normal(_)) {
            return Err(DirectBondAdmissionRefusalV1::UnsupportedPresentation);
        }
        let endpoint_point = |intent: &DirectBondEndpointIntentV2| -> Result<DirectBondPoint2V1, DirectBondAdmissionRefusalV1> { match intent { DirectBondEndpointIntentV2::ExistingAtom { atom } => self.direct_atom_point(atom).ok_or(DirectBondAdmissionRefusalV1::UnknownEndAtom), DirectBondEndpointIntentV2::NewAtomAt { raw_point } => Ok(*raw_point) } };
        let raw_start = endpoint_point(&gesture.start)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnknownStartAtom)?;
        let raw_end = endpoint_point(&end)?;
        let (start_point, end_point) = match (&gesture.start, &end) {
            (
                DirectBondEndpointIntentV2::ExistingAtom { .. },
                DirectBondEndpointIntentV2::NewAtomAt { raw_point },
            ) => (
                raw_start,
                snap_point(raw_start, *raw_point, gesture.snap)
                    .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?,
            ),
            (
                DirectBondEndpointIntentV2::NewAtomAt { raw_point },
                DirectBondEndpointIntentV2::ExistingAtom { .. },
            ) => (
                snap_point(raw_end, *raw_point, gesture.snap)
                    .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?,
                raw_end,
            ),
            (
                DirectBondEndpointIntentV2::NewAtomAt { .. },
                DirectBondEndpointIntentV2::NewAtomAt { raw_point },
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
                DirectBondEndpointIntentV2::ExistingAtom { atom: start },
                DirectBondEndpointIntentV2::ExistingAtom { atom: finish },
            ) => {
                if start == finish {
                    return Err(DirectBondAdmissionRefusalV1::SelfLoop);
                }
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
                DirectBondEndpointIntentV2::ExistingAtom { atom },
                DirectBondEndpointIntentV2::NewAtomAt { .. },
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
                )?;
                DirectBondAdmittedCandidateV2::ExistingNew {
                    existing: atom.clone(),
                    new_point: end_point,
                    element: gesture.new_atom_element.clone(),
                    presentation: gesture.presentation,
                }
            }
            (
                DirectBondEndpointIntentV2::NewAtomAt { .. },
                DirectBondEndpointIntentV2::ExistingAtom { atom },
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
                )?;
                DirectBondAdmittedCandidateV2::NewExisting {
                    new_point: start_point,
                    existing: atom.clone(),
                    element: gesture.new_atom_element.clone(),
                    presentation: gesture.presentation,
                }
            }
            (
                DirectBondEndpointIntentV2::NewAtomAt { .. },
                DirectBondEndpointIntentV2::NewAtomAt { .. },
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
        Ok(direct_bond_gesture_v2::admission(
            gesture,
            candidate,
            start_point,
            end_point,
        ))
    }

    pub fn commit_direct_bond_admission_v2(
        &mut self,
        admission: &DirectBondAdmissionV2,
    ) -> Result<CommittedDirectBondGestureV2, DirectBondCommitErrorV1> {
        if !admission.capability.belongs_to(self.direct_bond_origin) {
            return Err(DirectBondCommitErrorV1::ForeignSession);
        }
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
            } => {
                let position = Point3V1::new(new_point.x(), new_point.y(), 0.0)
                    .map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?;
                let mut pending = self
                    .prepare_create_bonded_atom_v2(
                        admission.fence.revision(),
                        existing,
                        element,
                        position,
                        *presentation,
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
            } => {
                let (_, end_id) = self
                    .resolve_bond_atom(existing)
                    .map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?;
                let position = Point3V1::new(new_point.x(), new_point.y(), 0.0)
                    .map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?;
                let mut pending = self
                    .prepare_create_bonded_atom_v2(
                        admission.fence.revision(),
                        existing,
                        element,
                        position,
                        *presentation,
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
                let DocumentBondPresentationV1::Normal(order) = presentation else {
                    return Err(DirectBondCommitErrorV1::CandidateApplicationFailed);
                };
                let make_atom = |point: DirectBondPoint2V1| -> Result<MoleculeInsertionAtomV1, DirectBondCommitErrorV1> { MoleculeInsertionAtomV1::new(element, Point3V1::new(point.x(), point.y(), 0.0).map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?, None, None, None).map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed) };
                let molecule = MoleculeInsertionV1::new(
                    vec![make_atom(*start)?, make_atom(*end)?],
                    vec![MoleculeInsertionBondV1::new(0, 1, *order)],
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
    /// Begin a pure direct normal-bond gesture from one existing direct atom.
    pub fn begin_direct_bond_gesture_v1(
        &self,
        fence: DocumentFenceV1,
        start_atom: DocumentObjectIdV1,
        presentation: DocumentBondPresentationV1,
        new_atom_element: String,
        snap: DirectBondSnapPolicyV1,
    ) -> Result<DirectBondGestureV1, DirectBondGestureErrorV1> {
        self.require_fence(fence)?;
        if !matches!(presentation, DocumentBondPresentationV1::Normal(_)) {
            return Err(DirectBondGestureErrorV1::UnsupportedPresentation);
        }
        let (start_molecule, _) = self
            .resolve_bond_atom(&start_atom)
            .map_err(|_| DirectBondGestureErrorV1::UnknownStartAtom)?;
        let start_point = self
            .direct_atom_point(&start_atom)
            .ok_or(DirectBondGestureErrorV1::UnknownStartAtom)?;
        Ok(DirectBondGestureV1 {
            capability: self.direct_bond_origin.issue_gesture(),
            fence,
            start_atom,
            start_molecule,
            presentation,
            new_atom_element,
            snap,
            start_point,
        })
    }

    /// Compute one disposable direct-bond preview without changing the document.
    pub fn preview_direct_bond_gesture_v1(
        &self,
        gesture: &DirectBondGestureV1,
        end: DirectBondEndIntentV1,
    ) -> Result<DirectBondPreviewV1, DirectBondGestureErrorV1> {
        self.admit_direct_bond_candidate_v1(gesture, end)
            .map(|admission| DirectBondPreviewV1 { admission })
            .map_err(Into::into)
    }

    /// Admit one complete direct normal-bond candidate without reserving IDs,
    /// provisional tokens, history, or mutable document state.
    pub fn admit_direct_bond_candidate_v1(
        &self,
        gesture: &DirectBondGestureV1,
        end: DirectBondEndIntentV1,
    ) -> Result<DirectBondAdmissionV1, DirectBondAdmissionRefusalV1> {
        self.require_direct_bond_origin(gesture)
            .map_err(|_| DirectBondAdmissionRefusalV1::ForeignSession)?;
        self.require_direct_bond_admission_fence(gesture.fence)?;
        if !matches!(gesture.presentation, DocumentBondPresentationV1::Normal(_)) {
            return Err(DirectBondAdmissionRefusalV1::UnsupportedPresentation);
        }
        let (start_molecule, start_atom_id) = self
            .resolve_bond_atom(&gesture.start_atom)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnknownStartAtom)?;
        let start_point = self
            .direct_atom_point(&gesture.start_atom)
            .ok_or(DirectBondAdmissionRefusalV1::UnknownStartAtom)?;
        match end {
            DirectBondEndIntentV1::ExistingAtom { atom } => {
                if atom == gesture.start_atom {
                    return Err(DirectBondAdmissionRefusalV1::SelfLoop);
                }
                let (molecule, end_atom_id) = self
                    .resolve_bond_atom(&atom)
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnknownEndAtom)?;
                if molecule != start_molecule {
                    return Err(DirectBondAdmissionRefusalV1::CrossMolecule);
                }
                self.reject_existing_bond_for_object_ids(&gesture.start_atom, &atom)
                    .map_err(|_| DirectBondAdmissionRefusalV1::DuplicateBond)?;
                let point = self
                    .direct_atom_point(&atom)
                    .ok_or(DirectBondAdmissionRefusalV1::UnknownEndAtom)?;
                self.admit_direct_bond_existing_chemistry(
                    &start_molecule,
                    &start_atom_id,
                    &end_atom_id,
                    gesture.presentation,
                )?;
                Ok(direct_bond_gesture_v1::admission(
                    gesture,
                    DirectBondAdmittedCandidateV1::ExistingEndpoint {
                        start_atom: gesture.start_atom.clone(),
                        end_atom: atom,
                        molecule: start_molecule,
                        presentation: gesture.presentation,
                    },
                    point,
                    false,
                ))
            }
            DirectBondEndIntentV1::NewAtomAt { raw_point } => {
                let point = snap_point(gesture.start_point, raw_point, gesture.snap)
                    .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?;
                if same_point(start_point, point) {
                    return Err(DirectBondAdmissionRefusalV1::CollapsedEndpoint);
                }
                self.admit_direct_bond_new_chemistry(
                    &start_molecule,
                    &start_atom_id,
                    &gesture.new_atom_element,
                    point,
                    gesture.presentation,
                )?;
                Ok(direct_bond_gesture_v1::admission(
                    gesture,
                    DirectBondAdmittedCandidateV1::NewEndpoint {
                        start_atom: gesture.start_atom.clone(),
                        molecule: start_molecule,
                        point,
                        element: gesture.new_atom_element.clone(),
                        presentation: gesture.presentation,
                    },
                    point,
                    true,
                ))
            }
        }
    }

    /// Commit one checked preview through the existing prepared insertion seam.
    pub fn commit_direct_bond_gesture_v1(
        &mut self,
        gesture: &DirectBondGestureV1,
        preview: &DirectBondPreviewV1,
    ) -> Result<CommittedDirectBondGestureV1, DirectBondGestureErrorV1> {
        self.require_direct_bond_origin(gesture)?;
        self.require_direct_bond_admission_origin(&preview.admission)
            .map_err(DirectBondGestureErrorV1::from)?;
        self.require_fence(gesture.fence)?;
        if preview.admission.capability != gesture.capability {
            return Err(DirectBondGestureErrorV1::PreviewMismatch);
        }
        self.commit_direct_bond_admission_v1(&preview.admission)
            .map_err(DirectBondGestureErrorV1::from)
    }

    /// Redeem an admitted candidate as one fenced, atomic document transition.
    pub fn commit_direct_bond_admission_v1(
        &mut self,
        admission: &DirectBondAdmissionV1,
    ) -> Result<CommittedDirectBondGestureV1, DirectBondCommitErrorV1> {
        self.require_direct_bond_admission_origin(admission)?;
        self.require_direct_bond_commit_fence(admission.fence)?;
        match &admission.candidate {
            DirectBondAdmittedCandidateV1::ExistingEndpoint {
                start_atom,
                end_atom: atom,
                presentation,
                ..
            } => {
                let (_, end_atom) = self
                    .resolve_bond_atom(atom)
                    .map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?;
                let mut pending = self
                    .prepare_create_bond_v2(
                        admission.fence.revision(),
                        start_atom,
                        atom,
                        *presentation,
                    )
                    .map_err(map_direct_bond_commit_error)?;
                let bond = pending.identifier().clone();
                let result = self
                    .commit_create_bond(admission.fence.revision(), &mut pending)
                    .map_err(map_direct_bond_commit_error)?;
                Ok(CommittedDirectBondGestureV1::ExistingEndpoint {
                    bond,
                    end_atom,
                    result,
                })
            }
            DirectBondAdmittedCandidateV1::NewEndpoint {
                start_atom,
                point,
                element,
                presentation,
                ..
            } => {
                let position = Point3V1::new(point.x(), point.y(), 0.0)
                    .map_err(|_| DirectBondCommitErrorV1::CandidateApplicationFailed)?;
                let mut pending = self
                    .prepare_create_bonded_atom_v2(
                        admission.fence.revision(),
                        start_atom,
                        element,
                        position,
                        *presentation,
                    )
                    .map_err(map_direct_bond_commit_error)?;
                let atom = pending.atom_identifier().clone();
                let bond = pending.bond_identifier().clone();
                let result = self
                    .commit_create_bonded_atom(admission.fence.revision(), &mut pending)
                    .map_err(map_direct_bond_commit_error)?;
                Ok(CommittedDirectBondGestureV1::NewEndpoint { bond, atom, result })
            }
        }
    }

    fn require_direct_bond_origin(
        &self,
        gesture: &DirectBondGestureV1,
    ) -> Result<(), DirectBondGestureErrorV1> {
        if gesture.capability.belongs_to(self.direct_bond_origin) {
            Ok(())
        } else {
            Err(DirectBondGestureErrorV1::ForeignSession)
        }
    }

    fn require_direct_bond_admission_origin(
        &self,
        admission: &DirectBondAdmissionV1,
    ) -> Result<(), DirectBondCommitErrorV1> {
        if admission.capability.belongs_to(self.direct_bond_origin) {
            Ok(())
        } else {
            Err(DirectBondCommitErrorV1::ForeignSession)
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
    ) -> Result<(), DirectBondAdmissionRefusalV1> {
        let atom_id = PersistentId::new("ferrum-direct-bond-admission-atom")
            .expect("static direct-bond admission identifier is nonblank");
        let bond_id = PersistentId::new("ferrum-direct-bond-admission-bond")
            .expect("static direct-bond admission identifier is nonblank");
        let position = Point3V1::new(point.x(), point.y(), 0.0)
            .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?;
        let insertion =
            BondedAtomInsertion::new(&atom_id, &bond_id, element, position, presentation);
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_bonded_atom(molecule_id, start_atom_id, insertion)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        self.require_admitted_direct_bond_capacity(&candidate, molecule_id)?;
        self.require_admitted_direct_bond_renderability(&candidate, molecule_id)
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
        let DocumentBondPresentationV1::Normal(order) = presentation else {
            return Err(DirectBondAdmissionRefusalV1::UnsupportedPresentation);
        };
        let molecule = MoleculeInsertionV1::new(
            vec![make_atom(start)?, make_atom(end)?],
            vec![MoleculeInsertionBondV1::new(0, 1, order)],
        )
        .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_molecule(&molecule_id, &atom_ids, &bond_ids, &molecule)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        self.require_admitted_direct_bond_capacity(&candidate, &molecule_id)?;
        self.require_admitted_direct_bond_renderability(&candidate, &molecule_id)
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

    fn require_admitted_direct_bond_renderability(
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
        let graph = crate::chemistry::document_molecule_graph_v1(&molecule)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?
            .into_parts()
            .0;
        crate::chemistry::validate_supported_complete_graph_facts_v1(&graph)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)
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

    fn point(x: f64, y: f64) -> DirectBondEndpointIntentV2 {
        DirectBondEndpointIntentV2::NewAtomAt {
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
            .begin_direct_bond_gesture_v2(
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
            .begin_direct_bond_gesture_v2(
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
            .begin_direct_bond_gesture_v2(
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
            .begin_direct_bond_gesture_v2(
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
            .begin_direct_bond_gesture_v2(
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
            .begin_direct_bond_gesture_v2(
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
            .begin_direct_bond_gesture_v2(
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
