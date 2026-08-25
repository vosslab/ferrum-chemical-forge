//! Bond target validation, clipping geometry, and stereo lowering.

use std::collections::HashMap;

use ferrum_core::{RecordId, RecordKind};
use ferrum_geometry::Vector2;

use crate::bond_style::BondStyle;
use crate::directed_stereo_bond::directed_stereo_operations;
use crate::haworth_front_bond::{HaworthFrontBondInput, build_haworth_front_batch};
use crate::{
    BatchSpace, DoubleBondCarrierMarkDirectionV1, DoubleBondCarrierMarkOp, GlyphBounds, LineOp,
    Paint, PositiveFinite, RenderBatch, RenderError, RenderIssueKind, RenderOp, RenderTarget,
};

use super::{RenderEndpointGeometry, TargetVisibility, geometry_to_render_point};

/// A bond with explicit endpoint atom identities and source style facts.
#[derive(Clone, Debug, PartialEq)]
pub struct BondRenderTarget {
    pub(super) target: RenderTarget,
    first_endpoint: RecordId,
    second_endpoint: RecordId,
    pub(super) style: BondStyle,
    pub(super) visibility: TargetVisibility,
    pub(super) appearance: Option<BondLineAppearance>,
    carrier_marks: Vec<CarrierMarkRenderFact>,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct BondLineAppearance {
    pub(super) stroke_width: PositiveFinite,
    pub(super) lane_spacing: PositiveFinite,
    pub(super) wedge_width: PositiveFinite,
    pub(super) paint: Paint,
}
#[derive(Clone, Debug, PartialEq)]
struct CarrierMarkRenderFact {
    shared_endpoint_is_start: bool,
    direction: DoubleBondCarrierMarkDirectionV1,
    central_double_bond: RecordId,
}
impl BondRenderTarget {
    /// Construct a valid bond target for this render slice.
    pub fn new(
        target: RenderTarget,
        first_atom: RecordId,
        second_atom: RecordId,
        style: BondStyle,
        visibility: TargetVisibility,
    ) -> Result<Self, RenderError> {
        if target.record_id().kind() != RecordKind::Bond {
            return Err(RenderError::InvalidRequest(
                "bond render target requires a bond RecordId".to_owned(),
            ));
        }
        if !matches!(first_atom.kind(), RecordKind::Atom | RecordKind::Group)
            || !matches!(second_atom.kind(), RecordKind::Atom | RecordKind::Group)
        {
            return Err(RenderError::InvalidRequest(
                "bond endpoints require atom or compact-group RecordIds".to_owned(),
            ));
        }
        Ok(Self {
            target,
            first_endpoint: first_atom,
            second_endpoint: second_atom,
            style,
            visibility,
            appearance: None,
            carrier_marks: Vec::new(),
        })
    }

    /// Return the durable target and source order.
    #[must_use]
    pub fn target(&self) -> &RenderTarget {
        &self.target
    }

    /// Attach source-resolved stroke and parallel-lane facts for this bond only.
    #[must_use]
    pub fn with_appearance(
        mut self,
        stroke_width: PositiveFinite,
        lane_spacing: PositiveFinite,
        wedge_width: PositiveFinite,
        paint: Paint,
    ) -> Self {
        self.appearance = Some(BondLineAppearance {
            stroke_width,
            lane_spacing,
            wedge_width,
            paint,
        });
        self
    }

