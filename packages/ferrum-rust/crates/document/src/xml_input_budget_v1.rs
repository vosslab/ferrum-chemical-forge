//! Explicit, caller-owned admission limits for untrusted XML source text.

use thiserror::Error;
use xmlparser::{ElementEnd, Token, Tokenizer};

/// Resource limits an XML ingress explicitly chooses before retaining a tree.
///
/// All quantities are UTF-8 bytes or token counts from the supplied source. This
/// crate deliberately provides no production default; policy belongs to the ingress
/// that knows its supported document population and available resources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XmlInputBudgetV1 {
    /// Maximum source length in UTF-8 bytes.
    pub max_utf8_bytes: usize,
    /// Maximum number of element start tags.
    pub max_elements: usize,
    /// Maximum simultaneous element nesting depth, including the root element.
    pub max_depth: usize,
    /// Maximum total XML attributes, including namespace declarations.
    pub max_attributes: usize,
    /// Maximum aggregate lexical UTF-8 bytes in `xmlparser` text and CDATA spans.
    ///
    /// This counts source slices, not decoded or retained character bytes. For
    /// example, predefined and numeric entity spellings consume their source-byte
    /// length here; the independently enforced whole-source limit also bounds them.
    pub max_text_bytes: usize,
}

/// Complete tokenizer accounting for one decoded XML source.
///
/// These values use exactly the same token meanings as [`XmlInputBudgetV1`].
/// Measurement deliberately selects no admission policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XmlInputMeasurementV1 {
    /// Source length in UTF-8 bytes.
    pub utf8_bytes: usize,
    /// Number of element-start tokens.
    pub elements: usize,
    /// Greatest simultaneous open-element count, including the root.
    pub max_depth: usize,
    /// Number of attribute tokens, including namespace declarations.
    pub attributes: usize,
    /// Aggregate lexical UTF-8 source bytes in text and CDATA tokens.
    pub lexical_text_utf8_bytes: usize,
}

/// One dimension of a caller-supplied XML resource budget was exceeded.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum XmlBudgetError {
    /// The decoded UTF-8 input was too large.
    #[error("XML input is {actual} UTF-8 bytes, exceeding the {limit}-byte limit")]
    Utf8Bytes {
        /// Configured maximum source bytes.
        limit: usize,
        /// Observed source bytes.
        actual: usize,
    },
    /// The input contained too many elements.
    #[error("XML input has {actual} elements, exceeding the {limit}-element limit")]
    Elements {
        /// Configured maximum element count.
        limit: usize,
        /// Observed element count.
        actual: usize,
    },
    /// The input nested elements too deeply.
    #[error("XML input has nesting depth {actual}, exceeding the {limit}-level limit")]
    Depth {
        /// Configured maximum nesting depth.
        limit: usize,
        /// Observed nesting depth.
        actual: usize,
    },
    /// The input contained too many attributes.
    #[error("XML input has {actual} attributes, exceeding the {limit}-attribute limit")]
    Attributes {
        /// Configured maximum attribute count.
        limit: usize,
        /// Observed attribute count.
        actual: usize,
    },
    /// The input had too many lexical text or CDATA UTF-8 source bytes.
    #[error("XML input has {actual} text UTF-8 bytes, exceeding the {limit}-byte limit")]
    TextBytes {
        /// Configured maximum aggregate text bytes.
        limit: usize,
        /// Observed aggregate text bytes.
        actual: usize,
    },
}

