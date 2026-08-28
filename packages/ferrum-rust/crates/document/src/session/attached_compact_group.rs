//! Opaque reviewed compact-group authoring transaction.

use thiserror::Error;

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentFenceV1, DocumentObjectIdV1, DocumentSession,
    PersistentId, PreparedSessionTransitionV1, RevisionState, SessionDocumentObservationV1,
    SessionOperationResultV1,
};
use crate::{
    AttachCompactGroupV1, AttachedCompactGroupErrorV1, AuthoringCapabilityIssuerV1,
    CompactGroupCatalogKeyV1, Point3V1,
    attached_compact_group_v1::attached_compact_group_candidate_from_resolved_pose_v1,
    compact_group_materialization_v1::TypedCompactGroupMaterializationRequestV1,
};
use ferrum_chemistry::{
    OrdinaryAttachmentAnchorV1, OrdinaryAttachmentBondOrderV1, OrdinaryAttachmentCapacityOutcomeV1,
    OrdinaryAttachmentProfileV1, admit_ordinary_attachment_capacity_v1,
};
use ferrum_document_model::materialization_recipe_v1;
use ferrum_render::{
    AcceptedRenderOverlayRequestV1, AcceptedRenderOverlayTargetV1, DepictionProfileV1,
    DocumentPrecommitOverlayV1, RenderPoint, ResolvedAttachedCompactGroupPoseV1,
    resolve_attached_compact_group_pose_v2,
};

/// Durable pair selected for one attached compact-group operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachedCompactGroupTargetV1 {
    molecule_id: DocumentObjectIdV1,
    anchor_atom_id: DocumentObjectIdV1,
}

impl AttachedCompactGroupTargetV1 {
    /// Construct one opaque molecule-plus-direct-anchor target.
    #[must_use]
    pub fn new(molecule_id: DocumentObjectIdV1, anchor_atom_id: DocumentObjectIdV1) -> Self {
        Self {
            molecule_id,
            anchor_atom_id,
        }
    }

    /// Return the selected direct-root molecule identifier.
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }

    /// Return the selected direct anchor atom identifier.
    #[must_use]
    pub const fn anchor_atom_id(&self) -> &DocumentObjectIdV1 {
        &self.anchor_atom_id
    }
}

/// Opaque session-affine, one-use pending compact-group attachment.
pub struct PendingAttachedCompactGroupV1 {
    session_issuer: AuthoringCapabilityIssuerV1,
    fence: DocumentFenceV1,
    focus_object_id: DocumentObjectIdV1,
    compact_group_object_id: DocumentObjectIdV1,
    transition: PreparedSessionTransitionV1,
    precommit_overlay: DocumentPrecommitOverlayV1,
}

/// Authoritative durable facts from one accepted compact-group attachment.
#[derive(Clone, Debug, PartialEq)]
pub struct AttachedCompactGroupCommitResultV1 {
    result: SessionOperationResultV1,
    focus_object_id: DocumentObjectIdV1,
    compact_group_object_id: DocumentObjectIdV1,
}

impl AttachedCompactGroupCommitResultV1 {
    fn new(
        result: SessionOperationResultV1,
        focus_object_id: DocumentObjectIdV1,
        compact_group_object_id: DocumentObjectIdV1,
    ) -> Self {
        Self {
            result,
            focus_object_id,
            compact_group_object_id,
        }
    }

    /// Return the complete post-commit observation.
    #[must_use]
    pub fn observation(&self) -> &SessionDocumentObservationV1 {
        self.result.observation()
    }

    /// Return the selected direct anchor atom.
    #[must_use]
    pub const fn focus_object_id(&self) -> &DocumentObjectIdV1 {
        &self.focus_object_id
    }

    /// Return the newly authored compact-group object.
    #[must_use]
    pub const fn compact_group_object_id(&self) -> &DocumentObjectIdV1 {
        &self.compact_group_object_id
    }
}

impl std::fmt::Debug for PendingAttachedCompactGroupV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingAttachedCompactGroupV1")
            .field("revision", &self.fence.revision())
            .field("is_consumed", &self.transition.is_consumed_v1())
            .finish()
    }
}

impl PendingAttachedCompactGroupV1 {
    #[must_use]
    pub fn precommit_overlay_v1(&self) -> Option<&DocumentPrecommitOverlayV1> {
        (!self.transition.is_consumed_v1()).then_some(&self.precommit_overlay)
    }
}

