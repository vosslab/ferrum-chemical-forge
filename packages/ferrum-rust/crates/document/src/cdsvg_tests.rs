use xmlparser::{ElementEnd, Token, Tokenizer};

use super::{
    CdsvgExtractionError, TypedClass, TypedDocument, XmlBudgetError, XmlInputBudgetV1,
    XmlInputError, extract_cdml_from_svg, extract_cdml_from_svg_with_budget,
    measure_cdsvg_input_v1,
};

const CDML_NAMESPACE: &str = "urn:ferrum:cdml";

#[test]
fn extracts_the_canonical_cdml_payload_and_discards_rendered_svg() {
    let source = format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="10" height="10"/><g>"#,
            r#"<cdml xmlns="{CDML_NAMESPACE}" version="26.08"><paper type="A4"/>"#,
            r#"<arrow id="a1" type="normal"/></cdml></g></svg>"#,
        ),
        CDML_NAMESPACE = CDML_NAMESPACE,
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
fn measurement_keeps_original_wrapper_and_normalized_payload_separate() {
    let source = format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><!-- wrapper -->"#,
            r#"<cdml xmlns="{CDML_NAMESPACE}" version="26.08"><paper type="A4"/>"#,
            r#"</cdml></svg>"#,
        ),
        CDML_NAMESPACE = CDML_NAMESPACE,
    );
    let measured = measure_cdsvg_input_v1(&source).expect("typed-valid canonical CD-SVG measures");
    assert_eq!(measured.wrapper.utf8_bytes, source.len());
    assert!(measured.normalized_payload.utf8_bytes < measured.wrapper.utf8_bytes);
    assert_eq!(measured.normalized_payload.elements, 2);
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
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><cdml xmlns="{CDML_NAMESPACE}"/>"#,
            r#"<g><cdml xmlns="{CDML_NAMESPACE}"/></g></svg>"#,
        ),
        CDML_NAMESPACE = CDML_NAMESPACE,
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

#[test]
fn budgeted_extraction_preserves_canonical_missing_and_duplicate_rules() {
    let missing = r#"<svg xmlns="http://www.w3.org/2000/svg"><cdml><paper/></cdml></svg>"#;
    let duplicate = format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><cdml xmlns="{CDML_NAMESPACE}"/>"#,
            r#"<cdml xmlns="{CDML_NAMESPACE}"/></svg>"#,
        ),
        CDML_NAMESPACE = CDML_NAMESPACE,
    );

    for (source, expected_count) in [(missing, None), (duplicate.as_str(), Some(2))] {
        let error = extract_cdml_from_svg_with_budget(
            source,
            exact_budget(source),
            unconstrained_test_budget(),
        )
        .expect_err("canonical payload selection must occur after wrapper admission");
        assert!(
            matches!(
                error,
                CdsvgExtractionError::MissingCdmlPayload if expected_count.is_none()
            ) || matches!(
                error,
                CdsvgExtractionError::MultipleCdmlPayload { count } if expected_count == Some(count)
            )
        );
    }
}

#[test]
fn budgeted_extraction_accepts_exact_wrapper_and_payload_boundaries() {
    let source = canonical_cdsvg("<paper type=\"A4\"/>");
    let payload = extract_cdml_from_svg(&source)
        .expect("unbudgeted characterization extraction must succeed")
        .to_xml()
        .expect("selected payload must serialize");

    let document =
        extract_cdml_from_svg_with_budget(&source, exact_budget(&source), exact_budget(&payload))
            .expect("input exactly at both caller-selected limits must extract");
    assert_eq!(document.root().class(), TypedClass::Cdml);
}

