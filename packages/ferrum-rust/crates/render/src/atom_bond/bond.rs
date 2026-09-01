//! Bond target validation, clipping geometry, and stereo lowering.

mod ink;

use std::collections::HashMap;

use ferrum_core::{RecordId, RecordKind};
use ferrum_geometry::{Point2, Vector2};

use crate::bond_presentation_geometry;
use crate::bond_style::BondStyle;
use crate::directed_stereo_bond::directed_stereo_operations;
use crate::glyph_metrics::{AtomLabelAttachmentCorridor, GlyphBounds};
use crate::haworth_front_bond::{HaworthFrontBondInput, build_haworth_front_batch};
use crate::render_target::RenderPlanEntryContextV1;
use crate::{
    BondAttachmentAxisV1, BondRenderBatchV1, DoubleBondCarrierMarkDirectionV1,
    DoubleBondCarrierMarkOp, LineOp, PositiveFinite, RenderBatchV4, RenderError, RenderIssueKind,
    RenderOp, RenderPaintV3, RenderTarget,
};

use ink::{
    BondInkFootprint, EndpointBondInkFootprints, ParallelBondTerminalEnvelope, final_ink_footprint,
    final_ink_footprints,
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
    /// Resolve the sole font-derived clearance shared by all normal-bond paths.
    pub(crate) fn label_clearance_for_font(
        font_size: PositiveFinite,
    ) -> Result<BondInkClearance, RenderIssueKind> {
        const LABEL_GAP_FONT_FACTOR: f64 = 0.0625;
        let gap = PositiveFinite::new(font_size.get() * LABEL_GAP_FONT_FACTOR).map_err(|_| {
            RenderIssueKind::UnrenderableTarget {
                reason: "normal bond label clearance is not representable".to_owned(),
            }
        })?;
        Ok(BondInkClearance::new(gap))
    }

    /// Resolve the normal-single final-ink envelope from depiction facts.
    ///
    /// This is the sole renderer owner of the font-derived label clearance.
    pub(crate) fn from_depiction(
        stroke_width: PositiveFinite,
        font_size: PositiveFinite,
    ) -> Result<Self, RenderIssueKind> {
        // Ordinary lines have butt caps, so their transverse half-width is not
        // an axial reserve. This font-relative gap therefore owns the complete
        // core-to-final-ink separation at a normal endpoint. It is also used
        // for label envelopes and compact-group admission.
        let clearance = Self::label_clearance_for_font(font_size)?;
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
        let distance = glyph_bounds_directional_extent(bounds, direction)
            + self
                .footprint
                .axial_clip_reserve(self.clearance.gap().get())?;
        validate_clip_distance(distance)
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

#[derive(Clone)]
pub(super) struct ResolvedBondLineAppearance {
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
    pub(super) const fn first_endpoint(&self) -> &RecordId {
        &self.first_endpoint
    }

    pub(super) const fn second_endpoint(&self) -> &RecordId {
        &self.second_endpoint
    }

    pub(super) fn resolved_appearance(
        &self,
        stroke_width: PositiveFinite,
        lane_spacing: PositiveFinite,
        wedge_width: PositiveFinite,
        paint: &RenderPaintV3,
    ) -> ResolvedBondLineAppearance {
        self.appearance.as_ref().map_or_else(
            || ResolvedBondLineAppearance {
                stroke_width,
                lane_spacing,
                wedge_width,
                paint: paint.clone(),
            },
            |appearance| ResolvedBondLineAppearance {
                stroke_width: appearance.stroke_width,
                lane_spacing: appearance.lane_spacing,
                wedge_width: appearance.wedge_width,
                paint: appearance.paint.clone(),
            },
        )
    }

    pub(super) fn attachment_corridor(
        &self,
        direction: Vector2,
        endpoint_is_first: bool,
        appearance: &ResolvedBondLineAppearance,
        normal_single_policy: NormalBondEndpointClipPolicy,
    ) -> Result<AtomLabelAttachmentCorridor, RenderError> {
        let normal_single = matches!(self.style, BondStyle::NormalSingle);
        let footprints = if normal_single {
            EndpointBondInkFootprints::symmetric(normal_single_policy.footprint)
        } else {
            final_ink_footprints(&self.style, appearance.stroke_width, appearance.wedge_width)
                .map_err(render_issue_as_invalid_request)?
        };
        let lane_offsets = bond_lane_offsets(&self.style, appearance.lane_spacing)
            .map_err(render_issue_as_invalid_request)?;
        let clearance = if lane_offsets.len() > 1 {
            ParallelBondTerminalEnvelope::from_lanes(&lane_offsets, footprints)
                .and_then(|terminal| terminal.optical_clearance(normal_single_policy.clearance()))
                .map_err(render_issue_as_invalid_request)?
        } else {
            normal_single_policy.clearance()
        };
        let endpoint_footprint = if endpoint_is_first {
            footprints.first
        } else {
            footprints.second
        };
        let mut maximum_style_half_width = footprints
            .first
            .terminal_half_width()
            .max(footprints.second.terminal_half_width())
            .max(endpoint_footprint.terminal_half_width());
        if matches!(self.style, BondStyle::Wavy) {
            maximum_style_half_width = maximum_style_half_width.max(
                bond_presentation_geometry::wavy_transverse_half_width(appearance.stroke_width)
                    .map_err(render_issue_as_invalid_request)?,
            );
        }
        let (mut transverse_minimum, mut transverse_maximum) = lane_offsets.iter().fold(
            (f64::INFINITY, f64::NEG_INFINITY),
            |(minimum, maximum), offset| {
                (
                    minimum.min(*offset - maximum_style_half_width),
                    maximum.max(*offset + maximum_style_half_width),
                )
            },
        );
        if !endpoint_is_first {
            (transverse_minimum, transverse_maximum) = (-transverse_maximum, -transverse_minimum);
        }
        let separation = normal_single_policy.clearance().gap().get() * 0.25;
        AtomLabelAttachmentCorridor::new(
            direction,
            transverse_minimum,
            transverse_maximum,
            clearance.gap(),
            PositiveFinite::new(separation)?,
        )
    }

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
    let attachment_axis = BondAttachmentAxisV1::new(
        geometry_to_render_point(first.position)?,
        geometry_to_render_point(second.position)?,
    )
    .map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: error.to_string(),
    })?;
    let vector = Vector2::new(
        attachment_axis.end().x() - attachment_axis.start().x(),
        attachment_axis.end().y() - attachment_axis.start().y(),
    )
    .map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("bond attachment axis is not representable: {error}"),
    })?;
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
    let perpendicular = direction.perpendicular_left();
    let line_context = BondLineContext {
        attachment_axis,
        first,
        second,
        direction,
        perpendicular,
        length,
    };
    let normal_single_policy =
        matches!(bond.style, BondStyle::NormalSingle).then_some(resolved_normal_single_policy);
    let footprints = normal_single_policy.map_or_else(
        || final_ink_footprints(&bond.style, stroke_width, wedge_width),
        |policy| Ok(EndpointBondInkFootprints::symmetric(policy.footprint)),
    )?;
    let clip = BondClipConfiguration {
        clearance: resolved_normal_single_policy.clearance(),
        footprints,
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
        let axis = build_bond_line(&line_context, 0.0, stroke_width, clip, None, paint, 10)?;
        let operations = match &bond.style {
            BondStyle::Bold => bond_presentation_geometry::bold(axis),
            BondStyle::Dashed => bond_presentation_geometry::dashed(axis),
            BondStyle::Wavy => bond_presentation_geometry::wavy(axis),
            _ => unreachable!("styled bond branch admits only styled single-bond presentations"),
        }?;
        return RenderBatchV4::bond(
            bond.context.clone(),
            BondRenderBatchV1::from_render_operations(attachment_axis, operations).map_err(
                |error| RenderIssueKind::UnrenderableTarget {
                    reason: error.to_string(),
                },
            )?,
        )
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: format!("styled bond batch is not renderable: {error}"),
        });
    }
    let lane_offsets = bond_lane_offsets(&bond.style, lane_spacing)?;
    // A double or triple bond is one chemical connection with a multi-lane
    // final footprint.  Clip its complete lane envelope once at each label,
    // then lower every symmetric lane from those shared axial positions.
    // Per-lane clipping permits a near lane to enter label ink while a farther
    // lane happens to stop safely, which is neither a valid final footprint
    // nor a stable attachment relation.
    let shared_parallel_clips = (lane_offsets.len() > 1)
        .then(|| parallel_endpoint_clips(&line_context, &lane_offsets, clip))
        .transpose()?;
    let mut operations = Vec::with_capacity(lane_offsets.len());
    for (index, offset) in lane_offsets.into_iter().enumerate() {
        let line = build_bond_line(
            &line_context,
            offset,
            stroke_width,
            clip,
            shared_parallel_clips,
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
        BondRenderBatchV1::from_render_operations(attachment_axis, operations).map_err(
            |error| RenderIssueKind::UnrenderableTarget {
                reason: error.to_string(),
            },
        )?,
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
    let center = build_bond_line(context, 0.0, stroke_width, clip, None, paint.clone(), 10)?;
    build_haworth_front_batch(HaworthFrontBondInput {
        target: bond.context.target().clone(),
        paint_order: bond.context.paint_order(),
        attachment_axis: context.attachment_axis,
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
    let center = build_bond_line(context, 0.0, stroke_width, clip, None, paint.clone(), 10)?;
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
        BondRenderBatchV1::from_render_operations(context.attachment_axis, operations).map_err(
            |error| RenderIssueKind::UnrenderableTarget {
                reason: error.to_string(),
            },
        )?,
    )
    .map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("directed bond batch is not renderable: {error}"),
    })
}

