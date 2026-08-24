use super::{
    AdmittedSessionTransitionRefusalV1, DocumentSession, PersistentId, PreparedSessionTransitionV1,
    RevisionState,
};
use crate::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityIssuerV1, DocumentFenceV1,
    TextPlacementContentV1, TextPlacementErrorV1, TextPlacementGestureV1,
    derive_document_render_observation_from_accepted_operation_v1,
};
use ferrum_render::DocumentTextRenderV1;

/// Opaque renderer-admitted candidate for one authored Text root.
pub struct PendingTextPlacementV1 {
    issuer: AuthoringCapabilityIssuerV1,
    gesture: TextPlacementGestureV1,
    transition: PreparedSessionTransitionV1,
    identifier: PersistentId,
    overlay: DocumentTextRenderV1,
}

impl PendingTextPlacementV1 {
    #[must_use]
    pub fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
    #[must_use]
    pub fn overlay(&self) -> &DocumentTextRenderV1 {
        &self.overlay
    }
    #[must_use]
    pub fn matches(&self, gesture: &TextPlacementGestureV1) -> bool {
        self.gesture == *gesture
    }
}

pub(super) fn prepare(
    session: &mut DocumentSession,
    gesture: &TextPlacementGestureV1,
    content: TextPlacementContentV1,
) -> Result<PendingTextPlacementV1, TextPlacementErrorV1> {
    if !gesture
        .capability
        .belongs_to(&session.authoring_capability_issuer)
    {
        return Err(TextPlacementErrorV1::ForeignSession);
    }
    require_fence(session, gesture.fence)?;
    let (identifier, effects) = session
        .reserve_generated_ids_for_transition_v1(|sequences, indexed| {
            sequences.reserve_presentation(indexed)
        })
        .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
    let document = session
        .current_document_v1()
        .with_insert_authored_text_v1(
            &identifier,
            gesture.anchor,
            content.runs(),
            content.font_size(),
            content.color(),
        )
        .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
    let revision = session
        .next_revision_v1()
        .ok_or(TextPlacementErrorV1::SessionConflict)?;
    let candidate = RevisionState::from_document(revision, document)
        .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
    let snapshot = candidate.snapshot(!session.saved_baseline.is_current(&candidate));
    let observation = super::SessionDocumentObservationV1::from_snapshot(snapshot)
        .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
    let render = derive_document_render_observation_from_accepted_operation_v1(&observation)
        .map_err(|_| TextPlacementErrorV1::RenderPreparation)?;
    let overlay = render
        .resolved()
        .text_renders()
        .iter()
        .find(|value| value.target().source_id() == Some(identifier.as_str()))
        .cloned()
        .ok_or(TextPlacementErrorV1::UnrenderableStandard)?;
    let transition = session
        .prepare_changed_session_transition_v1(
            gesture.fence.revision(),
            gesture.fence.digest(),
            candidate,
            effects,
        )
        .map_err(map_prepare_error)?;
    Ok(PendingTextPlacementV1 {
        issuer: session.authoring_capability_issuer.clone(),
        gesture: gesture.clone(),
        transition,
        identifier,
        overlay,
    })
}

pub(super) fn commit(
    session: &mut DocumentSession,
    pending: &mut PendingTextPlacementV1,
) -> Result<crate::CommittedTextPlacementV1, TextPlacementErrorV1> {
    if pending.transition.is_consumed_v1() {
        return Err(TextPlacementErrorV1::ReplayedGesture);
    }
    if !pending
        .issuer
        .same_issuer(&session.authoring_capability_issuer)
        || !pending
            .gesture
            .capability
            .belongs_to(&session.authoring_capability_issuer)
    {
        return Err(TextPlacementErrorV1::ForeignSession);
    }
    let claim = pending
        .gesture
        .capability
        .claim_for_commit(&session.authoring_capability_issuer)
        .map_err(|error| match error {
            AuthoringCapabilityAccessErrorV1::ForeignSession => {
                TextPlacementErrorV1::ForeignSession
            }
            AuthoringCapabilityAccessErrorV1::Replayed => TextPlacementErrorV1::ReplayedGesture,
        })?;
    let result = session
        .commit_session_operation_transition_v1(&mut pending.transition)
        .map_err(map_commit_error)?;
    claim.consume();
    Ok(crate::CommittedTextPlacementV1::new(
        pending.identifier.clone(),
        result,
    ))
}

fn map_prepare_error(error: super::DocumentSessionError) -> TextPlacementErrorV1 {
    match error {
        super::DocumentSessionError::RendererAdmission => TextPlacementErrorV1::RenderPreparation,
        _ => TextPlacementErrorV1::SessionConflict,
    }
}

fn map_commit_error(error: AdmittedSessionTransitionRefusalV1) -> TextPlacementErrorV1 {
    match error {
        AdmittedSessionTransitionRefusalV1::ForeignSession => TextPlacementErrorV1::ForeignSession,
        AdmittedSessionTransitionRefusalV1::Replayed => TextPlacementErrorV1::ReplayedGesture,
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => TextPlacementErrorV1::StaleSnapshot,
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            TextPlacementErrorV1::RenderPreparation
        }
        AdmittedSessionTransitionRefusalV1::ProvisionalCapability
        | AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
            TextPlacementErrorV1::SessionConflict
        }
    }
}

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), TextPlacementErrorV1> {
    if session.current_revision_v1() != fence.revision()
        || session.current_digest_v1() != fence.digest()
    {
        Err(TextPlacementErrorV1::StaleSnapshot)
    } else {
        Ok(())
    }
}
