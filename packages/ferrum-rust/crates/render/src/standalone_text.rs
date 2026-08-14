//! Closed verified-Telex layouts for direct-root presentation text.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    FontFace, GlyphBounds, GlyphPlacement, Paint, PositiveFinite, RenderError, RenderPoint, TextOp,
    TextScript,
};

/// One exact text operation and its anchor-local ink bounds.
#[derive(Clone, Debug, PartialEq)]
pub struct CenteredTextLayout {
    operation: TextOp,
    bounds: GlyphBounds,
}

impl CenteredTextLayout {
    pub(crate) fn new(operation: TextOp, bounds: GlyphBounds) -> Self {
        Self { operation, bounds }
    }

    /// Return the exact Telex operation centered around the local origin.
    #[must_use]
    pub fn operation(&self) -> &TextOp {
        &self.operation
    }

    /// Return finite nonempty ink bounds relative to the authored anchor.
    #[must_use]
    pub const fn bounds(&self) -> GlyphBounds {
        self.bounds
    }
}

/// One source character-data run after CDML formatting has been normalized.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationTextSourceRun {
    text: String,
    script: TextScript,
}

impl PresentationTextSourceRun {
    /// Construct one source run. Newlines are retained as line breaks.
    pub fn new(text: impl Into<String>, script: TextScript) -> Result<Self, RenderError> {
        let text = text.into();
        if text.is_empty()
            || text
                .chars()
                .any(|character| character.is_control() && character != '\n')
        {
            return Err(RenderError::InvalidRequest(
                "presentation text run must be nonempty and contain no controls except newline"
                    .to_owned(),
            ));
        }
        Ok(Self { text, script })
    }

    /// Return exact rendered character data, including authored newlines.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the sole supported baseline role for this run.
    #[must_use]
    pub const fn script(&self) -> TextScript {
        self.script
    }
}

impl<'de> Deserialize<'de> for PresentationTextSourceRun {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            text: String,
            script: TextScript,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.text, wire.script).map_err(serde::de::Error::custom)
    }
}

/// One exact single-line Telex run positioned relative to a Text anchor.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationGlyphRun {
    text: String,
    script: TextScript,
    origin: RenderPoint,
    glyphs: Vec<GlyphPlacement>,
    scale: PositiveFinite,
}

impl PresentationGlyphRun {
    pub(crate) fn new(
        text: String,
        script: TextScript,
        origin: RenderPoint,
        glyphs: Vec<GlyphPlacement>,
        scale: PositiveFinite,
    ) -> Result<Self, RenderError> {
        if text.is_empty() || text.chars().any(char::is_control) {
            return Err(RenderError::InvalidRequest(
                "laid-out presentation text run must be nonempty and single-line".to_owned(),
            ));
        }
        if glyphs.len() != text.chars().count() {
            return Err(RenderError::InvalidRequest(
                "presentation text requires one Telex glyph placement per Unicode scalar"
                    .to_owned(),
            ));
        }
        Ok(Self {
            text,
            script,
            origin,
            glyphs,
            scale,
        })
    }

    /// Return exact single-line rendered character data.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the semantic baseline role.
    #[must_use]
    pub const fn script(&self) -> TextScript {
        self.script
    }

    /// Return the explicit anchor-local run origin.
    #[must_use]
    pub const fn origin(&self) -> RenderPoint {
        self.origin
    }

    /// Return exact face-local glyph IDs and run-local origins.
    #[must_use]
    pub fn glyphs(&self) -> &[GlyphPlacement] {
        &self.glyphs
    }

    /// Return the exact scale applied to the operation font size.
    #[must_use]
    pub const fn scale(&self) -> PositiveFinite {
        self.scale
    }
}

impl<'de> Deserialize<'de> for PresentationGlyphRun {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            text: String,
            script: TextScript,
            origin: RenderPoint,
            glyphs: Vec<GlyphPlacement>,
            scale: PositiveFinite,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.text, wire.script, wire.origin, wire.glyphs, wire.scale)
            .map_err(serde::de::Error::custom)
    }
}

/// Complete direct-root Text draw operation with no shaping left to a frontend.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationTextOp {
    runs: Vec<PresentationGlyphRun>,
    face: FontFace,
    size: PositiveFinite,
    paint: Paint,
    z: i32,
}

impl PresentationTextOp {
    pub(crate) fn new(
        runs: Vec<PresentationGlyphRun>,
        size: PositiveFinite,
        paint: Paint,
        z: i32,
    ) -> Result<Self, RenderError> {
        if runs.is_empty() {
            return Err(RenderError::InvalidRequest(
                "presentation Text requires at least one laid-out run".to_owned(),
            ));
        }
        Ok(Self {
            runs,
            face: FontFace::telex_regular(),
            size,
            paint,
            z,
        })
    }

    /// Return exact glyph runs in source paint order.
    #[must_use]
    pub fn runs(&self) -> &[PresentationGlyphRun] {
        &self.runs
    }

    /// Return the closed verified Telex face.
    #[must_use]
    pub fn face(&self) -> &FontFace {
        &self.face
    }

    /// Return the exact requested font size.
    #[must_use]
    pub const fn size(&self) -> PositiveFinite {
        self.size
    }

    /// Return the explicit foreground paint.
    #[must_use]
    pub fn paint(&self) -> &Paint {
        &self.paint
    }

    /// Return deterministic document-content z-order.
    #[must_use]
    pub const fn z(&self) -> i32 {
        self.z
    }
}

impl<'de> Deserialize<'de> for PresentationTextOp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            runs: Vec<PresentationGlyphRun>,
            face: FontFace,
            size: PositiveFinite,
            paint: Paint,
            z: i32,
        }
        let wire = Wire::deserialize(deserializer)?;
        if wire.face != FontFace::telex_regular() {
            return Err(serde::de::Error::custom(
                "presentation Text requires the verified Telex face",
            ));
        }
        Self::new(wire.runs, wire.size, wire.paint, wire.z).map_err(serde::de::Error::custom)
    }
}

/// Exact Text operation and logical bounds relative to the authored anchor.
#[derive(Clone, Debug, PartialEq)]
pub struct PresentationTextLayout {
    operation: PresentationTextOp,
    bounds: GlyphBounds,
}

impl PresentationTextLayout {
    pub(crate) const fn new(operation: PresentationTextOp, bounds: GlyphBounds) -> Self {
        Self { operation, bounds }
    }

    /// Return exact positioned glyph runs.
    #[must_use]
    pub const fn operation(&self) -> &PresentationTextOp {
        &self.operation
    }

    /// Return complete logical and ink bounds relative to the authored anchor.
    #[must_use]
    pub const fn bounds(&self) -> GlyphBounds {
        self.bounds
    }
}
