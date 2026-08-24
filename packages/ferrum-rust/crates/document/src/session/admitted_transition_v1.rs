//! Generic document-owned renderer-admitted state transitions.
//!
//! A changed visible state is prepared as one opaque value containing its exact
//! prospective state, observation, renderer proof, deferred session effects, and
//! result. Only this module may redeem that value into session history.

use super::{
    AuthoringCapabilityIssuerV1, DocumentSession, DocumentSessionError, RendererAdmittedPendingV1,
    RevisionState, SessionDocumentObservationV1, SessionOperation, SessionOperationError,
    SessionOperationResultV1,
};
use crate::{IndexedDocument, session_operation::Candidate};

use super::ProvisionalToken;

mod history;

pub(super) use history::AdmittedHistoryV1;

/// Construct the sole mutable timeline retained by a document session.
pub(super) fn initial_admitted_history_v1(initial: RevisionState) -> AdmittedHistoryV1 {
    AdmittedHistoryV1::new(initial, 20)
}

/// Closed refusal set for a document-owned admitted transition.
///
/// The pending value remains owned by its source session after every refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AdmittedSessionTransitionRefusalV1 {
    /// The opaque transition was prepared by another live session.
    ForeignSession,
    /// The opaque transition was already redeemed or retired.
    Replayed,
    /// The source revision or digest no longer identifies the current state.
    StaleSnapshot,
    /// The renderer no longer accepts the exact prospective observation.
    RendererAdmission,
    /// A deferred session capability can no longer be redeemed.
    ProvisionalCapability,
    /// The session could not reserve bounded history storage for this transition.
    HistoryCapacity,
}

/// Opaque prepared session transition.
///
/// It deliberately exposes neither candidate CDML, mutable state, renderer proof,
/// pending identity, nor deferred session effects. Callers can only redeem it through
/// [`DocumentSession::commit_session_operation_transition_v1`] or retire it through
/// [`DocumentSession::retire_session_operation_transition_v1`].
#[derive(Debug)]
pub struct PreparedSessionTransitionV1 {
    issuer: AuthoringCapabilityIssuerV1,
    source_revision: u64,
    source_digest: [u8; 32],
    kind: PreparedSessionTransitionKindV1,
}

/// Read-only metadata for an unredeemed admitted transition.
///
/// Route wrappers may use this to present the exact admitted observation and
/// renderer plan. It deliberately omits the prospective document, renderer
/// proof, pending identity, and deferred effects.
#[derive(Debug)]
pub(crate) struct PreparedSessionTransitionMetadataV1<'a> {
    observation: &'a SessionDocumentObservationV1,
    renderer_plan: Option<&'a ferrum_render::DocumentRenderPlanV1>,
}

impl PreparedSessionTransitionMetadataV1<'_> {
    /// Return the exact immutable observation admitted for this transition.
    #[must_use]
    pub(crate) const fn observation(&self) -> &SessionDocumentObservationV1 {
        self.observation
    }

    /// Return the immutable renderer plan for a changed transition.
    #[must_use]
    pub(crate) const fn renderer_plan(&self) -> Option<&ferrum_render::DocumentRenderPlanV1> {
        self.renderer_plan
    }
}

#[derive(Debug)]
enum PreparedSessionTransitionKindV1 {
    NoChange {
        result: Option<SessionOperationResultV1>,
    },
    Changed(PreparedChangedSessionTransitionV1),
}

