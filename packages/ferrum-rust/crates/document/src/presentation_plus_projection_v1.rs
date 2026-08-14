//! Resolved source facts for one direct-root CDML plus sign.

use serde::{Deserialize, Deserializer, Serialize};

use super::presentation_polyline_projection_v1::{RootStrokeDefaultsV1, point};
use super::presentation_shape_projection_v1::PresentationFillV1;
use super::presentation_stack_projection_v1::{
    PresentationFactProvenanceV1, PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1,
    PresentationTargetV1,
};
use super::{Point3V1, PositiveFiniteV1, Rgb24V1, TypedChild, TypedClass, TypedRecord};

const BUILTIN_PLUS_FONT_SIZE: f64 = 14.0;
const BUILTIN_PLUS_COLOR: &str = "#000000";

/// Complete resolved font facts for a fixed-content plus sign.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PresentationFontV1 {
    family: Option<String>,
    family_provenance: PresentationFactProvenanceV1,
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
        let family = wire.family.map(|value| value.trim().to_owned());
        if family.as_ref().is_some_and(String::is_empty)
            || (family.is_none()
                != (wire.family_provenance == PresentationFactProvenanceV1::Builtin))
        {
            return Err(serde::de::Error::custom(
                "presentation font family does not match its provenance",
            ));
        }
        let size = PositiveFiniteV1::new(wire.size)
            .ok_or_else(|| serde::de::Error::custom("invalid presentation font size"))?;
        if wire.size_provenance == PresentationFactProvenanceV1::Builtin
            && size.value() != BUILTIN_PLUS_FONT_SIZE
        {
            return Err(serde::de::Error::custom(
                "built-in plus font size must use the closed V1 value",
            ));
        }
        let color = Rgb24V1::new(wire.color)
            .ok_or_else(|| serde::de::Error::custom("invalid presentation font colour"))?;
        if wire.color_provenance == PresentationFactProvenanceV1::Builtin
            && color.as_str() != BUILTIN_PLUS_COLOR
        {
            return Err(serde::de::Error::custom(
                "built-in plus font colour must use the closed V1 value",
            ));
        }
        Ok(Self {
            family,
            family_provenance: wire.family_provenance,
            size,
            size_provenance: wire.size_provenance,
            color,
            color_provenance: wire.color_provenance,
        })
    }
}

impl PresentationFontV1 {
    /// Return an authored font family, or `None` for the verified Ferrum face.
    #[must_use]
    pub fn family(&self) -> Option<&str> {
        self.family.as_deref()
    }

    /// Return the precedence source for the family decision.
    #[must_use]
    pub fn family_provenance(&self) -> PresentationFactProvenanceV1 {
        self.family_provenance
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
        Ok(Self {
            target: wire.target,
            anchor: wire.anchor.into_point().map_err(serde::de::Error::custom)?,
            font: wire.font,
            background: wire.background,
        })
    }
}

