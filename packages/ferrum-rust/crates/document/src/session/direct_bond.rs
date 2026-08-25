//! Document-owned renderer-admitted direct-bond transaction.

use super::*;
use crate::AuthoringCapabilityClaimV1;
use crate::direct_bond_mutation::DirectBondEndpointIntent;
use crate::session_operation::CreateDirectBondV1;

#[derive(Clone, Debug, PartialEq)]
struct DirectBondGestureV2 {
    fence: DocumentFenceV1,
    start: DirectBondEndpointIntent,
    presentation: DocumentBondPresentationV1,
    new_atom_element: String,
    snap: DirectBondSnapPolicyV1,
}

#[derive(Clone, Debug, PartialEq)]
enum DirectBondCandidateV2 {
    ExistingExisting {
        start: DocumentObjectIdV1,
        end: DocumentObjectIdV1,
        presentation: DocumentBondPresentationV1,
    },
    ExistingNew {
        existing: DocumentObjectIdV1,
        point: DirectBondPoint2V1,
        element: String,
        presentation: DocumentBondPresentationV1,
        new_atom_is_start: bool,
    },
    NewExisting {
        existing: DocumentObjectIdV1,
        point: DirectBondPoint2V1,
        element: String,
        presentation: DocumentBondPresentationV1,
        new_atom_is_start: bool,
    },
    NewNew {
        start: DirectBondPoint2V1,
        end: DirectBondPoint2V1,
        element: String,
        presentation: DocumentBondPresentationV1,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct DirectBondSemanticCandidateV1 {
    candidate: DirectBondCandidateV2,
    start: DirectBondPoint2V1,
    end: DirectBondPoint2V1,
}

#[derive(Clone, Debug)]
pub(crate) struct DirectBondOutcomeStagingV1 {
    pub(crate) bond_document_object_id: DocumentObjectIdV1,
    pub(crate) end_atom_document_object_id: DocumentObjectIdV1,
    pub(crate) second_created_atom_document_object_id: Option<DocumentObjectIdV1>,
    pub(crate) created_new_atom: bool,
    pub(crate) created_new_molecule: bool,
}

impl DocumentSession {
    pub(crate) fn prepare_create_direct_bond_v1(
        &mut self,
        request: CreateDirectBondV1,
        authorization_claim: AuthoringCapabilityClaimV1,
    ) -> Result<PreparedSessionTransitionV1, DirectBondAdmissionRefusalV1> {
        let fence = request.fence();
        let gesture = self
            .begin_direct_bond_mutation(
                fence,
                request.start().clone(),
                request.presentation(),
                request.new_atom_element().to_owned(),
                request.snap(),
            )
            .map_err(map_gesture_refusal)?;
        let admitted = self.admit_direct_bond_candidate_v2(&gesture, request.end().clone())?;
        let source_digest = self.current_digest_v1();
        let built = self.build_direct_bond_candidate(&admitted.candidate)?;
        let mut transition = self
            .prepare_changed_session_transition_with_direct_bond_outcome_v1(
                fence.revision(),
                source_digest,
                built.candidate,
                built.effects,
                DirectBondOutcomeStagingV1 {
                    bond_document_object_id: built.bond_document_object_id,
                    end_atom_document_object_id: built.end_atom_document_object_id,
                    second_created_atom_document_object_id: built
                        .second_created_atom_document_object_id,
                    created_new_atom: built.created_new_atom,
                    created_new_molecule: built.created_new_molecule,
                },
                authorization_claim,
            )
            .map_err(|_| DirectBondAdmissionRefusalV1::UnrenderableCandidate)?;
        let overlay_targets = built.overlay_targets;
        let overlay_request = ferrum_render::AcceptedRenderOverlayRequestV1::new(overlay_targets)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnrenderableCandidate)?;
        let overlay = transition
            .renderer_precommit_overlay_v1(&overlay_request)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnrenderableCandidate)?;
        transition
            .install_precommit_overlay_v1(overlay)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnrenderableCandidate)?;
        Ok(transition)
    }

    fn build_direct_bond_candidate(
        &self,
        candidate: &DirectBondCandidateV2,
    ) -> Result<BuiltDirectBondCandidateV1, DirectBondAdmissionRefusalV1> {
        let revision = self
            .current_state_v1()
            .next_revision()
            .ok_or(DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        let document = self.current_document_v1();
        match candidate {
            DirectBondCandidateV2::ExistingExisting {
                start,
                end,
                presentation,
            } => {
                let (molecule, start_id) = self
                    .resolve_bond_atom(start)
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnknownStartAtom)?;
                let (_, end_atom) = self
                    .resolve_bond_atom(end)
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnknownEndAtom)?;
                let (bond, effects) = self
                    .reserve_generated_ids_for_transition_v1(|ids, indexed| {
                        ids.reserve_bond(indexed)
                    })
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
                let typed = document
                    .with_insert_bond(&molecule, &bond, &start_id, &end_atom, *presentation)
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
                Ok(BuiltDirectBondCandidateV1::new(
                    DirectBondCandidateContextV1 {
                        revision,
                        document: typed,
                        effects,
                    },
                    DirectBondAdmittedFactsV1 {
                        bond,
                        end_atom,
                        second_created_atom: None,
                        created_atoms: Vec::new(),
                        created_new_atom: false,
                        created_new_molecule: false,
                    },
                )?)
            }
            DirectBondCandidateV2::ExistingNew {
                existing,
                point,
                element,
                presentation,
                new_atom_is_start,
            }
            | DirectBondCandidateV2::NewExisting {
                existing,
                point,
                element,
                presentation,
                new_atom_is_start,
            } => {
                let (molecule, existing_id) = self
                    .resolve_bond_atom(existing)
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnknownStartAtom)?;
                let (identities, effects) = self
                    .reserve_generated_ids_for_transition_v1(|ids, indexed| {
                        ids.reserve_bonded_atom(indexed)
                    })
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
                let position = Point3V1::new(point.x(), point.y(), 0.0)
                    .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?;
                let typed = document
                    .with_insert_bonded_atom(
                        &molecule,
                        &existing_id,
                        BondedAtomInsertion::new(
                            &identities.atom,
                            &identities.bond,
                            element,
                            position,
                            *presentation,
                            *new_atom_is_start,
                        ),
                    )
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
                let end_atom = if matches!(candidate, DirectBondCandidateV2::NewExisting { .. }) {
                    existing_id
                } else {
                    identities.atom.clone()
                };
                Ok(BuiltDirectBondCandidateV1::new(
                    DirectBondCandidateContextV1 {
                        revision,
                        document: typed,
                        effects,
                    },
                    DirectBondAdmittedFactsV1 {
                        bond: identities.bond,
                        end_atom,
                        second_created_atom: None,
                        created_atoms: vec![identities.atom],
                        created_new_atom: true,
                        created_new_molecule: false,
                    },
                )?)
            }
            DirectBondCandidateV2::NewNew {
                start,
                end,
                element,
                presentation,
            } => {
                let (identities, effects) = self
                    .reserve_generated_ids_for_transition_v1(|ids, indexed| {
                        ids.reserve_molecule(indexed, 2, 1)
                    })
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
                let atom = |point: DirectBondPoint2V1| {
                    MoleculeInsertionAtomV1::new(
                        element,
                        Point3V1::new(point.x(), point.y(), 0.0)
                            .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?,
                        None,
                        None,
                        None,
                    )
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)
                };
                let molecule = MoleculeInsertionV1::new(
                    vec![atom(*start)?, atom(*end)?],
                    vec![MoleculeInsertionBondV1::new_with_presentation(
                        0,
                        1,
                        *presentation,
                    )],
                )
                .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
                let typed = document
                    .with_insert_molecule(
                        &identities.molecule,
                        &identities.atoms,
                        &identities.bonds,
                        &molecule,
                    )
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
                Ok(BuiltDirectBondCandidateV1::new(
                    DirectBondCandidateContextV1 {
                        revision,
                        document: typed,
                        effects,
                    },
                    DirectBondAdmittedFactsV1 {
                        bond: identities.bonds[0].clone(),
                        end_atom: identities.atoms[1].clone(),
                        second_created_atom: Some(identities.atoms[0].clone()),
                        created_atoms: identities.atoms,
                        created_new_atom: true,
                        created_new_molecule: true,
                    },
                )?)
            }
        }
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
    ) -> Result<DirectBondSemanticCandidateV1, DirectBondAdmissionRefusalV1> {
        self.require_direct_bond_admission_fence(gesture.fence)?;
        let point_for = |intent: &DirectBondEndpointIntent, unknown| match intent {
            DirectBondEndpointIntent::ExistingAtom { atom } => self
                .direct_atom_point(atom)
                .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?
                .ok_or(unknown),
            DirectBondEndpointIntent::NewAtomAt { raw_point } => Ok(*raw_point),
        };
        let raw_start = point_for(
            &gesture.start,
            DirectBondAdmissionRefusalV1::UnknownStartAtom,
        )?;
        let raw_end = point_for(&end, DirectBondAdmissionRefusalV1::UnknownEndAtom)?;
        if matches!((&gesture.start, &end), (DirectBondEndpointIntent::ExistingAtom { atom: a }, DirectBondEndpointIntent::ExistingAtom { atom: b }) if a == b)
        {
            return Err(DirectBondAdmissionRefusalV1::SelfLoop);
        }
        let (start, finish) =
            snapped_endpoints(raw_start, raw_end, &gesture.start, &end, gesture.snap)?;
        if start.x() == finish.x() && start.y() == finish.y() {
            return Err(DirectBondAdmissionRefusalV1::CollapsedEndpoint);
        }
        let candidate = match (&gesture.start, &end) {
            (
                DirectBondEndpointIntent::ExistingAtom { atom: start },
                DirectBondEndpointIntent::ExistingAtom { atom: end },
            ) => {
                let (molecule, start_id) = self
                    .resolve_bond_atom(start)
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnknownStartAtom)?;
                let (other, end_id) = self
                    .resolve_bond_atom(end)
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnknownEndAtom)?;
                if molecule != other {
                    return Err(DirectBondAdmissionRefusalV1::CrossMolecule);
                }
                self.reject_existing_bond_for_object_ids(start, end)
                    .map_err(|_| DirectBondAdmissionRefusalV1::DuplicateBond)?;
                self.admit_direct_bond_existing_chemistry(
                    &molecule,
                    &start_id,
                    &end_id,
                    gesture.presentation,
                )?;
                DirectBondCandidateV2::ExistingExisting {
                    start: start.clone(),
                    end: end.clone(),
                    presentation: gesture.presentation,
                }
            }
            (
                DirectBondEndpointIntent::ExistingAtom { atom },
                DirectBondEndpointIntent::NewAtomAt { .. },
            ) => {
                let (molecule, id) = self
                    .resolve_bond_atom(atom)
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnknownStartAtom)?;
                self.admit_direct_bond_new_chemistry(
                    &molecule,
                    &id,
                    &gesture.new_atom_element,
                    finish,
                    gesture.presentation,
                    false,
                )?;
                DirectBondCandidateV2::ExistingNew {
                    existing: atom.clone(),
                    point: finish,
                    element: gesture.new_atom_element.clone(),
                    presentation: gesture.presentation,
                    new_atom_is_start: false,
                }
            }
            (
                DirectBondEndpointIntent::NewAtomAt { .. },
                DirectBondEndpointIntent::ExistingAtom { atom },
            ) => {
                let (molecule, id) = self
                    .resolve_bond_atom(atom)
                    .map_err(|_| DirectBondAdmissionRefusalV1::UnknownEndAtom)?;
                self.admit_direct_bond_new_chemistry(
                    &molecule,
                    &id,
                    &gesture.new_atom_element,
                    start,
                    gesture.presentation,
                    true,
                )?;
                DirectBondCandidateV2::NewExisting {
                    existing: atom.clone(),
                    point: start,
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
                    start,
                    finish,
                    gesture.presentation,
                )?;
                DirectBondCandidateV2::NewNew {
                    start,
                    end: finish,
                    element: gesture.new_atom_element.clone(),
                    presentation: gesture.presentation,
                }
            }
        };
        Ok(DirectBondSemanticCandidateV1 {
            candidate,
            start,
            end: finish,
        })
    }

    fn require_direct_bond_admission_fence(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<(), DirectBondAdmissionRefusalV1> {
        if self.current_revision_v1() != fence.revision() {
            return Err(DirectBondAdmissionRefusalV1::StaleRevision);
        }
        if self.current_digest_v1() != fence.digest() {
            return Err(DirectBondAdmissionRefusalV1::StaleDigest);
        }
        Ok(())
    }
    fn admit_direct_bond_existing_chemistry(
        &self,
        molecule: &PersistentId,
        start: &PersistentId,
        end: &PersistentId,
        presentation: DocumentBondPresentationV1,
    ) -> Result<(), DirectBondAdmissionRefusalV1> {
        let bond =
            PersistentId::new("ferrum-direct-bond-admission-bond").expect("static identifier");
        let candidate = self
            .current_state_v1()
            .document()
            .with_insert_bond(molecule, &bond, start, end, presentation)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        self.require_admitted_direct_bond_capacity(&candidate, molecule)
    }
    fn admit_direct_bond_new_chemistry(
        &self,
        molecule: &PersistentId,
        start: &PersistentId,
        element: &str,
        point: DirectBondPoint2V1,
        presentation: DocumentBondPresentationV1,
        new_atom_is_start: bool,
    ) -> Result<(), DirectBondAdmissionRefusalV1> {
        let atom =
            PersistentId::new("ferrum-direct-bond-admission-atom").expect("static identifier");
        let bond =
            PersistentId::new("ferrum-direct-bond-admission-bond").expect("static identifier");
        let position = Point3V1::new(point.x(), point.y(), 0.0)
            .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?;
        let candidate = self
            .current_state_v1()
            .document()
            .with_insert_bonded_atom(
                molecule,
                start,
                BondedAtomInsertion::new(
                    &atom,
                    &bond,
                    element,
                    position,
                    presentation,
                    new_atom_is_start,
                ),
            )
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        self.require_admitted_direct_bond_capacity(&candidate, molecule)
    }
    fn admit_direct_bond_new_molecule_chemistry(
        &self,
        element: &str,
        start: DirectBondPoint2V1,
        end: DirectBondPoint2V1,
        presentation: DocumentBondPresentationV1,
    ) -> Result<(), DirectBondAdmissionRefusalV1> {
        let molecule_id =
            PersistentId::new("ferrum-direct-bond-admission-molecule").expect("static identifier");
        let atoms = [
            PersistentId::new("ferrum-direct-bond-admission-start-atom")
                .expect("static identifier"),
            PersistentId::new("ferrum-direct-bond-admission-end-atom").expect("static identifier"),
        ];
        let bonds =
            [PersistentId::new("ferrum-direct-bond-admission-bond").expect("static identifier")];
        let atom = |point: DirectBondPoint2V1| {
            MoleculeInsertionAtomV1::new(
                element,
                Point3V1::new(point.x(), point.y(), 0.0)
                    .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)?,
                None,
                None,
                None,
            )
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)
        };
        let molecule = MoleculeInsertionV1::new(
            vec![atom(start)?, atom(end)?],
            vec![MoleculeInsertionBondV1::new_with_presentation(
                0,
                1,
                presentation,
            )],
        )
        .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        let candidate = self
            .current_state_v1()
            .document()
            .with_insert_molecule(&molecule_id, &atoms, &bonds, &molecule)
            .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        self.require_admitted_direct_bond_capacity(&candidate, &molecule_id)
    }
    fn require_admitted_direct_bond_capacity(
        &self,
        candidate: &TypedDocument,
        molecule: &PersistentId,
    ) -> Result<(), DirectBondAdmissionRefusalV1> {
        let object = candidate
            .document_object_id_for_source_id_v1(molecule)
            .ok_or(DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?;
        let molecule = candidate
            .core_molecule(&object)
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

struct DirectBondCandidateContextV1 {
    revision: u64,
    document: TypedDocument,
    effects: SessionTransitionEffectsV1,
}

struct DirectBondAdmittedFactsV1 {
    bond: PersistentId,
    end_atom: PersistentId,
    second_created_atom: Option<PersistentId>,
    created_atoms: Vec<PersistentId>,
    created_new_atom: bool,
    created_new_molecule: bool,
}

struct BuiltDirectBondCandidateV1 {
    candidate: RevisionState,
    effects: SessionTransitionEffectsV1,
    bond_document_object_id: DocumentObjectIdV1,
    end_atom_document_object_id: DocumentObjectIdV1,
    second_created_atom_document_object_id: Option<DocumentObjectIdV1>,
    overlay_targets: Vec<ferrum_render::AcceptedRenderOverlayTargetV1>,
    created_new_atom: bool,
    created_new_molecule: bool,
}
impl BuiltDirectBondCandidateV1 {
    fn new(
        context: DirectBondCandidateContextV1,
        facts: DirectBondAdmittedFactsV1,
    ) -> Result<Self, DirectBondAdmissionRefusalV1> {
        let mut overlay_targets = facts
            .created_atoms
            .iter()
            .map(|atom| {
                context
                    .document
                    .document_object_id_for_source_id_v1(atom)
                    .map(ferrum_render::AcceptedRenderOverlayTargetV1::atom)
                    .ok_or(DirectBondAdmissionRefusalV1::UnrenderableCandidate)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let bond_object_id = context
            .document
            .document_object_id_for_source_id_v1(&facts.bond)
            .ok_or(DirectBondAdmissionRefusalV1::UnrenderableCandidate)?;
        let end_atom_document_object_id = context
            .document
            .document_object_id_for_source_id_v1(&facts.end_atom)
            .ok_or(DirectBondAdmissionRefusalV1::UnrenderableCandidate)?;
        let second_created_atom_document_object_id = facts
            .second_created_atom
            .as_ref()
            .map(|atom| {
                context
                    .document
                    .document_object_id_for_source_id_v1(atom)
                    .ok_or(DirectBondAdmissionRefusalV1::UnrenderableCandidate)
            })
            .transpose()?;
        overlay_targets.push(ferrum_render::AcceptedRenderOverlayTargetV1::bond(
            bond_object_id.clone(),
        ));
        Ok(Self {
            candidate: RevisionState::from_document(context.revision, context.document)
                .map_err(|_| DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission)?,
            effects: context.effects,
            bond_document_object_id: bond_object_id,
            end_atom_document_object_id,
            second_created_atom_document_object_id,
            overlay_targets,
            created_new_atom: facts.created_new_atom,
            created_new_molecule: facts.created_new_molecule,
        })
    }
}

fn snapped_endpoints(
    start: DirectBondPoint2V1,
    end: DirectBondPoint2V1,
    start_intent: &DirectBondEndpointIntent,
    end_intent: &DirectBondEndpointIntent,
    policy: DirectBondSnapPolicyV1,
) -> Result<(DirectBondPoint2V1, DirectBondPoint2V1), DirectBondAdmissionRefusalV1> {
    let snap = |origin, raw| {
        snap_point(origin, raw, policy)
            .map_err(|_| DirectBondAdmissionRefusalV1::InvalidEndpointInput)
    };
    match (start_intent, end_intent) {
        (
            DirectBondEndpointIntent::ExistingAtom { .. },
            DirectBondEndpointIntent::NewAtomAt { .. },
        ) => Ok((start, snap(start, end)?)),
        (
            DirectBondEndpointIntent::NewAtomAt { .. },
            DirectBondEndpointIntent::ExistingAtom { .. },
        ) => Ok((snap(end, start)?, end)),
        (
            DirectBondEndpointIntent::NewAtomAt { .. },
            DirectBondEndpointIntent::NewAtomAt { .. },
        ) => Ok((start, snap(start, end)?)),
        _ => Ok((start, end)),
    }
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
        let angle = (dy.atan2(dx) / step).round() * step;
        dx = length * angle.cos();
        dy = length * angle.sin();
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
fn map_gesture_refusal(error: DirectBondGestureErrorV1) -> DirectBondAdmissionRefusalV1 {
    match error {
        DirectBondGestureErrorV1::StaleRevision => DirectBondAdmissionRefusalV1::StaleRevision,
        DirectBondGestureErrorV1::StaleDigest => DirectBondAdmissionRefusalV1::StaleDigest,
        _ => DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission,
    }
}
