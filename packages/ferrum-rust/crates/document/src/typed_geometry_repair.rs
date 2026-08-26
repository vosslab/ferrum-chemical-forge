//! Atomic document adapter for implemented pure-Rust geometry repair.

use std::collections::{HashMap, HashSet};

use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_domain::repair::{
    DepictionBond, DepictionGraph, DepictionVertex, RepairKind, RepairRequest, plan_repair,
};
use ferrum_geometry::Point2;
use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, GeometryRepairKindV1, GeometryRepairV1, PersistentId, TypedDocument,
    TypedDocumentError, element_name,
};

pub(super) struct MoleculeGraph {
    pub(super) graph: DepictionGraph,
    pub(super) atom_ids: HashMap<RecordId, PersistentId>,
    pub(super) atom_source_order: Vec<PersistentId>,
}

struct PlannedReplacement {
    atom_id: PersistentId,
    expected: Point2,
    replacement: Point2,
}

struct MoleculePlan {
    molecule_id: PersistentId,
    replacements: Vec<PlannedReplacement>,
}

impl TypedDocument {
    /// Return a detached candidate after all selected repairs plan successfully.
    pub(crate) fn with_geometry_repair(
        &self,
        request: &GeometryRepairV1,
    ) -> Result<Self, TypedDocumentError> {
        let plans = request
            .molecule_ids()
            .iter()
            .map(|molecule_id| plan_molecule(self, request, molecule_id))
            .collect::<Result<Vec<_>, _>>()?;
        let mut candidate = self.detached_candidate()?;
        if plans.iter().all(|plan| {
            plan.replacements.iter().all(|replacement| {
                !super::typed_coordinate::coordinate_changes(
                    replacement.expected.x(),
                    replacement.replacement.x(),
                ) && !super::typed_coordinate::coordinate_changes(
                    -replacement.expected.y(),
                    -replacement.replacement.y(),
                )
            })
        }) {
            return Ok(candidate);
        }
        let indexed = candidate.detached_indexed_mut();
        for plan in plans {
            let molecule =
                direct_molecule(&indexed.xml.tree, indexed.xml.document, &plan.molecule_id)
                    .ok_or_else(|| {
                        TypedDocumentError::UnknownGeometryRepairMolecule(plan.molecule_id.clone())
                    })?;
            for replacement in plan.replacements {
                let point = direct_atom_point(&indexed.xml.tree, molecule, &replacement.atom_id)?;
                let current = point_coordinate(&indexed.xml.tree, point, &replacement.atom_id)?;
                if current != replacement.expected {
                    return Err(TypedDocumentError::GeometryRepairPrecondition(
                        replacement.atom_id,
                    ));
                }
                set_coordinate(
                    &mut indexed.xml.tree,
                    point,
                    "x",
                    current.x(),
                    replacement.replacement.x(),
                );
                set_coordinate(
                    &mut indexed.xml.tree,
                    point,
                    "y",
                    -current.y(),
                    -replacement.replacement.y(),
                );
            }
            super::typed_linear_form_metadata::remove_invalid_generated_linear_forms(
                &mut indexed.xml.tree,
                molecule,
            )?;
        }
        Self::parse(&candidate.to_xml()?)
    }

    /// Apply complete planned y-up coordinates only after every direct target and
    /// source coordinate has been revalidated against this retained document.
    pub(super) fn with_prepared_straightening(
        &self,
        updates: &[(PersistentId, Vec<Point2>, Vec<Point2>)],
    ) -> Result<Self, TypedDocumentError> {
        for (molecule_id, expected, replacement) in updates {
            let source = molecule_graph(self, molecule_id)?;
            if source.atom_source_order.len() != expected.len()
                || expected.len() != replacement.len()
            {
                return Err(TypedDocumentError::MoleculePositionCountMismatch {
                    molecule: molecule_id.clone(),
                    expected: source.atom_source_order.len(),
                    actual: replacement.len(),
                });
            }
            for (atom_id, expected) in source.atom_source_order.iter().zip(expected) {
                let molecule = direct_molecule(
                    &self.indexed().xml.tree,
                    self.indexed().xml.document,
                    molecule_id,
                )
                .expect("molecule graph resolves exactly one direct molecule");
                let point = direct_atom_point(&self.indexed().xml.tree, molecule, atom_id)?;
                if point_coordinate(&self.indexed().xml.tree, point, atom_id)? != *expected {
                    return Err(TypedDocumentError::GeometryRepairPrecondition(
                        atom_id.clone(),
                    ));
                }
            }
        }

        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        for (molecule_id, _expected, replacement) in updates {
            let molecule = direct_molecule(&indexed.xml.tree, indexed.xml.document, molecule_id)
                .expect("validated direct molecule remains in detached candidate");
            let atom_ids = source_atom_ids(&indexed.xml.tree, molecule, molecule_id)?;
            for (atom_id, replacement) in atom_ids.iter().zip(replacement) {
                let point = direct_atom_point(&indexed.xml.tree, molecule, atom_id)?;
                let current = point_coordinate(&indexed.xml.tree, point, atom_id)?;
                set_coordinate(
                    &mut indexed.xml.tree,
                    point,
                    "x",
                    current.x(),
                    replacement.x(),
                );
                set_coordinate(
                    &mut indexed.xml.tree,
                    point,
                    "y",
                    -current.y(),
                    -replacement.y(),
                );
            }
            super::typed_linear_form_metadata::remove_invalid_generated_linear_forms(
                &mut indexed.xml.tree,
                molecule,
            )?;
        }
        Self::parse(&candidate.to_xml()?)
    }
}

