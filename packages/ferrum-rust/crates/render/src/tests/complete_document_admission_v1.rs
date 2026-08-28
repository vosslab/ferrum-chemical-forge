use crate::{
    AcceptedRenderOverlayTargetKindV1, AcceptedRenderOverlayTargetV1,
    COMPLETE_DOCUMENT_RENDERER_SCHEMA_V1, classify_document_render_roots_v1,
};
use ferrum_document_projection::DocumentObjectIdV1;
use ferrum_render_contract::{
    CompleteDocumentSourceFenceV1, CompleteRenderPendingIdentityV1, CompleteRenderPrimitiveV1,
    CompleteRenderRootCandidateV1, CompleteRenderRootClassV1, CompleteRenderRootIdentityV1,
    CompleteRenderRootLoweringV1, DocumentCompleteRenderCandidateV1, RefusedRootReasonV1,
};

fn candidate_with_root(
    lowering: CompleteRenderRootLoweringV1,
) -> DocumentCompleteRenderCandidateV1 {
    DocumentCompleteRenderCandidateV1::new(
        CompleteDocumentSourceFenceV1::new(7, 3, [9; 32]),
        CompleteRenderPendingIdentityV1::new(7, 11),
        vec![CompleteRenderRootCandidateV1::new(
            CompleteRenderRootIdentityV1::new("durable-root-1").expect("identity"),
            4,
            lowering,
        )],
    )
    .expect("ordered candidate")
}

#[test]
fn complete_visual_candidate_returns_immutable_presentation_only() {
    let accepted = classify_document_render_roots_v1(&candidate_with_root(
        CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Molecule),
    ));

    assert_eq!(
        COMPLETE_DOCUMENT_RENDERER_SCHEMA_V1,
        "ferrum-complete-document-renderer-v1"
    );
    let presentation = accepted;
    assert_eq!(presentation.roots().len(), 1);
    assert_eq!(presentation.roots()[0].paint_order(), 4);
    assert_eq!(
        presentation.roots()[0].class(),
        CompleteRenderRootClassV1::VisualMolecule
    );
}

#[test]
fn missing_visual_primitive_is_recorded_for_complete_plan_delta_admission() {
    assert_eq!(
        classify_document_render_roots_v1(&candidate_with_root(
            CompleteRenderRootLoweringV1::MissingRequiredPrimitive,
        ))
        .roots()[0]
            .class(),
        CompleteRenderRootClassV1::Refused(RefusedRootReasonV1::MissingRequiredPrimitive)
    );
}

#[test]
fn v1_empty_nonvisual_policy_is_recorded_for_complete_plan_delta_admission() {
    assert_eq!(
        classify_document_render_roots_v1(&candidate_with_root(
            CompleteRenderRootLoweringV1::Nonvisual,
        ))
        .roots()[0]
            .class(),
        CompleteRenderRootClassV1::Refused(RefusedRootReasonV1::ProfileExcluded)
    );
}

#[test]
fn overlay_target_keeps_durable_identity_and_closed_kind() {
    let document_object_id = DocumentObjectIdV1::from_entropy_bytes([0x31; 16]);
    let target = AcceptedRenderOverlayTargetV1::bond(document_object_id.clone());

    assert_eq!(target.document_object_id(), &document_object_id);
    assert_eq!(target.kind(), AcceptedRenderOverlayTargetKindV1::Bond);
}
