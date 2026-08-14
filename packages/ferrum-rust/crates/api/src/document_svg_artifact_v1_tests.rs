use ferrum_document::DocumentSession;
use ferrum_render::{SvgOutputBudgetV1, SvgRenderError};

use crate::{DocumentSvgArtifactErrorV1, render_document_session_to_svg_v1};

const CDML: &str = concat!(
    "<cdml><molecule id=\"m\">",
    "<atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom>",
    "</molecule></cdml>",
);

#[test]
fn exact_session_revision_produces_one_authenticated_svg_without_mutation() {
    let session = DocumentSession::load(CDML).expect("session");
    let before = session.snapshot().expect("before snapshot");

    let artifact = render_document_session_to_svg_v1(
        &session,
        before.revision(),
        SvgOutputBudgetV1::new(64 * 1024).expect("output budget"),
    )
    .expect("complete SVG");

    assert!(artifact.artifact().as_str().starts_with("<svg "));
    assert_eq!(artifact.report().provenance().revision().get(), 0);
    assert_eq!(artifact.report().provenance().digest(), *before.digest());
    assert!(artifact.report().exclusions().is_empty());
    assert_eq!(session.snapshot().expect("after snapshot"), before);
}

#[test]
fn output_refusal_returns_no_partial_artifact_and_preserves_the_session() {
    let session = DocumentSession::load(CDML).expect("session");
    let before = session.snapshot().expect("before snapshot");

    let error = render_document_session_to_svg_v1(
        &session,
        before.revision(),
        SvgOutputBudgetV1::new(1).expect("output budget"),
    )
    .expect_err("tiny output budget must refuse the completed SVG");

    assert!(matches!(
        error,
        DocumentSvgArtifactErrorV1::Render(SvgRenderError::OutputBudgetExceeded { .. })
    ));
    assert_eq!(session.snapshot().expect("after snapshot"), before);
}
