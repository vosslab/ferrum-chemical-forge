//! Private complete-render admission retained by document-owned pending transitions.

use ferrum_document_projection::PresentationRootProjectionV1;
use ferrum_render::{
    AcceptedCompleteRenderV1, AcceptedRenderOverlayRequestV1,
    admit_complete_document_render_with_resolved_v1,
};
use ferrum_render_contract::{
    CompleteDocumentSourceFenceV1, CompleteRenderPendingIdentityV1, CompleteRenderPrimitiveV1,
    CompleteRenderRootCandidateV1, CompleteRenderRootIdentityV1, CompleteRenderRootLoweringV1,
    DocumentCompleteRenderCandidateV1,
};

use super::{DocumentSession, SessionDocumentObservationV1};
use crate::derive_document_render_observation_from_accepted_operation_v1;

/// Renderer acceptance bound to one document-session pending identity.
#[derive(Debug)]
pub(super) struct RendererAdmittedPendingV1 {
    identity: CompleteRenderPendingIdentityV1,
    candidate: DocumentCompleteRenderCandidateV1,
    acceptance: AcceptedCompleteRenderV1,
}

impl RendererAdmittedPendingV1 {
    pub(super) fn admit(
        session: &mut DocumentSession,
        observation: &SessionDocumentObservationV1,
    ) -> Result<Self, RendererAdmittedPendingErrorV1> {
        let identity = session.next_renderer_pending_identity_v1();
        let candidate = candidate_from_observation_v1(
            observation,
            session.renderer_admission_issuer,
            identity,
        )?;
        let render_observation =
            derive_document_render_observation_from_accepted_operation_v1(observation)
                .map_err(|_| RendererAdmittedPendingErrorV1::Admission)?;
        let acceptance = admit_complete_document_render_with_resolved_v1(
            &candidate,
            render_observation.resolved(),
        )
        .map_err(|_| RendererAdmittedPendingErrorV1::Admission)?;
        Ok(Self {
            identity,
            candidate,
            acceptance,
        })
    }

    pub(super) fn verify(
        &self,
        observation: &SessionDocumentObservationV1,
    ) -> Result<(), RendererAdmittedPendingErrorV1> {
        let candidate = candidate_from_observation_v1(
            observation,
            self.candidate.source_fence().issuer(),
            self.identity,
        )?;
        if candidate != self.candidate {
            return Err(RendererAdmittedPendingErrorV1::Admission);
        }
        let render_observation =
            derive_document_render_observation_from_accepted_operation_v1(observation)
                .map_err(|_| RendererAdmittedPendingErrorV1::Admission)?;
        let reaccepted = admit_complete_document_render_with_resolved_v1(
            &candidate,
            render_observation.resolved(),
        )
        .map_err(|_| RendererAdmittedPendingErrorV1::Admission)?;
        (reaccepted == self.acceptance)
            .then_some(())
            .ok_or(RendererAdmittedPendingErrorV1::Admission)
    }

    pub(super) fn precommit_overlay_v1(
        &self,
        request: &AcceptedRenderOverlayRequestV1,
    ) -> Result<ferrum_render::DocumentPrecommitOverlayV1, RendererAdmittedPendingErrorV1> {
        self.acceptance
            .precommit_overlay_v1(request)
            .map_err(|_| RendererAdmittedPendingErrorV1::Admission)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RendererAdmittedPendingErrorV1 {
    Admission,
}

impl DocumentSession {
    fn next_renderer_pending_identity_v1(&mut self) -> CompleteRenderPendingIdentityV1 {
        let identity = CompleteRenderPendingIdentityV1::new(
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
    issuer: u64,
    identity: CompleteRenderPendingIdentityV1,
) -> Result<DocumentCompleteRenderCandidateV1, RendererAdmittedPendingErrorV1> {
    let projection = observation.projection();
    let mut roots = Vec::with_capacity(
        projection.molecules().len()
            + projection.presentation_stack().roots().len()
            + projection.presentation_stack().issues().len(),
    );
    for molecule in projection.molecules() {
        roots.push(CompleteRenderRootCandidateV1::new(
            durable_identity_v1(molecule.id())?,
            molecule.source_order(),
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Molecule),
        ));
    }
    for root in projection.presentation_stack().roots() {
        roots.push(CompleteRenderRootCandidateV1::new(
            durable_identity_v1(root.target().id())?,
            root.target().source_order(),
            presentation_root_lowering_v1(root),
        ));
    }
    for issue in projection.presentation_stack().issues() {
        roots.push(CompleteRenderRootCandidateV1::new(
            durable_identity_v1(issue.target().id())?,
            issue.target().source_order(),
            CompleteRenderRootLoweringV1::MissingRequiredPrimitive,
        ));
    }
    roots.sort_by_key(CompleteRenderRootCandidateV1::source_order);
    DocumentCompleteRenderCandidateV1::new(
        CompleteDocumentSourceFenceV1::new(
            issuer,
            observation.snapshot().revision(),
            *observation.snapshot().digest(),
        ),
        identity,
        roots,
    )
    .map_err(|_| RendererAdmittedPendingErrorV1::Admission)
}

fn presentation_root_lowering_v1(
    root: &PresentationRootProjectionV1,
) -> CompleteRenderRootLoweringV1 {
    match root {
        PresentationRootProjectionV1::Plus { .. } | PresentationRootProjectionV1::Text { .. } => {
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Text)
        }
        PresentationRootProjectionV1::Arrow { .. }
        | PresentationRootProjectionV1::Polyline { .. }
        | PresentationRootProjectionV1::Wavy { .. }
        | PresentationRootProjectionV1::RoundBracket { .. }
        | PresentationRootProjectionV1::Rectangle { .. }
        | PresentationRootProjectionV1::Square { .. }
        | PresentationRootProjectionV1::Oval { .. }
        | PresentationRootProjectionV1::Circle { .. }
        | PresentationRootProjectionV1::Polygon { .. } => {
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Vector)
        }
    }
}

fn durable_identity_v1(
    identity: Option<&crate::DocumentObjectIdV1>,
) -> Result<CompleteRenderRootIdentityV1, RendererAdmittedPendingErrorV1> {
    identity
        .ok_or(RendererAdmittedPendingErrorV1::Admission)
        .and_then(|value| {
            CompleteRenderRootIdentityV1::new(value.as_str())
                .map_err(|_| RendererAdmittedPendingErrorV1::Admission)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_render_contract::{
        CompleteRenderAdmissionRefusalV1, CompleteRenderRootClassV1, RefusedRootReasonV1,
    };

    #[test]
    fn candidate_classifies_text_layout_and_missing_vector_primitives() {
        let source = concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\"><plus id=\"label\"><point x=\"1\" y=\"2\"/>",
            "</plus><polygon id=\"legacy\"/></cdml>",
        );
        let session = DocumentSession::load(source).expect("source loads");
        let observation = session.observe(0).expect("observation projects");
        let candidate = candidate_from_observation_v1(
            &observation,
            7,
            CompleteRenderPendingIdentityV1::new(7, 1),
        )
        .expect("candidate derives");

        assert_eq!(
            candidate.roots()[0].lowering(),
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Text)
        );
        assert_eq!(
            candidate.roots()[1].lowering(),
            CompleteRenderRootLoweringV1::MissingRequiredPrimitive
        );
        assert!(matches!(
            ferrum_render::admit_complete_document_render_v1(&candidate),
            Err(CompleteRenderAdmissionRefusalV1::RootRefused {
                class: CompleteRenderRootClassV1::Refused(
                    RefusedRootReasonV1::MissingRequiredPrimitive
                ),
                ..
            })
        ));
    }
}
