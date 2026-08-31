//! Closed compact-group label primitives derived from typed document projections.

use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_document_projection::{CompactGroupProjectionV1, DocumentObjectIdV1};
use ferrum_geometry::{Point2, Vector2};

use crate::glyph_metrics::GlyphBounds;
use crate::render_target::RenderPlanEntryContextV1;
use crate::{
    CompactGroupRenderBatchV1, CompactGroupRenderOpV1, LineOp, PositiveFinite, RenderBatchV4,
    RenderError, RenderPaintV3, RenderPoint, RenderTarget, VerifiedMoleculeLabelGlyphMetrics,
};

pub(crate) const GROUP_LABEL_SIZE_PT_V1: f64 = 14.0;
pub(crate) const GROUP_LABEL_ATTACHMENT_MARKER_LENGTH_PT_V1: f64 = 7.0;
const GROUP_GLYPH_WIDTH_PT_V1: f64 = 1.0;

/// One rendered compact-group label and its attachment-direction marker.
///
/// This is a display primitive for a first-class document group. It is not an
/// atom or chemistry graph vertex.
#[derive(Clone, Debug, PartialEq)]
pub struct CompactGroupRenderPrimitiveV1 {
    context: RenderPlanEntryContextV1,
    identifier: String,
    anchor: RenderPoint,
    attachment: RenderPoint,
    label_ink_bounds: GlyphBounds,
    bounds: GlyphBounds,
    batch: RenderBatchV4,
}

/// Finite group geometry used only as a normal exterior-bond endpoint.
///
/// The compact group remains a label primitive. This endpoint carries its
/// renderer-derived attachment point and local exclusion bounds without
/// turning the group into an atom.
#[derive(Clone, Debug, PartialEq)]
pub struct CompactGroupBondEndpointV1 {
    context: RenderPlanEntryContextV1,
    connection_point: RenderPoint,
    label_ink_exclusion: CompactGroupLabelInkEnvelope,
}

/// Finite scene-space label ink occupied by a compact-group primitive.
///
/// Unlike `GlyphBounds`, this rectangle need not contain the compact group's
/// exterior bond connection point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CompactGroupLabelInkEnvelope {
    minimum: Point2,
    maximum: Point2,
}

impl CompactGroupLabelInkEnvelope {
    fn from_anchor_local(label: GlyphBounds, anchor: RenderPoint) -> Result<Self, RenderError> {
        let minimum = Point2::new(label.min_x() + anchor.x(), label.min_y() + anchor.y()).map_err(
            |error| {
                RenderError::InvalidRequest(format!(
                    "compact-group label ink minimum is invalid geometry: {error}"
                ))
            },
        )?;
        let maximum = Point2::new(label.max_x() + anchor.x(), label.max_y() + anchor.y()).map_err(
            |error| {
                RenderError::InvalidRequest(format!(
                    "compact-group label ink maximum is invalid geometry: {error}"
                ))
            },
        )?;
        Ok(Self { minimum, maximum })
    }

    /// Return whether a forward ray crosses this envelope's open interior.
    pub(crate) fn ray_enters_interior(&self, origin: Point2, direction: Vector2) -> bool {
        let x = ray_slab(
            self.minimum.x(),
            self.maximum.x(),
            origin.x(),
            direction.x(),
        );
        let y = ray_slab(
            self.minimum.y(),
            self.maximum.y(),
            origin.y(),
            direction.y(),
        );
        let entry = x.0.max(y.0).max(0.0);
        let exit = x.1.min(y.1);
        exit > entry
    }
}

impl CompactGroupBondEndpointV1 {
    fn new(
        context: RenderPlanEntryContextV1,
        connection_point: RenderPoint,
        label_ink_exclusion: CompactGroupLabelInkEnvelope,
    ) -> Result<Self, RenderError> {
        if context.record_id().kind() != RecordKind::Group {
            return Err(RenderError::InvalidRequest(
                "compact-group bond endpoint requires a group RecordId".to_owned(),
            ));
        }
        Ok(Self {
            context,
            connection_point,
            label_ink_exclusion,
        })
    }

    /// Return the durable compact-group target.
    #[must_use]
    pub fn target(&self) -> &RenderTarget {
        self.context.target()
    }

