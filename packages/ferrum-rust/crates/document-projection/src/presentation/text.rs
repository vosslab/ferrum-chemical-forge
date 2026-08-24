//! Immutable text projection values and validated wire conversion.

use serde::{Deserialize, Deserializer, Serialize};

use super::{
    PresentationFactProvenanceV1, PresentationFillV1, PresentationFontFaceV1,
    PresentationStackProjectionV1Error, PresentationTargetV1,
};
use crate::{Point3V1, PositiveFiniteV1, Rgb24V1};
const BUILTIN_TEXT_FONT_SIZE: f64 = 12.0;
const BUILTIN_TEXT_COLOR: &str = "#000000";

/// One supported formatting fact carried by a CDML formatted-text run.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationTextStyleV1 {
    /// Bold authored text. Rendering requires a verified bold face.
    Bold,
    /// Italic authored text. Rendering requires a verified italic face.
    Italic,
    /// Lowered script rendered with the regular face.
    Subscript,
    /// Raised script rendered with the regular face.
    Superscript,
}

/// One nonempty normalized character-data run and its closed style set.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PresentationTextRunV1 {
    text: String,
    styles: Vec<PresentationTextStyleV1>,
}

/// Complete resolved font facts for one direct-root Text label.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PresentationTextFontV1 {
    font_face: PresentationFontFaceV1,
    font_face_provenance: PresentationFactProvenanceV1,
    size: PositiveFiniteV1,
    size_provenance: PresentationFactProvenanceV1,
    color: Rgb24V1,
    color_provenance: PresentationFactProvenanceV1,
}

impl<'de> Deserialize<'de> for PresentationTextFontV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = TextFontWireV1::deserialize(deserializer)?;
        let size = PositiveFiniteV1::new(wire.size)
            .ok_or_else(|| serde::de::Error::custom("invalid presentation Text font size"))?;
        let color = Rgb24V1::new(wire.color)
            .ok_or_else(|| serde::de::Error::custom("invalid presentation Text font colour"))?;
        Self::try_new(
            wire.font_face,
            wire.font_face_provenance,
            size,
            wire.size_provenance,
            color,
            wire.color_provenance,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl PresentationTextFontV1 {
    /// Construct resolved text font facts with matching provenance.
    pub fn try_new(
        font_face: PresentationFontFaceV1,
        font_face_provenance: PresentationFactProvenanceV1,
        size: PositiveFiniteV1,
        size_provenance: PresentationFactProvenanceV1,
        color: Rgb24V1,
        color_provenance: PresentationFactProvenanceV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        if (size_provenance == PresentationFactProvenanceV1::Builtin
            && size.value() != BUILTIN_TEXT_FONT_SIZE)
            || (color_provenance == PresentationFactProvenanceV1::Builtin
                && color.as_str() != BUILTIN_TEXT_COLOR)
        {
            return Err(PresentationStackProjectionV1Error::InvalidFont);
        }
        Ok(Self {
            font_face,
            font_face_provenance,
            size,
            size_provenance,
            color,
            color_provenance,
        })
    }
    /// Return the closed semantic identity of the bundled presentation face.
    #[must_use]
    pub const fn font_face(&self) -> PresentationFontFaceV1 {
        self.font_face
    }

    /// Return the precedence source for the semantic face decision.
    #[must_use]
    pub const fn font_face_provenance(&self) -> PresentationFactProvenanceV1 {
        self.font_face_provenance
    }

    /// Return the positive finite display size.
    #[must_use]
    pub const fn size(&self) -> PositiveFiniteV1 {
        self.size
    }

    /// Return the precedence source for the display size.
    #[must_use]
    pub const fn size_provenance(&self) -> PresentationFactProvenanceV1 {
        self.size_provenance
    }

    /// Return the explicit foreground colour.
    #[must_use]
    pub fn color(&self) -> &Rgb24V1 {
        &self.color
    }

    /// Return the precedence source for the foreground colour.
    #[must_use]
    pub const fn color_provenance(&self) -> PresentationFactProvenanceV1 {
        self.color_provenance
    }
}

impl<'de> Deserialize<'de> for PresentationTextRunV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = TextRunWireV1::deserialize(deserializer)?;
        Self::try_new(wire.text, wire.styles).map_err(serde::de::Error::custom)
    }
}

