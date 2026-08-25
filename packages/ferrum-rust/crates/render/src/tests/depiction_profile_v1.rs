use ferrum_document_projection::{
    DocumentProjectionProvenanceV1, DocumentProjectionV1, PaperAttributesV1, PaperLayoutFactsV1,
    PaperLayoutProjectionV1, PaperOrientationV1, PaperPageV1, PositiveFiniteV1,
    PresentationStackProjectionV1, ViewportAttributesV1,
};

use crate::{DepictionProfileV1, render_document_projection_v1};

fn empty_projection() -> DocumentProjectionV1 {
    let revision = 19;
    let digest = [5; 32];
    let stack =
        PresentationStackProjectionV1::new(revision, digest, Vec::new(), Vec::new(), Vec::new())
            .expect("empty presentation stack is valid");
    let page = PaperPageV1::from_resolved_dimensions(
        PositiveFiniteV1::new(210.0).expect("A4 width is positive"),
        PositiveFiniteV1::new(297.0).expect("A4 height is positive"),
        PaperOrientationV1::Portrait,
        None,
    )
    .expect("A4 dimensions have finite bounds");
    DocumentProjectionV1::try_new(
        DocumentProjectionProvenanceV1::new(revision, digest, false),
        None,
        PaperLayoutProjectionV1::new(
            revision,
            digest,
            PaperLayoutFactsV1 {
                paper_present: false,
                paper_attributes: PaperAttributesV1::default(),
                effective_paper_attributes: PaperAttributesV1::default(),
                viewport_attributes: ViewportAttributesV1::default(),
                default_type: "A4".to_owned(),
                default_orientation: PaperOrientationV1::Portrait,
                page,
            },
        ),
        Vec::new(),
        Vec::new(),
        stack,
        Vec::new(),
    )
    .expect("matching lower immutable values form one projection")
}

#[test]
fn renderer_consumes_a_real_lower_document_projection() {
    let resolution =
        render_document_projection_v1(&empty_projection(), &DepictionProfileV1::ferrum_default())
            .expect("the renderer accepts a valid lower aggregate without a document session");

    assert_eq!(resolution.projection_revision(), 19);
    assert_eq!(resolution.projection_digest(), &[5; 32]);
    assert!(resolution.plans().is_empty());
}
