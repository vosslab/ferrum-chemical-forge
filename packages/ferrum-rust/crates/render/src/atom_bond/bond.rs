//! Bond target validation, clipping geometry, and stereo lowering.

use std::collections::HashMap;

use ferrum_core::{RecordId, RecordKind};
use ferrum_geometry::{Point2, Vector2};

use crate::bond_presentation_geometry;
use crate::bond_style::BondStyle;
use crate::directed_stereo_bond::directed_stereo_operations;
use crate::glyph_metrics::GlyphBounds;
use crate::haworth_front_bond::{HaworthFrontBondInput, build_haworth_front_batch};
use crate::render_target::RenderPlanEntryContextV1;
use crate::{
    BondAttachmentAxisV1, BondRenderBatchV1, DoubleBondCarrierMarkDirectionV1,
    DoubleBondCarrierMarkOp, LineOp, PositiveFinite, RenderBatchV4, RenderError, RenderIssueKind,
    RenderOp, RenderPaintV3, RenderTarget,
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
    let lane_offsets = offsets
        .iter()
        .map(|factor| lane_offset(lane_spacing, *factor))
        .collect::<Result<Vec<_>, _>>()?;
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
        const TERMINAL_WIDTH_GAP_FACTOR: f64 = 0.25;
        const MAXIMUM_BASE_CLEARANCE_FACTOR: f64 = 1.75;

        let maximum_lane_offset = lane_offsets
            .iter()
            .map(|offset| offset.abs())
            .fold(0.0_f64, f64::max);
        let endpoint_half_width = [self.footprints.first, self.footprints.second]
            .into_iter()
            .map(|footprint| {
                footprint
                    .endpoint_radius
                    .max(footprint.transverse_half_width)
            })
            .fold(0.0_f64, f64::max);
        let terminal_width = 2.0 * (maximum_lane_offset + endpoint_half_width);
        let base_clearance = self.clearance.gap().get();
        let width_clearance = terminal_width * TERMINAL_WIDTH_GAP_FACTOR;
        let maximum_clearance = base_clearance * MAXIMUM_BASE_CLEARANCE_FACTOR;
        let resolved_clearance = base_clearance.max(width_clearance.min(maximum_clearance));
        let gap = PositiveFinite::new(resolved_clearance).map_err(|_| {
            RenderIssueKind::UnrenderableTarget {
                reason: "parallel-bond optical clearance is not representable".to_owned(),
            }
        })?;
        Ok(Self {
            clearance: BondInkClearance::new(gap),
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
            non_core_run_ink_bounds,
        } => {
            // Masks and decorations are collision exclusions, not optical
            // attachment targets. Their bounds reserve the emitted footprint
            // plus a small cross-raster separation; the complete optical gap
            // belongs solely to the core.
            let exclusion_gap = clearance.gap().get() * 0.25;
            let (x_inflation, y_inflation) =
                footprint.glyph_bounds_inflation(exclusion_gap, direction)?;
            let core_distance = core_outline_support.directional_extent(direction)
                - local_offset.dot(direction)
                + footprint.axial_clip_reserve(clearance.gap().get())?;
            let mut distance = validate_clip_distance(core_distance)?;
            if let Some(mask) = label_mask_ink_bounds {
                distance = distance.max(clip_glyph_distance(
                    inflate_glyph_bounds(*mask, x_inflation, y_inflation)?,
                    direction,
                    local_offset,
                )?);
            }
            for decoration in non_core_run_ink_bounds {
                distance = distance.max(clip_glyph_distance(
                    inflate_glyph_bounds(*decoration, x_inflation, y_inflation)?,
                    direction,
                    local_offset,
                )?);
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct BondInkFootprint {
    /// Radius of final ink at the endpoint along every direction (for example,
    /// a round line cap). This preserves the exact circular normal-bond
    /// envelope instead of conservatively expanding diagonal labels twice.
    endpoint_radius: f64,
    /// Half-width normal to the carrier axis at the endpoint. Butt-capped
    /// lines and filled wedge bases contribute here, never to axial reach.
    transverse_half_width: f64,
    /// Extra final ink extending beyond the clipped endpoint along the carrier axis.
    axial_overhang: f64,
    /// Final ink beginning inward from the clipped carrier endpoint.
    axial_retreat: f64,
}

impl BondInkFootprint {
    fn axial_clip_reserve(self, clearance: f64) -> Result<f64, RenderIssueKind> {
        const ENDPOINT_WIDTH_GAP_FACTOR: f64 = 0.25;
        let endpoint_width = 2.0 * self.endpoint_radius.max(self.transverse_half_width);
        let optical_clearance = clearance.max(endpoint_width * ENDPOINT_WIDTH_GAP_FACTOR);
        let reserve =
            optical_clearance + self.endpoint_radius + self.axial_overhang - self.axial_retreat;
        validate_clip_distance(reserve)
    }

    fn glyph_bounds_inflation(
        self,
        clearance: f64,
        direction: Vector2,
    ) -> Result<(f64, f64), RenderIssueKind> {
        let perpendicular = direction.perpendicular_left();
        let base = clearance + self.endpoint_radius;
        let x = base
            + self.axial_overhang * direction.x().abs()
            + self.transverse_half_width * perpendicular.x().abs();
        let y = base
            + self.axial_overhang * direction.y().abs()
            + self.transverse_half_width * perpendicular.y().abs();
        if x.is_finite() && x >= 0.0 && y.is_finite() && y >= 0.0 {
            Ok((x, y))
        } else {
            Err(RenderIssueKind::UnrenderableTarget {
                reason: "bond ink clearance is not representable".to_owned(),
            })
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

/// Directional endpoint envelopes for one final bond footprint.
#[derive(Clone, Copy)]
struct EndpointBondInkFootprints {
    first: BondInkFootprint,
    second: BondInkFootprint,
}

impl EndpointBondInkFootprints {
    const fn symmetric(footprint: BondInkFootprint) -> Self {
        Self {
            first: footprint,
            second: footprint,
        }
    }
}

fn final_ink_footprints(
    style: &BondStyle,
    stroke_width: PositiveFinite,
    wedge_width: PositiveFinite,
) -> Result<EndpointBondInkFootprints, RenderIssueKind> {
    let symmetric = final_ink_footprint(style, stroke_width, wedge_width)?;
    match style {
        // Directed wedge lowering emits a narrow tip at source endpoint one
        // and its full width only at endpoint two. Reserving the base radius
        // at both ends detached the tip from its intended atom character.
        BondStyle::SolidWedge | BondStyle::HashedWedge | BondStyle::HaworthFrontWedge => {
            Ok(EndpointBondInkFootprints {
                first: BondInkFootprint {
                    endpoint_radius: 0.0,
                    transverse_half_width: 0.0,
                    axial_overhang: 0.0,
                    axial_retreat: if matches!(style, BondStyle::HashedWedge) {
                        stroke_width.get() / 2.0
                    } else {
                        0.0
                    },
                },
                second: symmetric,
            })
        }
        _ => Ok(EndpointBondInkFootprints::symmetric(symmetric)),
    }
}

fn final_ink_footprint(
    style: &BondStyle,
    stroke_width: PositiveFinite,
    wedge_width: PositiveFinite,
) -> Result<BondInkFootprint, RenderIssueKind> {
    let endpoint_radius = match style {
        // A wavy bond starts and ends on its carrier axis. Its endpoint clip
        // needs only the round-cap radius there; the later lateral amplitude
        // belongs to complete-plan collision admission, not label clearance.
        BondStyle::Wavy => stroke_width.get() / 2.0,
        BondStyle::HaworthFrontStroke => wedge_width.get() / 2.0,
        _ => 0.0,
    };
    let transverse_half_width = match style {
        BondStyle::SolidWedge | BondStyle::HashedWedge | BondStyle::HaworthFrontWedge => {
            wedge_width.get() / 2.0
        }
        BondStyle::Bold => stroke_width.get(),
        BondStyle::Wavy | BondStyle::HaworthFrontStroke => 0.0,
        _ => stroke_width.get() / 2.0,
    };
    let axial_overhang = match style {
        // Each terminal hashed-wedge stroke is perpendicular to the carrier,
        // so its butt-capped stroke extends half a line width along the axis.
        BondStyle::HashedWedge => stroke_width.get() / 2.0,
        // q1 pads its emitted centerline 0.35w toward each label; its separate
        // round-cap radius is represented above.
        BondStyle::HaworthFrontStroke => wedge_width.get() * 0.35,
        // The filled Haworth-front wedge extends its base exactly 0.25w past
        // the already-clipped carrier endpoint.
        BondStyle::HaworthFrontWedge => wedge_width.get() * 0.25,
        _ => 0.0,
    };
    if endpoint_radius.is_finite()
        && endpoint_radius >= 0.0
        && transverse_half_width.is_finite()
        && transverse_half_width >= 0.0
        && axial_overhang.is_finite()
        && axial_overhang >= 0.0
    {
        Ok(BondInkFootprint {
            endpoint_radius,
            transverse_half_width,
            axial_overhang,
            axial_retreat: 0.0,
        })
    } else {
        Err(RenderIssueKind::UnrenderableTarget {
            reason: "final bond ink footprint is not representable".to_owned(),
        })
    }
}

fn inflate_glyph_bounds(
    bounds: GlyphBounds,
    x_amount: f64,
    y_amount: f64,
) -> Result<GlyphBounds, RenderIssueKind> {
    if !x_amount.is_finite() || x_amount < 0.0 || !y_amount.is_finite() || y_amount < 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "bond ink clearance is not representable".to_owned(),
        });
    }
    GlyphBounds::new(
        bounds.min_x() - x_amount,
        bounds.min_y() - y_amount,
        bounds.max_x() + x_amount,
        bounds.max_y() + y_amount,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_terminal_envelope_increases_clearance_as_one_visual_unit() {
        let base = BondInkClearance::new(PositiveFinite::new(0.75).expect("base clearance"));
        let footprint = final_ink_footprint(
            &BondStyle::Double,
            PositiveFinite::new(1.0).expect("stroke width"),
            PositiveFinite::new(5.0).expect("wedge width"),
        )
        .expect("double-bond footprint");
        let clip = BondClipConfiguration {
            clearance: base,
            footprints: EndpointBondInkFootprints::symmetric(footprint),
            normal_single_policy: None,
        };
        let double = clip
            .for_parallel_lanes(&[-3.0, 3.0])
            .expect("double-bond clearance")
            .clearance
            .gap()
            .get();
        let triple = clip
            .for_parallel_lanes(&[-4.2, 0.0, 4.2])
            .expect("triple-bond clearance")
            .clearance
            .gap()
            .get();
        assert!(base.gap().get() < double && double <= triple);
    }

    #[test]
    fn wide_endpoint_uses_a_quarter_width_optical_gap_floor() {
        let wide = BondInkFootprint {
            endpoint_radius: 2.0,
            transverse_half_width: 0.0,
            axial_overhang: 1.4,
            axial_retreat: 0.0,
        };
        let ordinary = BondInkFootprint {
            endpoint_radius: 0.0,
            transverse_half_width: 0.4,
            axial_overhang: 0.0,
            axial_retreat: 0.0,
        };
        assert_eq!(wide.axial_clip_reserve(0.75), Ok(4.4));
        assert_eq!(ordinary.axial_clip_reserve(0.75), Ok(0.75));
    }
}
