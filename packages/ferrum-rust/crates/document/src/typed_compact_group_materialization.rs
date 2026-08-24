//! Canonical detached CDML replacement of one first-class compact group.

use xot::Node;

use super::compact_group_v1::CompactGroupMaterializationDefinitionV1;
use super::typed_coordinate::{canonical_authored_coordinate, parse_coordinate};
use super::{CDML_NAMESPACE, PersistentId, TypedDocument, TypedDocumentError, element_name};

/// Authenticated source facts for one closed compact-group replacement.
#[derive(Clone, Debug)]
pub(crate) struct CompactGroupMaterializationSourceV1 {
    pub(crate) catalog_key: super::CompactGroupCatalogKeyV1,
    pub(crate) exterior_atom: Option<PersistentId>,
    pub(crate) exterior_bond: Option<PersistentId>,
}

impl TypedDocument {
    /// Inspect one direct compact-group record before session identifiers are reserved.
    pub(crate) fn compact_group_materialization_source_v1(
        &self,
        molecule_id: &PersistentId,
        group_id: &PersistentId,
    ) -> Result<CompactGroupMaterializationSourceV1, TypedDocumentError> {
        let indexed = self.indexed();
        let tree = &indexed.xml().tree;
        let root = tree
            .document_element(indexed.xml().document)
            .expect("typed document has root");
        let molecule = direct_molecule(tree, root, molecule_id)?;
        let group = direct_child(tree, molecule, "compact-group", group_id)?;
        let key = super::CompactGroupCatalogKeyV1::parse(
            attribute(tree, group, "catalog-key").ok_or_else(|| {
                TypedDocumentError::InvalidCompactGroupMaterialization(group_id.clone())
            })?,
        )
        .ok_or_else(|| TypedDocumentError::InvalidCompactGroupMaterialization(group_id.clone()))?;
        let exterior = exterior_bonds(tree, molecule, group_id)?;
        if exterior.len() > 1 {
            return Err(TypedDocumentError::InvalidCompactGroupMaterialization(
                group_id.clone(),
            ));
        }
        let (exterior_atom, exterior_bond) =
            exterior
                .into_iter()
                .next()
                .map_or(Ok((None, None)), |bond| {
                    if attribute(tree, bond, "type") != Some("n1") {
                        return Err(TypedDocumentError::InvalidCompactGroupMaterialization(
                            group_id.clone(),
                        ));
                    }
                    let start = attribute(tree, bond, "start").ok_or_else(|| {
                        TypedDocumentError::InvalidCompactGroupMaterialization(group_id.clone())
                    })?;
                    let end = attribute(tree, bond, "end").ok_or_else(|| {
                        TypedDocumentError::InvalidCompactGroupMaterialization(group_id.clone())
                    })?;
                    let other = if start == group_id.as_str() {
                        end
                    } else if end == group_id.as_str() {
                        start
                    } else {
                        return Err(TypedDocumentError::InvalidCompactGroupMaterialization(
                            group_id.clone(),
                        ));
                    };
                    let atom = PersistentId::new(other.to_owned()).map_err(|_| {
                        TypedDocumentError::InvalidCompactGroupMaterialization(group_id.clone())
                    })?;
                    direct_child(tree, molecule, "atom", &atom)?;
                    let bond_id = PersistentId::new(
                        attribute(tree, bond, "id")
                            .ok_or_else(|| {
                                TypedDocumentError::InvalidCompactGroupMaterialization(
                                    group_id.clone(),
                                )
                            })?
                            .to_owned(),
                    )
                    .map_err(|_| {
                        TypedDocumentError::InvalidCompactGroupMaterialization(group_id.clone())
                    })?;
                    Ok((Some(atom), Some(bond_id)))
                })?;
        Ok(CompactGroupMaterializationSourceV1 {
            catalog_key: key,
            exterior_atom,
            exterior_bond,
        })
    }

