//! Neutral, label-clipped geometry for declared Haworth front bonds.

use ferrum_geometry::Vector2;

use crate::bond_style::BondStyle;
use crate::{
    BatchSpace, PathOpV3, PositiveFinite, RenderBatch, RenderDisplayLayerV1, RenderIssueKind,
    RenderOp, RenderPaintV3, RenderPoint, RenderTarget, ScenePathCommandV3, ScenePathStrokeV3,
    VectorStrokeLineCapV1,
};

const FRONT_PAD_RATIO: f64 = 0.35;
const OVERLAP_RATIO: f64 = 0.25;

/// Lower one already label-clipped declared Haworth front edge.
pub(crate) fn haworth_front_operations(
    style: BondStyle,
    tip: RenderPoint,
    base: RenderPoint,
    perpendicular: Vector2,
    stroke_width: PositiveFinite,
    wedge_width: PositiveFinite,
    paint: RenderPaintV3,
) -> Result<Vec<RenderOp>, RenderIssueKind> {
    match style {
        BondStyle::HaworthFrontStroke => {
            let [start, end] = padded_q(tip, base, wedge_width)?;
            let path = PathOpV3::new(
                vec![
                    ScenePathCommandV3::MoveTo(start),
                    ScenePathCommandV3::LineTo(end),
                ],
                Some(
                    ScenePathStrokeV3::new(paint, wedge_width)
                        .with_line_cap(VectorStrokeLineCapV1::Round),
                ),
                None,
                10,
            )
            .map_err(path_issue)?;
            Ok(vec![RenderOp::Path(path)])
        }
        BondStyle::HaworthFrontWedge => {
            let base = extended_base(tip, base, wedge_width)?;
            let commands = rounded_wedge(tip, base, perpendicular, stroke_width, wedge_width)?;
            let path = PathOpV3::new(commands, None, Some(paint), 10).map_err(path_issue)?;
            Ok(vec![RenderOp::Path(path)])
        }
        _ => Err(RenderIssueKind::UnrenderableTarget {
            reason: "declared Haworth front geometry requires q1 or w1".to_owned(),
        }),
    }
}

/// Build exact V3 operations for a detached, source-owned Haworth front preview.
///
/// The normal committed pipeline calls the same geometry after label clipping.
pub fn build_haworth_front_preview_ops(
    style: BondStyle,
    tip: RenderPoint,
    base: RenderPoint,
    stroke_width: PositiveFinite,
    wedge_width: PositiveFinite,
    paint: RenderPaintV3,
) -> Result<Vec<RenderOp>, crate::RenderError> {
    let dx = base.x() - tip.x();
    let dy = base.y() - tip.y();
    let length = dx.hypot(dy);
    if !length.is_finite() || length <= 0.0 {
        return Err(crate::RenderError::InvalidRequest(
            "Haworth preview endpoints must form a finite segment".to_owned(),
        ));
    }
    let direction = Vector2::new(dx / length, dy / length).map_err(|error| {
        crate::RenderError::InvalidRequest(format!("Haworth preview direction is invalid: {error}"))
    })?;
    haworth_front_operations(
        style,
        tip,
        base,
        direction.perpendicular_left(),
        stroke_width,
        wedge_width,
        paint,
    )
    .map_err(|issue| {
        crate::RenderError::InvalidRequest(format!("Haworth preview is invalid: {issue:?}"))
    })
}

/// Complete source facts for one already label-clipped Haworth front edge.
pub(crate) struct HaworthFrontBondInput {
    pub(crate) target: RenderTarget,
    pub(crate) paint_order: u32,
    pub(crate) style: BondStyle,
    pub(crate) tip: RenderPoint,
    pub(crate) base: RenderPoint,
    pub(crate) perpendicular: Vector2,
    pub(crate) stroke_width: PositiveFinite,
    pub(crate) wedge_width: PositiveFinite,
    pub(crate) paint: RenderPaintV3,
}