    pub(crate) const fn context(&self) -> &RenderPlanEntryContextV1 {
        &self.context
    }

    /// Return the exact finite scene point where an exterior bond connects.
    #[must_use]
    pub const fn connection_point(&self) -> RenderPoint {
        self.connection_point
    }

    /// Return the scene-space compact-label ink excluded from exterior bonds.
    #[must_use]
    pub(crate) const fn label_ink_exclusion(&self) -> CompactGroupLabelInkEnvelope {
        self.label_ink_exclusion
    }
}

impl CompactGroupRenderPrimitiveV1 {
    /// Lower one typed compact-group projection using Ferrum-owned geometry.
    pub fn from_projection(
        group: &CompactGroupProjectionV1,
        owner_molecule_object_id: &DocumentObjectIdV1,
        metrics: &VerifiedMoleculeLabelGlyphMetrics,
        paint: RenderPaintV3,
    ) -> Result<Self, RenderError> {
        let identifier = group.id().as_str().to_owned();
        let source = Identifier::new(&identifier).map_err(|_| {
            RenderError::InvalidRequest(
                "compact-group identity must be a valid identifier".to_owned(),
            )
        })?;
        let context = RenderPlanEntryContextV1::new(
            RenderTarget::document_object(group.id().clone()),
            RecordId::new(RecordKind::Group, source).map_err(|_| {
                RenderError::InvalidRequest(
                    "compact-group identity must be a valid identifier".to_owned(),
                )
            })?,
            group.source_order(),
            Some(owner_molecule_object_id.clone()),
        );
        let anchor = RenderPoint::new(group.anchor().x(), group.anchor().y())?;
        let size = PositiveFinite::new(GROUP_LABEL_SIZE_PT_V1)?;
        let layout = metrics.layout_centered_compact_group_label(
            group.catalog_key(),
            size,
            paint.clone(),
        )?;
        let release_ray = normalized_attachment_ray(group.orientation_degrees())?;
        let connection_ray = negated_render_point(release_ray)?;
        let (marker_start, marker_end) = label_attachment_marker(layout.bounds(), connection_ray)?;
        let glyph = LineOp::new(
            marker_start,
            marker_end,
            PositiveFinite::new(GROUP_GLYPH_WIDTH_PT_V1)?,
            paint,
            10,
        )?;
        let batch = RenderBatchV4::compact_group(
            context.clone(),
            CompactGroupRenderBatchV1::new(
                anchor,
                vec![
                    CompactGroupRenderOpV1::Line(glyph),
                    CompactGroupRenderOpV1::Text(layout.operation().clone()),
                ],
            )?,
        )?;
        let bounds = union_bounds(layout.bounds(), marker_start, marker_end)?;
        let attachment =
            RenderPoint::new(anchor.x() + marker_end.x(), anchor.y() + marker_end.y())?;
        Ok(Self {
            context,
            identifier,
            anchor,
            attachment,
            label_ink_bounds: layout.bounds(),
            bounds,
            batch,
        })
    }

    /// Return the durable group target carried by the render primitive.
    #[must_use]
    pub fn target(&self) -> &RenderTarget {
        self.context.target()
    }

    /// Return the durable compact-group identifier.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Return the document-scene anchor.
    #[must_use]
    pub const fn anchor(&self) -> RenderPoint {
        self.anchor
    }

    /// Return the finite scene point at the compact group's declared attachment direction.
    #[must_use]
    pub const fn attachment(&self) -> RenderPoint {
        self.attachment
    }

    /// Return the finite anchor-local hit and paint bounds.
    #[must_use]
    pub const fn bounds(&self) -> GlyphBounds {
        self.bounds
    }

    /// Return the one exact render batch used in the molecule plan.
    #[must_use]
    pub fn batch(&self) -> &RenderBatchV4 {
        &self.batch
    }

    /// Return the renderer-owned endpoint for one supported exterior bond.
    pub fn bond_endpoint(&self) -> Result<CompactGroupBondEndpointV1, RenderError> {
        let label_ink_exclusion =
            CompactGroupLabelInkEnvelope::from_anchor_local(self.label_ink_bounds, self.anchor)?;
        CompactGroupBondEndpointV1::new(self.context.clone(), self.attachment, label_ink_exclusion)
    }
}