#[test]
fn budgeted_extraction_measures_original_wrapper_and_normalized_payload_separately() {
    let source = format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><legacy:cdml "#,
            r#"xmlns:legacy="{CDML_NAMESPACE}"><foreign z="last" a="first">"#,
            r#"<![CDATA[alpha]]>&#x41;&amp;</foreign></legacy:cdml></svg>"#,
        ),
        CDML_NAMESPACE = CDML_NAMESPACE,
    );
    let serialized = extract_cdml_from_svg(&source)
        .expect("normalizing source must extract")
        .to_xml()
        .expect("payload must serialize");
    assert!(!serialized.contains("CDATA"));
    assert!(!serialized.contains("&#x41;"));
    assert!(serialized.contains("alphaA&amp;"));

    let wrapper_error = extract_cdml_from_svg_with_budget(
        &source,
        XmlInputBudgetV1 {
            max_utf8_bytes: source.len() - 1,
            ..unconstrained_test_budget()
        },
        unconstrained_test_budget(),
    )
    .expect_err("wrapper policy must measure the supplied lexical SVG source");
    assert!(matches!(
        wrapper_error,
        CdsvgExtractionError::WrapperInput(XmlInputError::Budget(XmlBudgetError::Utf8Bytes {
            limit,
            actual
        })) if limit == source.len() - 1 && actual == source.len()
    ));

    let document = extract_cdml_from_svg_with_budget(
        &source,
        exact_budget(&source),
        exact_budget(&serialized),
    )
    .expect("payload policy must measure the selected structural serialization");
    assert_eq!(
        document.to_xml().expect("payload must serialize"),
        serialized
    );
}

#[test]
fn budgeted_extraction_rejects_each_wrapper_dimension_before_payload_admission() {
    let source = canonical_cdsvg("<paper/>");
    let cases = [
        (
            XmlInputBudgetV1 {
                max_utf8_bytes: source.len() - 1,
                ..unconstrained_test_budget()
            },
            XmlBudgetError::Utf8Bytes {
                limit: source.len() - 1,
                actual: source.len(),
            },
        ),
        (
            XmlInputBudgetV1 {
                max_elements: 2,
                ..unconstrained_test_budget()
            },
            XmlBudgetError::Elements {
                limit: 2,
                actual: 3,
            },
        ),
        (
            XmlInputBudgetV1 {
                max_depth: 2,
                ..unconstrained_test_budget()
            },
            XmlBudgetError::Depth {
                limit: 2,
                actual: 3,
            },
        ),
        (
            XmlInputBudgetV1 {
                max_attributes: 1,
                ..unconstrained_test_budget()
            },
            XmlBudgetError::Attributes {
                limit: 1,
                actual: 2,
            },
        ),
        (
            XmlInputBudgetV1 {
                max_text_bytes: 0,
                ..unconstrained_test_budget()
            },
            XmlBudgetError::TextBytes {
                limit: 0,
                actual: 1,
            },
        ),
    ];
    let source_with_text = canonical_cdsvg("<paper/>x");

    for (index, (wrapper_budget, expected)) in cases.into_iter().enumerate() {
        let candidate = if index == 4 {
            &source_with_text
        } else {
            &source
        };
        assert_wrapper_budget_error(candidate, wrapper_budget, expected);
    }
}

#[test]
fn budgeted_extraction_rejects_a_deep_svg_wrapper_outside_the_cdml_payload() {
    let source = canonical_cdsvg("<g><g><g><g/></g></g></g><paper/>");
    let error = extract_cdml_from_svg_with_budget(
        &source,
        XmlInputBudgetV1 {
            max_depth: 3,
            ..unconstrained_test_budget()
        },
        unconstrained_test_budget(),
    )
    .expect_err("disposable SVG depth must not bypass the wrapper budget");

    assert!(matches!(
        error,
        CdsvgExtractionError::WrapperInput(XmlInputError::Budget(XmlBudgetError::Depth {
            limit: 3,
            actual: 4
        }))
    ));
}

