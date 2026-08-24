//! Renderer-preflighted, Rust-owned quadratic electron-arrow authoring.

use ferrum_document::{
    ArrowProjectionKindV1, AuthoringCapabilityV1, CurvedTerminalArrowKindV1, DocumentFenceV1,
    DocumentSession, PendingCreatePresentationV1, Point3V1, PositiveFiniteV1,
    PresentationArrowPreviewRequestV1, PresentationCreateErrorV1, PresentationCreateRequestV1,
    PresentationFactProvenanceV1, PresentationGesturePoint2V1, PresentationRootSelectorV1,
    PresentationStrokeV1, Rgb24V1, SessionOperationResultV1,
};
use ferrum_render::{PresentationRenderPlanV1, lower_arrow_preview_v1};
use thiserror::Error;

const MINIMUM_SPAN_PT: f64 = 2.0;
const MINIMUM_CONTROL_DISTANCE_PT: f64 = 1.0;
const MAXIMUM_EXTENT_PT: f64 = 20_000.0;

#[derive(Clone, Debug)]
pub struct CurvedElectronArrowGestureV1 {
    kind: CurvedTerminalArrowKindV1,
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
}

#[derive(Clone, Debug)]
pub struct CurvedElectronArrowPreviewV1 {
    gesture: CurvedElectronArrowGestureV1,
    end: PresentationGesturePoint2V1,
    plan: PresentationRenderPlanV1,
}

impl CurvedElectronArrowPreviewV1 {
    #[must_use]
    pub const fn plan(&self) -> &PresentationRenderPlanV1 {
        &self.plan
    }
}

#[derive(Debug)]
pub struct PreparedCurvedElectronArrowV1 {
    pending: Option<PendingCreatePresentationV1>,
    identifier: String,
}

#[derive(Clone, Debug)]
pub struct CommittedCurvedElectronArrowV1 {
    root: PresentationRootSelectorV1,
    result: SessionOperationResultV1,
}

