//! Private-profile rendering for one checked direct glycosidic Haworth pair.

use std::collections::BTreeSet;

use ferrum_core::RecordId;
use ferrum_domain::haworth::{
    DirectGlycosidicHaworthBondStyleV1, DirectGlycosidicHaworthDepictionSpecV1,
    DirectGlycosidicHaworthPositionV1, HaworthPoint,
};

use crate::{Paint, PositiveFinite, RenderError, RenderPoint, RenderProvenance};

const MAX_TARGETS: usize = 14;
const OVERLAP_RATIO: f64 = 0.25;
const FRONT_PAD_RATIO: f64 = 0.35;
const CUBIC_ARC_LIMIT: f64 = std::f64::consts::FRAC_PI_2;

/// Exact input for the closed direct-glycosidic renderer profile.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectGlycosidicHaworthRenderRequestV1 {
    provenance: RenderProvenance,
    spec: DirectGlycosidicHaworthDepictionSpecV1,
    paint: Paint,
    line_width: PositiveFinite,
    wedge_width: PositiveFinite,
}

impl DirectGlycosidicHaworthRenderRequestV1 {
    /// Construct a direct profile request with no caller-selected paint order.
    #[must_use]
    pub fn new(
        provenance: RenderProvenance,
        spec: DirectGlycosidicHaworthDepictionSpecV1,
        paint: Paint,
        line_width: PositiveFinite,
        wedge_width: PositiveFinite,
    ) -> Self {
        Self {
            provenance,
            spec,
            paint,
            line_width,
            wedge_width,
        }
    }

    /// Return the immutable source observation identity.
    #[must_use]
    pub(crate) const fn provenance(&self) -> RenderProvenance {
        self.provenance
    }
    /// Return the accepted direct depiction facts.
    #[must_use]
    #[cfg(test)]
    pub(crate) const fn spec(&self) -> &DirectGlycosidicHaworthDepictionSpecV1 {
        &self.spec
    }
    /// Return the exact paint.
    #[must_use]
    pub(crate) const fn paint(&self) -> &Paint {
        &self.paint
    }
    /// Return ordinary bond width.
    #[must_use]
    pub(crate) const fn line_width(&self) -> PositiveFinite {
        self.line_width
    }
    /// Return front-face width.
    #[must_use]
    pub(crate) const fn wedge_width(&self) -> PositiveFinite {
        self.wedge_width
    }
}

/// Closed direct-profile operation; this is deliberately not a wire DTO.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum DirectGlycosidicHaworthDrawOpV1 {
    /// An ordinary ring or bridge stroke with butt caps.
    OrdinaryLine {
        bond: RecordId,
        source_order: u32,
        endpoints: [RenderPoint; 2],
        width: PositiveFinite,
    },
    /// One padded, round-capped q1 front stroke.
    HaworthFrontStroke {
        bond: RecordId,
        source_order: u32,
        endpoints: [RenderPoint; 2],
        width: PositiveFinite,
    },
    /// One directional, rounded, filled w1 wedge.
    RoundedFrontWedge {
        bond: RecordId,
        source_order: u32,
        tip: RenderPoint,
        base: RenderPoint,
        width: PositiveFinite,
        commands: Vec<DirectGlycosidicHaworthPathCommandV1>,
    },
}

impl DirectGlycosidicHaworthDrawOpV1 {
    /// Return the durable target identity.
    #[must_use]
    #[cfg(test)]
    pub(crate) const fn bond(&self) -> &RecordId {
        match self {
            Self::OrdinaryLine { bond, .. }
            | Self::HaworthFrontStroke { bond, .. }
            | Self::RoundedFrontWedge { bond, .. } => bond,
        }
    }
    /// Return checked graph-local provenance, never a paint-order input.
    #[must_use]
    #[cfg(test)]
    pub(crate) const fn source_order(&self) -> u32 {
        match self {
            Self::OrdinaryLine { source_order, .. }
            | Self::HaworthFrontStroke { source_order, .. }
            | Self::RoundedFrontWedge { source_order, .. } => *source_order,
        }
    }
}

