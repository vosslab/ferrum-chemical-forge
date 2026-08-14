//! Extraction of the canonical CDML payload from a decoded CD-SVG document.

use thiserror::Error;
use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, IndexedDocument, TypedDocument, TypedDocumentError, XmlDocument,
    XmlInputBudgetV1, XmlInputError, XmlInputMeasurementV1, element_name, measure_xml_input_v1,
};

const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

/// Failures while locating and validating a CDML payload in a CD-SVG container.
#[derive(Debug, Error)]
pub enum CdsvgExtractionError {
    /// The supplied SVG text was not well-formed XML.
    #[error("cannot parse CD-SVG XML: {0}")]
    Xml(#[from] xot::ParseError),

    /// A caller-supplied resource limit or XML safety policy rejected the SVG wrapper.
    #[error("cannot admit CD-SVG wrapper: {0}")]
    WrapperInput(#[source] XmlInputError),

    /// A caller-supplied resource limit or XML safety policy rejected the selected CDML payload.
    #[error("cannot admit embedded CDML payload: {0}")]
    PayloadInput(#[source] XmlInputError),

    /// The XML document root was not an SVG element in the SVG namespace.
    #[error("expected an SVG root element, found {root_name} in namespace {namespace:?}")]
    NotSvgRoot {
        /// Root element local name.
        root_name: String,
        /// Root element namespace URI.
        namespace: String,
    },

    /// The SVG container did not contain a canonical CDML payload.
    #[error("CD-SVG contains no canonical CDML payload")]
    MissingCdmlPayload,

    /// The SVG container contained more than one canonical CDML payload.
    #[error("CD-SVG contains {count} canonical CDML payloads; exactly one is required")]
    MultipleCdmlPayload {
        /// Number of canonical CDML payload candidates found.
        count: usize,
    },

    /// The retained payload could not be structurally serialized before validation.
    #[error("cannot serialize embedded CDML payload: {0}")]
    PayloadSerialization(#[source] xot::Error),

    /// The extracted payload was not a valid typed CDML document.
    #[error("embedded CDML payload is invalid: {0}")]
    Typed(#[from] TypedDocumentError),
}

/// Separate tokenizer measurements for one CD-SVG wrapper and normalized CDML payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CdsvgInputMeasurementV1 {
    /// Measurement of the original decoded SVG wrapper.
    pub wrapper: XmlInputMeasurementV1,
    /// Measurement of the structurally serialized, typed-valid CDML payload.
    pub normalized_payload: XmlInputMeasurementV1,
}

/// Measure and validate a CD-SVG wrapper and its one canonical CDML payload.
///
/// The payload metric is deliberately for structural serialization, not a lexical SVG substring.
/// A successful result therefore means the selected payload also passes current typed CDML
/// validation without choosing a production resource budget.
pub fn measure_cdsvg_input_v1(
    source: &str,
) -> Result<CdsvgInputMeasurementV1, CdsvgExtractionError> {
    let wrapper = measure_xml_input_v1(source).map_err(CdsvgExtractionError::WrapperInput)?;
    let mut tree = Xot::new();
    let document = tree.parse(source)?;
    let payload = extract_from_tree(&tree, document)?;
    let cdml = tree
        .to_string(payload)
        .map_err(CdsvgExtractionError::PayloadSerialization)?;
    let normalized_payload =
        measure_xml_input_v1(&cdml).map_err(CdsvgExtractionError::PayloadInput)?;
    TypedDocument::parse(&cdml).map_err(CdsvgExtractionError::Typed)?;
    Ok(CdsvgInputMeasurementV1 {
        wrapper,
        normalized_payload,
    })
}

/// Extract and validate the one canonical CDML document embedded in decoded CD-SVG XML.
///
/// The SVG wrapper is disposable presentation data. This API retains only the embedded
/// CDML subtree, structurally normalizes it through [`TypedDocument`], and never attempts
/// to infer editable document state from rendered SVG elements. Compressed `.svgz` input
/// is deliberately outside this decoded-text boundary.
pub fn extract_cdml_from_svg(source: &str) -> Result<TypedDocument, CdsvgExtractionError> {
    let mut tree = Xot::new();
    let document = tree.parse(source)?;
    let payload = extract_from_tree(&tree, document)?;
    let cdml = tree
        .to_string(payload)
        .map_err(CdsvgExtractionError::PayloadSerialization)?;
    TypedDocument::parse(&cdml).map_err(CdsvgExtractionError::from)
}

/// Extract one canonical CDML payload while enforcing explicit XML budgets at both boundaries.
///
/// `wrapper_budget` applies to the original decoded SVG lexical source before `xot` retains the
/// disposable wrapper tree. After exactly one canonical payload is selected, `cdml_budget` applies
/// independently to `xot`'s structural serialization of that subtree before it becomes an
/// authoritative typed CDML document. It therefore does not measure the original lexical CDML
/// substring: entity spellings, CDATA boundaries, prefixes, attribute ordering, and namespace
/// representation may differ after structural serialization. The wrapper budget still bounds the
/// complete original source before that normalization. The caller owns both policies; this crate
/// intentionally provides no production default budget.
///
/// The unbudgeted [`extract_cdml_from_svg`] entry point remains available for already-admitted
/// internal text and intentionally does not select a resource policy on a caller's behalf.
pub fn extract_cdml_from_svg_with_budget(
    source: &str,
    wrapper_budget: XmlInputBudgetV1,
    cdml_budget: XmlInputBudgetV1,
) -> Result<TypedDocument, CdsvgExtractionError> {
    let wrapper = XmlDocument::parse_with_budget(source, wrapper_budget)
        .map_err(CdsvgExtractionError::WrapperInput)?;
    let payload = extract_from_tree(&wrapper.tree, wrapper.document)?;
    let cdml = wrapper
        .tree
        .to_string(payload)
        .map_err(CdsvgExtractionError::PayloadSerialization)?;
    let payload = XmlDocument::parse_with_budget(&cdml, cdml_budget)
        .map_err(CdsvgExtractionError::PayloadInput)?;
    let indexed = IndexedDocument::from_xml(payload)
        .map_err(super::IndexedDocumentError::from)
        .map_err(TypedDocumentError::from)?;
    TypedDocument::from_indexed(indexed).map_err(CdsvgExtractionError::Typed)
}

fn extract_from_tree(tree: &Xot, document: Node) -> Result<Node, CdsvgExtractionError> {
    let root = tree
        .document_element(document)
        .expect("a parsed XML document has a document element");
    let (root_name, namespace) =
        element_name(tree, root).expect("a document element is always an XML element");
    if root_name != "svg" || namespace != SVG_NAMESPACE {
        return Err(CdsvgExtractionError::NotSvgRoot {
            root_name,
            namespace,
        });
    }

    let payloads = canonical_cdml_descendants(tree, root);
    let payload = match payloads.as_slice() {
        [] => return Err(CdsvgExtractionError::MissingCdmlPayload),
        [payload] => *payload,
        many => {
            return Err(CdsvgExtractionError::MultipleCdmlPayload { count: many.len() });
        }
    };
    Ok(payload)
}

fn canonical_cdml_descendants(tree: &Xot, root: Node) -> Vec<Node> {
    let mut payloads = Vec::new();
    collect_canonical_cdml_descendants(tree, root, &mut payloads);
    payloads
}

fn collect_canonical_cdml_descendants(tree: &Xot, node: Node, payloads: &mut Vec<Node>) {
    for child in tree.children(node) {
        if let Some((local_name, namespace)) = element_name(tree, child) {
            if local_name == "cdml" && namespace == CDML_NAMESPACE {
                payloads.push(child);
            }
            collect_canonical_cdml_descendants(tree, child, payloads);
        }
    }
}
