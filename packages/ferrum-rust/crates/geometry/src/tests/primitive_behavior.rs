use super::super::*;
use super::test_support::point;

#[test]
fn explicit_library_conversions_reject_nonfinite_external_points() {
    assert_eq!(
        Point2::try_from(kurbo::Point::new(f64::NAN, 0.0)),
        Err(GeometryError::NonFiniteCoordinate)
    );
    let original = point(1.5, -2.0);
    assert_eq!(Point2::try_from(original.to_kurbo()), Ok(original));
    assert_eq!(Point2::try_from(original.to_nalgebra()), Ok(original));
}

#[test]
fn transform_composition_has_explicit_order() {
    let move_right = Transform2::translation(2.0, 0.0).expect("finite");
    let turn = Transform2::rotation(std::f64::consts::FRAC_PI_2).expect("finite");
    let result = turn
        .after(move_right)
        .expect("finite")
        .apply(point(1.0, 0.0))
        .expect("finite");
    assert!(result.x().abs() < 1e-12);
    assert!((result.y() - 3.0).abs() < 1e-12);
}

#[test]
fn wedge_expands_from_tip_to_base() {
    let wedge =
        WedgeGeometry::new(point(0.0, 0.0), point(10.0, 0.0), 4.0, 0.0).expect("valid wedge");
    assert_eq!(wedge.wide_left, point(10.0, 2.0));
    assert_eq!(wedge.wide_right, point(10.0, -2.0));
    assert_eq!(wedge.area, 20.0);
}

#[test]
fn wedge_rejects_finite_inputs_that_overflow_public_geometry_facts() {
    assert_eq!(
        WedgeGeometry::new(point(0.0, 0.0), point(f64::MAX, 0.0), 3.0, 0.0),
        Err(GeometryError::UnrepresentableGeometry)
    );
    assert_eq!(
        WedgeGeometry::new(point(f64::MIN, 0.0), point(f64::MAX, 0.0), 1.0, 0.0),
        Err(GeometryError::UnrepresentableGeometry)
    );
    assert_eq!(
        WedgeGeometry::new(point(f64::MAX, 0.0), point(f64::MAX, 1.0), f64::MAX, 0.0),
        Err(GeometryError::UnrepresentableGeometry)
    );
}

#[test]
fn wedge_accepts_representable_geometry_near_f64_maximum() {
    let wedge = WedgeGeometry::new(point(0.0, 0.0), point(f64::MAX, 0.0), 1.0, 0.0)
        .expect("the area and every public fact are finite");
    assert!(wedge.length.is_finite());
    assert!(wedge.area.is_finite());
    assert_eq!(wedge.area, f64::MAX / 2.0);
    assert!(wedge.wide_left.x().is_finite());
    assert!(wedge.wide_right.x().is_finite());
}
