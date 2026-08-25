//! Renderer-interaction lifecycle for translating admitted top-level roots.
//!
//! The interaction boundary validates the opaque gesture and preview. The
//! document session remains the sole owner of the prepared transition, renderer
//! admission, revision fence, and committed history.

use ferrum_document::renderer_admission::RendererTranslationSnapRefusalV1;
use ferrum_document::{
    AdmittedSessionTransitionRefusalV1, DocumentSessionError, SessionOperation,
    SessionOperationTransitionRequestV1, SessionOperationV1, TopLevelRootSelectorV1,
    TopLevelRootTranslationV1, TransitionAuthorizationV1,
};
use ferrum_geometry::{HexGrid, Point2};

use super::{
    CommittedRenderInteractionTranslationV1, RenderInteractionAxisV1, RenderInteractionErrorV1,
    RenderInteractionGridSnapPolicyV1, RenderInteractionSelectionV1, RenderInteractionSessionV1,
    RenderInteractionSnapV1, RenderInteractionTranslationGestureV1,
    RenderInteractionTranslationPreviewV1, VIEW_HEX_GRID_SPACING_PT_V1,
};

pub(crate) fn begin_root_translation_interaction_v1(
    session: &RenderInteractionSessionV1,
    selection: &RenderInteractionSelectionV1,
    press_x: f64,
    press_y: f64,
    snap: RenderInteractionSnapV1,
) -> Result<RenderInteractionTranslationGestureV1, RenderInteractionErrorV1> {
    session.require_selection(selection)?;
    if selection.is_empty() {
        return Err(RenderInteractionErrorV1::EmptySelection);
    }
    if !press_x.is_finite() || !press_y.is_finite() {
        return Err(RenderInteractionErrorV1::NonFinitePoint);
    }
    Ok(RenderInteractionTranslationGestureV1 {
        origin: session.origin,
        authoring_capability: session.issue_authoring_capability_v1(),
        selection: selection.clone(),
        press_x,
        press_y,
        snap,
    })
}

pub(crate) fn preview_root_translation_interaction_v1(
    session: &RenderInteractionSessionV1,
    gesture: &RenderInteractionTranslationGestureV1,
    pointer_x: f64,
    pointer_y: f64,
) -> Result<RenderInteractionTranslationPreviewV1, RenderInteractionErrorV1> {
    validate_root_translation_gesture_v1(session, gesture)?;
    if !pointer_x.is_finite() || !pointer_y.is_finite() {
        return Err(RenderInteractionErrorV1::NonFinitePoint);
    }
    let raw_dx = pointer_x - gesture.press_x;
    let raw_dy = pointer_y - gesture.press_y;
    let (mut dx, mut dy) = match gesture.snap.grid_policy {
        RenderInteractionGridSnapPolicyV1::Free => (raw_dx, raw_dy),
        RenderInteractionGridSnapPolicyV1::ViewHexGrid => {
            snap_translation_to_view_hex_grid_v1(session, gesture, raw_dx, raw_dy)?
        }
    };
    match gesture.snap.axis {
        RenderInteractionAxisV1::Free => {}
        RenderInteractionAxisV1::Horizontal => dy = 0.0,
        RenderInteractionAxisV1::Vertical => dx = 0.0,
    }
    Ok(RenderInteractionTranslationPreviewV1 {
        dx,
        dy,
        bounds: gesture
            .selection
            .roots
            .iter()
            .map(|root| root.bounds.translated(dx, dy))
            .collect(),
    })
}

pub(crate) fn commit_root_translation_interaction_v1(
    session: &mut RenderInteractionSessionV1,
    gesture: RenderInteractionTranslationGestureV1,
    release_x: f64,
    release_y: f64,
) -> Result<CommittedRenderInteractionTranslationV1, RenderInteractionErrorV1> {
    let preview = preview_root_translation_interaction_v1(session, &gesture, release_x, release_y)?;
    let targets = root_translation_targets_v1(&gesture)?;
    let translation = TopLevelRootTranslationV1::new(targets, preview.dx, preview.dy)
        .map_err(|_| RenderInteractionErrorV1::SelectionChanged)?;
    let mut prepared = session
        .session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            gesture.selection.fence.revision(),
            SessionOperation::V1(SessionOperationV1::TranslateTopLevelRootsV1(translation)),
            TransitionAuthorizationV1::AuthoringCapability(gesture.authoring_capability),
        ))
        .map_err(map_translation_prepare_error)?;
    let result = session
        .session
        .commit_session_operation_transition_v1(&mut prepared)
        .map_err(map_translation_commit_error)?;
    Ok(CommittedRenderInteractionTranslationV1 {
        changed: preview.dx != 0.0 || preview.dy != 0.0,
        result,
        selection: gesture.selection.clone(),
    })
}

fn validate_root_translation_gesture_v1(
    session: &RenderInteractionSessionV1,
    gesture: &RenderInteractionTranslationGestureV1,
) -> Result<(), RenderInteractionErrorV1> {
    session.require_gesture(gesture)?;
    session.require_selection(&gesture.selection)
}

fn root_translation_targets_v1(
    gesture: &RenderInteractionTranslationGestureV1,
) -> Result<Vec<TopLevelRootSelectorV1>, RenderInteractionErrorV1> {
    let targets = gesture
        .selection
        .roots
        .iter()
        .map(|root| TopLevelRootSelectorV1::new(root.document_object_id().clone(), root.kind()))
        .collect();
    Ok(targets)
}

fn snap_translation_to_view_hex_grid_v1(
    session: &RenderInteractionSessionV1,
    gesture: &RenderInteractionTranslationGestureV1,
    raw_dx: f64,
    raw_dy: f64,
) -> Result<(f64, f64), RenderInteractionErrorV1> {
    if raw_dx == 0.0 && raw_dy == 0.0 {
        return Ok((0.0, 0.0));
    }
    let origin = Point2::new(0.0, 0.0).map_err(|_| RenderInteractionErrorV1::Observation)?;
    let grid = HexGrid::new(VIEW_HEX_GRID_SPACING_PT_V1, origin)
        .map_err(|_| RenderInteractionErrorV1::Observation)?;
    let delta = session
        .session
        .snap_top_level_translation_for_renderer_v1(
            gesture.selection.fence.revision(),
            root_translation_targets_v1(gesture)?,
            raw_dx,
            raw_dy,
            grid,
        )
        .map_err(map_translation_snap_error)?;
    Ok((delta.dx(), delta.dy()))
}

fn map_translation_snap_error(error: RendererTranslationSnapRefusalV1) -> RenderInteractionErrorV1 {
    match error {
        RendererTranslationSnapRefusalV1::NonFiniteDelta => {
            RenderInteractionErrorV1::NonFinitePoint
        }
        RendererTranslationSnapRefusalV1::Grid => RenderInteractionErrorV1::Observation,
        RendererTranslationSnapRefusalV1::StaleRevision
        | RendererTranslationSnapRefusalV1::Selection => RenderInteractionErrorV1::SelectionChanged,
    }
}

fn map_translation_prepare_error(error: DocumentSessionError) -> RenderInteractionErrorV1 {
    match error {
        DocumentSessionError::RendererAdmission => RenderInteractionErrorV1::RendererAdmission,
        _ => RenderInteractionErrorV1::SessionConflict,
    }
}

fn map_translation_commit_error(
    error: AdmittedSessionTransitionRefusalV1,
) -> RenderInteractionErrorV1 {
    match error {
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            RenderInteractionErrorV1::RendererAdmission
        }
        _ => RenderInteractionErrorV1::SessionConflict,
    }
}
