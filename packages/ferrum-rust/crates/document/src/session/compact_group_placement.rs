//! Renderer-gated session transaction for first-class compact-group placement.

use thiserror::Error;

use super::{
    AdmittedSessionTransitionRefusalV1, AuthoringCapabilityIssuerV1, DocumentFenceV1,
    DocumentObjectIdV1, DocumentSession, PersistentId, Point3V1, PreparedSessionTransitionV1,
    RevisionState, SessionOperationResultV1,
};
use crate::{
    CompactGroupAttachmentV1, CompactGroupCatalogKeyV1, TypedClass, TypedDocument,
    chemistry::{
        OrdinaryAttachmentCandidateWitnessV1, admit_candidate_ordinary_attachment_capacity_v1,
    },
};
use ferrum_chemistry::OrdinaryAttachmentProfileV1;

/// Placement intent for the future compact-group public operation.
#[derive(Clone, Debug, PartialEq)]
pub enum CompactGroupPlacementModeV1 {
    /// Create a new direct molecule root containing only the typed group.
    Free,
    /// Attach the group to one durable atom in one direct molecular root.
    Attached {
        molecule_id: DocumentObjectIdV1,
        anchor_atom_id: DocumentObjectIdV1,
    },
}

/// Fenced Rust-only request for one compact-group placement transaction.
#[derive(Clone, Debug, PartialEq)]
pub struct CompactGroupPlacementRequestV1 {
    fence: DocumentFenceV1,
    catalog_key: CompactGroupCatalogKeyV1,
    anchor: Point3V1,
    mode: CompactGroupPlacementModeV1,
}

impl CompactGroupPlacementRequestV1 {
    #[must_use]
    pub const fn new(
        fence: DocumentFenceV1,
        catalog_key: CompactGroupCatalogKeyV1,
        anchor: Point3V1,
        mode: CompactGroupPlacementModeV1,
    ) -> Self {
        Self {
            fence,
            catalog_key,
            anchor,
            mode,
        }
    }
}

/// Closed refusal taxonomy for compact-group placement preparation or redemption.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CompactGroupPlacementRefusalV1 {
    #[error("compact-group placement source is stale")]
    StaleObservation,
    #[error("compact-group placement receipt belongs to a different document session")]
    ForeignSession,
    #[error("compact-group placement source digest differs")]
    DigestMismatch,
    #[error("selected molecule is not a direct molecular root")]
    UnknownDirectMolecule,
    #[error("selected anchor is not a direct atom in the selected root")]
    UnknownAnchorAtom,
    #[error("selected anchor does not belong to the selected root")]
    AnchorNotInSelectedRoot,
    #[error("ordinary attachment capacity is unavailable")]
    AttachmentUnavailable,
    #[error("compact-group candidate exceeds a supported resource limit")]
    ResourceLimit,
    #[error("compact-group candidate cannot be prepared")]
    UnsupportedDocument,
    #[error("compact-group candidate was refused by renderer admission")]
    RendererAdmission,
}

/// Opaque one-use placement candidate. The renderer bridge is its sole redeemer.
#[derive(Debug)]
pub struct PendingCompactGroupPlacementV1 {
    issuer: AuthoringCapabilityIssuerV1,
    transition: PreparedSessionTransitionV1,
}

impl PendingCompactGroupPlacementV1 {
    #[must_use]
    pub fn is_consumed_v1(&self) -> bool {
        self.transition.is_consumed_v1()
    }
}

