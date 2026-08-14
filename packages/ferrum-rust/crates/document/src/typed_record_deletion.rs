//! Structured deletion of durable typed molecule records.

use xot::Xot;

use super::{CDML_NAMESPACE, PersistentId, TypedDocument, TypedDocumentError, element_name};

impl TypedDocument {
    /// Return a detached candidate without one durable atom or its incident bonds.
    ///
    /// Only direct molecule children participate. Opaque subtrees remain literal
    /// preservation content even when they contain reference-looking attributes.
    pub(crate) fn with_delete_atom(
        &self,
        identifier: &PersistentId,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let id_name = indexed.xml.tree.add_name("id");
        let start_name = indexed.xml.tree.add_name("start");
        let end_name = indexed.xml.tree.add_name("end");
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let molecules = indexed
            .xml
            .tree
            .descendants(root)
            .filter(|node| is_cdml_element(&indexed.xml.tree, *node, "molecule"))
            .collect::<Vec<_>>();
        let atom = molecules.iter().copied().find_map(|molecule| {
            indexed.xml.tree.children(molecule).find(|node| {
                is_cdml_element(&indexed.xml.tree, *node, "atom")
                    && indexed.xml.tree.get_attribute(*node, id_name) == Some(identifier.as_str())
            })
        });
        let Some(atom) = atom else {
            return Ok(None);
        };
        let incident_bonds = molecules
            .iter()
            .flat_map(|molecule| indexed.xml.tree.children(*molecule))
            .filter(|node| {
                is_cdml_element(&indexed.xml.tree, *node, "bond")
                    && (indexed.xml.tree.get_attribute(*node, start_name)
                        == Some(identifier.as_str())
                        || indexed.xml.tree.get_attribute(*node, end_name)
                            == Some(identifier.as_str()))
            })
            .collect::<Vec<_>>();
        for bond in incident_bonds {
            indexed
                .xml
                .tree
                .remove(bond)
                .map_err(TypedDocumentError::Mutation)?;
        }
        indexed
            .xml
            .tree
            .remove(atom)
            .map_err(TypedDocumentError::Mutation)?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }

    /// Return a detached candidate without one durable typed bond.
    ///
    /// Opaque elements are preservation content and cannot be selected by this
    /// operation even when they carry an identical `id` spelling.
    pub(crate) fn with_delete_bond(
        &self,
        identifier: &PersistentId,
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
        indexed
            .xml
            .tree
            .remove(bond)
            .map_err(TypedDocumentError::Mutation)?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn is_cdml_element(tree: &Xot, node: xot::Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && (namespace.is_empty() || namespace == CDML_NAMESPACE)
    })
}
