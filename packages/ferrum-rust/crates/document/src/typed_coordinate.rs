//! Shared persistent CDML coordinate parsing and canonical authored precision.

const POINTS_PER_CENTIMETRE: f64 = 72.0 / 2.54;

pub(crate) fn parse_coordinate(value: &str) -> Result<f64, ()> {
    let (raw, scale) = value.strip_suffix("cm").map_or_else(
        || (value.strip_suffix("px").unwrap_or(value), 1.0),
        |raw| (raw, POINTS_PER_CENTIMETRE),
    );
    let value = raw.parse::<f64>().map_err(|_| ())? * scale;
    normalize_finite_coordinate(value)
}

pub(crate) fn coordinate_changes(old_points: f64, new_points: f64) -> bool {
    normalize_finite_coordinate(old_points) != normalize_finite_coordinate(new_points)
}

pub(crate) fn canonical_authored_coordinate(points: f64) -> String {
    let points = normalize_finite_coordinate(points)
        .expect("typed document mutations serialize only finite coordinates");
    points.to_string()
}

fn normalize_finite_coordinate(value: f64) -> Result<f64, ()> {
    if !value.is_finite() {
        return Err(());
    }
    Ok(if value == 0.0 { 0.0 } else { value })
}
