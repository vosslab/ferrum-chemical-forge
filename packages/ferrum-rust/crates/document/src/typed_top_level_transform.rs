//! Atomic affine movement of durable direct-root document objects.

use std::collections::HashSet;

use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, PersistentId, TopLevelRootSelectorV1, TopLevelTransformModeV1,
    TopLevelTransformV1, TypedDocument, TypedDocumentError, element_name,
};

#[derive(Clone, Copy)]
struct CoordinatePair {
    node: Node,
    x_name: &'static str,
    y_name: &'static str,
    x: f64,
    y: f64,
}

pub(crate) struct RootGeometry {
    id: PersistentId,
    node: Node,
    is_molecule: bool,
    pairs: Vec<CoordinatePair>,
    pub(crate) bounds: (f64, f64, f64, f64),
}

#[derive(Clone, Copy)]
enum CoordinateTransform {
    Translate {
        dx: f64,
        dy: f64,
    },
    ScaleAround {
        pivot_x: f64,
        pivot_y: f64,
        scale_x: f64,
        scale_y: f64,
    },
}

impl CoordinateTransform {
    fn apply(self, x: f64, y: f64) -> (f64, f64) {
        match self {
            Self::Translate { dx, dy } => (x + dx, y + dy),
            Self::ScaleAround {
                pivot_x,
                pivot_y,
                scale_x,
                scale_y,
            } => (
                pivot_x + scale_x * (x - pivot_x),
                pivot_y + scale_y * (y - pivot_y),
            ),
        }
    }
}

impl TypedDocument {
    /// Return a detached candidate after one complete root transform.
    pub(crate) fn with_top_level_transform(
        &self,
        request: &TopLevelTransformV1,
    ) -> Result<Self, TypedDocumentError> {
        validate_complete_bracket_selection(self, request)?;
        let geometries = resolve_geometries(self, request.targets())?;
        let transforms = transforms(&geometries, request.transform());
        validate_results(&geometries, &transforms)?;
        let changes_coordinates =
            geometries
                .iter()
                .zip(&transforms)
                .any(|(geometry, transform)| {
                    geometry.pairs.iter().any(|pair| {
                        let (x, y) = transform.apply(pair.x, pair.y);
                        super::typed_coordinate::coordinate_changes(pair.x, x)
                            || super::typed_coordinate::coordinate_changes(pair.y, y)
                    })
                });

        let mut candidate = self.detached_candidate()?;
        let candidate_geometries = resolve_geometries(&candidate, request.targets())?;
        let indexed = candidate.detached_indexed_mut();
        for (geometry, transform) in candidate_geometries.iter().zip(&transforms) {
            for pair in &geometry.pairs {
                let (x, y) = transform.apply(pair.x, pair.y);
                set_coordinate(&mut indexed.xml.tree, pair.node, pair.x_name, pair.x, x);
                set_coordinate(&mut indexed.xml.tree, pair.node, pair.y_name, pair.y, y);
            }
        }
        if changes_coordinates {
            for geometry in candidate_geometries
                .iter()
                .filter(|geometry| geometry.is_molecule)
            {
                super::typed_linear_form_metadata::retire_invalid_generated_linear_forms(
                    &mut indexed.xml.tree,
                    geometry.node,
                )?;
            }
        }
        Self::parse(&candidate.to_xml()?)
    }
}

pub(crate) fn validate_complete_bracket_selection(
    document: &TypedDocument,
    request: &TopLevelTransformV1,
) -> Result<(), TypedDocumentError> {
    let selected = request
        .targets()
        .iter()
        .map(|target| target.root_id().as_str())
        .collect::<HashSet<_>>();
    for pair in super::bracket_pair_projection_v1::bracket_pairs(document) {
        let selected_members = pair
            .member_ids()
            .iter()
            .filter(|identifier| selected.contains(identifier.as_str()))
            .count();
        if selected_members == 1 {
            return Err(TypedDocumentError::PartialBracketTransform(
                pair.pair_id().to_owned(),
            ));
        }
    }
    Ok(())
}

