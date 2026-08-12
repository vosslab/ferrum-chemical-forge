//! Extraction of the canonical CDML payload from a decoded CD-SVG document.

use thiserror::Error;
use xot::{Node, Xot};

use super::{CDML_NAMESPACE, TypedDocument, TypedDocumentError, element_name};

const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

/// Failures while locating and validating a CDML payload in a CD-SVG container.
#[derive(Debug, Error)]
pub enum CdsvgExtractionError {
    /// The supplied SVG text was not well-formed XML.
    #[error("cannot parse CD-SVG XML: {0}")]
    Xml(#[from] xot::ParseError),

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

/// Extract and validate the one canonical CDML document embedded in decoded CD-SVG XML.
///
/// The SVG wrapper is disposable presentation data. This API retains only the embedded
/// CDML subtree, structurally normalizes it through [`TypedDocument`], and never attempts
/// to infer editable document state from rendered SVG elements. Compressed `.svgz` input
/// is deliberately outside this decoded-text boundary.
pub fn extract_cdml_from_svg(source: &str) -> Result<TypedDocument, CdsvgExtractionError> {
    let mut tree = Xot::new();
    let document = tree.parse(source)?;
    let root = tree
        .document_element(document)
        .expect("a parsed XML document has a document element");
    let (root_name, namespace) =
        element_name(&tree, root).expect("a document element is always an XML element");
    if root_name != "svg" || namespace != SVG_NAMESPACE {
        return Err(CdsvgExtractionError::NotSvgRoot {
            root_name,
            namespace,
        });
    }

    let payloads = canonical_cdml_descendants(&tree, root);
    let payload = match payloads.as_slice() {
        [] => return Err(CdsvgExtractionError::MissingCdmlPayload),
        [payload] => *payload,
        many => {
            return Err(CdsvgExtractionError::MultipleCdmlPayload { count: many.len() });
        }
    };
    let cdml = tree
        .to_string(payload)
        .map_err(CdsvgExtractionError::PayloadSerialization)?;
    TypedDocument::parse(&cdml).map_err(CdsvgExtractionError::from)
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
