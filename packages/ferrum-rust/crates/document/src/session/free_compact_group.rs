//! Opaque transaction for placing one free Methyl compact group.

use thiserror::Error;

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentFenceV1, DocumentObjectIdV1, DocumentSession,
    PreparedSessionTransitionV1, RevisionState, SessionDocumentObservationV1,
    SessionOperationResultV1,
};
use crate::{
    AuthoringCapabilityIssuerV1, FreeCompactGroupErrorV1, PlaceFreeCompactGroupV1,
    free_compact_group_v1::free_compact_group_candidate_v1,
};

/// Opaque session-affine, one-use pending free compact-group placement.
pub struct PendingPlaceFreeCompactGroupV1 {
    session_issuer: AuthoringCapabilityIssuerV1,
    transition: PreparedSessionTransitionV1,
    molecule_object_id: DocumentObjectIdV1,
    compact_group_object_id: DocumentObjectIdV1,
}

impl std::fmt::Debug for PendingPlaceFreeCompactGroupV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingPlaceFreeCompactGroupV1")
            .field("is_resolved", &self.transition.is_consumed_v1())
            .finish()
    }
}

/// Authoritative durable facts from one accepted free compact-group placement.
#[derive(Clone, Debug, PartialEq)]
pub struct FreeCompactGroupPlacementCommitResultV1 {
    result: SessionOperationResultV1,
    molecule_object_id: DocumentObjectIdV1,
    compact_group_object_id: DocumentObjectIdV1,
}

impl FreeCompactGroupPlacementCommitResultV1 {
    fn new(
        result: SessionOperationResultV1,
        molecule_object_id: DocumentObjectIdV1,
        compact_group_object_id: DocumentObjectIdV1,
    ) -> Self {
        Self {
            result,
            molecule_object_id,
            compact_group_object_id,
        }
    }

    /// Return the complete post-commit observation.
    #[must_use]
    pub fn observation(&self) -> &SessionDocumentObservationV1 {
        self.result.observation()
    }

    /// Return the newly authored direct-root molecule.
    #[must_use]
    pub const fn molecule_object_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_object_id
    }

    /// Return the newly authored compact-group object.
    #[must_use]
    pub const fn compact_group_object_id(&self) -> &DocumentObjectIdV1 {
        &self.compact_group_object_id
    }
}

/// Closed refusal vocabulary for free compact-group placement.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum FreeCompactGroupPlacementSessionErrorV1 {
    #[error("free compact-group placement revision is stale")]
    StaleRevision,
    #[error("free compact-group placement digest is stale")]
    StaleDigest,
    #[error("free compact-group placement belongs to another session")]
    ForeignSession,
    #[error("free compact-group placement is retired")]
    Retired,
    #[error("free compact-group placement supports the Methyl catalog key only")]
    UnsupportedCatalogKey,
    #[error("free compact-group placement candidate could not be admitted")]
    CandidateAdmission,
    #[error("free compact-group placement candidate could not be rendered completely")]
    RendererAdmission,
    #[error("free compact-group placement session conflict")]
    SessionConflict,
}

