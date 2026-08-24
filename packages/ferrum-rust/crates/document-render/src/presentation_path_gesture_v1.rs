//! Renderer-preflighted multi-point Polyline and Polygon authoring.

use ferrum_document::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityV1, DocumentFenceV1, DocumentSession,
    GeometricLineWidthV1, PRESENTATION_PATH_MAXIMUM_POINTS_V1, PendingCreatePresentationV1,
    PresentationAppearanceV1, PresentationCreateErrorV1, PresentationCreateRequestV1,
    PresentationGesturePoint2V1, PresentationPathGestureErrorV1, PresentationPathGestureV1,
    PresentationPathKindV1, PresentationRootSelectorV1, Rgb24V1, SessionOperationResultV1,
    TransparentOrRgb24V1,
};
use thiserror::Error;

use super::require_fence;

#[derive(Clone, Debug, PartialEq)]
pub struct PresentationPathAppearanceV1 {
    stroke_color: Rgb24V1,
    stroke_width: GeometricLineWidthV1,
    fill_color: Option<Rgb24V1>,
}
impl PresentationPathAppearanceV1 {
    #[must_use]
    pub fn stroke_color(&self) -> &str {
        self.stroke_color.as_str()
    }
    #[must_use]
    pub fn stroke_width(&self) -> f64 {
        self.stroke_width.value()
    }
    #[must_use]
    pub fn fill_color(&self) -> Option<&str> {
        self.fill_color.as_ref().map(Rgb24V1::as_str)
    }
}

#[derive(Clone, Debug)]
/// Opaque, session-fenced candidate for one incremental presentation path.
pub struct PresentationPathRenderGestureV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    kind: PresentationPathKindV1,
    appearance: PresentationPathAppearanceV1,
    points: Vec<PresentationGesturePoint2V1>,
}

impl PresentationPathRenderGestureV1 {
    /// Return accepted vertices in their exact authored order.
    #[must_use]
    pub fn points(&self) -> &[PresentationGesturePoint2V1] {
        &self.points
    }
}

/// Rust-derived progress for one opaque incremental path candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PresentationPathProgressV1 {
    accepted_points: usize,
    minimum_points: usize,
}

impl PresentationPathProgressV1 {
    #[must_use]
    pub const fn accepted_points(self) -> usize {
        self.accepted_points
    }

    #[must_use]
    pub const fn minimum_points(self) -> usize {
        self.minimum_points
    }

    #[must_use]
    pub const fn can_prepare(self) -> bool {
        self.accepted_points >= self.minimum_points
    }
}

/// Immutable Rust-issued display state for an incremental path candidate.
#[derive(Clone, Debug)]
pub struct PresentationPathOverlayV1 {
    gesture: PresentationPathRenderGestureV1,
    hover: Option<PresentationGesturePoint2V1>,
    path: Option<PresentationPathGestureV1>,
}

impl PresentationPathOverlayV1 {
    /// Return the immutable appearance issued by Rust for this display state.
    #[must_use]
    pub const fn appearance(&self) -> &PresentationPathAppearanceV1 {
        &self.gesture.appearance
    }

    #[must_use]
    pub fn accepted_points(&self) -> &[PresentationGesturePoint2V1] {
        &self.gesture.points
    }

    #[must_use]
    pub const fn hover(&self) -> Option<PresentationGesturePoint2V1> {
        self.hover
    }

