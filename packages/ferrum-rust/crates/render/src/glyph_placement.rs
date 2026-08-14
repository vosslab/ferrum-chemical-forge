//! Closed Unicode-scalar Telex glyph placement facts.

use serde::{Deserialize, Serialize};

use crate::{RenderError, RenderPoint};

/// Position of a text run in a structured atom label.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextScript {
    /// The primary element label.
    Baseline,
    /// A lowered script such as explicit hydrogen count.
    Subscript,
    /// A raised script such as a formal charge.
    Superscript,
}

/// One exact verified-Telex outline placement relative to its containing run.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GlyphPlacement {
    glyph_index: u32,
    origin: RenderPoint,
}

impl GlyphPlacement {
    /// Construct one non-missing face-local glyph at a finite run-local origin.
    pub(crate) fn new(glyph_index: u32, origin: RenderPoint) -> Result<Self, RenderError> {
        if glyph_index == 0 {
            return Err(RenderError::InvalidRequest(
                "text glyph index must not be the missing-glyph identifier".to_owned(),
            ));
        }
        Ok(Self {
            glyph_index,
            origin,
        })
    }
    /// Return the exact Telex glyph identifier used by `QRawFont.pathForGlyph`.
    #[must_use]
    pub const fn glyph_index(self) -> u32 {
        self.glyph_index
    }
    /// Return the exact finite origin relative to the containing run.
    #[must_use]
    pub const fn origin(self) -> RenderPoint {
        self.origin
    }
}

impl<'de> Deserialize<'de> for GlyphPlacement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireGlyphPlacement {
            glyph_index: u32,
            origin: RenderPoint,
        }
        let wire = WireGlyphPlacement::deserialize(deserializer)?;
        Self::new(wire.glyph_index, wire.origin).map_err(serde::de::Error::custom)
    }
}
