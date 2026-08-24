//! Immutable plus projection values and validated wire conversion.

use serde::{Deserialize, Deserializer, Serialize};

use super::{
    PresentationFactProvenanceV1, PresentationFillV1, PresentationStackProjectionV1Error,
    PresentationTargetV1,
};
use crate::{Point3V1, PositiveFiniteV1, Rgb24V1};
const BUILTIN_PLUS_FONT_SIZE: f64 = 14.0;
const BUILTIN_PLUS_COLOR: &str = "#000000";

/// Complete resolved font facts for a fixed-content plus sign.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PresentationFontV1 {
    font_face: super::PresentationFontFaceV1,
    font_face_provenance: PresentationFactProvenanceV1,
    size: PositiveFiniteV1,
    size_provenance: PresentationFactProvenanceV1,
    color: Rgb24V1,
    color_provenance: PresentationFactProvenanceV1,
}

impl<'de> Deserialize<'de> for PresentationFontV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PresentationFontWireV1::deserialize(deserializer)?;
        let size = PositiveFiniteV1::new(wire.size)
            .ok_or_else(|| serde::de::Error::custom("invalid presentation font size"))?;
        let color = Rgb24V1::new(wire.color)
            .ok_or_else(|| serde::de::Error::custom("invalid presentation font colour"))?;
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

impl PresentationFontV1 {
    /// Construct resolved plus font facts with matching provenance.
    pub fn try_new(
        font_face: super::PresentationFontFaceV1,
        font_face_provenance: PresentationFactProvenanceV1,
        size: PositiveFiniteV1,
        size_provenance: PresentationFactProvenanceV1,
        color: Rgb24V1,
        color_provenance: PresentationFactProvenanceV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        if (size_provenance == PresentationFactProvenanceV1::Builtin
            && size.value() != BUILTIN_PLUS_FONT_SIZE)
            || (color_provenance == PresentationFactProvenanceV1::Builtin
                && color.as_str() != BUILTIN_PLUS_COLOR)
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
    pub const fn font_face(&self) -> super::PresentationFontFaceV1 {
        self.font_face
    }

    /// Return the precedence source for the semantic face decision.
    #[must_use]
    pub fn font_face_provenance(&self) -> PresentationFactProvenanceV1 {
        self.font_face_provenance
    }

    /// Return the positive finite display size.
    #[must_use]
    pub fn size(&self) -> PositiveFiniteV1 {
        self.size
    }

    /// Return the precedence source for the display size.
    #[must_use]
    pub fn size_provenance(&self) -> PresentationFactProvenanceV1 {
        self.size_provenance
    }

    /// Return the explicit foreground colour.
    #[must_use]
    pub fn color(&self) -> &Rgb24V1 {
        &self.color
    }

    /// Return the precedence source for the foreground colour.
    #[must_use]
    pub fn color_provenance(&self) -> PresentationFactProvenanceV1 {
        self.color_provenance
    }
}

/// One fixed-content plus sign before verified glyph layout.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlusProjectionV1 {
    target: PresentationTargetV1,
    anchor: Point3V1,
    font: PresentationFontV1,
    background: PresentationFillV1,
}

impl<'de> Deserialize<'de> for PlusProjectionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PlusWireV1::deserialize(deserializer)?;
        Self::try_new(
            wire.target,
            wire.anchor.into_point().map_err(serde::de::Error::custom)?,
            wire.font,
            wire.background,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl PlusProjectionV1 {
    /// Construct a plus projection for a Plus target.
    pub fn try_new(
        target: PresentationTargetV1,
        anchor: Point3V1,
        font: PresentationFontV1,
        background: PresentationFillV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        (target.record_kind() == super::PresentationRecordKindV1::Plus)
            .then_some(Self {
                target,
                anchor,
                font,
                background,
            })
            .ok_or(PresentationStackProjectionV1Error::RootKindMismatch)
    }
    /// Return durable-or-local identity and root source order.
    #[must_use]
    pub fn target(&self) -> &PresentationTargetV1 {
        &self.target
    }

    /// Return the authored scene anchor around which the glyph is centered.
    #[must_use]
    pub fn anchor(&self) -> Point3V1 {
        self.anchor
    }

    /// Return fully resolved source font facts.
    #[must_use]
    pub fn font(&self) -> &PresentationFontV1 {
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
struct PresentationFontWireV1 {
    pub font_face: super::PresentationFontFaceV1,
    pub font_face_provenance: PresentationFactProvenanceV1,
    pub size: f64,
    pub size_provenance: PresentationFactProvenanceV1,
    pub color: String,
    pub color_provenance: PresentationFactProvenanceV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlusWireV1 {
    pub target: PresentationTargetV1,
    pub anchor: PointWireV1,
    pub font: PresentationFontV1,
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
