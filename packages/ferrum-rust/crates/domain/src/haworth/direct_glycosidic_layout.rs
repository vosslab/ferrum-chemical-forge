//! Fixed-size local geometry for one validated direct glycosidic Haworth pair.

use std::collections::BTreeMap;

use ferrum_core::RecordId;

use crate::haworth::{
    DirectGlycosidicHaworthTopologyV1, HaworthDepiction, HaworthError, HaworthLayoutRequest,
    HaworthPoint, layout_single_ring,
};

/// Owned input for the deterministic two-ring local-layout convention.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectGlycosidicHaworthLayoutRequestV1 {
    /// Previously graph-validated two-ring topology.
    pub topology: DirectGlycosidicHaworthTopologyV1,
    /// Drawing-unit ring-edge length and bridge-segment length.
    pub scale: f64,
}

/// Immutable local layout for one exterior-oxygen direct glycosidic profile.
///
/// Canonical ring zero occupies the left role and canonical ring one the right
/// role. Their selected carbon attachments lie at `(-scale, 0)` and
/// `(+scale, 0)` respectively; the bridge oxygen lies at the origin. This is a
/// local drawing convention, not a source-coordinate or stereochemical claim.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectGlycosidicHaworthLayoutV1 {
    depictions: [HaworthDepiction; 2],
    bridge_atom: RecordId,
    bridge_point: HaworthPoint,
    bridge_endpoints: BTreeMap<RecordId, [RecordId; 2]>,
    atom_source_orders: BTreeMap<RecordId, usize>,
    bond_source_orders: BTreeMap<RecordId, usize>,
    bounds: [HaworthPoint; 2],
}

impl DirectGlycosidicHaworthLayoutV1 {
    /// Return transformed depictions in canonical topology order.
    #[must_use]
    pub const fn depictions(&self) -> &[HaworthDepiction; 2] {
        &self.depictions
    }

    /// Return the validated exterior oxygen identity.
    #[must_use]
    pub const fn bridge_atom(&self) -> &RecordId {
        &self.bridge_atom
    }

    /// Return the exterior oxygen coordinate.
    #[must_use]
    pub const fn bridge_point(&self) -> HaworthPoint {
        self.bridge_point
    }

    /// Return bridge endpoint identities keyed by selected bridge-bond identity.
    #[must_use]
    pub fn bridge_endpoints(&self) -> &BTreeMap<RecordId, [RecordId; 2]> {
        &self.bridge_endpoints
    }

    /// Return frozen selected-atom source positions from the topology receipt.
    ///
    /// These are graph-local facts only. `BTreeMap` key iteration is identity
    /// sorting, not source, drawing, or stereochemical role order.
    #[must_use]
    pub fn atom_source_orders(&self) -> &BTreeMap<RecordId, usize> {
        &self.atom_source_orders
    }

    /// Return frozen selected-bond source positions from the topology receipt.
    ///
    /// These are graph-local facts only. `BTreeMap` key iteration is identity
    /// sorting, not source, drawing, or stereochemical role order.
    #[must_use]
    pub fn bond_source_orders(&self) -> &BTreeMap<RecordId, usize> {
        &self.bond_source_orders
    }

    /// Return finite bounds derived from both rings and the bridge oxygen.
    #[must_use]
    pub const fn bounds(&self) -> [HaworthPoint; 2] {
        self.bounds
    }
}