/// Closed refusal vocabulary for reviewed compact-group attachment.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum AttachedCompactGroupSessionErrorV1 {
    #[error("compact-group attachment revision is stale")]
    StaleRevision,
    #[error("compact-group attachment digest is stale")]
    StaleDigest,
    #[error("compact-group attachment belongs to another session")]
    ForeignSession,
    #[error("compact-group attachment was already consumed")]
    Consumed,
    #[error("compact-group attachment molecule is unknown or not a direct root")]
    UnknownMolecule,
    #[error("compact-group attachment anchor is unknown or not a direct atom")]
    UnknownAnchor,
    #[error("compact-group attachment anchor belongs to another molecule")]
    ForeignTarget,
    #[error("compact-group attachment pose is invalid")]
    InvalidPose,
    #[error("compact-group attachment candidate could not be admitted")]
    CandidateAdmission,
    #[error("compact-group attachment candidate could not be rendered completely")]
    RendererAdmission,
    #[error("compact-group attachment session conflict")]
    SessionConflict,
}

/// Closed categories for the read-only attached compact-group availability observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachedCompactGroupAvailabilityCategoryV1 {
    /// The current fenced direct atom has sufficient ordinary-single capacity.
    Available,
    /// The caller's revision no longer identifies the current session state.
    StaleRevision,
    /// The caller's digest does not match the current session state.
    StaleDigest,
    /// The durable selection does not identify a current direct-root molecule.
    UnknownMolecule,
    /// The durable selection does not identify a current direct atom.
    UnknownAnchor,
    /// The durable anchor belongs to another current direct-root molecule.
    ForeignTarget,
    /// The bounded immutable candidate proof could not be admitted.
    CandidateAdmission,
    /// The immutable session observation could not be constructed.
    SessionConflict,
}

/// Immutable facts for enabling the attached compact-group action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachedCompactGroupAvailabilityV1 {
    revision: u64,
    digest: [u8; 32],
    anchor_object_id: DocumentObjectIdV1,
    catalog_key: CompactGroupCatalogKeyV1,
    category: AttachedCompactGroupAvailabilityCategoryV1,
}

impl AttachedCompactGroupAvailabilityV1 {
    fn new(
        revision: u64,
        digest: [u8; 32],
        anchor_object_id: DocumentObjectIdV1,
        catalog_key: CompactGroupCatalogKeyV1,
        category: AttachedCompactGroupAvailabilityCategoryV1,
    ) -> Self {
        Self {
            revision,
            digest,
            anchor_object_id,
            catalog_key,
            category,
        }
    }

    /// Return the current revision observed for this availability result.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Return the current digest observed for this availability result.
    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    /// Return the selected molecule-scoped anchor address supplied to the observation.
    #[must_use]
    pub const fn anchor_object_id(&self) -> &DocumentObjectIdV1 {
        &self.anchor_object_id
    }

    /// Return the reviewed catalog key observed for this availability result.
    #[must_use]
    pub const fn catalog_key(&self) -> CompactGroupCatalogKeyV1 {
        self.catalog_key
    }

    /// Return the stable availability category.
    #[must_use]
    pub const fn category(&self) -> AttachedCompactGroupAvailabilityCategoryV1 {
        self.category
    }

    /// Return whether the current read-only facts permit action enablement.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        matches!(
            self.category,
            AttachedCompactGroupAvailabilityCategoryV1::Available
        )
    }
}

