//! API-owned verified glyph layout for direct-root plus signs.

use crate::glyph_metrics::GlyphBounds;
use crate::{
    PositiveFinite, RenderError, RenderPaintV3, RenderPoint, Rgb24, TextOp,
    VerifiedMoleculeLabelGlyphMetrics,
};
use ferrum_document_projection::{
    PlusProjectionV1, PresentationFactProvenanceV1, PresentationFontFaceV1,
    PresentationRecordKindV1, PresentationTargetV1, Rgb24V1 as DocumentRgb24V1,
};
use serde::{Deserialize, Deserializer, Serialize};

/// Finite nonempty plus ink bounds relative to its authored anchor.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PresentationTextBoundsV1 {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl<'de> Deserialize<'de> for PresentationTextBoundsV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = BoundsWireV1::deserialize(deserializer)?;
        Self::new(wire.left, wire.top, wire.right, wire.bottom)
            .ok_or_else(|| serde::de::Error::custom("invalid plus ink bounds"))
    }
}

impl PresentationTextBoundsV1 {
    pub(crate) fn new(left: f64, top: f64, right: f64, bottom: f64) -> Option<Self> {
        [left, top, right, bottom]
            .iter()
            .all(|value| value.is_finite())
            .then_some(())
            .filter(|()| left < right && top < bottom)
            .filter(|()| left <= 0.0 && right >= 0.0 && top <= 0.0 && bottom >= 0.0)
            .map(|()| Self {
                left,
                top,
                right,
                bottom,
            })
    }

    pub(crate) fn from_glyph_bounds(bounds: GlyphBounds) -> Self {
        Self {
            left: bounds.min_x(),
            top: bounds.min_y(),
            right: bounds.max_x(),
            bottom: bounds.max_y(),
        }
    }

    #[must_use]
    pub const fn left(self) -> f64 {
        self.left
    }

    #[must_use]
    pub const fn top(self) -> f64 {
        self.top
    }

    #[must_use]
    pub const fn right(self) -> f64 {
        self.right
    }

    #[must_use]
    pub const fn bottom(self) -> f64 {
        self.bottom
    }
}

/// One document-root plus with exact verified Atkinson Hyperlegible Next layout.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DocumentPlusRenderV1 {
    target: PresentationTargetV1,
    anchor: RenderPoint,
    operation: TextOp,
    bounds: PresentationTextBoundsV1,
    background: Option<RenderPaintV3>,
}

impl<'de> Deserialize<'de> for DocumentPlusRenderV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PlusRenderWireV1::deserialize(deserializer)?;
        Self::validated(
            wire.target,
            wire.anchor,
            wire.operation,
            wire.bounds,
            wire.background,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl DocumentPlusRenderV1 {
    pub(crate) fn from_projection(
        plus: &PlusProjectionV1,
        metrics: &VerifiedMoleculeLabelGlyphMetrics,
    ) -> Result<Self, RenderError> {
        match plus.font().font_face() {
            PresentationFontFaceV1::MoleculeLabel => {}
        }
        let foreground = paint(plus.font().color(), plus.font().color_provenance())?;
        let layout = metrics
            .layout_centered_plus(PositiveFinite::new(plus.font().size().value())?, foreground)?;
        Ok(Self {
            target: plus.target().clone(),
            anchor: RenderPoint::new(plus.anchor().x(), plus.anchor().y())?,
            operation: layout.operation().clone(),
            bounds: PresentationTextBoundsV1::from_glyph_bounds(layout.bounds()),
            background: plus
                .background()
                .color()
                .map(|color| paint(color, PresentationFactProvenanceV1::Root))
                .transpose()?,
        })
    }

    fn validated(
        target: PresentationTargetV1,
        anchor: RenderPoint,
        operation: TextOp,
        bounds: PresentationTextBoundsV1,
        background: Option<RenderPaintV3>,
    ) -> Result<Self, String> {
        if target.record_kind() != PresentationRecordKindV1::Plus {
            return Err("plus render target has the wrong persistent kind".to_owned());
        }
        let environment =
            crate::FerrumFontEnvironment::load().map_err(|error| error.to_string())?;
        let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
            .map_err(|error| error.to_string())?;
        let expected = metrics
            .layout_centered_plus(operation.size(), operation.paint().clone())
            .map_err(|error| error.to_string())?;
        if expected.operation() != &operation
            || PresentationTextBoundsV1::from_glyph_bounds(expected.bounds()) != bounds
        {
            return Err(
                "plus operation or bounds do not match verified Atkinson Hyperlegible Next layout"
                    .to_owned(),
            );
        }
        Ok(Self {
            target,
            anchor,
            operation,
            bounds,
            background,
        })
    }

    #[must_use]
    pub fn target(&self) -> &PresentationTargetV1 {
        &self.target
    }

    #[must_use]
    pub const fn anchor(&self) -> RenderPoint {
        self.anchor
    }

    #[must_use]
    pub fn operation(&self) -> &TextOp {
        &self.operation
    }

    #[must_use]
    pub const fn bounds(&self) -> PresentationTextBoundsV1 {
        self.bounds
    }

    #[must_use]
    pub fn background(&self) -> Option<&RenderPaintV3> {
        self.background.as_ref()
    }
}

/// Lower the fixed standard Plus appearance for an identifier-free preview.
pub(crate) fn lower_standard_plus_preview_v1(
    metrics: &VerifiedMoleculeLabelGlyphMetrics,
) -> Result<(TextOp, PresentationTextBoundsV1), RenderError> {
    let layout = metrics.layout_centered_plus(
        PositiveFinite::new(14.0)?,
        RenderPaintV3::document_foreground(),
    )?;
    Ok((
        layout.operation().clone(),
        PresentationTextBoundsV1::from_glyph_bounds(layout.bounds()),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BoundsWireV1 {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlusRenderWireV1 {
    target: PresentationTargetV1,
    anchor: RenderPoint,
    operation: TextOp,
    bounds: PresentationTextBoundsV1,
    background: Option<RenderPaintV3>,
}

fn paint(
    color: &DocumentRgb24V1,
    provenance: PresentationFactProvenanceV1,
) -> Result<RenderPaintV3, RenderError> {
    if provenance == PresentationFactProvenanceV1::Builtin {
        return Ok(RenderPaintV3::document_foreground());
    }
    let digits = color.as_str().strip_prefix('#').unwrap_or(color.as_str());
    Ok(RenderPaintV3::authored_rgb24(Rgb24::new(digits)?))
}