/// Build the complete source-tiered batch for one already label-clipped front edge.
pub(crate) fn build_haworth_front_batch(
    input: HaworthFrontBondInput,
) -> Result<RenderBatch, RenderIssueKind> {
    let HaworthFrontBondInput {
        target,
        paint_order,
        style,
        tip,
        base,
        perpendicular,
        stroke_width,
        wedge_width,
        paint,
    } = input;
    let layer = match style {
        BondStyle::HaworthFrontStroke => RenderDisplayLayerV1::HaworthFrontStroke,
        BondStyle::HaworthFrontWedge => RenderDisplayLayerV1::HaworthFrontWedge,
        _ => {
            return Err(RenderIssueKind::UnrenderableTarget {
                reason: "declared Haworth front batch requires q1 or w1".to_owned(),
            });
        }
    };
    let operations = haworth_front_operations(
        style,
        tip,
        base,
        perpendicular,
        stroke_width,
        wedge_width,
        paint,
    )?;
    RenderBatch::new(target, paint_order, BatchSpace::Scene, operations)
        .map(|batch| batch.with_display_layer(layer))
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: format!("Haworth front bond batch is not renderable: {error}"),
        })
}

fn padded_q(
    tip: RenderPoint,
    base: RenderPoint,
    width: PositiveFinite,
) -> Result<[RenderPoint; 2], RenderIssueKind> {
    let (dx, dy, length) = segment(tip, base)?;
    let pad = width.get() * FRONT_PAD_RATIO;
    if !pad.is_finite() || pad < 0.0 || 2.0 * pad >= length {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "Haworth front stroke cannot be padded inside its visible segment".to_owned(),
        });
    }
    Ok([
        point(tip.x() - dx * pad / length, tip.y() - dy * pad / length)?,
        point(base.x() + dx * pad / length, base.y() + dy * pad / length)?,
    ])
}

fn extended_base(
    tip: RenderPoint,
    base: RenderPoint,
    width: PositiveFinite,
) -> Result<RenderPoint, RenderIssueKind> {
    let (dx, dy, length) = segment(tip, base)?;
    let extension = width.get() * OVERLAP_RATIO;
    if !extension.is_finite() || extension < 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "Haworth front wedge extension is not representable".to_owned(),
        });
    }
    point(
        base.x() + dx * extension / length,
        base.y() + dy * extension / length,
    )
}

fn rounded_wedge(
    tip: RenderPoint,
    base: RenderPoint,
    perpendicular: Vector2,
    stroke_width: PositiveFinite,
    wedge_width: PositiveFinite,
) -> Result<Vec<ScenePathCommandV3>, RenderIssueKind> {
    let half = wedge_width.get() / 2.0;
    let radius = (stroke_width.get() / 2.0).min(half);
    if !half.is_finite() || !radius.is_finite() || half <= 0.0 || radius <= 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "Haworth front wedge width is not representable".to_owned(),
        });
    }
    let left = point(
        base.x() + perpendicular.x() * half,
        base.y() + perpendicular.y() * half,
    )?;
    let right = point(
        base.x() - perpendicular.x() * half,
        base.y() - perpendicular.y() * half,
    )?;
    let c1 = point(
        left.x() - perpendicular.x() * radius,
        left.y() - perpendicular.y() * radius,
    )?;
    let c2 = point(
        right.x() + perpendicular.x() * radius,
        right.y() + perpendicular.y() * radius,
    )?;
    Ok(vec![
        ScenePathCommandV3::MoveTo(tip),
        ScenePathCommandV3::LineTo(left),
        ScenePathCommandV3::CubicTo {
            control_1: c1,
            control_2: c2,
            end: right,
        },
        ScenePathCommandV3::LineTo(tip),
        ScenePathCommandV3::Close,
    ])
}

fn segment(tip: RenderPoint, base: RenderPoint) -> Result<(f64, f64, f64), RenderIssueKind> {
    let dx = base.x() - tip.x();
    let dy = base.y() - tip.y();
    let length = (dx * dx + dy * dy).sqrt();
    if !length.is_finite() || length <= 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "Haworth front endpoints must form a finite segment".to_owned(),
        });
    }
    Ok((dx, dy, length))
}

fn point(x: f64, y: f64) -> Result<RenderPoint, RenderIssueKind> {
    RenderPoint::new(x, y).map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("Haworth front geometry is not finite: {error}"),
    })
}

fn path_issue(error: crate::RenderError) -> RenderIssueKind {
    RenderIssueKind::UnrenderableTarget {
        reason: format!("Haworth front path is not renderable: {error}"),
    }
}
