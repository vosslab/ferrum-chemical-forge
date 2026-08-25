//! Opaque methyl-only compact-group authoring transaction.

use thiserror::Error;

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentFenceV1, DocumentObjectIdV1, DocumentSession,
    PersistentId, PreparedSessionTransitionV1, RevisionState, SessionDocumentObservationV1,
    SessionOperationResultV1,
};
use crate::{
    AttachedCompactGroupErrorV1, AttachedCompactGroupReleaseV1, AuthoringCapabilityIssuerV1,
    Point3V1,
    attached_compact_group_v1::attached_methyl_candidate_v1,
    compact_group_materialization_v1::TypedCompactGroupMaterializationRequestV1,
    DocumentBondCapacityOutcomeV1,
};
use ferrum_render::{AcceptedRenderOverlayRequestV1, AcceptedRenderOverlayTargetV1, DocumentPrecommitOverlayV1};

/// Opaque session-affine, one-use pending methyl compact-group attachment.
pub struct PendingAttachedCompactGroupV1 {
    session_issuer: AuthoringCapabilityIssuerV1,
    fence: DocumentFenceV1,
    focus_object_id: DocumentObjectIdV1,
    compact_group_object_id: DocumentObjectIdV1,
    transition: PreparedSessionTransitionV1,
    precommit_overlay: DocumentPrecommitOverlayV1,
}

/// Authoritative durable facts from one accepted methyl compact-group attachment.
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
        formatter.debug_struct("PendingAttachedCompactGroupV1")
            .field("revision", &self.fence.revision())
            .field("is_resolved", &self.transition.is_consumed_v1())
            .finish()
    }
}

impl PendingAttachedCompactGroupV1 {
    #[must_use]
    pub fn precommit_overlay_v1(&self) -> Option<&DocumentPrecommitOverlayV1> {
        (!self.transition.is_consumed_v1()).then_some(&self.precommit_overlay)
    }
}

/// Closed refusal vocabulary for methyl-only compact-group attachment.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum AttachedCompactGroupSessionErrorV1 {
    #[error("compact-group attachment revision is stale")]
    StaleRevision,
    #[error("compact-group attachment digest is stale")]
    StaleDigest,
    #[error("compact-group attachment belongs to another session")]
    ForeignSession,
    #[error("compact-group attachment is retired")]
    Retired,
    #[error("compact-group attachment anchor is unknown or not a direct atom")]
    UnknownAnchor,
    #[error("compact-group attachment pose is invalid")]
    InvalidPose,
    #[error("compact-group attachment candidate could not be admitted")]
    CandidateAdmission,
    #[error("compact-group attachment candidate could not be rendered completely")]
    RendererAdmission,
    #[error("compact-group attachment session conflict")]
    SessionConflict,
}

/// Closed categories for the read-only attached-methyl availability observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachedCompactGroupAvailabilityCategoryV1 {
    /// The current fenced direct atom has sufficient ordinary-single capacity.
    Available,
    /// The caller's revision no longer identifies the current session state.
    StaleRevision,
    /// The caller's digest does not match the current session state.
    StaleDigest,
    /// The durable selection does not identify a current direct atom.
    UnknownAnchor,
    /// The bounded immutable candidate proof could not be admitted.
    CandidateAdmission,
    /// The immutable session observation could not be constructed.
    SessionConflict,
}

/// Immutable facts for enabling the attached-methyl compact-group action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachedCompactGroupAvailabilityV1 {
    revision: u64,
    digest: [u8; 32],
    anchor_object_id: DocumentObjectIdV1,
    category: AttachedCompactGroupAvailabilityCategoryV1,
}