#[derive(Debug)]
struct PreparedChangedSessionTransitionV1 {
    state: Option<RevisionState>,
    observation: SessionDocumentObservationV1,
    renderer_admission: RendererAdmittedPendingV1,
    effects: SessionTransitionEffectsV1,
    commit: ChangedTransitionCommitV1,
    result: Option<SessionOperationResultV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ChangedTransitionCommitV1 {
    Append,
    Navigate(HistoryNavigationDirectionV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HistoryNavigationDirectionV1 {
    Undo,
    Redo,
}

/// Deferred document-owned effects redeemed with one changed transition.
///
/// Routes add effects here instead of mutating the live document during preparation.
#[derive(Debug, Default)]
pub(super) struct SessionTransitionEffectsV1 {
    provisional_token: Option<ProvisionalToken>,
    next_generated_ids: Option<super::GeneratedIdSequences>,
}

/// Refusal from combining two document-owned deferred effect sets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::session) enum SessionTransitionEffectCompositionRefusalV1 {
    /// Both effect sets attempted to consume a provisional token.
    DuplicateProvisionalToken,
    /// Both effect sets attempted to install generated-ID sequences.
    DuplicateGeneratedIds,
}

impl SessionTransitionEffectsV1 {
    pub(super) fn none() -> Self {
        Self::default()
    }

    /// Add a capability consumption to this deferred transition.
    #[must_use]
    pub(super) fn consuming_provisional_token(mut self, token: ProvisionalToken) -> Self {
        self.provisional_token = Some(token);
        self
    }

    /// Add installation of one tentative generated-ID sequence set.
    #[must_use]
    pub(super) fn installing_generated_ids(
        mut self,
        next_generated_ids: super::GeneratedIdSequences,
    ) -> Self {
        self.next_generated_ids = Some(next_generated_ids);
        self
    }

    fn compose(
        self,
        extension: Self,
    ) -> Result<Self, SessionTransitionEffectCompositionRefusalV1> {
        let provisional_token = match (self.provisional_token, extension.provisional_token) {
            (Some(_), Some(_)) => {
                return Err(SessionTransitionEffectCompositionRefusalV1::DuplicateProvisionalToken);
            }
            (primary, extension) => primary.or(extension),
        };
        let next_generated_ids = match (self.next_generated_ids, extension.next_generated_ids) {
            (Some(_), Some(_)) => {
                return Err(SessionTransitionEffectCompositionRefusalV1::DuplicateGeneratedIds);
            }
            (primary, extension) => primary.or(extension),
        };
        Ok(Self {
            provisional_token,
            next_generated_ids,
        })
    }

    fn verify_provisional_token(
        &self,
        session: &DocumentSession,
    ) -> Result<(), DocumentSessionError> {
        if let Some(token) = &self.provisional_token {
            session
                .admitted_history
                .current()
                .document()
                .verify_provisional_token(token)
                .map_err(super::prepared::map_prepared_token_error)?;
        }
        Ok(())
    }

    fn consume_provisional_token(&self, session: &mut DocumentSession) {
        if let Some(token) = &self.provisional_token {
            session
                .admitted_history
                .current_mut()
                .document_mut()
                .consume_provisional_token(token)
                .expect(
                    "the immediately preceding capability verification established this invariant",
                );
        }
    }

    fn install_generated_ids(&self, session: &mut DocumentSession) {
        if let Some(next_generated_ids) = self.next_generated_ids {
            session.generated_ids = next_generated_ids;
        }
    }
}

impl PreparedSessionTransitionV1 {
    /// Return whether this prepared transition was already redeemed or retired.
    #[must_use]
    pub fn is_consumed_v1(&self) -> bool {
        match &self.kind {
            PreparedSessionTransitionKindV1::NoChange { result } => result.is_none(),
            PreparedSessionTransitionKindV1::Changed(changed) => changed.state.is_none(),
        }
    }

    /// Return immutable route-facing metadata while this transition is redeemable.
    #[must_use]
    pub(crate) fn metadata_v1(&self) -> Option<PreparedSessionTransitionMetadataV1<'_>> {
        match &self.kind {
            PreparedSessionTransitionKindV1::NoChange {
                result: Some(result),
            } => Some(PreparedSessionTransitionMetadataV1 {
                observation: result.observation(),
                renderer_plan: None,
            }),
            PreparedSessionTransitionKindV1::Changed(PreparedChangedSessionTransitionV1 {
                state: Some(_),
                observation,
                renderer_admission,
                result: Some(result),
                ..
            }) => Some(PreparedSessionTransitionMetadataV1 {
                observation,
                renderer_plan: Some(renderer_admission.plan()),
            }),
            _ => None,
        }
    }
}

impl DocumentSession {
    /// Read the retained revision state without exposing timeline navigation.
    #[must_use]
    pub(crate) fn current_state_v1(&self) -> &RevisionState {
        self.admitted_history.current()
    }

    /// Read the current typed document without granting mutation authority.
    #[must_use]
    pub(crate) fn current_document_v1(&self) -> &super::TypedDocument {
        self.current_state_v1().document()
    }

    /// Read the current indexed document without exposing a mutable timeline.
    #[must_use]
    pub(crate) fn current_index_v1(&self) -> &IndexedDocument {
        self.current_document_v1().indexed()
    }

    /// Read the authoritative current revision.
    #[must_use]
    pub(crate) fn current_revision_v1(&self) -> u64 {
        self.current_state_v1().revision()
    }

    /// Read the authoritative digest for the current state.
    #[must_use]
    pub(crate) fn current_digest_v1(&self) -> [u8; 32] {
        *self.current_state_v1().digest()
    }

    /// Read the next monotonic revision if the sequence has capacity.
    #[must_use]
    pub(crate) fn next_revision_v1(&self) -> Option<u64> {
        self.current_state_v1().next_revision()
    }

    pub(super) fn has_undo_history_v1(&self) -> bool {
        self.admitted_history.undo_target().is_some()
    }