/// Lay out two validated rings around their exterior oxygen with a fixed local convention.
pub fn layout_direct_glycosidic_haworth_v1(
    request: &DirectGlycosidicHaworthLayoutRequestV1,
) -> Result<DirectGlycosidicHaworthLayoutV1, HaworthError> {
    if !request.scale.is_finite() || request.scale <= 0.0 {
        return Err(HaworthError::InvalidSpec(
            "scale must be finite and positive",
        ));
    }

    let topology = &request.topology;
    let bridge_point = HaworthPoint { x: 0.0, y: 0.0 };
    let targets = [
        HaworthPoint {
            x: -request.scale,
            y: 0.0,
        },
        HaworthPoint {
            x: request.scale,
            y: 0.0,
        },
    ];
    if targets.iter().any(|point| !finite(*point)) {
        return Err(HaworthError::Unplaceable(
            "attachment coordinate is not finite",
        ));
    }

    let mut depictions = Vec::with_capacity(2);
    let mut bridge_endpoints = BTreeMap::new();
    for (index, ring) in topology.rings().iter().enumerate() {
        let depiction = layout_single_ring(&HaworthLayoutRequest {
            topology: ring.topology().clone(),
            scale: request.scale,
        })?;
        let transformed = transform_ring(
            &depiction,
            ring.topology().vertices(),
            ring.attachment_atom(),
            targets[index],
        )?;
        ensure_bridge_does_not_cross_ring(
            &transformed,
            ring.topology().bond_ids(),
            ring.topology().vertices(),
            ring.attachment_atom(),
            bridge_point,
        )?;
        bridge_endpoints.insert(
            ring.attachment_bond().clone(),
            [
                ring.attachment_atom().clone(),
                topology.bridge().atom().clone(),
            ],
        );
        depictions.push(transformed);
    }
    let depictions: [HaworthDepiction; 2] = depictions
        .try_into()
        .map_err(|_| HaworthError::Unplaceable("two-ring layout was not constructed"))?;
    let bounds = layout_bounds(&depictions, bridge_point)?;
    Ok(DirectGlycosidicHaworthLayoutV1 {
        depictions,
        bridge_atom: topology.bridge().atom().clone(),
        bridge_point,
        bridge_endpoints,
        atom_source_orders: topology.atom_source_orders().clone(),
        bond_source_orders: topology.bond_source_orders().clone(),
        bounds,
    })
}