impl DocumentSession {
    /// Prepare one free Methyl group as a fully renderer-admitted atomic transition.
    ///
    /// The pending capability intentionally exposes no precommit overlay: the accepted
    /// overlay protocol currently addresses only atoms and bonds, while this candidate
    /// contains neither. Complete-document renderer admission remains mandatory here.
    pub fn prepare_place_free_compact_group_v1(
        &mut self,
        fence: DocumentFenceV1,
        request: PlaceFreeCompactGroupV1,
    ) -> Result<PendingPlaceFreeCompactGroupV1, FreeCompactGroupPlacementSessionErrorV1> {
        require_fence(self, fence)?;
        let candidate = free_compact_group_candidate_v1(request).map_err(map_core_error)?;
        // Provisional reservation derives durable identities without mutating the live allocator.
        let ((molecule_id, group_id), effects) = self
            .reserve_generated_ids_for_transition_v1(|ids, indexed| {
                let (molecules, ids) = ids.reserve_molecule_roots(indexed, 1)?;
                let (group_id, ids) = ids.reserve_group(indexed)?;
                let [molecule_id] = molecules
                    .try_into()
                    .expect("one root reservation returns one identity");
                Ok(((molecule_id, group_id), ids))
            })
            .map_err(|_| FreeCompactGroupPlacementSessionErrorV1::SessionConflict)?;
        // Candidate admission validates the exact durable identities that the transition will commit.
        let document = self
            .current_document_v1()
            .with_place_free_compact_group_v1(&molecule_id, &group_id, candidate)
            .map_err(|_| FreeCompactGroupPlacementSessionErrorV1::CandidateAdmission)?;
        let molecule_object_id = document
            .document_object_id_for_source_id_v1(&molecule_id)
            .ok_or(FreeCompactGroupPlacementSessionErrorV1::CandidateAdmission)?;
        let compact_group_object_id = document
            .document_object_id_for_source_id_v1(&group_id)
            .ok_or(FreeCompactGroupPlacementSessionErrorV1::CandidateAdmission)?;
        let revision = self
            .next_revision_v1()
            .ok_or(FreeCompactGroupPlacementSessionErrorV1::SessionConflict)?;
        let state = RevisionState::from_document(revision, document)
            .map_err(|_| FreeCompactGroupPlacementSessionErrorV1::CandidateAdmission)?;
        // Transition assembly retains reservation effects for the single renderer-admitted commit.
        let transition = self
            .prepare_changed_session_transition_v1(fence.revision(), fence.digest(), state, effects)
            .map_err(map_prepare_error)?;
        Ok(PendingPlaceFreeCompactGroupV1 {
            session_issuer: self.authoring_capability_issuer.clone(),
            transition,
            molecule_object_id,
            compact_group_object_id,
        })
    }

    /// Commit one already renderer-admitted free compact-group placement.
    pub fn commit_place_free_compact_group_v1(
        &mut self,
        pending: &mut PendingPlaceFreeCompactGroupV1,
    ) -> Result<FreeCompactGroupPlacementCommitResultV1, FreeCompactGroupPlacementSessionErrorV1>
    {
        if pending.transition.is_consumed_v1() {
            return Err(FreeCompactGroupPlacementSessionErrorV1::Retired);
        }
        if !pending
            .session_issuer
            .same_issuer(&self.authoring_capability_issuer)
        {
            return Err(FreeCompactGroupPlacementSessionErrorV1::ForeignSession);
        }
        let result = self
            .commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(map_commit_error)?;
        Ok(FreeCompactGroupPlacementCommitResultV1::new(
            result,
            pending.molecule_object_id.clone(),
            pending.compact_group_object_id.clone(),
        ))
    }

    /// Retire one pending free compact-group placement without mutating the document.
    pub fn retire_place_free_compact_group_v1(
        &mut self,
        pending: &mut PendingPlaceFreeCompactGroupV1,
    ) -> Result<(), FreeCompactGroupPlacementSessionErrorV1> {
        if !pending
            .session_issuer
            .same_issuer(&self.authoring_capability_issuer)
        {
            return Err(FreeCompactGroupPlacementSessionErrorV1::ForeignSession);
        }
        if pending.transition.is_consumed_v1() {
            return Err(FreeCompactGroupPlacementSessionErrorV1::Retired);
        }
        self.retire_session_operation_transition_v1(&mut pending.transition)
            .map_err(map_commit_error)
    }
}

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), FreeCompactGroupPlacementSessionErrorV1> {
    if session.current_revision_v1() != fence.revision() {
        return Err(FreeCompactGroupPlacementSessionErrorV1::StaleRevision);
    }
    if session.current_digest_v1() != fence.digest() {
        return Err(FreeCompactGroupPlacementSessionErrorV1::StaleDigest);
    }
    Ok(())
}

fn map_core_error(error: FreeCompactGroupErrorV1) -> FreeCompactGroupPlacementSessionErrorV1 {
    match error {
        FreeCompactGroupErrorV1::UnsupportedCatalogKey => {
            FreeCompactGroupPlacementSessionErrorV1::UnsupportedCatalogKey
        }
    }
}

fn map_prepare_error(
    error: super::DocumentSessionError,
) -> FreeCompactGroupPlacementSessionErrorV1 {
    match error {
        super::DocumentSessionError::RendererAdmission => {
            FreeCompactGroupPlacementSessionErrorV1::RendererAdmission
        }
        _ => FreeCompactGroupPlacementSessionErrorV1::SessionConflict,
    }
}

