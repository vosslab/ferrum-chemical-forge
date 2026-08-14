use crate::{GeometryError, MoleculePlacementV1, Point2, place_molecule_depiction_v1};

fn point(x: f64, y: f64) -> Point2 {
    Point2::new(x, y).expect("finite test point")
}

#[test]
fn placement_scales_centers_and_converts_y_up_to_y_down() {
    let placement =
        MoleculePlacementV1::new(40.0, point(100.0, 200.0)).expect("positive placement");
    let placed = place_molecule_depiction_v1(
        &[point(0.0, 0.0), point(2.0, 2.0), point(4.0, 0.0)],
        &[(0, 1), (1, 2)],
        placement,
    )
    .expect("valid molecule placement");
    let centroid_x = placed.iter().map(|value| value.x()).sum::<f64>() / 3.0;
    let centroid_y = placed.iter().map(|value| value.y()).sum::<f64>() / 3.0;
    assert_eq!((centroid_x, centroid_y), (100.0, 200.0));
    assert!((placed[0].distance_to(placed[1]) - 40.0).abs() < f64::EPSILON * 32.0);
    assert!(placed[1].y() < placed[0].y());
}

#[test]
fn bondless_molecule_is_centered_without_inventing_scale() {
    let placement = MoleculePlacementV1::new(40.0, point(8.0, 9.0)).expect("positive placement");
    let placed = place_molecule_depiction_v1(&[point(5.0, -4.0)], &[], placement)
        .expect("single atom placement");
    assert_eq!(placed, vec![point(8.0, 9.0)]);
}

#[test]
fn malformed_topology_and_coincident_bonds_fail_closed() {
    let placement = MoleculePlacementV1::new(40.0, point(0.0, 0.0)).expect("positive placement");
    assert_eq!(
        place_molecule_depiction_v1(&[point(0.0, 0.0)], &[(0, 1)], placement),
        Err(GeometryError::BondIndexOutOfBounds { index: 1, len: 1 })
    );
    assert_eq!(
        place_molecule_depiction_v1(&[point(0.0, 0.0), point(0.0, 0.0)], &[(0, 1)], placement,),
        Err(GeometryError::ZeroLengthBond)
    );
}