impl DocumentSession {
    /// Prepare one compact-group candidate without mutating history or durable IDs.
    pub fn prepare_compact_group_placement_v1(
        &mut self,
        request: &CompactGroupPlacementRequestV1,
    ) -> Result<PendingCompactGroupPlacementV1, CompactGroupPlacementRefusalV1> {
        require_fence(self, request.fence)?;
        let source = self.current_document_v1();
        let (attached_molecule, anchor_id, orientation_degrees) = match &request.mode {
            CompactGroupPlacementModeV1::Free => (None, None, 0.0),
            CompactGroupPlacementModeV1::Attached {
                molecule_id,
                anchor_atom_id,
            } => {
                let (molecule, anchor) = resolve_attached(source, molecule_id, anchor_atom_id)?;
                let orientation =
                    attached_orientation_degrees(source, molecule_id, &anchor, request.anchor)?;
                (Some(molecule), Some(anchor), orientation)
            }
        };
        let attachment = CompactGroupAttachmentV1::new(request.catalog_key, 0, orientation_degrees)
            .map_err(|_| CompactGroupPlacementRefusalV1::UnsupportedDocument)?;
        let ((group, molecule, bond, anchor), effects) = self
            .reserve_generated_ids_for_transition_v1(|mut sequences, indexed| {
                let (group, next) = sequences.reserve_compact_group(indexed)?;
                sequences = next;
                let (molecule, bond, anchor) = match (&request.mode, anchor_id.clone()) {
                    (CompactGroupPlacementModeV1::Free, _) => {
                        let (ids, next) = sequences.reserve_molecule(indexed, 0, 0)?;
                        sequences = next;
                        (ids.molecule, None, None)
                    }
                    (CompactGroupPlacementModeV1::Attached { .. }, Some(anchor)) => {
                        let (bond, next) = sequences.reserve_bond(indexed)?;
                        sequences = next;
                        (
                            attached_molecule
                                .clone()
                                .expect("attached request resolves a molecule"),
                            Some(bond),
                            Some(anchor),
                        )
                    }
                    _ => unreachable!("validated compact-group request has a matching anchor mode"),
                };
                Ok(((group, molecule, bond, anchor), sequences))
            })
            .map_err(|_| CompactGroupPlacementRefusalV1::ResourceLimit)?;
        let insertion = CompactGroupCandidateInsertionV1 {
            molecule: &molecule,
            group: &group,
            bond: bond.as_ref(),
            anchor_atom: anchor.as_ref(),
            key: request.catalog_key,
            anchor: request.anchor,
            attachment,
        };
        let candidate = build_candidate(source, insertion)?;
        if let (Some(anchor), Some(bond)) = (&anchor, &bond) {
            admit_attached_candidate(&candidate, &request.mode, anchor, &group, bond)?;
        }
        let revision = self
            .next_revision_v1()
            .ok_or(CompactGroupPlacementRefusalV1::UnsupportedDocument)?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(|_| CompactGroupPlacementRefusalV1::UnsupportedDocument)?;
        let transition = self
            .prepare_changed_session_transition_v1(
                request.fence.revision(),
                request.fence.digest(),
                state,
                effects,
            )
            .map_err(map_prepare_error)?;
        Ok(PendingCompactGroupPlacementV1 {
            issuer: self.authoring_capability_issuer_v1(),
            transition,
        })
    }

    /// Commit one renderer-admitted compact-group candidate as one history transition.
    pub fn commit_compact_group_placement_v1(
        &mut self,
        pending: &mut PendingCompactGroupPlacementV1,
    ) -> Result<SessionOperationResultV1, CompactGroupPlacementRefusalV1> {
        if !pending
            .issuer
            .same_issuer(&self.authoring_capability_issuer_v1())
        {
            return Err(CompactGroupPlacementRefusalV1::ForeignSession);
        }
        self.commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(map_commit_error)
    }
}

fn map_prepare_error(error: super::DocumentSessionError) -> CompactGroupPlacementRefusalV1 {
    match error {
        super::DocumentSessionError::RendererAdmission => {
            CompactGroupPlacementRefusalV1::RendererAdmission
        }
        _ => CompactGroupPlacementRefusalV1::UnsupportedDocument,
    }
}

