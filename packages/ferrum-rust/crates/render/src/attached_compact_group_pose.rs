//! Internal renderer geometry for one attached compact-group label.

use ferrum_document_model::CompactGroupCatalogKeyV1;
use ferrum_geometry::Vector2;

use crate::atom_bond::bond::NormalBondEndpointClipPolicy;
use crate::compact_group::{GROUP_LABEL_SIZE_PT_V1, compact_group_marker_back_distance};
use crate::glyph_metrics::GlyphMetrics;
use crate::{
    AtomLabelFacts, AtomLabelFontProfile, PositiveFinite, RenderPaintV3, RenderPoint,
    VerifiedMoleculeLabelGlyphMetrics,
};

/// Renderer facts resolved from one projection/profile pair.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AttachedCompactGroupAnchorRenderFacts {
    anchor: RenderPoint,
    atom_label: AtomLabelFacts,
    atom_label_font: AtomLabelFontProfile,
    compact_group_paint: RenderPaintV3,
    normal_single_clip_policy: NormalBondEndpointClipPolicy,
}

impl AttachedCompactGroupAnchorRenderFacts {
    pub(crate) const fn new(
        anchor: RenderPoint,
        atom_label: AtomLabelFacts,
        atom_label_font: AtomLabelFontProfile,
        compact_group_paint: RenderPaintV3,
        normal_single_clip_policy: NormalBondEndpointClipPolicy,
    ) -> Self {
        Self {
            anchor,
            atom_label,
            atom_label_font,
            compact_group_paint,
            normal_single_clip_policy,
        }
    }
}

/// Resolve one finite release into the first final-bond-admissible pose.
pub(crate) fn resolve_attached_compact_group_pose(
    facts: &AttachedCompactGroupAnchorRenderFacts,
    catalog_key: CompactGroupCatalogKeyV1,
    raw_release: RenderPoint,
    metrics: &VerifiedMoleculeLabelGlyphMetrics,
) -> Result<ResolvedAttachedCompactGroupPoseV1, AttachedCompactGroupPoseErrorV2> {
    let delta_x = raw_release.x() - facts.anchor.x();
    let delta_y = raw_release.y() - facts.anchor.y();
    let raw_radius = delta_x.hypot(delta_y);
    if !delta_x.is_finite() || !delta_y.is_finite() || !raw_radius.is_finite() || raw_radius == 0.0
    {
        return Err(AttachedCompactGroupPoseErrorV2::InvalidReleaseDirection);
    }
    let direction = RenderPoint::new(delta_x / raw_radius, delta_y / raw_radius)
        .map_err(|_| AttachedCompactGroupPoseErrorV2::InvalidReleaseDirection)?;
    let direction_vector = Vector2::new(direction.x(), direction.y())
        .map_err(|_| AttachedCompactGroupPoseErrorV2::InvalidReleaseDirection)?;
    let atom_bounds = metrics
        .layout_atom_label(&facts.atom_label, &facts.atom_label_font)
        .map_err(|error| AttachedCompactGroupPoseErrorV2::RenderGeometry(error.to_string()))?
        .bounds();
    let compact_bounds = metrics
        .layout_centered_compact_group_label(
            catalog_key,
            PositiveFinite::new(GROUP_LABEL_SIZE_PT_V1)
                .expect("built-in compact-group label size is positive"),
            facts.compact_group_paint.clone(),
        )
        .map_err(|error| AttachedCompactGroupPoseErrorV2::RenderGeometry(error.to_string()))?
        .bounds();
    let atom_exit = facts
        .normal_single_clip_policy
        .atom_label_forward_exit_distance(atom_bounds, direction_vector)
        .map_err(|error| AttachedCompactGroupPoseErrorV2::RenderGeometry(format!("{error:?}")))?;
    let marker_back = compact_group_marker_back_distance(compact_bounds, direction)
        .map_err(|error| AttachedCompactGroupPoseErrorV2::RenderGeometry(error.to_string()))?;
    let minimum_radius = atom_exit + marker_back;
    if !minimum_radius.is_finite() {
        return Err(AttachedCompactGroupPoseErrorV2::UnrepresentableResolvedAnchor);
    }
    let (minimum_radius, disposition) = if facts
        .normal_single_clip_policy
        .has_positive_visible_segment(raw_radius, atom_exit, marker_back)
    {
        (
            raw_radius,
            AttachedCompactGroupPlacementDispositionV1::AtPointer,
        )
    } else {
        (
            minimum_radius.next_up(),
            AttachedCompactGroupPlacementDispositionV1::SnappedOutward,
        )
    };
    let representable_radius = minimum_distinct_scene_radius(facts.anchor, direction)?;
    let initial_radius = minimum_radius.max(representable_radius);
    let radius_candidates = [
        initial_radius,
        initial_radius
            .mul_add(1.0, representable_radius * 2.0)
            .max(initial_radius.next_up()),
    ];
    for radius in radius_candidates {
        if !radius.is_finite() {
            continue;
        }
        let anchor = RenderPoint::new(
            facts.anchor.x() + direction.x() * radius,
            facts.anchor.y() + direction.y() * radius,
        )
        .map_err(|_| AttachedCompactGroupPoseErrorV2::UnrepresentableResolvedAnchor)?;
        let realized_radius = (anchor.x() - facts.anchor.x()).hypot(anchor.y() - facts.anchor.y());
        if facts
            .normal_single_clip_policy
            .has_positive_visible_segment(realized_radius, atom_exit, marker_back)
        {
            return Ok(ResolvedAttachedCompactGroupPoseV1 {
                anchor,
                orientation_degrees: direction.y().atan2(direction.x()).to_degrees(),
                disposition,
            });
        }
    }
    Err(AttachedCompactGroupPoseErrorV2::UnrepresentableResolvedAnchor)
}