pub(crate) fn resolve_geometries(
    document: &TypedDocument,
    targets: &[TopLevelRootSelectorV1],
) -> Result<Vec<RootGeometry>, TypedDocumentError> {
    let tree = &document.indexed().xml.tree;
    let root = tree
        .document_element(document.indexed().xml.document)
        .expect("a parsed CDML document has a document element");
    let id_name = tree
        .name("id")
        .expect("validated root IDs intern the id name");
    targets
        .iter()
        .map(|target| {
            let matching = tree
                .children(root)
                .filter(|node| {
                    is_cdml_element(tree, *node, target.kind().local_name())
                        && tree.get_attribute(*node, id_name) == Some(target.root_id().as_str())
                })
                .collect::<Vec<_>>();
            if matching.len() != 1 {
                return Err(TypedDocumentError::UnknownTopLevelTransformRoot(
                    target.root_id().clone(),
                ));
            }
            geometry(tree, matching[0], target)
        })
        .collect()
}

fn geometry(
    tree: &Xot,
    root: Node,
    target: &TopLevelRootSelectorV1,
) -> Result<RootGeometry, TypedDocumentError> {
    let name = target.kind().local_name();
    let pairs = if name == "molecule" {
        molecule_pairs(tree, root, target.root_id())?
    } else if matches!(name, "arrow" | "polygon" | "polyline" | "text" | "plus") {
        point_root_pairs(tree, root, target.root_id(), name)?
    } else {
        box_pairs(tree, root, target.root_id())?
    };
    reject_ambiguous_coordinates(tree, root, &pairs, target.root_id())?;
    let minimum_x = pairs
        .iter()
        .map(|pair| pair.x)
        .reduce(f64::min)
        .expect("validated geometry");
    let minimum_y = pairs
        .iter()
        .map(|pair| pair.y)
        .reduce(f64::min)
        .expect("validated geometry");
    let maximum_x = pairs
        .iter()
        .map(|pair| pair.x)
        .reduce(f64::max)
        .expect("validated geometry");
    let maximum_y = pairs
        .iter()
        .map(|pair| pair.y)
        .reduce(f64::max)
        .expect("validated geometry");
    Ok(RootGeometry {
        id: target.root_id().clone(),
        node: root,
        is_molecule: name == "molecule",
        pairs,
        bounds: (minimum_x, minimum_y, maximum_x, maximum_y),
    })
}

fn molecule_pairs(
    tree: &Xot,
    root: Node,
    id: &PersistentId,
) -> Result<Vec<CoordinatePair>, TypedDocumentError> {
    let mut pairs = Vec::new();
    for vertex in tree.children(root).filter(|node| {
        element_name(tree, *node).is_some_and(|(name, namespace)| {
            matches!(name.as_str(), "atom" | "group" | "text" | "query")
                && valid_namespace(&namespace)
        })
    }) {
        let points = tree
            .children(vertex)
            .filter(|node| is_cdml_element(tree, *node, "point"))
            .collect::<Vec<_>>();
        if points.len() != 1 {
            return invalid_geometry(id);
        }
        pairs.push(point_pair(tree, points[0], id)?);
        for mark in tree
            .children(vertex)
            .filter(|node| is_cdml_element(tree, *node, "mark"))
        {
            let x = attribute_coordinate(tree, mark, "x", id);
            let y = attribute_coordinate(tree, mark, "y", id);
            match (x, y) {
                (Ok(Some(x)), Ok(Some(y))) => pairs.push(CoordinatePair {
                    node: mark,
                    x_name: "x",
                    y_name: "y",
                    x,
                    y,
                }),
                (Ok(None), Ok(None)) => {}
                _ => return invalid_geometry(id),
            }
        }
    }
    if pairs.is_empty() {
        return invalid_geometry(id);
    }
    Ok(pairs)
}

fn point_root_pairs(
    tree: &Xot,
    root: Node,
    id: &PersistentId,
    kind: &str,
) -> Result<Vec<CoordinatePair>, TypedDocumentError> {
    let points = tree
        .children(root)
        .filter(|node| is_cdml_element(tree, *node, "point"))
        .collect::<Vec<_>>();
    let minimum = match kind {
        "arrow" | "polyline" => 2,
        "polygon" => 3,
        "text" | "plus" => 1,
        _ => unreachable!(),
    };
    if points.len() < minimum || (matches!(kind, "text" | "plus") && points.len() != 1) {
        return invalid_geometry(id);
    }
    points
        .into_iter()
        .map(|point| point_pair(tree, point, id))
        .collect()
}