/// One private-profile path command retained for the private draw stream.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DirectGlycosidicHaworthPathCommandV1 {
    /// Move without painting.
    MoveTo(RenderPoint),
    /// Straight side or base segment.
    LineTo(RenderPoint),
    /// Cubic approximation of a source rounded corner arc.
    CubicTo {
        control_1: RenderPoint,
        control_2: RenderPoint,
        end: RenderPoint,
    },
    /// Close the filled wedge boundary.
    Close,
}

/// Owned, non-serializable result with renderer-owned operation order.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectGlycosidicHaworthRenderPlanV1 {
    provenance: RenderProvenance,
    paint: Paint,
    operations: Vec<DirectGlycosidicHaworthDrawOpV1>,
}

impl DirectGlycosidicHaworthRenderPlanV1 {
    /// Return the exact plan paint.
    #[must_use]
    pub(crate) const fn paint(&self) -> &Paint {
        &self.paint
    }
    /// Return operations in the closed direct-profile paint order.
    #[must_use]
    pub(crate) fn operations(&self) -> &[DirectGlycosidicHaworthDrawOpV1] {
        &self.operations
    }
}

/// Lower an accepted direct depiction receipt into the bounded private profile.
pub fn lower_direct_glycosidic_haworth_v1(
    request: &DirectGlycosidicHaworthRenderRequestV1,
) -> Result<DirectGlycosidicHaworthRenderPlanV1, RenderError> {
    let mut operations = Vec::new();
    operations
        .try_reserve(MAX_TARGETS)
        .map_err(|_| RenderError::ResourceExhausted)?;
    let mut seen = BTreeSet::new();
    for ring in request.spec.rings() {
        for bond in ring.bonds_in_canonical_cycle_order() {
            let fact = request.spec.ring_bonds().get(bond).ok_or_else(missing)?;
            if fact.style() == DirectGlycosidicHaworthBondStyleV1::N1
                && fact.haworth_position() == DirectGlycosidicHaworthPositionV1::Back
            {
                push_ordinary(
                    &mut operations,
                    &mut seen,
                    fact.bond(),
                    fact.source_order(),
                    fact.endpoints(),
                    request,
                )?;
            }
        }
    }
    // Bridge identity has a fixed ring-zero/ring-one role in the topology receipt.
    for ring in request.spec.rings() {
        let bridge = request
            .spec
            .bridge_bonds()
            .values()
            .find(|bridge| {
                ring.bonds_in_canonical_cycle_order().iter().any(|bond| {
                    request
                        .spec
                        .ring_bonds()
                        .get(bond)
                        .is_some_and(|fact| fact.endpoints().contains(&bridge.endpoints()[0]))
                })
            })
            .ok_or_else(missing)?;
        push_ordinary(
            &mut operations,
            &mut seen,
            bridge.bond(),
            bridge.source_order(),
            bridge.endpoints(),
            request,
        )?;
    }
    for ring in request.spec.rings() {
        for bond in ring.bonds_in_canonical_cycle_order() {
            let fact = request.spec.ring_bonds().get(bond).ok_or_else(missing)?;
            if fact.style() == DirectGlycosidicHaworthBondStyleV1::Q1
                && fact.haworth_position() == DirectGlycosidicHaworthPositionV1::Front
            {
                let geometry = endpoints(request, fact.endpoints())?;
                let source_order = source_order(fact.source_order())?;
                let endpoints = padded_direct_q(geometry, request.wedge_width())?;
                push_unique(&mut seen, fact.bond())?;
                operations.push(DirectGlycosidicHaworthDrawOpV1::HaworthFrontStroke {
                    bond: fact.bond().clone(),
                    source_order,
                    endpoints,
                    width: request.wedge_width(),
                });
            }
        }
    }
    for ring in request.spec.rings() {
        for bond in ring.bonds_in_canonical_cycle_order() {
            let fact = request.spec.ring_bonds().get(bond).ok_or_else(missing)?;
            if fact.style() == DirectGlycosidicHaworthBondStyleV1::W1
                && fact.haworth_position() == DirectGlycosidicHaworthPositionV1::Front
            {
                let [tip, base] = endpoints(request, fact.endpoints())?;
                let source_order = source_order(fact.source_order())?;
                let base = extended_direct_base(tip, base, request.wedge_width())?;
                let commands =
                    rounded_direct_wedge(tip, base, request.line_width(), request.wedge_width())?;
                push_unique(&mut seen, fact.bond())?;
                operations.push(DirectGlycosidicHaworthDrawOpV1::RoundedFrontWedge {
                    bond: fact.bond().clone(),
                    source_order,
                    tip,
                    base,
                    width: request.wedge_width(),
                    commands,
                });
            }
        }
    }
    let expected: BTreeSet<_> = request
        .spec
        .ring_bonds()
        .keys()
        .chain(request.spec.bridge_bonds().keys())
        .cloned()
        .collect();
    if seen != expected || operations.len() != expected.len() || operations.len() > MAX_TARGETS {
        return Err(RenderError::InvalidRequest(
            "direct profile facts do not partition selected bonds".to_owned(),
        ));
    }
    Ok(DirectGlycosidicHaworthRenderPlanV1 {
        provenance: request.provenance(),
        paint: request.paint().clone(),
        operations,
    })
}