    pub(super) fn has_redo_history_v1(&self) -> bool {
        self.admitted_history.redo_target().is_some()
    }

    /// Issue one source-document provisional token as a deferred transition effect.
    ///
    /// The token remains document-owned and is consumed only when the matching
    /// renderer-admitted transition commits. Route modules cannot mutate the
    /// retained document to issue tokens directly.
    pub(super) fn issue_transition_provisional_token_effect_v1(
        &mut self,
    ) -> Result<SessionTransitionEffectsV1, DocumentSessionError> {
        let token = super::prepared::issue_prepared_token(
            self.admitted_history.current_mut().document_mut(),
        )?;
        Ok(SessionTransitionEffectsV1::none().consuming_provisional_token(token))
    }

    /// Combine complementary deferred effects for one admitted transition.
    ///
    /// The core owns effect-slot conflict detection so routes cannot silently
    /// overwrite token consumption or generated-ID installation behavior.
    pub(super) fn compose_transition_effects_v1(
        primary: SessionTransitionEffectsV1,
        extension: SessionTransitionEffectsV1,
    ) -> Result<SessionTransitionEffectsV1, SessionTransitionEffectCompositionRefusalV1> {
        primary.compose(extension)
    }

    #[cfg(test)]
    pub(super) fn set_current_revision_for_test_v1(&mut self, revision: u64) {
        self.admitted_history
            .set_current_revision_for_test(revision);
    }

    /// Retire one opaque transition without changing document state or effects.
    ///
    /// Retirement is issuer-bound and one-use. It invalidates the prospective
    /// state, renderer proof, and deferred effects without installing them.
    /// A foreign session cannot invalidate the owner's pending transition.
    pub fn retire_session_operation_transition_v1(
        &mut self,
        prepared: &mut PreparedSessionTransitionV1,
    ) -> Result<(), AdmittedSessionTransitionRefusalV1> {
        if !prepared
            .issuer
            .same_issuer(&self.authoring_capability_issuer_v1())
        {
            return Err(AdmittedSessionTransitionRefusalV1::ForeignSession);
        }
        if prepared.is_consumed_v1() {
            return Err(AdmittedSessionTransitionRefusalV1::Replayed);
        }
        match &mut prepared.kind {
            PreparedSessionTransitionKindV1::NoChange { result } => {
                let _ = result.take();
            }
            PreparedSessionTransitionKindV1::Changed(changed) => {
                let _ = changed.state.take();
                let _ = changed.result.take();
            }
        }
        Ok(())
    }

    /// Prepare one canonical history-free no-change result for a route.
    ///
    /// This intentionally has no renderer admission, deferred effect, or
    /// history reservation. It is the only route adapter for a semantic no-op.
    pub(super) fn prepare_no_change_session_transition_v1(
        &self,
        expected_revision: u64,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let current = self.admitted_history.current();
        Ok(PreparedSessionTransitionV1 {
            issuer: self.authoring_capability_issuer_v1(),
            source_revision: current.revision(),
            source_digest: *current.digest(),
            kind: PreparedSessionTransitionKindV1::NoChange {
                result: Some(self.operation_result()?),
            },
        })
    }

    /// Prepare one renderer-admitted typed session operation without changing history.
    pub fn prepare_session_operation_transition_v1(
        &mut self,
        expected_revision: u64,
        operation: SessionOperation,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let current = self.admitted_history.current();
        let source_revision = current.revision();
        let source_digest = *current.digest();
        let issuer = self.authoring_capability_issuer_v1();
        match operation.prepare(current.document(), source_revision, &source_digest)? {
            Candidate::NoChange => Ok(PreparedSessionTransitionV1 {
                issuer,
                source_revision,
                source_digest,
                kind: PreparedSessionTransitionKindV1::NoChange {
                    result: Some(self.operation_result()?),
                },
            }),
            Candidate::Changed(document) => {
                let revision = current
                    .next_revision()
                    .ok_or(DocumentSessionError::RevisionExhausted)?;
                let state = RevisionState::from_document(revision, *document)
                    .map_err(DocumentSessionError::Load)?;
                self.prepare_changed_session_transition_v1(
                    source_revision,
                    source_digest,
                    state,
                    SessionTransitionEffectsV1::none(),
                )
            }
        }
    }