#[test]
fn budgeted_extraction_rejects_each_payload_dimension_after_wrapper_admission() {
    for (payload_body, constrained_budget) in [
        ("<paper/>", PayloadBudgetConstraint::Utf8Bytes),
        (
            "<paper/><arrow id=\"a1\" type=\"normal\"/>",
            PayloadBudgetConstraint::Elements,
        ),
        (
            "<foreign><inner/></foreign>",
            PayloadBudgetConstraint::Depth,
        ),
        ("<paper type=\"A4\"/>", PayloadBudgetConstraint::Attributes),
        ("<foreign>x</foreign>", PayloadBudgetConstraint::TextBytes),
    ] {
        let source = canonical_cdsvg(payload_body);
        let payload = extract_cdml_from_svg(&source)
            .expect("unbudgeted characterization extraction must succeed")
            .to_xml()
            .expect("selected payload must serialize");
        let exact = exact_budget(&payload);
        let (budget, expected) = constrained_budget.one_less_than(exact);
        let error = extract_cdml_from_svg_with_budget(&source, exact_budget(&source), budget)
            .expect_err("payload over one independently selected limit must not admit CDML");
        assert!(matches!(
            error,
            CdsvgExtractionError::PayloadInput(XmlInputError::Budget(actual)) if actual == expected
        ));
    }
}

#[test]
fn budgeted_extraction_rejects_dtds_and_custom_entities_in_the_cdsvg_document() {
    let dtd = format!(
        concat!(
            r#"<!DOCTYPE svg SYSTEM "https://example.invalid/drawing.dtd">"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><cdml "#,
            r#"xmlns="{CDML_NAMESPACE}"/></svg>"#,
        ),
        CDML_NAMESPACE = CDML_NAMESPACE,
    );
    let custom_entity_in_payload = format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><cdml "#,
            r#"xmlns="{CDML_NAMESPACE}"><foreign>&unsafe;</foreign></cdml></svg>"#,
        ),
        CDML_NAMESPACE = CDML_NAMESPACE,
    );

    for source in [dtd, custom_entity_in_payload] {
        let error = extract_cdml_from_svg_with_budget(
            &source,
            unconstrained_test_budget(),
            unconstrained_test_budget(),
        )
        .expect_err("DTD and custom entity XML must not yield a document");
        assert!(matches!(
            error,
            CdsvgExtractionError::WrapperInput(XmlInputError::DtdForbidden)
                | CdsvgExtractionError::WrapperInput(XmlInputError::Xml(_))
                | CdsvgExtractionError::WrapperInput(XmlInputError::Preflight(_))
        ));
    }
}

#[test]
fn budgeted_extraction_keeps_wrapper_and_payload_failures_distinct() {
    let malformed_wrapper = "<svg xmlns=\"http://www.w3.org/2000/svg\">";
    let malformed_payload = format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><cdml "#,
            r#"xmlns="{CDML_NAMESPACE}"><foreign></cdml></svg>"#,
        ),
        CDML_NAMESPACE = CDML_NAMESPACE,
    );
    for source in [malformed_wrapper, malformed_payload.as_str()] {
        let wrapper_error = extract_cdml_from_svg_with_budget(
            source,
            unconstrained_test_budget(),
            unconstrained_test_budget(),
        )
        .expect_err("malformed CD-SVG cannot publish a payload");
        assert!(matches!(
            wrapper_error,
            CdsvgExtractionError::WrapperInput(XmlInputError::Preflight(_))
                | CdsvgExtractionError::WrapperInput(XmlInputError::Xml(_))
        ));
    }

    let source = canonical_cdsvg("<paper/>");
    let payload_error = extract_cdml_from_svg_with_budget(
        &source,
        exact_budget(&source),
        XmlInputBudgetV1 {
            max_elements: 0,
            ..unconstrained_test_budget()
        },
    )
    .expect_err("selected payload must still pass its own admission boundary");
    assert!(matches!(
        payload_error,
        CdsvgExtractionError::PayloadInput(XmlInputError::Budget(XmlBudgetError::Elements {
            limit: 0,
            actual: 1
        }))
    ));
}

#[derive(Clone, Copy)]
enum PayloadBudgetConstraint {
    Utf8Bytes,
    Elements,
    Depth,
    Attributes,
    TextBytes,
}

