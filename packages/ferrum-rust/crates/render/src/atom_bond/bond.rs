//! Bond target validation, clipping geometry, and stereo lowering.

use std::collections::HashMap;

use ferrum_core::{RecordId, RecordKind};
use ferrum_geometry::Vector2;

use crate::bond_presentation_geometry;
use crate::bond_style::BondStyle;
use crate::directed_stereo_bond::directed_stereo_operations;
use crate::glyph_metrics::GlyphBounds;
use crate::haworth_front_bond::{HaworthFrontBondInput, build_haworth_front_batch};
use crate::render_target::RenderPlanEntryContextV1;
use crate::{
    BondRenderBatchV1, DoubleBondCarrierMarkDirectionV1, DoubleBondCarrierMarkOp, LineOp,
    PositiveFinite, RenderBatchV4, RenderError, RenderIssueKind, RenderOp, RenderPaintV3,
    RenderTarget,
};

use super::{
    BondInkClearance, EndpointClipGeometry, RenderEndpointGeometry, TargetVisibility,
    geometry_to_render_point,
};

/// Exact final-normal-single endpoint clipping owned by bond lowering.
///
/// Attached compact-group pose admission uses this same value so a prepared
/// pose is admitted against the final painted normal-bond envelope, not a
/// weaker pre-lowering approximation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NormalBondEndpointClipPolicy {
    clearance: BondInkClearance,
    footprint: BondInkFootprint,
}

impl NormalBondEndpointClipPolicy {
    /// Resolve the normal-single final-ink envelope from depiction facts.
    ///
    /// This is the sole renderer owner of the font-derived label clearance.
    pub(crate) fn from_depiction(
        stroke_width: PositiveFinite,
        font_size: PositiveFinite,
    ) -> Result<Self, RenderIssueKind> {
        let clearance =
            BondInkClearance::new(PositiveFinite::new(font_size.get() * 0.125).map_err(|_| {
                RenderIssueKind::UnrenderableTarget {
                    reason: "normal bond label clearance is not representable".to_owned(),
                }
            })?);
        Ok(Self {
            clearance,
            footprint: final_ink_footprint(&BondStyle::NormalSingle, stroke_width, stroke_width)?,
        })
    }

    pub(crate) const fn clearance(self) -> BondInkClearance {
        self.clearance
    }

    #[cfg(test)]
    pub(crate) fn from_test_facts(
        stroke_width: PositiveFinite,
        clearance: BondInkClearance,
    ) -> Result<Self, RenderIssueKind> {
        Ok(Self {
            clearance,
            footprint: final_ink_footprint(&BondStyle::NormalSingle, stroke_width, stroke_width)?,
        })
    }

    pub(crate) fn atom_label_forward_exit_distance(
        self,
        bounds: GlyphBounds,
        direction: Vector2,
    ) -> Result<f64, RenderIssueKind> {
        self.endpoint_clip_distance(
            &RenderEndpointGeometry {
                kind: RecordKind::Atom,
                position: ferrum_geometry::Point2::new(0.0, 0.0).map_err(|error| {
                    RenderIssueKind::UnrenderableTarget {
                        reason: format!(
                            "atom label clipping position is not representable: {error}"
                        ),
                    }
                })?,
                clipping: EndpointClipGeometry::AtomLabelInk(bounds),
            },
            direction,
            Vector2::new(0.0, 0.0).map_err(|error| RenderIssueKind::UnrenderableTarget {
                reason: format!("atom label clipping origin is not representable: {error}"),
            })?,
        )
    }

    pub(crate) fn has_positive_visible_segment(
        self,
        center_distance: f64,
        first_clip: f64,
        second_clip: f64,
    ) -> bool {
        normal_bond_has_positive_visible_segment(center_distance, first_clip, second_clip)
    }

    fn endpoint_clip_distance(
        self,
        endpoint: &RenderEndpointGeometry,
        direction: Vector2,
        local_offset: Vector2,
    ) -> Result<f64, RenderIssueKind> {
        endpoint_clip_distance_with_footprint(
            endpoint,
            direction,
            local_offset,
            self.clearance,
            self.footprint,
        )
    }
}

