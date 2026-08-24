//! Shared point, polyline, and stroke projection for direct-root presentations.

use super::presentation_stack_projection_v1::presentation_target_from_child_v1;
use super::{
    Point3V1, PositiveFiniteV1, Rgb24V1, TypedChild, TypedClass, TypedDocument, TypedRecord,
};
use ferrum_document_projection::{
    PolylinePathV1, PolylineProjectionV1, PresentationFactProvenanceV1,
    PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1, PresentationStrokeV1,
    PresentationTargetV1,
};

const POINTS_PER_CENTIMETRE: f64 = 72.0 / 2.54;
const BUILTIN_LINE_COLOR: &str = "#000000";
const BUILTIN_LINE_WIDTH: f64 = 1.0;

#[derive(Clone, Copy)]
pub(crate) struct RootStrokeDefaultsV1<'a> {
    pub(crate) standard: Option<&'a TypedRecord>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PolylineProjectionKindV1 {
    Ordinary,
    Wavy,
    RoundBracket,
}

impl<'a> RootStrokeDefaultsV1<'a> {
    pub(crate) fn from_document(document: &'a TypedDocument) -> Self {
        Self {
            standard: document
                .root()
                .typed_children()
                .iter()
                .find(|child| child.record().class() == TypedClass::Standard)
                .map(TypedChild::record),
        }
    }
}

