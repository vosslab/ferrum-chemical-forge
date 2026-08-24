//! Typed-CDML adapter for immutable plus projection values.

use ferrum_document_projection::{
    PlusProjectionV1, PresentationFactProvenanceV1, PresentationFillV1, PresentationFontFaceV1,
    PresentationFontV1, PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1,
    PresentationTargetV1,
};

use super::presentation_polyline_projection_v1::{RootStrokeDefaultsV1, point};
use super::presentation_stack_projection_v1::presentation_target_from_child_v1;
use super::{PositiveFiniteV1, Rgb24V1, TypedChild, TypedClass, TypedRecord};

const BUILTIN_PLUS_FONT_SIZE: f64 = 14.0;
const BUILTIN_PLUS_COLOR: &str = "#000000";

pub(crate) fn plus(
    child: &TypedChild,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Result<Option<PlusProjectionV1>, crate::ProjectionError> {
    let target = presentation_target_from_child_v1(child)?;
    Ok(plus_with_target(child, target, defaults, issues))
}

fn plus_with_target(
    child: &TypedChild,
    target: PresentationTargetV1,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<PlusProjectionV1> {
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
    let font = resolve_font(record, font_record, defaults.standard, &target, issues);
    let background = resolve_background(record, &target, issues);
    PlusProjectionV1::try_new(target, anchor, font, background).ok()
}

fn resolve_font(
    root: &TypedRecord,
    font: Option<&TypedRecord>,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> PresentationFontV1 {
    let (font_face, font_face_provenance) = font_face(font, standard, target, issues);
    let (size, size_provenance) = size(root, font, standard, target, issues);
    let (color, color_provenance) = color(root, font, standard, target, issues);
    PresentationFontV1::try_new(
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