fn map_commit_error(
    error: AdmittedSessionTransitionRefusalV1,
) -> FreeCompactGroupPlacementSessionErrorV1 {
    match error {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            FreeCompactGroupPlacementSessionErrorV1::ForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Replayed => {
            FreeCompactGroupPlacementSessionErrorV1::Retired
        }
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            FreeCompactGroupPlacementSessionErrorV1::StaleRevision
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            FreeCompactGroupPlacementSessionErrorV1::RendererAdmission
        }
        AdmittedSessionTransitionRefusalV1::ProvisionalCapability
        | AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
            FreeCompactGroupPlacementSessionErrorV1::SessionConflict
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompactGroupCatalogKeyV1, Point3V1};

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        DocumentFenceV1::new(session.current_revision_v1(), session.current_digest_v1())
    }

    fn methyl_request() -> PlaceFreeCompactGroupV1 {
        PlaceFreeCompactGroupV1::new(
            CompactGroupCatalogKeyV1::Methyl,
            Point3V1::new(12.0, -4.0, 0.0).expect("anchor"),
        )
    }

    #[test]
    fn free_methyl_is_one_group_only_molecule_with_history_and_persistence() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty document");
        let before = session.snapshot().expect("before");
        let mut pending = session
            .prepare_place_free_compact_group_v1(fence(&session), methyl_request())
            .expect("prepare renderer-admitted placement");
        assert_eq!(session.snapshot().expect("prepare is pure"), before);
        let result = session
            .commit_place_free_compact_group_v1(&mut pending)
            .expect("commit");
        let after = result.observation().snapshot().clone();
        assert_eq!(after.revision(), before.revision() + 1);
        let molecule = result
            .observation()
            .projection()
            .molecules()
            .iter()
            .find(|item| item.id() == Some(result.molecule_object_id()))
            .expect("returned direct molecule is projected");
        assert!(molecule.atoms().is_empty());
        assert!(molecule.bonds().is_empty());
        assert_eq!(molecule.compact_groups().len(), 1);
        let group = &molecule.compact_groups()[0];
        assert_eq!(group.id(), result.compact_group_object_id());
        assert_eq!(group.catalog_key(), CompactGroupCatalogKeyV1::Methyl);
        assert_eq!(
            group.anchor(),
            Point3V1::new(12.0, -4.0, 0.0).expect("anchor")
        );
        assert_eq!(group.orientation_degrees(), 0.0);
        session.undo(after.revision()).expect("undo");
        assert_eq!(
            session.snapshot().expect("undo snapshot").cdml(),
            before.cdml()
        );
        let redone = session.redo(session.current_revision_v1()).expect("redo");
        assert_eq!(redone.observation().snapshot().cdml(), after.cdml());
        let reopened = DocumentSession::load(after.cdml()).expect("reopen serialized candidate");
        let reopened_observation = reopened
            .document_observation()
            .expect("reopened observation");
        let reopened_group = &reopened_observation.projection().molecules()[0].compact_groups()[0];
        assert_eq!(
            reopened_group.catalog_key(),
            CompactGroupCatalogKeyV1::Methyl
        );
        assert_eq!(
            reopened_group.anchor(),
            Point3V1::new(12.0, -4.0, 0.0).expect("anchor")
        );
    }

    #[test]
    fn free_placement_refusals_and_retirement_leave_the_document_unchanged() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty document");
        let before = session.snapshot().expect("before");
        let unsupported = PlaceFreeCompactGroupV1::new(
            CompactGroupCatalogKeyV1::Nitro,
            Point3V1::new(0.0, 0.0, 0.0).expect("anchor"),
        );
        assert!(matches!(
            session.prepare_place_free_compact_group_v1(fence(&session), unsupported),
            Err(FreeCompactGroupPlacementSessionErrorV1::UnsupportedCatalogKey),
        ));
        assert_eq!(session.snapshot().expect("unsupported is pure"), before);
        let stale = DocumentFenceV1::new(before.revision() + 1, *before.digest());
        assert!(matches!(
            session.prepare_place_free_compact_group_v1(stale, methyl_request()),
            Err(FreeCompactGroupPlacementSessionErrorV1::StaleRevision),
        ));
        let mut pending = session
            .prepare_place_free_compact_group_v1(fence(&session), methyl_request())
            .expect("prepare");
        session
            .retire_place_free_compact_group_v1(&mut pending)
            .expect("retire");
        assert_eq!(session.snapshot().expect("retirement is pure"), before);
        assert_eq!(
            session.commit_place_free_compact_group_v1(&mut pending),
            Err(FreeCompactGroupPlacementSessionErrorV1::Retired),
        );
    }
}
