use super::DocumentSession;
use crate::{
    AuthoringGesturePairAccessErrorV1, CreatePresentationRootV1, DocumentFenceV1,
    PresentationCreationGestureV1, PresentationCreationPreviewV1, PresentationGestureErrorV1,
    PresentationGestureKindV1, PresentationGesturePoint2V1, PresentationGestureSnapPolicyV1,
    PresentationGestureStyleV1, SessionOperation, SessionOperationTransitionRequestV1,
    SessionOperationV1, TransitionAuthorizationV1,
};

pub(super) fn begin(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    kind: PresentationGestureKindV1,
    start: PresentationGesturePoint2V1,
    style: PresentationGestureStyleV1,
    snap: PresentationGestureSnapPolicyV1,
) -> Result<PresentationCreationGestureV1, PresentationGestureErrorV1> {
    require_fence(session, fence)?;
    if !matches!(
        (kind, style),
        (
            PresentationGestureKindV1::StraightNormalArrow,
            PresentationGestureStyleV1::Normal(_)
        ) | (
            PresentationGestureKindV1::StraightEquilibriumArrow,
            PresentationGestureStyleV1::Equilibrium
        ) | (
            PresentationGestureKindV1::Plus,
            PresentationGestureStyleV1::Plus
        )
    ) {
        return Err(PresentationGestureErrorV1::InvalidGestureStyle);
    }
    Ok(PresentationCreationGestureV1 {
        capability: session.authoring_capability_issuer.issue(),
        fence,
        kind,
        start,
        style,
        snap,
    })
}

pub(super) fn preview(
    session: &DocumentSession,
    gesture: &PresentationCreationGestureV1,
    raw_end: PresentationGesturePoint2V1,
) -> Result<PresentationCreationPreviewV1, PresentationGestureErrorV1> {
    validate_gesture_owner_and_fence(session, gesture)?;
    crate::presentation_creation_gesture_v1::preview(gesture.clone(), raw_end)
}

pub(super) fn resolve(
    session: &DocumentSession,
    gesture: &PresentationCreationGestureV1,
    preview: &PresentationCreationPreviewV1,
) -> Result<SessionOperationTransitionRequestV1, PresentationGestureErrorV1> {
    session
        .authoring_capability_issuer
        .validate_gesture_pair_for_prepare_v1(
            &gesture.capability,
            &preview.gesture.capability,
            preview.matches_gesture(gesture),
        )
        .map_err(map_pair_error)?;
    require_fence(session, gesture.fence)?;
    let operation = match (gesture.kind, gesture.style) {
        (
            PresentationGestureKindV1::StraightNormalArrow,
            PresentationGestureStyleV1::Normal(style),
        ) => CreatePresentationRootV1::straight_normal_arrow(gesture.start, preview.end, style),
        (
            PresentationGestureKindV1::StraightEquilibriumArrow,
            PresentationGestureStyleV1::Equilibrium,
        ) => CreatePresentationRootV1::straight_equilibrium_arrow(gesture.start, preview.end),
        (PresentationGestureKindV1::Plus, PresentationGestureStyleV1::Plus) => {
            CreatePresentationRootV1::standard_plus(gesture.start)
        }
        _ => return Err(PresentationGestureErrorV1::InvalidGestureStyle),
    };
    Ok(SessionOperationTransitionRequestV1::new(
        gesture.fence.revision(),
        SessionOperation::V1(SessionOperationV1::CreatePresentationRootV1(operation)),
        TransitionAuthorizationV1::authoring_capability(gesture.capability.clone()),
    ))
}

fn validate_gesture_owner_and_fence(
    session: &DocumentSession,
    gesture: &PresentationCreationGestureV1,
) -> Result<(), PresentationGestureErrorV1> {
    if !gesture
        .capability
        .belongs_to(&session.authoring_capability_issuer)
    {
        return Err(PresentationGestureErrorV1::ForeignSession);
    }
    require_fence(session, gesture.fence)
}

fn map_pair_error(error: AuthoringGesturePairAccessErrorV1) -> PresentationGestureErrorV1 {
    match error {
        AuthoringGesturePairAccessErrorV1::ForeignSession => {
            PresentationGestureErrorV1::ForeignSession
        }
        AuthoringGesturePairAccessErrorV1::PreviewMismatch => {
            PresentationGestureErrorV1::PreviewMismatch
        }
        AuthoringGesturePairAccessErrorV1::Replayed => PresentationGestureErrorV1::ReplayedGesture,
    }
}

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), PresentationGestureErrorV1> {
    if session.current_revision_v1() != fence.revision() {
        Err(PresentationGestureErrorV1::StaleRevision)
    } else if session.current_digest_v1() != fence.digest() {
        Err(PresentationGestureErrorV1::StaleDigest)
    } else {
        Ok(())
    }
}