impl CommittedCurvedElectronArrowV1 {
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
pub enum CurvedElectronArrowGestureCategoryV1 {
    ForeignSession,
    StaleSnapshot,
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
pub enum CurvedElectronArrowGestureRecoveryV1 {
    RefreshAndRestart,
    ChangeGeometry,
    DocumentUnchanged,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum CurvedElectronArrowGestureErrorV1 {
    #[error("curved terminal-arrow belongs to a different document session")]
    ForeignSession,
    #[error("curved terminal-arrow snapshot is stale")]
    StaleSnapshot,
    #[error("curved terminal-arrow preview belongs to a different gesture")]
    MismatchedPreview,
    #[error("curved terminal-arrow receipt was already consumed")]
    ReplayedGesture,
    #[error("curved terminal-arrow point is invalid")]
    InvalidPoint,
    #[error("curved terminal-arrow start and end are too close")]
    CollapsedSpan,
    #[error("curved terminal-arrow control point is too close to its chord")]
    ControlTooNearChord,
    #[error("curved terminal-arrow exceeds the geometry limit")]
    ExceedsGeometryLimit,
    #[error("curved terminal-arrow candidate failed renderer preflight")]
    RenderPreparation,
    #[error("curved terminal-arrow session transaction failed")]
    SessionConflict,
}

impl CurvedElectronArrowGestureErrorV1 {
    #[must_use]
    pub const fn category(&self) -> CurvedElectronArrowGestureCategoryV1 {
        match self {
            Self::ForeignSession => CurvedElectronArrowGestureCategoryV1::ForeignSession,
            Self::StaleSnapshot => CurvedElectronArrowGestureCategoryV1::StaleSnapshot,
            Self::MismatchedPreview => CurvedElectronArrowGestureCategoryV1::MismatchedPreview,
            Self::ReplayedGesture => CurvedElectronArrowGestureCategoryV1::ReplayedGesture,
            Self::InvalidPoint => CurvedElectronArrowGestureCategoryV1::InvalidPoint,
            Self::CollapsedSpan => CurvedElectronArrowGestureCategoryV1::CollapsedSpan,
            Self::ControlTooNearChord => CurvedElectronArrowGestureCategoryV1::ControlTooNearChord,
            Self::ExceedsGeometryLimit => {
                CurvedElectronArrowGestureCategoryV1::ExceedsGeometryLimit
            }
            Self::RenderPreparation => CurvedElectronArrowGestureCategoryV1::RenderPreparation,
            Self::SessionConflict => CurvedElectronArrowGestureCategoryV1::SessionConflict,
        }
    }
    #[must_use]
    pub const fn recovery(&self) -> CurvedElectronArrowGestureRecoveryV1 {
        match self {
            Self::ForeignSession
            | Self::StaleSnapshot
            | Self::MismatchedPreview
            | Self::ReplayedGesture
            | Self::SessionConflict => CurvedElectronArrowGestureRecoveryV1::RefreshAndRestart,
            Self::CollapsedSpan | Self::ControlTooNearChord | Self::ExceedsGeometryLimit => {
                CurvedElectronArrowGestureRecoveryV1::ChangeGeometry
            }
            Self::InvalidPoint | Self::RenderPreparation => {
                CurvedElectronArrowGestureRecoveryV1::DocumentUnchanged
            }
        }
    }
}

pub fn begin_curved_electron_arrow_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
) -> Result<CurvedElectronArrowGestureV1, CurvedElectronArrowGestureErrorV1> {
    begin_curved_terminal_arrow_gesture_v1(
        session,
        fence,
        start,
        control,
        CurvedTerminalArrowKindV1::Electron,
    )
}

fn begin_curved_terminal_arrow_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
    kind: CurvedTerminalArrowKindV1,
) -> Result<CurvedElectronArrowGestureV1, CurvedElectronArrowGestureErrorV1> {
    require_fence(session, fence)?;
    Ok(CurvedElectronArrowGestureV1 {
        kind,
        capability: session.authoring_capability_issuer_v1().issue(),
        fence,
        start,
        control,
    })
}

pub fn preview_curved_electron_arrow_gesture_v1(
    session: &DocumentSession,
    gesture: &CurvedElectronArrowGestureV1,
    end: PresentationGesturePoint2V1,
) -> Result<CurvedElectronArrowPreviewV1, CurvedElectronArrowGestureErrorV1> {
    if !gesture
        .capability
        .belongs_to(&session.authoring_capability_issuer_v1())
    {
        return Err(CurvedElectronArrowGestureErrorV1::ForeignSession);
    }
    require_fence(session, gesture.fence)?;
    let plan = preview_plan(gesture.kind, gesture.start, gesture.control, end)?;
    Ok(CurvedElectronArrowPreviewV1 {
        gesture: gesture.clone(),
        end,
        plan,
    })
}

pub fn prepare_curved_electron_arrow_gesture_v1(
    session: &mut DocumentSession,
    gesture: &CurvedElectronArrowGestureV1,
    preview: &CurvedElectronArrowPreviewV1,
) -> Result<PreparedCurvedElectronArrowV1, CurvedElectronArrowGestureErrorV1> {
    if !gesture
        .capability
        .belongs_to(&session.authoring_capability_issuer_v1())
    {
        return Err(CurvedElectronArrowGestureErrorV1::ForeignSession);
    }
    if !gesture
        .capability
        .same_capability(&preview.gesture.capability)
    {
        return Err(CurvedElectronArrowGestureErrorV1::MismatchedPreview);
    }
    require_fence(session, gesture.fence)?;
    let pending = session
        .prepare_create_presentation_v1(
            &gesture.capability,
            gesture.fence,
            PresentationCreateRequestV1::CurvedTerminalArrow {
                kind: gesture.kind,
                start: gesture.start,
                control: gesture.control,
                end: preview.end,
            },
        )
        .map_err(map_presentation_create_error)?;
    let identifier = pending.identifier().as_str().to_owned();
    Ok(PreparedCurvedElectronArrowV1 {
        pending: Some(pending),
        identifier,
    })
}

pub fn commit_curved_electron_arrow_gesture_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedCurvedElectronArrowV1,
) -> Result<CommittedCurvedElectronArrowV1, CurvedElectronArrowGestureErrorV1> {
    if prepared
        .pending
        .as_ref()
        .ok_or(CurvedElectronArrowGestureErrorV1::ReplayedGesture)?
        .identifier()
        .as_str()
        != prepared.identifier
    {
        return Err(CurvedElectronArrowGestureErrorV1::RenderPreparation);
    }
    let mut pending = prepared
        .pending
        .take()
        .ok_or(CurvedElectronArrowGestureErrorV1::ReplayedGesture)?;
    let result = (|| {
        session
            .commit_create_presentation_v1(&mut pending)
            .map_err(map_presentation_create_error)
    })();
    match result {
        Ok(result) => {
            let root = PresentationRootSelectorV1::new(&prepared.identifier, pending.root_kind())
                .expect("generated electron arrow identifier is valid");
            Ok(CommittedCurvedElectronArrowV1 { root, result })
        }
        Err(error) => {
            prepared.pending = Some(pending);
            Err(error)
        }
    }
}

