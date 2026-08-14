//! Detached mutation of the first direct core document drawing standard.

use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, DrawingStandardPatchV1, DrawingStandardPropertyChangeV1, TypedDocument,
    TypedDocumentError, element_name,
};

impl TypedDocument {
    pub(crate) fn with_drawing_standard(
        &self,
        patch: &DrawingStandardPatchV1,
    ) -> Result<Self, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let standard = ensure_standard(&mut indexed.xml.tree, root)?;
        apply_changes(&mut indexed.xml.tree, standard, patch.changes())?;
        Self::parse(&candidate.to_xml()?)
    }
}

fn ensure_standard(tree: &mut Xot, root: Node) -> Result<Node, TypedDocumentError> {
    if let Some(standard) = direct_child(tree, root, "standard") {
        return Ok(standard);
    }
    let namespace = element_name(tree, root)
        .expect("a typed CDML root is an element")
        .1;
    let name = element_name_id(tree, "standard", &namespace);
    let standard = tree.new_element(name);
    let before = tree.children(root).find(|node| {
        element_name(tree, *node).is_some_and(|(local, child_namespace)| {
            (child_namespace.is_empty() || child_namespace == CDML_NAMESPACE)
                && !matches!(local.as_str(), "info" | "metadata")
        })
    });
    if let Some(before) = before {
        tree.insert_before(before, standard)
            .map_err(TypedDocumentError::Mutation)?;
    } else {
        tree.append(root, standard)
            .map_err(TypedDocumentError::Mutation)?;
    }
    Ok(standard)
}

fn apply_changes(
    tree: &mut Xot,
    standard: Node,
    changes: &[DrawingStandardPropertyChangeV1],
) -> Result<(), TypedDocumentError> {
    for change in changes {
        match change {
            DrawingStandardPropertyChangeV1::LineWidth(value) => {
                set(tree, standard, "line_width", &value.to_string())
            }
            DrawingStandardPropertyChangeV1::FontSize(value) => {
                set(tree, standard, "font_size", &value.to_string())
            }
            DrawingStandardPropertyChangeV1::FontFamily(value) => {
                set(tree, standard, "font_family", value)
            }
            DrawingStandardPropertyChangeV1::LineColor(value) => {
                set(tree, standard, "line_color", value.as_str())
            }
            DrawingStandardPropertyChangeV1::AreaColor(value) => set(
                tree,
                standard,
                "area_color",
                value.as_ref().map_or("", |color| color.as_str()),
            ),
            DrawingStandardPropertyChangeV1::BondWidth(value) => {
                let bond = ensure_standard_child(tree, standard, "bond")?;
                set(tree, bond, "width", &value.to_string());
            }
            DrawingStandardPropertyChangeV1::WedgeWidth(value) => {
                let bond = ensure_standard_child(tree, standard, "bond")?;
                set(tree, bond, "wedge-width", &value.to_string());
            }
            DrawingStandardPropertyChangeV1::DoubleRatio(value) => {
                let bond = ensure_standard_child(tree, standard, "bond")?;
                set(tree, bond, "double-ratio", &value.to_string());
            }
            DrawingStandardPropertyChangeV1::ShowHydrogens(value) => {
                let atom = ensure_standard_child(tree, standard, "atom")?;
                set(tree, atom, "show_hydrogens", if *value { "1" } else { "0" });
            }
        }
    }
    Ok(())
}

fn ensure_standard_child(
    tree: &mut Xot,
    standard: Node,
    local_name: &str,
) -> Result<Node, TypedDocumentError> {
    if let Some(child) = direct_child(tree, standard, local_name) {
        return Ok(child);
    }
    let namespace = element_name(tree, standard)
        .expect("a typed standard is an element")
        .1;
    let name = element_name_id(tree, local_name, &namespace);
    let child = tree.new_element(name);
    tree.append(standard, child)
        .map_err(TypedDocumentError::Mutation)?;
    Ok(child)
}

fn direct_child(tree: &Xot, parent: Node, expected: &str) -> Option<Node> {
    tree.children(parent)
        .find(|node| is_cdml_element(tree, *node, expected))
}

fn set(tree: &mut Xot, node: Node, name: &str, value: &str) {
    let name = tree.add_name(name);
    tree.set_attribute(node, name, value);
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