fn normalized_attachment_ray(orientation_degrees: f64) -> Result<RenderPoint, RenderError> {
    let radians = orientation_degrees.to_radians();
    let ray_x = radians.cos();
    let ray_y = radians.sin();
    let length = ray_x.hypot(ray_y);
    if !length.is_finite() || length <= 0.0 {
        return Err(RenderError::InvalidRequest(
            "compact-group orientation must produce a finite direction".to_owned(),
        ));
    }
    RenderPoint::new(ray_x / length, ray_y / length)
}

/// Return the renderer-issued distance from a compact-group anchor back to its
/// exterior-bond connection point along `direction_from_atom`.
pub(crate) fn compact_group_marker_back_distance(
    label: GlyphBounds,
    direction_from_atom: RenderPoint,
) -> Result<f64, RenderError> {
    let connection_ray = negated_render_point(direction_from_atom)?;
    let (marker_start, marker_end) = label_attachment_marker(label, connection_ray)?;
    let distance = marker_end.x().hypot(marker_end.y());
    if !distance.is_finite() || distance <= 0.0 {
        return Err(RenderError::InvalidRequest(
            "compact-group marker-back distance is not representable".to_owned(),
        ));
    }
    let _ = marker_start;
    Ok(distance)
}

fn negated_render_point(point: RenderPoint) -> Result<RenderPoint, RenderError> {
    RenderPoint::new(-point.x(), -point.y())
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

fn label_attachment_marker(
    label: GlyphBounds,
    normalized_ray: RenderPoint,
) -> Result<(RenderPoint, RenderPoint), RenderError> {
    let ray_x = normalized_ray.x();
    let ray_y = normalized_ray.y();
    let x_distance = if ray_x > 0.0 {
        label.max_x() / ray_x
    } else if ray_x < 0.0 {
        label.min_x() / ray_x
    } else {
        f64::INFINITY
    };
    let y_distance = if ray_y > 0.0 {
        label.max_y() / ray_y
    } else if ray_y < 0.0 {
        label.min_y() / ray_y
    } else {
        f64::INFINITY
    };
    let edge_distance = x_distance.min(y_distance);
    if !edge_distance.is_finite() || edge_distance <= 0.0 {
        return Err(RenderError::InvalidRequest(
            "compact-group label bounds must intersect the attachment direction".to_owned(),
        ));
    }
    let marker_start = RenderPoint::new(ray_x * edge_distance, ray_y * edge_distance)?;
    let marker_end = RenderPoint::new(
        ray_x * (edge_distance + GROUP_LABEL_ATTACHMENT_MARKER_LENGTH_PT_V1),
        ray_y * (edge_distance + GROUP_LABEL_ATTACHMENT_MARKER_LENGTH_PT_V1),
    )?;
    Ok((marker_start, marker_end))
}

fn union_bounds(
    label: GlyphBounds,
    marker_start: RenderPoint,
    marker_end: RenderPoint,
) -> Result<GlyphBounds, RenderError> {
    let half = GROUP_GLYPH_WIDTH_PT_V1 / 2.0;
    GlyphBounds::new(
        label
            .min_x()
            .min(marker_start.x() - half)
            .min(marker_end.x() - half),
        label
            .min_y()
            .min(marker_start.y() - half)
            .min(marker_end.y() - half),
        label
            .max_x()
            .max(marker_start.x() + half)
            .max(marker_end.x() + half),
        label
            .max_y()
            .max(marker_start.y() + half)
            .max(marker_end.y() + half),
    )
}

#[cfg(test)]
mod tests {
    use ferrum_core::{Identifier, RecordId, RecordKind};
    use ferrum_document_model::CompactGroupCatalogKeyV1;
    use ferrum_document_projection::{
        CompactGroupAttachmentV1, CompactGroupV1, DocumentObjectIdV1, Point3V1,
    };

    use super::*;
    use crate::atom_bond::bond::NormalBondEndpointClipPolicy;
    use crate::atom_bond::build_atom_bond_plan;
    use crate::attached_compact_group_pose::{
        AttachedCompactGroupAnchorRenderFacts, resolve_attached_compact_group_pose,
    };
    use crate::render_target::RenderPlanEntryContextV1;
    use crate::{
        AtomBondRenderRequest, AtomLabelFacts, AtomLabelFontProfile, AtomRenderTarget,
        AttachedCompactGroupPlacementDispositionV1, BondInkClearance, BondRenderTarget, BondStyle,
        FerrumFontEnvironment, FontFace, RenderOp, RenderProvenance, RenderRevision, Rgb24,
        TargetVisibility,
    };

    fn point(x: f64, y: f64) -> RenderPoint {
        RenderPoint::new(x, y).expect("test point is finite")
    }

    fn positive(value: f64) -> PositiveFinite {
        PositiveFinite::new(value).expect("test extent is positive and finite")
    }

    fn paint() -> RenderPaintV3 {
        RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test RGB is valid"))
    }

    fn object_id(entropy: u8) -> DocumentObjectIdV1 {
        DocumentObjectIdV1::from_entropy_bytes([entropy; 16])
    }

    fn record_id(kind: RecordKind, value: &str) -> RecordId {
        RecordId::new(
            kind,
            Identifier::new(value).expect("test identifier is valid"),
        )
        .expect("test record ID is valid")
    }

    fn context(
        entropy: u8,
        kind: RecordKind,
        value: &str,
        paint_order: u32,
    ) -> RenderPlanEntryContextV1 {
        RenderPlanEntryContextV1::new(
            RenderTarget::document_object(object_id(entropy)),
            record_id(kind, value),
            paint_order,
            Some(object_id(0xb6)),
        )
    }

    fn metrics() -> VerifiedMoleculeLabelGlyphMetrics {
        let environment =
            FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
        VerifiedMoleculeLabelGlyphMetrics::new(&environment)
            .expect("verified Atkinson Hyperlegible Next opens")
    }

    fn compact_group_projection(orientation_degrees: f64) -> CompactGroupProjectionV1 {
        let catalog_key = CompactGroupCatalogKeyV1::Methoxy;
        let attachment = CompactGroupAttachmentV1::new(catalog_key, 0, orientation_degrees)
            .expect("Methoxy accepts its attachment orientation");
        let group = CompactGroupV1::new(
            object_id(0x12),
            catalog_key,
            Point3V1::new(0.0, 0.0, 0.0).expect("finite compact-group anchor"),
            attachment,
        );
        CompactGroupProjectionV1::from_group(&group, 3)
    }

    fn attached_compact_group_facts(anchor: RenderPoint) -> AttachedCompactGroupAnchorRenderFacts {
        AttachedCompactGroupAnchorRenderFacts::new(
            anchor,
            AtomLabelFacts::new("C", None, 0, 0).expect("atom label facts"),
            AtomLabelFontProfile::new(FontFace::molecule_label(), positive(10.0), paint()),
            paint(),
            NormalBondEndpointClipPolicy::from_test_facts(
                positive(1.0),
                BondInkClearance::new(positive(1.25)),
            )
            .expect("test normal-single clipping policy"),
        )
    }

    #[test]
    fn snapped_diagonal_attached_pose_lowers_an_exterior_bond_without_an_issue() {
        let metrics = metrics();
        let atom_position = point(0.0, 0.0);
        let pose = resolve_attached_compact_group_pose(
            &attached_compact_group_facts(atom_position),
            CompactGroupCatalogKeyV1::Methoxy,
            point(1.0, 1.0),
            &metrics,
        )
        .expect("short diagonal release resolves");
        assert_eq!(
            pose.disposition(),
            AttachedCompactGroupPlacementDispositionV1::SnappedOutward
        );
        let catalog_key = CompactGroupCatalogKeyV1::Methoxy;
        let attachment = CompactGroupAttachmentV1::new(catalog_key, 0, pose.orientation_degrees())
            .expect("resolved orientation is a valid attachment");
        let group = CompactGroupV1::new(
            object_id(0x12),
            catalog_key,
            Point3V1::new(pose.anchor().x(), pose.anchor().y(), 0.0)
                .expect("resolved anchor is finite"),
            attachment,
        );
        let primitive = CompactGroupRenderPrimitiveV1::from_projection(
            &CompactGroupProjectionV1::from_group(&group, 3),
            &object_id(0xb6),
            &metrics,
            paint(),
        )
        .expect("resolved compact group lowers to the real primitive");
        let compact_endpoint = primitive.bond_endpoint().expect("renderer endpoint");
        let exterior_bond = BondRenderTarget::new(
            context(0x13, RecordKind::Bond, "exterior", 2),
            record_id(RecordKind::Atom, "anchor"),
            compact_endpoint.context().record_id().clone(),
            BondStyle::NormalSingle,
            TargetVisibility::Visible,
        )
        .expect("exterior bond target");
        let atom = AtomRenderTarget::new(
            context(0x11, RecordKind::Atom, "anchor", 1),
            // The final lowerer receives the same anchor used by pose
            // admission. A short same-ray release therefore proves the shared
            // normal-single policy emits an exterior bond after snapping.
            atom_position,
            AtomLabelFacts::new("C", None, 0, 0).expect("atom label facts"),
            TargetVisibility::Visible,
        )
        .expect("atom target");
        let request = AtomBondRenderRequest::new(
            RenderProvenance::new(RenderRevision::new(1).expect("revision"), [0x22; 32]),
            vec![atom],
            vec![exterior_bond],
            AtomLabelFontProfile::new(FontFace::molecule_label(), positive(10.0), paint()),
            positive(1.0),
            positive(6.0),
            BondInkClearance::new(positive(1.25)),
            paint(),
        )
        .expect("render request")
        .with_compact_group_endpoints(vec![compact_endpoint])
        .expect("compact endpoint registration");
        let plan = build_atom_bond_plan(&request, &metrics).expect("render plan");
        assert!(plan.issues().is_empty());
        assert!(plan.batches().iter().any(|batch| {
            batch.paint_order() == 2
                && matches!(batch.operations().first(), Some(RenderOp::Line(_)))
        }));
    }

    #[test]
    fn wide_compact_label_connects_exterior_bonds_at_its_declared_marker() {
        let metrics = metrics();

        for (orientation_degrees, compact_group_is_first) in
            [(0.0, false), (0.0, true), (45.0, false), (45.0, true)]
        {
            let projection = compact_group_projection(orientation_degrees);
            let primitive = CompactGroupRenderPrimitiveV1::from_projection(
                &projection,
                &object_id(0xb6),
                &metrics,
                paint(),
            )
            .expect("real compact-group render primitive");
            let compact_endpoint = primitive.bond_endpoint().expect("renderer endpoint");
            let connection_point = compact_endpoint.connection_point();
            let label_ink_exclusion = compact_endpoint.label_ink_exclusion();
            let group_record_id = compact_endpoint.context().record_id().clone();
            let release_ray =
                normalized_attachment_ray(orientation_degrees).expect("finite orientation");
            let atom_position = point(
                primitive.anchor().x() - (40.0 * release_ray.x()),
                primitive.anchor().y() - (40.0 * release_ray.y()),
            );
            let atom = AtomRenderTarget::new(
                context(0x11, RecordKind::Atom, "anchor", 1),
                atom_position,
                AtomLabelFacts::new("C", None, 0, 0).expect("atom label facts"),
                TargetVisibility::Visible,
            )
            .expect("atom target");
            let (first_endpoint, second_endpoint) = if compact_group_is_first {
                (
                    group_record_id.clone(),
                    record_id(RecordKind::Atom, "anchor"),
                )
            } else {
                (
                    record_id(RecordKind::Atom, "anchor"),
                    group_record_id.clone(),
                )
            };
            let exterior_bond = BondRenderTarget::new(
                context(0x13, RecordKind::Bond, "exterior", 2),
                first_endpoint,
                second_endpoint,
                BondStyle::NormalSingle,
                TargetVisibility::Visible,
            )
            .expect("exterior bond target");
            let request = AtomBondRenderRequest::new(
                RenderProvenance::new(RenderRevision::new(1).expect("revision"), [0x22; 32]),
                vec![atom],
                vec![exterior_bond],
                AtomLabelFontProfile::new(FontFace::molecule_label(), positive(10.0), paint()),
                positive(1.0),
                positive(6.0),
                BondInkClearance::new(positive(1.25)),
                paint(),
            )
            .expect("render request")
            .with_compact_group_endpoints(vec![compact_endpoint])
            .expect("compact endpoint registration");
            let plan = build_atom_bond_plan(&request, &metrics).expect("render plan");
            assert!(plan.issues().is_empty());
            let batch = plan
                .batches()
                .iter()
                .find(|batch| batch.paint_order() == 2)
                .expect("exterior bond has an admitted batch");
            let RenderOp::Line(line) = &batch.operations()[0] else {
                panic!("admitted exterior bond is a line operation");
            };
            let geometry_tolerance = 1.0e-10;
            let compact_line_end = if compact_group_is_first {
                line.start()
            } else {
                line.end()
            };
            let atom_line_end = if compact_group_is_first {
                line.end()
            } else {
                line.start()
            };
            assert!(
                (compact_line_end.x() - connection_point.x()).abs() <= geometry_tolerance
                    && (compact_line_end.y() - connection_point.y()).abs() <= geometry_tolerance,
                "the exterior bond terminates at the renderer-declared compact connection point"
            );
            let connection_from_anchor_x = compact_line_end.x() - primitive.anchor().x();
            let connection_from_anchor_y = compact_line_end.y() - primitive.anchor().y();
            assert!(
                (connection_from_anchor_x * release_ray.x())
                    + (connection_from_anchor_y * release_ray.y())
                    < 0.0,
                "the compact marker lies on the anchor-facing side of the label"
            );
            let line_end = Point2::new(compact_line_end.x(), compact_line_end.y())
                .expect("rendered compact line end is finite");
            assert!(
                !label_ink_exclusion.ray_enters_interior(
                    line_end,
                    Vector2::new(
                        atom_line_end.x() - compact_line_end.x(),
                        atom_line_end.y() - compact_line_end.y(),
                    )
                    .expect("outward exterior-bond direction is finite"),
                ),
                "the exterior bond does not cross compact-label ink"
            );
        }
    }

    #[test]
    fn compact_endpoint_refuses_an_approach_through_label_ink() {
        let metrics = metrics();
        let projection = compact_group_projection(0.0);
        let primitive = CompactGroupRenderPrimitiveV1::from_projection(
            &projection,
            &object_id(0xb6),
            &metrics,
            paint(),
        )
        .expect("real compact-group render primitive");
        let compact_endpoint = primitive.bond_endpoint().expect("renderer endpoint");
        let group_record_id = compact_endpoint.context().record_id().clone();
        let atom = AtomRenderTarget::new(
            context(0x11, RecordKind::Atom, "far-side-atom", 1),
            point(40.0, 0.0),
            AtomLabelFacts::new("C", None, 0, 0).expect("atom label facts"),
            TargetVisibility::Visible,
        )
        .expect("atom target");
        for compact_group_is_first in [false, true] {
            let (first_endpoint, second_endpoint) = if compact_group_is_first {
                (
                    group_record_id.clone(),
                    record_id(RecordKind::Atom, "far-side-atom"),
                )
            } else {
                (
                    record_id(RecordKind::Atom, "far-side-atom"),
                    group_record_id.clone(),
                )
            };
            let bond = BondRenderTarget::new(
                context(0x13, RecordKind::Bond, "through-label", 2),
                first_endpoint,
                second_endpoint,
                BondStyle::NormalSingle,
                TargetVisibility::Visible,
            )
            .expect("bond target");
            let request = AtomBondRenderRequest::new(
                RenderProvenance::new(RenderRevision::new(1).expect("revision"), [0x22; 32]),
                vec![atom.clone()],
                vec![bond],
                AtomLabelFontProfile::new(FontFace::molecule_label(), positive(10.0), paint()),
                positive(1.0),
                positive(6.0),
                BondInkClearance::new(positive(1.25)),
                paint(),
            )
            .expect("render request")
            .with_compact_group_endpoints(vec![compact_endpoint.clone()])
            .expect("compact endpoint registration");
            let plan = build_atom_bond_plan(&request, &metrics).expect("render plan");
            assert_eq!(plan.issues().len(), 1);
            assert!(matches!(
                plan.issues()[0].kind(),
                crate::RenderIssueKind::UnrenderableTarget { reason }
                    if reason == "compact-group exterior bond approaches through label ink"
            ));
        }
    }
}