pub(crate) fn polyline(
    child: &TypedChild,
    defaults: RootStrokeDefaultsV1<'_>,
    round_bracket_member: bool,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Result<Option<(PolylineProjectionKindV1, PolylineProjectionV1)>, crate::ProjectionError> {
    let target = presentation_target_from_child_v1(child)?;
    Ok(polyline_with_target(
        child,
        target,
        defaults,
        round_bracket_member,
        issues,
    ))
}

fn polyline_with_target(
    child: &TypedChild,
    target: PresentationTargetV1,
    defaults: RootStrokeDefaultsV1<'_>,
    round_bracket_member: bool,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<(PolylineProjectionKindV1, PolylineProjectionV1)> {
    let record = child.record();
    let is_wavy = record.attribute("style") == Some("wavy");
    let is_spline = spline(record);
    if is_spline.is_none() || (is_spline == Some(true) && !round_bracket_member) {
        issues.push(PresentationProjectionIssueV1::new(
            target,
            PresentationProjectionIssueCodeV1::UnsupportedSpline,
            "polyline spline must be absent, no, false, or 0",
        ));
        return None;
    }
    let points = match points(record, 2, "polyline") {
        Ok(points) => points,
        Err(detail) => {
            issues.push(PresentationProjectionIssueV1::new(
                target,
                PresentationProjectionIssueCodeV1::InvalidPolylineGeometry,
                detail,
            ));
            return None;
        }
    };
    let stroke = stroke(record, defaults, &target, issues);
    let kind = if round_bracket_member {
        PolylineProjectionKindV1::RoundBracket
    } else if is_wavy {
        PolylineProjectionKindV1::Wavy
    } else {
        PolylineProjectionKindV1::Ordinary
    };
    Some((
        kind,
        match PolylinePathV1::try_new(points)
            .and_then(|path| PolylineProjectionV1::new(target.clone(), path, stroke.clone()))
        {
            Ok(polyline) => polyline,
            Err(error) => {
                issues.push(PresentationProjectionIssueV1::new(
                    target,
                    PresentationProjectionIssueCodeV1::InvalidPolylineGeometry,
                    error.to_string(),
                ));
                return None;
            }
        },
    ))
}

fn spline(record: &TypedRecord) -> Option<bool> {
    match record.attribute("spline") {
        None | Some("no" | "false" | "0") => Some(false),
        Some("yes" | "true" | "1") => Some(true),
        Some(_) => None,
    }
}

pub(crate) fn points(
    record: &TypedRecord,
    minimum: usize,
    kind: &'static str,
) -> Result<Vec<Point3V1>, String> {
    let records = record.children_of(TypedClass::Point).collect::<Vec<_>>();
    if records.len() < minimum {
        return Err(format!("{kind} requires at least {minimum} point children"));
    }
    records.into_iter().map(point).collect()
}

pub(crate) fn point(record: &TypedRecord) -> Result<Point3V1, String> {
    Point3V1::new(
        coordinate(record, "x")?,
        coordinate(record, "y")?,
        record
            .attribute("z")
            .map(|_| coordinate(record, "z"))
            .transpose()?
            .unwrap_or(0.0),
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn coordinate(record: &TypedRecord, field: &'static str) -> Result<f64, String> {
    let value = record
        .attribute(field)
        .ok_or_else(|| format!("{field} is absent"))?;
    let (raw, scale) = value.strip_suffix("cm").map_or_else(
        || (value.strip_suffix("px").unwrap_or(value), 1.0),
        |raw| (raw, POINTS_PER_CENTIMETRE),
    );
    let value = raw
        .parse::<f64>()
        .map_err(|_| format!("{field} value {value:?} is invalid"))?
        * scale;
    value
        .is_finite()
        .then_some(value)
        .ok_or_else(|| format!("{field} is not finite"))
}

pub(crate) fn stroke(
    record: &TypedRecord,
    defaults: RootStrokeDefaultsV1<'_>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> PresentationStrokeV1 {
    stroke_with_color_field(record, defaults, target, issues, "line_color")
}

pub(crate) fn stroke_with_color_field(
    record: &TypedRecord,
    defaults: RootStrokeDefaultsV1<'_>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
    root_color_field: &'static str,
) -> PresentationStrokeV1 {
    let (color, color_provenance) =
        color(record, defaults.standard, target, issues, root_color_field);
    let (width, width_provenance) = width(record, defaults.standard, target, issues);
    PresentationStrokeV1::new(color, color_provenance, width, width_provenance)
        .expect("typed-CDML stroke resolution always selects valid closed facts")
}

fn color(
    root: &TypedRecord,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
    root_color_field: &'static str,
) -> (Rgb24V1, PresentationFactProvenanceV1) {
    for (record, field, provenance) in [
        (
            Some(root),
            root_color_field,
            PresentationFactProvenanceV1::Root,
        ),
        (
            (root_color_field == "line_color").then_some(root),
            "color",
            PresentationFactProvenanceV1::Root,
        ),
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
            PresentationProjectionIssueCodeV1::InvalidStrokeFact,
            format!("{field} must be #rgb or #rrggbb"),
        ));
    }
    (
        Rgb24V1::new(BUILTIN_LINE_COLOR).expect("closed built-in colour is valid"),
        PresentationFactProvenanceV1::Builtin,
    )
}

fn width(
    root: &TypedRecord,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> (PositiveFiniteV1, PresentationFactProvenanceV1) {
    for (record, field, provenance) in [
        (Some(root), "width", PresentationFactProvenanceV1::Root),
        (
            standard,
            "line_width",
            PresentationFactProvenanceV1::Standard,
        ),
    ] {
        let Some(value) = record.and_then(|record| record.attribute(field)) else {
            continue;
        };
        if let Some(width) = parse_width(value) {
            return (width, provenance);
        }
        issues.push(PresentationProjectionIssueV1::new(
            target.clone(),
            PresentationProjectionIssueCodeV1::InvalidStrokeFact,
            format!("{field} must be a positive finite bare or px length"),
        ));
    }
    (
        PositiveFiniteV1::new(BUILTIN_LINE_WIDTH).expect("closed built-in width is valid"),
        PresentationFactProvenanceV1::Builtin,
    )
}

pub(crate) fn parse_width(value: &str) -> Option<PositiveFiniteV1> {
    let value = value.strip_suffix("px").unwrap_or(value);
    PositiveFiniteV1::new(value.parse().ok()?)
}
