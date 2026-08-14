//! Structured coordinate replacement for one durable typed atom.

use super::{
    CDML_NAMESPACE, PersistentId, Point3V1, TypedDocument, TypedDocumentError, element_name,
};

impl TypedDocument {
    /// Return a detached candidate with one atom point replaced.
    ///
    /// The existing point element and namespace are retained. An absent z value
    /// remains absent when the requested z is zero, preserving authored structure
    /// without giving callers an XML mutation surface.
    pub(crate) fn with_atom_position(
        &self,
        identifier: &PersistentId,
        position: Point3V1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let id_name = indexed.xml.tree.add_name("id");
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let atom = indexed.xml.tree.descendants(root).find(|node| {
            element_name(&indexed.xml.tree, *node).is_some_and(|(local_name, namespace)| {
                local_name == "atom"
                    && valid_namespace(&namespace)
                    && indexed.xml.tree.get_attribute(*node, id_name) == Some(identifier.as_str())
            })
        });
        let Some(atom) = atom else {
            return Ok(None);
        };
        let point = indexed.xml.tree.children(atom).find(|node| {
            element_name(&indexed.xml.tree, *node).is_some_and(|(local_name, namespace)| {
                local_name == "point" && valid_namespace(&namespace)
            })
        });
        let point =
            point.ok_or_else(|| TypedDocumentError::MissingAtomPosition(identifier.clone()))?;
        let x_name = indexed.xml.tree.add_name("x");
        let y_name = indexed.xml.tree.add_name("y");
        let z_name = indexed.xml.tree.add_name("z");
        let requested_x = position.x().to_string();
        let requested_y = position.y().to_string();
        let requested_z = position.z().to_string();
        let unchanged_z = indexed.xml.tree.get_attribute(point, z_name) == Some(&requested_z)
            || (position.z() == 0.0 && indexed.xml.tree.get_attribute(point, z_name).is_none());
        if indexed.xml.tree.get_attribute(point, x_name) == Some(&requested_x)
            && indexed.xml.tree.get_attribute(point, y_name) == Some(&requested_y)
            && unchanged_z
        {
            return Ok(Some(candidate));
        }
        indexed.xml.tree.set_attribute(point, x_name, requested_x);
        indexed.xml.tree.set_attribute(point, y_name, requested_y);
        if position.z() != 0.0 || indexed.xml.tree.get_attribute(point, z_name).is_some() {
            indexed.xml.tree.set_attribute(point, z_name, requested_z);
        }
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn valid_namespace(namespace: &str) -> bool {
    namespace.is_empty() || namespace == CDML_NAMESPACE
}
