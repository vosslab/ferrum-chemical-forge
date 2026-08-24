//! Typed-CDML adapter for immutable shape projection values.

use ferrum_document_projection::{
    BoxShapeProjectionV1, PolygonPathV1, PolygonProjectionV1, PresentationBoundsV1,
    PresentationFactProvenanceV1, PresentationFillV1, PresentationProjectionIssueCodeV1,
    PresentationProjectionIssueV1, PresentationTargetV1, TransparentOrRgb24V1,
};

use super::presentation_polyline_projection_v1::{
    RootStrokeDefaultsV1, coordinate, points, stroke,
};
use super::presentation_stack_projection_v1::presentation_target_from_child_v1;
use super::{TypedChild, TypedRecord};

pub(crate) fn box_shape(
    child: &TypedChild,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Result<Option<BoxShapeProjectionV1>, crate::ProjectionError> {
    let target = presentation_target_from_child_v1(child)?;
    Ok(box_shape_with_target(child, target, defaults, issues))
}

fn box_shape_with_target(
    child: &TypedChild,
    target: PresentationTargetV1,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<BoxShapeProjectionV1> {
    let record = child.record();
    let bounds = match bounds(record) {
        Ok(bounds) => bounds,
        Err(detail) => {
            issues.push(PresentationProjectionIssueV1::new(
                target,
                PresentationProjectionIssueCodeV1::InvalidShapeGeometry,
                detail,
            ));
            return None;
        }
    };
    let stroke = stroke(record, defaults, &target, issues);
    let fill = fill(record, defaults.standard, &target, issues);
    BoxShapeProjectionV1::try_new(target, bounds, stroke, fill).ok()
}

pub(crate) fn polygon(
    child: &TypedChild,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Result<Option<PolygonProjectionV1>, crate::ProjectionError> {
    let target = presentation_target_from_child_v1(child)?;
    Ok(polygon_with_target(child, target, defaults, issues))
}

fn polygon_with_target(
    child: &TypedChild,
    target: PresentationTargetV1,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<PolygonProjectionV1> {
    let record = child.record();
    let path = match points(record, 3, "polygon") {
        Ok(points) => match PolygonPathV1::try_new(points) {
            Ok(path) => path,
            Err(error) => {
                issues.push(PresentationProjectionIssueV1::new(
                    target,
                    PresentationProjectionIssueCodeV1::InvalidPolygonGeometry,
                    error.to_string(),
                ));
                return None;
            }
        },
        Err(detail) => {
            issues.push(PresentationProjectionIssueV1::new(
                target,
                PresentationProjectionIssueCodeV1::InvalidPolygonGeometry,
                detail,
            ));
            return None;
        }
    };
    let stroke = stroke(record, defaults, &target, issues);
    let fill = fill(record, defaults.standard, &target, issues);
    PolygonProjectionV1::try_new(target, path, stroke, fill).ok()
}

fn bounds(record: &TypedRecord) -> Result<PresentationBoundsV1, String> {
    let x1 = coordinate(record, "x1")?;
    let y1 = coordinate(record, "y1")?;
    let x2 = coordinate(record, "x2")?;
    let y2 = coordinate(record, "y2")?;
    PresentationBoundsV1::from_corners(x1, y1, x2, y2).map_err(|error| error.to_string())
}

fn fill(
    root: &TypedRecord,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> PresentationFillV1 {
    for (record, field, provenance) in [
        (Some(root), "area_color", PresentationFactProvenanceV1::Root),
        (
            root.attribute("area_color").is_none().then_some(root),
            "background-color",
            PresentationFactProvenanceV1::Root,
        ),
        (
            standard,
            "area_color",
            PresentationFactProvenanceV1::Standard,
        ),
    ] {
        let Some(record) = record else {
            continue;
        };
        let Some(value) = record.attribute(field) else {
            continue;
        };
        match TransparentOrRgb24V1::new(value) {
            Some(TransparentOrRgb24V1::Transparent) => {
                return PresentationFillV1::try_new(None, provenance)
                    .expect("transparent fill provenance is always valid");
            }
            Some(TransparentOrRgb24V1::Rgb24(color)) => {
                return PresentationFillV1::try_new(Some(color), provenance)
                    .expect("non-built-in validated fill colour is valid");
            }
            None => issues.push(PresentationProjectionIssueV1::new(
                target.clone(),
                PresentationProjectionIssueCodeV1::InvalidFillFact,
                format!("{field} must be empty, none, #rgb, or #rrggbb"),
            )),
        }
    }
    PresentationFillV1::try_new(None, PresentationFactProvenanceV1::Builtin)
        .expect("closed built-in transparent fill is valid")
}
