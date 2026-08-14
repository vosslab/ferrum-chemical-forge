//! Structured direct-atom property mutation preserving retained XML content.

use xot::{Node, Xot};

use super::{
    AtomPropertiesPatchV1, AtomPropertyChangeV1, CDML_NAMESPACE, TypedDocument, TypedDocumentError,
    element_name,
};

impl TypedDocument {
    /// Return a detached candidate with one complete atom-properties patch applied.
    pub(crate) fn with_atom_properties(
        &self,
        patch: &AtomPropertiesPatchV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let atom = direct_atom(
            &mut indexed.xml.tree,
            indexed.xml.document,
            patch.atom_id().as_str(),
        );
        let Some(atom) = atom else {
            return Ok(None);
        };
        let font_changes = patch.changes().iter().any(|change| {
            matches!(
                change,
                AtomPropertyChangeV1::FontSize(_) | AtomPropertyChangeV1::LabelColor(_)
            )
        });
        let font = font_changes
            .then(|| direct_font(&mut indexed.xml.tree, atom, patch.atom_id().as_str()))
            .transpose()?;
        apply_changes(&mut indexed.xml.tree, atom, font, patch.changes())?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn direct_atom(tree: &mut Xot, document: Node, identifier: &str) -> Option<Node> {
    let id_name = tree.add_name("id");
    let root = tree
        .document_element(document)
        .expect("a parsed CDML document has a document element");
    tree.children(root)
        .filter(|node| is_cdml_element(tree, *node, "molecule"))
        .find_map(|molecule| {
            tree.children(molecule).find(|node| {
                is_cdml_element(tree, *node, "atom")
                    && tree.get_attribute(*node, id_name) == Some(identifier)
            })
        })
}

fn direct_font(tree: &mut Xot, atom: Node, atom_id: &str) -> Result<Node, TypedDocumentError> {
    let fonts = tree
        .children(atom)
        .filter(|node| is_cdml_element(tree, *node, "font"))
        .collect::<Vec<_>>();
    match fonts.as_slice() {
        [font] => Ok(*font),
        [] => {
            let namespace = element_name(tree, atom)
                .expect("a typed atom is an element")
                .1;
            let name = element_name_id(tree, "font", &namespace);
            let font = tree.new_element(name);
            tree.append(atom, font)
                .map_err(TypedDocumentError::Mutation)?;
            Ok(font)
        }
        _ => Err(TypedDocumentError::AmbiguousAtomFonts(
            super::PersistentId::new(atom_id.to_owned())
                .expect("a validated patch carries a persistent ID"),
        )),
    }
}

fn apply_changes(
    tree: &mut Xot,
    atom: Node,
    font: Option<Node>,
    changes: &[AtomPropertyChangeV1],
) -> Result<(), TypedDocumentError> {
    for change in changes {
        match change {
            AtomPropertyChangeV1::Element(value) => set(tree, atom, "name", value),
            AtomPropertyChangeV1::FormalCharge(0) => remove(tree, atom, "charge"),
            AtomPropertyChangeV1::FormalCharge(value) => {
                set(tree, atom, "charge", value.to_string())
            }
            AtomPropertyChangeV1::Valence(value) => set_optional(tree, atom, "valency", *value),
            AtomPropertyChangeV1::Isotope(value) => set_optional(tree, atom, "isotope", *value),
            AtomPropertyChangeV1::Multiplicity(1) => remove(tree, atom, "multiplicity"),
            AtomPropertyChangeV1::Multiplicity(value) => {
                set(tree, atom, "multiplicity", value.to_string())
            }
            AtomPropertyChangeV1::Show(value) => {
                set(tree, atom, "show", if *value { "yes" } else { "no" })
            }
            AtomPropertyChangeV1::ShowHydrogens(value) => {
                set(tree, atom, "hydrogens", if *value { "on" } else { "off" })
            }
            AtomPropertyChangeV1::FontSize(value) => {
                set(
                    tree,
                    font.expect("font changes resolve one direct font"),
                    "size",
                    value.value().to_string(),
                );
            }
            AtomPropertyChangeV1::LabelColor(value) => {
                set(
                    tree,
                    font.expect("font changes resolve one direct font"),
                    "color",
                    value.as_str(),
                );
            }
        }
    }
    Ok(())
}

fn set_optional(tree: &mut Xot, node: Node, name: &str, value: Option<u16>) {
    match value {
        Some(value) => set(tree, node, name, value.to_string()),
        None => remove(tree, node, name),
    }
}

fn set(tree: &mut Xot, node: Node, name: &str, value: impl AsRef<str>) {
    let name = tree.add_name(name);
    tree.set_attribute(node, name, value.as_ref());
}

fn remove(tree: &mut Xot, node: Node, name: &str) {
    let name = tree.add_name(name);
    tree.remove_attribute(node, name);
}

fn element_name_id(tree: &mut Xot, local_name: &str, namespace: &str) -> xot::NameId {
    if namespace.is_empty() {
        tree.add_name(local_name)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local_name, namespace)
    }
}

fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && (namespace.is_empty() || namespace == CDML_NAMESPACE)
    })
}