fn map_presentation_create_error(
    error: PresentationCreateErrorV1,
) -> CurvedElectronArrowGestureErrorV1 {
    match error {
        PresentationCreateErrorV1::ForeignSession => {
            CurvedElectronArrowGestureErrorV1::ForeignSession
        }
        PresentationCreateErrorV1::StaleSnapshot => {
            CurvedElectronArrowGestureErrorV1::StaleSnapshot
        }
        PresentationCreateErrorV1::Replayed => CurvedElectronArrowGestureErrorV1::ReplayedGesture,
        PresentationCreateErrorV1::SessionConflict => {
            CurvedElectronArrowGestureErrorV1::SessionConflict
        }
        PresentationCreateErrorV1::RendererAdmission => {
            CurvedElectronArrowGestureErrorV1::RenderPreparation
        }
    }
}

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), CurvedElectronArrowGestureErrorV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| CurvedElectronArrowGestureErrorV1::SessionConflict)?;
    (snapshot.revision() == fence.revision() && snapshot.digest() == &fence.digest())
        .then_some(())
        .ok_or(CurvedElectronArrowGestureErrorV1::StaleSnapshot)
}

fn preview_plan(
    kind: CurvedTerminalArrowKindV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
    end: PresentationGesturePoint2V1,
) -> Result<PresentationRenderPlanV1, CurvedElectronArrowGestureErrorV1> {
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let span = dx.hypot(dy);
    if !span.is_finite() || span < MINIMUM_SPAN_PT {
        return Err(CurvedElectronArrowGestureErrorV1::CollapsedSpan);
    }
    if [start, control, end]
        .into_iter()
        .any(|point| point.x().abs() > MAXIMUM_EXTENT_PT || point.y().abs() > MAXIMUM_EXTENT_PT)
    {
        return Err(CurvedElectronArrowGestureErrorV1::ExceedsGeometryLimit);
    }
    let distance = ((control.x() - start.x()) * dy - (control.y() - start.y()) * dx).abs() / span;
    if !distance.is_finite() || distance < MINIMUM_CONTROL_DISTANCE_PT {
        return Err(CurvedElectronArrowGestureErrorV1::ControlTooNearChord);
    }
    let tangent_x = end.x() - control.x();
    let tangent_y = end.y() - control.y();
    let tangent = tangent_x.hypot(tangent_y);
    if tangent < MINIMUM_CONTROL_DISTANCE_PT {
        return Err(CurvedElectronArrowGestureErrorV1::ControlTooNearChord);
    }
    let request = PresentationArrowPreviewRequestV1::new(
        vec![point3(start), point3(control), point3(end)],
        ArrowProjectionKindV1::CurvedTerminal {
            terminal_kind: kind,
        },
        builtin_stroke(),
    )
    .map_err(|_| CurvedElectronArrowGestureErrorV1::InvalidPoint)?;
    lower_arrow_preview_v1(&request)
        .map_err(|_| CurvedElectronArrowGestureErrorV1::ControlTooNearChord)
}

