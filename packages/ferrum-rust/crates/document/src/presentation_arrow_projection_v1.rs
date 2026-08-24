//! Typed-CDML adapter for immutable arrow projection values.

pub use ferrum_document_projection::{
    ArrowHeadShapeV1, ArrowPathV1, ArrowProjectionKindV1, ArrowProjectionV1,
    CurvedTerminalArrowKindV1,
};

use ferrum_document_projection::Point3V1;

use super::presentation_polyline_projection_v1::{
    RootStrokeDefaultsV1, points, stroke_with_color_field,
};
use super::presentation_stack_projection_v1::presentation_target_from_child_v1;
use super::{TypedChild, TypedRecord};
use ferrum_document_projection::{
    PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1, PresentationStrokeV1,
    PresentationTargetV1,
};

const DEFAULT_HEAD_LINE_INSET: f64 = 8.0;
const DEFAULT_HEAD_TOTAL_LENGTH: f64 = 10.0;
const DEFAULT_HEAD_HALF_WIDTH: f64 = 3.0;

pub(crate) fn arrow(
    child: &TypedChild,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Result<Option<ArrowProjectionV1>, crate::ProjectionError> {
    let target = presentation_target_from_child_v1(child)?;
    Ok(match child.record().attribute("type").unwrap_or("normal") {
        "normal" => normal_arrow(child.record(), target, defaults, issues),
        "equilibrium" => equilibrium_arrow(child.record(), target, defaults, issues),
        "curved-equilibrium" => curved_equilibrium_arrow(child.record(), target, defaults, issues),
        "electron" => curved_terminal_arrow(
            child.record(),
            target,
            defaults,
            issues,
            CurvedTerminalArrowKindV1::Electron,
        ),
        "retro" => curved_terminal_arrow(
            child.record(),
            target,
            defaults,
            issues,
            CurvedTerminalArrowKindV1::Retro,
        ),
        "curved-normal" => curved_terminal_arrow(
            child.record(),
            target,
            defaults,
            issues,
            CurvedTerminalArrowKindV1::Normal,
        ),
        other => unsupported_arrow(target, issues, other),
    })
}

fn normal_arrow(
    record: &TypedRecord,
    target: PresentationTargetV1,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<ArrowProjectionV1> {
    if !is_nonspline(record) {
        issues.push(PresentationProjectionIssueV1::new(
            target,
            PresentationProjectionIssueCodeV1::UnsupportedArrowSpline,
            "normal arrow spline must be absent, no, false, or 0",
        ));
        return None;
    }
    let source_points = match points(record, 2, "arrow") {
        Ok(points) => points,
        Err(detail) => return invalid_geometry(target, issues, detail.to_string()),
    };
    let (start_head, end_head) = match normal_heads(record) {
        Ok(heads) => heads,
        Err(detail) => return invalid_fact(target, issues, detail),
    };
    let shape = match head_shape(record) {
        Ok(shape) => shape,
        Err(detail) => return invalid_fact(target, issues, detail),
    };
    let stroke = stroke(record, defaults, &target, issues);
    match ArrowProjectionV1::normal(
        target.clone(),
        source_points,
        shape,
        start_head,
        end_head,
        stroke,
    ) {
        Ok(projection) => Some(projection),
        Err(detail) => invalid_geometry(target, issues, detail.to_string()),
    }
}

fn equilibrium_arrow(
    record: &TypedRecord,
    target: PresentationTargetV1,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<ArrowProjectionV1> {
    if !is_nonspline(record) {
        issues.push(PresentationProjectionIssueV1::new(
            target,
            PresentationProjectionIssueCodeV1::UnsupportedArrowSpline,
            "equilibrium arrow spline must be absent, no, false, or 0",
        ));
        return None;
    }
    if has_normal_head_facts(record) {
        return invalid_fact(
            target,
            issues,
            "equilibrium arrows have no normal-arrow head facts",
        );
    }
    let source_points = match exact_points(record, "equilibrium arrow", 2) {
        Ok(points) => points,
        Err(detail) => return invalid_geometry(target, issues, detail.to_string()),
    };
    let [_start, _end] = source_points.as_slice() else {
        return invalid_geometry(
            target,
            issues,
            "equilibrium arrow requires exactly two points",
        );
    };
    let stroke = stroke(record, defaults, &target, issues);
    let source_path = match ArrowPathV1::try_new(source_points) {
        Ok(path) => path,
        Err(detail) => return invalid_geometry(target, issues, detail.to_string()),
    };
    match ArrowProjectionV1::try_new(
        target.clone(),
        source_path,
        ArrowProjectionKindV1::Equilibrium,
        stroke,
    ) {
        Ok(projection) => Some(projection),
        Err(detail) => invalid_geometry(target, issues, detail.to_string()),
    }
}

fn curved_terminal_arrow(
    record: &TypedRecord,
    target: PresentationTargetV1,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
    kind: CurvedTerminalArrowKindV1,
) -> Option<ArrowProjectionV1> {
    if has_normal_head_facts(record) {
        return invalid_fact(
            target,
            issues,
            "curved terminal arrows have no normal-arrow head facts",
        );
    }
    if record.children_of(super::TypedClass::Point).count() != 3 {
        return invalid_geometry(
            target,
            issues,
            "curved terminal arrow requires exactly three points",
        );
    }
    let source_points = match exact_points(record, "curved terminal arrow", 3) {
        Ok(points) => points,
        Err(detail) => return invalid_geometry(target, issues, detail.to_string()),
    };
    let [_start, _control, _end] = source_points.as_slice() else {
        return invalid_geometry(
            target,
            issues,
            "curved terminal arrow requires exactly three points",
        );
    };
    let stroke = stroke(record, defaults, &target, issues);
    let source_path = match ArrowPathV1::try_new(source_points) {
        Ok(path) => path,
        Err(detail) => return invalid_geometry(target, issues, detail.to_string()),
    };
    match ArrowProjectionV1::try_new(
        target.clone(),
        source_path,
        ArrowProjectionKindV1::CurvedTerminal {
            terminal_kind: kind,
        },
        stroke,
    ) {
        Ok(projection) => Some(projection),
        Err(detail) => invalid_geometry(target, issues, detail.to_string()),
    }
}

fn curved_equilibrium_arrow(
    record: &TypedRecord,
    target: PresentationTargetV1,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<ArrowProjectionV1> {
    if has_nonterminal_facts(record) {
        return invalid_fact(
            target,
            issues,
            "curved-equilibrium arrows have no normal-arrow or association facts",
        );
    }
    let source_points = match exact_points(record, "curved-equilibrium arrow", 3) {
        Ok(points) => points,
        Err(detail) => return invalid_geometry(target, issues, detail.to_string()),
    };
    let [start, control, end] = source_points.as_slice() else {
        return invalid_geometry(
            target,
            issues,
            "curved-equilibrium arrow requires exactly three points",
        );
    };
    if !forward_tangents(*start, *control, *end) {
        return invalid_geometry(
            target,
            issues,
            "curved-equilibrium endpoint tangents must point along the start-to-end direction",
        );
    }
    let stroke = stroke(record, defaults, &target, issues);
    let source_path = match ArrowPathV1::try_new(source_points) {
        Ok(path) => path,
        Err(detail) => return invalid_geometry(target, issues, detail.to_string()),
    };
    match ArrowProjectionV1::try_new(
        target.clone(),
        source_path,
        ArrowProjectionKindV1::CurvedEquilibrium,
        stroke,
    ) {
        Ok(projection) => Some(projection),
        Err(detail) => invalid_geometry(target, issues, detail.to_string()),
    }
}

fn unsupported_arrow(
    target: PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
    kind: &str,
) -> Option<ArrowProjectionV1> {
    issues.push(PresentationProjectionIssueV1::new(
        target,
        PresentationProjectionIssueCodeV1::UnsupportedArrowType,
        format!("unsupported arrow type {kind:?}"),
    ));
    None
}

fn invalid_geometry(
    target: PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
    detail: impl Into<String>,
) -> Option<ArrowProjectionV1> {
    issues.push(PresentationProjectionIssueV1::new(
        target,
        PresentationProjectionIssueCodeV1::InvalidArrowGeometry,
        detail,
    ));
    None
}

fn invalid_fact(
    target: PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
    detail: impl Into<String>,
) -> Option<ArrowProjectionV1> {
    issues.push(PresentationProjectionIssueV1::new(
        target,
        PresentationProjectionIssueCodeV1::InvalidArrowFact,
        detail,
    ));
    None
}

fn is_nonspline(record: &TypedRecord) -> bool {
    matches!(
        record.attribute("spline"),
        None | Some("no" | "false" | "0")
    )
}

fn normal_heads(record: &TypedRecord) -> Result<(bool, bool), String> {
    Ok((
        head_fact(record, "start")?.unwrap_or(false),
        head_fact(record, "end")?.unwrap_or(true),
    ))
}

fn head_fact(record: &TypedRecord, field: &'static str) -> Result<Option<bool>, String> {
    match record.attribute(field) {
        None => Ok(None),
        Some("yes" | "true" | "1") => Ok(Some(true)),
        Some("no" | "false" | "0") => Ok(Some(false)),
        Some(_) => Err(format!("{field} must be yes, no, true, false, 1, or 0")),
    }
}

fn head_shape(record: &TypedRecord) -> Result<ArrowHeadShapeV1, String> {
    let Some(value) = record.attribute("shape") else {
        return ArrowHeadShapeV1::new(
            DEFAULT_HEAD_LINE_INSET,
            DEFAULT_HEAD_TOTAL_LENGTH,
            DEFAULT_HEAD_HALF_WIDTH,
        )
        .ok_or_else(|| "closed default arrow head shape is invalid".to_owned());
    };
    let Some(value) = value
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Err("shape must be (line_inset,total_length,half_width)".to_owned());
    };
    let values = value.split(',').map(str::trim).collect::<Vec<_>>();
    let [line_inset, total_length, half_width] = values.as_slice() else {
        return Err("shape must be (line_inset,total_length,half_width)".to_owned());
    };
    let parse = |value: &str| value.parse::<f64>().ok();
    ArrowHeadShapeV1::new(
        parse(line_inset).unwrap_or(f64::NAN),
        parse(total_length).unwrap_or(f64::NAN),
        parse(half_width).unwrap_or(f64::NAN),
    )
    .ok_or_else(|| {
        "shape dimensions must be finite, positive, and total_length >= line_inset".to_owned()
    })
}

fn exact_points(
    record: &TypedRecord,
    kind: &'static str,
    count: usize,
) -> Result<Vec<Point3V1>, String> {
    let points = points(record, 2, kind)?;
    (points.len() == count)
        .then_some(points)
        .ok_or_else(|| format!("{kind} requires exactly {count} points"))
}

fn has_normal_head_facts(record: &TypedRecord) -> bool {
    ["start", "end", "shape"]
        .into_iter()
        .any(|field| record.attribute(field).is_some())
}

fn has_nonterminal_facts(record: &TypedRecord) -> bool {
    has_normal_head_facts(record)
        || ["spline", "properties", "association", "factory"]
            .into_iter()
            .any(|field| record.attribute(field).is_some())
}

fn forward_tangents(start: Point3V1, control: Point3V1, end: Point3V1) -> bool {
    let chord_x = end.x() - start.x();
    let chord_y = end.y() - start.y();
    let start_dot = (control.x() - start.x()) * chord_x + (control.y() - start.y()) * chord_y;
    let end_dot = (end.x() - control.x()) * chord_x + (end.y() - control.y()) * chord_y;
    start_dot.is_finite() && end_dot.is_finite() && start_dot > 0.0 && end_dot > 0.0
}

fn stroke(
    record: &TypedRecord,
    defaults: RootStrokeDefaultsV1<'_>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> PresentationStrokeV1 {
    stroke_with_color_field(record, defaults, target, issues, "color")
}
