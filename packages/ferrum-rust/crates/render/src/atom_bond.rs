//! Atom-label and closed bond render-plan generation.
//!
//! This facade owns request-wide ordering and endpoint registration. Focused
//! children lower atom-local depiction and clipped bond geometry independently.

mod atom;
pub(crate) mod bond;
mod final_ink_collision;

use std::collections::{HashMap, HashSet};

use ferrum_core::RecordKind;
use ferrum_geometry::{Point2, Vector2};

use crate::glyph_metrics::GlyphBounds;
use crate::glyph_outline_support::GlyphOutlineSupport;
use crate::render_target::RenderPlanEntryContextV1;
use crate::{
    CompactGroupBondEndpointV1, MoleculeRenderPlanV4, PositiveFinite, RenderBatchV4, RenderError,
    RenderIssue, RenderIssueKind, RenderPaintV3, RenderPoint, RenderProvenance,
    VerifiedMoleculeLabelGlyphMetrics,
};

pub use atom::{
    AtomLabelFacts, AtomLabelFontProfile, AtomMarkRenderFacts, AtomMarkRenderKind,
    AtomNumberLabelFacts, AtomRenderTarget,
};
pub use bond::BondRenderTarget;

use atom::build_atom_batch;
use bond::{NormalBondEndpointClipPolicy, build_bond_batch};
use final_ink_collision::{LabelInkEnvelope, batch_intersects_non_endpoint_label};

/// Explicit gap between final bond ink and visible atom-label ink.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BondInkClearance {
    gap: PositiveFinite,
}

impl BondInkClearance {
    /// Construct the exact resolved gap with no renderer fallback.
    #[must_use]
    pub const fn new(gap: PositiveFinite) -> Self {
        Self { gap }
    }

    /// Return the resolved positive gap.
    #[must_use]
    pub const fn gap(self) -> PositiveFinite {
        self.gap
    }
}

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
    normal_single_clip_policy: NormalBondEndpointClipPolicy,
    line_paint: RenderPaintV3,
    compact_group_endpoints: Vec<CompactGroupBondEndpointV1>,
}