fn map_commit_error(error: AdmittedSessionTransitionRefusalV1) -> CompactGroupPlacementRefusalV1 {
    match error {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            CompactGroupPlacementRefusalV1::ForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Replayed => {
            CompactGroupPlacementRefusalV1::StaleObservation
        }
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            CompactGroupPlacementRefusalV1::StaleObservation
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            CompactGroupPlacementRefusalV1::RendererAdmission
        }
        AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            CompactGroupPlacementRefusalV1::UnsupportedDocument
        }
        AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
            CompactGroupPlacementRefusalV1::ResourceLimit
        }
    }
}

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), CompactGroupPlacementRefusalV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| CompactGroupPlacementRefusalV1::UnsupportedDocument)?;
    if snapshot.revision() != fence.revision() {
        return Err(CompactGroupPlacementRefusalV1::StaleObservation);
    }
    if *snapshot.digest() != fence.digest() {
        return Err(CompactGroupPlacementRefusalV1::DigestMismatch);
    }
    Ok(())
}

fn resolve_attached(
    document: &TypedDocument,
    molecule_id: &DocumentObjectIdV1,
    anchor_id: &DocumentObjectIdV1,
) -> Result<(PersistentId, PersistentId), CompactGroupPlacementRefusalV1> {
    let molecule = document
        .resolve_document_object_id(molecule_id)
        .filter(|record| {
            record.class() == TypedClass::Molecule && record.path().components().len() == 1
        })
        .ok_or(CompactGroupPlacementRefusalV1::UnknownDirectMolecule)?;
    let anchor = document
        .resolve_document_object_id(anchor_id)
        .filter(|record| record.class() == TypedClass::Atom)
        .ok_or(CompactGroupPlacementRefusalV1::UnknownAnchorAtom)?;
    if anchor.path().components().len() != 2
        || anchor.path().components().first() != molecule.path().components().first()
    {
        return Err(CompactGroupPlacementRefusalV1::AnchorNotInSelectedRoot);
    }
    let molecule = PersistentId::new(
        molecule
            .attribute("id")
            .ok_or(CompactGroupPlacementRefusalV1::UnsupportedDocument)?
            .to_owned(),
    )
    .map_err(|_| CompactGroupPlacementRefusalV1::UnsupportedDocument)?;
    let anchor = PersistentId::new(
        anchor
            .attribute("id")
            .ok_or(CompactGroupPlacementRefusalV1::UnsupportedDocument)?
            .to_owned(),
    )
    .map_err(|_| CompactGroupPlacementRefusalV1::UnsupportedDocument)?;
    Ok((molecule, anchor))
}

fn attached_orientation_degrees(
    document: &TypedDocument,
    molecule_id: &DocumentObjectIdV1,
    anchor_id: &PersistentId,
    placement_anchor: Point3V1,
) -> Result<f64, CompactGroupPlacementRefusalV1> {
    let molecule = document
        .core_molecule(molecule_id)
        .map_err(|_| CompactGroupPlacementRefusalV1::UnsupportedDocument)?
        .ok_or(CompactGroupPlacementRefusalV1::UnknownDirectMolecule)?;
    let atom = molecule
        .atoms()
        .iter()
        .find(|atom| {
            atom.source_id()
                .is_some_and(|id| id.as_str() == anchor_id.as_str())
        })
        .ok_or(CompactGroupPlacementRefusalV1::UnknownAnchorAtom)?;
    let position = atom.position();
    let dx = placement_anchor.x() - position.x();
    let dy = placement_anchor.y() - position.y();
    if dx == 0.0 && dy == 0.0 {
        return Ok(0.0);
    }
    Ok(dy.atan2(dx).to_degrees().rem_euclid(360.0))
}

/// Complete generated identity and authored state for one compact-group candidate.
struct CompactGroupCandidateInsertionV1<'a> {
    molecule: &'a PersistentId,
    group: &'a PersistentId,
    bond: Option<&'a PersistentId>,
    anchor_atom: Option<&'a PersistentId>,
    key: CompactGroupCatalogKeyV1,
    anchor: Point3V1,
    attachment: CompactGroupAttachmentV1,
}