    /// Verify and atomically redeem one opaque renderer-admitted session transition.
    pub fn commit_session_operation_transition_v1(
        &mut self,
        prepared: &mut PreparedSessionTransitionV1,
    ) -> Result<SessionOperationResultV1, AdmittedSessionTransitionRefusalV1> {
        if !prepared
            .issuer
            .same_issuer(&self.authoring_capability_issuer_v1())
        {
            return Err(AdmittedSessionTransitionRefusalV1::ForeignSession);
        }
        if prepared.is_consumed_v1() {
            return Err(AdmittedSessionTransitionRefusalV1::Replayed);
        }
        if self.admitted_history.current().revision() != prepared.source_revision
            || *self.admitted_history.current().digest() != prepared.source_digest
        {
            return Err(AdmittedSessionTransitionRefusalV1::StaleSnapshot);
        }
        match &mut prepared.kind {
            PreparedSessionTransitionKindV1::NoChange { result } => Ok(result
                .take()
                .expect("the consumed check established the no-change result invariant")),
            PreparedSessionTransitionKindV1::Changed(changed) => {
                changed
                    .renderer_admission
                    .verify(&changed.observation)
                    .map_err(|_| AdmittedSessionTransitionRefusalV1::RendererAdmission)?;
                changed
                    .effects
                    .verify_provisional_token(self)
                    .map_err(|_| AdmittedSessionTransitionRefusalV1::ProvisionalCapability)?;
                match changed.commit {
                    ChangedTransitionCommitV1::Append => {
                        changed.effects.consume_provisional_token(self);
                        let state = changed
                            .state
                            .take()
                            .expect("the consumed check established the changed state invariant");
                        let result = changed
                            .result
                            .take()
                            .expect("the consumed check established the changed result invariant");
                        self.admitted_history.append(state);
                        changed.effects.install_generated_ids(self);
                        Ok(result)
                    }
                    ChangedTransitionCommitV1::Navigate(direction) => {
                        changed.effects.consume_provisional_token(self);
                        let state = changed
                            .state
                            .take()
                            .expect("the consumed check established the changed state invariant");
                        let result = changed
                            .result
                            .take()
                            .expect("the consumed check established the changed result invariant");
                        match direction {
                            HistoryNavigationDirectionV1::Undo => self.admitted_history.move_undo(),
                            HistoryNavigationDirectionV1::Redo => self.admitted_history.move_redo(),
                        }
                        self.admitted_history.replace_current(state);
                        changed.effects.install_generated_ids(self);
                        Ok(result)
                    }
                }
            }
        }
    }

    /// Execute one typed operation through the renderer-admitted transition boundary.
    pub fn execute_session_operation_transition_v1(
        &mut self,
        expected_revision: u64,
        operation: SessionOperation,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        let mut prepared =
            self.prepare_session_operation_transition_v1(expected_revision, operation)?;
        self.commit_session_operation_transition_v1(&mut prepared)
            .map_err(|refusal| self.map_admitted_transition_refusal_v1(&prepared, refusal))
    }

    /// Build one changed renderer-admitted transition from a session-owned candidate.
    pub(super) fn prepare_changed_session_transition_v1(
        &mut self,
        source_revision: u64,
        source_digest: [u8; 32],
        state: RevisionState,
        effects: SessionTransitionEffectsV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.prepare_changed_session_transition_with_commit_v1(
            source_revision,
            source_digest,
            state,
            effects,
            ChangedTransitionCommitV1::Append,
        )
    }

    /// Reserve generated IDs for a candidate without changing this live session.
    ///
    /// The returned effects install the resulting sequence only after the core
    /// commits the prepared transition. Routes must retain those effects rather
    /// than assigning `generated_ids` after an append.
    pub(super) fn reserve_generated_ids_for_transition_v1<T>(
        &self,
        reserve: impl FnOnce(
            super::GeneratedIdSequences,
            &IndexedDocument,
        ) -> Result<(T, super::GeneratedIdSequences), SessionOperationError>,
    ) -> Result<(T, SessionTransitionEffectsV1), SessionOperationError> {
        let (value, next_generated_ids) = reserve(
            self.generated_ids,
            self.admitted_history.current().document().indexed(),
        )?;
        Ok((
            value,
            SessionTransitionEffectsV1::none().installing_generated_ids(next_generated_ids),
        ))
    }

    pub(super) fn prepare_history_navigation_transition_v1(
        &mut self,
        expected_revision: u64,
        direction: HistoryNavigationDirectionV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let source = match direction {
            HistoryNavigationDirectionV1::Undo => self.admitted_history.undo_target(),
            HistoryNavigationDirectionV1::Redo => self.admitted_history.redo_target(),
        }
        .ok_or(DocumentSessionError::HistoryUnavailable)?
        .canonical_cdml()
        .to_owned();
        let (source_revision, source_digest, revision) = {
            let current = self.admitted_history.current();
            (
                current.revision(),
                *current.digest(),
                current
                    .next_revision()
                    .ok_or(DocumentSessionError::RevisionExhausted)?,
            )
        };
        let document = super::TypedDocument::parse(&source).map_err(DocumentSessionError::Load)?;
        let state =
            RevisionState::from_document(revision, document).map_err(DocumentSessionError::Load)?;
        self.prepare_changed_session_transition_with_commit_v1(
            source_revision,
            source_digest,
            state,
            SessionTransitionEffectsV1::none(),
            ChangedTransitionCommitV1::Navigate(direction),
        )
    }

