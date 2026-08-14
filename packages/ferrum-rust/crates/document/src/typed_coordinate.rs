//! Shared persistent CDML coordinate parsing and canonical authored precision.

const POINTS_PER_CENTIMETRE: f64 = 72.0 / 2.54;

pub(crate) fn parse_coordinate(value: &str) -> Result<f64, ()> {
    let (raw, scale) = value.strip_suffix("cm").map_or_else(
        || (value.strip_suffix("px").unwrap_or(value), 1.0),
        |raw| (raw, POINTS_PER_CENTIMETRE),
    );
    let value = raw.parse::<f64>().map_err(|_| ())? * scale;
    value.is_finite().then_some(value).ok_or(())
}

pub(crate) fn coordinate_changes(old_points: f64, new_points: f64) -> bool {
    canonical_authored_coordinate(old_points) != canonical_authored_coordinate(new_points)
}

pub(crate) fn canonical_authored_coordinate(points: f64) -> String {
    let centimetres = points / POINTS_PER_CENTIMETRE;
    let centimetres = if centimetres.abs() < 0.0005 {
        0.0
    } else {
        centimetres
    };
    format!("{centimetres:.3}cm")
}
