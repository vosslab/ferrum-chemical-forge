//! Structured insertion of one complete Rust-owned Wavy path.

use xot::{NameId, Xot};

use super::{
    CDML_NAMESPACE, PersistentId, TypedDocument, TypedDocumentError, WavyInsertionV1, element_name,
};

impl TypedDocument {
    /// Append one validated Wavy root while preserving all retained root content.
    pub(crate) fn with_insert_wavy(
        &self,
        wavy_id: &PersistentId,
        insertion: &WavyInsertionV1,
    ) -> Result<Self, TypedDocumentError> {
        if self.indexed().resolve_id(wavy_id).is_some() {
            return Err(TypedDocumentError::DuplicateInsertionId(wavy_id.clone()));
        }
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let (_, namespace) = element_name(&indexed.xml.tree, root)
            .expect("a parsed CDML document has an XML root element");
        let names = WavyNames::new(&mut indexed.xml.tree, namespace);
        let polyline = indexed.xml.tree.new_element(names.polyline);
        indexed
            .xml
            .tree
            .set_attribute(polyline, names.id, wavy_id.as_str());
        indexed
            .xml
            .tree
            .set_attribute(polyline, names.line_color, "#000000");
        indexed.xml.tree.set_attribute(polyline, names.width, "1.5");
        indexed.xml.tree.set_attribute(polyline, names.spline, "no");
        indexed
            .xml
            .tree
            .set_attribute(polyline, names.style, "wavy");
        for point in insertion.points() {
            let point_node = indexed.xml.tree.new_element(names.point);
            indexed
                .xml
                .tree
                .set_attribute(point_node, names.x, point.x().to_string());
            indexed
                .xml
                .tree
                .set_attribute(point_node, names.y, point.y().to_string());
            indexed
                .xml
                .tree
                .set_attribute(point_node, names.z, point.z().to_string());
            indexed
                .xml
                .tree
                .append(polyline, point_node)
                .map_err(TypedDocumentError::Mutation)?;
        }
        indexed
            .xml
            .tree
            .append(root, polyline)
            .map_err(TypedDocumentError::Mutation)?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized)
    }
}

#[derive(Clone, Copy)]
struct WavyNames {
    polyline: NameId,
    point: NameId,
    id: NameId,
    line_color: NameId,
    width: NameId,
    spline: NameId,
    style: NameId,
    x: NameId,
    y: NameId,
    z: NameId,
}

impl WavyNames {
    fn new(tree: &mut Xot, namespace: String) -> Self {
        let namespace = if namespace == CDML_NAMESPACE {
            namespace
        } else {
            unreachable!("TypedDocument accepts only no-namespace or CDML roots")
        };
        Self {
            polyline: element_name_id(tree, "polyline", &namespace),
            point: element_name_id(tree, "point", &namespace),
            id: tree.add_name("id"),
            line_color: tree.add_name("line_color"),
            width: tree.add_name("width"),
            spline: tree.add_name("spline"),
            style: tree.add_name("style"),
            x: tree.add_name("x"),
            y: tree.add_name("y"),
            z: tree.add_name("z"),
        }
    }
}

fn element_name_id(tree: &mut Xot, local_name: &str, namespace: &str) -> NameId {
    if namespace.is_empty() {
        tree.add_name(local_name)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local_name, namespace)
    }
}