    /// Attach an explicit E/Z carrier mark without changing bond connectivity.
    pub fn with_double_bond_carrier_mark(
        mut self,
        direction: DoubleBondCarrierMarkDirectionV1,
        shared_endpoint_is_start: bool,
        central_double_bond: RecordId,
    ) -> Result<Self, RenderError> {
        if self.style != BondStyle::NormalSingle {
            return Err(RenderError::InvalidRequest(
                "E/Z carrier mark requires one ordinary single-bond carrier".to_owned(),
            ));
        }
        self.carrier_marks.push(CarrierMarkRenderFact {
            shared_endpoint_is_start,
            direction,
            central_double_bond,
        });
        Ok(self)
    }
}

pub(super) fn build_bond_batch(
    bond: &BondRenderTarget,
    endpoints: &HashMap<RecordId, RenderEndpointGeometry>,
    stroke_width: PositiveFinite,
    lane_spacing: PositiveFinite,
    wedge_width: PositiveFinite,
    paint: Paint,
) -> Result<RenderBatch, RenderIssueKind> {
    let Some(first) = endpoints.get(&bond.first_endpoint) else {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "first bond endpoint has no renderable geometry".to_owned(),
        });
    };
    let Some(second) = endpoints.get(&bond.second_endpoint) else {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "second bond endpoint has no renderable geometry".to_owned(),
        });
    };
    let has_group = first.kind == RecordKind::Group || second.kind == RecordKind::Group;
    if first.kind == RecordKind::Group && second.kind == RecordKind::Group {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "compact-group to compact-group bonds have no V1 exterior geometry".to_owned(),
        });
    }
    if has_group && !matches!(bond.style, BondStyle::NormalSingle) {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "compact-group exterior bonds require the normal single style".to_owned(),
        });
    }
    let vector = second.position - first.position;
    let length = vector.length();
    if !length.is_finite() || length == 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "bond endpoints are coincident or not representable".to_owned(),
        });
    }
    let direction = Vector2::new(vector.x() / length, vector.y() / length).map_err(|error| {
        RenderIssueKind::UnrenderableTarget {
            reason: format!("bond direction is not representable: {error}"),
        }
    })?;
    // Retain the established CDML depiction convention: `bond_width` is the
    // centered double-lane separation, while triple outer lanes use 70% of it.
    const TRIPLE_OUTER_LANE_FACTOR: f64 = 0.7;
    let offsets: &[f64] = match &bond.style {
        BondStyle::NormalSingle
        | BondStyle::DoubleBondCarrierUp
        | BondStyle::DoubleBondCarrierDown => &[0.0],
        BondStyle::Double => &[-0.5, 0.5],
        BondStyle::Triple => &[-TRIPLE_OUTER_LANE_FACTOR, 0.0, TRIPLE_OUTER_LANE_FACTOR],
        BondStyle::SolidWedge
        | BondStyle::HashedWedge
        | BondStyle::HaworthFrontStroke
        | BondStyle::HaworthFrontWedge => &[],
        _ => unreachable!("unsupported styles are excluded before bond geometry"),
    };
    let perpendicular = direction.perpendicular_left();
    let line_context = BondLineContext {
        first,
        second,
        direction,
        perpendicular,
        length,
    };
    if matches!(bond.style, BondStyle::SolidWedge | BondStyle::HashedWedge) {
        return build_directed_stereo_batch(bond, &line_context, stroke_width, wedge_width, paint);
    }
    if matches!(
        bond.style,
        BondStyle::HaworthFrontStroke | BondStyle::HaworthFrontWedge
    ) {
        return build_haworth_front_bond_batch(
            bond,
            &line_context,
            stroke_width,
            wedge_width,
            paint,
        );
    }
    let mut operations = Vec::with_capacity(offsets.len());
    for (index, factor) in offsets.iter().enumerate() {
        let offset = lane_spacing.get() * *factor;
        if !offset.is_finite() {
            return Err(RenderIssueKind::UnrenderableTarget {
                reason: "bond line spacing is not representable".to_owned(),
            });
        }
        let line = build_bond_line(
            &line_context,
            offset,
            stroke_width,
            paint.clone(),
            10 + i32::try_from(index).expect("bond line count fits i32"),
        )?;
        operations.push(RenderOp::Line(line));
    }
    for (index, mark) in bond.carrier_marks.iter().enumerate() {
        let RenderOp::Line(carrier_line) = &operations[0] else {
            unreachable!("ordinary carrier bonds begin with their base line")
        };
        let operation = DoubleBondCarrierMarkOp::from_carrier_line(
            carrier_line,
            mark.shared_endpoint_is_start,
            mark.direction,
            mark.central_double_bond.clone(),
            11 + i32::try_from(index).expect("carrier mark count fits i32"),
        )
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: error.to_string(),
        })?;
        operations.push(RenderOp::DoubleBondCarrierMark(operation));
    }
    RenderBatch::new(bond.target.clone(), BatchSpace::Scene, operations).map_err(|error| {
        RenderIssueKind::UnrenderableTarget {
            reason: format!("bond batch is not renderable: {error}"),
        }
    })
}

