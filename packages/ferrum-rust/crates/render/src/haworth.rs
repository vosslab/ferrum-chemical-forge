//! Exact lowering of durable Haworth fragment geometry into V1 line batches.

use ferrum_domain::haworth::{BondDepiction, HaworthFragment, HaworthPoint};

use crate::{
    BatchSpace, LineOp, MoleculeRenderPlan, Paint, PositiveFinite, RenderBatch, RenderError,
    RenderOp, RenderPoint, RenderRevision, RenderTarget,
};

/// Complete presentation facts needed to lower a Haworth fragment.
#[derive(Clone, Debug, PartialEq)]
pub struct HaworthRenderRequest {
    /// Exact revision associated with the immutable fragment.
    pub revision: RenderRevision,
    /// Fully planned Haworth fragment, including semantic front-face roles.
    pub fragment: HaworthFragment,
    /// Explicit normal bond stroke width.
    pub line_width: PositiveFinite,
    /// Explicit line paint with no renderer fallback.
    pub line_paint: Paint,
}

/// Lower every accepted Haworth ring and glycosidic bond to a V1 batch.
///
/// Ordinary and link bonds lower to one finite line. Each Haworth front edge
/// lowers to three explicitly offset finite lines, producing a deterministic
/// broad-face mark without adding a hidden renderer wedge primitive. This
/// lowerer uses no glyph metrics because this fragment carries no text labels;
/// its label anchors are durable facts for a later explicit label request.
pub fn lower_haworth_fragment(
    request: &HaworthRenderRequest,
) -> Result<MoleculeRenderPlan, RenderError> {
    let mut source = Vec::new();
    for (bond, depiction) in request.fragment.ring_bonds() {
        let endpoints = request.fragment.bond_geometry().get(bond).ok_or_else(|| {
            RenderError::InvalidRequest("Haworth fragment lacks ring-bond geometry".to_owned())
        })?;
        source.push((bond.clone(), *endpoints, Some(*depiction)));
    }
    for (bond, link) in request.fragment.links() {
        source.push((bond.clone(), [link.parent, link.child], None));
    }
    source.sort_by_key(|(bond, _, _)| bond.clone());
    let batches = source
        .into_iter()
        .enumerate()
        .map(|(order, (bond, points, depiction))| {
            let operations = match depiction {
                Some(BondDepiction::HaworthFront { .. }) => {
                    wide_face_lines(points, request.line_width, &request.line_paint)?
                }
                Some(BondDepiction::Back { .. }) | None => vec![RenderOp::Line(LineOp::new(
                    point(points[0])?,
                    point(points[1])?,
                    request.line_width,
                    request.line_paint.clone(),
                    0,
                )?)],
            };
            RenderBatch::new(
                RenderTarget::new(bond, order as u32),
                BatchSpace::Scene,
                operations,
            )
        })
        .collect::<Result<Vec<_>, RenderError>>()?;
    MoleculeRenderPlan::new(request.revision, batches, Vec::new())
}

fn point(value: HaworthPoint) -> Result<RenderPoint, RenderError> {
    RenderPoint::new(value.x, value.y)
}

fn wide_face_lines(
    points: [HaworthPoint; 2],
    width: PositiveFinite,
    paint: &Paint,
) -> Result<Vec<RenderOp>, RenderError> {
    let dx = points[1].x - points[0].x;
    let dy = points[1].y - points[0].y;
    let length = dx.mul_add(dx, dy * dy).sqrt();
    if !length.is_finite() || length <= 0.0 {
        return Err(RenderError::InvalidRequest(
            "Haworth front edge must have finite nonzero length".to_owned(),
        ));
    }
    let offset = width.get().min(length / 6.0);
    let nx = -dy / length * offset;
    let ny = dx / length * offset;
    [(-1.0, 0), (0.0, 1), (1.0, 2)]
        .into_iter()
        .map(|(multiplier, z)| {
            let start = HaworthPoint {
                x: points[0].x + nx * multiplier,
                y: points[0].y + ny * multiplier,
            };
            let end = HaworthPoint {
                x: points[1].x + nx * multiplier,
                y: points[1].y + ny * multiplier,
            };
            Ok(RenderOp::Line(LineOp::new(
                point(start)?,
                point(end)?,
                width,
                paint.clone(),
                z,
            )?))
        })
        .collect()
}
