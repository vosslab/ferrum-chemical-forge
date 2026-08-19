//! API-owned composition from document presentation facts to a replay-only path.
//!
//! `ferrum-render` intentionally has no document dependency. This module is the
//! narrow boundary that turns validated document points into its frozen path
//! grammar. The source document keeps its authored controls unchanged.

use crate::{
    PathKindV1, PresentationPathErrorV1, PresentationPathV1, RenderError, RenderPoint,
    lower_authored_control_path_v1,
};
use ferrum_document::{Point3V1, PolylineProjectionV1};
use thiserror::Error;

/// Failure while composing a document presentation path for a renderer.
#[derive(Debug, Error)]
pub enum PresentationPathCompositionErrorV1 {
    /// A document root did not provide a start and an end point.
    #[error("presentation path requires at least two issued points")]
    InsufficientPoints,
    /// A typed document point could not be represented in the render grammar.
    #[error(transparent)]
    RenderPoint(#[from] RenderError),
    /// Render-only lowering could not represent derived geometry.
    #[error(transparent)]
    Lowering(#[from] PresentationPathErrorV1),
}

/// Lower one validated document polyline without giving `render` document ownership.
pub fn lower_presentation_polyline_path_v1(
    polyline: &PolylineProjectionV1,
    kind: PathKindV1,
) -> Result<PresentationPathV1, PresentationPathCompositionErrorV1> {
    lower_presentation_points_path_v1(polyline.path().points(), kind)
}

/// Lower validated document points into a frozen, toolkit-neutral path.
///
/// Callers select `AuthoredSpline` only for a document root that explicitly
/// permits it. Ordinary CDML splines remain refused by document projection.
pub fn lower_presentation_points_path_v1(
    points: &[Point3V1],
    kind: PathKindV1,
) -> Result<PresentationPathV1, PresentationPathCompositionErrorV1> {
    let Some((start, remainder)) = points.split_first() else {
        return Err(PresentationPathCompositionErrorV1::InsufficientPoints);
    };
    let Some((end, controls)) = remainder.split_last() else {
        return Err(PresentationPathCompositionErrorV1::InsufficientPoints);
    };
    let start = render_point(*start)?;
    let end = render_point(*end)?;
    let controls = controls
        .iter()
        .copied()
        .map(render_point)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(lower_authored_control_path_v1(kind, start, &controls, end)?)
}

fn render_point(point: Point3V1) -> Result<RenderPoint, RenderError> {
    RenderPoint::new(point.x(), point.y())
}