fn missing() -> RenderError {
    RenderError::InvalidRequest("direct depiction receipt is incomplete".to_owned())
}
fn push_unique(seen: &mut BTreeSet<RecordId>, bond: &RecordId) -> Result<(), RenderError> {
    if seen.insert(bond.clone()) {
        Ok(())
    } else {
        Err(RenderError::InvalidRequest(
            "direct profile repeats a selected bond".to_owned(),
        ))
    }
}
fn source_order(value: usize) -> Result<u32, RenderError> {
    u32::try_from(value).map_err(|_| {
        RenderError::InvalidRequest("direct profile source order exceeds u32".to_owned())
    })
}
fn point(value: HaworthPoint) -> Result<RenderPoint, RenderError> {
    RenderPoint::new(value.x, value.y)
}
fn endpoints(
    request: &DirectGlycosidicHaworthRenderRequestV1,
    ids: &[RecordId; 2],
) -> Result<[RenderPoint; 2], RenderError> {
    Ok([
        point(
            *request
                .spec
                .coordinates()
                .get(&ids[0])
                .ok_or_else(missing)?,
        )?,
        point(
            *request
                .spec
                .coordinates()
                .get(&ids[1])
                .ok_or_else(missing)?,
        )?,
    ])
}
fn push_ordinary(
    operations: &mut Vec<DirectGlycosidicHaworthDrawOpV1>,
    seen: &mut BTreeSet<RecordId>,
    bond: &RecordId,
    order: usize,
    ids: &[RecordId; 2],
    request: &DirectGlycosidicHaworthRenderRequestV1,
) -> Result<(), RenderError> {
    push_unique(seen, bond)?;
    operations.push(DirectGlycosidicHaworthDrawOpV1::OrdinaryLine {
        bond: bond.clone(),
        source_order: source_order(order)?,
        endpoints: endpoints(request, ids)?,
        width: request.line_width(),
    });
    Ok(())
}
fn unit(a: RenderPoint, b: RenderPoint) -> Result<(f64, f64, f64), RenderError> {
    let dx = b.x() - a.x();
    let dy = b.y() - a.y();
    let length = dx.hypot(dy);
    if !length.is_finite() || length <= 0.0 {
        return Err(RenderError::InvalidRequest(
            "direct profile bond geometry must have finite nonzero length".to_owned(),
        ));
    }
    Ok((dx / length, dy / length, length))
}
fn shifted(point: RenderPoint, dx: f64, dy: f64) -> Result<RenderPoint, RenderError> {
    RenderPoint::new(point.x() + dx, point.y() + dy)
}
pub(crate) fn padded_direct_q(
    points: [RenderPoint; 2],
    width: PositiveFinite,
) -> Result<[RenderPoint; 2], RenderError> {
    let (x, y, _) = unit(points[0], points[1])?;
    let front_pad = FRONT_PAD_RATIO * width.get();
    let compensation = width.get() / 2.0;
    let extension = front_pad - compensation;
    Ok([
        shifted(points[0], -x * extension, -y * extension)?,
        shifted(points[1], x * extension, y * extension)?,
    ])
}
pub(crate) fn extended_direct_base(
    tip: RenderPoint,
    base: RenderPoint,
    width: PositiveFinite,
) -> Result<RenderPoint, RenderError> {
    let (x, y, _) = unit(tip, base)?;
    shifted(
        base,
        x * OVERLAP_RATIO * width.get(),
        y * OVERLAP_RATIO * width.get(),
    )
}