struct BondLineContext<'a> {
    attachment_axis: BondAttachmentAxisV1,
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
    endpoint_clips: Option<(f64, f64)>,
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
    let (first_clip, second_clip) = endpoint_clips.map_or_else(
        || {
            Ok((
                clip.first_endpoint_clip_distance(context.first, context.direction, local_offset)?,
                clip.second_endpoint_clip_distance(context.second, reverse, local_offset)?,
            ))
        },
        Ok,
    )?;
    let has_positive_segment = clip.normal_single_policy.map_or_else(
        || normal_bond_has_positive_visible_segment(context.length, first_clip, second_clip),
        |policy| policy.has_positive_visible_segment(context.length, first_clip, second_clip),
    );
    if !has_positive_segment {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "label clipping leaves no positive visible bond segment".to_owned(),
        });
    }
    let start_axis = context.attachment_axis.start();
    let start = Point2::new(start_axis.x(), start_axis.y())
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: format!("bond attachment-axis start is not representable: {error}"),
        })?
        .offset(context.perpendicular, offset)
        .and_then(|point| point.offset(context.direction, first_clip))
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: format!("bond start is not representable: {error}"),
        })?;
    let end_axis = context.attachment_axis.end();
    let end = Point2::new(end_axis.x(), end_axis.y())
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: format!("bond attachment-axis end is not representable: {error}"),
        })?
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

