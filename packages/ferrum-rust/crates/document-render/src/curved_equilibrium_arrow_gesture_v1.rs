//! Renderer-preflighted, Rust-owned quadratic equilibrium-arrow authoring.

use ferrum_document::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityV1, CurvedEquilibriumArrowGeometryErrorV1,
    DocumentFenceV1, DocumentSession, PendingCreatePresentationV1, Point3V1,
    PresentationCreateErrorV1, PresentationCreateRequestV1, PresentationGesturePoint2V1,
    PresentationRootSelectorV1, SessionOperationResultV1, curved_equilibrium_arrow_geometry_v1,
};
use ferrum_render::{
    DocumentRenderOutcomeV1, compose_document_render_plan_v1,
    document_observation_from_accepted_operation_v1,
};
use thiserror::Error;

const MAXIMUM_EXTENT_PT: f64 = 20_000.0;

#[derive(Clone, Debug)]
/// Opaque, session-fenced input for one curved equilibrium-arrow lifecycle.
pub struct CurvedEquilibriumArrowGestureV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
}

#[derive(Clone, Debug)]
/// Rust-issued preview that remains bound to its originating gesture and session.
pub struct CurvedEquilibriumArrowPreviewV1 {
    gesture: CurvedEquilibriumArrowGestureV1,
    end: PresentationGesturePoint2V1,
    overlay: CurvedEquilibriumArrowOverlayV1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurvedEquilibriumArrowOverlayV1 {
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
    end: PresentationGesturePoint2V1,
    lower_axis: [PresentationGesturePoint2V1; 4],
    upper_axis: [PresentationGesturePoint2V1; 4],
    lower_head: [PresentationGesturePoint2V1; 4],
    upper_head: [PresentationGesturePoint2V1; 4],
}

impl CurvedEquilibriumArrowOverlayV1 {
    #[must_use]
    pub const fn start(&self) -> PresentationGesturePoint2V1 {
        self.start
    }
    #[must_use]
    pub const fn control(&self) -> PresentationGesturePoint2V1 {
        self.control
    }
    #[must_use]
    pub const fn end(&self) -> PresentationGesturePoint2V1 {
        self.end
    }
    #[must_use]
    pub const fn lower_axis(&self) -> &[PresentationGesturePoint2V1; 4] {
        &self.lower_axis
    }
    #[must_use]
    pub const fn upper_axis(&self) -> &[PresentationGesturePoint2V1; 4] {
        &self.upper_axis
    }
    #[must_use]
    pub const fn lower_head(&self) -> &[PresentationGesturePoint2V1; 4] {
        &self.lower_head
    }
    #[must_use]
    pub const fn upper_head(&self) -> &[PresentationGesturePoint2V1; 4] {
        &self.upper_head
    }
}

impl CurvedEquilibriumArrowPreviewV1 {
    #[must_use]
    pub const fn overlay(&self) -> &CurvedEquilibriumArrowOverlayV1 {
        &self.overlay
    }
}

#[derive(Debug)]
/// Renderer-preflighted arrow awaiting one commit; unsuccessful commits preserve it for retry.
pub struct PreparedCurvedEquilibriumArrowV1 {
    pending: Option<PendingCreatePresentationV1>,
    identifier: String,
}

#[derive(Clone, Debug)]
/// Result of the single successful commit for a prepared curved equilibrium arrow.
pub struct CommittedCurvedEquilibriumArrowV1 {
    root: PresentationRootSelectorV1,
    result: SessionOperationResultV1,
}

impl CommittedCurvedEquilibriumArrowV1 {
    #[must_use]
    pub const fn root(&self) -> &PresentationRootSelectorV1 {
        &self.root
    }
    #[must_use]
    pub const fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurvedEquilibriumArrowGestureCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    ReplayedGesture,
    InvalidPoint,
    CollapsedSpan,
    ControlTooNearChord,
    ExceedsGeometryLimit,
    RenderPreparation,
    SessionConflict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurvedEquilibriumArrowGestureRecoveryV1 {
    RefreshAndRestart,
    ChangeGeometry,
    DocumentUnchanged,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum CurvedEquilibriumArrowGestureErrorV1 {
    #[error("curved equilibrium-arrow snapshot is stale")]
    StaleSnapshot,
    #[error("curved equilibrium-arrow gesture belongs to another document session")]
    ForeignSession,
    #[error("curved equilibrium-arrow preview belongs to a different gesture")]
    MismatchedPreview,
    #[error("curved equilibrium-arrow receipt was already consumed")]
    ReplayedGesture,
    #[error("curved equilibrium-arrow point is invalid")]
    InvalidPoint,
    #[error("curved equilibrium-arrow start and end are too close")]
    CollapsedSpan,
    #[error("curved equilibrium-arrow control point is too close to its chord")]
    ControlTooNearChord,
    #[error("curved equilibrium-arrow exceeds the geometry limit")]
    ExceedsGeometryLimit,
    #[error("curved equilibrium-arrow candidate failed renderer preflight")]
    RenderPreparation,
    #[error("curved equilibrium-arrow session transaction failed")]
    SessionConflict,
}

impl CurvedEquilibriumArrowGestureErrorV1 {
    #[must_use]
    pub const fn category(&self) -> CurvedEquilibriumArrowGestureCategoryV1 {
        match self {
            Self::StaleSnapshot => CurvedEquilibriumArrowGestureCategoryV1::StaleSnapshot,
            Self::ForeignSession => CurvedEquilibriumArrowGestureCategoryV1::ForeignSession,
            Self::MismatchedPreview => CurvedEquilibriumArrowGestureCategoryV1::MismatchedPreview,
            Self::ReplayedGesture => CurvedEquilibriumArrowGestureCategoryV1::ReplayedGesture,
            Self::InvalidPoint => CurvedEquilibriumArrowGestureCategoryV1::InvalidPoint,
            Self::CollapsedSpan => CurvedEquilibriumArrowGestureCategoryV1::CollapsedSpan,
            Self::ControlTooNearChord => {
                CurvedEquilibriumArrowGestureCategoryV1::ControlTooNearChord
            }
            Self::ExceedsGeometryLimit => {
                CurvedEquilibriumArrowGestureCategoryV1::ExceedsGeometryLimit
            }
            Self::RenderPreparation => CurvedEquilibriumArrowGestureCategoryV1::RenderPreparation,
            Self::SessionConflict => CurvedEquilibriumArrowGestureCategoryV1::SessionConflict,
        }
    }
    #[must_use]
    pub const fn recovery(&self) -> CurvedEquilibriumArrowGestureRecoveryV1 {
        match self {
            Self::StaleSnapshot
            | Self::ForeignSession
            | Self::MismatchedPreview
            | Self::ReplayedGesture
            | Self::SessionConflict => CurvedEquilibriumArrowGestureRecoveryV1::RefreshAndRestart,
            Self::CollapsedSpan | Self::ControlTooNearChord | Self::ExceedsGeometryLimit => {
                CurvedEquilibriumArrowGestureRecoveryV1::ChangeGeometry
            }
            Self::InvalidPoint | Self::RenderPreparation => {
                CurvedEquilibriumArrowGestureRecoveryV1::DocumentUnchanged
            }
        }
    }
}

/// Begin a session-fenced gesture from one current document snapshot.
///
/// The returned opaque handle is accepted only by the issuing session.
pub fn begin_curved_equilibrium_arrow_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
) -> Result<CurvedEquilibriumArrowGestureV1, CurvedEquilibriumArrowGestureErrorV1> {
    require_fence(session, fence)?;
    Ok(CurvedEquilibriumArrowGestureV1 {
        capability: session.authoring_capability_issuer_v1().issue(),
        fence,
        start,
        control,
    })
}

pub fn preview_curved_equilibrium_arrow_gesture_v1(
    session: &DocumentSession,
    gesture: &CurvedEquilibriumArrowGestureV1,
    end: PresentationGesturePoint2V1,
) -> Result<CurvedEquilibriumArrowPreviewV1, CurvedEquilibriumArrowGestureErrorV1> {
    if !gesture
        .capability
        .belongs_to(&session.authoring_capability_issuer_v1())
    {
        return Err(CurvedEquilibriumArrowGestureErrorV1::ForeignSession);
    }
    require_fence(session, gesture.fence)?;
    let overlay = geometry(gesture.start, gesture.control, end)?;
    Ok(CurvedEquilibriumArrowPreviewV1 {
        gesture: gesture.clone(),
        end,
        overlay,
    })
}

/// Renderer-preflight a Rust-issued preview without mutating the document.
///
/// Preparation rejects foreign or mismatched handles and returns one receipt for commit.
pub fn prepare_curved_equilibrium_arrow_gesture_v1(
    session: &mut DocumentSession,
    gesture: &CurvedEquilibriumArrowGestureV1,
    preview: &CurvedEquilibriumArrowPreviewV1,
) -> Result<PreparedCurvedEquilibriumArrowV1, CurvedEquilibriumArrowGestureErrorV1> {
    if !gesture
        .capability
        .belongs_to(&session.authoring_capability_issuer_v1())
        || !preview
            .gesture
            .capability
            .belongs_to(&session.authoring_capability_issuer_v1())
    {
        return Err(CurvedEquilibriumArrowGestureErrorV1::ForeignSession);
    }
    if !gesture
        .capability
        .same_capability(&preview.gesture.capability)
    {
        return Err(CurvedEquilibriumArrowGestureErrorV1::MismatchedPreview);
    }
    match gesture
        .capability
        .claim_for_commit(&session.authoring_capability_issuer_v1())
    {
        Ok(claim) => drop(claim),
        Err(AuthoringCapabilityAccessErrorV1::ForeignSession) => {
            return Err(CurvedEquilibriumArrowGestureErrorV1::ForeignSession);
        }
        Err(AuthoringCapabilityAccessErrorV1::Replayed) => {
            return Err(CurvedEquilibriumArrowGestureErrorV1::ReplayedGesture);
        }
    }
    require_fence(session, gesture.fence)?;
    let pending = session
        .prepare_create_presentation_v1(
            &gesture.capability,
            gesture.fence,
            PresentationCreateRequestV1::CurvedEquilibriumArrow {
                start: gesture.start,
                control: gesture.control,
                end: preview.end,
            },
        )
        .map_err(map_presentation_create_error)?;
    let identifier = pending.identifier().as_str().to_owned();
    let candidate = pending
        .candidate_cdml_for_render_preflight_v1()
        .ok_or(CurvedEquilibriumArrowGestureErrorV1::ReplayedGesture)?;
    ferrum_render_contract::preflight_complete_document_v1(&candidate)
        .map_err(|_| CurvedEquilibriumArrowGestureErrorV1::RenderPreparation)?;
    let candidate_session = DocumentSession::load(&candidate)
        .map_err(|_| CurvedEquilibriumArrowGestureErrorV1::RenderPreparation)?;
    let observation = candidate_session
        .observe(0)
        .map_err(|_| CurvedEquilibriumArrowGestureErrorV1::RenderPreparation)?;
    let render_observation = document_observation_from_accepted_operation_v1(&observation)
        .map_err(|_| CurvedEquilibriumArrowGestureErrorV1::RenderPreparation)?;
    let plan = compose_document_render_plan_v1(&render_observation)
        .map_err(|_| CurvedEquilibriumArrowGestureErrorV1::RenderPreparation)?;
    if plan
        .outcomes()
        .iter()
        .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(CurvedEquilibriumArrowGestureErrorV1::RenderPreparation);
    }
    Ok(PreparedCurvedEquilibriumArrowV1 {
        pending: Some(pending),
        identifier,
    })
}

/// Commit one prepared arrow exactly once.
///
/// A non-successful transaction preserves the prepared receipt for retry; a successful commit consumes it.
pub fn commit_curved_equilibrium_arrow_gesture_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedCurvedEquilibriumArrowV1,
) -> Result<CommittedCurvedEquilibriumArrowV1, CurvedEquilibriumArrowGestureErrorV1> {
    let mut pending = prepared
        .pending
        .take()
        .ok_or(CurvedEquilibriumArrowGestureErrorV1::ReplayedGesture)?;
    let result = (|| {
        session
            .commit_create_presentation_v1(&mut pending)
            .map_err(map_presentation_create_error)
    })();
    match result {
        Ok(result) => {
            let root = PresentationRootSelectorV1::new(&prepared.identifier, pending.root_kind())
                .expect("generated equilibrium arrow identifier is valid");
            Ok(CommittedCurvedEquilibriumArrowV1 { root, result })
        }
        Err(error) => {
            prepared.pending = Some(pending);
            Err(error)
        }
    }
}