    #[must_use]
    pub fn path(&self) -> Option<&PresentationPathGestureV1> {
        self.path.as_ref()
    }
}
#[derive(Debug)]
/// Renderer-preflighted path awaiting one commit; unsuccessful commits preserve it for retry.
pub struct PreparedPresentationPathV1 {
    receipt: Option<PendingCreatePresentationV1>,
    identifier: String,
}
#[derive(Clone, Debug)]
/// Result of the single successful commit for a prepared presentation path.
pub struct CommittedPresentationPathV1 {
    root: PresentationRootSelectorV1,
    result: SessionOperationResultV1,
}
impl CommittedPresentationPathV1 {
    #[must_use]
    pub fn root(&self) -> &PresentationRootSelectorV1 {
        &self.root
    }
    #[must_use]
    pub fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationPathRenderCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    ReplayedGesture,
    Cancelled,
    InvalidGeometry,
    RenderPreparation,
    SessionConflict,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationPathRenderRecoveryV1 {
    RefreshAndRestart,
    ChangeGeometry,
    ReduceRequest,
    DocumentUnchanged,
}
#[derive(Clone, Debug, Error, PartialEq)]
pub enum PresentationPathRenderErrorV1 {
    #[error("presentation path gesture snapshot is stale")]
    StaleSnapshot,
    #[error("presentation path gesture belongs to another document session")]
    ForeignSession,
    #[error("presentation path preview belongs to another gesture")]
    MismatchedPreview,
    #[error("presentation path gesture was already committed")]
    ReplayedGesture,
    #[error("presentation path gesture was cancelled without changing the document")]
    Cancelled,
    #[error("{0}")]
    InvalidGeometry(PresentationPathGestureErrorV1),
    #[error("presentation path candidate could not be rendered for preview")]
    RenderPreparation,
    #[error("presentation path commit was rejected by the document session")]
    SessionConflict,
}
impl PresentationPathRenderErrorV1 {
    #[must_use]
    pub const fn category(&self) -> PresentationPathRenderCategoryV1 {
        match self {
            Self::StaleSnapshot => PresentationPathRenderCategoryV1::StaleSnapshot,
            Self::ForeignSession => PresentationPathRenderCategoryV1::ForeignSession,
            Self::MismatchedPreview => PresentationPathRenderCategoryV1::MismatchedPreview,
            Self::ReplayedGesture => PresentationPathRenderCategoryV1::ReplayedGesture,
            Self::Cancelled => PresentationPathRenderCategoryV1::Cancelled,
            Self::InvalidGeometry(_) => PresentationPathRenderCategoryV1::InvalidGeometry,
            Self::RenderPreparation => PresentationPathRenderCategoryV1::RenderPreparation,
            Self::SessionConflict => PresentationPathRenderCategoryV1::SessionConflict,
        }
    }
    #[must_use]
    pub const fn recovery(&self) -> PresentationPathRenderRecoveryV1 {
        match self {
            Self::StaleSnapshot
            | Self::ForeignSession
            | Self::MismatchedPreview
            | Self::ReplayedGesture
            | Self::SessionConflict => PresentationPathRenderRecoveryV1::RefreshAndRestart,
            Self::Cancelled => PresentationPathRenderRecoveryV1::DocumentUnchanged,
            Self::InvalidGeometry(PresentationPathGestureErrorV1::ResourceExhausted) => {
                PresentationPathRenderRecoveryV1::ReduceRequest
            }
            Self::InvalidGeometry(_) => PresentationPathRenderRecoveryV1::ChangeGeometry,
            Self::RenderPreparation => PresentationPathRenderRecoveryV1::DocumentUnchanged,
        }
    }
}

/// Begin a session-fenced incremental path from one current document snapshot.
///
/// The returned opaque handle is accepted only by the issuing session.
pub fn begin_presentation_path_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    kind: PresentationPathKindV1,
) -> Result<PresentationPathRenderGestureV1, PresentationPathRenderErrorV1> {
    require_fence(session, fence).map_err(|_| PresentationPathRenderErrorV1::StaleSnapshot)?;
    Ok(PresentationPathRenderGestureV1 {
        capability: session.authoring_capability_issuer_v1().issue(),
        fence,
        kind,
        appearance: appearance(session, fence)?,
        points: Vec::new(),
    })
}

/// Add exactly one finite scene point to an opaque candidate.
pub fn add_presentation_path_gesture_point_v1(
    session: &DocumentSession,
    gesture: &mut PresentationPathRenderGestureV1,
    point: PresentationGesturePoint2V1,
) -> Result<PresentationPathProgressV1, PresentationPathRenderErrorV1> {
    require_candidate(session, gesture)?;
    if gesture.points.len() >= PRESENTATION_PATH_MAXIMUM_POINTS_V1 {
        return Err(PresentationPathRenderErrorV1::InvalidGeometry(
            PresentationPathGestureErrorV1::ResourceExhausted,
        ));
    }
    if gesture.points.contains(&point) {
        return Err(PresentationPathRenderErrorV1::InvalidGeometry(
            PresentationPathGestureErrorV1::DegenerateGeometry,
        ));
    }
    gesture.points.push(point);
    Ok(progress(gesture.kind, gesture.points.len()))
}

