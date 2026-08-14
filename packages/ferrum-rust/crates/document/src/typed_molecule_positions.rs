//! Atomic typed-CDML replacement of every atom point in one molecule.

use ferrum_geometry::Point2;
use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, PersistentId, Point3V1, TypedDocument, TypedDocumentError, element_name,
};

impl TypedDocument {
    /// Return one detached candidate after validating every clean-geometry target.
    pub(crate) fn with_clean_geometry_positions(
        &self,
        updates: &[(PersistentId, Vec<Point2>)],
    ) -> Result<Self, TypedDocumentError> {
        for (molecule_id, positions) in updates {
            validate_xy_target(self, molecule_id, positions.len())?;
        }
        let mut candidate = self.detached_candidate()?;
        for (molecule_id, positions) in updates {
            apply_xy_target(&mut candidate, molecule_id, positions);
        }
        Self::parse(&candidate.to_xml()?)
    }

    /// Return a detached candidate with every direct atom point replaced in order.
    pub(crate) fn with_molecule_atom_positions(
        &self,
        molecule_identifier: &PersistentId,
        positions: &[Point3V1],
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let id_name = indexed.xml.tree.add_name("id");
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let molecule = indexed.xml.tree.descendants(root).find(|node| {
            element_name(&indexed.xml.tree, *node).is_some_and(|(local_name, namespace)| {
                local_name == "molecule"
                    && valid_namespace(&namespace)
                    && indexed.xml.tree.get_attribute(*node, id_name)
                        == Some(molecule_identifier.as_str())
            })
        });
        let Some(molecule) = molecule else {
            return Ok(None);
        };
        let atoms = indexed
            .xml
            .tree
            .children(molecule)
            .filter(|node| {
                element_name(&indexed.xml.tree, *node).is_some_and(|(local_name, namespace)| {
                    local_name == "atom" && valid_namespace(&namespace)
                })
            })
            .collect::<Vec<_>>();
        if atoms.len() != positions.len() {
            return Err(TypedDocumentError::MoleculePositionCountMismatch {
                molecule: molecule_identifier.clone(),
                expected: atoms.len(),
                actual: positions.len(),
            });
        }
        let x_name = indexed.xml.tree.add_name("x");
        let y_name = indexed.xml.tree.add_name("y");
        let z_name = indexed.xml.tree.add_name("z");
        for (atom_index, (atom, position)) in atoms.into_iter().zip(positions).enumerate() {
            let point = indexed.xml.tree.children(atom).find(|node| {
                element_name(&indexed.xml.tree, *node).is_some_and(|(local_name, namespace)| {
                    local_name == "point" && valid_namespace(&namespace)
                })
            });
            let point = point.ok_or_else(|| TypedDocumentError::MissingMoleculeAtomPosition {
                molecule: molecule_identifier.clone(),
                atom_index,
            })?;
            indexed
                .xml
                .tree
                .set_attribute(point, x_name, position.x().to_string());
            indexed
                .xml
                .tree
                .set_attribute(point, y_name, position.y().to_string());
            if position.z() != 0.0 || indexed.xml.tree.get_attribute(point, z_name).is_some() {
                indexed
                    .xml
                    .tree
                    .set_attribute(point, z_name, position.z().to_string());
            }
        }
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn validate_xy_target(
    document: &TypedDocument,
    molecule_id: &PersistentId,
    position_count: usize,
) -> Result<(), TypedDocumentError> {
    let indexed = document.indexed();
    let molecule = direct_molecule(&indexed.xml.tree, indexed.xml.document, molecule_id)
        .ok_or_else(|| TypedDocumentError::InvalidMoleculeCoordinateTarget(molecule_id.clone()))?;
    let atoms = direct_atoms(&indexed.xml.tree, molecule);
    if atoms.len() != position_count {
        return Err(TypedDocumentError::MoleculePositionCountMismatch {
            molecule: molecule_id.clone(),
            expected: atoms.len(),
            actual: position_count,
        });
    }
    for (atom_index, atom) in atoms.into_iter().enumerate() {
        let points = direct_points(&indexed.xml.tree, atom);
        if points.len() != 1 {
            return Err(TypedDocumentError::MissingMoleculeAtomPosition {
                molecule: molecule_id.clone(),
                atom_index,
            });
        }
        target_coordinate(&indexed.xml.tree, points[0], molecule_id, atom_index, "x")?;
        target_coordinate(&indexed.xml.tree, points[0], molecule_id, atom_index, "y")?;
    }
    Ok(())
}

fn apply_xy_target(document: &mut TypedDocument, molecule_id: &PersistentId, positions: &[Point2]) {
    let indexed = document.detached_indexed_mut();
    let molecule = direct_molecule(&indexed.xml.tree, indexed.xml.document, molecule_id)
        .expect("validated clean-geometry target remains in detached candidate");
    let atoms = direct_atoms(&indexed.xml.tree, molecule);
    let x_name = indexed.xml.tree.add_name("x");
    let y_name = indexed.xml.tree.add_name("y");
    for (atom_index, (atom, position)) in atoms.into_iter().zip(positions).enumerate() {
        let point = direct_points(&indexed.xml.tree, atom)[0];
        let old_x = target_coordinate(&indexed.xml.tree, point, molecule_id, atom_index, "x")
            .expect("validated clean-geometry x coordinate remains in detached candidate");
        let old_y = target_coordinate(&indexed.xml.tree, point, molecule_id, atom_index, "y")
            .expect("validated clean-geometry y coordinate remains in detached candidate");
        if super::typed_coordinate::coordinate_changes(old_x, position.x()) {
            indexed.xml.tree.set_attribute(
                point,
                x_name,
                super::typed_coordinate::canonical_authored_coordinate(position.x()),
            );
        }
        if super::typed_coordinate::coordinate_changes(old_y, position.y()) {
            indexed.xml.tree.set_attribute(
                point,
                y_name,
                super::typed_coordinate::canonical_authored_coordinate(position.y()),
            );
        }
    }
}

fn target_coordinate(
    tree: &Xot,
    point: Node,
    molecule_id: &PersistentId,
    atom_index: usize,
    field: &str,
) -> Result<f64, TypedDocumentError> {
    let value = unqualified_attribute(tree, point, field).ok_or_else(|| {
        TypedDocumentError::MissingMoleculeAtomPosition {
            molecule: molecule_id.clone(),
            atom_index,
        }
    })?;
    super::typed_coordinate::parse_coordinate(value).map_err(|()| {
        TypedDocumentError::MissingMoleculeAtomPosition {
            molecule: molecule_id.clone(),
            atom_index,
        }
    })
}

fn direct_molecule(tree: &Xot, document: Node, molecule_id: &PersistentId) -> Option<Node> {
    let root = tree
        .document_element(document)
        .expect("parsed CDML has a document element");
    tree.children(root).find(|node| {
        element_name(tree, *node).is_some_and(|(local_name, namespace)| {
            local_name == "molecule"
                && valid_namespace(&namespace)
                && unqualified_attribute(tree, *node, "id") == Some(molecule_id.as_str())
        })
    })
}

fn direct_atoms(tree: &Xot, molecule: Node) -> Vec<Node> {
    tree.children(molecule)
        .filter(|node| {
            element_name(tree, *node).is_some_and(|(local_name, namespace)| {
                local_name == "atom" && valid_namespace(&namespace)
            })
        })
        .collect()
}

fn direct_points(tree: &Xot, atom: Node) -> Vec<Node> {
    tree.children(atom)
        .filter(|node| {
            element_name(tree, *node).is_some_and(|(local_name, namespace)| {
                local_name == "point" && valid_namespace(&namespace)
            })
        })
        .collect()
}

fn unqualified_attribute<'a>(tree: &'a Xot, node: Node, expected: &str) -> Option<&'a str> {
    tree.attributes(node).iter().find_map(|(name, value)| {
        let (local_name, namespace) = tree.name_ns_str(name);
        (local_name == expected && namespace.is_empty()).then_some(value.as_str())
    })
}

fn valid_namespace(namespace: &str) -> bool {
    namespace.is_empty() || namespace == CDML_NAMESPACE
}
