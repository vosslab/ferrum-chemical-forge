//! Structured direct-bond property mutation preserving retained XML content.

use xot::{Node, Xot};

use super::{
    BondPropertiesPatchV1, BondPropertyChangeV1, CDML_NAMESPACE, DocumentBondPresentationV1,
    PersistentId, TypedDocument, TypedDocumentError, element_name,
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
    let replacement = changes.iter().find_map(|change| match change {
        BondPropertyChangeV1::Presentation(value) => Some(*value),
        _ => None,
    });
    let requires_presentation = replacement.is_some()
        || changes.iter().any(|change| {
            matches!(
                change,
                BondPropertyChangeV1::Center(Some(_))
                    | BondPropertyChangeV1::BondWidth(Some(_))
                    | BondPropertyChangeV1::WedgeWidth(Some(_))
            )
        });
    let presentation = if requires_presentation {
        let type_name = tree.add_name("type");
        let current = tree
            .get_attribute(bond, type_name)
            .and_then(DocumentBondPresentationV1::from_cdml_token)
            .ok_or_else(|| TypedDocumentError::UnsupportedBondType(bond_id.clone()))?;
        Some(replacement.unwrap_or(current))
    } else {
        None
    };
    if let Some(presentation) = presentation {
        validate_presentation_properties(tree, bond, presentation, changes, bond_id)?;
        if replacement.is_some() {
            set(tree, bond, "type", presentation.cdml_token());
        }
    }
    for change in changes {
        match change {
            BondPropertyChangeV1::Presentation(_) => {}
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

fn validate_presentation_properties(
    tree: &mut Xot,
    bond: Node,
    presentation: DocumentBondPresentationV1,
    changes: &[BondPropertyChangeV1],
    bond_id: &PersistentId,
) -> Result<(), TypedDocumentError> {
    let center = final_optional_bool_is_authored(tree, bond, changes);
    let bond_width = final_bond_width_is_authored(tree, bond, changes);
    let wedge_width = final_wedge_width_is_authored(tree, bond, changes);
    let center_compatible = matches!(
        presentation,
        DocumentBondPresentationV1::Normal(super::DocumentBondOrderV1::Double)
    );
    if center && !center_compatible {
        return Err(TypedDocumentError::IncompatibleBondPresentationProperty {
            bond_id: bond_id.clone(),
            property: "center",
        });
    }
    let bond_width_compatible = matches!(
        presentation,
        DocumentBondPresentationV1::Normal(
            super::DocumentBondOrderV1::Double | super::DocumentBondOrderV1::Triple
        )
    );
    if bond_width && !bond_width_compatible {
        return Err(TypedDocumentError::IncompatibleBondPresentationProperty {
            bond_id: bond_id.clone(),
            property: "bond_width",
        });
    }
    let wedge_width_compatible = matches!(
        presentation,
        DocumentBondPresentationV1::SolidWedge | DocumentBondPresentationV1::HashedWedge
    );
    if wedge_width && !wedge_width_compatible {
        return Err(TypedDocumentError::IncompatibleBondPresentationProperty {
            bond_id: bond_id.clone(),
            property: "wedge_width",
        });
    }
    Ok(())
}

fn final_optional_bool_is_authored(
    tree: &mut Xot,
    bond: Node,
    changes: &[BondPropertyChangeV1],
) -> bool {
    changes
        .iter()
        .find_map(|change| match change {
            BondPropertyChangeV1::Center(value) => Some(value.is_some()),
            _ => None,
        })
        .unwrap_or_else(|| {
            let name = tree.add_name("center");
            tree.get_attribute(bond, name).is_some()
        })
}

fn final_bond_width_is_authored(
    tree: &mut Xot,
    bond: Node,
    changes: &[BondPropertyChangeV1],
) -> bool {
    changes
        .iter()
        .find_map(|change| match change {
            BondPropertyChangeV1::BondWidth(value) => Some(value.is_some()),
            _ => None,
        })
        .unwrap_or_else(|| {
            let name = tree.add_name("bond_width");
            tree.get_attribute(bond, name).is_some()
        })
}

fn final_wedge_width_is_authored(
    tree: &mut Xot,
    bond: Node,
    changes: &[BondPropertyChangeV1],
) -> bool {
    changes
        .iter()
        .find_map(|change| match change {
            BondPropertyChangeV1::WedgeWidth(value) => Some(value.is_some()),
            _ => None,
        })
        .unwrap_or_else(|| {
            let name = tree.add_name("wedge_width");
            tree.get_attribute(bond, name).is_some()
        })
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
        local_name == expected && (namespace == CDML_NAMESPACE)
    })
}
