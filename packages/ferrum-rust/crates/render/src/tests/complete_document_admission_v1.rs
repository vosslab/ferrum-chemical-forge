use crate::{COMPLETE_DOCUMENT_RENDERER_SCHEMA_V1, admit_complete_document_render_v1};
use ferrum_render_contract::{
    CompleteDocumentSourceFenceV1, CompleteRenderAdmissionRefusalV1,
    CompleteRenderPendingIdentityV1, CompleteRenderPrimitiveV1, CompleteRenderRootCandidateV1,
    CompleteRenderRootClassV1, CompleteRenderRootIdentityV1, CompleteRenderRootLoweringV1,
    DocumentCompleteRenderCandidateV1, RefusedRootReasonV1,
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
    let accepted = admit_complete_document_render_v1(&candidate_with_root(
        CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Molecule),
    ))
    .expect("visual candidate is admitted");

    assert_eq!(
        COMPLETE_DOCUMENT_RENDERER_SCHEMA_V1,
        "ferrum-complete-document-renderer-v1"
    );
    let presentation = accepted.presentation();
    assert_eq!(presentation.roots().len(), 1);
    assert_eq!(
        presentation.roots()[0].class(),
        CompleteRenderRootClassV1::VisualMolecule
    );
}

#[test]
fn missing_visual_primitive_is_a_typed_root_refusal() {
    assert_eq!(
        admit_complete_document_render_v1(&candidate_with_root(
            CompleteRenderRootLoweringV1::MissingRequiredPrimitive,
        )),
        Err(CompleteRenderAdmissionRefusalV1::RootRefused {
            root: CompleteRenderRootIdentityV1::new("durable-root-1").expect("identity"),
            class: CompleteRenderRootClassV1::Refused(
                RefusedRootReasonV1::MissingRequiredPrimitive,
            ),
        })
    );
}

#[test]
fn v1_empty_nonvisual_policy_refuses_nonvisual_root() {
    assert_eq!(
        admit_complete_document_render_v1(&candidate_with_root(
            CompleteRenderRootLoweringV1::Nonvisual,
        )),
        Err(CompleteRenderAdmissionRefusalV1::RootRefused {
            root: CompleteRenderRootIdentityV1::new("durable-root-1").expect("identity"),
            class: CompleteRenderRootClassV1::Refused(RefusedRootReasonV1::ProfileExcluded),
        })
    );
}