fn transform_ring(
    depiction: &HaworthDepiction,
    vertices: &[crate::haworth::HaworthVertex],
    attachment: &RecordId,
    target: HaworthPoint,
) -> Result<HaworthDepiction, HaworthError> {
    let index = vertices
        .iter()
        .position(|vertex| &vertex.atom == attachment)
        .ok_or(HaworthError::Unplaceable(
            "ring attachment has no template coordinate",
        ))?;
    let count = vertices.len();
    if count < 3 {
        return Err(HaworthError::Unplaceable(
            "ring has too few template vertices",
        ));
    }
    let attachment_point = point(depiction, &vertices[index].atom)?;
    let previous = point(depiction, &vertices[(index + count - 1) % count].atom)?;
    let next = point(depiction, &vertices[(index + 1) % count].atom)?;
    let outward = normalized(negated(added(
        normalized(subtracted(previous, attachment_point))?,
        normalized(subtracted(next, attachment_point))?,
    )))?;
    let target_direction = if target.x < 0.0 {
        HaworthPoint { x: 1.0, y: 0.0 }
    } else {
        HaworthPoint { x: -1.0, y: 0.0 }
    };
    let cosine = dot(outward, target_direction);
    let sine = cross(outward, target_direction);
    if !cosine.is_finite() || !sine.is_finite() {
        return Err(HaworthError::Unplaceable("ring rotation is not finite"));
    }
    let coordinates = depiction
        .coordinates()
        .iter()
        .map(|(atom, source)| {
            let offset = subtracted(*source, attachment_point);
            let rotated = HaworthPoint {
                x: cosine * offset.x - sine * offset.y,
                y: sine * offset.x + cosine * offset.y,
            };
            let transformed = added(target, rotated);
            if !finite(transformed) {
                return Err(HaworthError::Unplaceable(
                    "transformed coordinate is not finite",
                ));
            }
            Ok((atom.clone(), transformed))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let bounds = coordinate_bounds(coordinates.values().copied())?;
    HaworthDepiction::new(
        depiction.ring_form(),
        coordinates,
        depiction.bonds().clone(),
        bounds,
    )
}

fn ensure_bridge_does_not_cross_ring(
    depiction: &HaworthDepiction,
    bonds: &[RecordId],
    vertices: &[crate::haworth::HaworthVertex],
    attachment: &RecordId,
    bridge: HaworthPoint,
) -> Result<(), HaworthError> {
    let attachment_point = point(depiction, attachment)?;
    for (index, _) in bonds.iter().enumerate() {
        let first = &vertices[index].atom;
        let second = &vertices[(index + 1) % vertices.len()].atom;
        if first == attachment || second == attachment {
            continue;
        }
        let edge_start = point(depiction, first)?;
        let edge_end = point(depiction, second)?;
        if properly_intersects(attachment_point, bridge, edge_start, edge_end) {
            return Err(HaworthError::Unplaceable(
                "bridge segment properly crosses a nonincident ring edge",
            ));
        }
    }
    Ok(())
}

fn layout_bounds(
    depictions: &[HaworthDepiction; 2],
    bridge: HaworthPoint,
) -> Result<[HaworthPoint; 2], HaworthError> {
    coordinate_bounds(
        depictions
            .iter()
            .flat_map(|depiction| depiction.coordinates().values().copied())
            .chain(std::iter::once(bridge)),
    )
}

fn coordinate_bounds(
    mut points: impl Iterator<Item = HaworthPoint>,
) -> Result<[HaworthPoint; 2], HaworthError> {
    let first = points
        .next()
        .filter(|point| finite(*point))
        .ok_or(HaworthError::Unplaceable(
            "layout has no finite coordinates",
        ))?;
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (first.x, first.y, first.x, first.y);
    for point in points {
        if !finite(point) {
            return Err(HaworthError::Unplaceable("layout coordinate is not finite"));
        }
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

fn point(depiction: &HaworthDepiction, atom: &RecordId) -> Result<HaworthPoint, HaworthError> {
    depiction
        .coordinates()
        .get(atom)
        .copied()
        .filter(|point| finite(*point))
        .ok_or(HaworthError::Unplaceable(
            "ring coordinate is absent or non-finite",
        ))
}

fn finite(point: HaworthPoint) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

fn subtracted(left: HaworthPoint, right: HaworthPoint) -> HaworthPoint {
    HaworthPoint {
        x: left.x - right.x,
        y: left.y - right.y,
    }
}

fn added(left: HaworthPoint, right: HaworthPoint) -> HaworthPoint {
    HaworthPoint {
        x: left.x + right.x,
        y: left.y + right.y,
    }
}

fn negated(point: HaworthPoint) -> HaworthPoint {
    HaworthPoint {
        x: -point.x,
        y: -point.y,
    }
}

fn dot(left: HaworthPoint, right: HaworthPoint) -> f64 {
    left.x * right.x + left.y * right.y
}

fn cross(left: HaworthPoint, right: HaworthPoint) -> f64 {
    left.x * right.y - left.y * right.x
}

fn normalized(point: HaworthPoint) -> Result<HaworthPoint, HaworthError> {
    let length = point.x.hypot(point.y);
    if !length.is_finite() || length == 0.0 {
        return Err(HaworthError::Unplaceable(
            "ring outward direction is degenerate",
        ));
    }
    let normalized = HaworthPoint {
        x: point.x / length,
        y: point.y / length,
    };
    if finite(normalized) {
        Ok(normalized)
    } else {
        Err(HaworthError::Unplaceable(
            "ring outward direction is not finite",
        ))
    }
}

fn properly_intersects(a: HaworthPoint, b: HaworthPoint, c: HaworthPoint, d: HaworthPoint) -> bool {
    let first = cross(subtracted(b, a), subtracted(c, a));
    let second = cross(subtracted(b, a), subtracted(d, a));
    let third = cross(subtracted(d, c), subtracted(a, c));
    let fourth = cross(subtracted(d, c), subtracted(b, c));
    finite(a)
        && finite(b)
        && finite(c)
        && finite(d)
        && ((first > 0.0 && second < 0.0) || (first < 0.0 && second > 0.0))
        && ((third > 0.0 && fourth < 0.0) || (third < 0.0 && fourth > 0.0))
}
