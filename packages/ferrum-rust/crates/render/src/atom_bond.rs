//! Atom-label and closed bond render-plan generation.
//!
//! This facade owns request-wide ordering and endpoint registration. Focused
//! children lower atom-local depiction and clipped bond geometry independently.

mod atom;
mod bond;

use std::collections::{HashMap, HashSet};

use ferrum_core::RecordKind;
use ferrum_geometry::Point2;

use crate::{
    CompactGroupBondEndpointV1, GlyphBounds, GlyphMetrics, MoleculeRenderPlan, Paint,
    PositiveFinite, RenderError, RenderIssue, RenderIssueKind, RenderPoint, RenderProvenance,
    RenderTarget,
};

pub use atom::{
    AtomLabelFacts, AtomLabelFontProfile, AtomMarkRenderFacts, AtomMarkRenderKind,
    AtomNumberLabelFacts, AtomRenderTarget,
};
pub use bond::BondRenderTarget;

use atom::build_atom_batch;
use bond::build_bond_batch;

/// Deliberate visibility supplied by the authoritative source projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TargetVisibility {
    /// The target is part of this visible render projection.
    Visible,
    /// The target must remain absent with an explanatory non-fatal diagnostic.
    Hidden { reason: String },
}

impl TargetVisibility {
    pub(super) fn issue(&self, noun: &str) -> Option<RenderIssueKind> {
        match self {
            Self::Visible => None,
            Self::Hidden { reason } if !reason.trim().is_empty() => {
                Some(RenderIssueKind::UnsupportedFeature {
                    feature: format!("invisible {noun}: {reason}"),
                })
            }
            Self::Hidden { .. } => Some(RenderIssueKind::UnsupportedFeature {
                feature: format!("invisible {noun}"),
            }),
        }
    }
}
/// Bond style carried by the source projection.
pub struct AtomBondRenderRequest {
    provenance: RenderProvenance,
    atoms: Vec<AtomRenderTarget>,
    bonds: Vec<BondRenderTarget>,
    font: AtomLabelFontProfile,
    line_width: PositiveFinite,
    bond_lane_spacing: PositiveFinite,
    line_paint: Paint,
    compact_group_endpoints: Vec<CompactGroupBondEndpointV1>,
}

impl AtomBondRenderRequest {
    /// Construct a request whose target identities and source orders are unique.
    pub fn new(
        provenance: RenderProvenance,
        atoms: Vec<AtomRenderTarget>,
        bonds: Vec<BondRenderTarget>,
        font: AtomLabelFontProfile,
        line_width: PositiveFinite,
        bond_lane_spacing: PositiveFinite,
        line_paint: Paint,
    ) -> Result<Self, RenderError> {
        let mut identifiers = HashSet::new();
        let mut source_orders = HashSet::new();
        for target in atoms
            .iter()
            .map(AtomRenderTarget::target)
            .chain(bonds.iter().map(BondRenderTarget::target))
        {
            if !identifiers.insert(target.record_id().clone()) {
                return Err(RenderError::InvalidRequest(
                    "atom-bond request has duplicate targets".to_owned(),
                ));
            }
            if !source_orders.insert(target.source_order()) {
                return Err(RenderError::InvalidRequest(
                    "atom-bond request has duplicate source order".to_owned(),
                ));
            }
        }
        Ok(Self {
            provenance,
            atoms,
            bonds,
            font,
            line_width,
            bond_lane_spacing,
            line_paint,
            compact_group_endpoints: Vec::new(),
        })
    }