pub(crate) fn rounded_direct_wedge(
    tip: RenderPoint,
    base: RenderPoint,
    narrow: PositiveFinite,
    wide: PositiveFinite,
) -> Result<Vec<DirectGlycosidicHaworthPathCommandV1>, RenderError> {
    let (x, y, _) = unit(tip, base)?;
    let (px, py) = (-y, x);
    let narrow_half = narrow.get() / 2.0;
    let wide_half = wide.get() / 2.0;
    let nl = shifted(tip, px * narrow_half, py * narrow_half)?;
    let nr = shifted(tip, -px * narrow_half, -py * narrow_half)?;
    let wl = shifted(base, px * wide_half, py * wide_half)?;
    let wr = shifted(base, -px * wide_half, -py * wide_half)?;
    // This is the upstream helper's independent left/right effective-radius limit.
    let requested = wide.get() * 0.25;
    let base_limit = wide.get() / 2.0;
    let left_side = normalized(wl, nl)?;
    let base_dir = normalized(wl, wr)?;
    let right_side = normalized(wr, nr)?;
    let left_limit = base_limit.min(distance(wl, nl)?);
    let right_limit = base_limit.min(distance(wr, nr)?);
    let left_max = max_corner_radius(angle_between(left_side, base_dir)?, left_limit)?;
    let right_max = max_corner_radius(
        angle_between((-base_dir.0, -base_dir.1), right_side)?,
        right_limit,
    )?;
    let radius = requested.min(base_limit).min(left_max).min(right_max);
    if !radius.is_finite() || radius <= 0.0 {
        return Err(RenderError::InvalidRequest(
            "direct rounded wedge has no usable corner radius".to_owned(),
        ));
    }
    let left = fillet(wl, left_side, base_dir, radius)?;
    let right = fillet(wr, (-base_dir.0, -base_dir.1), right_side, radius)?;
    let mut commands = Vec::new();
    commands
        .try_reserve(10)
        .map_err(|_| RenderError::ResourceExhausted)?;
    commands.push(DirectGlycosidicHaworthPathCommandV1::MoveTo(nl));
    commands.push(DirectGlycosidicHaworthPathCommandV1::LineTo(left.0));
    append_arc(&mut commands, left.2, left.0, left.1, left.3, left.4)?;
    commands.push(DirectGlycosidicHaworthPathCommandV1::LineTo(right.0));
    append_arc(&mut commands, right.2, right.0, right.1, right.3, right.4)?;
    commands.push(DirectGlycosidicHaworthPathCommandV1::LineTo(nr));
    commands.push(DirectGlycosidicHaworthPathCommandV1::Close);
    Ok(commands)
}
fn normalized(a: RenderPoint, b: RenderPoint) -> Result<(f64, f64), RenderError> {
    let (x, y, _) = unit(a, b)?;
    Ok((x, y))
}
fn distance(a: RenderPoint, b: RenderPoint) -> Result<f64, RenderError> {
    Ok(unit(a, b)?.2)
}
fn angle_between(left: (f64, f64), right: (f64, f64)) -> Result<f64, RenderError> {
    let value = (left.0 * right.0 + left.1 * right.1)
        .clamp(-1.0, 1.0)
        .acos();
    if value.is_finite() {
        Ok(value)
    } else {
        Err(RenderError::InvalidRequest(
            "direct rounded wedge has invalid corner angle".to_owned(),
        ))
    }
}
fn max_corner_radius(angle: f64, edge_limit: f64) -> Result<f64, RenderError> {
    if angle <= 0.0 {
        return Ok(0.0);
    }
    let tangent = (angle / 2.0).tan();
    let result = if tangent <= 0.0 {
        0.0
    } else {
        edge_limit * tangent
    };
    if result.is_finite() {
        Ok(result)
    } else {
        Err(RenderError::InvalidRequest(
            "direct rounded wedge has invalid corner radius".to_owned(),
        ))
    }
}
fn fillet(
    corner: RenderPoint,
    d1: (f64, f64),
    d2: (f64, f64),
    radius: f64,
) -> Result<(RenderPoint, RenderPoint, RenderPoint, f64, f64), RenderError> {
    let dot = (d1.0 * d2.0 + d1.1 * d2.1).clamp(-1.0, 1.0);
    let angle = dot.acos();
    let offset = radius / (angle / 2.0).tan();
    let bisector = normalized_pair(d1.0 + d2.0, d1.1 + d2.1)?;
    let center = shifted(
        corner,
        bisector.0 * radius / (angle / 2.0).sin(),
        bisector.1 * radius / (angle / 2.0).sin(),
    )?;
    let first = shifted(corner, d1.0 * offset, d1.1 * offset)?;
    let second = shifted(corner, d2.0 * offset, d2.1 * offset)?;
    let start = (first.y() - center.y()).atan2(first.x() - center.x());
    let end = (second.y() - center.y()).atan2(second.x() - center.x());
    let delta = normalized_angle(end - start);
    Ok((first, second, center, radius, start + delta))
}
fn normalized_pair(x: f64, y: f64) -> Result<(f64, f64), RenderError> {
    let l = x.hypot(y);
    if !l.is_finite() || l <= 0.0 {
        Err(RenderError::InvalidRequest(
            "direct rounded wedge has invalid corner geometry".to_owned(),
        ))
    } else {
        Ok((x / l, y / l))
    }
}
fn normalized_angle(mut value: f64) -> f64 {
    while value <= -std::f64::consts::PI {
        value += std::f64::consts::TAU;
    }
    while value > std::f64::consts::PI {
        value -= std::f64::consts::TAU;
    }
    value
}
fn append_arc(
    commands: &mut Vec<DirectGlycosidicHaworthPathCommandV1>,
    center: RenderPoint,
    start: RenderPoint,
    end: RenderPoint,
    radius: f64,
    end_angle: f64,
) -> Result<(), RenderError> {
    let start_angle = (start.y() - center.y()).atan2(start.x() - center.x());
    let delta = normalized_angle(end_angle - start_angle);
    let pieces = (delta.abs() / CUBIC_ARC_LIMIT).ceil().max(1.0) as usize;
    commands
        .try_reserve(pieces)
        .map_err(|_| RenderError::ResourceExhausted)?;
    for index in 0..pieces {
        let a = start_angle + delta * index as f64 / pieces as f64;
        let b = start_angle + delta * (index + 1) as f64 / pieces as f64;
        let k = 4.0 / 3.0 * ((b - a) / 4.0).tan();
        let p1 = RenderPoint::new(
            center.x() + radius * (a.cos() - k * a.sin()),
            center.y() + radius * (a.sin() + k * a.cos()),
        )?;
        let p2 = RenderPoint::new(
            center.x() + radius * (b.cos() + k * b.sin()),
            center.y() + radius * (b.sin() - k * b.cos()),
        )?;
        let p3 = if index + 1 == pieces {
            end
        } else {
            RenderPoint::new(center.x() + radius * b.cos(), center.y() + radius * b.sin())?
        };
        commands.push(DirectGlycosidicHaworthPathCommandV1::CubicTo {
            control_1: p1,
            control_2: p2,
            end: p3,
        });
    }
    Ok(())
}