fn box_pairs(
    tree: &Xot,
    root: Node,
    id: &PersistentId,
) -> Result<Vec<CoordinatePair>, TypedDocumentError> {
    let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
        attribute_coordinate(tree, root, "x1", id)?,
        attribute_coordinate(tree, root, "y1", id)?,
        attribute_coordinate(tree, root, "x2", id)?,
        attribute_coordinate(tree, root, "y2", id)?,
    ) else {
        return invalid_geometry(id);
    };
    Ok(vec![
        CoordinatePair {
            node: root,
            x_name: "x1",
            y_name: "y1",
            x: x1,
            y: y1,
        },
        CoordinatePair {
            node: root,
            x_name: "x2",
            y_name: "y2",
            x: x2,
            y: y2,
        },
    ])
}

fn point_pair(
    tree: &Xot,
    point: Node,
    id: &PersistentId,
) -> Result<CoordinatePair, TypedDocumentError> {
    if tree.children(point).any(|child| tree.is_element(child)) {
        return invalid_geometry(id);
    }
    let (Some(x), Some(y)) = (
        attribute_coordinate(tree, point, "x", id)?,
        attribute_coordinate(tree, point, "y", id)?,
    ) else {
        return invalid_geometry(id);
    };
    attribute_coordinate(tree, point, "z", id)?;
    Ok(CoordinatePair {
        node: point,
        x_name: "x",
        y_name: "y",
        x,
        y,
    })
}

fn reject_ambiguous_coordinates(
    tree: &Xot,
    root: Node,
    pairs: &[CoordinatePair],
    id: &PersistentId,
) -> Result<(), TypedDocumentError> {
    let accounted = pairs.iter().map(|pair| pair.node).collect::<HashSet<_>>();
    let x_name = tree.name("x");
    let y_name = tree.name("y");
    let ambiguous = tree.descendants(root).skip(1).any(|node| {
        let core =
            element_name(tree, node).is_some_and(|(_, namespace)| valid_namespace(&namespace));
        core && !accounted.contains(&node)
            && (x_name.is_some_and(|name| tree.get_attribute(node, name).is_some())
                || y_name.is_some_and(|name| tree.get_attribute(node, name).is_some()))
    });
    if ambiguous {
        invalid_geometry(id)
    } else {
        Ok(())
    }
}

fn attribute_coordinate(
    tree: &Xot,
    node: Node,
    field: &str,
    id: &PersistentId,
) -> Result<Option<f64>, TypedDocumentError> {
    let Some(name) = tree.name(field) else {
        return Ok(None);
    };
    tree.get_attribute(node, name)
        .map(super::typed_coordinate::parse_coordinate)
        .transpose()
        .map_err(|()| TypedDocumentError::InvalidTopLevelTransformGeometry(id.clone()))
}