    /// Attach typed compact-group attachment geometry for supported exterior bonds.
    pub fn with_compact_group_endpoints(
        mut self,
        compact_group_endpoints: Vec<CompactGroupBondEndpointV1>,
    ) -> Result<Self, RenderError> {
        let mut identifiers = self
            .atoms
            .iter()
            .map(AtomRenderTarget::target)
            .chain(self.bonds.iter().map(BondRenderTarget::target))
            .map(|target| target.record_id().clone())
            .collect::<HashSet<_>>();
        let mut source_orders = self
            .atoms
            .iter()
            .map(AtomRenderTarget::target)
            .chain(self.bonds.iter().map(BondRenderTarget::target))
            .map(RenderTarget::source_order)
            .collect::<HashSet<_>>();
        for endpoint in &compact_group_endpoints {
            if !identifiers.insert(endpoint.target().record_id().clone()) {
                return Err(RenderError::InvalidRequest(
                    "atom-bond request has duplicate compact-group endpoint target".to_owned(),
                ));
            }
            if !source_orders.insert(endpoint.target().source_order()) {
                return Err(RenderError::InvalidRequest(
                    "atom-bond request has duplicate compact-group endpoint source order"
                        .to_owned(),
                ));
            }
        }
        self.compact_group_endpoints = compact_group_endpoints;
        Ok(self)
    }
}

/// Build the total ordered batch-or-issue partition for this render slice.
pub fn build_atom_bond_plan<M: GlyphMetrics>(
    request: &AtomBondRenderRequest,
    metrics: &M,
) -> Result<MoleculeRenderPlan, RenderError> {
    let mut batches = Vec::new();
    let mut issues = Vec::new();
    let mut endpoints = HashMap::new();

    for atom in &request.atoms {
        let target = atom.target.clone();
        let outcome = atom.visibility.issue("atom target").map_or_else(
            || build_atom_batch(atom, atom.font.as_ref().unwrap_or(&request.font), metrics),
            |kind| Ok(Err(kind)),
        );
        match outcome? {
            Ok((batch, bounds)) => {
                endpoints.insert(
                    target.record_id().clone(),
                    RenderEndpointGeometry {
                        kind: RecordKind::Atom,
                        position: render_point_to_geometry(atom.position)?,
                        bounds,
                    },
                );
                batches.push(batch);
            }
            Err(kind) => issues.push(RenderIssue::new(target, kind)?),
        }
    }

    for endpoint in &request.compact_group_endpoints {
        endpoints.insert(
            endpoint.target().record_id().clone(),
            RenderEndpointGeometry {
                kind: RecordKind::Group,
                position: render_point_to_geometry(endpoint.position())?,
                bounds: endpoint.bounds(),
            },
        );
    }

    for bond in &request.bonds {
        let target = bond.target.clone();
        let outcome = if let Some(kind) = bond.visibility.issue("bond target") {
            Err(kind)
        } else if let Some(style) = bond.style.unsupported_name() {
            Err(RenderIssueKind::UnsupportedFeature {
                feature: style.to_owned(),
            })
        } else {
            let (stroke_width, lane_spacing, wedge_width, paint) =
                bond.appearance.as_ref().map_or_else(
                    || {
                        (
                            request.line_width,
                            request.bond_lane_spacing,
                            request.bond_lane_spacing,
                            request.line_paint.clone(),
                        )
                    },
                    |appearance| {
                        (
                            appearance.stroke_width,
                            appearance.lane_spacing,
                            appearance.wedge_width,
                            appearance.paint.clone(),
                        )
                    },
                );
            build_bond_batch(
                bond,
                &endpoints,
                stroke_width,
                lane_spacing,
                wedge_width,
                paint,
            )
        };
        match outcome {
            Ok(batch) => batches.push(batch),
            Err(kind) => issues.push(RenderIssue::new(target, kind)?),
        }
    }

    batches.sort_by_key(|batch| batch.target().source_order());
    issues.sort_by_key(|issue| issue.target().source_order());
    MoleculeRenderPlan::new(request.provenance, batches, issues)
}
struct RenderEndpointGeometry {
    kind: RecordKind,
    position: Point2,
    bounds: GlyphBounds,
}

fn render_point_to_geometry(point: RenderPoint) -> Result<Point2, RenderError> {
    Point2::new(point.x(), point.y()).map_err(|error| {
        RenderError::InvalidRequest(format!("render point is invalid geometry: {error}"))
    })
}

fn geometry_to_render_point(point: Point2) -> Result<RenderPoint, RenderIssueKind> {
    RenderPoint::new(point.x(), point.y()).map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("derived bond point is not finite: {error}"),
    })
}