fn source_atom_ids(
    tree: &Xot,
    molecule: Node,
    molecule_id: &PersistentId,
) -> Result<Vec<PersistentId>, TypedDocumentError> {
    tree.children(molecule)
        .filter(|node| is_core_element(tree, *node, "atom"))
        .map(|atom| {
            persistent_attribute(tree, atom, "id").ok_or_else(|| {
                TypedDocumentError::UnsupportedGeometryRepairMolecule(molecule_id.clone())
            })
        })
        .collect()
}

fn plan_molecule(
    document: &TypedDocument,
    request: &GeometryRepairV1,
    molecule_id: &PersistentId,
) -> Result<MoleculePlan, TypedDocumentError> {
    let source = molecule_graph(document, molecule_id)?;
    let kind = match request.kind() {
        GeometryRepairKindV1::SnapToHexGrid => RepairKind::SnapToHexGrid {
            spacing: request.target_spacing_points(),
            origin: Point2::new(0.0, 0.0).expect("zero is finite"),
        },
        GeometryRepairKindV1::StraightenBonds => RepairKind::StraightenTerminalBonds,
        GeometryRepairKindV1::NormalizeBondLengths => RepairKind::NormalizeBondLengths {
            spacing: request.target_spacing_points(),
        },
        GeometryRepairKindV1::NormalizeBondAngles => RepairKind::NormalizeBondAngles {
            spacing: request.target_spacing_points(),
        },
        GeometryRepairKindV1::NormalizeRings => RepairKind::NormalizeSingleRing {
            spacing: request.target_spacing_points(),
        },
    };
    let patch = plan_repair(&RepairRequest::new(source.graph, kind)).map_err(|error| {
        TypedDocumentError::GeometryRepairPlanning {
            molecule_id: molecule_id.clone(),
            detail: error.to_string(),
        }
    })?;
    let replacements = patch
        .replacements()
        .map(|(record_id, replacement)| {
            let atom_id = source
                .atom_ids
                .get(record_id)
                .expect("graph identities retain their source map")
                .clone();
            PlannedReplacement {
                atom_id,
                expected: replacement.expected(),
                replacement: replacement.replacement(),
            }
        })
        .collect();
    Ok(MoleculePlan {
        molecule_id: molecule_id.clone(),
        replacements,
    })
}

pub(super) fn molecule_graph(
    document: &TypedDocument,
    molecule_id: &PersistentId,
) -> Result<MoleculeGraph, TypedDocumentError> {
    let tree = &document.indexed().xml.tree;
    let molecule = direct_molecule(tree, document.indexed().xml.document, molecule_id)
        .ok_or_else(|| TypedDocumentError::UnknownGeometryRepairMolecule(molecule_id.clone()))?;
    if tree.children(molecule).any(|child| {
        element_name(tree, child).is_some_and(|(name, namespace)| {
            valid_namespace(&namespace) && matches!(name.as_str(), "group" | "text" | "query")
        })
    }) {
        return Err(TypedDocumentError::UnsupportedGeometryRepairMolecule(
            molecule_id.clone(),
        ));
    }
    let mut vertices = Vec::new();
    let mut atom_ids = HashMap::new();
    let mut atom_source_order = Vec::new();
    for atom in tree
        .children(molecule)
        .filter(|node| is_core_element(tree, *node, "atom"))
    {
        let atom_id = persistent_attribute(tree, atom, "id").ok_or_else(|| {
            TypedDocumentError::UnsupportedGeometryRepairMolecule(molecule_id.clone())
        })?;
        let point = direct_atom_point(tree, molecule, &atom_id)?;
        let coordinate = point_coordinate(tree, point, &atom_id)?;
        let record_id = record_id(RecordKind::Atom, &atom_id)?;
        vertices.push(
            DepictionVertex::new(record_id.clone(), coordinate).map_err(|error| {
                TypedDocumentError::GeometryRepairPlanning {
                    molecule_id: molecule_id.clone(),
                    detail: error.to_string(),
                }
            })?,
        );
        atom_source_order.push(atom_id.clone());
        atom_ids.insert(record_id, atom_id);
    }
    if vertices.is_empty() {
        return Err(TypedDocumentError::UnsupportedGeometryRepairMolecule(
            molecule_id.clone(),
        ));
    }
    let atom_records = atom_ids.keys().cloned().collect::<HashSet<_>>();
    let mut bonds = Vec::new();
    for bond in tree
        .children(molecule)
        .filter(|node| is_core_element(tree, *node, "bond"))
    {
        let bond_id = persistent_attribute(tree, bond, "id").ok_or_else(|| {
            TypedDocumentError::UnsupportedGeometryRepairMolecule(molecule_id.clone())
        })?;
        let start = endpoint_record(tree, bond, "start")?;
        let end = endpoint_record(tree, bond, "end")?;
        if !atom_records.contains(&start) || !atom_records.contains(&end) {
            return Err(TypedDocumentError::UnsupportedGeometryRepairMolecule(
                molecule_id.clone(),
            ));
        }
        bonds.push(
            DepictionBond::new(record_id(RecordKind::Bond, &bond_id)?, start, end).map_err(
                |error| TypedDocumentError::GeometryRepairPlanning {
                    molecule_id: molecule_id.clone(),
                    detail: error.to_string(),
                },
            )?,
        );
    }
    let graph = DepictionGraph::new(vertices, bonds).map_err(|error| {
        TypedDocumentError::GeometryRepairPlanning {
            molecule_id: molecule_id.clone(),
            detail: error.to_string(),
        }
    })?;
    Ok(MoleculeGraph {
        graph,
        atom_ids,
        atom_source_order,
    })
}