fn transforms(
    geometries: &[RootGeometry],
    transform: TopLevelTransformModeV1,
) -> Vec<CoordinateTransform> {
    if let TopLevelTransformModeV1::Translate { dx, dy } = transform {
        return vec![CoordinateTransform::Translate { dx, dy }; geometries.len()];
    }
    let minimum_x = geometries
        .iter()
        .map(|geometry| geometry.bounds.0)
        .reduce(f64::min)
        .unwrap();
    let minimum_y = geometries
        .iter()
        .map(|geometry| geometry.bounds.1)
        .reduce(f64::min)
        .unwrap();
    let maximum_x = geometries
        .iter()
        .map(|geometry| geometry.bounds.2)
        .reduce(f64::max)
        .unwrap();
    let maximum_y = geometries
        .iter()
        .map(|geometry| geometry.bounds.3)
        .reduce(f64::max)
        .unwrap();
    let deltas = match transform {
        TopLevelTransformModeV1::AlignTop => geometries
            .iter()
            .map(|g| (0.0, minimum_y - g.bounds.1))
            .collect(),
        TopLevelTransformModeV1::AlignBottom => geometries
            .iter()
            .map(|g| (0.0, maximum_y - g.bounds.3))
            .collect(),
        TopLevelTransformModeV1::AlignLeft => geometries
            .iter()
            .map(|g| (minimum_x - g.bounds.0, 0.0))
            .collect(),
        TopLevelTransformModeV1::AlignRight => geometries
            .iter()
            .map(|g| (maximum_x - g.bounds.2, 0.0))
            .collect(),
        TopLevelTransformModeV1::AlignCenterX => centered_deltas(geometries, true),
        TopLevelTransformModeV1::AlignCenterY => centered_deltas(geometries, false),
        TopLevelTransformModeV1::Scale { scale_x, scale_y } => {
            return vec![
                CoordinateTransform::ScaleAround {
                    pivot_x: (minimum_x + maximum_x) / 2.0,
                    pivot_y: (minimum_y + maximum_y) / 2.0,
                    scale_x,
                    scale_y,
                };
                geometries.len()
            ];
        }
        TopLevelTransformModeV1::MirrorVertical => {
            return vec![
                CoordinateTransform::ScaleAround {
                    pivot_x: (minimum_x + maximum_x) / 2.0,
                    pivot_y: (minimum_y + maximum_y) / 2.0,
                    scale_x: -1.0,
                    scale_y: 1.0,
                };
                geometries.len()
            ];
        }
        TopLevelTransformModeV1::MirrorHorizontal => {
            return vec![
                CoordinateTransform::ScaleAround {
                    pivot_x: (minimum_x + maximum_x) / 2.0,
                    pivot_y: (minimum_y + maximum_y) / 2.0,
                    scale_x: 1.0,
                    scale_y: -1.0,
                };
                geometries.len()
            ];
        }
        TopLevelTransformModeV1::Translate { .. } => unreachable!(),
    };
    deltas
        .into_iter()
        .map(|(dx, dy)| CoordinateTransform::Translate { dx, dy })
        .collect()
}

fn centered_deltas(geometries: &[RootGeometry], horizontal: bool) -> Vec<(f64, f64)> {
    let centers = geometries
        .iter()
        .map(|geometry| {
            if horizontal {
                (geometry.bounds.0 + geometry.bounds.2) / 2.0
            } else {
                (geometry.bounds.1 + geometry.bounds.3) / 2.0
            }
        })
        .collect::<Vec<_>>();
    let target = (centers.iter().copied().reduce(f64::min).unwrap()
        + centers.iter().copied().reduce(f64::max).unwrap())
        / 2.0;
    centers
        .into_iter()
        .map(|center| {
            if horizontal {
                (target - center, 0.0)
            } else {
                (0.0, target - center)
            }
        })
        .collect()
}

fn validate_results(
    geometries: &[RootGeometry],
    transforms: &[CoordinateTransform],
) -> Result<(), TypedDocumentError> {
    for (geometry, transform) in geometries.iter().zip(transforms) {
        if geometry.pairs.iter().any(|pair| {
            let (x, y) = transform.apply(pair.x, pair.y);
            !x.is_finite() || !y.is_finite()
        }) {
            return Err(TypedDocumentError::NonFiniteTopLevelTransform(
                geometry.id.clone(),
            ));
        }
    }
    Ok(())
}

fn set_coordinate(tree: &mut Xot, node: Node, field: &str, old: f64, new: f64) {
    if !super::typed_coordinate::coordinate_changes(old, new) {
        return;
    }
    let name = tree.add_name(field);
    tree.set_attribute(
        node,
        name,
        super::typed_coordinate::canonical_authored_coordinate(new),
    );
}

fn invalid_geometry<T>(id: &PersistentId) -> Result<T, TypedDocumentError> {
    Err(TypedDocumentError::InvalidTopLevelTransformGeometry(
        id.clone(),
    ))
}

fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node)
        .is_some_and(|(name, namespace)| name == expected && valid_namespace(&namespace))
}

fn valid_namespace(namespace: &str) -> bool {
    namespace == CDML_NAMESPACE
}