fn build_haworth_front_bond_batch(
    bond: &BondRenderTarget,
    context: &BondLineContext<'_>,
    stroke_width: PositiveFinite,
    wedge_width: PositiveFinite,
    paint: Paint,
) -> Result<RenderBatch, RenderIssueKind> {
    let center = build_bond_line(context, 0.0, stroke_width, paint.clone(), 10)?;
    build_haworth_front_batch(HaworthFrontBondInput {
        target: bond.target.clone(),
        style: bond.style.clone(),
        tip: center.start(),
        base: center.end(),
        perpendicular: context.perpendicular,
        stroke_width,
        wedge_width,
        paint,
    })
}

fn build_directed_stereo_batch(
    bond: &BondRenderTarget,
    context: &BondLineContext<'_>,
    stroke_width: PositiveFinite,
    wedge_width: PositiveFinite,
    paint: Paint,
) -> Result<RenderBatch, RenderIssueKind> {
    let center = build_bond_line(context, 0.0, stroke_width, paint.clone(), 10)?;
    let tip = center.start();
    let base = center.end();
    let operations = directed_stereo_operations(
        bond.style.clone(),
        tip,
        base,
        context.perpendicular,
        stroke_width,
        wedge_width,
        paint,
    )?;
    RenderBatch::new(bond.target.clone(), BatchSpace::Scene, operations).map_err(|error| {
        RenderIssueKind::UnrenderableTarget {
            reason: format!("directed bond batch is not renderable: {error}"),
        }
    })
}

struct BondLineContext<'a> {
    first: &'a RenderEndpointGeometry,
    second: &'a RenderEndpointGeometry,
    direction: Vector2,
    perpendicular: Vector2,
    length: f64,
}

fn build_bond_line(
    context: &BondLineContext<'_>,
    offset: f64,
    width: PositiveFinite,
    paint: Paint,
    z: i32,
) -> Result<LineOp, RenderIssueKind> {
    let local_offset = Vector2::new(
        context.perpendicular.x() * offset,
        context.perpendicular.y() * offset,
    )
    .map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("bond line offset is not representable: {error}"),
    })?;
    let reverse = negated(context.direction)?;
    let first_clip = clip_distance(context.first.bounds, context.direction, local_offset)?;
    let second_clip = clip_distance(context.second.bounds, reverse, local_offset)?;
    let remaining_length = context.length - first_clip - second_clip;
    if !remaining_length.is_finite() || remaining_length <= 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "label clipping leaves no positive visible bond segment".to_owned(),
        });
    }
    let start = context
        .first
        .position
        .offset(context.perpendicular, offset)
        .and_then(|point| point.offset(context.direction, first_clip))
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: format!("bond start is not representable: {error}"),
        })?;
    let end = context
        .second
        .position
        .offset(context.perpendicular, offset)
        .and_then(|point| point.offset(reverse, second_clip))
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: format!("bond end is not representable: {error}"),
        })?;
    LineOp::new(
        geometry_to_render_point(start)?,
        geometry_to_render_point(end)?,
        width,
        paint,
        z,
    )
    .map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("clipped bond is not renderable: {error}"),
    })
}

fn clip_distance(
    bounds: GlyphBounds,
    direction: Vector2,
    origin: Vector2,
) -> Result<f64, RenderIssueKind> {
    let x = ray_slab(bounds.min_x(), bounds.max_x(), origin.x(), direction.x());
    let y = ray_slab(bounds.min_y(), bounds.max_y(), origin.y(), direction.y());
    let near = x.0.max(y.0);
    let far = x.1.min(y.1);
    if far < near || far < 0.0 {
        return Ok(0.0);
    }
    let distance = far.max(0.0);
    if !distance.is_finite() {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "glyph clipping distance is not finite".to_owned(),
        });
    }
    Ok(distance)
}

fn ray_slab(minimum: f64, maximum: f64, origin: f64, direction: f64) -> (f64, f64) {
    if direction == 0.0 {
        return if origin < minimum || origin > maximum {
            (f64::INFINITY, f64::NEG_INFINITY)
        } else {
            (f64::NEG_INFINITY, f64::INFINITY)
        };
    }
    let first = (minimum - origin) / direction;
    let second = (maximum - origin) / direction;
    (first.min(second), first.max(second))
}

fn negated(vector: Vector2) -> Result<Vector2, RenderIssueKind> {
    Vector2::new(-vector.x(), -vector.y()).map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("bond direction is not representable: {error}"),
    })
}
