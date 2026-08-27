//! Source-owned directed solid and hashed wedge geometry.

use ferrum_geometry::Vector2;

use crate::bond_style::BondStyle;
use crate::{
    LineOp, PathOpV3, PositiveFinite, RenderError, RenderIssueKind, RenderOp, RenderPaintV3,
    RenderPoint, ScenePathCommandV3,
};

/// Maximum useful hatch strokes for one ordinary directed bond.
///
/// Each hatch occupies at least the source-owned spacing policy below. Beyond
/// sixty-four such intervals, additional strokes are not distinguishable as a
/// bond depiction at ordinary scene scale, while every extra line consumes the
/// same V2/PDF path-command resources as authored geometry. This source policy
/// keeps one pathological finite coordinate pair from expanding a single bond
/// without turning document-level caller-owned render budgets into a hidden
/// per-bond allocation limit.
const MAX_HASHED_WEDGE_STROKES_V1: usize = 64;

pub(crate) fn directed_stereo_operations(
    style: BondStyle,
    tip: RenderPoint,
    base: RenderPoint,
    perpendicular: Vector2,
    stroke_width: PositiveFinite,
    wedge_width: PositiveFinite,
    paint: RenderPaintV3,
) -> Result<Vec<RenderOp>, RenderIssueKind> {
    let half_base = wedge_width.get() / 2.0;
    if !half_base.is_finite() || half_base <= 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "directed wedge width is not representable".to_owned(),
        });
    }
    let base_left = render_offset(base, perpendicular, half_base)?;
    let base_right = render_offset(base, perpendicular, -half_base)?;
    match style {
        BondStyle::SolidWedge => Ok(vec![RenderOp::Path(
            PathOpV3::new(
                vec![
                    ScenePathCommandV3::MoveTo(tip),
                    ScenePathCommandV3::LineTo(base_left),
                    ScenePathCommandV3::LineTo(base_right),
                    ScenePathCommandV3::Close,
                ],
                None,
                Some(paint),
                10,
            )
            .map_err(render_path_issue)?,
        )]),
        BondStyle::HashedWedge => build_hashed_wedge_operations(
            tip,
            base,
            perpendicular,
            stroke_width,
            wedge_width,
            paint,
        ),
        _ => unreachable!("directed stereo geometry requires a directed style"),
    }
}

fn build_hashed_wedge_operations(
    tip: RenderPoint,
    base: RenderPoint,
    perpendicular: Vector2,
    stroke_width: PositiveFinite,
    wedge_width: PositiveFinite,
    paint: RenderPaintV3,
) -> Result<Vec<RenderOp>, RenderIssueKind> {
    let dx = base.x() - tip.x();
    let dy = base.y() - tip.y();
    let length = (dx * dx + dy * dy).sqrt();
    let spacing = (2.0 * stroke_width.get()).max(0.4 * wedge_width.get());
    if !length.is_finite() || length <= 0.0 || !spacing.is_finite() || spacing <= 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "hashed wedge geometry is not representable".to_owned(),
        });
    }
    let requested = (length / spacing).ceil();
    let count = if requested.is_finite() && requested > 0.0 {
        requested.min(MAX_HASHED_WEDGE_STROKES_V1 as f64) as usize
    } else {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "hashed wedge stroke density is not representable".to_owned(),
        });
    };
    let mut operations = Vec::with_capacity(count);
    for index in 1..=count {
        let fraction = index as f64 / (count + 1) as f64;
        let center = RenderPoint::new(tip.x() + dx * fraction, tip.y() + dy * fraction)
            .map_err(render_point_issue)?;
        let half_width = wedge_width.get() * fraction / 2.0;
        let start = render_offset(center, perpendicular, half_width)?;
        let end = render_offset(center, perpendicular, -half_width)?;
        let z = 10
            + i32::try_from(index).map_err(|_| RenderIssueKind::UnrenderableTarget {
                reason: "hashed wedge operation ordering is exhausted".to_owned(),
            })?;
        operations.push(RenderOp::Line(
            LineOp::new(start, end, stroke_width, paint.clone(), z).map_err(render_path_issue)?,
        ));
    }
    Ok(operations)
}

fn render_offset(
    point: RenderPoint,
    vector: Vector2,
    distance: f64,
) -> Result<RenderPoint, RenderIssueKind> {
    RenderPoint::new(
        point.x() + vector.x() * distance,
        point.y() + vector.y() * distance,
    )
    .map_err(render_point_issue)
}

fn render_point_issue(error: RenderError) -> RenderIssueKind {
    RenderIssueKind::UnrenderableTarget {
        reason: format!("directed wedge point is not finite: {error}"),
    }
}

fn render_path_issue(error: RenderError) -> RenderIssueKind {
    RenderIssueKind::UnrenderableTarget {
        reason: format!("directed wedge path is not renderable: {error}"),
    }
}