fn endpoint_record(tree: &Xot, bond: Node, field: &str) -> Result<RecordId, TypedDocumentError> {
    let value = persistent_attribute(tree, bond, field)
        .ok_or_else(|| TypedDocumentError::InvalidGeometryRepairBond(field.to_owned()))?;
    record_id(RecordKind::Atom, &value)
}

fn record_id(kind: RecordKind, id: &PersistentId) -> Result<RecordId, TypedDocumentError> {
    let identifier = Identifier::new(id.as_str().to_owned())
        .map_err(|_| TypedDocumentError::InvalidGeometryRepairIdentity(id.clone()))?;
    RecordId::new(kind, identifier)
        .map_err(|_| TypedDocumentError::InvalidGeometryRepairIdentity(id.clone()))
}

fn direct_molecule(tree: &Xot, document: Node, molecule_id: &PersistentId) -> Option<Node> {
    let root = tree.document_element(document).ok()?;
    let matches = tree
        .children(root)
        .filter(|node| {
            is_core_element(tree, *node, "molecule")
                && unqualified_attribute(tree, *node, "id") == Some(molecule_id.as_str())
        })
        .collect::<Vec<_>>();
    (matches.len() == 1).then(|| matches[0])
}

fn direct_atom_point(
    tree: &Xot,
    molecule: Node,
    atom_id: &PersistentId,
) -> Result<Node, TypedDocumentError> {
    let atoms = tree
        .children(molecule)
        .filter(|node| {
            is_core_element(tree, *node, "atom")
                && unqualified_attribute(tree, *node, "id") == Some(atom_id.as_str())
        })
        .collect::<Vec<_>>();
    if atoms.len() != 1 {
        return Err(TypedDocumentError::InvalidGeometryRepairAtom(
            atom_id.clone(),
        ));
    }
    let points = tree
        .children(atoms[0])
        .filter(|node| is_core_element(tree, *node, "point"))
        .collect::<Vec<_>>();
    if points.len() != 1 {
        return Err(TypedDocumentError::InvalidGeometryRepairAtom(
            atom_id.clone(),
        ));
    }
    Ok(points[0])
}

fn point_coordinate(
    tree: &Xot,
    point: Node,
    atom_id: &PersistentId,
) -> Result<Point2, TypedDocumentError> {
    let x = coordinate(tree, point, "x", atom_id)?;
    let y = coordinate(tree, point, "y", atom_id)?;
    if let Some(z) = unqualified_attribute(tree, point, "z") {
        super::typed_coordinate::parse_coordinate(z)
            .map_err(|()| TypedDocumentError::InvalidGeometryRepairAtom(atom_id.clone()))?;
    }
    Point2::new(x, -y).map_err(|_| TypedDocumentError::InvalidGeometryRepairAtom(atom_id.clone()))
}

fn coordinate(
    tree: &Xot,
    point: Node,
    field: &str,
    atom_id: &PersistentId,
) -> Result<f64, TypedDocumentError> {
    let value = unqualified_attribute(tree, point, field)
        .ok_or_else(|| TypedDocumentError::InvalidGeometryRepairAtom(atom_id.clone()))?;
    super::typed_coordinate::parse_coordinate(value)
        .map_err(|()| TypedDocumentError::InvalidGeometryRepairAtom(atom_id.clone()))
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

fn persistent_attribute(tree: &Xot, node: Node, field: &str) -> Option<PersistentId> {
    unqualified_attribute(tree, node, field).and_then(|value| PersistentId::new(value).ok())
}

fn unqualified_attribute<'a>(tree: &'a Xot, node: Node, expected: &str) -> Option<&'a str> {
    tree.attributes(node).iter().find_map(|(name, value)| {
        let (local_name, namespace) = tree.name_ns_str(name);
        (local_name == expected && namespace.is_empty()).then_some(value.as_str())
    })
}

fn is_core_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node)
        .is_some_and(|(name, namespace)| name == expected && valid_namespace(&namespace))
}

fn valid_namespace(namespace: &str) -> bool {
    namespace == CDML_NAMESPACE
}