/// XML admission failure before or during opaque tree retention.
#[derive(Debug, Error)]
pub enum XmlInputError {
    /// A caller-owned resource limit rejected the source before tree allocation.
    #[error(transparent)]
    Budget(#[from] XmlBudgetError),
    /// DTD declarations and their entity declarations are never accepted.
    #[error("XML DTD declarations are not supported")]
    DtdForbidden,
    /// The non-retaining XML tokenizer rejected malformed UTF-8 XML.
    #[error("XML preflight parse error: {0}")]
    Preflight(#[source] xmlparser::Error),
    /// The retained-tree parser rejected malformed or unsupported XML.
    #[error("XML parse error: {0}")]
    Xml(#[source] xot::ParseError),
}

pub(crate) fn preflight(source: &str, budget: XmlInputBudgetV1) -> Result<(), XmlInputError> {
    scan(source, Some(budget)).map(|_| ())
}

/// Measure one decoded XML source with the hardened tokenizer policy.
///
/// DTD declarations and tokenizer errors remain typed failures. No partial measurement is
/// returned for malformed input.
pub fn measure_xml_input_v1(source: &str) -> Result<XmlInputMeasurementV1, XmlInputError> {
    scan(source, None)
}

fn scan(
    source: &str,
    admission: Option<XmlInputBudgetV1>,
) -> Result<XmlInputMeasurementV1, XmlInputError> {
    if let Some(budget) = admission {
        enforce(source.len(), budget.max_utf8_bytes, |limit, actual| {
            XmlBudgetError::Utf8Bytes { limit, actual }
        })?;
    }

    let mut elements = 0_usize;
    let mut depth = 0_usize;
    let mut max_depth = 0_usize;
    let mut attributes = 0_usize;
    let mut text_bytes = 0_usize;
    for token in Tokenizer::from(source) {
        let token = token.map_err(XmlInputError::Preflight)?;
        match token {
            Token::DtdStart { .. }
            | Token::EmptyDtd { .. }
            | Token::EntityDeclaration { .. }
            | Token::DtdEnd { .. } => return Err(XmlInputError::DtdForbidden),
            Token::ElementStart { .. } => {
                elements = elements.saturating_add(1);
                if let Some(budget) = admission {
                    enforce(elements, budget.max_elements, |limit, actual| {
                        XmlBudgetError::Elements { limit, actual }
                    })?;
                }
                depth = depth.saturating_add(1);
                max_depth = max_depth.max(depth);
                if let Some(budget) = admission {
                    enforce(depth, budget.max_depth, |limit, actual| {
                        XmlBudgetError::Depth { limit, actual }
                    })?;
                }
            }
            Token::Attribute { .. } => {
                attributes = attributes.saturating_add(1);
                if let Some(budget) = admission {
                    enforce(attributes, budget.max_attributes, |limit, actual| {
                        XmlBudgetError::Attributes { limit, actual }
                    })?;
                }
            }
            Token::Text { text } | Token::Cdata { text, .. } => {
                text_bytes = text_bytes.saturating_add(text.as_str().len());
                if let Some(budget) = admission {
                    enforce(text_bytes, budget.max_text_bytes, |limit, actual| {
                        XmlBudgetError::TextBytes { limit, actual }
                    })?;
                }
            }
            Token::ElementEnd { end, .. } => match end {
                ElementEnd::Empty | ElementEnd::Close(_, _) => {
                    depth = depth.saturating_sub(1);
                }
                ElementEnd::Open => {}
            },
            Token::Declaration { .. }
            | Token::ProcessingInstruction { .. }
            | Token::Comment { .. } => {}
        }
    }
    Ok(XmlInputMeasurementV1 {
        utf8_bytes: source.len(),
        elements,
        max_depth,
        attributes,
        lexical_text_utf8_bytes: text_bytes,
    })
}

fn enforce(
    actual: usize,
    limit: usize,
    error: impl FnOnce(usize, usize) -> XmlBudgetError,
) -> Result<(), XmlInputError> {
    if actual > limit {
        return Err(error(limit, actual).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{XmlBudgetError, XmlInputBudgetV1, XmlInputError, measure_xml_input_v1};
    use crate::XmlDocument;

    fn budget() -> XmlInputBudgetV1 {
        XmlInputBudgetV1 {
            max_utf8_bytes: 256,
            max_elements: 8,
            max_depth: 4,
            max_attributes: 8,
            max_text_bytes: 64,
        }
    }

    fn assert_budget_error(source: &str, budget: XmlInputBudgetV1, expected: XmlBudgetError) {
        let error = XmlDocument::parse_with_budget(source, budget)
            .expect_err("over-budget XML must not create a retained document");
        assert!(matches!(error, XmlInputError::Budget(actual) if actual == expected));
    }

    #[test]
    fn admits_each_dimension_at_its_configured_limit() {
        let input = "<a one=\"1\" two=\"2\"><b>1234</b><c>5678</c></a>";
        let exact = XmlInputBudgetV1 {
            max_utf8_bytes: input.len(),
            max_elements: 3,
            max_depth: 2,
            max_attributes: 2,
            max_text_bytes: 8,
        };
        let _document = XmlDocument::parse_with_budget(input, exact)
            .expect("input at every configured limit must parse");
    }

    #[test]
    fn complete_measurement_matches_an_exact_admission_budget() {
        let input = "<a one=\"1\"><b><![CDATA[é]]></b>text</a>";
        let measured = measure_xml_input_v1(input).expect("well-formed XML measures");
        let exact = XmlInputBudgetV1 {
            max_utf8_bytes: measured.utf8_bytes,
            max_elements: measured.elements,
            max_depth: measured.max_depth,
            max_attributes: measured.attributes,
            max_text_bytes: measured.lexical_text_utf8_bytes,
        };
        XmlDocument::parse_with_budget(input, exact)
            .expect("the complete measurement admits every exact dimension");
    }

    #[test]
    fn measurement_preserves_dtd_and_malformed_failure_classes() {
        assert!(matches!(
            measure_xml_input_v1("<!DOCTYPE a><a/>"),
            Err(XmlInputError::DtdForbidden)
        ));
        assert!(matches!(
            measure_xml_input_v1("<a><"),
            Err(XmlInputError::Preflight(_))
        ));
    }

    #[test]
    fn rejects_one_utf8_byte_over_the_limit_before_tree_retention() {
        let input = "<a/>";
        let mut exact = budget();
        exact.max_utf8_bytes = input.len() - 1;
        assert_budget_error(
            input,
            exact,
            XmlBudgetError::Utf8Bytes {
                limit: input.len() - 1,
                actual: input.len(),
            },
        );
    }

    #[test]
    fn rejects_one_element_over_the_limit_before_tree_retention() {
        let mut exact = budget();
        exact.max_elements = 2;
        assert_budget_error(
            "<a><b/><c/></a>",
            exact,
            XmlBudgetError::Elements {
                limit: 2,
                actual: 3,
            },
        );
    }

    #[test]
    fn rejects_one_depth_level_over_the_limit_before_tree_retention() {
        let mut exact = budget();
        exact.max_depth = 2;
        assert_budget_error(
            "<a><b><c/></b></a>",
            exact,
            XmlBudgetError::Depth {
                limit: 2,
                actual: 3,
            },
        );
    }

    #[test]
    fn rejects_one_attribute_over_the_limit_before_tree_retention() {
        let mut exact = budget();
        exact.max_attributes = 2;
        assert_budget_error(
            "<a one=\"1\" two=\"2\" three=\"3\"/>",
            exact,
            XmlBudgetError::Attributes {
                limit: 2,
                actual: 3,
            },
        );
    }

    #[test]
    fn rejects_one_text_byte_over_the_limit_before_tree_retention() {
        let mut exact = budget();
        exact.max_text_bytes = 4;
        assert_budget_error(
            "<a>12345</a>",
            exact,
            XmlBudgetError::TextBytes {
                limit: 4,
                actual: 5,
            },
        );
    }

    #[test]
    fn counts_multibyte_text_as_utf8_source_bytes() {
        let mut exact = budget();
        exact.max_text_bytes = 1;
        assert_budget_error(
            "<a>é</a>",
            exact,
            XmlBudgetError::TextBytes {
                limit: 1,
                actual: 2,
            },
        );
    }

    #[test]
    fn counts_cdata_content_as_lexical_text_bytes() {
        let mut exact = budget();
        exact.max_text_bytes = 1;
        assert_budget_error(
            "<a><![CDATA[é]]></a>",
            exact,
            XmlBudgetError::TextBytes {
                limit: 1,
                actual: 2,
            },
        );
    }

    #[test]
    fn counts_namespace_declarations_as_attributes() {
        let mut exact = budget();
        exact.max_attributes = 1;
        assert_budget_error(
            "<a xmlns=\"urn:default\" xmlns:vendor=\"urn:vendor\"/>",
            exact,
            XmlBudgetError::Attributes {
                limit: 1,
                actual: 2,
            },
        );
    }

    #[test]
    fn rejects_dtd_and_entity_declarations_before_tree_retention() {
        let external = "<!DOCTYPE a SYSTEM \"https://example.invalid/a.dtd\"><a/>";
        let entity = "<!DOCTYPE a [<!ENTITY x \"expanded\">]><a>&x;</a>";
        for input in [external, entity] {
            let error = XmlDocument::parse_with_budget(input, budget())
                .expect_err("DTD-bearing XML must not create a retained document");
            assert!(matches!(error, XmlInputError::DtdForbidden));
        }
    }

    #[test]
    fn reports_malformed_xml_as_a_typed_preflight_failure() {
        let error = XmlDocument::parse_with_budget("<a><b></a>", budget())
            .expect_err("malformed XML must not create a retained document");
        assert!(matches!(
            error,
            XmlInputError::Xml(_) | XmlInputError::Preflight(_)
        ));
    }

    proptest::proptest! {
        #[test]
        fn property_checks_element_boundaries(child_count in 0_usize..6) {
            let children = "<b/>".repeat(child_count);
            let input = format!("<a>{children}</a>");
            let element_count = child_count + 1;
            let mut exact = budget();
            exact.max_elements = element_count;
            let _document = XmlDocument::parse_with_budget(&input, exact)
                .expect("generated XML at its element limit must parse");

            if child_count > 0 {
                exact.max_elements = element_count - 1;
                assert_budget_error(
                    &input,
                    exact,
                    XmlBudgetError::Elements {
                        limit: element_count - 1,
                        actual: element_count,
                    },
                );
            }
        }
    }
}
