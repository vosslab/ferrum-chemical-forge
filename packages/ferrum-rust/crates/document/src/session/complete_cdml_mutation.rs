//! Typed complete-CDML adapters over the generic admitted transition boundary.

use crate::{DocumentFenceV1, SessionOperationResultV1, TopLevelTransformV1, TypedDocument};

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentSession, PreparedSessionTransitionV1,
    RevisionState, SessionTransitionEffectsV1,
};

/// Closed refusal set for a renderer-admitted complete CDML replacement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CompleteCdmlMutationRefusalV1 {
    StaleSnapshot,
    ForeignSession,
    Replayed,
    InvalidCandidate,
    UnrenderableCandidate,
    RendererAdmission,
    SessionConflict,
}

/// Opaque one-use complete-CDML mutation with route-specific fence semantics.
#[derive(Debug)]
pub struct PendingCompleteCdmlMutationV1 {
    transition: PreparedSessionTransitionV1,
}

impl PendingCompleteCdmlMutationV1 {
    #[must_use]
    pub fn is_consumed_v1(&self) -> bool {
        self.transition.is_consumed_v1()
    }
}

impl DocumentSession {
    /// Parse and renderer-admit one complete-CDML replacement without changing history.
    pub fn prepare_complete_cdml_mutation_v1(
        &mut self,
        fence: DocumentFenceV1,
        candidate_cdml: &str,
    ) -> Result<PendingCompleteCdmlMutationV1, CompleteCdmlMutationRefusalV1> {
        self.require_complete_cdml_fence_v1(fence)?;
        let document = TypedDocument::parse(candidate_cdml)
            .map_err(|_| CompleteCdmlMutationRefusalV1::InvalidCandidate)?;
        self.prepare_complete_cdml_document_v1(fence, document)
    }

    /// Apply one complete-root transform and admit its typed prospective candidate.
    pub fn prepare_top_level_transform_complete_cdml_mutation_v1(
        &mut self,
        fence: DocumentFenceV1,
        transform: &TopLevelTransformV1,
    ) -> Result<PendingCompleteCdmlMutationV1, CompleteCdmlMutationRefusalV1> {
        self.require_complete_cdml_fence_v1(fence)?;
        let candidate = self
            .current_document_v1()
            .with_top_level_transform(transform)
            .map_err(|_| CompleteCdmlMutationRefusalV1::InvalidCandidate)?;
        self.prepare_complete_cdml_document_v1(fence, candidate)
    }

    /// Verify and atomically append one renderer-admitted complete-CDML mutation.
    pub fn commit_complete_cdml_mutation_v1(
        &mut self,
        pending: &mut PendingCompleteCdmlMutationV1,
    ) -> Result<SessionOperationResultV1, CompleteCdmlMutationRefusalV1> {
        self.commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(map_complete_cdml_refusal)
    }

    fn prepare_complete_cdml_document_v1(
        &mut self,
        fence: DocumentFenceV1,
        document: TypedDocument,
    ) -> Result<PendingCompleteCdmlMutationV1, CompleteCdmlMutationRefusalV1> {
        let revision = self
            .next_revision_v1()
            .ok_or(CompleteCdmlMutationRefusalV1::SessionConflict)?;
        let state = RevisionState::from_document(revision, document)
            .map_err(|_| CompleteCdmlMutationRefusalV1::InvalidCandidate)?;
        let transition = self
            .prepare_changed_session_transition_v1(
                fence.revision(),
                fence.digest(),
                state,
                SessionTransitionEffectsV1::none(),
            )
            .map_err(|error| match error {
                super::DocumentSessionError::RendererAdmission => {
                    CompleteCdmlMutationRefusalV1::RendererAdmission
                }
                _ => CompleteCdmlMutationRefusalV1::SessionConflict,
            })?;
        Ok(PendingCompleteCdmlMutationV1 { transition })
    }

    fn require_complete_cdml_fence_v1(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<(), CompleteCdmlMutationRefusalV1> {
        if self.require_current(fence.revision()).is_err()
            || self.current_digest_v1() != fence.digest()
        {
            return Err(CompleteCdmlMutationRefusalV1::StaleSnapshot);
        }
        Ok(())
    }
}

fn map_complete_cdml_refusal(
    refusal: AdmittedSessionTransitionRefusalV1,
) -> CompleteCdmlMutationRefusalV1 {
    match refusal {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            CompleteCdmlMutationRefusalV1::ForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Replayed => CompleteCdmlMutationRefusalV1::Replayed,
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            CompleteCdmlMutationRefusalV1::StaleSnapshot
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            CompleteCdmlMutationRefusalV1::RendererAdmission
        }
        AdmittedSessionTransitionRefusalV1::ProvisionalCapability
        | AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
            CompleteCdmlMutationRefusalV1::SessionConflict
        }
    }
}