impl DocumentSession {
    /// Observe whether one fenced molecule-plus-direct-atom target can accept a reviewed group.
    ///
    /// This advisory check allocates no durable identifiers and creates neither a
    /// pending capability nor a renderer candidate. Begin and commit repeat the
    /// authoritative checks with the actual finite release point.
    #[must_use]
    pub fn observe_attach_compact_group_availability_v1(
        &self,
        fence: DocumentFenceV1,
        target: AttachedCompactGroupTargetV1,
        catalog_key: CompactGroupCatalogKeyV1,
    ) -> AttachedCompactGroupAvailabilityV1 {
        let revision = self.current_revision_v1();
        let digest = self.current_digest_v1();
        let category = if revision != fence.revision() {
            AttachedCompactGroupAvailabilityCategoryV1::StaleRevision
        } else if digest != fence.digest() {
            AttachedCompactGroupAvailabilityCategoryV1::StaleDigest
        } else if let Ok(observation) = self.document_observation() {
            match resolve_anchor(&observation, &target) {
                Ok(resolved) => availability_category(resolved, catalog_key),
                Err(AttachedCompactGroupSessionErrorV1::UnknownMolecule) => {
                    AttachedCompactGroupAvailabilityCategoryV1::UnknownMolecule
                }
                Err(AttachedCompactGroupSessionErrorV1::ForeignTarget) => {
                    AttachedCompactGroupAvailabilityCategoryV1::ForeignTarget
                }
                Err(_) => AttachedCompactGroupAvailabilityCategoryV1::UnknownAnchor,
            }
        } else {
            AttachedCompactGroupAvailabilityCategoryV1::SessionConflict
        };
        AttachedCompactGroupAvailabilityV1::new(
            revision,
            digest,
            target.anchor_atom_id().clone(),
            catalog_key,
            category,
        )
    }

