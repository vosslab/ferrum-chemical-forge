#[test]
fn document_public_lib_has_no_vector_gesture_export() {
    let public_surface = include_str!("../src/lib.rs");
    assert!(
        !public_surface.contains("presentation_vector_gesture"),
        "vector gesture authority belongs to ferrum-document-render"
    );
    assert!(
        !public_surface.contains("PreparedPresentationVector"),
        "document must not export a vector pending receipt"
    );
}

#[test]
fn document_public_surface_has_no_reaction_authoring_authority() {
    let public_surface = include_str!("../src/lib.rs");
    for forbidden in [
        "ReactionCandidateV1",
        "ReactionCreateRequestV1",
        "prepare_reaction_candidate_v1",
        "commit_renderer_admitted_reaction_candidate_v1",
    ] {
        assert!(
            !public_surface.contains(forbidden),
            "reaction authoring belongs to ferrum-document-render: {forbidden}"
        );
    }
}