    pub(super) fn execute_history_navigation_transition_v1(
        &mut self,
        expected_revision: u64,
        direction: HistoryNavigationDirectionV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        let mut prepared =
            self.prepare_history_navigation_transition_v1(expected_revision, direction)?;
        self.commit_session_operation_transition_v1(&mut prepared)
            .map_err(|refusal| self.map_admitted_transition_refusal_v1(&prepared, refusal))
    }

    fn prepare_changed_session_transition_with_commit_v1(
        &mut self,
        source_revision: u64,
        source_digest: [u8; 32],
        state: RevisionState,
        effects: SessionTransitionEffectsV1,
        commit: ChangedTransitionCommitV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(source_revision)?;
        if *self.admitted_history.current().digest() != source_digest {
            return Err(DocumentSessionError::RevisionConflict {
                expected: source_revision,
                actual: self.admitted_history.current().revision(),
            });
        }
        if state.revision()
            != self
                .admitted_history
                .current()
                .next_revision()
                .ok_or(DocumentSessionError::RevisionExhausted)?
        {
            return Err(DocumentSessionError::RevisionConflict {
                expected: source_revision,
                actual: self.admitted_history.current().revision(),
            });
        }
        let snapshot = state.snapshot(!self.saved_baseline.is_current(&state));
        let observation = SessionDocumentObservationV1::from_snapshot(snapshot)
            .map_err(DocumentSessionError::Projection)?;
        let renderer_admission = RendererAdmittedPendingV1::admit(self, &observation)
            .map_err(|_| DocumentSessionError::RendererAdmission)?;
        effects.verify_provisional_token(self)?;
        if commit == ChangedTransitionCommitV1::Append {
            self.admitted_history.ensure_append_slot().map_err(|_| {
                DocumentSessionError::Operation(SessionOperationError::HistoryResourceExhausted)
            })?;
        }
        Ok(PreparedSessionTransitionV1 {
            issuer: self.authoring_capability_issuer_v1(),
            source_revision,
            source_digest,
            kind: PreparedSessionTransitionKindV1::Changed(PreparedChangedSessionTransitionV1 {
                state: Some(state),
                result: Some(SessionOperationResultV1::new(observation.clone())),
                observation,
                renderer_admission,
                effects,
                commit,
            }),
        })
    }

    fn map_admitted_transition_refusal_v1(
        &self,
        prepared: &PreparedSessionTransitionV1,
        refusal: AdmittedSessionTransitionRefusalV1,
    ) -> DocumentSessionError {
        match refusal {
            AdmittedSessionTransitionRefusalV1::ForeignSession => {
                DocumentSessionError::PreparedOperationForeignSession
            }
            AdmittedSessionTransitionRefusalV1::Replayed => {
                DocumentSessionError::PreparedOperationConsumed
            }
            AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
                DocumentSessionError::RevisionConflict {
                    expected: prepared.source_revision,
                    actual: self.admitted_history.current().revision(),
                }
            }
            AdmittedSessionTransitionRefusalV1::RendererAdmission => {
                DocumentSessionError::RendererAdmission
            }
            AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
                DocumentSessionError::PreparedOperationConsumed
            }
            AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
                DocumentSessionError::Operation(SessionOperationError::HistoryResourceExhausted)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DrawingStandardPatchV1, DrawingStandardPropertyChangeV1, SessionOperationV1};

    fn changed_operation(line_width: f64) -> SessionOperation {
        SessionOperation::V1(SessionOperationV1::SetDrawingStandard {
            patch: DrawingStandardPatchV1::new(vec![DrawingStandardPropertyChangeV1::LineWidth(
                line_width,
            )])
            .expect("test change is valid"),
        })
    }

    fn no_change_operation() -> SessionOperation {
        SessionOperation::V1(SessionOperationV1::SetDrawingStandard {
            patch: DrawingStandardPatchV1::new(Vec::new()).expect("empty patch is valid"),
        })
    }

    fn next_reserved_atom_identifier(session: &DocumentSession) -> String {
        let (identifier, _effects) = session
            .reserve_generated_ids_for_transition_v1(|sequences, indexed| {
                sequences.reserve_atom(indexed)
            })
            .expect("atom identifier reserves");
        identifier.as_str().to_owned()
    }

