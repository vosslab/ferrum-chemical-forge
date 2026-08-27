//! Direct-root molecule-name mutation over one detached retained CDML candidate.

use xot::Node;

use super::{CDML_NAMESPACE, DocumentObjectIdV1, TypedClass, TypedDocument, TypedDocumentError};

impl TypedDocument {
    /// Return a detached candidate with one direct-root molecule name replaced or removed.
    pub(crate) fn with_molecule_name(
        &self,
        molecule_id: &DocumentObjectIdV1,
        name: Option<&str>,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let element_index = {
            let Some(record) = candidate.resolve_document_object_id(molecule_id)? else {
                return Ok(None);
            };
            let components = record.path().components();
            if record.class() != TypedClass::Molecule || components.len() != 1 {
                return Ok(None);
            }
            components[0]
        };
        let indexed = candidate.detached_indexed_mut();
        let tree = &mut indexed.xml.tree;
        let root = tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let target = element_child(tree, root, element_index);
        let Some(target) = target.filter(|node| is_molecule(tree, *node)) else {
            return Ok(None);
        };
        let name_attribute = tree.add_name("name");
        if tree.get_attribute(target, name_attribute) == name {
            return Ok(Some(candidate));
        }
        match name {
            Some(value) => tree.set_attribute(target, name_attribute, value),
            None => tree.remove_attribute(target, name_attribute),
        }
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn element_child(tree: &xot::Xot, parent: Node, index: u32) -> Option<Node> {
    tree.children(parent)
        .filter(|node| tree.element(*node).is_some())
        .nth(index as usize)
}

fn is_molecule(tree: &xot::Xot, node: Node) -> bool {
    super::element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == "molecule" && (namespace == CDML_NAMESPACE)
    })
}