/// Whether the durable group anchor already met renderer clearance requirements.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachedCompactGroupPlacementDispositionV1 {
    /// The supplied release point was drawable without normalization.
    AtPointer,
    /// The finite release direction was retained while its anchor moved outward.
    SnappedOutward,
}

/// One renderer-admitted compact-group anchor and orientation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedAttachedCompactGroupPoseV1 {
    anchor: RenderPoint,
    orientation_degrees: f64,
    disposition: AttachedCompactGroupPlacementDispositionV1,
}

impl ResolvedAttachedCompactGroupPoseV1 {
    #[must_use]
    pub const fn anchor(self) -> RenderPoint {
        self.anchor
    }

    #[must_use]
    pub const fn orientation_degrees(self) -> f64 {
        self.orientation_degrees
    }

    #[must_use]
    pub const fn disposition(self) -> AttachedCompactGroupPlacementDispositionV1 {
        self.disposition
    }
}

/// Closed failures for the renderer-owned attached compact-group facade.
#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
pub enum AttachedCompactGroupPoseErrorV2 {
    /// The projection/profile could not issue required depiction facts.
    #[error("attached compact-group depiction facts could not be resolved: {detail}")]
    Depiction { detail: String },
    /// The verified Ferrum Atkinson Hyperlegible Next resource could not be opened.
    #[error(
        "attached compact-group Atkinson Hyperlegible Next resource could not be resolved: {detail}"
    )]
    FontResource { detail: String },
    /// The raw release cannot identify a finite, nonzero placement ray.
    #[error("attached compact-group release requires a finite nonzero direction")]
    InvalidReleaseDirection,
    /// No finite scene anchor can preserve the requested direction and clearance.
    #[error("attached compact-group anchor is not representable")]
    UnrepresentableResolvedAnchor,
    /// Existing renderer label geometry could not be resolved.
    #[error("attached compact-group renderer geometry failed: {0}")]
    RenderGeometry(String),
}