/// Return the shared axial clips for every lane in one parallel-bond footprint.
fn parallel_endpoint_clips(
    context: &BondLineContext<'_>,
    lane_offsets: &[f64],
    clip: BondClipConfiguration,
) -> Result<(f64, f64), RenderIssueKind> {
    // Parallel strokes read as one terminal mark. Resolve one bounded optical
    // gap from that complete terminal width before finding the shared axial
    // clips, so double and triple bonds do not inherit a single-stroke gap.
    let clip = clip.for_parallel_lanes(lane_offsets)?;
    let reverse = negated(context.direction)?;
    let mut first_clip = 0.0_f64;
    let mut second_clip = 0.0_f64;
    for offset in lane_offsets {
        let local_offset = Vector2::new(
            context.perpendicular.x() * offset,
            context.perpendicular.y() * offset,
        )
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: format!("bond line offset is not representable: {error}"),
        })?;
        first_clip = first_clip.max(clip.first_endpoint_clip_distance(
            context.first,
            context.direction,
            local_offset,
        )?);
        second_clip = second_clip.max(clip.second_endpoint_clip_distance(
            context.second,
            reverse,
            local_offset,
        )?);
    }
    Ok((first_clip, second_clip))
}

fn lane_offset(lane_spacing: PositiveFinite, factor: f64) -> Result<f64, RenderIssueKind> {
    let offset = lane_spacing.get() * factor;
    if offset.is_finite() {
        Ok(offset)
    } else {
        Err(RenderIssueKind::UnrenderableTarget {
            reason: "bond line spacing is not representable".to_owned(),
        })
    }
}