impl PresentationTextRunV1 {
    pub fn try_new(
        text: String,
        styles: Vec<PresentationTextStyleV1>,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        if text.is_empty() {
            return Err(PresentationStackProjectionV1Error::InvalidTextRuns);
        }
        if styles.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(PresentationStackProjectionV1Error::InvalidTextRuns);
        }
        if styles.contains(&PresentationTextStyleV1::Subscript)
            && styles.contains(&PresentationTextStyleV1::Superscript)
        {
            return Err(PresentationStackProjectionV1Error::InvalidTextRuns);
        }
        Ok(Self { text, styles })
    }

    /// Return rendered character data, not XML source.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the canonical unique style set in bold, italic, sub, sup order.
    #[must_use]
    pub fn styles(&self) -> &[PresentationTextStyleV1] {
        &self.styles
    }
}

/// One direct-root Text label before verified font layout.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TextProjectionV1 {
    target: PresentationTargetV1,
    anchor: Point3V1,
    runs: Vec<PresentationTextRunV1>,
    font: PresentationTextFontV1,
    background: PresentationFillV1,
}

impl<'de> Deserialize<'de> for TextProjectionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = TextProjectionWireV1::deserialize(deserializer)?;
        Self::try_new(
            wire.target,
            wire.anchor.into_point().map_err(serde::de::Error::custom)?,
            wire.runs,
            wire.font,
            wire.background,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl TextProjectionV1 {
    /// Construct a text projection for a Text target with normalized visible runs.
    pub fn try_new(
        target: PresentationTargetV1,
        anchor: Point3V1,
        runs: Vec<PresentationTextRunV1>,
        font: PresentationTextFontV1,
        background: PresentationFillV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        if target.record_kind() != super::PresentationRecordKindV1::Text {
            return Err(PresentationStackProjectionV1Error::RootKindMismatch);
        }
        validate_runs(&runs)?;
        Ok(Self {
            target,
            anchor,
            runs,
            font,
            background,
        })
    }
    /// Return durable-or-local identity and root source order.
    #[must_use]
    pub fn target(&self) -> &PresentationTargetV1 {
        &self.target
    }

    /// Return the authored scene anchor for the first text line.
    #[must_use]
    pub const fn anchor(&self) -> Point3V1 {
        self.anchor
    }

    /// Return normalized source runs in rendered character order.
    #[must_use]
    pub fn runs(&self) -> &[PresentationTextRunV1] {
        &self.runs
    }

    /// Return fully resolved source font facts.
    #[must_use]
    pub fn font(&self) -> &PresentationTextFontV1 {
        &self.font
    }

    /// Return the explicit optional background fact.
    #[must_use]
    pub fn background(&self) -> &PresentationFillV1 {
        &self.background
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextRunWireV1 {
    pub text: String,
    pub styles: Vec<PresentationTextStyleV1>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextFontWireV1 {
    pub font_face: PresentationFontFaceV1,
    pub font_face_provenance: PresentationFactProvenanceV1,
    pub size: f64,
    pub size_provenance: PresentationFactProvenanceV1,
    pub color: String,
    pub color_provenance: PresentationFactProvenanceV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextProjectionWireV1 {
    pub target: PresentationTargetV1,
    pub anchor: PointWireV1,
    pub runs: Vec<PresentationTextRunV1>,
    pub font: PresentationTextFontV1,
    pub background: PresentationFillV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointWireV1 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PointWireV1 {
    fn into_point(self) -> Result<Point3V1, String> {
        Point3V1::new(self.x, self.y, self.z).map_err(|error| error.to_string())
    }
}

fn validate_runs(runs: &[PresentationTextRunV1]) -> Result<(), PresentationStackProjectionV1Error> {
    if runs.is_empty() {
        return Err(PresentationStackProjectionV1Error::InvalidTextRuns);
    }
    if runs
        .windows(2)
        .any(|pair| pair[0].styles() == pair[1].styles())
    {
        return Err(PresentationStackProjectionV1Error::InvalidTextRuns);
    }
    if !runs
        .iter()
        .flat_map(|run| run.text().chars())
        .any(|character| !character.is_whitespace())
    {
        return Err(PresentationStackProjectionV1Error::InvalidTextRuns);
    }
    Ok(())
}
