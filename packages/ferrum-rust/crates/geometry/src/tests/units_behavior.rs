use super::super::*;

#[test]
fn cdml_centimetres_use_the_established_exact_v1_scale() {
    assert_eq!(CDML_POINTS_PER_CENTIMETRE_V1, 72.0 / 2.54);
    let centimetres = CdmlLength::try_from_centimetres(2.54).expect("finite centimetres");
    let points = centimetres.as_scene_points().expect("representable points");
    assert_eq!(points.as_scene_points(), 72.0);
    assert_eq!(
        points.as_centimetres().expect("representable centimetres"),
        centimetres
    );
}

#[test]
fn cdml_unit_conversions_reject_nonfinite_and_overflowing_results() {
    assert_eq!(
        CdmlLength::try_from_centimetres(f64::NAN),
        Err(GeometryError::NonFiniteCoordinate)
    );
    assert_eq!(
        ScenePoints::try_from_scene_points(f64::INFINITY),
        Err(GeometryError::NonFiniteCoordinate)
    );
    assert_eq!(
        CdmlLength::try_from_centimetres(f64::MAX).and_then(CdmlLength::as_scene_points),
        Err(GeometryError::UnrepresentableGeometry)
    );
}
