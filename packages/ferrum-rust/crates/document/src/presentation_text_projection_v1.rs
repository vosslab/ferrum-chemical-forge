//! Typed-CDML adapter for immutable Text projection values.

use ferrum_document_projection::{
    PresentationFactProvenanceV1, PresentationFillV1, PresentationFontFaceV1,
    PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1, PresentationTargetV1,
    PresentationTextFontV1, PresentationTextRunV1, PresentationTextStyleV1, TextProjectionV1,
};
use xmlparser::{ElementEnd, Reference, Stream, Token, Tokenizer};

use super::presentation_polyline_projection_v1::{RootStrokeDefaultsV1, point};
use super::presentation_stack_projection_v1::presentation_target_from_child_v1;
use super::{PositiveFiniteV1, Rgb24V1, TypedChild, TypedClass, TypedRecord};

const BUILTIN_TEXT_FONT_SIZE: f64 = 12.0;
const BUILTIN_TEXT_COLOR: &str = "#000000";

pub(crate) fn text(
    child: &TypedChild,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Result<Option<TextProjectionV1>, crate::ProjectionError> {
    let target = presentation_target_from_child_v1(child)?;
    Ok(text_with_target(child, target, defaults, issues))
}

fn text_with_target(
    child: &TypedChild,
    target: PresentationTargetV1,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<TextProjectionV1> {
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
    let font = resolve_font(fonts.first().copied(), defaults.standard, &target, issues);
    let background = resolve_background(record, &target, issues);
    TextProjectionV1::try_new(target, anchor, runs, font, background).ok()
}

fn resolve_font(
    font: Option<&TypedRecord>,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> PresentationTextFontV1 {
    let (font_face, font_face_provenance) = font_face(font, standard, target, issues);
    let (size, size_provenance) = size(font, standard, target, issues);
    let (color, color_provenance) = color(font, standard, target, issues);
    PresentationTextFontV1::try_new(
        font_face,
        font_face_provenance,
        size,
        size_provenance,
        color,
        color_provenance,
    )
    .expect("typed-CDML font resolution always selects valid closed facts")
}

fn font_face(
    font: Option<&TypedRecord>,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> (PresentationFontFaceV1, PresentationFactProvenanceV1) {
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
        if let Some(face) = PresentationFontFaceV1::from_cdml_family(value) {
            return (face, provenance);
        }
        issues.push(PresentationProjectionIssueV1::new(
            target.clone(),
            PresentationProjectionIssueCodeV1::UnsupportedTextFace,
            format!("unsupported_text_face: {field} must be Telex Regular (bundled)"),
        ));
    }
    (
        PresentationFontFaceV1::TelexRegularV1,
        PresentationFactProvenanceV1::Builtin,
    )
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
        return PresentationFillV1::try_new(None, PresentationFactProvenanceV1::Builtin)
            .expect("closed built-in transparent fill is valid");
    };
    if value.is_empty() || value == "none" {
        return PresentationFillV1::try_new(None, PresentationFactProvenanceV1::Root)
            .expect("transparent root fill is valid");
    }
    if let Some(color) = Rgb24V1::new(value) {
        return PresentationFillV1::try_new(Some(color), PresentationFactProvenanceV1::Root)
            .expect("validated root fill colour is valid");
    }
    issues.push(PresentationProjectionIssueV1::new(
        target.clone(),
        PresentationProjectionIssueCodeV1::InvalidFillFact,
        "background-color must be empty, none, #rgb, or #rrggbb",
    ));
    PresentationFillV1::try_new(None, PresentationFactProvenanceV1::Builtin)
        .expect("closed built-in transparent fill is valid")
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
    if let Some(previous) = runs
        .last_mut()
        .filter(|run| run.styles() == styles.as_slice())
    {
        let merged = format!("{}{}", previous.text(), text);
        *previous =
            PresentationTextRunV1::try_new(merged, styles).map_err(|error| error.to_string())?;
        return Ok(());
    }
    runs.push(PresentationTextRunV1::try_new(text, styles).map_err(|error| error.to_string())?);
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