    /// Prepare exactly one reviewed compact group from a selected target and finite release point.
    pub fn prepare_attach_compact_group_v1(
        &mut self,
        fence: DocumentFenceV1,
        target: AttachedCompactGroupTargetV1,
        request: AttachCompactGroupV1,
    ) -> Result<PendingAttachedCompactGroupV1, AttachedCompactGroupSessionErrorV1> {
        require_fence(self, fence)?;
        let observation = self
            .document_observation()
            .map_err(|_| AttachedCompactGroupSessionErrorV1::SessionConflict)?;
        let resolved = resolve_anchor(&observation, &target)?;
        let renderer_pose = resolve_renderer_admitted_pose(&observation, &resolved, request)?;
        let pose = attached_compact_group_candidate_from_resolved_pose_v1(
            request.catalog_key(),
            Point3V1::new(
                renderer_pose.anchor().x(),
                renderer_pose.anchor().y(),
                resolved.position.z(),
            )
            .map_err(|_| AttachedCompactGroupSessionErrorV1::RendererAdmission)?,
            renderer_pose.orientation_degrees(),
        )
        .map_err(map_core_error)?;
        let catalog_key = pose.catalog_key();
        let validation_group_id = validation_identifier(self.current_document_v1(), "group")?;
        let validation_bond_id = validation_identifier(self.current_document_v1(), "bond")?;
        let capacity_document = self
            .current_document_v1()
            .with_attach_compact_group_v1(
                &resolved.molecule_id,
                &resolved.anchor_id,
                &validation_group_id,
                &validation_bond_id,
                pose,
            )
            .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
        require_materialized_compact_group_capacity(
            &capacity_document,
            &resolved.molecule_id,
            &validation_group_id,
            catalog_key,
            &resolved,
        )?;
        let ((group_id, bond_id), effects) = self
            .reserve_generated_ids_for_transition_v1(|ids, indexed| {
                let (group_id, ids) = ids.reserve_group(indexed)?;
                let (bond_id, ids) = ids.reserve_bond(indexed)?;
                Ok(((group_id, bond_id), ids))
            })
            .map_err(|_| AttachedCompactGroupSessionErrorV1::SessionConflict)?;
        let document = self
            .current_document_v1()
            .with_attach_compact_group_v1(
                &resolved.molecule_id,
                &resolved.anchor_id,
                &group_id,
                &bond_id,
                pose,
            )
            .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
        let compact_group_object_id = document
            .document_object_id_for_source_id_v1(&group_id)
            .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?
            .ok_or(AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
        let bond_object_id = document
            .document_object_id_for_source_id_v1(&bond_id)
            .map_err(|_| AttachedCompactGroupSessionErrorV1::RendererAdmission)?
            .ok_or(AttachedCompactGroupSessionErrorV1::RendererAdmission)?;
        let revision = self
            .next_revision_v1()
            .ok_or(AttachedCompactGroupSessionErrorV1::SessionConflict)?;
        let state = RevisionState::from_document(revision, document)
            .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
        let mut transition = self
            .prepare_changed_session_transition_v1(fence.revision(), fence.digest(), state, effects)
            .map_err(map_prepare_error)?;
        let overlay_request =
            AcceptedRenderOverlayRequestV1::new(vec![AcceptedRenderOverlayTargetV1::bond(
                bond_object_id,
            )])
            .map_err(|_| AttachedCompactGroupSessionErrorV1::RendererAdmission)?;
        let overlay = transition
            .renderer_precommit_overlay_v1(&overlay_request)
            .map_err(|_| AttachedCompactGroupSessionErrorV1::RendererAdmission)?;
        transition
            .install_precommit_overlay_v1(overlay.clone())
            .map_err(|_| AttachedCompactGroupSessionErrorV1::RendererAdmission)?;
        Ok(PendingAttachedCompactGroupV1 {
            session_issuer: self.authoring_capability_issuer.clone(),
            fence,
            focus_object_id: target.anchor_atom_id().clone(),
            compact_group_object_id,
            transition,
            precommit_overlay: overlay,
        })
    }

    /// Commit one already-admitted compact-group candidate as one history transition.
    pub fn commit_attach_compact_group_v1(
        &mut self,
        pending: &mut PendingAttachedCompactGroupV1,
    ) -> Result<AttachedCompactGroupCommitResultV1, AttachedCompactGroupSessionErrorV1> {
        if pending.transition.is_consumed_v1() {
            return Err(AttachedCompactGroupSessionErrorV1::Consumed);
        }
        if !pending
            .session_issuer
            .same_issuer(&self.authoring_capability_issuer)
        {
            return Err(AttachedCompactGroupSessionErrorV1::ForeignSession);
        }
        let result = self
            .commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(map_commit_error)?;
        Ok(AttachedCompactGroupCommitResultV1::new(
            result,
            pending.focus_object_id.clone(),
            pending.compact_group_object_id.clone(),
        ))
    }

    /// Cancel one pending compact-group attachment without consuming document state or IDs.
    pub fn cancel_attach_compact_group_v1(
        &mut self,
        pending: &mut PendingAttachedCompactGroupV1,
    ) -> Result<(), AttachedCompactGroupSessionErrorV1> {
        if !pending
            .session_issuer
            .same_issuer(&self.authoring_capability_issuer)
        {
            return Err(AttachedCompactGroupSessionErrorV1::ForeignSession);
        }
        if pending.transition.is_consumed_v1() {
            return Err(AttachedCompactGroupSessionErrorV1::Consumed);
        }
        self.cancel_session_operation_transition_v1(&mut pending.transition)
            .map_err(map_commit_error)
    }
}

fn availability_category(
    resolved: ResolvedAnchorV1,
    catalog_key: CompactGroupCatalogKeyV1,
) -> AttachedCompactGroupAvailabilityCategoryV1 {
    if require_attached_compact_group_chemistry_support_v1(catalog_key).is_err() {
        return AttachedCompactGroupAvailabilityCategoryV1::CandidateAdmission;
    }
    match require_direct_anchor_attachment_capacity(&resolved) {
        Ok(()) => AttachedCompactGroupAvailabilityCategoryV1::Available,
        Err(_) => AttachedCompactGroupAvailabilityCategoryV1::CandidateAdmission,
    }
}

/// Admit the closed-catalog chemistry prerequisites for attached-group authoring.
///
/// A release point is intentionally absent: renderer admission owns durable compact-group pose.
fn require_attached_compact_group_chemistry_support_v1(
    catalog_key: CompactGroupCatalogKeyV1,
) -> Result<(), AttachedCompactGroupSessionErrorV1> {
    ferrum_document_model::supports_attached_compact_group_authoring_v1(catalog_key)
        .then_some(())
        .and_then(|()| materialization_recipe_v1(catalog_key).map(|_| ()))
        .ok_or(AttachedCompactGroupSessionErrorV1::CandidateAdmission)
}

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), AttachedCompactGroupSessionErrorV1> {
    if session.current_revision_v1() != fence.revision() {
        return Err(AttachedCompactGroupSessionErrorV1::StaleRevision);
    }
    if session.current_digest_v1() != fence.digest() {
        return Err(AttachedCompactGroupSessionErrorV1::StaleDigest);
    }
    Ok(())
}

fn map_core_error(error: AttachedCompactGroupErrorV1) -> AttachedCompactGroupSessionErrorV1 {
    match error {
        AttachedCompactGroupErrorV1::InvalidPose => AttachedCompactGroupSessionErrorV1::InvalidPose,
        AttachedCompactGroupErrorV1::UnsupportedCatalogKey => {
            AttachedCompactGroupSessionErrorV1::SessionConflict
        }
    }
}

