use super::model::{
    prepare_create_reaction, prepare_delete_reaction, prepare_replace_reaction_members,
};
use super::*;

impl SessionOperation {
    pub(crate) fn prepare(
        &self,
        current: &TypedDocument,
        current_revision: u64,
        current_digest: &[u8; 32],
    ) -> Result<Candidate, SessionOperationError> {
        match self {
            Self::V1(SessionOperationV1::MaterializeCompactGroupV1(_)) => {
                Err(SessionOperationError::CompactGroupMaterializationRequiresTransitionCore)
            }
            Self::V1(SessionOperationV1::MaterializeMoleculeHydrogensV1(_)) => {
                Err(SessionOperationError::HydrogenMaterializationRequiresTransitionCore)
            }
            Self::V1(SessionOperationV1::InsertMoleculeV1(_)) => {
                Err(SessionOperationError::MoleculeInsertionRequiresTransitionCore)
            }
            Self::V1(SessionOperationV1::CreateHaworthMoleculeV1(_)) => {
                Err(SessionOperationError::MoleculeInsertionRequiresTransitionCore)
            }
            Self::V1(SessionOperationV1::InsertInterchangeRecordBatchV1(_)) => {
                Err(SessionOperationError::InterchangeRecordBatchInsertionRequiresTransitionCore)
            }
            Self::V1(
                SessionOperationV1::CreateCurvedTerminalArrowV1(_)
                | SessionOperationV1::CreateCurvedEquilibriumArrowV1(_)
                | SessionOperationV1::CreatePresentationPathV1(_)
                | SessionOperationV1::CreatePresentationVectorV1(_)
                | SessionOperationV1::CreatePresentationRootV1(_),
            ) => Err(SessionOperationError::PresentationCreateRequiresTransitionCore),
            Self::V1(SessionOperationV1::PlaceCatalogMoleculeV1(_)) => {
                Err(SessionOperationError::InvalidCatalogPlacement(
                    "catalog placement must be prepared by the session transition core".to_owned(),
                ))
            }
            Self::V1(SessionOperationV1::CreateDirectBondV1(_)) => {
                Err(DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission.into())
            }
            Self::V1(SessionOperationV1::CreateAtomV1(_) | SessionOperationV1::CreateBondV1(_)) => {
                Err(SessionOperationError::PresentationCreateRequiresTransitionCore)
            }
            Self::V1(SessionOperationV1::CreateReactionV1(request)) => {
                let (candidate, _) = prepare_create_reaction(current, request)?;
                Ok(Candidate::Changed(Box::new(candidate)))
            }
            Self::V1(SessionOperationV1::ReplaceReactionMembersV1(request)) => {
                let (candidate, _) = prepare_replace_reaction_members(current, request)?;
                Ok(Candidate::Changed(Box::new(candidate)))
            }
            Self::V1(SessionOperationV1::DeleteReactionV1(request)) => {
                let (candidate, _) = prepare_delete_reaction(current, request)?;
                Ok(Candidate::Changed(Box::new(candidate)))
            }
            Self::V1(SessionOperationV1::SetAtomElement { atom_id, element }) => {
                if !valid_atom_element(element) {
                    return Err(SessionOperationError::InvalidAtomElement);
                }
                let identifier = PersistentId::new(atom_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let candidate = current.with_atom_element(&identifier, element)?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetAtomProperties { patch }) => {
                let candidate = current.with_atom_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownAtom(patch.atom_id().as_str().to_owned())
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetAtomNumber {
                molecule_id,
                atom_id,
                number,
                show_number,
            }) => {
                let valid_pair = matches!((number, show_number), (Some(value), Some(_)) if *value > 0)
                    || matches!((number, show_number), (None, None));
                if !valid_pair {
                    return Err(SessionOperationError::InvalidAtomNumberPair);
                }
                let molecule = PersistentId::new(molecule_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let atom = PersistentId::new(atom_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let assignment = number.zip(*show_number);
                let candidate = current.with_atom_number(&molecule, &atom, assignment)?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetMoleculeName { molecule_id, name }) => {
                let name = name.as_deref().filter(|value| !value.is_empty());
                let candidate = current.with_molecule_name(molecule_id, name)?;
                let candidate = candidate.ok_or(SessionOperationError::UnknownMolecule)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::ApplyAtomMark {
                molecule_id,
                atom_id,
                action,
                kind,
                matching_mark_index,
            }) => {
                if *action == AtomMarkActionV1::Add && matching_mark_index.is_some() {
                    return Err(SessionOperationError::InvalidAtomMarkSelector);
                }
                let molecule = PersistentId::new(molecule_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let atom = PersistentId::new(atom_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let candidate = current.with_atom_mark(
                    &molecule,
                    &atom,
                    *action,
                    *kind,
                    *matching_mark_index,
                )?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetAtomPosition { atom_id, position }) => {
                let identifier = PersistentId::new(atom_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let candidate = current.with_atom_position(&identifier, *position)?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::RotateAtoms { rotation }) => {
                let candidate = current.with_atom_rotation(rotation)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::RepairGeometry { repair }) => {
                let candidate = current.with_geometry_repair(repair)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::DeleteAtom { atom_id }) => {
                let identifier = PersistentId::new(atom_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let candidate = current.with_delete_atom(&identifier)?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                Ok(Candidate::Changed(Box::new(candidate)))
            }
            Self::V1(SessionOperationV1::DeleteBond { bond_id }) => {
                let identifier = PersistentId::new(bond_id.clone())
                    .map_err(|_| SessionOperationError::UnknownBond(bond_id.clone()))?;
                let candidate = current.with_delete_bond(&identifier)?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownBond(bond_id.clone()))?;
                Ok(Candidate::Changed(Box::new(candidate)))
            }
            Self::V1(SessionOperationV1::DeleteStructure {
                molecule_id,
                atom_ids,
                bond_ids,
            }) => {
                let _ = (molecule_id, atom_ids, bond_ids, current);
                Err(SessionOperationError::Candidate(
                    TypedDocumentError::StructuralDeletionRequiresSession,
                ))
            }
            Self::V1(SessionOperationV1::DeletePresentationRoot { deletion }) => {
                let candidate = current.with_delete_presentation_root(deletion)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownPresentationRoot(
                        deletion.document_object_id().as_str().to_owned(),
                    )
                })?;
                Ok(Candidate::Changed(Box::new(candidate)))
            }
            Self::V1(SessionOperationV1::DeletePresentationRoots { deletions }) => {
                let candidate = current.with_delete_presentation_roots(deletions)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownPresentationRoot(
                        deletions.targets()[0]
                            .document_object_id()
                            .as_str()
                            .to_owned(),
                    )
                })?;
                Ok(Candidate::Changed(Box::new(candidate)))
            }
            Self::V1(SessionOperationV1::ReorderPresentationRoots { reorder }) => {
                let candidate = current.with_reorder_presentation_roots(reorder)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownPresentationRoot(
                        reorder.targets()[0]
                            .document_object_id()
                            .as_str()
                            .to_owned(),
                    )
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::ApplyTopLevelRootLayoutTransformV1(transform)) => {
                let candidate = current.with_top_level_transform(transform.common_transform())?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::TranslateTopLevelRootsV1(transform)) => {
                let candidate = current.with_top_level_transform(transform.common_transform())?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetBondOrder { bond_id, order }) => {
                let identifier = PersistentId::new(bond_id.clone())
                    .map_err(|_| SessionOperationError::UnknownBond(bond_id.clone()))?;
                let candidate = current.with_bond_order(&identifier, *order)?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownBond(bond_id.clone()))?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetBondProperties { patch }) => {
                let candidate = current.with_bond_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownBond(patch.bond_id().as_str().to_owned())
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetPlusProperties { patch }) => {
                let candidate = current.with_plus_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownPlus(patch.plus_object_id().as_str().to_owned())
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetTextProperties { patch }) => {
                let candidate = current.with_text_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownText(patch.text_object_id().as_str().to_owned())
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetPaperProperties { patch }) => {
                if patch.changes().is_empty() {
                    return Ok(Candidate::NoChange);
                }
                let effective_type = patch
                    .changes()
                    .iter()
                    .find_map(|change| match change {
                        PaperPropertyChangeV1::Type(value) => Some(value.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| current.paper_type_or_default_v1());
                if effective_type != "custom"
                    && patch
                        .changes()
                        .iter()
                        .any(|change| matches!(change, PaperPropertyChangeV1::Dimensions(_)))
                {
                    return Err(SessionOperationError::PaperDimensionsRequireCustom);
                }
                let candidate = current.with_paper_properties(patch)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetDrawingStandard { patch }) => {
                if patch.changes().is_empty() {
                    return Ok(Candidate::NoChange);
                }
                let candidate = current.with_drawing_standard(patch)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetArrowProperties { patch }) => {
                let candidate = current.with_arrow_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownArrow(patch.arrow_object_id().as_str().to_owned())
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetGeometricProperties { patch }) => {
                let candidate = current.with_geometric_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownGeometricPresentation(
                        patch.presentation_id().as_str().to_owned(),
                    )
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetWavyProperties { patch }) => {
                let candidate = current.with_wavy_properties(patch)?;
                let candidate = candidate
                    .ok_or_else(|| SessionOperationError::UnknownWavy(patch.wavy_id().clone()))?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetBracketProperties { patch }) => {
                let candidate = current.with_bracket_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownBracketPair(patch.members().clone())
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetMoleculeAtomPositions { update }) => {
                if update.source_revision() != current_revision {
                    return Err(SessionOperationError::MoleculeCoordinateRevisionMismatch {
                        prepared: update.source_revision(),
                        current: current_revision,
                    });
                }
                if update.source_digest() != current_digest {
                    return Err(SessionOperationError::MoleculeCoordinateDigestMismatch);
                }
                let object_id = update.molecule_id().as_str().to_owned();
                let record = current
                    .resolve_document_object_id(update.molecule_id())
                    .ok_or_else(|| {
                        SessionOperationError::UnknownDocumentObject(object_id.clone())
                    })?;
                if record.class() != TypedClass::Molecule {
                    return Err(SessionOperationError::InvalidMoleculeCoordinateTarget(
                        object_id,
                    ));
                }
                let source_id = record.attribute("id").ok_or_else(|| {
                    SessionOperationError::InvalidMoleculeCoordinateTarget(object_id.clone())
                })?;
                let molecule_id = PersistentId::new(source_id.to_owned()).map_err(|_| {
                    SessionOperationError::InvalidMoleculeCoordinateTarget(object_id)
                })?;
                let candidate = current
                    .with_molecule_atom_positions(&molecule_id, update.positions())?
                    .ok_or_else(|| {
                        SessionOperationError::UnknownDocumentObject(
                            update.molecule_id().as_str().to_owned(),
                        )
                    })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetMoleculeAtomPositionsBatch { update }) => {
                if update.source_revision() != current_revision {
                    return Err(SessionOperationError::MoleculeCoordinateRevisionMismatch {
                        prepared: update.source_revision(),
                        current: current_revision,
                    });
                }
                if update.source_digest() != current_digest {
                    return Err(SessionOperationError::MoleculeCoordinateDigestMismatch);
                }
                let mut replacements = Vec::with_capacity(update.updates().len());
                for entry in update.updates() {
                    let object_id = entry.molecule_id().as_str().to_owned();
                    let record = current
                        .resolve_document_object_id(entry.molecule_id())
                        .ok_or_else(|| {
                            SessionOperationError::UnknownDocumentObject(object_id.clone())
                        })?;
                    if record.class() != TypedClass::Molecule {
                        return Err(SessionOperationError::InvalidMoleculeCoordinateTarget(
                            object_id,
                        ));
                    }
                    let source_id = record.attribute("id").ok_or_else(|| {
                        SessionOperationError::InvalidMoleculeCoordinateTarget(
                            entry.molecule_id().as_str().to_owned(),
                        )
                    })?;
                    let molecule_id = PersistentId::new(source_id.to_owned()).map_err(|_| {
                        SessionOperationError::InvalidMoleculeCoordinateTarget(
                            entry.molecule_id().as_str().to_owned(),
                        )
                    })?;
                    replacements.push((molecule_id, entry.positions().to_vec()));
                }
                let candidate = current.with_molecule_atom_positions_batch(&replacements)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetCleanGeometry { update }) => {
                if update.source_revision() != current_revision {
                    return Err(SessionOperationError::MoleculeCoordinateRevisionMismatch {
                        prepared: update.source_revision(),
                        current: current_revision,
                    });
                }
                if update.source_digest() != current_digest {
                    return Err(SessionOperationError::MoleculeCoordinateDigestMismatch);
                }
                let mut replacements = Vec::with_capacity(update.molecules().len());
                for molecule in update.molecules() {
                    let object_id = molecule.molecule_id().as_str().to_owned();
                    let record = current
                        .resolve_document_object_id(molecule.molecule_id())
                        .ok_or_else(|| {
                            SessionOperationError::UnknownDocumentObject(object_id.clone())
                        })?;
                    if record.class() != TypedClass::Molecule {
                        return Err(SessionOperationError::InvalidMoleculeCoordinateTarget(
                            object_id,
                        ));
                    }
                    let source_id = record.attribute("id").ok_or_else(|| {
                        SessionOperationError::InvalidMoleculeCoordinateTarget(object_id.clone())
                    })?;
                    let molecule_id = PersistentId::new(source_id.to_owned()).map_err(|_| {
                        SessionOperationError::InvalidMoleculeCoordinateTarget(object_id)
                    })?;
                    replacements.push((molecule_id, molecule.positions().to_vec()));
                }
                let candidate = current.with_clean_geometry_positions(&replacements)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::ApplyPreparedStraightenDepictions { update }) => {
                if update.source_revision() != current_revision {
                    return Err(SessionOperationError::MoleculeCoordinateRevisionMismatch {
                        prepared: update.source_revision(),
                        current: current_revision,
                    });
                }
                if update.source_digest() != current_digest {
                    return Err(SessionOperationError::MoleculeCoordinateDigestMismatch);
                }
                let mut replacements = Vec::with_capacity(update.molecules().len());
                for molecule in update.molecules() {
                    let object_id = molecule.molecule_id().as_str().to_owned();
                    let record = current
                        .resolve_document_object_id(molecule.molecule_id())
                        .ok_or_else(|| {
                            SessionOperationError::UnknownDocumentObject(object_id.clone())
                        })?;
                    if record.class() != TypedClass::Molecule {
                        return Err(SessionOperationError::InvalidMoleculeCoordinateTarget(
                            object_id,
                        ));
                    }
                    let source_id = record.attribute("id").ok_or_else(|| {
                        SessionOperationError::InvalidMoleculeCoordinateTarget(object_id.clone())
                    })?;
                    let molecule_id = PersistentId::new(source_id.to_owned()).map_err(|_| {
                        SessionOperationError::InvalidMoleculeCoordinateTarget(object_id)
                    })?;
                    replacements.push((
                        molecule_id,
                        molecule.expected_positions().to_vec(),
                        molecule.positions().to_vec(),
                    ));
                }
                let candidate = current.with_prepared_straightening(&replacements)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
        }
    }

    pub(crate) fn prepare_with_outcome_v1(
        &self,
        current: &TypedDocument,
        current_revision: u64,
        current_digest: &[u8; 32],
    ) -> Result<(Candidate, Option<ReactionOperationOutcomeStagingV1>), SessionOperationError> {
        match self {
            Self::V1(SessionOperationV1::CreateReactionV1(request)) => {
                let (candidate, reaction_id) = prepare_create_reaction(current, request)?;
                Ok((
                    Candidate::Changed(Box::new(candidate)),
                    Some(ReactionOperationOutcomeStagingV1::ReactionCreatedV1(
                        reaction_id,
                    )),
                ))
            }
            Self::V1(SessionOperationV1::ReplaceReactionMembersV1(request)) => {
                let (candidate, reaction_id) = prepare_replace_reaction_members(current, request)?;
                Ok((
                    Candidate::Changed(Box::new(candidate)),
                    Some(
                        ReactionOperationOutcomeStagingV1::ReactionMembershipReplacedV1(
                            reaction_id,
                        ),
                    ),
                ))
            }
            Self::V1(SessionOperationV1::DeleteReactionV1(request)) => {
                let (candidate, reaction_id) = prepare_delete_reaction(current, request)?;
                Ok((
                    Candidate::Changed(Box::new(candidate)),
                    Some(
                        ReactionOperationOutcomeStagingV1::ReactionDefinitionDeletedV1(reaction_id),
                    ),
                ))
            }
            _ => self
                .prepare(current, current_revision, current_digest)
                .map(|candidate| (candidate, None)),
        }
    }
}