fn point3(point: PresentationGesturePoint2V1) -> Point3V1 {
    Point3V1::new(point.x(), point.y(), 0.0).expect("validated finite geometry")
}

fn builtin_stroke() -> PresentationStrokeV1 {
    PresentationStrokeV1::new(
        Rgb24V1::new("#000000").expect("closed builtin arrow color is valid"),
        PresentationFactProvenanceV1::Builtin,
        PositiveFiniteV1::new(1.0).expect("closed builtin arrow width is positive"),
        PresentationFactProvenanceV1::Builtin,
    )
    .expect("closed builtin arrow stroke is coherent")
}

/// Closed retro-arrow aliases retain the trusted opaque lifecycle while the
/// document-owned policy selects the persisted `type="retro"` grammar.
pub type CurvedRetroArrowGestureV1 = CurvedElectronArrowGestureV1;
pub type CurvedRetroArrowPreviewV1 = CurvedElectronArrowPreviewV1;
pub type PreparedCurvedRetroArrowV1 = PreparedCurvedElectronArrowV1;
pub type CommittedCurvedRetroArrowV1 = CommittedCurvedElectronArrowV1;
pub type CurvedRetroArrowGestureCategoryV1 = CurvedElectronArrowGestureCategoryV1;
pub type CurvedRetroArrowGestureRecoveryV1 = CurvedElectronArrowGestureRecoveryV1;
pub type CurvedRetroArrowGestureErrorV1 = CurvedElectronArrowGestureErrorV1;

pub fn begin_curved_retro_arrow_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
) -> Result<CurvedRetroArrowGestureV1, CurvedRetroArrowGestureErrorV1> {
    begin_curved_terminal_arrow_gesture_v1(
        session,
        fence,
        start,
        control,
        CurvedTerminalArrowKindV1::Retro,
    )
}

pub fn preview_curved_retro_arrow_gesture_v1(
    session: &DocumentSession,
    gesture: &CurvedRetroArrowGestureV1,
    end: PresentationGesturePoint2V1,
) -> Result<CurvedRetroArrowPreviewV1, CurvedRetroArrowGestureErrorV1> {
    preview_curved_electron_arrow_gesture_v1(session, gesture, end)
}

pub fn prepare_curved_retro_arrow_gesture_v1(
    session: &mut DocumentSession,
    gesture: &CurvedRetroArrowGestureV1,
    preview: &CurvedRetroArrowPreviewV1,
) -> Result<PreparedCurvedRetroArrowV1, CurvedRetroArrowGestureErrorV1> {
    prepare_curved_electron_arrow_gesture_v1(session, gesture, preview)
}

pub fn commit_curved_retro_arrow_gesture_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedCurvedRetroArrowV1,
) -> Result<CommittedCurvedRetroArrowV1, CurvedRetroArrowGestureErrorV1> {
    commit_curved_electron_arrow_gesture_v1(session, prepared)
}

/// Closed curved-normal-reaction-arrow aliases retain the shared opaque lifecycle.
pub type CurvedNormalReactionArrowGestureV1 = CurvedElectronArrowGestureV1;
pub type CurvedNormalReactionArrowPreviewV1 = CurvedElectronArrowPreviewV1;
pub type PreparedCurvedNormalReactionArrowV1 = PreparedCurvedElectronArrowV1;
pub type CommittedCurvedNormalReactionArrowV1 = CommittedCurvedElectronArrowV1;
pub type CurvedNormalReactionArrowGestureCategoryV1 = CurvedElectronArrowGestureCategoryV1;
pub type CurvedNormalReactionArrowGestureRecoveryV1 = CurvedElectronArrowGestureRecoveryV1;
pub type CurvedNormalReactionArrowGestureErrorV1 = CurvedElectronArrowGestureErrorV1;

pub fn begin_curved_normal_reaction_arrow_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
) -> Result<CurvedNormalReactionArrowGestureV1, CurvedNormalReactionArrowGestureErrorV1> {
    begin_curved_terminal_arrow_gesture_v1(
        session,
        fence,
        start,
        control,
        CurvedTerminalArrowKindV1::Normal,
    )
}

