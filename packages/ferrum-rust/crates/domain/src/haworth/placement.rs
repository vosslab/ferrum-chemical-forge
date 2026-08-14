//! Bounded deterministic collision-safe placement for Haworth ring trees.

use std::collections::BTreeMap;

use crate::haworth::{
    HaworthError, HaworthLayoutRequest, HaworthPoint, HaworthRingNode, HaworthTreeRequest,
    layout_single_ring,
};

pub(crate) fn find_translation(
    request: &HaworthTreeRequest,
    rings: &BTreeMap<u32, &HaworthRingNode>,
    placements: &mut BTreeMap<u32, (f64, f64)>,
    child_node: u32,
    parent: HaworthPoint,
    child: HaworthPoint,
    rank: usize,
) -> Result<(f64, f64), HaworthError> {
    for candidate in candidate_translations(parent, child, request.scale, rank) {
        placements.insert(child_node, candidate);
        if geometry_safe(request, rings, placements) {
            return Ok(candidate);
        }
        placements.remove(&child_node);
    }
    Err(HaworthError::Unplaceable(
        "bounded tree placement candidates overlap or cross",
    ))
}

fn candidate_translations(
    parent: HaworthPoint,
    child: HaworthPoint,
    scale: f64,
    rank: usize,
) -> Vec<(f64, f64)> {
    const ANGLES: usize = 16;
    const DISTANCES: [f64; 5] = [4.0, 5.5, 7.0, 8.5, 10.0];
    (0..ANGLES)
        .flat_map(|offset| {
            let angle =
                2.0 * std::f64::consts::PI * ((rank + offset) % ANGLES) as f64 / ANGLES as f64;
            DISTANCES.into_iter().map(move |factor| {
                let distance = scale * factor;
                (
                    parent.x + distance * angle.cos() - child.x,
                    parent.y + distance * angle.sin() - child.y,
                )
            })
        })
        .collect()
}

fn geometry_safe(
    request: &HaworthTreeRequest,
    rings: &BTreeMap<u32, &HaworthRingNode>,
    placements: &BTreeMap<u32, (f64, f64)>,
) -> bool {
    let mut polygons = Vec::new();
    let mut points = BTreeMap::new();
    for (node, translation) in placements {
        let Ok(depiction) = placed_depiction(rings[node], request.scale, *translation) else {
            return false;
        };
        polygons.push(
            rings[node]
                .topology
                .vertices()
                .iter()
                .map(|vertex| depiction.coordinates()[&vertex.atom])
                .collect::<Vec<_>>(),
        );
        points.extend(
            depiction
                .coordinates()
                .iter()
                .map(|(atom, point)| (atom.clone(), *point)),
        );
    }
    if !clearance_ok(&points, request.scale)
        || polygons.iter().enumerate().any(|(index, polygon)| {
            polygons[index + 1..]
                .iter()
                .any(|other| polygons_overlap(polygon, other))
        })
    {
        return false;
    }
    let links = request
        .links
        .iter()
        .filter_map(|link| {
            let parent = placements.get(&link.parent.node_id)?;
            let child = placements.get(&link.child.node_id)?;
            Some((
                placed_depiction(rings[&link.parent.node_id], request.scale, *parent)
                    .ok()?
                    .coordinates()[&link.parent.atom],
                placed_depiction(rings[&link.child.node_id], request.scale, *child)
                    .ok()?
                    .coordinates()[&link.child.atom],
            ))
        })
        .collect::<Vec<_>>();
    links.iter().enumerate().all(|(index, (a, b))| {
        links[index + 1..]
            .iter()
            .all(|(c, d)| !segments_cross(*a, *b, *c, *d))
            && polygons.iter().all(|polygon| {
                polygon.iter().enumerate().all(|(edge_index, start)| {
                    let end = polygon[(edge_index + 1) % polygon.len()];
                    !segments_cross(*a, *b, *start, end)
                        || *a == *start
                        || *a == end
                        || *b == *start
                        || *b == end
                })
            })
    })
}

pub(crate) fn placed_depiction(
    ring: &HaworthRingNode,
    scale: f64,
    translation: (f64, f64),
) -> Result<crate::haworth::HaworthDepiction, HaworthError> {
    let base = layout_single_ring(&HaworthLayoutRequest {
        topology: ring.topology.clone(),
        scale,
    })?;
    let mut coordinates = base.coordinates().clone();
    for point in coordinates.values_mut() {
        point.x += translation.0;
        point.y += translation.1;
    }
    crate::haworth::HaworthDepiction::new(
        base.ring_form(),
        coordinates,
        base.bonds().clone(),
        [
            HaworthPoint {
                x: base.bounds()[0].x + translation.0,
                y: base.bounds()[0].y + translation.1,
            },
            HaworthPoint {
                x: base.bounds()[1].x + translation.0,
                y: base.bounds()[1].y + translation.1,
            },
        ],
    )
}

fn clearance_ok(points: &BTreeMap<ferrum_core::RecordId, HaworthPoint>, scale: f64) -> bool {
    let clearance_sq = (scale * 0.32).powi(2);
    let values = points.values().copied().collect::<Vec<_>>();
    values.iter().enumerate().all(|(index, point)| {
        values[index + 1..].iter().all(|other| {
            let dx = point.x - other.x;
            let dy = point.y - other.y;
            dx.mul_add(dx, dy * dy) >= clearance_sq
        })
    })
}

fn polygons_overlap(a: &[HaworthPoint], b: &[HaworthPoint]) -> bool {
    a.iter().any(|point| point_in_polygon(*point, b))
        || b.iter().any(|point| point_in_polygon(*point, a))
        || a.iter().enumerate().any(|(index, start)| {
            b.iter().enumerate().any(|(other, other_start)| {
                segments_cross(
                    *start,
                    a[(index + 1) % a.len()],
                    *other_start,
                    b[(other + 1) % b.len()],
                )
            })
        })
}
fn point_in_polygon(point: HaworthPoint, polygon: &[HaworthPoint]) -> bool {
    let mut inside = false;
    for index in 0..polygon.len() {
        let a = polygon[index];
        let b = polygon[(index + 1) % polygon.len()];
        if (a.y > point.y) != (b.y > point.y)
            && point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x
        {
            inside = !inside;
        }
    }
    inside
}
fn segments_cross(a: HaworthPoint, b: HaworthPoint, c: HaworthPoint, d: HaworthPoint) -> bool {
    fn cross(a: HaworthPoint, b: HaworthPoint, c: HaworthPoint) -> f64 {
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    }
    let ab_c = cross(a, b, c);
    let ab_d = cross(a, b, d);
    let cd_a = cross(c, d, a);
    let cd_b = cross(c, d, b);
    ab_c * ab_d < -1e-10 && cd_a * cd_b < -1e-10
}

#[cfg(test)]
mod tests {
    use super::{HaworthPoint, polygons_overlap};

    #[test]
    fn canonical_polygon_edges_detect_crossing_without_identifier_order() {
        let square = [
            HaworthPoint { x: -2.0, y: -2.0 },
            HaworthPoint { x: 2.0, y: -2.0 },
            HaworthPoint { x: 2.0, y: 2.0 },
            HaworthPoint { x: -2.0, y: 2.0 },
        ];
        let diamond = [
            HaworthPoint { x: -3.0, y: 0.0 },
            HaworthPoint { x: 0.0, y: 3.0 },
            HaworthPoint { x: 3.0, y: 0.0 },
            HaworthPoint { x: 0.0, y: -3.0 },
        ];
        assert!(polygons_overlap(&square, &diamond));
    }
}