fn map_presentation_create_error(
    error: PresentationCreateErrorV1,
) -> CurvedEquilibriumArrowGestureErrorV1 {
    match error {
        PresentationCreateErrorV1::ForeignSession => {
            CurvedEquilibriumArrowGestureErrorV1::ForeignSession
        }
        PresentationCreateErrorV1::StaleSnapshot => {
            CurvedEquilibriumArrowGestureErrorV1::StaleSnapshot
        }
        PresentationCreateErrorV1::Replayed => {
            CurvedEquilibriumArrowGestureErrorV1::ReplayedGesture
        }
        PresentationCreateErrorV1::SessionConflict => {
            CurvedEquilibriumArrowGestureErrorV1::SessionConflict
        }
    }
}

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), CurvedEquilibriumArrowGestureErrorV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| CurvedEquilibriumArrowGestureErrorV1::SessionConflict)?;
    (snapshot.revision() == fence.revision() && snapshot.digest() == &fence.digest())
        .then_some(())
        .ok_or(CurvedEquilibriumArrowGestureErrorV1::StaleSnapshot)
}

fn geometry(
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
    end: PresentationGesturePoint2V1,
) -> Result<CurvedEquilibriumArrowOverlayV1, CurvedEquilibriumArrowGestureErrorV1> {
    if [start, control, end]
        .into_iter()
        .any(|point| point.x().abs() > MAXIMUM_EXTENT_PT || point.y().abs() > MAXIMUM_EXTENT_PT)
    {
        return Err(CurvedEquilibriumArrowGestureErrorV1::ExceedsGeometryLimit);
    }
    let issued = curved_equilibrium_arrow_geometry_v1(point3(start), point3(control), point3(end))
        .map_err(geometry_error)?;
    Ok(CurvedEquilibriumArrowOverlayV1 {
        start,
        control,
        end,
        lower_axis: issued.lower().axis().map(point2),
        upper_axis: issued.upper().axis().map(point2),
        lower_head: issued.lower().head().map(point2),
        upper_head: issued.upper().head().map(point2),
    })
}

