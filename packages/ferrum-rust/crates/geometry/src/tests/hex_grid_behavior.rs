use super::super::*;
use super::test_support::point;

#[test]
fn hex_grid_uses_euclidean_nearest_vertex_and_deterministic_ties() {
    let grid = HexGrid::new(2.0, point(0.0, 0.0)).expect("valid grid");
    assert_eq!(
        grid.point(HexIndex { n: 1, m: 0 }).expect("finite"),
        point(3_f64.sqrt(), 1.0)
    );
    assert_eq!(
        grid.nearest_index(point(3_f64.sqrt() / 2.0, 0.5))
            .expect("finite"),
        HexIndex { n: 0, m: 0 }
    );
    assert_eq!(
        grid.snap(point(1.72, 1.1)).expect("finite"),
        point(3_f64.sqrt(), 1.0)
    );
}

#[test]
fn hex_grid_constructor_guarantees_finite_basis_at_f64_maximum() {
    let grid = HexGrid::new(f64::MAX, point(0.0, 0.0)).expect("largest finite spacing");
    let (diagonal, vertical) = grid.basis_vectors();
    assert!(diagonal.x().is_finite());
    assert!(diagonal.y().is_finite());
    assert!(vertical.x().is_finite());
    assert!(vertical.y().is_finite());
    assert_eq!(vertical.y(), f64::MAX);
}

#[test]
fn hex_grid_rejects_nonfinite_or_nonpositive_spacing_before_basis_construction() {
    for spacing in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            HexGrid::new(spacing, point(0.0, 0.0)),
            Err(GeometryError::NonFiniteCoordinate)
        );
    }
    assert_eq!(
        HexGrid::new(0.0, point(0.0, 0.0)),
        Err(GeometryError::NonPositiveExtent)
    );
}

#[test]
fn hex_grid_generates_bounded_overlay_geometry() {
    let grid = HexGrid::new(1.0, point(0.0, 0.0)).expect("valid grid");
    let minimum = point(0.0, 0.0);
    let maximum = point(3.0, 3.0);
    let points = grid
        .points_in_rect(minimum, maximum)
        .expect("finite bounds")
        .expect("small overlay");
    let edges = grid
        .honeycomb_edges_in_rect(minimum, maximum)
        .expect("finite bounds")
        .expect("small overlay");
    assert!(
        points
            .iter()
            .all(|candidate| candidate.x() >= 0.0 && candidate.y() >= 0.0)
    );
    assert!(!edges.is_empty());
}

#[test]
fn hex_grid_rejects_unrepresentable_extreme_rectangles_without_panicking() {
    let grid = HexGrid::new(f64::MIN_POSITIVE, point(0.0, 0.0)).expect("valid grid");
    assert_eq!(
        grid.points_in_rect(point(-1.0, -1.0), point(1.0, 1.0)),
        Err(GeometryError::GridIndexUnrepresentable)
    );
    assert_eq!(
        grid.honeycomb_edges_in_rect(point(-1.0, -1.0), point(1.0, 1.0)),
        Err(GeometryError::GridIndexUnrepresentable)
    );
}

#[test]
fn hex_grid_nearest_index_handles_exact_i64_float_boundaries_without_saturation() {
    let grid = HexGrid::new(1.0, point(0.0, 0.0)).expect("valid grid");
    let positive_limit = i64::MAX as f64;
    let negative_limit = i64::MIN as f64;
    let horizontal_step = 3_f64.sqrt() / 2.0;
    assert_eq!(
        grid.nearest_index(point(positive_limit * horizontal_step, 0.0)),
        Err(GeometryError::GridIndexUnrepresentable)
    );
    assert_eq!(
        grid.snap(point(positive_limit * horizontal_step, 0.0)),
        Err(GeometryError::GridIndexUnrepresentable)
    );
    assert_eq!(
        grid.snap(point(positive_limit * horizontal_step, 0.0)),
        Err(GeometryError::GridIndexUnrepresentable)
    );
    assert_eq!(
        grid.nearest_index(point(0.0, positive_limit)),
        Err(GeometryError::GridIndexUnrepresentable)
    );
    assert_eq!(
        grid.snap(point(0.0, positive_limit)),
        Err(GeometryError::GridIndexUnrepresentable)
    );
    assert_eq!(
        grid.snap(point(0.0, positive_limit)),
        Err(GeometryError::GridIndexUnrepresentable)
    );
    assert_eq!(
        grid.nearest_index(point(
            negative_limit * horizontal_step,
            negative_limit / 2.0,
        )),
        Ok(HexIndex { n: i64::MIN, m: 0 })
    );
    assert_eq!(
        grid.snap(point(
            negative_limit * horizontal_step,
            negative_limit / 2.0,
        )),
        Ok(point(
            negative_limit * horizontal_step,
            negative_limit / 2.0,
        ))
    );
    assert_eq!(
        grid.nearest_index(point(0.0, negative_limit)),
        Ok(HexIndex { n: 0, m: i64::MIN })
    );
    assert_eq!(
        grid.snap(point(0.0, negative_limit)),
        Ok(point(0.0, negative_limit))
    );
}

#[test]
fn hex_grid_accepts_the_largest_representable_float_index_below_the_upper_boundary() {
    let grid = HexGrid::new(1.0, point(0.0, 0.0)).expect("valid grid");
    let upper_boundary = i64::MAX as f64;
    let adjacent = f64::from_bits(upper_boundary.to_bits() - 1);
    assert!(
        grid.nearest_index(point(adjacent * 3_f64.sqrt() / 2.0, adjacent / 2.0))
            .is_ok()
    );
    assert!(grid.snap(point(0.0, adjacent)).is_ok());
}

#[test]
fn hex_grid_rectangle_boundary_returns_a_typed_error_without_range_overflow() {
    let grid = HexGrid::new(1.0, point(0.0, 0.0)).expect("valid grid");
    let boundary = i64::MAX as f64 * 3_f64.sqrt() / 2.0;
    assert_eq!(
        grid.points_in_rect(point(boundary, 0.0), point(boundary, 1.0)),
        Err(GeometryError::GridIndexUnrepresentable)
    );
}

#[test]
fn hex_grid_distinguishes_invalid_bounds_from_a_bounded_display_omission() {
    let grid = HexGrid::new(1.0, point(-10.0, -10.0)).expect("valid grid");
    assert_eq!(
        grid.points_in_rect(point(2.0, 0.0), point(-2.0, 0.0)),
        Err(GeometryError::InvalidBounds)
    );
    assert_eq!(
        grid.points_in_rect(point(-1_000.0, -1_000.0), point(1_000.0, 1_000.0)),
        Ok(None)
    );
}