impl AttachedCompactGroupAvailabilityV1 {
    fn new(
        revision: u64,
        digest: [u8; 32],
        anchor_object_id: DocumentObjectIdV1,
        category: AttachedCompactGroupAvailabilityCategoryV1,
    ) -> Self {
        Self {
            revision,
            digest,
            anchor_object_id,
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

    /// Return the selected durable atom address supplied to the observation.
    #[must_use]
    pub const fn anchor_object_id(&self) -> &DocumentObjectIdV1 {
        &self.anchor_object_id
    }

    /// Return the stable availability category.
    #[must_use]
    pub const fn category(&self) -> AttachedCompactGroupAvailabilityCategoryV1 {
        self.category
    }

    /// Return whether the current read-only facts permit action enablement.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        matches!(self.category, AttachedCompactGroupAvailabilityCategoryV1::Available)
    }
}

impl DocumentSession {
    /// Observe whether one fenced direct atom can currently accept attached methyl.
    ///
    /// This advisory check allocates no durable identifiers and creates neither a
    /// pending capability nor a renderer candidate. Begin and commit repeat the
    /// authoritative checks with the actual finite release point.
    #[must_use]
    pub fn observe_attach_methyl_compact_group_availability_v1(
        &self,
        fence: DocumentFenceV1,
        anchor: DocumentObjectIdV1,
    ) -> AttachedCompactGroupAvailabilityV1 {
        let revision = self.current_revision_v1();
        let digest = self.current_digest_v1();
        let category = if revision != fence.revision() {
            AttachedCompactGroupAvailabilityCategoryV1::StaleRevision
        } else if digest != fence.digest() {
            AttachedCompactGroupAvailabilityCategoryV1::StaleDigest
        } else if let Ok(observation) = self.document_observation() {
            match resolve_anchor(&observation, &anchor) {
                Ok(resolved) => availability_category(self.current_document_v1(), resolved),
                Err(_) => AttachedCompactGroupAvailabilityCategoryV1::UnknownAnchor,
            }
        } else {
            AttachedCompactGroupAvailabilityCategoryV1::SessionConflict
        };
        AttachedCompactGroupAvailabilityV1::new(revision, digest, anchor, category)
    }

    /// Prepare exactly one Methyl compact group from a direct atom and finite release point.
    pub fn prepare_attach_methyl_compact_group_v1(
        &mut self,
        fence: DocumentFenceV1,
        anchor: DocumentObjectIdV1,
        release: AttachedCompactGroupReleaseV1,
    ) -> Result<PendingAttachedCompactGroupV1, AttachedCompactGroupSessionErrorV1> {
        require_fence(self, fence)?;
        let observation = self.document_observation().map_err(|_| AttachedCompactGroupSessionErrorV1::SessionConflict)?;
        let resolved = resolve_anchor(&observation, &anchor)?;
        let pose = attached_methyl_candidate_v1(resolved.position, release).map_err(map_core_error)?;
        let validation_group_id = validation_identifier(self.current_document_v1(), "group")?;
        let validation_bond_id = validation_identifier(self.current_document_v1(), "bond")?;
        let capacity_document = self.current_document_v1()
            .with_attach_methyl_compact_group_v1(
                &resolved.molecule_id,
                &resolved.anchor_id,
                &validation_group_id,
                &validation_bond_id,
                pose,
            )
            .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
        require_materialized_methyl_capacity(&capacity_document, &resolved.molecule_id, &validation_group_id)?;
        let ((group_id, bond_id), effects) = self.reserve_generated_ids_for_transition_v1(|ids, indexed| {
            let (group_id, ids) = ids.reserve_group(indexed)?;
            let (bond_id, ids) = ids.reserve_bond(indexed)?;
            Ok(((group_id, bond_id), ids))
        }).map_err(|_| AttachedCompactGroupSessionErrorV1::SessionConflict)?;
        let document = self.current_document_v1()
            .with_attach_methyl_compact_group_v1(&resolved.molecule_id, &resolved.anchor_id, &group_id, &bond_id, pose)
            .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
        let compact_group_object_id = DocumentObjectIdV1::from_class_source(
            "molecule/compact-group",
            group_id.as_str(),
        )
        .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
        let revision = self.next_revision_v1().ok_or(AttachedCompactGroupSessionErrorV1::SessionConflict)?;
        let state = RevisionState::from_document(revision, document)
            .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
        let mut transition = self.prepare_changed_session_transition_v1(fence.revision(), fence.digest(), state, effects)
            .map_err(map_prepare_error)?;
        let overlay_request = AcceptedRenderOverlayRequestV1::new(vec![AcceptedRenderOverlayTargetV1::bond(bond_id.as_str())])
            .map_err(|_| AttachedCompactGroupSessionErrorV1::RendererAdmission)?;
        let overlay = transition.renderer_precommit_overlay_v1(&overlay_request)
            .map_err(|_| AttachedCompactGroupSessionErrorV1::RendererAdmission)?;
        transition.install_precommit_overlay_v1(overlay.clone())
            .map_err(|_| AttachedCompactGroupSessionErrorV1::RendererAdmission)?;
        Ok(PendingAttachedCompactGroupV1 {
            session_issuer: self.authoring_capability_issuer.clone(),
            fence,
            focus_object_id: anchor,
            compact_group_object_id,
            transition,
            precommit_overlay: overlay,
        })
    }