/// A bond with explicit endpoint atom identities and source style facts.
#[derive(Clone, Debug, PartialEq)]
pub struct BondRenderTarget {
    pub(super) context: RenderPlanEntryContextV1,
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
    pub(super) paint: RenderPaintV3,
}
#[derive(Clone, Debug, PartialEq)]
struct CarrierMarkRenderFact {
    shared_endpoint_is_start: bool,
    direction: DoubleBondCarrierMarkDirectionV1,
    central_double_bond: RecordId,
}
impl BondRenderTarget {
    /// Construct a valid bond target for this render slice.
    pub(crate) fn new(
        context: RenderPlanEntryContextV1,
        first_atom: RecordId,
        second_atom: RecordId,
        style: BondStyle,
        visibility: TargetVisibility,
    ) -> Result<Self, RenderError> {
        if context.record_id().kind() != RecordKind::Bond {
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
            context,
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
        self.context.target()
    }

    pub(super) const fn context(&self) -> &RenderPlanEntryContextV1 {
        &self.context
    }

    pub(super) const fn endpoints(&self) -> (&RecordId, &RecordId) {
        (&self.first_endpoint, &self.second_endpoint)
    }

    /// Attach source-resolved stroke and parallel-lane facts for this bond only.
    #[must_use]
    pub fn with_appearance(
        mut self,
        stroke_width: PositiveFinite,
        lane_spacing: PositiveFinite,
        wedge_width: PositiveFinite,
        paint: RenderPaintV3,
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
    resolved_normal_single_policy: NormalBondEndpointClipPolicy,
    paint: RenderPaintV3,
) -> Result<RenderBatchV4, RenderIssueKind> {
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
        | BondStyle::HaworthFrontWedge
        | BondStyle::Bold
        | BondStyle::Dashed
        | BondStyle::Wavy => &[],
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
    let normal_single_policy =
        matches!(bond.style, BondStyle::NormalSingle).then_some(resolved_normal_single_policy);
    let footprint = normal_single_policy.map_or_else(
        || final_ink_footprint(&bond.style, stroke_width, wedge_width),
        |policy| Ok(policy.footprint),
    )?;
    let clip = BondClipConfiguration {
        clearance: resolved_normal_single_policy.clearance(),
        footprint,
        normal_single_policy,
    };
    if matches!(bond.style, BondStyle::SolidWedge | BondStyle::HashedWedge) {
        return build_directed_stereo_batch(
            bond,
            &line_context,
            stroke_width,
            wedge_width,
            clip,
            paint,
        );
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
            clip,
            paint,
        );
    }
    if matches!(
        bond.style,
        BondStyle::Bold | BondStyle::Dashed | BondStyle::Wavy
    ) {
        let axis = build_bond_line(&line_context, 0.0, stroke_width, clip, paint, 10)?;
        let operations = match &bond.style {
            BondStyle::Bold => bond_presentation_geometry::bold(axis),
            BondStyle::Dashed => bond_presentation_geometry::dashed(axis),
            BondStyle::Wavy => bond_presentation_geometry::wavy(axis),
            _ => unreachable!("styled bond branch admits only styled single-bond presentations"),
        }?;
        return RenderBatchV4::bond(
            bond.context.clone(),
            BondRenderBatchV1::from_render_operations(operations).map_err(|error| {
                RenderIssueKind::UnrenderableTarget {
                    reason: error.to_string(),
                }
            })?,
        )
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: format!("styled bond batch is not renderable: {error}"),
        });
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
            clip,
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
    RenderBatchV4::bond(
        bond.context.clone(),
        BondRenderBatchV1::from_render_operations(operations).map_err(|error| {
            RenderIssueKind::UnrenderableTarget {
                reason: error.to_string(),
            }
        })?,
    )
    .map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("bond batch is not renderable: {error}"),
    })
}

