//! Private renderer proof retained by document-owned pending transactions.

use ferrum_render::{
    AdmittedDocumentRenderCandidateV1, DocumentRenderCandidateV1, DocumentRenderPendingIdentityV1,
    admit_document_render_candidate_v1, compose_document_render_plan_v1,
};

use super::{DocumentSession, SessionDocumentObservationV1};
use crate::derive_document_render_observation_from_accepted_operation_v1;

/// Renderer proof bound to one document-session pending identity.
#[derive(Debug)]
pub(super) struct RendererAdmittedPendingV1 {
    identity: DocumentRenderPendingIdentityV1,
    admission: AdmittedDocumentRenderCandidateV1,
}

impl RendererAdmittedPendingV1 {
    pub(super) fn admit(
        session: &mut DocumentSession,
        observation: &SessionDocumentObservationV1,
    ) -> Result<Self, RendererAdmittedPendingErrorV1> {
        let identity = session.next_renderer_pending_identity_v1();
        let candidate = candidate_from_observation_v1(observation, identity)?;
        let admission = admit_document_render_candidate_v1(&candidate)
            .map_err(|_| RendererAdmittedPendingErrorV1::Admission)?;
        Ok(Self {
            identity,
            admission,
        })
    }

    pub(super) fn verify(
        &self,
        observation: &SessionDocumentObservationV1,
    ) -> Result<(), RendererAdmittedPendingErrorV1> {
        let candidate = candidate_from_observation_v1(observation, self.identity)?;
        self.admission
            .verify_candidate_v1(&candidate)
            .map_err(|_| RendererAdmittedPendingErrorV1::Admission)
    }

    pub(super) fn plan(&self) -> &ferrum_render::DocumentRenderPlanV1 {
        self.admission.plan()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RendererAdmittedPendingErrorV1 {
    Admission,
}

impl DocumentSession {
    fn next_renderer_pending_identity_v1(&mut self) -> DocumentRenderPendingIdentityV1 {
        let identity = DocumentRenderPendingIdentityV1::new(
            self.renderer_admission_issuer,
            self.next_renderer_admission_sequence,
        );
        self.next_renderer_admission_sequence = self
            .next_renderer_admission_sequence
            .checked_add(1)
            .expect("renderer pending sequence remains representable");
        identity
    }
}

fn candidate_from_observation_v1(
    observation: &SessionDocumentObservationV1,
    identity: DocumentRenderPendingIdentityV1,
) -> Result<DocumentRenderCandidateV1, RendererAdmittedPendingErrorV1> {
    let render_observation =
        derive_document_render_observation_from_accepted_operation_v1(observation)
            .map_err(|_| RendererAdmittedPendingErrorV1::Admission)?;
    let plan = compose_document_render_plan_v1(render_observation.resolved())
        .map_err(|_| RendererAdmittedPendingErrorV1::Admission)?;
    DocumentRenderCandidateV1::from_complete_plan(plan, identity)
        .map_err(|_| RendererAdmittedPendingErrorV1::Admission)
}