fn build_candidate(
    source: &TypedDocument,
    insertion: CompactGroupCandidateInsertionV1<'_>,
) -> Result<TypedDocument, CompactGroupPlacementRefusalV1> {
    let candidate = if source.indexed().resolve_id(insertion.molecule).is_some() {
        source.with_insert_compact_group(
            insertion.molecule,
            insertion.group,
            insertion.key,
            insertion.anchor,
            insertion.attachment,
        )
    } else {
        source.with_insert_free_compact_group(
            insertion.molecule,
            insertion.group,
            insertion.key,
            insertion.anchor,
            insertion.attachment,
        )
    }
    .map_err(|_| CompactGroupPlacementRefusalV1::UnsupportedDocument)?;
    match (insertion.bond, insertion.anchor_atom) {
        (Some(bond), Some(atom)) => candidate
            .with_insert_compact_group_bond(insertion.molecule, bond, atom, insertion.group)
            .map_err(|_| CompactGroupPlacementRefusalV1::UnsupportedDocument),
        (None, None) => Ok(candidate),
        _ => Err(CompactGroupPlacementRefusalV1::UnsupportedDocument),
    }
}

fn admit_attached_candidate(
    candidate: &TypedDocument,
    mode: &CompactGroupPlacementModeV1,
    anchor: &PersistentId,
    group: &PersistentId,
    bond: &PersistentId,
) -> Result<(), CompactGroupPlacementRefusalV1> {
    let CompactGroupPlacementModeV1::Attached { molecule_id, .. } = mode else {
        return Ok(());
    };
    let molecule = candidate
        .core_molecule(molecule_id)
        .map_err(|_| CompactGroupPlacementRefusalV1::UnsupportedDocument)?
        .ok_or(CompactGroupPlacementRefusalV1::UnknownDirectMolecule)?;
    let anchor_identity = molecule
        .atoms()
        .iter()
        .find(|atom| {
            atom.source_id()
                .is_some_and(|id| id.as_str() == anchor.as_str())
        })
        .ok_or(CompactGroupPlacementRefusalV1::UnsupportedDocument)?
        .identity()
        .clone();
    let group_identity = molecule
        .groups()
        .iter()
        .find(|entry| {
            entry
                .source_id()
                .is_some_and(|id| id.as_str() == group.as_str())
        })
        .ok_or(CompactGroupPlacementRefusalV1::UnsupportedDocument)?
        .identity()
        .clone();
    let bond_identity = molecule
        .bonds()
        .iter()
        .find(|entry| {
            entry
                .source_id()
                .is_some_and(|id| id.as_str() == bond.as_str())
        })
        .ok_or(CompactGroupPlacementRefusalV1::UnsupportedDocument)?
        .identity()
        .clone();
    let witness = OrdinaryAttachmentCandidateWitnessV1::new(
        anchor_identity,
        group_identity,
        bond_identity,
        OrdinaryAttachmentProfileV1::NormalSingle,
    );
    match admit_candidate_ordinary_attachment_capacity_v1(&molecule, &witness)
        .map_err(|_| CompactGroupPlacementRefusalV1::UnsupportedDocument)?
    {
        crate::chemistry::DocumentOrdinaryAttachmentAvailabilityV1::Available => Ok(()),
        crate::chemistry::DocumentOrdinaryAttachmentAvailabilityV1::Unavailable {
            reason, ..
        } => match reason {
            crate::chemistry::DocumentOrdinaryAttachmentReasonV1::ResourceLimit => {
                Err(CompactGroupPlacementRefusalV1::ResourceLimit)
            }
            _ => Err(CompactGroupPlacementRefusalV1::AttachmentUnavailable),
        },
    }
}