fn bond_lane_offsets(
    style: &BondStyle,
    lane_spacing: PositiveFinite,
) -> Result<Vec<f64>, RenderIssueKind> {
    // CDML `bond_width` is the centered double-lane separation. Triple outer
    // lanes use 70 percent of that separation.
    const TRIPLE_OUTER_LANE_FACTOR: f64 = 0.7;
    let factors: &[f64] = match style {
        BondStyle::Double => &[-0.5, 0.5],
        BondStyle::Triple => &[-TRIPLE_OUTER_LANE_FACTOR, 0.0, TRIPLE_OUTER_LANE_FACTOR],
        BondStyle::NormalSingle
        | BondStyle::DoubleBondCarrierUp
        | BondStyle::DoubleBondCarrierDown
        | BondStyle::SolidWedge
        | BondStyle::HashedWedge
        | BondStyle::HaworthFrontStroke
        | BondStyle::HaworthFrontWedge
        | BondStyle::Bold
        | BondStyle::Dashed
        | BondStyle::Wavy => &[0.0],
        _ => unreachable!("unsupported styles are excluded before bond geometry"),
    };
    factors
        .iter()
        .map(|factor| lane_offset(lane_spacing, *factor))
        .collect()
}

fn render_issue_as_invalid_request(issue: RenderIssueKind) -> RenderError {
    RenderError::InvalidRequest(format!("bond attachment corridor is invalid: {issue:?}"))
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
    footprints: EndpointBondInkFootprints,
    normal_single_policy: Option<NormalBondEndpointClipPolicy>,
}

impl BondClipConfiguration {
    fn for_parallel_lanes(self, lane_offsets: &[f64]) -> Result<Self, RenderIssueKind> {
        let terminal = ParallelBondTerminalEnvelope::from_lanes(lane_offsets, self.footprints)?;
        Ok(Self {
            clearance: terminal.optical_clearance(self.clearance)?,
            ..self
        })
    }

    fn first_endpoint_clip_distance(
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
                    self.footprints.first,
                )
            },
            |policy| policy.endpoint_clip_distance(endpoint, direction, local_offset),
        )
    }

    fn second_endpoint_clip_distance(
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
                    self.footprints.second,
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
        EndpointClipGeometry::AtomLabelInk {
            core_outline_support,
            label_mask_ink_bounds,
        } => {
            // Only the structural core owns axial attachment. The complete
            // emitted bond is checked against endpoint decorations after style
            // lowering, so collision can produce a typed refusal rather than
            // an artificially detached terminal.
            let core_distance = core_outline_support.directional_extent(direction)
                - local_offset.dot(direction)
                + footprint.axial_clip_reserve(clearance.gap().get())?;
            let mut distance = validate_clip_distance(core_distance)?;
            if let Some(mask) = label_mask_ink_bounds {
                let mask_distance = clip_glyph_distance(*mask, direction, local_offset)?
                    + footprint.axial_clip_reserve(clearance.gap().get())?;
                distance = distance.max(validate_clip_distance(mask_distance)?);
            }
            Ok(distance)
        }
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

fn glyph_bounds_directional_extent(bounds: GlyphBounds, direction: Vector2) -> f64 {
    let x = if direction.x() >= 0.0 {
        bounds.max_x()
    } else {
        bounds.min_x()
    };
    let y = if direction.y() >= 0.0 {
        bounds.max_y()
    } else {
        bounds.min_y()
    };
    x * direction.x() + y * direction.y()
}

fn validate_clip_distance(distance: f64) -> Result<f64, RenderIssueKind> {
    if distance.is_finite() && distance >= 0.0 {
        Ok(distance)
    } else {
        Err(RenderIssueKind::UnrenderableTarget {
            reason: "bond endpoint clip distance is not representable".to_owned(),
        })
    }
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