fn map_prepare_error(error: super::DocumentSessionError) -> AttachedCompactGroupSessionErrorV1 {
    match error {
        super::DocumentSessionError::RendererAdmission => {
            AttachedCompactGroupSessionErrorV1::RendererAdmission
        }
        _ => AttachedCompactGroupSessionErrorV1::SessionConflict,
    }
}

fn map_commit_error(
    error: AdmittedSessionTransitionRefusalV1,
) -> AttachedCompactGroupSessionErrorV1 {
    match error {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            AttachedCompactGroupSessionErrorV1::ForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Consumed => {
            AttachedCompactGroupSessionErrorV1::Consumed
        }
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            AttachedCompactGroupSessionErrorV1::StaleRevision
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            AttachedCompactGroupSessionErrorV1::RendererAdmission
        }
        AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            AttachedCompactGroupSessionErrorV1::SessionConflict
        }
    }
}

/// Prove the immutable recipe and admit its ordinary exterior attachment at the direct anchor.
///
/// The materialized compact group may contain charged atoms (for example, NO2), so neutral
/// whole-molecule capacity is not an appropriate proof. The recipe validates the closed
/// topology, while the dedicated ordinary-attachment profile validates the one new bond at
/// the uncharged direct anchor.
fn require_materialized_compact_group_capacity(
    document: &crate::TypedDocument,
    molecule_id: &PersistentId,
    group_id: &PersistentId,
    catalog_key: CompactGroupCatalogKeyV1,
    resolved: &ResolvedAnchorV1,
) -> Result<(), AttachedCompactGroupSessionErrorV1> {
    let recipe = materialization_recipe_v1(catalog_key)
        .ok_or(AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
    let atom_ids = recipe
        .atoms
        .iter()
        .enumerate()
        .map(|(index, _)| validation_identifier(document, &format!("capacity-probe-atom-{index}")))
        .collect::<Result<Vec<_>, _>>()?;
    let bond_ids = recipe
        .bonds
        .iter()
        .enumerate()
        .map(|(index, _)| validation_identifier(document, &format!("capacity-probe-bond-{index}")))
        .collect::<Result<Vec<_>, _>>()?;
    let plan = document
        .prepare_compact_group_materialization_v1(TypedCompactGroupMaterializationRequestV1::new(
            molecule_id.clone(),
            group_id.clone(),
            atom_ids,
            bond_ids,
        ))
        .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
    document
        .materialize_compact_group_v1(&plan)
        .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
    require_direct_anchor_attachment_capacity(resolved)
}

/// Admit the one exterior normal bond using only direct-anchor chemistry facts.
fn require_direct_anchor_attachment_capacity(
    resolved: &ResolvedAnchorV1,
) -> Result<(), AttachedCompactGroupSessionErrorV1> {
    match admit_ordinary_attachment_capacity_v1(
        OrdinaryAttachmentProfileV1::NormalSingle,
        OrdinaryAttachmentAnchorV1 {
            element: &resolved.element,
            formal_charge: resolved.formal_charge,
            explicit_hydrogens: resolved.explicit_hydrogens,
            authored_valence: resolved.valence,
            multiplicity: resolved.multiplicity,
            free_sites: None,
            incident_bond_orders: &resolved.incident_bond_orders,
        },
    ) {
        OrdinaryAttachmentCapacityOutcomeV1::Admitted(_) => Ok(()),
        OrdinaryAttachmentCapacityOutcomeV1::Unavailable { .. } => {
            Err(AttachedCompactGroupSessionErrorV1::CandidateAdmission)
        }
    }
}

fn validation_identifier(
    document: &crate::TypedDocument,
    kind: &str,
) -> Result<PersistentId, AttachedCompactGroupSessionErrorV1> {
    for sequence in 0_u64..1_000 {
        let identifier = PersistentId::new(format!(
            "ferrum-attached-compact-group-validation-{kind}-{sequence}"
        ))
        .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
        if document.indexed().resolve_id(&identifier).is_none() {
            return Ok(identifier);
        }
    }
    Err(AttachedCompactGroupSessionErrorV1::CandidateAdmission)
}

struct ResolvedAnchorV1 {
    molecule_id: PersistentId,
    anchor_id: PersistentId,
    position: Point3V1,
    element: String,
    formal_charge: Option<i32>,
    explicit_hydrogens: Option<u16>,
    valence: Option<u16>,
    multiplicity: Option<u16>,
    incident_bond_orders: Vec<OrdinaryAttachmentBondOrderV1>,
    atom: crate::AtomProjectionV1,
}

/// Resolve the only durable attachment pose from the current fenced render observation.
///
/// Persistent identifier allocation deliberately follows this renderer-owned geometry step.
fn resolve_renderer_admitted_pose(
    observation: &SessionDocumentObservationV1,
    resolved: &ResolvedAnchorV1,
    request: AttachCompactGroupV1,
) -> Result<ResolvedAttachedCompactGroupPoseV1, AttachedCompactGroupSessionErrorV1> {
    let profile = DepictionProfileV1::ferrum_default();
    let raw_release = RenderPoint::new(request.release().x(), request.release().y())
        .map_err(|_| AttachedCompactGroupSessionErrorV1::RendererAdmission)?;
    resolve_attached_compact_group_pose_v2(
        observation.projection(),
        &resolved.atom,
        &profile,
        request.catalog_key(),
        raw_release,
    )
    .map_err(|_| AttachedCompactGroupSessionErrorV1::RendererAdmission)
}

fn resolve_anchor(
    observation: &SessionDocumentObservationV1,
    target: &AttachedCompactGroupTargetV1,
) -> Result<ResolvedAnchorV1, AttachedCompactGroupSessionErrorV1> {
    let molecule_id = target.molecule_id();
    let anchor = target.anchor_atom_id();
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.document_object_id() == molecule_id)
        .ok_or(AttachedCompactGroupSessionErrorV1::UnknownMolecule)?;
    let atom = match molecule
        .atoms()
        .iter()
        .find(|atom| atom.document_object_id() == anchor)
    {
        Some(atom) => atom,
        None => {
            let exists_elsewhere = observation
                .projection()
                .molecules()
                .iter()
                .any(|candidate| {
                    candidate.document_object_id() != molecule_id
                        && candidate
                            .atoms()
                            .iter()
                            .any(|atom| atom.document_object_id() == anchor)
                });
            return Err(if exists_elsewhere {
                AttachedCompactGroupSessionErrorV1::ForeignTarget
            } else {
                AttachedCompactGroupSessionErrorV1::UnknownAnchor
            });
        }
    };
    let molecule_id = molecule
        .source_id()
        .and_then(|id| PersistentId::new(id.to_owned()).ok())
        .ok_or(AttachedCompactGroupSessionErrorV1::UnknownAnchor)?;
    let anchor_id = atom
        .source_id()
        .and_then(|id| PersistentId::new(id.to_owned()).ok())
        .ok_or(AttachedCompactGroupSessionErrorV1::UnknownAnchor)?;
    let element = atom
        .element()
        .ok_or(AttachedCompactGroupSessionErrorV1::UnknownAnchor)?;
    let incident_bond_orders = molecule
        .bonds()
        .iter()
        .filter(|bond| {
            bond.start().source_id() == Some(anchor_id.as_str())
                || bond.end().source_id() == Some(anchor_id.as_str())
        })
        .map(|bond| match bond.source_type() {
            Some("n1") => OrdinaryAttachmentBondOrderV1::Single,
            Some("n2") => OrdinaryAttachmentBondOrderV1::Double,
            Some("n3") => OrdinaryAttachmentBondOrderV1::Triple,
            Some("aromatic") => OrdinaryAttachmentBondOrderV1::Aromatic,
            _ => OrdinaryAttachmentBondOrderV1::Unsupported,
        })
        .collect();
    Ok(ResolvedAnchorV1 {
        molecule_id,
        anchor_id,
        position: atom.position(),
        element: element.to_owned(),
        formal_charge: atom.formal_charge(),
        explicit_hydrogens: atom.explicit_hydrogens(),
        valence: atom.valence(),
        multiplicity: atom.multiplicity(),
        incident_bond_orders,
        atom: atom.clone(),
    })
}

#[cfg(test)]
#[path = "attached_compact_group_tests.rs"]
pub(super) mod tests;

#[cfg(test)]
#[path = "attached_compact_group_recipe_semantics_tests.rs"]
mod recipe_semantics_tests;

#[cfg(test)]
#[path = "attached_acyl_chloride_tests.rs"]
mod acyl_chloride_tests;

#[cfg(test)]
#[path = "attached_phenyl_tests.rs"]
mod phenyl_tests;