    /// Commit one already-admitted methyl compact-group candidate as one history transition.
    pub fn commit_attach_methyl_compact_group_v1(
        &mut self,
        pending: &mut PendingAttachedCompactGroupV1,
    ) -> Result<AttachedCompactGroupCommitResultV1, AttachedCompactGroupSessionErrorV1> {
        if pending.transition.is_consumed_v1() { return Err(AttachedCompactGroupSessionErrorV1::Retired); }
        if !pending.session_issuer.same_issuer(&self.authoring_capability_issuer) { return Err(AttachedCompactGroupSessionErrorV1::ForeignSession); }
        let result = self.commit_session_operation_transition_v1(&mut pending.transition).map_err(map_commit_error)?;
        Ok(AttachedCompactGroupCommitResultV1::new(
            result,
            pending.focus_object_id.clone(),
            pending.compact_group_object_id.clone(),
        ))
    }

    /// Retire one pending compact-group attachment without consuming document state or IDs.
    pub fn retire_attach_methyl_compact_group_v1(
        &mut self,
        pending: &mut PendingAttachedCompactGroupV1,
    ) -> Result<(), AttachedCompactGroupSessionErrorV1> {
        if !pending.session_issuer.same_issuer(&self.authoring_capability_issuer) { return Err(AttachedCompactGroupSessionErrorV1::ForeignSession); }
        if pending.transition.is_consumed_v1() { return Err(AttachedCompactGroupSessionErrorV1::Retired); }
        self.retire_session_operation_transition_v1(&mut pending.transition).map_err(map_commit_error)
    }
}

fn availability_category(
    document: &crate::TypedDocument,
    resolved: ResolvedAnchorV1,
) -> AttachedCompactGroupAvailabilityCategoryV1 {
    let release = availability_release(resolved.position);
    let Ok(candidate) = attached_methyl_candidate_v1(resolved.position, release) else {
        return AttachedCompactGroupAvailabilityCategoryV1::CandidateAdmission;
    };
    let Ok(group_id) = validation_identifier(document, "availability-group") else {
        return AttachedCompactGroupAvailabilityCategoryV1::CandidateAdmission;
    };
    let Ok(bond_id) = validation_identifier(document, "availability-bond") else {
        return AttachedCompactGroupAvailabilityCategoryV1::CandidateAdmission;
    };
    let Ok(candidate_document) = document.with_attach_methyl_compact_group_v1(
        &resolved.molecule_id,
        &resolved.anchor_id,
        &group_id,
        &bond_id,
        candidate,
    ) else {
        return AttachedCompactGroupAvailabilityCategoryV1::CandidateAdmission;
    };
    match require_materialized_methyl_capacity(&candidate_document, &resolved.molecule_id, &group_id) {
        Ok(()) => AttachedCompactGroupAvailabilityCategoryV1::Available,
        Err(_) => AttachedCompactGroupAvailabilityCategoryV1::CandidateAdmission,
    }
}

fn availability_release(anchor: Point3V1) -> AttachedCompactGroupReleaseV1 {
    let offset_x = anchor.x() + 1.0;
    let x = if offset_x.is_finite() && offset_x != anchor.x() {
        offset_x
    } else {
        let next_x = anchor.x().next_up();
        if next_x.is_finite() { next_x } else { anchor.x().next_down() }
    };
    AttachedCompactGroupReleaseV1::new(x, anchor.y())
        .expect("a neighboring finite coordinate gives the geometry-independent capacity probe a pose")
}

fn require_fence(session: &DocumentSession, fence: DocumentFenceV1) -> Result<(), AttachedCompactGroupSessionErrorV1> {
    if session.current_revision_v1() != fence.revision() { return Err(AttachedCompactGroupSessionErrorV1::StaleRevision); }
    if session.current_digest_v1() != fence.digest() { return Err(AttachedCompactGroupSessionErrorV1::StaleDigest); }
    Ok(())
}

fn map_core_error(error: AttachedCompactGroupErrorV1) -> AttachedCompactGroupSessionErrorV1 {
    match error { AttachedCompactGroupErrorV1::InvalidPose => AttachedCompactGroupSessionErrorV1::InvalidPose }
}

fn map_prepare_error(error: super::DocumentSessionError) -> AttachedCompactGroupSessionErrorV1 {
    match error { super::DocumentSessionError::RendererAdmission => AttachedCompactGroupSessionErrorV1::RendererAdmission, _ => AttachedCompactGroupSessionErrorV1::SessionConflict }
}

fn map_commit_error(error: AdmittedSessionTransitionRefusalV1) -> AttachedCompactGroupSessionErrorV1 {
    match error {
        AdmittedSessionTransitionRefusalV1::ForeignSession => AttachedCompactGroupSessionErrorV1::ForeignSession,
        AdmittedSessionTransitionRefusalV1::Replayed => AttachedCompactGroupSessionErrorV1::Retired,
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => AttachedCompactGroupSessionErrorV1::StaleRevision,
        AdmittedSessionTransitionRefusalV1::RendererAdmission => AttachedCompactGroupSessionErrorV1::RendererAdmission,
        AdmittedSessionTransitionRefusalV1::ProvisionalCapability | AdmittedSessionTransitionRefusalV1::HistoryCapacity => AttachedCompactGroupSessionErrorV1::SessionConflict,
    }
}

/// Reuse the ordinary candidate-aware capacity lane through the immutable Methyl
/// materialization equivalence. Compact vertices intentionally have no neutral
/// atom capacity themselves; their one-carbon materialization is the canonical
/// chemistry candidate and leaves the exterior bond unchanged.
fn require_materialized_methyl_capacity(
    document: &crate::TypedDocument,
    molecule_id: &PersistentId,
    group_id: &PersistentId,
) -> Result<(), AttachedCompactGroupSessionErrorV1> {
    let probe_atom = probe_atom_id(document)?;
    let plan = document.prepare_compact_group_materialization_v1(
        TypedCompactGroupMaterializationRequestV1::new(
            molecule_id.clone(), group_id.clone(), vec![probe_atom], Vec::new(),
        ),
    ).map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
    let materialized = document.materialize_compact_group_v1(&plan)
        .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
    let molecule_object = DocumentObjectIdV1::from_class_source("cdml/molecule", molecule_id.as_str())
        .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
    let molecule = materialized.into_candidate().core_molecule(&molecule_object)
        .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?
        .ok_or(AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
    match crate::chemistry::evaluate_document_molecule_neutral_capacity_v1(&molecule) {
        Ok(DocumentBondCapacityOutcomeV1::WithinCapacity { .. }) => Ok(()),
        Ok(DocumentBondCapacityOutcomeV1::ExceedsCapacity { .. } | DocumentBondCapacityOutcomeV1::NotChecked { .. }) | Err(_) => Err(AttachedCompactGroupSessionErrorV1::CandidateAdmission),
    }
}

fn validation_identifier(
    document: &crate::TypedDocument,
    kind: &str,
) -> Result<PersistentId, AttachedCompactGroupSessionErrorV1> {
    for sequence in 0_u64..1_000 {
        let identifier = PersistentId::new(format!("ferrum-attached-methyl-validation-{kind}-{sequence}"))
            .map_err(|_| AttachedCompactGroupSessionErrorV1::CandidateAdmission)?;
        if document.indexed().resolve_id(&identifier).is_none() {
            return Ok(identifier);
        }
    }
    Err(AttachedCompactGroupSessionErrorV1::CandidateAdmission)
}

fn probe_atom_id(document: &crate::TypedDocument) -> Result<PersistentId, AttachedCompactGroupSessionErrorV1> {
    validation_identifier(document, "capacity-probe")
}

struct ResolvedAnchorV1 { molecule_id: PersistentId, anchor_id: PersistentId, position: Point3V1 }

fn resolve_anchor(observation: &SessionDocumentObservationV1, anchor: &DocumentObjectIdV1) -> Result<ResolvedAnchorV1, AttachedCompactGroupSessionErrorV1> {
    let molecule = observation.projection().molecules().iter().find(|molecule| molecule.atoms().iter().any(|atom| atom.id() == Some(anchor)))
        .ok_or(AttachedCompactGroupSessionErrorV1::UnknownAnchor)?;
    let atom = molecule.atoms().iter().find(|atom| atom.id() == Some(anchor))
        .ok_or(AttachedCompactGroupSessionErrorV1::UnknownAnchor)?;
    let molecule_id = molecule.source_id().and_then(|id| PersistentId::new(id.to_owned()).ok())
        .ok_or(AttachedCompactGroupSessionErrorV1::UnknownAnchor)?;
    let anchor_id = atom.source_id().and_then(|id| PersistentId::new(id.to_owned()).ok())
        .ok_or(AttachedCompactGroupSessionErrorV1::UnknownAnchor)?;
    Ok(ResolvedAnchorV1 { molecule_id, anchor_id, position: atom.position() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DocumentCompactGroupMaterializationRequestV1, SessionOperation, SessionOperationV1,
        SessionOperationTransitionRequestV1, TransitionAuthorizationV1,
    };

    const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        DocumentFenceV1::new(session.current_revision_v1(), session.current_digest_v1())
    }

    fn anchor(session: &DocumentSession) -> DocumentObjectIdV1 {
        session.document_observation().expect("observation").projection()
            .molecules()[0].atoms()[0].id().expect("direct atom selector").clone()
    }

    fn commit_attachment(session: &mut DocumentSession) -> AttachedCompactGroupCommitResultV1 {
        let mut pending = session
            .prepare_attach_methyl_compact_group_v1(
                fence(session),
                anchor(session),
                AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
            )
            .expect("prepare");
        session
            .commit_attach_methyl_compact_group_v1(&mut pending)
            .expect("commit")
    }

    #[test]
    fn prepare_commit_cancel_and_replay_preserve_the_closed_transaction_contract() {
        let mut session = DocumentSession::load(SOURCE).expect("source");
        let before = session.snapshot().expect("before");
        let mut pending = session.prepare_attach_methyl_compact_group_v1(
            fence(&session), anchor(&session), AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
        ).expect("prepare");
        assert_eq!(session.snapshot().expect("prepare is pure"), before);
        session.retire_attach_methyl_compact_group_v1(&mut pending).expect("cancel");
        assert_eq!(session.snapshot().expect("cancel is pure"), before);
        assert_eq!(session.commit_attach_methyl_compact_group_v1(&mut pending), Err(AttachedCompactGroupSessionErrorV1::Retired));
        let result = commit_attachment(&mut session);
        let after = result.observation().snapshot();
        assert_eq!(result.focus_object_id(), &anchor(&session));
        assert!(!result.compact_group_object_id().as_str().is_empty());
        assert_ne!(result.compact_group_object_id(), result.focus_object_id());
        assert_eq!(after.revision(), before.revision() + 1);
        let authored_group = result
            .observation()
            .projection()
            .molecules()
            .iter()
            .flat_map(|molecule| molecule.compact_groups())
            .find(|group| group.id() == result.compact_group_object_id())
            .expect("returned compact-group identity remains selectable");
        let group_source_id = session
            .current_document_v1()
            .resolve_document_object_id(authored_group.id())
            .and_then(|record| record.attribute("id"))
            .expect("selected compact group has a durable source ID");
        let materialization = DocumentCompactGroupMaterializationRequestV1::new(
            after.revision(), *after.digest(), PersistentId::new("m".to_owned()).expect("molecule"),
            PersistentId::new(group_source_id.to_owned()).expect("group"),
        );
        let mut materialized = session.prepare_session_operation_transition_v1(
            SessionOperationTransitionRequestV1::new(
                after.revision(), SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(materialization)),
                TransitionAuthorizationV1::None,
            ),
        ).expect("existing materialization prepares");
        session.commit_session_operation_transition_v1(&mut materialized).expect("existing materialization commits");
        assert_ne!(session.snapshot().expect("materialized snapshot"), *after);
    }

    #[test]
    fn foreign_and_stale_pending_attachments_preserve_the_next_accepted_observation() {
        let mut owner = DocumentSession::load(SOURCE).expect("owner source");
        let mut other = DocumentSession::load(SOURCE).expect("other source");
        let mut foreign = owner
            .prepare_attach_methyl_compact_group_v1(
                fence(&owner),
                anchor(&owner),
                AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
            )
            .expect("owner prepare");
        assert_eq!(
            other.commit_attach_methyl_compact_group_v1(&mut foreign),
            Err(AttachedCompactGroupSessionErrorV1::ForeignSession)
        );
        let committed = owner
            .commit_attach_methyl_compact_group_v1(&mut foreign)
            .expect("owner commit");
        let mut fresh_owner = DocumentSession::load(SOURCE).expect("fresh owner source");
        assert_eq!(
            committed.compact_group_object_id(),
            commit_attachment(&mut fresh_owner).compact_group_object_id(),
        );

        let mut session = DocumentSession::load(SOURCE).expect("source");
        let mut first = session
            .prepare_attach_methyl_compact_group_v1(
                fence(&session),
                anchor(&session),
                AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
            )
            .expect("first prepare");
        let mut stale = session
            .prepare_attach_methyl_compact_group_v1(
                fence(&session),
                anchor(&session),
                AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
            )
            .expect("stale prepare");
        let first_committed = session
            .commit_attach_methyl_compact_group_v1(&mut first)
            .expect("first commit");
        assert_eq!(
            session.commit_attach_methyl_compact_group_v1(&mut stale),
            Err(AttachedCompactGroupSessionErrorV1::StaleRevision)
        );
        let mut fresh_session = DocumentSession::load(SOURCE).expect("fresh source");
        assert_eq!(
            first_committed.compact_group_object_id(),
            commit_attachment(&mut fresh_session).compact_group_object_id(),
        );
    }

    #[test]
    fn refusal_categories_leave_the_next_accepted_observation_unchanged() {
        let mut selector_session = DocumentSession::load(SOURCE).expect("source");
        let before = selector_session.snapshot().expect("before");
        let missing = DocumentObjectIdV1::from_class_source("molecule/atom", "missing")
            .expect("durable missing selector");
        assert!(matches!(
            selector_session.prepare_attach_methyl_compact_group_v1(
                fence(&selector_session),
                missing,
                AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
            ),
            Err(AttachedCompactGroupSessionErrorV1::UnknownAnchor)
        ));
        assert_eq!(selector_session.snapshot().expect("unchanged"), before);
        let mut fresh_selector = DocumentSession::load(SOURCE).expect("fresh source");
        assert_eq!(
            commit_attachment(&mut selector_session).compact_group_object_id(),
            commit_attachment(&mut fresh_selector).compact_group_object_id(),
        );

        let mut pose_session = DocumentSession::load(SOURCE).expect("source");
        let before = pose_session.snapshot().expect("before");
        assert!(matches!(
            pose_session.prepare_attach_methyl_compact_group_v1(
                fence(&pose_session),
                anchor(&pose_session),
                AttachedCompactGroupReleaseV1::new(0.0, 0.0).expect("release"),
            ),
            Err(AttachedCompactGroupSessionErrorV1::InvalidPose)
        ));
        assert_eq!(pose_session.snapshot().expect("unchanged"), before);
        let mut fresh_pose = DocumentSession::load(SOURCE).expect("fresh source");
        assert_eq!(
            commit_attachment(&mut pose_session).compact_group_object_id(),
            commit_attachment(&mut fresh_pose).compact_group_object_id(),
        );

        let capacity_source = concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\">",
            "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
            "<atom id=\"h1\" name=\"H\"><point x=\"1\" y=\"0\"/></atom>",
            "<atom id=\"h2\" name=\"H\"><point x=\"-1\" y=\"0\"/></atom>",
            "<atom id=\"h3\" name=\"H\"><point x=\"0\" y=\"1\"/></atom>",
            "<atom id=\"h4\" name=\"H\"><point x=\"0\" y=\"-1\"/></atom>",
            "<bond id=\"b1\" start=\"a\" end=\"h1\" type=\"n1\"/>",
            "<bond id=\"b2\" start=\"a\" end=\"h2\" type=\"n1\"/>",
            "<bond id=\"b3\" start=\"a\" end=\"h3\" type=\"n1\"/>",
            "<bond id=\"b4\" start=\"a\" end=\"h4\" type=\"n1\"/>",
            "</molecule></cdml>",
        );
        let mut capacity_session = DocumentSession::load(capacity_source).expect("capacity source");
        let before = capacity_session.snapshot().expect("before");
        assert!(matches!(
            capacity_session.prepare_attach_methyl_compact_group_v1(
                fence(&capacity_session),
                anchor(&capacity_session),
                AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
            ),
            Err(AttachedCompactGroupSessionErrorV1::CandidateAdmission)
        ));
        assert_eq!(capacity_session.snapshot().expect("unchanged"), before);
    }

    #[test]
    fn availability_is_advisory_for_eligible_and_unavailable_anchors() {
        let session = DocumentSession::load(SOURCE).expect("source");
        let before = session.snapshot().expect("before");
        let available = session.observe_attach_methyl_compact_group_availability_v1(
            fence(&session),
            anchor(&session),
        );
        assert!(available.is_available());
        assert_eq!(available.category(), AttachedCompactGroupAvailabilityCategoryV1::Available);
        assert_eq!(available.revision(), before.revision());
        assert_eq!(available.digest(), before.digest());
        assert_eq!(session.snapshot().expect("availability is pure"), before);

        let missing = DocumentObjectIdV1::from_class_source("molecule/atom", "missing")
            .expect("durable missing selector");
        let unavailable = session.observe_attach_methyl_compact_group_availability_v1(
            fence(&session),
            missing,
        );
        assert!(!unavailable.is_available());
        assert_eq!(unavailable.category(), AttachedCompactGroupAvailabilityCategoryV1::UnknownAnchor);
        assert_eq!(session.snapshot().expect("unavailable availability is pure"), before);
    }
}