/// Derive immutable display geometry from accepted vertices and one optional hover point.
pub fn preview_incremental_presentation_path_gesture_v1(
    session: &DocumentSession,
    gesture: &PresentationPathRenderGestureV1,
    hover: Option<PresentationGesturePoint2V1>,
) -> Result<PresentationPathOverlayV1, PresentationPathRenderErrorV1> {
    require_candidate(session, gesture)?;
    if let Some(point) = hover {
        if gesture.points.contains(&point) {
            return Err(PresentationPathRenderErrorV1::InvalidGeometry(
                PresentationPathGestureErrorV1::DegenerateGeometry,
            ));
        }
    }
    // Hover is display-only. Persistent candidates contain accepted points only.
    let path = (gesture.points.len() >= minimum_points(gesture.kind))
        .then(|| PresentationPathGestureV1::new(gesture.kind, gesture.points.clone()))
        .transpose()
        .map_err(PresentationPathRenderErrorV1::InvalidGeometry)?;
    Ok(PresentationPathOverlayV1 {
        gesture: gesture.clone(),
        hover,
        path,
    })
}

/// Consume a path candidate without a document transition.
pub fn cancel_presentation_path_gesture_v1(
    session: &DocumentSession,
    gesture: &PresentationPathRenderGestureV1,
) -> Result<(), PresentationPathRenderErrorV1> {
    if !gesture
        .capability
        .belongs_to(&session.authoring_capability_issuer_v1())
    {
        return Err(PresentationPathRenderErrorV1::ForeignSession);
    }
    gesture
        .capability
        .consume_without_commit(&session.authoring_capability_issuer_v1())
        .map_err(|error| match error {
            AuthoringCapabilityAccessErrorV1::ForeignSession => {
                PresentationPathRenderErrorV1::ForeignSession
            }
            AuthoringCapabilityAccessErrorV1::Replayed => {
                PresentationPathRenderErrorV1::ReplayedGesture
            }
        })?;
    Err(PresentationPathRenderErrorV1::Cancelled)
}

/// Renderer-preflight an incremental candidate only from its Rust-issued overlay.
///
/// Preparation rejects foreign or mismatched handles and returns one receipt for commit.
pub fn prepare_incremental_presentation_path_gesture_v1(
    session: &mut DocumentSession,
    gesture: &PresentationPathRenderGestureV1,
    overlay: &PresentationPathOverlayV1,
) -> Result<PreparedPresentationPathV1, PresentationPathRenderErrorV1> {
    require_candidate(session, gesture)?;
    if !overlay
        .gesture
        .capability
        .belongs_to(&session.authoring_capability_issuer_v1())
        || !overlay
            .gesture
            .capability
            .same_capability(&gesture.capability)
        || overlay.gesture.points != gesture.points
    {
        return Err(PresentationPathRenderErrorV1::MismatchedPreview);
    }
    let path = overlay
        .path
        .as_ref()
        .ok_or(PresentationPathRenderErrorV1::InvalidGeometry(
            PresentationPathGestureErrorV1::InsufficientPoints,
        ))?;
    prepare_path(session, gesture, path)
}

fn prepare_path(
    session: &mut DocumentSession,
    gesture: &PresentationPathRenderGestureV1,
    path: &PresentationPathGestureV1,
) -> Result<PreparedPresentationPathV1, PresentationPathRenderErrorV1> {
    let pending = session
        .prepare_create_presentation_v1(
            &gesture.capability,
            gesture.fence,
            PresentationCreateRequestV1::Path {
                path: path.clone(),
                appearance: PresentationAppearanceV1::new(
                    gesture.appearance.stroke_color.clone(),
                    gesture.appearance.stroke_width,
                    gesture.appearance.fill_color.clone(),
                ),
            },
        )
        .map_err(map_presentation_create_error)?;
    let identifier = pending.identifier().as_str().to_owned();
    Ok(PreparedPresentationPathV1 {
        receipt: Some(pending),
        identifier,
    })
}

