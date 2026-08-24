//! Normalized display facts for direct atom-owned marks.

use std::collections::HashMap;

use crate::{
    AtomMarkKindV1, AtomMarkProjectionV1, Point3V1, PositiveFiniteV1, ProjectionIssueCodeV1,
    ProjectionIssueV1, TypedClass, TypedRecord, VisibilityV1,
};

const POINTS_PER_CENTIMETRE: f64 = 72.0 / 2.54;

pub(crate) fn atom_marks(
    atom: &TypedRecord,
    atom_position: Point3V1,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Vec<AtomMarkProjectionV1> {
    let mut ordinals = HashMap::<AtomMarkKindV1, u32>::new();
    atom.typed_children()
        .iter()
        .filter(|child| child.record().class() == TypedClass::Mark)
        .filter_map(|child| {
            let record = child.record();
            let Some(source_kind) = record.attribute("type") else {
                invalid(issues, record, "atom mark type is absent");
                return None;
            };
            let Some(kind) = AtomMarkKindV1::parse(source_kind) else {
                invalid(
                    issues,
                    record,
                    format!("atom mark type {source_kind:?} is unsupported"),
                );
                return None;
            };
            let ordinal = *ordinals.entry(kind).or_default();
            *ordinals.entry(kind).or_default() = ordinal.saturating_add(1);
            let (angle_degrees, radial_offset) = geometry(atom_position, record, issues);
            Some(
                AtomMarkProjectionV1::new(
                    kind,
                    child.position(),
                    ordinal,
                    angle_degrees,
                    radial_offset,
                    positive(record, "size", default_size(kind), issues),
                    circle(record, issues),
                    positive(record, "line_width", 1.0, issues),
                )
                .expect("document adapter supplies finite nonnegative atom-mark geometry"),
            )
        })
        .collect()
}

fn geometry(
    atom_position: Point3V1,
    mark: &TypedRecord,
    issues: &mut Vec<ProjectionIssueV1>,
) -> (f64, f64) {
    let (x, y) = match (mark.attribute("x"), mark.attribute("y")) {
        (None, None) => return (0.0, 12.0),
        (Some(x), Some(y)) => (x, y),
        _ => {
            invalid(issues, mark, "atom mark coordinates require both x and y");
            return (0.0, 12.0);
        }
    };
    let Some(x) = scene_coordinate(x) else {
        invalid(issues, mark, "atom mark x coordinate is invalid");
        return (0.0, 12.0);
    };
    let Some(y) = scene_coordinate(y) else {
        invalid(issues, mark, "atom mark y coordinate is invalid");
        return (0.0, 12.0);
    };
    let dx = x - atom_position.x();
    let dy = y - atom_position.y();
    let offset = dx.hypot(dy);
    if !offset.is_finite() {
        invalid(issues, mark, "atom mark radial offset is not finite");
        return (0.0, 12.0);
    }
    let angle = if offset == 0.0 {
        0.0
    } else {
        dy.atan2(dx).to_degrees()
    };
    (angle, offset)
}

fn circle(record: &TypedRecord, issues: &mut Vec<ProjectionIssueV1>) -> bool {
    let Some(source) = record.attribute("draw_circle") else {
        return true;
    };
    match VisibilityV1::parse(source) {
        Some(value) => value == VisibilityV1::Enabled,
        None => {
            invalid(
                issues,
                record,
                "atom mark draw_circle must be a supported boolean",
            );
            true
        }
    }
}

fn scene_coordinate(value: &str) -> Option<f64> {
    let (raw, scale) = value
        .strip_suffix("cm")
        .map_or((value, 1.0), |raw| (raw, POINTS_PER_CENTIMETRE));
    let value = raw.parse::<f64>().ok()? * scale;
    value.is_finite().then_some(value)
}

fn default_size(kind: AtomMarkKindV1) -> f64 {
    match kind {
        AtomMarkKindV1::Plus | AtomMarkKindV1::Minus | AtomMarkKindV1::Electronpair => 10.0,
        AtomMarkKindV1::PzOrbital => 40.0,
        AtomMarkKindV1::Radical
        | AtomMarkKindV1::Biradical
        | AtomMarkKindV1::DottedElectronpair => 4.0,
    }
}

fn positive(
    record: &TypedRecord,
    field: &'static str,
    default: f64,
    issues: &mut Vec<ProjectionIssueV1>,
) -> PositiveFiniteV1 {
    let parsed = record
        .attribute(field)
        .and_then(|value| value.parse::<f64>().ok())
        .and_then(PositiveFiniteV1::new);
    if record.attribute(field).is_some() && parsed.is_none() {
        invalid(
            issues,
            record,
            format!("atom mark {field} must be positive and finite"),
        );
    }
    parsed.unwrap_or_else(|| {
        PositiveFiniteV1::new(default).expect("atom mark compatibility defaults are positive")
    })
}

fn invalid(issues: &mut Vec<ProjectionIssueV1>, record: &TypedRecord, detail: impl Into<String>) {
    issues.push(
        ProjectionIssueV1::try_new(
            ProjectionIssueCodeV1::InvalidPresentationFact,
            record.path().to_string(),
            detail.into(),
        )
        .expect("typed record paths are nonempty structural locations"),
    );
}
