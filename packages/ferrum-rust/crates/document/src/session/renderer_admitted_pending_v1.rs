//! Private complete-render admission retained by document-owned pending transitions.

use ferrum_document_projection::{
    DocumentDirectRootKindV1, DocumentDirectRootV1, PresentationRecordKindV1,
    PresentationRootProjectionV1,
};
use ferrum_render::{
    AcceptedCompleteRenderV1, AcceptedRenderOverlayRequestV1, admit_complete_document_render_v1,
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
    render_observation: crate::DocumentRenderObservationV1,
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
        let acceptance = admit_complete_document_render_v1(&candidate)
            .map_err(|_| RendererAdmittedPendingErrorV1::Admission)?;
        Ok(Self {
            identity,
            candidate,
            acceptance,
            render_observation,
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
        let _render_observation =
            derive_document_render_observation_from_accepted_operation_v1(observation)
                .map_err(|_| RendererAdmittedPendingErrorV1::Admission)?;
        let reaccepted = admit_complete_document_render_v1(&candidate)
            .map_err(|_| RendererAdmittedPendingErrorV1::Admission)?;
        (reaccepted == self.acceptance)
            .then_some(())
            .ok_or(RendererAdmittedPendingErrorV1::Admission)
    }

    pub(super) fn precommit_overlay_v1(
        &self,
        request: &AcceptedRenderOverlayRequestV1,
    ) -> Result<ferrum_render::DocumentPrecommitOverlayV1, RendererAdmittedPendingErrorV1> {
        admit_complete_document_render_with_resolved_v1(
            &self.candidate,
            self.render_observation.resolved(),
        )
        .map_err(|_| RendererAdmittedPendingErrorV1::Admission)?
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
    let roots = projection
        .direct_roots()
        .iter()
        .map(|root| direct_root_candidate_v1(projection, root))
        .collect::<Result<Vec<_>, RendererAdmittedPendingErrorV1>>()?;
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

fn direct_root_candidate_v1(
    projection: &ferrum_document_projection::DocumentProjectionV1,
    direct_root: &DocumentDirectRootV1,
) -> Result<CompleteRenderRootCandidateV1, RendererAdmittedPendingErrorV1> {
    let identity = durable_identity_v1(Some(direct_root.document_object_id()))?;
    let lowering = match direct_root.kind() {
        DocumentDirectRootKindV1::Molecule => projection
            .molecules()
            .iter()
            .any(|molecule| molecule.id() == Some(direct_root.document_object_id()))
            .then_some(CompleteRenderRootLoweringV1::Visual(
                CompleteRenderPrimitiveV1::Molecule,
            ))
            .ok_or(RendererAdmittedPendingErrorV1::Admission)?,
        DocumentDirectRootKindV1::Presentation(kind) => projection
            .presentation_stack()
            .entries()
            .iter()
            .find(|entry| {
                entry.root().target().document_object_id() == direct_root.document_object_id()
            })
            .and_then(|entry| presentation_root_lowering_v1(entry.root(), kind))
            .ok_or(RendererAdmittedPendingErrorV1::Admission)?,
        DocumentDirectRootKindV1::RejectedPresentation(code) => projection
            .presentation_stack()
            .issues()
            .iter()
            .any(|issue| {
                issue.target().document_object_id() == direct_root.document_object_id()
                    && issue.code() == code
            })
            .then_some(CompleteRenderRootLoweringV1::MissingRequiredPrimitive)
            .ok_or(RendererAdmittedPendingErrorV1::Admission)?,
    };
    Ok(CompleteRenderRootCandidateV1::new(
        identity,
        direct_root.paint_order(),
        lowering,
    ))
}

fn presentation_root_lowering_v1(
    root: &PresentationRootProjectionV1,
    expected_kind: PresentationRecordKindV1,
) -> Option<CompleteRenderRootLoweringV1> {
    let (kind, lowering) = match root {
        PresentationRootProjectionV1::Plus { .. } => (
            PresentationRecordKindV1::Plus,
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Text),
        ),
        PresentationRootProjectionV1::Text { .. } => (
            PresentationRecordKindV1::Text,
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Text),
        ),
        PresentationRootProjectionV1::Arrow { .. } => (
            PresentationRecordKindV1::Arrow,
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Vector),
        ),
        PresentationRootProjectionV1::Polyline { .. } => (
            PresentationRecordKindV1::Polyline,
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Vector),
        ),
        PresentationRootProjectionV1::Wavy { .. } => (
            PresentationRecordKindV1::Polyline,
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Vector),
        ),
        PresentationRootProjectionV1::RoundBracket { .. } => (
            PresentationRecordKindV1::Polyline,
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Vector),
        ),
        PresentationRootProjectionV1::Rectangle { .. } => (
            PresentationRecordKindV1::Rectangle,
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Vector),
        ),
        PresentationRootProjectionV1::Square { .. } => (
            PresentationRecordKindV1::Square,
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Vector),
        ),
        PresentationRootProjectionV1::Oval { .. } => (
            PresentationRecordKindV1::Oval,
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Vector),
        ),
        PresentationRootProjectionV1::Circle { .. } => (
            PresentationRecordKindV1::Circle,
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Vector),
        ),
        PresentationRootProjectionV1::Polygon { .. } => (
            PresentationRecordKindV1::Polygon,
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Vector),
        ),
    };
    (kind == expected_kind).then_some(lowering)
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

    #[test]
    fn candidate_preserves_authoritative_mixed_direct_root_positions_and_gaps() {
        let source = concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule-first\">",
            "<atom id=\"atom-first\" name=\"C\"><point x=\"0\" y=\"0\"/>",
            "</atom></molecule><plus id=\"label\"><point x=\"1\" y=\"2\"/>",
            "</plus><polygon id=\"rejected\"/><standard line_color=\"#123456\"/>",
            "<molecule id=\"molecule-last\"><atom id=\"atom-last\" name=\"O\">",
            "<point x=\"3\" y=\"4\"/></atom></molecule></cdml>",
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
            observation
                .projection()
                .direct_roots()
                .iter()
                .map(DocumentDirectRootV1::paint_order)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 4]
        );
        assert_eq!(
            candidate
                .roots()
                .iter()
                .map(CompleteRenderRootCandidateV1::paint_order)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 4]
        );
        assert_eq!(
            candidate
                .roots()
                .iter()
                .map(CompleteRenderRootCandidateV1::lowering)
                .collect::<Vec<_>>(),
            vec![
                CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Molecule),
                CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Text),
                CompleteRenderRootLoweringV1::MissingRequiredPrimitive,
                CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Molecule),
            ]
        );
    }
}