fn require_candidate(
    session: &DocumentSession,
    gesture: &PresentationPathRenderGestureV1,
) -> Result<(), PresentationPathRenderErrorV1> {
    if !gesture
        .capability
        .belongs_to(&session.authoring_capability_issuer_v1())
    {
        return Err(PresentationPathRenderErrorV1::ForeignSession);
    }
    match gesture
        .capability
        .claim_for_commit(&session.authoring_capability_issuer_v1())
    {
        Ok(claim) => drop(claim),
        Err(AuthoringCapabilityAccessErrorV1::ForeignSession) => {
            return Err(PresentationPathRenderErrorV1::ForeignSession);
        }
        Err(AuthoringCapabilityAccessErrorV1::Replayed) => {
            return Err(PresentationPathRenderErrorV1::ReplayedGesture);
        }
    }
    require_fence(session, gesture.fence).map_err(|_| PresentationPathRenderErrorV1::StaleSnapshot)
}

const fn minimum_points(kind: PresentationPathKindV1) -> usize {
    match kind {
        PresentationPathKindV1::Polyline => 2,
        PresentationPathKindV1::Polygon => 3,
    }
}

const fn progress(
    kind: PresentationPathKindV1,
    accepted_points: usize,
) -> PresentationPathProgressV1 {
    PresentationPathProgressV1 {
        accepted_points,
        minimum_points: minimum_points(kind),
    }
}

/// Commit one prepared presentation path exactly once.
///
/// A non-successful transaction preserves the prepared receipt for retry; a successful commit consumes it.
pub fn commit_presentation_path_gesture_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedPresentationPathV1,
) -> Result<CommittedPresentationPathV1, PresentationPathRenderErrorV1> {
    let mut pending = prepared
        .receipt
        .take()
        .ok_or(PresentationPathRenderErrorV1::ReplayedGesture)?;
    if pending.identifier().as_str() != prepared.identifier {
        return Err(PresentationPathRenderErrorV1::RenderPreparation);
    }
    let result = session
        .commit_create_presentation_v1(&mut pending)
        .map_err(map_presentation_create_error);
    let result = match result {
        Ok(result) => result,
        Err(error) => {
            prepared.receipt = Some(pending);
            return Err(error);
        }
    };
    let root = PresentationRootSelectorV1::new(&prepared.identifier, pending.root_kind())
        .expect("prepared receipt contains a valid generated selector");
    Ok(CommittedPresentationPathV1 { root, result })
}

fn map_presentation_create_error(
    error: PresentationCreateErrorV1,
) -> PresentationPathRenderErrorV1 {
    match error {
        PresentationCreateErrorV1::ForeignSession => PresentationPathRenderErrorV1::ForeignSession,
        PresentationCreateErrorV1::StaleSnapshot => PresentationPathRenderErrorV1::StaleSnapshot,
        PresentationCreateErrorV1::Replayed => PresentationPathRenderErrorV1::ReplayedGesture,
        PresentationCreateErrorV1::SessionConflict => {
            PresentationPathRenderErrorV1::SessionConflict
        }
        PresentationCreateErrorV1::RendererAdmission => {
            PresentationPathRenderErrorV1::RenderPreparation
        }
    }
}