/// Return a radius that can move every directed scene coordinate by one ULP.
fn minimum_distinct_scene_radius(
    anchor: RenderPoint,
    direction: RenderPoint,
) -> Result<f64, AttachedCompactGroupPoseErrorV2> {
    let mut radius = 0.0_f64;
    for (coordinate, component) in [(anchor.x(), direction.x()), (anchor.y(), direction.y())] {
        if component == 0.0 {
            continue;
        }
        let adjacent_coordinate = if component.is_sign_positive() {
            coordinate.next_up()
        } else {
            coordinate.next_down()
        };
        if !adjacent_coordinate.is_finite() {
            return Err(AttachedCompactGroupPoseErrorV2::UnrepresentableResolvedAnchor);
        }
        let coordinate_ulp = (adjacent_coordinate - coordinate).abs();
        let component_radius = coordinate_ulp / component.abs();
        if !component_radius.is_finite() {
            return Err(AttachedCompactGroupPoseErrorV2::UnrepresentableResolvedAnchor);
        }
        radius = radius.max(component_radius.next_up());
    }
    if radius.is_finite() && radius > 0.0 {
        Ok(radius)
    } else {
        Err(AttachedCompactGroupPoseErrorV2::UnrepresentableResolvedAnchor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FerrumFontEnvironment, FontFace};

    fn metrics() -> VerifiedMoleculeLabelGlyphMetrics {
        let environment =
            FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
        VerifiedMoleculeLabelGlyphMetrics::new(&environment)
            .expect("verified Atkinson Hyperlegible Next opens")
    }

    fn facts(anchor: RenderPoint) -> AttachedCompactGroupAnchorRenderFacts {
        AttachedCompactGroupAnchorRenderFacts::new(
            anchor,
            AtomLabelFacts::new("Cl", None, 0, 0).expect("valid atom label"),
            AtomLabelFontProfile::new(
                FontFace::molecule_label(),
                PositiveFinite::new(12.0).expect("positive font size"),
                RenderPaintV3::document_foreground(),
            ),
            RenderPaintV3::document_foreground(),
            NormalBondEndpointClipPolicy::from_depiction(
                PositiveFinite::new(1.0).expect("positive normal-single width"),
                PositiveFinite::new(12.0).expect("positive font size"),
            )
            .expect("normal-single clipping policy resolves"),
        )
    }

    #[test]
    fn short_finite_release_snaps_outward_on_the_same_ray() {
        let pose = resolve_attached_compact_group_pose(
            &facts(RenderPoint::new(0.0, 0.0).expect("finite anchor")),
            CompactGroupCatalogKeyV1::Methyl,
            RenderPoint::new(1.0, 1.0).expect("finite release"),
            &metrics(),
        )
        .expect("short release resolves outward");
        assert_eq!(
            pose.disposition(),
            AttachedCompactGroupPlacementDispositionV1::SnappedOutward
        );
        assert!(pose.anchor().x() > 1.0);
        assert!((pose.anchor().x() - pose.anchor().y()).abs() < 1.0e-12);
    }

    #[test]
    fn drawable_release_preserves_its_anchor_and_direction() {
        let release = RenderPoint::new(200.0, 0.0).expect("finite release");
        let pose = resolve_attached_compact_group_pose(
            &facts(RenderPoint::new(0.0, 0.0).expect("finite anchor")),
            CompactGroupCatalogKeyV1::Methoxy,
            release,
            &metrics(),
        )
        .expect("release is already drawable");
        assert_eq!(
            pose.disposition(),
            AttachedCompactGroupPlacementDispositionV1::AtPointer
        );
        assert_eq!(pose.anchor(), release);
        assert_eq!(pose.orientation_degrees(), 0.0);
    }

    #[test]
    fn coincident_release_has_a_closed_direction_refusal() {
        let error = resolve_attached_compact_group_pose(
            &facts(RenderPoint::new(0.0, 0.0).expect("finite anchor")),
            CompactGroupCatalogKeyV1::Methyl,
            RenderPoint::new(0.0, 0.0).expect("finite release"),
            &metrics(),
        )
        .expect_err("coincident release has no direction");
        assert_eq!(
            error,
            AttachedCompactGroupPoseErrorV2::InvalidReleaseDirection
        );
    }

    #[test]
    fn large_finite_anchor_has_a_closed_placement_outcome() {
        let anchor = RenderPoint::new(f64::MAX / 4.0, 0.0).expect("finite anchor");
        let raw_release = RenderPoint::new(anchor.x(), 1.0).expect("finite release");
        let outcome = resolve_attached_compact_group_pose(
            &facts(anchor),
            CompactGroupCatalogKeyV1::Methyl,
            raw_release,
            &metrics(),
        );
        assert!(matches!(
            outcome,
            Ok(ResolvedAttachedCompactGroupPoseV1 {
                disposition: AttachedCompactGroupPlacementDispositionV1::SnappedOutward,
                ..
            }) | Err(AttachedCompactGroupPoseErrorV2::UnrepresentableResolvedAnchor)
        ));
    }
}