    /// Replace a typed compact group with one fully specified closed recipe.
    pub(crate) fn with_materialized_compact_group_v1(
        &self,
        molecule_id: &PersistentId,
        group_id: &PersistentId,
        definition: CompactGroupMaterializationDefinitionV1,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
    ) -> Result<Self, TypedDocumentError> {
        if atom_ids.len() != definition.atoms.len() || bond_ids.len() != definition.bonds.len() {
            return Err(TypedDocumentError::InvalidCompactGroupMaterialization(
                group_id.clone(),
            ));
        }
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let tree = &mut indexed.xml.tree;
        let root = tree
            .document_element(indexed.xml.document)
            .expect("typed document has root");
        let molecule = direct_molecule(tree, root, molecule_id)?;
        let group = direct_child(tree, molecule, "compact-group", group_id)?;
        let anchor = point(tree, group)?;
        let orientation = attribute(tree, group, "orientation-degrees")
            .ok_or_else(|| {
                TypedDocumentError::InvalidCompactGroupMaterialization(group_id.clone())
            })?
            .parse::<f64>()
            .map_err(|_| {
                TypedDocumentError::InvalidCompactGroupMaterialization(group_id.clone())
            })?;
        if !orientation.is_finite() {
            return Err(TypedDocumentError::InvalidCompactGroupMaterialization(
                group_id.clone(),
            ));
        }
        let exterior = exterior_bonds(tree, molecule, group_id)?;
        if exterior.len() > 1 {
            return Err(TypedDocumentError::InvalidCompactGroupMaterialization(
                group_id.clone(),
            ));
        }
        let id = tree.add_name("id");
        let start = tree.add_name("start");
        let end = tree.add_name("end");
        let name = tree.add_name("name");
        let charge = tree.add_name("charge");
        let explicit_hydrogens = tree.add_name("explicit_hydrogens");
        let bond_type = tree.add_name("type");
        let x = tree.add_name("x");
        let y = tree.add_name("y");
        let z = tree.add_name("z");
        let namespace = element_name(tree, molecule).expect("typed molecule").1;
        let atom_name = qualified(tree, "atom", &namespace);
        let point_name = qualified(tree, "point", &namespace);
        let bond_name = qualified(tree, "bond", &namespace);
        tree.remove(group).map_err(TypedDocumentError::Mutation)?;
        if let Some(exterior_bond) = exterior.first() {
            let endpoint = if tree.get_attribute(*exterior_bond, start) == Some(group_id.as_str()) {
                start
            } else if tree.get_attribute(*exterior_bond, end) == Some(group_id.as_str()) {
                end
            } else {
                return Err(TypedDocumentError::InvalidCompactGroupMaterialization(
                    group_id.clone(),
                ));
            };
            tree.set_attribute(*exterior_bond, endpoint, atom_ids[0].as_str());
        }
        let radians = orientation.to_radians();
        let (sin, cos) = radians.sin_cos();
        for (fact, atom_id) in definition.atoms.iter().zip(atom_ids) {
            let atom = tree.new_element(atom_name);
            tree.set_attribute(atom, id, atom_id.as_str());
            tree.set_attribute(atom, name, fact.element);
            tree.set_attribute(atom, charge, fact.formal_charge.to_string());
            tree.set_attribute(
                atom,
                explicit_hydrogens,
                fact.explicit_hydrogens.to_string(),
            );
            let point = tree.new_element(point_name);
            tree.set_attribute(
                point,
                x,
                canonical_authored_coordinate(anchor.0 + fact.local_x * cos - fact.local_y * sin),
            );
            tree.set_attribute(
                point,
                y,
                canonical_authored_coordinate(anchor.1 + fact.local_x * sin + fact.local_y * cos),
            );
            tree.set_attribute(point, z, canonical_authored_coordinate(anchor.2));
            tree.append(atom, point)
                .map_err(TypedDocumentError::Mutation)?;
            tree.append(molecule, atom)
                .map_err(TypedDocumentError::Mutation)?;
        }
        for (fact, bond_id) in definition.bonds.iter().zip(bond_ids) {
            let bond = tree.new_element(bond_name);
            tree.set_attribute(bond, id, bond_id.as_str());
            tree.set_attribute(bond, bond_type, fact.cdml_type);
            tree.set_attribute(bond, start, atom_ids[fact.start].as_str());
            tree.set_attribute(bond, end, atom_ids[fact.end].as_str());
            tree.append(molecule, bond)
                .map_err(TypedDocumentError::Mutation)?;
        }
        Self::parse(&candidate.to_xml()?)
    }
}

fn direct_molecule(
    tree: &xot::Xot,
    root: Node,
    molecule_id: &PersistentId,
) -> Result<Node, TypedDocumentError> {
    tree.children(root)
        .find(|node| {
            is_named(tree, *node, "molecule")
                && attribute(tree, *node, "id") == Some(molecule_id.as_str())
        })
        .ok_or_else(|| TypedDocumentError::UnknownMolecule(molecule_id.clone()))
}

fn direct_child(
    tree: &xot::Xot,
    molecule: Node,
    expected: &str,
    id: &PersistentId,
) -> Result<Node, TypedDocumentError> {
    tree.children(molecule)
        .find(|node| {
            is_named(tree, *node, expected) && attribute(tree, *node, "id") == Some(id.as_str())
        })
        .ok_or_else(|| TypedDocumentError::InvalidCompactGroupMaterialization(id.clone()))
}

fn exterior_bonds(
    tree: &xot::Xot,
    molecule: Node,
    group_id: &PersistentId,
) -> Result<Vec<Node>, TypedDocumentError> {
    let mut bonds = Vec::new();
    for node in tree
        .children(molecule)
        .filter(|node| is_named(tree, *node, "bond"))
    {
        let start = attribute(tree, node, "start").ok_or_else(|| {
            TypedDocumentError::InvalidCompactGroupMaterialization(group_id.clone())
        })?;
        let end = attribute(tree, node, "end").ok_or_else(|| {
            TypedDocumentError::InvalidCompactGroupMaterialization(group_id.clone())
        })?;
        if start == group_id.as_str() || end == group_id.as_str() {
            bonds.push(node);
        }
    }
    Ok(bonds)
}

fn point(tree: &xot::Xot, group: Node) -> Result<(f64, f64, f64), TypedDocumentError> {
    let point = tree
        .children(group)
        .find(|node| is_named(tree, *node, "point"))
        .ok_or(TypedDocumentError::InvalidCompactGroupMaterialization(
            PersistentId::new("group".to_owned()).expect("constant id"),
        ))?;
    let parse = |name| attribute(tree, point, name).and_then(|value| parse_coordinate(value).ok());
    match (parse("x"), parse("y")) {
        (Some(x), Some(y)) => Ok((x, y, parse("z").unwrap_or(0.0))),
        _ => Err(TypedDocumentError::InvalidCompactGroupMaterialization(
            PersistentId::new("group".to_owned()).expect("constant id"),
        )),
    }
}

fn attribute<'a>(tree: &'a xot::Xot, node: Node, name: &str) -> Option<&'a str> {
    tree.get_attribute(node, tree.name(name)?)
}
fn is_named(tree: &xot::Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node)
        .is_some_and(|(name, namespace)| name == expected && namespace == CDML_NAMESPACE)
}
fn qualified(tree: &mut xot::Xot, local: &str, namespace: &str) -> xot::NameId {
    if namespace.is_empty() {
        tree.add_name(local)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local, namespace)
    }
}
