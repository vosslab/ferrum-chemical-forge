//! Deterministic coordinate-only repair planners.

use ferrum_geometry::{HexGrid, Point2, straighten_depiction};

use crate::repair::{CoordinatePatch, DepictionGraph, RepairError, RepairKind, RepairRequest};

/// Calculate a sparse replacement patch without mutating the request graph.
pub fn plan_repair(request: &RepairRequest) -> Result<CoordinatePatch, RepairError> {
    match request.kind() {
        RepairKind::SnapToHexGrid { spacing, origin } => {
            snap_to_hex(request.graph(), spacing, origin)
        }
        RepairKind::Straighten { minimize_rotation } => {
            straighten(request.graph(), minimize_rotation)
        }
    }
}

fn snap_to_hex(
    graph: &DepictionGraph,
    spacing: f64,
    origin: Point2,
) -> Result<CoordinatePatch, RepairError> {
    let grid = HexGrid::new(spacing, origin)?;
    let candidates = graph
        .coordinates()
        .iter()
        .map(|(id, point)| grid.snap(*point).map(|snapped| (id.clone(), snapped)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CoordinatePatch::from_candidates(
        graph.coordinates(),
        candidates,
    ))
}

fn straighten(
    graph: &DepictionGraph,
    minimize_rotation: bool,
) -> Result<CoordinatePatch, RepairError> {
    let ids: Vec<_> = graph.coordinates().keys().cloned().collect();
    let coordinates: Vec<_> = graph.coordinates().values().copied().collect();
    let indices: std::collections::BTreeMap<_, _> = ids
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, id)| (id, index))
        .collect();
    let bonds = graph
        .edges()
        .values()
        .map(|(start, end)| (indices[start], indices[end]))
        .collect::<Vec<_>>();
    let straightened = straighten_depiction(&coordinates, &bonds, minimize_rotation)?;
    Ok(CoordinatePatch::from_candidates(
        graph.coordinates(),
        ids.into_iter().zip(straightened.coordinates),
    ))
}