    fn changed_state(session: &DocumentSession, line_width: f64) -> (u64, [u8; 32], RevisionState) {
        let current = session.admitted_history.current();
        let source_revision = current.revision();
        let source_digest = *current.digest();
        let Candidate::Changed(document) = changed_operation(line_width)
            .prepare(current.document(), source_revision, &source_digest)
            .expect("changed operation prepares")
        else {
            panic!("test operation must change the document");
        };
        let revision = current.next_revision().expect("revision advances");
        let state = RevisionState::from_document(revision, *document).expect("candidate state");
        (source_revision, source_digest, state)
    }

    fn prepared_generated_id_transition(
        session: &mut DocumentSession,
        line_width: f64,
    ) -> (String, PreparedSessionTransitionV1) {
        let (identifier, effects) = session
            .reserve_generated_ids_for_transition_v1(|sequences, indexed| {
                sequences.reserve_atom(indexed)
            })
            .expect("atom identifier reserves");
        let (source_revision, source_digest, state) = changed_state(session, line_width);
        let prepared = session
            .prepare_changed_session_transition_v1(source_revision, source_digest, state, effects)
            .expect("admitted transition prepares");
        (identifier.as_str().to_owned(), prepared)
    }

    #[test]
    fn changed_transition_is_renderer_admitted_atomic_and_one_use() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let mut prepared = session
            .prepare_session_operation_transition_v1(0, changed_operation(2.0))
            .expect("changed transition is admitted");
        let _result = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("admitted transition commits");
        assert_eq!(session.snapshot().expect("snapshot").revision(), 1);
        assert!(prepared.is_consumed_v1());
        assert_eq!(
            session.commit_session_operation_transition_v1(&mut prepared),
            Err(AdmittedSessionTransitionRefusalV1::Replayed)
        );
    }

    #[test]
    fn no_change_transition_is_history_free_and_one_use() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let mut prepared = session
            .prepare_session_operation_transition_v1(0, no_change_operation())
            .expect("no-change transition prepares");
        let _result = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("no-change transition completes");
        assert_eq!(session.snapshot().expect("snapshot").revision(), 0);
        assert_eq!(
            session.commit_session_operation_transition_v1(&mut prepared),
            Err(AdmittedSessionTransitionRefusalV1::Replayed)
        );
    }

    #[test]
    fn route_no_change_adapter_is_history_and_renderer_free() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let mut prepared = session
            .prepare_no_change_session_transition_v1(0)
            .expect("no-change route transition prepares");
        let metadata = prepared.metadata_v1().expect("metadata remains available");
        assert_eq!(metadata.observation().snapshot().revision(), 0);
        assert!(metadata.renderer_plan().is_none());
        assert!(session.admitted_history.undo_target().is_none());
        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("no-change route transition commits");
        assert!(session.admitted_history.undo_target().is_none());
    }

    #[test]
    fn prepared_siblings_share_the_history_append_slot_without_mutating_history() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let mut first = session
            .prepare_session_operation_transition_v1(0, changed_operation(2.0))
            .expect("first sibling prepares");
        let mut second = session
            .prepare_session_operation_transition_v1(0, changed_operation(3.0))
            .expect("second sibling prepares against the same preallocated history slot");
        assert_eq!(session.snapshot().expect("snapshot").revision(), 0);
        assert!(session.admitted_history.undo_target().is_none());

        session
            .commit_session_operation_transition_v1(&mut first)
            .expect("first sibling commits");
        assert_eq!(
            session.commit_session_operation_transition_v1(&mut second),
            Err(AdmittedSessionTransitionRefusalV1::StaleSnapshot)
        );
        assert!(!second.is_consumed_v1());
    }

    #[test]
    fn retirement_is_semantic_cancellation_without_history_resource_cleanup() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let before = session.snapshot().expect("snapshot");
        let (identifier, mut prepared) = prepared_generated_id_transition(&mut session, 2.0);

        session
            .retire_session_operation_transition_v1(&mut prepared)
            .expect("owner retires transition");

        assert_eq!(session.snapshot().expect("snapshot"), before);
        assert!(session.admitted_history.undo_target().is_none());
        assert_eq!(next_reserved_atom_identifier(&session), identifier);
        assert!(prepared.is_consumed_v1());
        assert!(prepared.metadata_v1().is_none());
        assert_eq!(
            session.commit_session_operation_transition_v1(&mut prepared),
            Err(AdmittedSessionTransitionRefusalV1::Replayed)
        );

        let mut replacement = session
            .prepare_session_operation_transition_v1(0, changed_operation(2.0))
            .expect("fresh transition prepares after cancellation");
        session
            .commit_session_operation_transition_v1(&mut replacement)
            .expect("replacement renderer-admitted transition commits");
        assert_eq!(session.snapshot().expect("snapshot").revision(), 1);
        assert_eq!(
            session.retire_session_operation_transition_v1(&mut replacement),
            Err(AdmittedSessionTransitionRefusalV1::Replayed)
        );
    }

    #[test]
    fn dropped_preparation_leaves_generated_ids_tentative_until_replacement_commits() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let (identifier, abandoned) = prepared_generated_id_transition(&mut session, 2.0);
        assert_eq!(identifier, "ferrum-atom-v1-0");
        drop(abandoned);

        assert_eq!(next_reserved_atom_identifier(&session), identifier);
        let (replacement_identifier, mut replacement) =
            prepared_generated_id_transition(&mut session, 3.0);
        assert_eq!(replacement_identifier, identifier);
        assert_eq!(next_reserved_atom_identifier(&session), identifier);

        session
            .commit_session_operation_transition_v1(&mut replacement)
            .expect("replacement commits");
        assert_eq!(next_reserved_atom_identifier(&session), "ferrum-atom-v1-1");
    }

    #[test]
    fn foreign_retirement_cannot_invalidate_the_owner_pending_transition() {
        let mut owner = DocumentSession::create_empty_document_v1().expect("owner session");
        let mut other = DocumentSession::create_empty_document_v1().expect("other session");
        let mut prepared = owner
            .prepare_session_operation_transition_v1(0, changed_operation(2.0))
            .expect("owner transition prepares");

        assert_eq!(
            other.retire_session_operation_transition_v1(&mut prepared),
            Err(AdmittedSessionTransitionRefusalV1::ForeignSession)
        );
        assert!(!prepared.is_consumed_v1());
        owner
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("foreign retirement refusal leaves owner transition redeemable");
    }

    #[test]
    fn retired_transition_cannot_expose_or_bypass_renderer_admission() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let mut retired = session
            .prepare_session_operation_transition_v1(0, changed_operation(2.0))
            .expect("renderer-admitted transition prepares");
        assert!(
            retired
                .metadata_v1()
                .expect("live transition exposes its renderer plan")
                .renderer_plan()
                .is_some()
        );

        session
            .retire_session_operation_transition_v1(&mut retired)
            .expect("transition retires");
        assert!(retired.metadata_v1().is_none());
        assert_eq!(
            session.commit_session_operation_transition_v1(&mut retired),
            Err(AdmittedSessionTransitionRefusalV1::Replayed)
        );

        let mut replacement = session
            .prepare_session_operation_transition_v1(0, changed_operation(2.0))
            .expect("only a fresh renderer-admitted transition can replace it");
        session
            .commit_session_operation_transition_v1(&mut replacement)
            .expect("fresh renderer-admitted transition commits");
    }

    #[test]
    fn generated_ids_install_only_after_successful_renderer_admitted_redemption() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let (identifier, mut prepared) = prepared_generated_id_transition(&mut session, 2.0);
        assert_eq!(identifier, "ferrum-atom-v1-0");
        assert_eq!(next_reserved_atom_identifier(&session), identifier);
        let metadata = prepared.metadata_v1().expect("metadata remains available");
        assert_eq!(metadata.observation().snapshot().revision(), 1);
        assert!(metadata.renderer_plan().is_some());

        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("admitted transition commits");
        assert_eq!(next_reserved_atom_identifier(&session), "ferrum-atom-v1-1");
        assert_eq!(
            session.commit_session_operation_transition_v1(&mut prepared),
            Err(AdmittedSessionTransitionRefusalV1::Replayed)
        );
        assert_eq!(next_reserved_atom_identifier(&session), "ferrum-atom-v1-1");
    }

    #[test]
    fn generated_ids_remain_tentative_after_foreign_stale_or_renderer_refusal() {
        let mut owner = DocumentSession::create_empty_document_v1().expect("owner session");
        let mut other = DocumentSession::create_empty_document_v1().expect("other session");
        let (identifier, mut foreign) = prepared_generated_id_transition(&mut owner, 2.0);
        assert_eq!(
            other.commit_session_operation_transition_v1(&mut foreign),
            Err(AdmittedSessionTransitionRefusalV1::ForeignSession)
        );
        assert_eq!(next_reserved_atom_identifier(&owner), identifier);

        let (_identifier, mut stale) = prepared_generated_id_transition(&mut owner, 3.0);
        owner
            .execute_session_operation_transition_v1(0, changed_operation(4.0))
            .expect("independent transition advances source fence");
        assert_eq!(
            owner.commit_session_operation_transition_v1(&mut stale),
            Err(AdmittedSessionTransitionRefusalV1::StaleSnapshot)
        );
        assert_eq!(next_reserved_atom_identifier(&owner), identifier);

        let mut renderer_session =
            DocumentSession::create_empty_document_v1().expect("renderer session");
        let (identifier, mut renderer_refused) =
            prepared_generated_id_transition(&mut renderer_session, 2.0);
        let PreparedSessionTransitionKindV1::Changed(changed) = &mut renderer_refused.kind else {
            panic!("test transition must be changed");
        };
        changed.observation = renderer_session
            .document_observation()
            .expect("current observation");
        assert_eq!(
            renderer_session.commit_session_operation_transition_v1(&mut renderer_refused),
            Err(AdmittedSessionTransitionRefusalV1::RendererAdmission)
        );
        assert_eq!(next_reserved_atom_identifier(&renderer_session), identifier);
    }

    #[test]
    fn route_capability_claim_precedes_generic_replay_refusal() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let mut prepared = session
            .prepare_session_operation_transition_v1(0, changed_operation(2.0))
            .expect("changed transition prepares");
        let mut claimed = false;
        session
            .commit_session_transition_after_route_claim_v1(&mut prepared, || {
                claimed = true;
                Ok::<(), &'static str>(())
            })
            .expect("claimed transition commits");
        assert!(claimed);
        assert_eq!(
            session.commit_session_transition_after_route_claim_v1(&mut prepared, || {
                Err::<(), _>("route capability already claimed")
            }),
            Err(RouteClaimedTransitionErrorV1::Claim(
                "route capability already claimed"
            ))
        );
    }

    #[test]
    fn admitted_history_navigation_preserves_monotonic_revisions() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let changed = session
            .execute_session_operation_transition_v1(0, changed_operation(2.0))
            .expect("changed transition commits");
        let undone = session
            .undo(changed.observation().snapshot().revision())
            .expect("renderer-admitted undo commits");
        let redone = session
            .redo(undone.observation().snapshot().revision())
            .expect("renderer-admitted redo commits");
        assert_eq!(changed.observation().snapshot().revision(), 1);
        assert_eq!(undone.observation().snapshot().revision(), 2);
        assert_eq!(redone.observation().snapshot().revision(), 3);
    }

    #[test]
    fn foreign_or_stale_refusal_preserves_the_owner_pending_transition() {
        let mut owner = DocumentSession::create_empty_document_v1().expect("owner session");
        let mut other = DocumentSession::create_empty_document_v1().expect("other session");
        let mut foreign = owner
            .prepare_session_operation_transition_v1(0, changed_operation(2.0))
            .expect("owner transition prepares");
        assert_eq!(
            other.commit_session_operation_transition_v1(&mut foreign),
            Err(AdmittedSessionTransitionRefusalV1::ForeignSession)
        );
        assert!(!foreign.is_consumed_v1());
        owner
            .commit_session_operation_transition_v1(&mut foreign)
            .expect("foreign refusal left owner transition redeemable");

        let mut stale = owner
            .prepare_session_operation_transition_v1(1, changed_operation(3.0))
            .expect("stale transition prepares");
        owner
            .submit(1, changed_operation(4.0))
            .expect("independent transition advances source fence");
        assert_eq!(
            owner.commit_session_operation_transition_v1(&mut stale),
            Err(AdmittedSessionTransitionRefusalV1::StaleSnapshot)
        );
        assert!(!stale.is_consumed_v1());
    }

    #[test]
    fn renderer_refusal_leaves_the_source_session_unchanged() {
        let source = concat!(
            "<c:cdml xmlns:c=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\">",
            "<c:info/><v:before/><c:metadata/>",
            "<c:standard line_width=\"1\" font_size=\"12\" font_family=\"Telex\" ",
            "line_color=\"#000000\" area_color=\"\" paper_type=\"Letter\" v:keep=\"yes\">",
            "<c:bond width=\"6\" wedge-width=\"5\" double-ratio=\"0.75\" ",
            "v:bond=\"keep\"><v:child/></c:bond><c:atom show_hydrogens=\"0\"/>",
            "</c:standard><c:molecule id=\"m\"><c:atom id=\"a\" name=\"C\"><c:point x=\"0\" y=\"0\"/></c:atom></c:molecule><c:standard line_width=\"99\"/>",
            "</c:cdml>"
        );
        let mut session = DocumentSession::load(source).expect("retained source is valid");
        assert!(matches!(
            session.prepare_session_operation_transition_v1(0, changed_operation(2.0)),
            Err(DocumentSessionError::RendererAdmission)
        ));
        assert_eq!(session.snapshot().expect("snapshot").revision(), 0);
    }
}