fn geometry_error(
    error: CurvedEquilibriumArrowGeometryErrorV1,
) -> CurvedEquilibriumArrowGestureErrorV1 {
    match error {
        CurvedEquilibriumArrowGeometryErrorV1::InvalidPoint => {
            CurvedEquilibriumArrowGestureErrorV1::InvalidPoint
        }
        CurvedEquilibriumArrowGeometryErrorV1::CollapsedSpan => {
            CurvedEquilibriumArrowGestureErrorV1::CollapsedSpan
        }
        CurvedEquilibriumArrowGeometryErrorV1::ControlTooNearChord => {
            CurvedEquilibriumArrowGestureErrorV1::ControlTooNearChord
        }
    }
}

fn point3(point: PresentationGesturePoint2V1) -> Point3V1 {
    Point3V1::new(point.x(), point.y(), 0.0).expect("validated finite geometry")
}

fn point2(point: Point3V1) -> PresentationGesturePoint2V1 {
    PresentationGesturePoint2V1::new(point.x(), point.y()).expect("issued finite geometry")
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: &str = r#"<cdml xmlns="urn:ferrum:cdml" version="26.07"/>"#;

    fn point(x: f64, y: f64) -> PresentationGesturePoint2V1 {
        PresentationGesturePoint2V1::new(x, y).expect("finite test point")
    }

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("test snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    #[test]
    fn curved_equilibrium_authoring_issues_two_renderable_lanes_and_commits_closed_cdml() {
        for control_y in [12.0, 35.0] {
            let mut session = DocumentSession::load(EMPTY).expect("empty session");
            let gesture = begin_curved_equilibrium_arrow_gesture_v1(
                &session,
                fence(&session),
                point(0.0, 0.0),
                point(40.0, control_y),
            )
            .expect("valid tangent geometry");
            let preview =
                preview_curved_equilibrium_arrow_gesture_v1(&session, &gesture, point(80.0, 0.0))
                    .expect("valid preview");
            assert!(
                preview
                    .overlay()
                    .lower_axis()
                    .iter()
                    .chain(preview.overlay().upper_axis().iter())
                    .all(|point| point.x().is_finite() && point.y().is_finite())
            );
            let mut prepared =
                prepare_curved_equilibrium_arrow_gesture_v1(&mut session, &gesture, &preview)
                    .expect("renderer-preflighted receipt");
            let committed = commit_curved_equilibrium_arrow_gesture_v1(&mut session, &mut prepared)
                .expect("atomic commit");
            assert!(
                committed
                    .result()
                    .observation()
                    .snapshot()
                    .cdml()
                    .contains("type=\"curved-equilibrium\"")
            );
        }
    }

    #[test]
    fn curved_equilibrium_refuses_reverse_or_cusp_tangents_without_mutation() {
        let session = DocumentSession::load(EMPTY).expect("empty session");
        let source = session.snapshot().expect("snapshot").cdml().to_owned();
        let gesture = begin_curved_equilibrium_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 0.0),
            point(-10.0, 5.0),
        )
        .expect("fence is current");
        assert!(matches!(
            preview_curved_equilibrium_arrow_gesture_v1(&session, &gesture, point(80.0, 0.0)),
            Err(CurvedEquilibriumArrowGestureErrorV1::ControlTooNearChord)
        ));
        assert_eq!(session.snapshot().expect("snapshot").cdml(), source);
    }

    #[test]
    fn curved_equilibrium_receipts_are_fenced_and_single_use() {
        let mut session = DocumentSession::load(EMPTY).expect("empty session");
        let stale = DocumentFenceV1::new(1, *session.snapshot().expect("snapshot").digest());
        assert!(matches!(
            begin_curved_equilibrium_arrow_gesture_v1(
                &session,
                stale,
                point(0.0, 0.0),
                point(40.0, 20.0),
            ),
            Err(CurvedEquilibriumArrowGestureErrorV1::StaleSnapshot)
        ));
        let gesture = begin_curved_equilibrium_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 0.0),
            point(40.0, 20.0),
        )
        .expect("current gesture");
        let preview =
            preview_curved_equilibrium_arrow_gesture_v1(&session, &gesture, point(80.0, 0.0))
                .expect("preview");
        let mut prepared =
            prepare_curved_equilibrium_arrow_gesture_v1(&mut session, &gesture, &preview)
                .expect("prepared receipt");
        commit_curved_equilibrium_arrow_gesture_v1(&mut session, &mut prepared).expect("commit");
        assert!(matches!(
            commit_curved_equilibrium_arrow_gesture_v1(&mut session, &mut prepared),
            Err(CurvedEquilibriumArrowGestureErrorV1::ReplayedGesture)
        ));
    }

    #[test]
    fn curved_equilibrium_capabilities_refuse_an_identical_foreign_session_without_mutation() {
        let mut owner = DocumentSession::load(EMPTY).expect("owner session");
        let mut foreign = DocumentSession::load(EMPTY).expect("foreign session");
        let foreign_source = foreign
            .snapshot()
            .expect("foreign snapshot")
            .cdml()
            .to_owned();
        let gesture = begin_curved_equilibrium_arrow_gesture_v1(
            &owner,
            fence(&owner),
            point(0.0, 0.0),
            point(40.0, 20.0),
        )
        .expect("owner gesture");
        assert!(matches!(
            preview_curved_equilibrium_arrow_gesture_v1(&foreign, &gesture, point(80.0, 0.0)),
            Err(CurvedEquilibriumArrowGestureErrorV1::ForeignSession)
        ));
        assert_eq!(
            foreign.snapshot().expect("foreign snapshot").cdml(),
            foreign_source
        );

        let preview =
            preview_curved_equilibrium_arrow_gesture_v1(&owner, &gesture, point(80.0, 0.0))
                .expect("owner preview");
        let mut prepared =
            prepare_curved_equilibrium_arrow_gesture_v1(&mut owner, &gesture, &preview)
                .expect("owner receipt");
        assert!(matches!(
            commit_curved_equilibrium_arrow_gesture_v1(&mut foreign, &mut prepared),
            Err(CurvedEquilibriumArrowGestureErrorV1::ForeignSession)
        ));
        assert_eq!(
            foreign.snapshot().expect("foreign snapshot").cdml(),
            foreign_source
        );
        commit_curved_equilibrium_arrow_gesture_v1(&mut owner, &mut prepared)
            .expect("foreign refusal preserves owner receipt");
    }
}