pub fn preview_curved_normal_reaction_arrow_gesture_v1(
    session: &DocumentSession,
    gesture: &CurvedNormalReactionArrowGestureV1,
    end: PresentationGesturePoint2V1,
) -> Result<CurvedNormalReactionArrowPreviewV1, CurvedNormalReactionArrowGestureErrorV1> {
    preview_curved_electron_arrow_gesture_v1(session, gesture, end)
}

pub fn prepare_curved_normal_reaction_arrow_gesture_v1(
    session: &mut DocumentSession,
    gesture: &CurvedNormalReactionArrowGestureV1,
    preview: &CurvedNormalReactionArrowPreviewV1,
) -> Result<PreparedCurvedNormalReactionArrowV1, CurvedNormalReactionArrowGestureErrorV1> {
    prepare_curved_electron_arrow_gesture_v1(session, gesture, preview)
}

pub fn commit_curved_normal_reaction_arrow_gesture_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedCurvedNormalReactionArrowV1,
) -> Result<CommittedCurvedNormalReactionArrowV1, CurvedNormalReactionArrowGestureErrorV1> {
    commit_curved_electron_arrow_gesture_v1(session, prepared)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderInteractionModifierV1, RenderInteractionQueryV1, RenderInteractionSessionV1,
    };
    use ferrum_document::TopLevelRootKindV1;
    const EMPTY: &str = r#"<cdml xmlns="urn:ferrum:cdml" version="26.07"/>"#;
    fn point(x: f64, y: f64) -> PresentationGesturePoint2V1 {
        PresentationGesturePoint2V1::new(x, y).expect("point")
    }
    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().unwrap();
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    #[test]
    fn retro_arrow_uses_the_shared_geometry_and_persists_its_closed_type() {
        let mut session = DocumentSession::load(EMPTY).unwrap();
        let gesture = begin_curved_retro_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 0.0),
            point(20.0, 20.0),
        )
        .unwrap();
        let preview =
            preview_curved_retro_arrow_gesture_v1(&session, &gesture, point(40.0, 0.0)).unwrap();
        let mut prepared =
            prepare_curved_retro_arrow_gesture_v1(&mut session, &gesture, &preview).unwrap();
        let committed = commit_curved_retro_arrow_gesture_v1(&mut session, &mut prepared).unwrap();
        assert!(
            committed
                .result()
                .observation()
                .snapshot()
                .cdml()
                .contains("type=\"retro\"")
        );
        assert!(matches!(
            commit_curved_retro_arrow_gesture_v1(&mut session, &mut prepared),
            Err(CurvedRetroArrowGestureErrorV1::ReplayedGesture)
        ));
    }

    #[test]
    fn shared_retro_error_uses_family_neutral_public_text() {
        assert_eq!(
            CurvedRetroArrowGestureErrorV1::ControlTooNearChord.to_string(),
            "curved terminal-arrow control point is too close to its chord"
        );
        assert_eq!(
            CurvedRetroArrowGestureErrorV1::ControlTooNearChord.category(),
            CurvedRetroArrowGestureCategoryV1::ControlTooNearChord
        );
        assert_eq!(
            CurvedRetroArrowGestureErrorV1::ControlTooNearChord.recovery(),
            CurvedRetroArrowGestureRecoveryV1::ChangeGeometry
        );
    }

    fn assert_foreign_session_rejects_terminal_arrow_family(
        begin: fn(
            &DocumentSession,
            DocumentFenceV1,
            PresentationGesturePoint2V1,
            PresentationGesturePoint2V1,
        )
            -> Result<CurvedElectronArrowGestureV1, CurvedElectronArrowGestureErrorV1>,
        preview: fn(
            &DocumentSession,
            &CurvedElectronArrowGestureV1,
            PresentationGesturePoint2V1,
        )
            -> Result<CurvedElectronArrowPreviewV1, CurvedElectronArrowGestureErrorV1>,
        prepare: fn(
            &mut DocumentSession,
            &CurvedElectronArrowGestureV1,
            &CurvedElectronArrowPreviewV1,
        )
            -> Result<PreparedCurvedElectronArrowV1, CurvedElectronArrowGestureErrorV1>,
        commit: fn(
            &mut DocumentSession,
            &mut PreparedCurvedElectronArrowV1,
        )
            -> Result<CommittedCurvedElectronArrowV1, CurvedElectronArrowGestureErrorV1>,
    ) {
        let mut owner = DocumentSession::load(EMPTY).unwrap();
        let mut foreign = DocumentSession::load(EMPTY).unwrap();
        let gesture = begin(&owner, fence(&owner), point(0.0, 0.0), point(20.0, 20.0)).unwrap();
        assert!(matches!(
            preview(&foreign, &gesture, point(40.0, 0.0)),
            Err(CurvedElectronArrowGestureErrorV1::ForeignSession)
        ));
        let issued = preview(&owner, &gesture, point(40.0, 0.0)).unwrap();
        assert!(matches!(
            prepare(&mut foreign, &gesture, &issued),
            Err(CurvedElectronArrowGestureErrorV1::ForeignSession)
        ));
        let mut prepared = prepare(&mut owner, &gesture, &issued).unwrap();
        assert!(matches!(
            commit(&mut foreign, &mut prepared),
            Err(CurvedElectronArrowGestureErrorV1::ForeignSession)
        ));
        assert_eq!(foreign.snapshot().unwrap().revision(), 0);
        assert_eq!(owner.snapshot().unwrap().revision(), 0);
        assert_eq!(
            commit(&mut owner, &mut prepared)
                .unwrap()
                .result()
                .observation()
                .snapshot()
                .revision(),
            1
        );
        assert!(matches!(
            commit(&mut owner, &mut prepared),
            Err(CurvedElectronArrowGestureErrorV1::ReplayedGesture)
        ));
    }

    #[test]
    fn terminal_arrow_families_fence_identical_foreign_sessions() {
        assert_foreign_session_rejects_terminal_arrow_family(
            begin_curved_electron_arrow_gesture_v1,
            preview_curved_electron_arrow_gesture_v1,
            prepare_curved_electron_arrow_gesture_v1,
            commit_curved_electron_arrow_gesture_v1,
        );
        assert_foreign_session_rejects_terminal_arrow_family(
            begin_curved_retro_arrow_gesture_v1,
            preview_curved_retro_arrow_gesture_v1,
            prepare_curved_retro_arrow_gesture_v1,
            commit_curved_retro_arrow_gesture_v1,
        );
        assert_foreign_session_rejects_terminal_arrow_family(
            begin_curved_normal_reaction_arrow_gesture_v1,
            preview_curved_normal_reaction_arrow_gesture_v1,
            prepare_curved_normal_reaction_arrow_gesture_v1,
            commit_curved_normal_reaction_arrow_gesture_v1,
        );
    }

    #[test]
    fn curved_normal_reaction_arrow_persists_its_closed_type() {
        let mut session = DocumentSession::load(EMPTY).unwrap();
        let gesture = begin_curved_normal_reaction_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 0.0),
            point(20.0, 20.0),
        )
        .unwrap();
        let preview =
            preview_curved_normal_reaction_arrow_gesture_v1(&session, &gesture, point(40.0, 0.0))
                .unwrap();
        let mut prepared =
            prepare_curved_normal_reaction_arrow_gesture_v1(&mut session, &gesture, &preview)
                .unwrap();
        let committed =
            commit_curved_normal_reaction_arrow_gesture_v1(&mut session, &mut prepared).unwrap();
        assert!(
            committed
                .result()
                .observation()
                .snapshot()
                .cdml()
                .contains("type=\"curved-normal\"")
        );
        assert!(matches!(
            commit_curved_normal_reaction_arrow_gesture_v1(&mut session, &mut prepared),
            Err(CurvedNormalReactionArrowGestureErrorV1::ReplayedGesture)
        ));
    }

    #[test]
    fn quadratic_arrow_preflights_and_commits_one_semantic_root() {
        let mut session = DocumentSession::load(EMPTY).unwrap();
        let gesture = begin_curved_electron_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 0.0),
            point(30.0, 20.0),
        )
        .unwrap();
        let preview =
            preview_curved_electron_arrow_gesture_v1(&session, &gesture, point(60.0, 0.0)).unwrap();
        let mut prepared =
            prepare_curved_electron_arrow_gesture_v1(&mut session, &gesture, &preview).unwrap();
        let committed =
            commit_curved_electron_arrow_gesture_v1(&mut session, &mut prepared).unwrap();
        assert_eq!(committed.result().observation().snapshot().revision(), 1);
        assert!(
            committed
                .result()
                .observation()
                .snapshot()
                .cdml()
                .contains("type=\"electron\"")
        );
    }

    #[test]
    fn committed_curved_electron_arrow_uses_plan_backed_point_and_marquee_selection() {
        let mut session = DocumentSession::load(EMPTY).unwrap();
        let gesture = begin_curved_electron_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 0.0),
            point(30.0, 20.0),
        )
        .unwrap();
        let preview =
            preview_curved_electron_arrow_gesture_v1(&session, &gesture, point(60.0, 0.0)).unwrap();
        let mut prepared =
            prepare_curved_electron_arrow_gesture_v1(&mut session, &gesture, &preview).unwrap();
        let _committed =
            commit_curved_electron_arrow_gesture_v1(&mut session, &mut prepared).unwrap();
        let snapshot = session.snapshot().expect("current snapshot");
        let plan = ferrum_render::render_presentation_stack_v1(
            session
                .observe(snapshot.revision())
                .expect("current observation")
                .projection()
                .presentation_stack(),
        )
        .expect("curved electron semantic projection renders");
        let interaction = RenderInteractionSessionV1::new(session);
        let fence = DocumentFenceV1::new(snapshot.revision(), *snapshot.digest());
        let observation = interaction
            .observe_render_interaction_with_presentation_plan_v1(fence, &plan)
            .expect("renderer-issued plan admits interaction");
        let [arrow] = observation.roots() else {
            panic!("expected one interaction arrow root");
        };
        assert_eq!(arrow.kind(), TopLevelRootKindV1::Arrow);
        let bounds = arrow.bounds();
        let point_selection = interaction
            .select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Point {
                    x: (bounds.left() + bounds.right()) / 2.0,
                    y: (bounds.top() + bounds.bottom()) / 2.0,
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("renderer plan bounds select the arrow");
        assert_eq!(point_selection.roots().len(), 1);
        let marquee_selection = interaction
            .select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Marquee {
                    left: bounds.left(),
                    top: bounds.top(),
                    right: bounds.right(),
                    bottom: bounds.bottom(),
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("renderer plan bounds marquee-select the arrow");
        assert_eq!(marquee_selection.roots().len(), 1);
    }

    #[test]
    fn invalid_geometry_and_replayed_receipts_leave_the_document_unchanged() {
        let mut session = DocumentSession::load(EMPTY).unwrap();
        let gesture = begin_curved_electron_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 0.0),
            point(20.0, 0.0),
        )
        .unwrap();
        assert!(matches!(
            preview_curved_electron_arrow_gesture_v1(&session, &gesture, point(40.0, 0.0)),
            Err(CurvedElectronArrowGestureErrorV1::ControlTooNearChord)
        ));
        assert_eq!(session.snapshot().unwrap().revision(), 0);
        let gesture = begin_curved_electron_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 0.0),
            point(20.0, 10.0),
        )
        .unwrap();
        let preview =
            preview_curved_electron_arrow_gesture_v1(&session, &gesture, point(40.0, 0.0)).unwrap();
        let mut prepared =
            prepare_curved_electron_arrow_gesture_v1(&mut session, &gesture, &preview).unwrap();
        commit_curved_electron_arrow_gesture_v1(&mut session, &mut prepared).unwrap();
        let revision = session.snapshot().unwrap().revision();
        assert!(matches!(
            commit_curved_electron_arrow_gesture_v1(&mut session, &mut prepared),
            Err(CurvedElectronArrowGestureErrorV1::ReplayedGesture)
        ));
        assert_eq!(session.snapshot().unwrap().revision(), revision);
    }
}