impl PayloadBudgetConstraint {
    fn one_less_than(self, exact: XmlInputBudgetV1) -> (XmlInputBudgetV1, XmlBudgetError) {
        match self {
            Self::Utf8Bytes => (
                XmlInputBudgetV1 {
                    max_utf8_bytes: exact.max_utf8_bytes - 1,
                    ..exact
                },
                XmlBudgetError::Utf8Bytes {
                    limit: exact.max_utf8_bytes - 1,
                    actual: exact.max_utf8_bytes,
                },
            ),
            Self::Elements => (
                XmlInputBudgetV1 {
                    max_elements: exact.max_elements - 1,
                    ..exact
                },
                XmlBudgetError::Elements {
                    limit: exact.max_elements - 1,
                    actual: exact.max_elements,
                },
            ),
            Self::Depth => (
                XmlInputBudgetV1 {
                    max_depth: exact.max_depth - 1,
                    ..exact
                },
                XmlBudgetError::Depth {
                    limit: exact.max_depth - 1,
                    actual: exact.max_depth,
                },
            ),
            Self::Attributes => (
                XmlInputBudgetV1 {
                    max_attributes: exact.max_attributes - 1,
                    ..exact
                },
                XmlBudgetError::Attributes {
                    limit: exact.max_attributes - 1,
                    actual: exact.max_attributes,
                },
            ),
            Self::TextBytes => (
                XmlInputBudgetV1 {
                    max_text_bytes: exact.max_text_bytes - 1,
                    ..exact
                },
                XmlBudgetError::TextBytes {
                    limit: exact.max_text_bytes - 1,
                    actual: exact.max_text_bytes,
                },
            ),
        }
    }
}

fn canonical_cdsvg(payload: &str) -> String {
    format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><cdml "#,
            r#"xmlns="{CDML_NAMESPACE}">{payload}</cdml></svg>"#,
        ),
        CDML_NAMESPACE = CDML_NAMESPACE,
        payload = payload
    )
}

fn unconstrained_test_budget() -> XmlInputBudgetV1 {
    XmlInputBudgetV1 {
        max_utf8_bytes: usize::MAX,
        max_elements: usize::MAX,
        max_depth: usize::MAX,
        max_attributes: usize::MAX,
        max_text_bytes: usize::MAX,
    }
}

fn assert_wrapper_budget_error(
    source: &str,
    wrapper_budget: XmlInputBudgetV1,
    expected: XmlBudgetError,
) {
    let error =
        extract_cdml_from_svg_with_budget(source, wrapper_budget, unconstrained_test_budget())
            .expect_err("over-budget wrapper must not retain or publish CDML");
    assert!(matches!(
        error,
        CdsvgExtractionError::WrapperInput(XmlInputError::Budget(actual)) if actual == expected
    ));
}

fn exact_budget(source: &str) -> XmlInputBudgetV1 {
    let mut elements = 0_usize;
    let mut depth = 0_usize;
    let mut maximum_depth = 0_usize;
    let mut attributes = 0_usize;
    let mut text_bytes = 0_usize;
    for token in Tokenizer::from(source) {
        let token = token.expect("test XML must tokenize");
        match token {
            Token::ElementStart { .. } => {
                elements += 1;
                depth += 1;
                maximum_depth = maximum_depth.max(depth);
            }
            Token::Attribute { .. } => attributes += 1,
            Token::Text { text } | Token::Cdata { text, .. } => text_bytes += text.as_str().len(),
            Token::ElementEnd { end, .. } => match end {
                ElementEnd::Empty | ElementEnd::Close(_, _) => depth -= 1,
                ElementEnd::Open => {}
            },
            Token::Declaration { .. }
            | Token::ProcessingInstruction { .. }
            | Token::Comment { .. }
            | Token::DtdStart { .. }
            | Token::EmptyDtd { .. }
            | Token::EntityDeclaration { .. }
            | Token::DtdEnd { .. } => {}
        }
    }
    XmlInputBudgetV1 {
        max_utf8_bytes: source.len(),
        max_elements: elements,
        max_depth: maximum_depth,
        max_attributes: attributes,
        max_text_bytes: text_bytes,
    }
}
