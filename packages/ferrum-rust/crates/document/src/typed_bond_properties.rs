//! Structured direct-bond property mutation preserving retained XML content.

use xot::{Node, Xot};

use super::{
    BondPropertiesPatchV1, BondPropertyChangeV1, CDML_NAMESPACE, DocumentBondOrderV1,
    DocumentBondStyleV1, PersistentId, TypedDocument, TypedDocumentError, element_name,
};

impl TypedDocument {
    /// Return a detached candidate with one complete bond-properties patch applied.
    pub(crate) fn with_bond_properties(
        &self,
        patch: &BondPropertiesPatchV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let bond = direct_bond(
            &mut indexed.xml.tree,
            indexed.xml.document,
            patch.bond_id().as_str(),
        );
        let Some(bond) = bond else {
            return Ok(None);
        };
        apply_changes(
            &mut indexed.xml.tree,
            bond,
            patch.bond_id(),
            patch.changes(),
        )?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn direct_bond(tree: &mut Xot, document: Node, identifier: &str) -> Option<Node> {
    let id_name = tree.add_name("id");
    let root = tree
        .document_element(document)
        .expect("a parsed CDML document has a document element");
    tree.children(root)
        .filter(|node| is_cdml_element(tree, *node, "molecule"))
        .find_map(|molecule| {
            tree.children(molecule).find(|node| {
                is_cdml_element(tree, *node, "bond")
                    && tree.get_attribute(*node, id_name) == Some(identifier)
            })
        })
}

fn apply_changes(
    tree: &mut Xot,
    bond: Node,
    bond_id: &PersistentId,
    changes: &[BondPropertyChangeV1],
) -> Result<(), TypedDocumentError> {
    let order = changes.iter().find_map(|change| match change {
        BondPropertyChangeV1::Order(value) => Some(*value),
        _ => None,
    });
    let style = changes.iter().find_map(|change| match change {
        BondPropertyChangeV1::Style(value) => Some(*value),
        _ => None,
    });
    if order.is_some() || style.is_some() {
        let type_name = tree.add_name("type");
        let current = tree
            .get_attribute(bond, type_name)
            .ok_or_else(|| TypedDocumentError::UnsupportedBondType(bond_id.clone()))?;
        let (current_style, current_order) = parse_editable_type(current)
            .ok_or_else(|| TypedDocumentError::UnsupportedBondType(bond_id.clone()))?;
        let style = style.unwrap_or(current_style);
        let order = order.unwrap_or(current_order);
        if !style.supports_order(order) {
            return Err(TypedDocumentError::UnsupportedBondStyleOrder(
                bond_id.clone(),
            ));
        }
        set(tree, bond, "type", type_token(style, order));
    }
    for change in changes {
        match change {
            BondPropertyChangeV1::Order(_) | BondPropertyChangeV1::Style(_) => {}
            BondPropertyChangeV1::Center(value) => set_optional_bool(tree, bond, "center", *value),
            BondPropertyChangeV1::LineWidth(value) => {
                set_optional_scalar(tree, bond, "line_width", *value)
            }
            BondPropertyChangeV1::BondWidth(value) => {
                set_optional_signed(tree, bond, "bond_width", *value)
            }
            BondPropertyChangeV1::WedgeWidth(value) => {
                set_optional_scalar(tree, bond, "wedge_width", *value)
            }
            BondPropertyChangeV1::Color(value) => match value {
                Some(value) => set(tree, bond, "color", value.as_str()),
                None => remove(tree, bond, "color"),
            },
        }
    }
    Ok(())
}

fn parse_editable_type(value: &str) -> Option<(DocumentBondStyleV1, DocumentBondOrderV1)> {
    let mut characters = value.chars();
    let style = DocumentBondStyleV1::from_cdml_prefix(characters.next()?)?;
    let order = match (characters.next()?, characters.next()) {
        ('1', None) => DocumentBondOrderV1::Single,
        ('2', None) => DocumentBondOrderV1::Double,
        ('3', None) => DocumentBondOrderV1::Triple,
        _ => return None,
    };
    Some((style, order))
}

fn type_token(style: DocumentBondStyleV1, order: DocumentBondOrderV1) -> String {
    format!("{}{}", style.cdml_prefix(), &order.cdml_token()[1..])
}

fn set_optional_bool(tree: &mut Xot, node: Node, name: &str, value: Option<bool>) {
    match value {
        Some(true) => set(tree, node, name, "yes"),
        Some(false) => set(tree, node, name, "no"),
        None => remove(tree, node, name),
    }
}

fn set_optional_scalar(
    tree: &mut Xot,
    node: Node,
    name: &str,
    value: Option<super::PositiveFiniteV1>,
) {
    match value {
        Some(value) => set(tree, node, name, value.value().to_string()),
        None => remove(tree, node, name),
    }
}

fn set_optional_signed(
    tree: &mut Xot,
    node: Node,
    name: &str,
    value: Option<super::NonZeroFiniteV1>,
) {
    match value {
        Some(value) => set(tree, node, name, value.value().to_string()),
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

fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && (namespace.is_empty() || namespace == CDML_NAMESPACE)
    })
}
