//! Atomic rotation of durable direct-core atom points.

use std::collections::HashSet;

use xot::{Node, Xot};

use super::{
    AtomRotationTargetV1, AtomRotationV1, CDML_NAMESPACE, PersistentId, TypedDocument,
    TypedDocumentError, element_name,
};

#[derive(Clone, Copy)]
struct AtomPoint {
    molecule: Node,
    point: Node,
    x: f64,
    y: f64,
}

impl TypedDocument {
    /// Return a detached candidate after one complete selected-atom rotation.
    pub(crate) fn with_atom_rotation(
        &self,
        request: &AtomRotationV1,
    ) -> Result<Self, TypedDocumentError> {
        let points = resolve_points(self, request.targets())?;
        let rotations = rotated_points(request, &points)?;
        let changed = points.iter().zip(&rotations).any(|(point, (x, y))| {
            super::typed_coordinate::coordinate_changes(point.x, *x)
                || super::typed_coordinate::coordinate_changes(point.y, *y)
        });
        let mut candidate = self.detached_candidate()?;
        if !changed {
            return Ok(candidate);
        }
        let candidate_points = resolve_points(&candidate, request.targets())?;
        let indexed = candidate.detached_indexed_mut();
        for (point, (x, y)) in candidate_points.iter().zip(rotations) {
            set_coordinate(&mut indexed.xml.tree, point.point, "x", point.x, x);
            set_coordinate(&mut indexed.xml.tree, point.point, "y", point.y, y);
        }
        let mut molecules = HashSet::new();
        for point in candidate_points {
            if molecules.insert(point.molecule) {
                super::typed_linear_form_metadata::remove_invalid_generated_linear_forms(
                    &mut indexed.xml.tree,
                    point.molecule,
                )?;
            }
        }
        Self::parse(&candidate.to_xml()?)
    }
}

fn rotated_points(
    request: &AtomRotationV1,
    points: &[AtomPoint],
) -> Result<Vec<(f64, f64)>, TypedDocumentError> {
    let (center_x, center_y) = request.center();
    let (sine, cosine) = request.angle_radians().sin_cos();
    points
        .iter()
        .zip(request.targets())
        .map(|(point, target)| {
            let x = center_x + (point.x - center_x) * cosine - (point.y - center_y) * sine;
            let y = center_y + (point.x - center_x) * sine + (point.y - center_y) * cosine;
            if !x.is_finite() || !y.is_finite() {
                return Err(TypedDocumentError::NonFiniteAtomRotation(
                    target.atom_id().clone(),
                ));
            }
            Ok((x, y))
        })
        .collect()
}

fn resolve_points(
    document: &TypedDocument,
    targets: &[AtomRotationTargetV1],
) -> Result<Vec<AtomPoint>, TypedDocumentError> {
    let tree = &document.indexed().xml.tree;
    let root = tree
        .document_element(document.indexed().xml.document)
        .expect("a parsed CDML document has a document element");
    targets
        .iter()
        .map(|target| {
            let molecules = tree
                .children(root)
                .filter(|node| {
                    is_core_element(tree, *node, "molecule")
                        && unqualified_attribute(tree, *node, "id")
                            == Some(target.molecule_id().as_str())
                })
                .collect::<Vec<_>>();
            if molecules.len() != 1 {
                return Err(TypedDocumentError::UnknownAtomRotationTarget {
                    molecule_id: target.molecule_id().clone(),
                    atom_id: target.atom_id().clone(),
                });
            }
            let atoms = tree
                .children(molecules[0])
                .filter(|node| {
                    is_core_element(tree, *node, "atom")
                        && unqualified_attribute(tree, *node, "id")
                            == Some(target.atom_id().as_str())
                })
                .collect::<Vec<_>>();
            if atoms.len() != 1 {
                return Err(TypedDocumentError::UnknownAtomRotationTarget {
                    molecule_id: target.molecule_id().clone(),
                    atom_id: target.atom_id().clone(),
                });
            }
            let points = tree
                .children(atoms[0])
                .filter(|node| is_core_element(tree, *node, "point"))
                .collect::<Vec<_>>();
            if points.len() != 1 {
                return invalid_geometry(target.atom_id());
            }
            let x = coordinate(tree, points[0], "x", target.atom_id())?;
            let y = coordinate(tree, points[0], "y", target.atom_id())?;
            if let Some(z) = unqualified_attribute(tree, points[0], "z") {
                super::typed_coordinate::parse_coordinate(z)
                    .map_err(|()| invalid_geometry_error(target.atom_id()))?;
            }
            Ok(AtomPoint {
                molecule: molecules[0],
                point: points[0],
                x,
                y,
            })
        })
        .collect()
}

fn coordinate(
    tree: &Xot,
    point: Node,
    field: &str,
    atom_id: &PersistentId,
) -> Result<f64, TypedDocumentError> {
    let value =
        unqualified_attribute(tree, point, field).ok_or_else(|| invalid_geometry_error(atom_id))?;
    super::typed_coordinate::parse_coordinate(value).map_err(|()| invalid_geometry_error(atom_id))
}

fn set_coordinate(tree: &mut Xot, point: Node, field: &str, old: f64, new: f64) {
    if super::typed_coordinate::coordinate_changes(old, new) {
        let name = tree.add_name(field);
        tree.set_attribute(
            point,
            name,
            super::typed_coordinate::canonical_authored_coordinate(new),
        );
    }
}

fn invalid_geometry<T>(atom_id: &PersistentId) -> Result<T, TypedDocumentError> {
    Err(invalid_geometry_error(atom_id))
}

fn invalid_geometry_error(atom_id: &PersistentId) -> TypedDocumentError {
    TypedDocumentError::InvalidAtomRotationGeometry(atom_id.clone())
}

fn unqualified_attribute<'a>(tree: &'a Xot, node: Node, expected: &str) -> Option<&'a str> {
    tree.attributes(node).iter().find_map(|(name, value)| {
        let (local_name, namespace) = tree.name_ns_str(name);
        (local_name == expected && namespace.is_empty()).then_some(value.as_str())
    })
}

fn is_core_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node)
        .is_some_and(|(name, namespace)| name == expected && (namespace == CDML_NAMESPACE))
}