fn build_haworth_front_bond_batch(
    bond: &BondRenderTarget,
    context: &BondLineContext<'_>,
    stroke_width: PositiveFinite,
    wedge_width: PositiveFinite,
    clip: BondClipConfiguration,
    paint: RenderPaintV3,
) -> Result<RenderBatchV4, RenderIssueKind> {
    let center = build_bond_line(context, 0.0, stroke_width, clip, paint.clone(), 10)?;
    build_haworth_front_batch(HaworthFrontBondInput {
        target: bond.context.target().clone(),
        paint_order: bond.context.paint_order(),
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
    clip: BondClipConfiguration,
    paint: RenderPaintV3,
) -> Result<RenderBatchV4, RenderIssueKind> {
    let center = build_bond_line(context, 0.0, stroke_width, clip, paint.clone(), 10)?;
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
    RenderBatchV4::bond(
        bond.context.clone(),
        BondRenderBatchV1::from_render_operations(operations).map_err(|error| {
            RenderIssueKind::UnrenderableTarget {
                reason: error.to_string(),
            }
        })?,
    )
    .map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("directed bond batch is not renderable: {error}"),
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
    clip: BondClipConfiguration,
    paint: RenderPaintV3,
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
    let first_clip = clip.endpoint_clip_distance(context.first, context.direction, local_offset)?;
    let second_clip = clip.endpoint_clip_distance(context.second, reverse, local_offset)?;
    let has_positive_segment = clip.normal_single_policy.map_or_else(
        || normal_bond_has_positive_visible_segment(context.length, first_clip, second_clip),
        |policy| policy.has_positive_visible_segment(context.length, first_clip, second_clip),
    );
    if !has_positive_segment {
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

/// Return whether a normal bond retains a strictly positive visible segment.
fn normal_bond_has_positive_visible_segment(
    center_distance: f64,
    first_clip: f64,
    second_clip: f64,
) -> bool {
    let remaining_length = center_distance - first_clip - second_clip;
    remaining_length.is_finite() && remaining_length > 0.0
}

#[derive(Clone, Copy)]
struct BondClipConfiguration {
    clearance: BondInkClearance,
    footprint: BondInkFootprint,
    normal_single_policy: Option<NormalBondEndpointClipPolicy>,
}

impl BondClipConfiguration {
    fn endpoint_clip_distance(
        self,
        endpoint: &RenderEndpointGeometry,
        direction: Vector2,
        local_offset: Vector2,
    ) -> Result<f64, RenderIssueKind> {
        self.normal_single_policy.map_or_else(
            || {
                endpoint_clip_distance_with_footprint(
                    endpoint,
                    direction,
                    local_offset,
                    self.clearance,
                    self.footprint,
                )
            },
            |policy| policy.endpoint_clip_distance(endpoint, direction, local_offset),
        )
    }
}

fn endpoint_clip_distance_with_footprint(
    endpoint: &RenderEndpointGeometry,
    direction: Vector2,
    local_offset: Vector2,
    clearance: BondInkClearance,
    footprint: BondInkFootprint,
) -> Result<f64, RenderIssueKind> {
    match &endpoint.clipping {
        EndpointClipGeometry::AtomLabelInk(bounds) => clip_glyph_distance(
            inflate_glyph_bounds(
                *bounds,
                clearance.gap().get() + footprint.transverse_radius + footprint.axial_overhang,
            )?,
            direction,
            local_offset,
        ),
        EndpointClipGeometry::FixedConnectionPoint {
            label_ink_exclusion,
        } => {
            let origin = ferrum_geometry::Point2::new(
                endpoint.position.x() + local_offset.x(),
                endpoint.position.y() + local_offset.y(),
            )
            .map_err(|error| RenderIssueKind::UnrenderableTarget {
                reason: format!("compact-group endpoint offset is not representable: {error}"),
            })?;
            if label_ink_exclusion.ray_enters_interior(origin, direction) {
                return Err(RenderIssueKind::UnrenderableTarget {
                    reason: "compact-group exterior bond approaches through label ink".to_owned(),
                });
            }
            Ok(0.0)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct BondInkFootprint {
    transverse_radius: f64,
    axial_overhang: f64,
}

fn final_ink_footprint(
    style: &BondStyle,
    stroke_width: PositiveFinite,
    wedge_width: PositiveFinite,
) -> Result<BondInkFootprint, RenderIssueKind> {
    let transverse_radius = match style {
        BondStyle::Bold => stroke_width.get(),
        BondStyle::Wavy => stroke_width.get() * 2.5,
        BondStyle::SolidWedge
        | BondStyle::HashedWedge
        | BondStyle::HaworthFrontStroke
        | BondStyle::HaworthFrontWedge => wedge_width.get() / 2.0,
        _ => stroke_width.get() / 2.0,
    };
    let axial_overhang = match style {
        // q1 pads its emitted centerline 0.35w toward each label after the
        // shared axis has been clipped. Its round cap is already captured by
        // the transverse radius, while this distinct fact reserves the pad.
        BondStyle::HaworthFrontStroke => wedge_width.get() * 0.35,
        _ => 0.0,
    };
    if transverse_radius.is_finite()
        && transverse_radius >= 0.0
        && axial_overhang.is_finite()
        && axial_overhang >= 0.0
    {
        Ok(BondInkFootprint {
            transverse_radius,
            axial_overhang,
        })
    } else {
        Err(RenderIssueKind::UnrenderableTarget {
            reason: "final bond ink footprint is not representable".to_owned(),
        })
    }
}

fn inflate_glyph_bounds(bounds: GlyphBounds, amount: f64) -> Result<GlyphBounds, RenderIssueKind> {
    if !amount.is_finite() || amount < 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "bond ink clearance is not representable".to_owned(),
        });
    }
    GlyphBounds::new(
        bounds.min_x() - amount,
        bounds.min_y() - amount,
        bounds.max_x() + amount,
        bounds.max_y() + amount,
    )
    .map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("inflated atom-label ink is not representable: {error}"),
    })
}

fn clip_glyph_distance(
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
