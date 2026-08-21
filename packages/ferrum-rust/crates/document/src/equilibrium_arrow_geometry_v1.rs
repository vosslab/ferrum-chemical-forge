//! Shared closed geometry for straight, non-spline equilibrium arrows.

use crate::Point3V1;

/// Perpendicular centre-line offset for each equilibrium shaft, in scene points.
pub const EQUILIBRIUM_HALF_SPACING_PT_V1: f64 = 4.0;
/// Distance from an arrowhead tip to the shortened shaft endpoint.
pub const EQUILIBRIUM_HEAD_LINE_INSET_PT_V1: f64 = 8.0;
/// Full arrowhead length from tip to its two rear corners.
pub const EQUILIBRIUM_HEAD_TOTAL_LENGTH_PT_V1: f64 = 10.0;
/// Arrowhead half-width.
pub const EQUILIBRIUM_HEAD_HALF_WIDTH_PT_V1: f64 = 3.0;
/// Minimum source span that leaves each shortened shaft and its opposing head distinct.
pub const EQUILIBRIUM_MINIMUM_LENGTH_PT_V1: f64 = 20.0;

/// Backend-issued display geometry for one semantic equilibrium arrow.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EquilibriumArrowGeometryV1 {
    pub(crate) axes: [[Point3V1; 2]; 2],
    pub(crate) heads: [[Point3V1; 4]; 2],
}

/// Derive both shafts and their opposing heads from exactly two persisted centre-line points.
pub(crate) fn geometry(
    start: Point3V1,
    end: Point3V1,
) -> Result<EquilibriumArrowGeometryV1, String> {
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let length = dx.hypot(dy);
    if !length.is_finite() || length < EQUILIBRIUM_MINIMUM_LENGTH_PT_V1 {
        return Err("equilibrium arrow source span is below its fixed geometry minimum".to_owned());
    }
    let ux = dx / length;
    let uy = dy / length;
    let px = -uy;
    let py = ux;
    let point = |source: Point3V1, along: f64, perpendicular: f64| {
        Point3V1::new(
            source.x() + ux * along + px * perpendicular,
            source.y() + uy * along + py * perpendicular,
            source.z(),
        )
        .map_err(|error| error.to_string())
    };
    let lower_start = point(start, 0.0, -EQUILIBRIUM_HALF_SPACING_PT_V1)?;
    let lower_end = point(end, 0.0, -EQUILIBRIUM_HALF_SPACING_PT_V1)?;
    let upper_start = point(start, 0.0, EQUILIBRIUM_HALF_SPACING_PT_V1)?;
    let upper_end = point(end, 0.0, EQUILIBRIUM_HALF_SPACING_PT_V1)?;
    let lower_axis_start = point(lower_start, EQUILIBRIUM_HEAD_LINE_INSET_PT_V1, 0.0)?;
    let upper_axis_end = point(upper_end, -EQUILIBRIUM_HEAD_LINE_INSET_PT_V1, 0.0)?;
    let lower_head = [
        lower_start,
        point(
            lower_start,
            EQUILIBRIUM_HEAD_TOTAL_LENGTH_PT_V1,
            EQUILIBRIUM_HEAD_HALF_WIDTH_PT_V1,
        )?,
        lower_axis_start,
        point(
            lower_start,
            EQUILIBRIUM_HEAD_TOTAL_LENGTH_PT_V1,
            -EQUILIBRIUM_HEAD_HALF_WIDTH_PT_V1,
        )?,
    ];
    let upper_head = [
        upper_end,
        point(
            upper_end,
            -EQUILIBRIUM_HEAD_TOTAL_LENGTH_PT_V1,
            EQUILIBRIUM_HEAD_HALF_WIDTH_PT_V1,
        )?,
        upper_axis_end,
        point(
            upper_end,
            -EQUILIBRIUM_HEAD_TOTAL_LENGTH_PT_V1,
            -EQUILIBRIUM_HEAD_HALF_WIDTH_PT_V1,
        )?,
    ];
    Ok(EquilibriumArrowGeometryV1 {
        axes: [[lower_axis_start, lower_end], [upper_start, upper_axis_end]],
        heads: [lower_head, upper_head],
    })
}
