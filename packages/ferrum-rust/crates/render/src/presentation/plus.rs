//! API-owned verified glyph layout for direct-root plus signs.

use crate::{
    GlyphBounds, Paint, PositiveFinite, RenderError, RenderPoint, Rgb24, TextOp,
    VerifiedTelexGlyphMetrics,
};
use ferrum_document::{
    PlusProjectionV1, PresentationRecordKindV1, PresentationTargetV1, Rgb24V1 as DocumentRgb24V1,
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

/// One document-root plus with exact verified Telex layout.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DocumentPlusRenderV1 {
    target: PresentationTargetV1,
    anchor: RenderPoint,
    operation: TextOp,
    bounds: PresentationTextBoundsV1,
    background: Option<Paint>,
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
        metrics: &VerifiedTelexGlyphMetrics,
    ) -> Result<Self, RenderError> {
        let foreground = paint(plus.font().color())?;
        let layout = metrics
            .layout_centered_plus(PositiveFinite::new(plus.font().size().value())?, foreground)?;
        Ok(Self {
            target: plus.target().clone(),
            anchor: RenderPoint::new(plus.anchor().x(), plus.anchor().y())?,
            operation: layout.operation().clone(),
            bounds: PresentationTextBoundsV1::from_glyph_bounds(layout.bounds()),
            background: plus.background().color().map(paint).transpose()?,
        })
    }

    fn validated(
        target: PresentationTargetV1,
        anchor: RenderPoint,
        operation: TextOp,
        bounds: PresentationTextBoundsV1,
        background: Option<Paint>,
    ) -> Result<Self, String> {
        if target.record_kind() != PresentationRecordKindV1::Plus {
            return Err("plus render target has the wrong persistent kind".to_owned());
        }
        let environment =
            crate::FerrumFontEnvironmentV1::load().map_err(|error| error.to_string())?;
        let metrics =
            VerifiedTelexGlyphMetrics::new(&environment).map_err(|error| error.to_string())?;
        let expected = metrics
            .layout_centered_plus(operation.size(), operation.paint().clone())
            .map_err(|error| error.to_string())?;
        if expected.operation() != &operation
            || PresentationTextBoundsV1::from_glyph_bounds(expected.bounds()) != bounds
        {
            return Err("plus operation or bounds do not match verified Telex layout".to_owned());
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
    pub fn background(&self) -> Option<&Paint> {
        self.background.as_ref()
    }
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
    background: Option<Paint>,
}

fn paint(color: &DocumentRgb24V1) -> Result<Paint, RenderError> {
    let digits = color.as_str().strip_prefix('#').unwrap_or(color.as_str());
    Ok(Paint::rgb24(Rgb24::new(digits)?))
}
