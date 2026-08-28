//! Structured order replacement for one durable typed bond.

use xot::Xot;

use super::{
    CDML_NAMESPACE, DocumentBondOrderV1, DocumentBondPresentationV1, PersistentId, TypedDocument,
    TypedDocumentError, element_name,
};

impl TypedDocument {
    /// Return a detached candidate with one normal bond order replaced.
    ///
    /// Only direct molecule-child bonds participate. The operation retains the
    /// bond element, its identity, endpoints, unknown attributes, and opaque
    /// children while replacing the typed `type` fact with `n1`, `n2`, or `n3`.
    pub(crate) fn with_bond_order(
        &self,
        identifier: &PersistentId,
        order: DocumentBondOrderV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let id_name = indexed.xml.tree.add_name("id");
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let bond = indexed
            .xml
            .tree
            .descendants(root)
            .filter(|node| is_cdml_element(&indexed.xml.tree, *node, "molecule"))
            .find_map(|molecule| {
                indexed.xml.tree.children(molecule).find(|node| {
                    is_cdml_element(&indexed.xml.tree, *node, "bond")
                        && indexed.xml.tree.get_attribute(*node, id_name)
                            == Some(identifier.as_str())
                })
            });
        let Some(bond) = bond else {
            return Ok(None);
        };
        let type_name = indexed.xml.tree.add_name("type");
        let current = indexed
            .xml
            .tree
            .get_attribute(bond, type_name)
            .and_then(DocumentBondPresentationV1::from_cdml_token)
            .ok_or_else(|| TypedDocumentError::UnsupportedBondType(identifier.clone()))?;
        let presentation = DocumentBondPresentationV1::Normal(order);
        if !matches!(current, DocumentBondPresentationV1::Normal(_)) {
            return Err(TypedDocumentError::UnsupportedBondPresentationOrder(
                identifier.clone(),
            ));
        }
        if indexed.xml.tree.get_attribute(bond, type_name) == Some(presentation.cdml_token()) {
            return Ok(Some(candidate));
        }
        indexed
            .xml
            .tree
            .set_attribute(bond, type_name, presentation.cdml_token());
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn is_cdml_element(tree: &Xot, node: xot::Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && (namespace == CDML_NAMESPACE)
    })
}
