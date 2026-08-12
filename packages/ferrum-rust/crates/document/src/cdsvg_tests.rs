use super::{CdsvgExtractionError, TypedClass, TypedDocument, extract_cdml_from_svg};

const CDML_NAMESPACE: &str = "http://www.freesoftware.fsf.org/bkchem/cdml";

#[test]
fn extracts_the_canonical_cdml_payload_and_discards_rendered_svg() {
    let source = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="10" height="10"/><g><cdml xmlns="{CDML_NAMESPACE}" version="26.08"><paper type="A4"/><arrow id="a1" type="normal"/></cdml></g></svg>"#
    );

    let document = extract_cdml_from_svg(&source).expect("canonical payload must extract");
    assert_eq!(document.root().class(), TypedClass::Cdml);
    assert_eq!(document.root().children_of(TypedClass::Paper).count(), 1);
    assert_eq!(
        document.root().children_of(TypedClass::CanvasArrow).count(),
        1
    );

    let serialized = document.to_xml().expect("payload must serialize");
    assert!(!serialized.contains("<svg"));
    let reparsed = TypedDocument::parse(&serialized).expect("serialized payload must reparse");
    assert_eq!(reparsed.root().attribute("version"), Some("26.08"));
}

#[test]
fn rejects_svg_without_a_canonical_cdml_payload() {
    let error = extract_cdml_from_svg(
        r#"<svg xmlns="http://www.w3.org/2000/svg"><text>presentation only</text></svg>"#,
    )
    .expect_err("missing payload must fail");

    assert!(matches!(error, CdsvgExtractionError::MissingCdmlPayload));
}

#[test]
fn rejects_svg_with_multiple_canonical_cdml_payloads() {
    let source = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg"><cdml xmlns="{CDML_NAMESPACE}"/><g><cdml xmlns="{CDML_NAMESPACE}"/></g></svg>"#
    );

    let error = extract_cdml_from_svg(&source).expect_err("duplicate payload must fail");
    assert!(matches!(
        error,
        CdsvgExtractionError::MultipleCdmlPayload { count: 2 }
    ));
}

#[test]
fn rejects_cdml_payload_without_the_canonical_namespace() {
    let error = extract_cdml_from_svg(
        r#"<svg xmlns="http://www.w3.org/2000/svg"><cdml><paper/></cdml></svg>"#,
    )
    .expect_err("legacy no-namespace payload must not be accepted in CD-SVG");

    assert!(matches!(error, CdsvgExtractionError::MissingCdmlPayload));
}

#[test]
fn rejects_non_svg_xml_before_examining_possible_payloads() {
    let source = format!(r#"<cdml xmlns="{CDML_NAMESPACE}"/>"#);
    let error = extract_cdml_from_svg(&source).expect_err("bare CDML is not CD-SVG");

    assert!(matches!(error, CdsvgExtractionError::NotSvgRoot { .. }));
}
