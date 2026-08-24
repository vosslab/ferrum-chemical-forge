use super::{
    AdmittedSessionTransitionRefusalV1, DocumentSession, PersistentId, PreparedSessionTransitionV1,
    RevisionState, SessionDocumentObservationV1,
};
use crate::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityIssuerV1, CommittedPresentationGestureV1,
    DocumentFenceV1, PresentationCreationGestureV1, PresentationGestureErrorV1,
    PresentationGestureKindV1, PresentationGesturePoint2V1, PresentationGestureSnapPolicyV1,
    PresentationGestureStyleV1, PresentationRecordKindV1,
    derive_document_render_observation_from_accepted_operation_v1,
};
use ferrum_render::{PresentationRenderPlanV1, render_presentation_stack_v1};

/// Opaque renderer-admitted candidate for a straight arrow, equilibrium arrow, or Plus root.
pub struct PendingPresentationGestureV1 {
    issuer: AuthoringCapabilityIssuerV1,
    gesture: PresentationCreationGestureV1,
    transition: PreparedSessionTransitionV1,
    identifier: PersistentId,
    root_kind: PresentationRecordKindV1,
    plan: PresentationRenderPlanV1,
}

impl PendingPresentationGestureV1 {
    #[must_use]
    pub fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
    #[must_use]
    pub const fn root_kind(&self) -> PresentationRecordKindV1 {
        self.root_kind
    }
    #[must_use]
    pub fn plan(&self) -> &PresentationRenderPlanV1 {
        &self.plan
    }
    #[must_use]
    pub fn matches(&self, gesture: &PresentationCreationGestureV1) -> bool {
        self.gesture == *gesture
    }
}

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

pub(super) fn prepare(
    session: &mut DocumentSession,
    gesture: &PresentationCreationGestureV1,
    raw_end: PresentationGesturePoint2V1,
) -> Result<PendingPresentationGestureV1, PresentationGestureErrorV1> {
    if !gesture
        .capability
        .belongs_to(&session.authoring_capability_issuer)
    {
        return Err(PresentationGestureErrorV1::ForeignSession);
    }
    require_fence(session, gesture.fence)?;
    let semantic = crate::presentation_creation_gesture_v1::preview(gesture.clone(), raw_end)?;
    let (identifier, effects) = session
        .reserve_generated_ids_for_transition_v1(|sequences, indexed| {
            sequences.reserve_presentation(indexed)
        })
        .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
    let document = match (gesture.kind, gesture.style) {
        (
            PresentationGestureKindV1::StraightNormalArrow,
            PresentationGestureStyleV1::Normal(style),
        ) => session
            .current_document_v1()
            .with_insert_straight_normal_arrow(
                &identifier,
                gesture.start,
                semantic.end,
                style.start_head(),
                style.end_head(),
            ),
        (
            PresentationGestureKindV1::StraightEquilibriumArrow,
            PresentationGestureStyleV1::Equilibrium,
        ) => session
            .current_document_v1()
            .with_insert_straight_equilibrium_arrow(&identifier, gesture.start, semantic.end),
        (PresentationGestureKindV1::Plus, PresentationGestureStyleV1::Plus) => session
            .current_document_v1()
            .with_insert_standard_plus(&identifier, gesture.start),
        _ => return Err(PresentationGestureErrorV1::InvalidGestureStyle),
    }
    .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
    let revision = session
        .next_revision_v1()
        .ok_or(PresentationGestureErrorV1::SessionConflict)?;
    let candidate = RevisionState::from_document(revision, document)
        .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
    let snapshot = candidate.snapshot(!session.saved_baseline.is_current(&candidate));
    let observation = SessionDocumentObservationV1::from_snapshot(snapshot)
        .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
    let render = derive_document_render_observation_from_accepted_operation_v1(&observation)
        .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
    let plan = render_presentation_stack_v1(render.resolved().projection().presentation_stack())
        .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
    if !plan
        .roots()
        .iter()
        .any(|root| root.target().source_id() == Some(identifier.as_str()))
    {
        return Err(PresentationGestureErrorV1::SessionConflict);
    }
    let transition = session
        .prepare_changed_session_transition_v1(
            gesture.fence.revision(),
            gesture.fence.digest(),
            candidate,
            effects,
        )
        .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
    let root_kind = if gesture.kind == PresentationGestureKindV1::Plus {
        PresentationRecordKindV1::Plus
    } else {
        PresentationRecordKindV1::Arrow
    };
    Ok(PendingPresentationGestureV1 {
        issuer: session.authoring_capability_issuer.clone(),
        gesture: gesture.clone(),
        transition,
        identifier,
        root_kind,
        plan,
    })
}

pub(super) fn commit(
    session: &mut DocumentSession,
    pending: &mut PendingPresentationGestureV1,
) -> Result<CommittedPresentationGestureV1, PresentationGestureErrorV1> {
    if pending.transition.is_consumed_v1() {
        return Err(PresentationGestureErrorV1::ReplayedGesture);
    }
    if !pending
        .issuer
        .same_issuer(&session.authoring_capability_issuer)
        || !pending
            .gesture
            .capability
            .belongs_to(&session.authoring_capability_issuer)
    {
        return Err(PresentationGestureErrorV1::ForeignSession);
    }
    let claim = pending
        .gesture
        .capability
        .claim_for_commit(&session.authoring_capability_issuer)
        .map_err(|error| match error {
            AuthoringCapabilityAccessErrorV1::ForeignSession => {
                PresentationGestureErrorV1::ForeignSession
            }
            AuthoringCapabilityAccessErrorV1::Replayed => {
                PresentationGestureErrorV1::ReplayedGesture
            }
        })?;
    let result = session
        .commit_session_operation_transition_v1(&mut pending.transition)
        .map_err(map_transition_error)?;
    claim.consume();
    Ok(CommittedPresentationGestureV1::new(
        pending.gesture.kind,
        pending.identifier.clone(),
        result,
    ))
}

fn map_transition_error(error: AdmittedSessionTransitionRefusalV1) -> PresentationGestureErrorV1 {
    match error {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            PresentationGestureErrorV1::ForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Replayed => PresentationGestureErrorV1::ReplayedGesture,
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            PresentationGestureErrorV1::StaleRevision
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission
        | AdmittedSessionTransitionRefusalV1::ProvisionalCapability
        | AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
            PresentationGestureErrorV1::SessionConflict
        }
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