impl PlusProjectionV1 {
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
    family: Option<String>,
    family_provenance: PresentationFactProvenanceV1,
    size: f64,
    size_provenance: PresentationFactProvenanceV1,
    color: String,
    color_provenance: PresentationFactProvenanceV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlusWireV1 {
    target: PresentationTargetV1,
    anchor: PointWireV1,
    font: PresentationFontV1,
    background: PresentationFillV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointWireV1 {
    x: f64,
    y: f64,
    z: f64,
}

impl PointWireV1 {
    fn into_point(self) -> Result<Point3V1, String> {
        Point3V1::new(self.x, self.y, self.z).map_err(|error| error.to_string())
    }
}

pub(crate) fn plus(
    child: &TypedChild,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<PlusProjectionV1> {
    let target = PresentationTargetV1::from_child(child);
    let record = child.record();
    let anchor = match record.children_of(TypedClass::Point).next().map(point) {
        Some(Ok(anchor)) => anchor,
        Some(Err(detail)) => {
            issues.push(PresentationProjectionIssueV1::new(
                target,
                PresentationProjectionIssueCodeV1::InvalidPlusGeometry,
                detail,
            ));
            return None;
        }
        None => {
            issues.push(PresentationProjectionIssueV1::new(
                target,
                PresentationProjectionIssueCodeV1::InvalidPlusGeometry,
                "plus requires one point child",
            ));
            return None;
        }
    };
    let font_record = record.children_of(TypedClass::Font).next();
    Some(PlusProjectionV1 {
        font: resolve_font(record, font_record, defaults.standard, &target, issues),
        background: resolve_background(record, &target, issues),
        target,
        anchor,
    })
}

fn resolve_font(
    root: &TypedRecord,
    font: Option<&TypedRecord>,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> PresentationFontV1 {
    let (family, family_provenance) = family(font, standard, target, issues);
    let (size, size_provenance) = size(root, font, standard, target, issues);
    let (color, color_provenance) = color(root, font, standard, target, issues);
    PresentationFontV1 {
        family,
        family_provenance,
        size,
        size_provenance,
        color,
        color_provenance,
    }
}

fn family(
    font: Option<&TypedRecord>,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> (Option<String>, PresentationFactProvenanceV1) {
    for (record, field, provenance) in [
        (font, "family", PresentationFactProvenanceV1::Root),
        (
            standard,
            "font_family",
            PresentationFactProvenanceV1::Standard,
        ),
    ] {
        let Some(value) = record.and_then(|record| record.attribute(field)) else {
            continue;
        };
        let value = value.trim();
        if !value.is_empty() {
            return (Some(value.to_owned()), provenance);
        }
        issues.push(PresentationProjectionIssueV1::new(
            target.clone(),
            PresentationProjectionIssueCodeV1::InvalidFontFact,
            format!("{field} must not be blank"),
        ));
    }
    (None, PresentationFactProvenanceV1::Builtin)
}

fn size(
    root: &TypedRecord,
    font: Option<&TypedRecord>,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> (PositiveFiniteV1, PresentationFactProvenanceV1) {
    for (record, field, provenance) in [
        (font, "size", PresentationFactProvenanceV1::Root),
        (Some(root), "font_size", PresentationFactProvenanceV1::Root),
        (
            standard,
            "font_size",
            PresentationFactProvenanceV1::Standard,
        ),
    ] {
        let Some(value) = record.and_then(|record| record.attribute(field)) else {
            continue;
        };
        if let Some(size) = value.parse().ok().and_then(PositiveFiniteV1::new) {
            return (size, provenance);
        }
        issues.push(PresentationProjectionIssueV1::new(
            target.clone(),
            PresentationProjectionIssueCodeV1::InvalidFontFact,
            format!("{field} must be finite and positive"),
        ));
    }
    (
        PositiveFiniteV1::new(BUILTIN_PLUS_FONT_SIZE).expect("closed built-in size is valid"),
        PresentationFactProvenanceV1::Builtin,
    )
}

fn color(
    root: &TypedRecord,
    font: Option<&TypedRecord>,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> (Rgb24V1, PresentationFactProvenanceV1) {
    for (record, field, provenance) in [
        (font, "color", PresentationFactProvenanceV1::Root),
        (Some(root), "color", PresentationFactProvenanceV1::Root),
        (
            standard,
            "line_color",
            PresentationFactProvenanceV1::Standard,
        ),
    ] {
        let Some(value) = record.and_then(|record| record.attribute(field)) else {
            continue;
        };
        if let Some(color) = Rgb24V1::new(value) {
            return (color, provenance);
        }
        issues.push(PresentationProjectionIssueV1::new(
            target.clone(),
            PresentationProjectionIssueCodeV1::InvalidFontFact,
            format!("{field} must be #rgb or #rrggbb"),
        ));
    }
    (
        Rgb24V1::new(BUILTIN_PLUS_COLOR).expect("closed built-in colour is valid"),
        PresentationFactProvenanceV1::Builtin,
    )
}

fn resolve_background(
    root: &TypedRecord,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> PresentationFillV1 {
    let Some(value) = root.attribute("background-color") else {
        return PresentationFillV1::resolved(None, PresentationFactProvenanceV1::Builtin);
    };
    if value.is_empty() || value == "none" {
        return PresentationFillV1::resolved(None, PresentationFactProvenanceV1::Root);
    }
    if let Some(color) = Rgb24V1::new(value) {
        return PresentationFillV1::resolved(Some(color), PresentationFactProvenanceV1::Root);
    }
    issues.push(PresentationProjectionIssueV1::new(
        target.clone(),
        PresentationProjectionIssueCodeV1::InvalidFillFact,
        "background-color must be empty, none, #rgb, or #rrggbb",
    ));
    PresentationFillV1::resolved(None, PresentationFactProvenanceV1::Builtin)
}
