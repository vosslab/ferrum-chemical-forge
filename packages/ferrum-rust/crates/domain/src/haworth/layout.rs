//! Deterministic templates for one validated Haworth ring.

use std::collections::BTreeMap;

use ferrum_geometry::Point2;

use crate::haworth::{
    BondDepiction, Face, HaworthDepiction, HaworthError, HaworthLayoutRequest, HaworthPoint,
    RingForm, WedgeEdgeRole,
};

/// Create a deterministic planar depiction from a validated, explicit cycle.
pub fn layout_single_ring(
    request: &HaworthLayoutRequest,
) -> Result<HaworthDepiction, HaworthError> {
    if !request.scale.is_finite() || request.scale <= 0.0 {
        return Err(HaworthError::InvalidSpec(
            "scale must be finite and positive",
        ));
    }
    let template = template(request.topology.ring_form());
    let mut coordinates = BTreeMap::new();
    for (vertex, [x, y]) in request.topology.vertices().iter().zip(template) {
        let point = Point2::new(x * request.scale, y * request.scale)
            .map_err(|_| HaworthError::Unplaceable("scaled coordinate is not finite"))?;
        coordinates.insert(
            vertex.atom.clone(),
            HaworthPoint {
                x: point.x(),
                y: point.y(),
            },
        );
    }
    let mut bonds = BTreeMap::new();
    for (index, bond_id) in request.topology.bond_ids().iter().enumerate() {
        let depiction = match index {
            1 => BondDepiction::HaworthFront {
                edge_role: WedgeEdgeRole::LeftShoulder,
                face: Face::Front,
            },
            2 => BondDepiction::HaworthFront {
                edge_role: WedgeEdgeRole::Center,
                face: Face::Front,
            },
            3 => BondDepiction::HaworthFront {
                edge_role: WedgeEdgeRole::RightShoulder,
                face: Face::Front,
            },
            _ => BondDepiction::Back { face: Face::Back },
        };
        bonds.insert(bond_id.clone(), depiction);
    }
    let bounds = bounds(&coordinates)?;
    HaworthDepiction::new(request.topology.ring_form(), coordinates, bonds, bounds)
}

fn template(ring_form: RingForm) -> &'static [[f64; 2]] {
    // Independent regularized projection templates. The oxygen-first ordering
    // is a contract; coordinates are geometry, not encoded stereochemistry.
    match ring_form {
        RingForm::Pyranose => &[
            [0.5, 0.866_025_403_784_438_6],
            [-0.5, 0.866_025_403_784_438_6],
            [-1.0, 0.0],
            [-0.5, -0.866_025_403_784_438_6],
            [0.5, -0.866_025_403_784_438_6],
            [1.0, 0.0],
        ],
        RingForm::Furanose => &[
            [0.309_016_994_374_947_45, 0.951_056_516_295_153_5],
            [-0.809_016_994_374_947_5, 0.587_785_252_292_473_1],
            [-0.809_016_994_374_947_5, -0.587_785_252_292_473_1],
            [0.309_016_994_374_947_45, -0.951_056_516_295_153_5],
            [1.0, 0.0],
        ],
    }
}

fn bounds(
    coordinates: &BTreeMap<ferrum_core::RecordId, HaworthPoint>,
) -> Result<[HaworthPoint; 2], HaworthError> {
    let mut values = coordinates.values();
    let first = values
        .next()
        .copied()
        .ok_or(HaworthError::Unplaceable("ring has no coordinates"))?;
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (first.x, first.y, first.x, first.y);
    for point in values {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    Ok([
        HaworthPoint { x: min_x, y: min_y },
        HaworthPoint { x: max_x, y: max_y },
    ])
}
