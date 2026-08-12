use crate::Point2;

/// Creates finite coordinates used by geometry unit tests.
pub(super) fn point(x: f64, y: f64) -> Point2 {
    Point2::new(x, y).expect("test coordinates are finite")
}
