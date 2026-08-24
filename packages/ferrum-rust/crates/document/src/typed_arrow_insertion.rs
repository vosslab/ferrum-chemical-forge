//! Canonical direct-root arrow insertion owned by the typed CDML layer.
use super::{
    PersistentId, PresentationGesturePoint2V1, TypedDocument, TypedDocumentError, element_name,
    typed_coordinate::canonical_authored_coordinate,
};
use xot::Xot;
impl TypedDocument {
    pub(crate) fn with_insert_straight_equilibrium_arrow(
        &self,
        identifier: &PersistentId,
        start: PresentationGesturePoint2V1,
        end: PresentationGesturePoint2V1,
    ) -> Result<Self, TypedDocumentError> {
        self.with_insert_straight_arrow(identifier, start, end, "equilibrium", None)
    }

    pub(crate) fn with_insert_straight_normal_arrow(
        &self,
        identifier: &PersistentId,
        start: PresentationGesturePoint2V1,
        end: PresentationGesturePoint2V1,
        start_head: bool,
        end_head: bool,
    ) -> Result<Self, TypedDocumentError> {
        self.with_insert_straight_arrow(
            identifier,
            start,
            end,
            "normal",
            Some((start_head, end_head)),
        )
    }

    fn with_insert_straight_arrow(
        &self,
        identifier: &PersistentId,
        start: PresentationGesturePoint2V1,
        end: PresentationGesturePoint2V1,
        arrow_type: &'static str,
        normal_heads: Option<(bool, bool)>,
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
        let arrow_name = name(&mut indexed.xml.tree, "arrow", &namespace);
        let arrow = indexed.xml.tree.new_element(arrow_name);
        for (attribute, value) in [
            ("id", identifier.as_str()),
            ("type", arrow_type),
            ("width", "1.0"),
            ("color", "#000000"),
            ("spline", "no"),
        ] {
            let attribute = indexed.xml.tree.add_name(attribute);
            indexed.xml.tree.set_attribute(arrow, attribute, value)
        }
        if let Some((start_head, end_head)) = normal_heads {
            for (attribute, value) in [
                ("start", if start_head { "yes" } else { "no" }),
                ("end", if end_head { "yes" } else { "no" }),
            ] {
                let attribute = indexed.xml.tree.add_name(attribute);
                indexed.xml.tree.set_attribute(arrow, attribute, value)
            }
        }
        for point in [start, end] {
            let point_name = name(&mut indexed.xml.tree, "point", &namespace);
            let node = indexed.xml.tree.new_element(point_name);
            let x = indexed.xml.tree.add_name("x");
            let y = indexed.xml.tree.add_name("y");
            indexed
                .xml
                .tree
                .set_attribute(node, x, canonical_authored_coordinate(point.x()));
            indexed
                .xml
                .tree
                .set_attribute(node, y, canonical_authored_coordinate(point.y()));
            indexed
                .xml
                .tree
                .append(arrow, node)
                .map_err(TypedDocumentError::Mutation)?
        }
        indexed
            .xml
            .tree
            .append(root, arrow)
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
