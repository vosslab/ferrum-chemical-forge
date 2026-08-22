//! Direct atom-number mutation over one detached retained CDML candidate.

use xot::{Node, Xot};

use super::{CDML_NAMESPACE, PersistentId, TypedDocument, TypedDocumentError, element_name};

impl TypedDocument {
    /// Return a detached candidate with one atom number pair assigned or cleared.
    pub(crate) fn with_atom_number(
        &self,
        molecule_id: &PersistentId,
        atom_id: &PersistentId,
        assignment: Option<(u64, bool)>,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let Some(atom) = direct_atom(
            &mut indexed.xml.tree,
            indexed.xml.document,
            molecule_id.as_str(),
            atom_id.as_str(),
        ) else {
            return Ok(None);
        };
        if has_legacy_number_mark(&mut indexed.xml.tree, atom) {
            return Err(TypedDocumentError::LegacyAtomNumberMark(atom_id.clone()));
        }
        match assignment {
            Some((number, show_number)) => {
                set(&mut indexed.xml.tree, atom, "number", number.to_string());
                set(
                    &mut indexed.xml.tree,
                    atom,
                    "show_number",
                    if show_number { "yes" } else { "no" },
                );
            }
            None => {
                remove(&mut indexed.xml.tree, atom, "number");
                remove(&mut indexed.xml.tree, atom, "show_number");
            }
        }
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn direct_atom(tree: &mut Xot, document: Node, molecule_id: &str, atom_id: &str) -> Option<Node> {
    let id_name = tree.add_name("id");
    let root = tree
        .document_element(document)
        .expect("a parsed CDML document has a document element");
    let molecule = tree.children(root).find(|node| {
        is_cdml_element(tree, *node, "molecule")
            && tree.get_attribute(*node, id_name) == Some(molecule_id)
    })?;
    tree.children(molecule).find(|node| {
        is_cdml_element(tree, *node, "atom") && tree.get_attribute(*node, id_name) == Some(atom_id)
    })
}

fn has_legacy_number_mark(tree: &mut Xot, atom: Node) -> bool {
    let type_name = tree.add_name("type");
    tree.children(atom).any(|node| {
        is_cdml_element(tree, node, "mark")
            && tree.get_attribute(node, type_name) == Some("atom_number")
    })
}

fn set(tree: &mut Xot, node: Node, name: &str, value: impl AsRef<str>) {
    let name = tree.add_name(name);
    tree.set_attribute(node, name, value.as_ref());
}

fn remove(tree: &mut Xot, node: Node, name: &str) {
    let name = tree.add_name(name);
    tree.remove_attribute(node, name);
}

fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && (namespace == CDML_NAMESPACE)
    })
}