fn appearance(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<PresentationPathAppearanceV1, PresentationPathRenderErrorV1> {
    let observation = session
        .observe(fence.revision())
        .map_err(|_| PresentationPathRenderErrorV1::RenderPreparation)?;
    let standard = observation.projection().drawing_standard();
    Ok(PresentationPathAppearanceV1 {
        stroke_color: standard
            .and_then(|value| value.line_color())
            .cloned()
            .unwrap_or_else(|| Rgb24V1::new("#000000").expect("closed built-in colour")),
        stroke_width: GeometricLineWidthV1::new(
            standard
                .and_then(|value| value.line_width())
                .map_or(1.0, |value| value.value()),
        )
        .ok_or(PresentationPathRenderErrorV1::RenderPreparation)?,
        fill_color: standard
            .and_then(|value| value.area_color())
            .and_then(|value| match value {
                TransparentOrRgb24V1::Transparent => None,
                TransparentOrRgb24V1::Rgb24(color) => Some(color.clone()),
            }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }
    fn point(x: f64, y: f64) -> ferrum_document::PresentationGesturePoint2V1 {
        ferrum_document::PresentationGesturePoint2V1::new(x, y).expect("point")
    }
    #[test]
    fn incremental_polyline_commits_history_and_reopens() {
        let mut session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"/>")
                .expect("session");
        let mut gesture = begin_presentation_path_gesture_v1(
            &session,
            fence(&session),
            PresentationPathKindV1::Polyline,
        )
        .expect("gesture");
        for point in [point(30.0, 0.0), point(50.0, 10.0)] {
            add_presentation_path_gesture_point_v1(&session, &mut gesture, point)
                .expect("accepted point");
        }
        let overlay = preview_incremental_presentation_path_gesture_v1(&session, &gesture, None)
            .expect("overlay");
        let mut prepared =
            prepare_incremental_presentation_path_gesture_v1(&mut session, &gesture, &overlay)
                .expect("prepared");
        let commit = commit_presentation_path_gesture_v1(&mut session, &mut prepared)
            .expect("polyline commit");
        assert_eq!(
            commit.root().kind(),
            ferrum_document::PresentationRecordKindV1::Polyline
        );
        let committed = commit.result().observation().snapshot();
        let undo = session.undo(committed.revision()).expect("undo");
        let redo = session
            .redo(undo.observation().snapshot().revision())
            .expect("redo");
        let reopened = DocumentSession::load(redo.observation().snapshot().cdml()).expect("reopen");
        assert!(
            reopened
                .snapshot()
                .expect("snapshot")
                .cdml()
                .contains(commit.root().presentation_id().as_str())
        );
    }

    #[test]
    fn incremental_polygon_preserves_source_order_history_and_reopen() {
        let mut session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"/>")
                .expect("session");
        let mut gesture = begin_presentation_path_gesture_v1(
            &session,
            fence(&session),
            PresentationPathKindV1::Polygon,
        )
        .expect("gesture");
        let first = add_presentation_path_gesture_point_v1(&session, &mut gesture, point(0.0, 0.0))
            .expect("first point");
        assert_eq!(first.accepted_points(), 1);
        assert_eq!(first.minimum_points(), 3);
        assert!(!first.can_prepare());
        let incomplete = preview_incremental_presentation_path_gesture_v1(&session, &gesture, None)
            .expect("incomplete overlay");
        assert!(incomplete.path().is_none());
        add_presentation_path_gesture_point_v1(&session, &mut gesture, point(20.0, 0.0))
            .expect("second point");
        let final_progress =
            add_presentation_path_gesture_point_v1(&session, &mut gesture, point(0.0, 10.0))
                .expect("third point");
        assert!(final_progress.can_prepare());
        let overlay = preview_incremental_presentation_path_gesture_v1(&session, &gesture, None)
            .expect("complete overlay");
        let mut prepared =
            prepare_incremental_presentation_path_gesture_v1(&mut session, &gesture, &overlay)
                .expect("prepared");
        let commit =
            commit_presentation_path_gesture_v1(&mut session, &mut prepared).expect("commit");
        let committed = commit.result().observation().snapshot();
        let cdml = committed.cdml();
        let first_point = cdml.find("x=\"0\" y=\"0\"").expect("first point");
        let second_point = cdml.find("x=\"20\" y=\"0\"").expect("second point");
        let third_point = cdml.find("x=\"0\" y=\"10\"").expect("third point");
        assert!(first_point < second_point && second_point < third_point);
        let undo = session.undo(committed.revision()).expect("undo");
        assert!(
            !undo
                .observation()
                .snapshot()
                .cdml()
                .contains(commit.root().presentation_id().as_str())
        );
        let redo = session
            .redo(undo.observation().snapshot().revision())
            .expect("redo");
        let reopened = DocumentSession::load(redo.observation().snapshot().cdml()).expect("reopen");
        assert!(
            reopened
                .snapshot()
                .expect("reopened snapshot")
                .cdml()
                .contains(commit.root().presentation_id().as_str())
        );
    }

    #[test]
    fn incremental_refusal_and_cancellation_never_mutate() {
        let mut session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"/>")
                .expect("session");
        let before = session.snapshot().expect("before");
        let mut gesture = begin_presentation_path_gesture_v1(
            &session,
            fence(&session),
            PresentationPathKindV1::Polyline,
        )
        .expect("gesture");
        add_presentation_path_gesture_point_v1(&session, &mut gesture, point(0.0, 0.0))
            .expect("first point");
        assert!(matches!(
            add_presentation_path_gesture_point_v1(&session, &mut gesture, point(0.0, 0.0)),
            Err(PresentationPathRenderErrorV1::InvalidGeometry(
                PresentationPathGestureErrorV1::DegenerateGeometry
            ))
        ));
        assert!(matches!(
            cancel_presentation_path_gesture_v1(&session, &gesture),
            Err(PresentationPathRenderErrorV1::Cancelled)
        ));
        let cancelled = PresentationPathRenderErrorV1::Cancelled;
        assert_eq!(
            cancelled.category(),
            PresentationPathRenderCategoryV1::Cancelled
        );
        assert_eq!(
            cancelled.recovery(),
            PresentationPathRenderRecoveryV1::DocumentUnchanged
        );
        assert_eq!(session.snapshot().expect("after cancellation"), before);
        assert!(matches!(
            preview_incremental_presentation_path_gesture_v1(&session, &gesture, None),
            Err(PresentationPathRenderErrorV1::ReplayedGesture)
        ));
        let stale = begin_presentation_path_gesture_v1(
            &session,
            fence(&session),
            PresentationPathKindV1::Polyline,
        )
        .expect("stale gesture");
        let mut revision_advance = session
            .prepare_complete_cdml_mutation_v1(fence(&session), before.cdml())
            .expect("prepare revision advance");
        session
            .commit_complete_cdml_mutation_v1(&mut revision_advance)
            .expect("advance revision");
        let after_advance = session.snapshot().expect("advanced snapshot");
        assert!(matches!(
            preview_incremental_presentation_path_gesture_v1(&session, &stale, None),
            Err(PresentationPathRenderErrorV1::StaleSnapshot)
        ));
        assert_eq!(
            session.snapshot().expect("after stale refusal"),
            after_advance
        );
    }

    #[test]
    fn resource_refusal_preserves_its_distinct_recovery() {
        let refusal = PresentationPathRenderErrorV1::InvalidGeometry(
            PresentationPathGestureErrorV1::ResourceExhausted,
        );

        assert_eq!(
            refusal.recovery(),
            PresentationPathRenderRecoveryV1::ReduceRequest
        );
    }

    #[test]
    fn render_preflight_refusal_is_atomic_for_incremental_candidate() {
        let source = concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"><polygon id=\"legacy\"/>",
            "</cdml>"
        );
        let mut session = DocumentSession::load(source).expect("session");
        let before = session.snapshot().expect("before");
        let mut gesture = begin_presentation_path_gesture_v1(
            &session,
            fence(&session),
            PresentationPathKindV1::Polygon,
        )
        .expect("gesture");
        for point in [point(0.0, 0.0), point(20.0, 0.0), point(0.0, 10.0)] {
            add_presentation_path_gesture_point_v1(&session, &mut gesture, point)
                .expect("accepted point");
        }
        let overlay = preview_incremental_presentation_path_gesture_v1(&session, &gesture, None)
            .expect("overlay");
        assert!(matches!(
            prepare_incremental_presentation_path_gesture_v1(&mut session, &gesture, &overlay),
            Err(PresentationPathRenderErrorV1::RenderPreparation)
        ));
        assert_eq!(session.snapshot().expect("after refusal"), before);
    }

    #[test]
    fn hover_is_display_only_and_cannot_become_persistent_geometry() {
        let mut session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"/>")
                .expect("session");
        let before = session.snapshot().expect("before");
        let mut gesture = begin_presentation_path_gesture_v1(
            &session,
            fence(&session),
            PresentationPathKindV1::Polygon,
        )
        .expect("gesture");
        for point in [point(0.0, 0.0), point(20.0, 0.0)] {
            add_presentation_path_gesture_point_v1(&session, &mut gesture, point)
                .expect("accepted point");
        }
        let incomplete = preview_incremental_presentation_path_gesture_v1(
            &session,
            &gesture,
            Some(point(0.0, 10.0)),
        )
        .expect("hover overlay");
        assert!(incomplete.path().is_none());
        assert!(matches!(
            prepare_incremental_presentation_path_gesture_v1(&mut session, &gesture, &incomplete),
            Err(PresentationPathRenderErrorV1::InvalidGeometry(
                PresentationPathGestureErrorV1::InsufficientPoints
            ))
        ));
        assert_eq!(session.snapshot().expect("after hover refusal"), before);

        add_presentation_path_gesture_point_v1(&session, &mut gesture, point(0.0, 10.0))
            .expect("third accepted point");
        let overlay = preview_incremental_presentation_path_gesture_v1(
            &session,
            &gesture,
            Some(point(40.0, 40.0)),
        )
        .expect("complete overlay with hover");
        assert_eq!(overlay.path().expect("accepted path").points().len(), 3);
        let mut prepared =
            prepare_incremental_presentation_path_gesture_v1(&mut session, &gesture, &overlay)
                .expect("prepared");
        let commit =
            commit_presentation_path_gesture_v1(&mut session, &mut prepared).expect("commit");
        assert!(
            !commit
                .result()
                .observation()
                .snapshot()
                .cdml()
                .contains("x=\"40\" y=\"40\"")
        );
    }

    #[test]
    fn foreign_and_stale_incremental_receipts_cannot_mutate() {
        let source = "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"/>";
        let mut session = DocumentSession::load(source).expect("session");
        let mut foreign = DocumentSession::load(source).expect("foreign session");
        let mut gesture = begin_presentation_path_gesture_v1(
            &session,
            fence(&session),
            PresentationPathKindV1::Polyline,
        )
        .expect("gesture");
        for point in [point(0.0, 0.0), point(20.0, 0.0)] {
            add_presentation_path_gesture_point_v1(&session, &mut gesture, point)
                .expect("accepted point");
        }
        let overlay = preview_incremental_presentation_path_gesture_v1(&session, &gesture, None)
            .expect("overlay");
        let foreign_before = foreign.snapshot().expect("foreign before");
        assert!(matches!(
            prepare_incremental_presentation_path_gesture_v1(&mut foreign, &gesture, &overlay),
            Err(PresentationPathRenderErrorV1::ForeignSession)
        ));
        assert_eq!(foreign.snapshot().expect("foreign after"), foreign_before);

        let mut prepared =
            prepare_incremental_presentation_path_gesture_v1(&mut session, &gesture, &overlay)
                .expect("prepared");
        assert!(matches!(
            commit_presentation_path_gesture_v1(&mut foreign, &mut prepared),
            Err(PresentationPathRenderErrorV1::ForeignSession)
        ));
        assert_eq!(
            foreign.snapshot().expect("foreign commit after"),
            foreign_before
        );

        let before_advance = session.snapshot().expect("before advance");
        let mut revision_advance = session
            .prepare_complete_cdml_mutation_v1(fence(&session), before_advance.cdml())
            .expect("prepare revision advance");
        session
            .commit_complete_cdml_mutation_v1(&mut revision_advance)
            .expect("advance revision");
        let advanced = session.snapshot().expect("advanced");
        assert!(matches!(
            commit_presentation_path_gesture_v1(&mut session, &mut prepared),
            Err(PresentationPathRenderErrorV1::StaleSnapshot)
        ));
        assert_eq!(session.snapshot().expect("after stale commit"), advanced);
    }
}
