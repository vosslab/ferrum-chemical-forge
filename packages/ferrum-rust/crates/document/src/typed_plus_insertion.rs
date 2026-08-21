//! Canonical direct-root Plus insertion owned by the typed CDML layer.

use super::{
    PersistentId, PresentationGesturePoint2V1, TypedDocument, TypedDocumentError, element_name,
};
use xot::Xot;

const POINTS_PER_CM: f64 = 72.0 / 2.54;

impl TypedDocument {
    /// Insert one unstyled Plus at a document-space point.
    ///
    /// The omitted font and colour attributes deliberately retain the existing
    /// drawing-standard precedence when the canonical document is projected.
    pub(crate) fn with_insert_standard_plus(
        &self,
        identifier: &PersistentId,
        anchor: PresentationGesturePoint2V1,
    ) -> Result<Self, TypedDocumentError> {
        if self.indexed().resolve_id(identifier).is_some() {
            return Err(TypedDocumentError::DuplicateBondId(identifier.clone()));
        }
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("parsed CDML has root");
        let namespace = element_name(&indexed.xml.tree, root)
            .map(|(_, namespace)| namespace)
            .unwrap_or_default();
        let plus_name = name(&mut indexed.xml.tree, "plus", &namespace);
        let plus = indexed.xml.tree.new_element(plus_name);
        let id = indexed.xml.tree.add_name("id");
        indexed
            .xml
            .tree
            .set_attribute(plus, id, identifier.as_str());
        let point_name = name(&mut indexed.xml.tree, "point", &namespace);
        let point = indexed.xml.tree.new_element(point_name);
        let x = indexed.xml.tree.add_name("x");
        let y = indexed.xml.tree.add_name("y");
        indexed
            .xml
            .tree
            .set_attribute(point, x, format!("{:.3}cm", anchor.x() / POINTS_PER_CM));
        indexed
            .xml
            .tree
            .set_attribute(point, y, format!("{:.3}cm", anchor.y() / POINTS_PER_CM));
        indexed
            .xml
            .tree
            .append(plus, point)
            .map_err(TypedDocumentError::Mutation)?;
        indexed
            .xml
            .tree
            .append(root, plus)
            .map_err(TypedDocumentError::Mutation)?;
        Self::parse(&candidate.to_xml()?)
    }
}

fn name(tree: &mut Xot, local: &str, namespace: &str) -> xot::NameId {
    if namespace.is_empty() {
        tree.add_name(local)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local, namespace)
    }
}
