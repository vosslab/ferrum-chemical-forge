//! Resolved source facts for one direct-root CDML Text label.

use serde::{Deserialize, Deserializer, Serialize};
use xmlparser::{ElementEnd, Reference, Stream, Token, Tokenizer};

use super::presentation_polyline_projection_v1::{RootStrokeDefaultsV1, point};
use super::presentation_shape_projection_v1::PresentationFillV1;
use super::presentation_stack_projection_v1::{
    PresentationFactProvenanceV1, PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1,
    PresentationTargetV1,
};
use super::{Point3V1, PositiveFiniteV1, Rgb24V1, TypedChild, TypedClass, TypedRecord};

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
    family: Option<String>,
    family_provenance: PresentationFactProvenanceV1,
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
        let family = wire.family.map(|value| value.trim().to_owned());
        if family.as_ref().is_some_and(String::is_empty)
            || (family.is_none()
                != (wire.family_provenance == PresentationFactProvenanceV1::Builtin))
        {
            return Err(serde::de::Error::custom(
                "presentation Text font family does not match its provenance",
            ));
        }
        let size = PositiveFiniteV1::new(wire.size)
            .ok_or_else(|| serde::de::Error::custom("invalid presentation Text font size"))?;
        if wire.size_provenance == PresentationFactProvenanceV1::Builtin
            && size.value() != BUILTIN_TEXT_FONT_SIZE
        {
            return Err(serde::de::Error::custom(
                "built-in Text font size must use the closed V1 value",
            ));
        }
        let color = Rgb24V1::new(wire.color)
            .ok_or_else(|| serde::de::Error::custom("invalid presentation Text font colour"))?;
        if wire.color_provenance == PresentationFactProvenanceV1::Builtin
            && color.as_str() != BUILTIN_TEXT_COLOR
        {
            return Err(serde::de::Error::custom(
                "built-in Text font colour must use the closed V1 value",
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

impl PresentationTextFontV1 {
    /// Return an authored or standard font family, or `None` for Ferrum Telex.
    #[must_use]
    pub fn family(&self) -> Option<&str> {
        self.family.as_deref()
    }

    /// Return the precedence source for the family decision.
    #[must_use]
    pub const fn family_provenance(&self) -> PresentationFactProvenanceV1 {
        self.family_provenance
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
        Self::new(wire.text, wire.styles).map_err(serde::de::Error::custom)
    }
}

impl PresentationTextRunV1 {
    fn new(text: String, styles: Vec<PresentationTextStyleV1>) -> Result<Self, String> {
        if text.is_empty() {
            return Err("presentation text runs must not be empty".to_owned());
        }
        if styles.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err("presentation text styles must use canonical unique order".to_owned());
        }
        if styles.contains(&PresentationTextStyleV1::Subscript)
            && styles.contains(&PresentationTextStyleV1::Superscript)
        {
            return Err("presentation text cannot combine subscript and superscript".to_owned());
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
        validate_runs(&wire.runs).map_err(serde::de::Error::custom)?;
        Ok(Self {
            target: wire.target,
            anchor: wire.anchor.into_point().map_err(serde::de::Error::custom)?,
            runs: wire.runs,
            font: wire.font,
            background: wire.background,
        })
    }
}

impl TextProjectionV1 {
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
    text: String,
    styles: Vec<PresentationTextStyleV1>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextFontWireV1 {
    family: Option<String>,
    family_provenance: PresentationFactProvenanceV1,
    size: f64,
    size_provenance: PresentationFactProvenanceV1,
    color: String,
    color_provenance: PresentationFactProvenanceV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextProjectionWireV1 {
    target: PresentationTargetV1,
    anchor: PointWireV1,
    runs: Vec<PresentationTextRunV1>,
    font: PresentationTextFontV1,
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

pub(crate) fn text(
    child: &TypedChild,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<TextProjectionV1> {
    let target = PresentationTargetV1::from_child(child);
    let record = child.record();
    let points = record.children_of(TypedClass::Point).collect::<Vec<_>>();
    if points.len() != 1 {
        issues.push(PresentationProjectionIssueV1::new(
            target,
            PresentationProjectionIssueCodeV1::InvalidTextGeometry,
            "Text requires exactly one point child",
        ));
        return None;
    }
    let anchor = match point(points[0]) {
        Ok(value) => value,
        Err(detail) => {
            issues.push(PresentationProjectionIssueV1::new(
                target,
                PresentationProjectionIssueCodeV1::InvalidTextGeometry,
                detail,
            ));
            return None;
        }
    };
    let ftexts = record
        .children_of(TypedClass::FormattedText)
        .collect::<Vec<_>>();
    if ftexts.len() != 1 {
        issues.push(PresentationProjectionIssueV1::new(
            target,
            PresentationProjectionIssueCodeV1::InvalidTextContent,
            "Text requires exactly one formatted-text child",
        ));
        return None;
    }
    let runs = match decode_ftext(&ftexts[0].text_content()) {
        Ok(value) => value,
        Err(detail) => {
            issues.push(PresentationProjectionIssueV1::new(
                target,
                PresentationProjectionIssueCodeV1::InvalidTextContent,
                detail,
            ));
            return None;
        }
    };
    if !runs
        .iter()
        .flat_map(|run| run.text().chars())
        .any(|character| !character.is_whitespace())
    {
        issues.push(PresentationProjectionIssueV1::new(
            target,
            PresentationProjectionIssueCodeV1::InvalidTextContent,
            "Text content must contain a visible character",
        ));
        return None;
    }
    let fonts = record.children_of(TypedClass::Font).collect::<Vec<_>>();
    if fonts.len() > 1 {
        issues.push(PresentationProjectionIssueV1::new(
            target,
            PresentationProjectionIssueCodeV1::InvalidFontFact,
            "Text permits at most one font child",
        ));
        return None;
    }
    Some(TextProjectionV1 {
        font: resolve_font(fonts.first().copied(), defaults.standard, &target, issues),
        background: resolve_background(record, &target, issues),
        target,
        anchor,
        runs,
    })
}

fn resolve_font(
    font: Option<&TypedRecord>,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> PresentationTextFontV1 {
    let (family, family_provenance) = family(font, standard, target, issues);
    let (size, size_provenance) = size(font, standard, target, issues);
    let (color, color_provenance) = color(font, standard, target, issues);
    PresentationTextFontV1 {
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
    font: Option<&TypedRecord>,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> (PositiveFiniteV1, PresentationFactProvenanceV1) {
    for (record, field, provenance) in [
        (font, "size", PresentationFactProvenanceV1::Root),
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
        PositiveFiniteV1::new(BUILTIN_TEXT_FONT_SIZE).expect("closed built-in Text size is valid"),
        PresentationFactProvenanceV1::Builtin,
    )
}

fn color(
    font: Option<&TypedRecord>,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> (Rgb24V1, PresentationFactProvenanceV1) {
    for (record, field, provenance) in [
        (font, "color", PresentationFactProvenanceV1::Root),
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
        Rgb24V1::new(BUILTIN_TEXT_COLOR).expect("closed built-in Text colour is valid"),
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

fn validate_runs(runs: &[PresentationTextRunV1]) -> Result<(), String> {
    if runs.is_empty() {
        return Err("presentation Text requires at least one nonempty run".to_owned());
    }
    if runs
        .windows(2)
        .any(|pair| pair[0].styles() == pair[1].styles())
    {
        return Err("adjacent presentation text runs must be normalized".to_owned());
    }
    if !runs
        .iter()
        .flat_map(|run| run.text().chars())
        .any(|character| !character.is_whitespace())
    {
        return Err("presentation Text must contain a visible character".to_owned());
    }
    Ok(())
}

fn decode_ftext(authored: &str) -> Result<Vec<PresentationTextRunV1>, String> {
    let upper = authored.to_ascii_uppercase();
    if upper.contains("<!DOCTYPE") || upper.contains("<!ENTITY") {
        return Err("formatted Text cannot declare DTDs or entities".to_owned());
    }
    let source = format!("<ftext-root>{authored}</ftext-root>");
    let mut elements = Vec::new();
    let mut pending = None;
    let mut runs = Vec::new();
    for token in Tokenizer::from(source.as_str()) {
        let token = token.map_err(|_| "formatted Text markup is malformed".to_owned())?;
        match token {
            Token::ElementStart { prefix, local, .. } => {
                if !prefix.as_str().is_empty() {
                    return Err("formatted Text tags cannot use namespaces".to_owned());
                }
                let tag = TextTag::parse(local.as_str(), elements.is_empty())?;
                if tag != TextTag::Root {
                    let style = tag.style().expect("non-root tag has a style");
                    let current = styles(&elements);
                    if current.contains(&style)
                        || (style == PresentationTextStyleV1::Subscript
                            && current.contains(&PresentationTextStyleV1::Superscript))
                        || (style == PresentationTextStyleV1::Superscript
                            && current.contains(&PresentationTextStyleV1::Subscript))
                    {
                        return Err(
                            "formatted Text styles cannot repeat or combine sub and sup".to_owned()
                        );
                    }
                }
                pending = Some(tag);
            }
            Token::Attribute { .. } => {
                return Err("formatted Text tags cannot have attributes or namespaces".to_owned());
            }
            Token::ElementEnd { end, .. } => match end {
                ElementEnd::Open => elements.push(
                    pending
                        .take()
                        .ok_or_else(|| "formatted Text markup is malformed".to_owned())?,
                ),
                ElementEnd::Empty => {
                    pending
                        .take()
                        .ok_or_else(|| "formatted Text markup is malformed".to_owned())?;
                }
                ElementEnd::Close(prefix, local) => {
                    if !prefix.as_str().is_empty() {
                        return Err("formatted Text tags cannot use namespaces".to_owned());
                    }
                    let expected = elements
                        .pop()
                        .ok_or_else(|| "formatted Text markup is malformed".to_owned())?;
                    if expected.name() != local.as_str() {
                        return Err("formatted Text markup is malformed".to_owned());
                    }
                }
            },
            Token::Text { text } => append_text(
                &mut runs,
                decode_entities(text.as_str())?,
                styles(&elements),
            )?,
            Token::Cdata { text, .. } => {
                append_text(&mut runs, text.as_str().to_owned(), styles(&elements))?;
            }
            Token::Declaration { .. }
            | Token::ProcessingInstruction { .. }
            | Token::Comment { .. }
            | Token::DtdStart { .. }
            | Token::EmptyDtd { .. }
            | Token::EntityDeclaration { .. }
            | Token::DtdEnd { .. } => {
                return Err(
                    "formatted Text cannot contain declarations, comments, or instructions"
                        .to_owned(),
                );
            }
        }
    }
    if !elements.is_empty() || pending.is_some() {
        return Err("formatted Text markup is malformed".to_owned());
    }
    validate_runs(&runs)?;
    Ok(runs)
}

fn append_text(
    runs: &mut Vec<PresentationTextRunV1>,
    text: String,
    styles: Vec<PresentationTextStyleV1>,
) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }
    if let Some(previous) = runs.last_mut().filter(|run| run.styles == styles) {
        previous.text.push_str(&text);
        return Ok(());
    }
    runs.push(PresentationTextRunV1::new(text, styles)?);
    Ok(())
}

fn decode_entities(text: &str) -> Result<String, String> {
    let mut stream = Stream::from(text);
    let mut start = 0;
    let mut result = String::with_capacity(text.len());
    while !stream.at_end() {
        if stream
            .curr_byte()
            .map_err(|_| "invalid formatted Text".to_owned())?
            == b'&'
        {
            result.push_str(&text[start..stream.pos()]);
            let reference = stream
                .consume_reference()
                .map_err(|_| "formatted Text contains an invalid entity reference".to_owned())?;
            match reference {
                Reference::Char(character) => result.push(character),
                Reference::Entity(_) => {
                    return Err("formatted Text cannot use custom entity references".to_owned());
                }
            }
            start = stream.pos();
        } else {
            let character = text[stream.pos()..]
                .chars()
                .next()
                .ok_or_else(|| "invalid formatted Text".to_owned())?;
            stream.advance(character.len_utf8());
        }
    }
    result.push_str(&text[start..]);
    Ok(result)
}

fn styles(elements: &[TextTag]) -> Vec<PresentationTextStyleV1> {
    [
        PresentationTextStyleV1::Bold,
        PresentationTextStyleV1::Italic,
        PresentationTextStyleV1::Subscript,
        PresentationTextStyleV1::Superscript,
    ]
    .into_iter()
    .filter(|style| {
        elements
            .iter()
            .any(|element| element.style() == Some(*style))
    })
    .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TextTag {
    Root,
    Bold,
    Italic,
    Subscript,
    Superscript,
}

impl TextTag {
    fn parse(value: &str, root: bool) -> Result<Self, String> {
        match (root, value) {
            (true, "ftext-root") => Ok(Self::Root),
            (false, "b") => Ok(Self::Bold),
            (false, "i") => Ok(Self::Italic),
            (false, "sub") => Ok(Self::Subscript),
            (false, "sup") => Ok(Self::Superscript),
            _ => Err("formatted Text contains an unsupported tag".to_owned()),
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Root => "ftext-root",
            Self::Bold => "b",
            Self::Italic => "i",
            Self::Subscript => "sub",
            Self::Superscript => "sup",
        }
    }

    const fn style(self) -> Option<PresentationTextStyleV1> {
        match self {
            Self::Root => None,
            Self::Bold => Some(PresentationTextStyleV1::Bold),
            Self::Italic => Some(PresentationTextStyleV1::Italic),
            Self::Subscript => Some(PresentationTextStyleV1::Subscript),
            Self::Superscript => Some(PresentationTextStyleV1::Superscript),
        }
    }
}
