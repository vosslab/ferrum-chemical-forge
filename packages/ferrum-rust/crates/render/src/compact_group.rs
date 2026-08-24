//! Closed compact-group label primitives derived from typed document projections.

use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_document_projection::CompactGroupProjectionV1;

use crate::{
    BatchSpace, GlyphBounds, LineOp, Paint, PositiveFinite, RenderBatch, RenderError, RenderOp,
    RenderPoint, RenderTarget, VerifiedTelexGlyphMetrics,
};

const GROUP_LABEL_SIZE_PT_V1: f64 = 14.0;
const GROUP_GLYPH_LENGTH_PT_V1: f64 = 7.0;
const GROUP_GLYPH_WIDTH_PT_V1: f64 = 1.0;

/// One rendered compact-group label and its attachment-direction marker.
///
/// This is a display primitive for a first-class document group. It is not an
/// atom or chemistry graph vertex.
#[derive(Clone, Debug, PartialEq)]
pub struct CompactGroupRenderPrimitiveV1 {
    target: RenderTarget,
    identifier: String,
    anchor: RenderPoint,
    attachment: RenderPoint,
    bounds: GlyphBounds,
    batch: RenderBatch,
}

/// Finite group geometry used only as a normal exterior-bond endpoint.
///
/// The compact group remains a label primitive. This endpoint carries its
/// renderer-derived attachment point and local exclusion bounds without
/// turning the group into an atom.
#[derive(Clone, Debug, PartialEq)]
pub struct CompactGroupBondEndpointV1 {
    target: RenderTarget,
    position: RenderPoint,
    bounds: GlyphBounds,
}

impl CompactGroupBondEndpointV1 {
    fn new(
        target: RenderTarget,
        position: RenderPoint,
        bounds: GlyphBounds,
    ) -> Result<Self, RenderError> {
        if target.record_id().kind() != RecordKind::Group {
            return Err(RenderError::InvalidRequest(
                "compact-group bond endpoint requires a group RecordId".to_owned(),
            ));
        }
        Ok(Self {
            target,
            position,
            bounds,
        })
    }

    /// Return the durable compact-group target.
    #[must_use]
    pub fn target(&self) -> &RenderTarget {
        &self.target
    }

    /// Return the finite scene attachment point.
    #[must_use]
    pub const fn position(&self) -> RenderPoint {
        self.position
    }

    /// Return attachment-local label and glyph exclusion bounds.
    #[must_use]
    pub const fn bounds(&self) -> GlyphBounds {
        self.bounds
    }
}

impl CompactGroupRenderPrimitiveV1 {
    /// Lower one typed compact-group projection using Ferrum-owned geometry.
    pub fn from_projection(
        group: &CompactGroupProjectionV1,
        metrics: &VerifiedTelexGlyphMetrics,
        paint: Paint,
    ) -> Result<Self, RenderError> {
        let identifier = group.id().as_str().to_owned();
        let source = Identifier::new(&identifier).map_err(|_| {
            RenderError::InvalidRequest(
                "compact-group identity must be a valid identifier".to_owned(),
            )
        })?;
        let target = RenderTarget::new(
            RecordId::from_source(RecordKind::Group, &source),
            group.source_order(),
        );
        let anchor = RenderPoint::new(group.anchor().x(), group.anchor().y())?;
        let size = PositiveFinite::new(GROUP_LABEL_SIZE_PT_V1)?;
        let layout = metrics.layout_centered_compact_group_label(
            group.catalog_key(),
            size,
            paint.clone(),
        )?;
        let radians = group.orientation_degrees().to_radians();
        let end = RenderPoint::new(
            GROUP_GLYPH_LENGTH_PT_V1 * radians.cos(),
            GROUP_GLYPH_LENGTH_PT_V1 * radians.sin(),
        )?;
        let glyph = LineOp::new(
            RenderPoint::new(0.0, 0.0)?,
            end,
            PositiveFinite::new(GROUP_GLYPH_WIDTH_PT_V1)?,
            paint,
            10,
        )?;
        let batch = RenderBatch::new(
            target.clone(),
            BatchSpace::AtomLocal { anchor },
            vec![
                RenderOp::Line(glyph),
                RenderOp::Text(layout.operation().clone()),
            ],
        )?;
        let bounds = union_bounds(layout.bounds(), end)?;
        let attachment = RenderPoint::new(anchor.x() + end.x(), anchor.y() + end.y())?;
        Ok(Self {
            target,
            identifier,
            anchor,
            attachment,
            bounds,
            batch,
        })
    }

    /// Return the durable group target carried by the render primitive.
    #[must_use]
    pub fn target(&self) -> &RenderTarget {
        &self.target
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
    pub fn batch(&self) -> &RenderBatch {
        &self.batch
    }

    /// Return the renderer-owned endpoint for one supported exterior bond.
    pub fn bond_endpoint(&self) -> Result<CompactGroupBondEndpointV1, RenderError> {
        let offset_x = self.attachment.x() - self.anchor.x();
        let offset_y = self.attachment.y() - self.anchor.y();
        let bounds = GlyphBounds::new(
            self.bounds.min_x() - offset_x,
            self.bounds.min_y() - offset_y,
            self.bounds.max_x() - offset_x,
            self.bounds.max_y() - offset_y,
        )?;
        CompactGroupBondEndpointV1::new(self.target.clone(), self.attachment, bounds)
    }
}

fn union_bounds(label: GlyphBounds, glyph_end: RenderPoint) -> Result<GlyphBounds, RenderError> {
    let half = GROUP_GLYPH_WIDTH_PT_V1 / 2.0;
    GlyphBounds::new(
        label.min_x().min(glyph_end.x() - half).min(-half),
        label.min_y().min(glyph_end.y() - half).min(-half),
        label.max_x().max(glyph_end.x() + half).max(half),
        label.max_y().max(glyph_end.y() + half).max(half),
    )
}