impl AtomBondRenderRequest {
    /// Construct a request whose target identities and source orders are unique.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_with_normal_single_clip_policy(
        provenance: RenderProvenance,
        atoms: Vec<AtomRenderTarget>,
        bonds: Vec<BondRenderTarget>,
        font: AtomLabelFontProfile,
        line_width: PositiveFinite,
        bond_lane_spacing: PositiveFinite,
        normal_single_clip_policy: NormalBondEndpointClipPolicy,
        line_paint: RenderPaintV3,
    ) -> Result<Self, RenderError> {
        let mut identifiers = HashSet::new();
        let mut source_orders = HashSet::new();
        for target in atoms
            .iter()
            .map(AtomRenderTarget::context)
            .chain(bonds.iter().map(BondRenderTarget::context))
        {
            if !identifiers.insert(target.record_id().clone()) {
                return Err(RenderError::InvalidRequest(
                    "atom-bond request has duplicate targets".to_owned(),
                ));
            }
            if !source_orders.insert(target.paint_order()) {
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
            normal_single_clip_policy,
            line_paint,
            compact_group_endpoints: Vec::new(),
        })
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        provenance: RenderProvenance,
        atoms: Vec<AtomRenderTarget>,
        bonds: Vec<BondRenderTarget>,
        font: AtomLabelFontProfile,
        line_width: PositiveFinite,
        bond_lane_spacing: PositiveFinite,
        bond_ink_clearance: BondInkClearance,
        line_paint: RenderPaintV3,
    ) -> Result<Self, RenderError> {
        let normal_single_clip_policy =
            NormalBondEndpointClipPolicy::from_test_facts(line_width, bond_ink_clearance)
                .map_err(|issue| RenderError::InvalidRequest(format!("{issue:?}")))?;
        Self::new_with_normal_single_clip_policy(
            provenance,
            atoms,
            bonds,
            font,
            line_width,
            bond_lane_spacing,
            normal_single_clip_policy,
            line_paint,
        )
    }

    /// Attach typed compact-group attachment geometry for supported exterior bonds.
    pub fn with_compact_group_endpoints(
        mut self,
        compact_group_endpoints: Vec<CompactGroupBondEndpointV1>,
    ) -> Result<Self, RenderError> {
        let mut identifiers = self
            .atoms
            .iter()
            .map(AtomRenderTarget::context)
            .chain(self.bonds.iter().map(BondRenderTarget::context))
            .map(|context| context.record_id().clone())
            .collect::<HashSet<_>>();
        let mut source_orders = self
            .atoms
            .iter()
            .map(AtomRenderTarget::context)
            .chain(self.bonds.iter().map(BondRenderTarget::context))
            .map(RenderPlanEntryContextV1::paint_order)
            .collect::<HashSet<_>>();
        for endpoint in &compact_group_endpoints {
            if !identifiers.insert(endpoint.context().record_id().clone()) {
                return Err(RenderError::InvalidRequest(
                    "atom-bond request has duplicate compact-group endpoint target".to_owned(),
                ));
            }
            if !source_orders.insert(endpoint.context().paint_order()) {
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
pub(crate) fn build_atom_bond_plan(
    request: &AtomBondRenderRequest,
    metrics: &VerifiedMoleculeLabelGlyphMetrics,
) -> Result<MoleculeRenderPlanV4, RenderError> {
    let mut batches = Vec::new();
    let mut issues = Vec::new();
    let mut endpoints = HashMap::new();
    let mut label_envelopes = HashMap::new();

    for atom in &request.atoms {
        let context = atom.context.clone();
        let outcome = atom.visibility.issue("atom target").map_or_else(
            || {
                build_atom_batch(
                    atom,
                    atom.font.as_ref().unwrap_or(&request.font),
                    request.normal_single_clip_policy.clearance().gap(),
                    sole_visible_bond_direction(atom, &request.atoms, &request.bonds)?,
                    metrics,
                )
            },
            |kind| Ok(Err(kind)),
        );
        match outcome? {
            Ok((
                batch,
                bounds,
                attachment,
                core_outline_support,
                label_mask_ink_bounds,
                non_core_run_ink_bounds,
            )) => {
                let center = attachment.core_element_ink_center();
                let position = render_point_to_geometry(atom.position)?
                    .offset(
                        ferrum_geometry::Vector2::new(center.x(), center.y()).map_err(|error| {
                            RenderError::InvalidRequest(format!(
                                "atom-label attachment center is invalid geometry: {error}"
                            ))
                        })?,
                        1.0,
                    )
                    .map_err(|error| {
                        RenderError::InvalidRequest(format!(
                            "atom-label attachment position is invalid geometry: {error}"
                        ))
                    })?;
                endpoints.insert(
                    context.record_id().clone(),
                    RenderEndpointGeometry {
                        kind: RecordKind::Atom,
                        position,
                        // The structural glyph owns optical attachment while
                        // every painted mask and non-core run remains an
                        // explicit final-ink exclusion.
                        clipping: EndpointClipGeometry::AtomLabelInk {
                            core_outline_support,
                            label_mask_ink_bounds,
                            non_core_run_ink_bounds,
                        },
                    },
                );
                let envelope = LabelInkEnvelope::from_local_bounds(
                    bounds,
                    atom.position,
                    request.normal_single_clip_policy.clearance().gap(),
                )
                .map_err(RenderError::InvalidRequest)?;
                label_envelopes.insert(context.record_id().clone(), envelope);
                batches.push(batch);
            }
            Err(kind) => issues.push(RenderIssue::from_context(context, kind)?),
        }
    }

    for endpoint in &request.compact_group_endpoints {
        endpoints.insert(
            endpoint.context().record_id().clone(),
            RenderEndpointGeometry {
                kind: RecordKind::Group,
                position: render_point_to_geometry(endpoint.connection_point())?,
                clipping: EndpointClipGeometry::FixedConnectionPoint {
                    label_ink_exclusion: endpoint.label_ink_exclusion(),
                },
            },
        );
    }

    for bond in &request.bonds {
        let context = bond.context.clone();
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
            let (first_endpoint, second_endpoint) = bond.endpoints();
            build_bond_batch(
                bond,
                &endpoints,
                stroke_width,
                lane_spacing,
                wedge_width,
                request.normal_single_clip_policy,
                paint,
            )
            .and_then(|batch| {
                let crate::RenderBatchContentV4::Bond(content) = batch.content() else {
                    return Err(RenderIssueKind::UnrenderableTarget {
                        reason: "bond lowering did not produce closed bond content".to_owned(),
                    });
                };
                if batch_intersects_non_endpoint_label(
                    content,
                    &label_envelopes,
                    first_endpoint,
                    second_endpoint,
                )
                .map_err(|reason| RenderIssueKind::UnrenderableTarget { reason })?
                {
                    Err(RenderIssueKind::UnrenderableTarget {
                        reason: "bond final ink intersects a non-endpoint atom label".to_owned(),
                    })
                } else {
                    Ok(batch)
                }
            })
        };
        match outcome {
            Ok(batch) => batches.push(batch),
            Err(kind) => issues.push(RenderIssue::from_context(context, kind)?),
        }
    }

    batches.sort_by_key(RenderBatchV4::paint_order);
    issues.sort_by_key(RenderIssue::paint_order);
    MoleculeRenderPlanV4::new(request.provenance, batches, issues)
}

fn sole_visible_bond_direction(
    atom: &AtomRenderTarget,
    atoms: &[AtomRenderTarget],
    bonds: &[BondRenderTarget],
) -> Result<Option<Vector2>, RenderError> {
    let atom_id = atom.context.record_id();
    let mut neighbor = None;
    for bond in bonds {
        if !matches!(&bond.visibility, TargetVisibility::Visible) {
            continue;
        }
        let other_id = if bond.first_endpoint() == atom_id {
            bond.second_endpoint()
        } else if bond.second_endpoint() == atom_id {
            bond.first_endpoint()
        } else {
            continue;
        };
        if neighbor.is_some() {
            return Ok(None);
        }
        neighbor = atoms
            .iter()
            .find(|candidate| candidate.context.record_id() == other_id)
            .map(|candidate| candidate.position);
        if neighbor.is_none() {
            return Ok(None);
        }
    }
    let Some(neighbor) = neighbor else {
        return Ok(None);
    };
    Ok(Vector2::new(
        neighbor.x() - atom.position.x(),
        neighbor.y() - atom.position.y(),
    )
    .and_then(Vector2::normalized)
    .ok())
}
struct RenderEndpointGeometry {
    kind: RecordKind,
    position: Point2,
    clipping: EndpointClipGeometry,
}

#[derive(Clone, Debug)]
enum EndpointClipGeometry {
    /// Exact visible ink of the structural element run at an atom endpoint.
    ///
    /// The exact core outline owns attachment. Painted mask and decoration
    /// rectangles remain exclusions rather than substitute attachment targets.
    AtomLabelInk {
        core_outline_support: GlyphOutlineSupport,
        label_mask_ink_bounds: Option<GlyphBounds>,
        non_core_run_ink_bounds: Vec<GlyphBounds>,
    },
    FixedConnectionPoint {
        label_ink_exclusion: crate::compact_group::CompactGroupLabelInkEnvelope,
    },
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
