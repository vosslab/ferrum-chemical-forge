use ferrum_document::DocumentSession;
use ferrum_render::SvgOutputBudgetV1;

use crate::{
    DocumentSelectionSvgErrorV1, DocumentSvgSelectionV1, render_document_selection_to_svg_v1,
};

#[test]
fn selected_objects_render_complete_roots_with_content_fitted_provenance() {
    let session = DocumentSession::load(concat!(
        "<cdml><plus id=\"p\"><point x=\"40\" y=\"20\"/></plus>",
        "<molecule id=\"near\"><atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom>",
        "<atom id=\"b\" name=\"O\"><point x=\"25\" y=\"20\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/></molecule>",
        "<molecule id=\"far\"><atom id=\"z\" name=\"N\"><point x=\"300\" y=\"20\"/>",
        "</atom></molecule></cdml>"
    ))
    .expect("session");
    let observation = session.observe(0).expect("observation");
    let plus = observation.projection().presentation_stack().roots()[0]
        .target()
        .id()
        .expect("plus ID")
        .clone();
    let atom = observation.projection().molecules()[0].atoms()[0]
        .id()
        .expect("atom ID")
        .clone();
    let before = session.snapshot().expect("before snapshot");

    let receipt = render_document_selection_to_svg_v1(
        &observation,
        DocumentSvgSelectionV1::new(vec![atom.clone(), plus.clone()]).expect("selection"),
        SvgOutputBudgetV1::new(64 * 1024).expect("budget"),
    )
    .expect("selected SVG");

    assert_eq!(receipt.selected_objects(), &[plus, atom]);
    assert_eq!(receipt.selected_roots().len(), 2);
    assert!(
        receipt.viewport().x() + receipt.viewport().width() < 100.0
            && receipt.svg().as_str().starts_with("<svg ")
            && session.snapshot().expect("after snapshot") == before
    );
}

#[test]
fn selected_profile_exclusion_withholds_the_svg_artifact() {
    let session = DocumentSession::load(
        "<cdml><text id=\"unsupported\"><point x=\"10\" y=\"20\"/><font family=\"Arial\"/><ftext>label</ftext></text></cdml>",
    )
    .expect("session");
    let observation = session.observe(0).expect("observation");
    let selected = observation.projection().presentation_stack().roots()[0]
        .target()
        .id()
        .expect("Text ID")
        .clone();

    let error = render_document_selection_to_svg_v1(
        &observation,
        DocumentSvgSelectionV1::new(vec![selected]).expect("selection"),
        SvgOutputBudgetV1::new(64 * 1024).expect("budget"),
    )
    .expect_err("excluded selected root");

    assert!(matches!(
        error,
        DocumentSelectionSvgErrorV1::SelectedRootExcluded
    ));
}
