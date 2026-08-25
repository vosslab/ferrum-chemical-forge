//! Exact lowering of durable Haworth fragment geometry into V1 line batches.

use ferrum_domain::haworth::{BondDepiction, HaworthFragment, HaworthPoint};

use crate::{
    BatchSpace, LineOp, Paint, PositiveFinite, RenderError, RenderOp, RenderPoint, RenderProvenance,
};

/// Identifier-free Haworth paint batch for detached previews.
#[derive(Clone, Debug, PartialEq)]
pub struct HaworthPreviewBatchV1 {
    paint_order: u32,
    coordinate_space: BatchSpace,
    operations: Vec<RenderOp>,
}

impl HaworthPreviewBatchV1 {
    #[must_use]
    pub const fn paint_order(&self) -> u32 {
        self.paint_order
    }
    #[must_use]
    pub const fn coordinate_space(&self) -> &BatchSpace {
        &self.coordinate_space
    }
    #[must_use]
    pub fn operations(&self) -> &[RenderOp] {
        &self.operations
    }
}

/// Identifier-free Haworth preview preserving admitted geometry and paint order.
#[derive(Clone, Debug, PartialEq)]
pub struct HaworthPreviewV1 {
    provenance: RenderProvenance,
    batches: Vec<HaworthPreviewBatchV1>,
}

impl HaworthPreviewV1 {
    #[must_use]
    pub const fn provenance(&self) -> RenderProvenance {
        self.provenance
    }
    #[must_use]
    pub fn batches(&self) -> &[HaworthPreviewBatchV1] {
        &self.batches
    }
}

/// Complete presentation facts needed to lower a Haworth fragment.
#[derive(Clone, Debug, PartialEq)]
pub struct HaworthRenderRequest {
    /// Exact revision associated with the immutable fragment.
    pub provenance: RenderProvenance,
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
) -> Result<HaworthPreviewV1, RenderError> {
    let mut source = Vec::new();
    for (bond, depiction) in request.fragment.ring_bonds() {
        let endpoints = request.fragment.bond_geometry().get(bond).ok_or_else(|| {
            RenderError::InvalidRequest("Haworth fragment lacks ring-bond geometry".to_owned())
        })?;
        let order = *request.fragment.source_orders().get(bond).ok_or_else(|| {
            RenderError::InvalidRequest("Haworth fragment lacks graph source order".to_owned())
        })?;
        source.push((order, bond.clone(), *endpoints, Some(*depiction)));
    }
    for (bond, link) in request.fragment.links() {
        let order = *request.fragment.source_orders().get(bond).ok_or_else(|| {
            RenderError::InvalidRequest("Haworth fragment lacks graph source order".to_owned())
        })?;
        source.push((order, bond.clone(), [link.parent, link.child], None));
    }
    source.sort_by_key(|(order, _, _, _)| *order);
    if source.windows(2).any(|values| values[0].0 == values[1].0) {
        return Err(RenderError::InvalidRequest(
            "Haworth fragment source orders must be unique".to_owned(),
        ));
    }
    let batches = source
        .into_iter()
        .map(|(order, _, points, depiction)| {
            let operations = match depiction {
                Some(BondDepiction::HaworthFront {
                    face: ferrum_domain::haworth::Face::Front,
                    ..
                }) => wide_face_lines(points, request.line_width, &request.line_paint)?,
                Some(BondDepiction::Back {
                    face: ferrum_domain::haworth::Face::Back,
                })
                | None => vec![RenderOp::Line(LineOp::new(
                    point(points[0])?,
                    point(points[1])?,
                    request.line_width,
                    request.line_paint.clone(),
                    0,
                )?)],
                Some(_) => {
                    return Err(RenderError::InvalidRequest(
                        "Haworth fragment contains invalid face semantics".to_owned(),
                    ));
                }
            };
            Ok(HaworthPreviewBatchV1 {
                paint_order: order,
                coordinate_space: BatchSpace::Scene,
                operations,
            })
        })
        .collect::<Result<Vec<_>, RenderError>>()?;
    Ok(HaworthPreviewV1 {
        provenance: request.provenance,
        batches,
    })
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

#[cfg(test)]
mod tests {
    use ferrum_core::{Atom, Bond, BondOrder, Identifier, Molecule, Position, VertexRef};
    use ferrum_domain::haworth::{
        HaworthLayoutRequest, HaworthTopologyBuilder, HaworthVertex, RingForm, layout_single_ring,
    };

    use super::{HaworthPreviewBatchV1, HaworthRenderRequest, lower_haworth_fragment};
    use crate::{Paint, PositiveFinite, RenderOp, RenderProvenance, RenderRevision, Rgb24};

    fn atom(index: usize, element: &str) -> Atom {
        Atom::new(
            Identifier::new(format!("render-a{index}")).expect("identifier"),
            Some(element.to_owned()),
            Position::new(index as f64, 0.0, 0.0).expect("position"),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("atom")
    }

    fn fragment() -> ferrum_domain::haworth::HaworthFragment {
        let atoms = ["O", "C", "C", "C", "C", "C"]
            .into_iter()
            .enumerate()
            .map(|(index, element)| atom(index, element))
            .collect::<Vec<_>>();
        let bonds = (0..6)
            .map(|index| {
                Bond::new(
                    Identifier::new(format!("render-b{index}")).expect("identifier"),
                    VertexRef::Atom(atoms[index].identity().clone()),
                    VertexRef::Atom(atoms[(index + 1) % 6].identity().clone()),
                    None,
                    Some(BondOrder::Single),
                    None,
                    Some(false),
                )
                .expect("bond")
            })
            .collect::<Vec<_>>();
        let molecule = Molecule::new(
            Identifier::new("render-molecule").expect("identifier"),
            None,
            atoms.clone(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            bonds,
        )
        .expect("molecule");
        let topology = HaworthTopologyBuilder::new(
            RingForm::Pyranose,
            atoms[1].identity().clone(),
            atoms
                .iter()
                .map(|atom| HaworthVertex {
                    atom: atom.identity().clone(),
                })
                .collect(),
        )
        .build(&molecule)
        .expect("topology");
        // Construct only through the public validated single-ring planner, then use
        // its fragment-producing tree request as the integration boundary.
        let _depiction = layout_single_ring(&HaworthLayoutRequest {
            topology: topology.clone(),
            scale: 10.0,
        })
        .expect("depiction");
        ferrum_domain::haworth::layout_tree(&ferrum_domain::haworth::HaworthTreeRequest {
            molecule,
            rings: vec![ferrum_domain::haworth::HaworthRingNode {
                node_id: 0,
                topology,
            }],
            links: Vec::new(),
            root: 0,
            scale: 10.0,
        })
        .expect("fragment")
    }

    #[test]
    fn lowerer_preserves_graph_source_order_and_front_partition() {
        let fragment = fragment();
        let plan = lower_haworth_fragment(&HaworthRenderRequest {
            provenance: RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
            fragment,
            line_width: PositiveFinite::new(1.0).expect("width"),
            line_paint: Paint::rgb24(Rgb24::new("000000").expect("paint")),
        })
        .expect("plan");
        assert_eq!(plan.batches().len(), 6);
        let orders = plan
            .batches()
            .iter()
            .map(HaworthPreviewBatchV1::paint_order)
            .collect::<Vec<_>>();
        assert_eq!(orders, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(
            plan.batches()
                .iter()
                .filter(|batch| batch.operations().len() == 3)
                .count(),
            3
        );
        assert!(plan.batches().iter().all(|batch| {
            batch
                .operations()
                .iter()
                .all(|op| matches!(op, RenderOp::Line(_)))
        }));
    }
}
