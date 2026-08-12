use super::super::*;
use super::test_support::point;

#[test]
fn straighten_default_branch_prefers_chemical_orientation() {
    let angle = 10_f64.to_radians();
    let output = straighten_depiction(
        &[point(0.0, 0.0), point(angle.cos(), angle.sin())],
        &[(0, 1)],
        false,
    )
    .expect("valid bond");
    assert!((output.rotation_radians + angle).abs() < 1e-12);
    assert!((output.coordinates[1].x() - 1.0).abs() < 1e-12);
    assert!(output.coordinates[1].y().abs() < 1e-12);
}

#[test]
fn straighten_minimize_rotation_preserves_already_horizontal_orientation() {
    let output = straighten_depiction(&[point(0.0, 0.0), point(1.0, 0.0)], &[(0, 1)], true)
        .expect("valid bond");
    assert_eq!(output.rotation_radians, 0.0);
    assert_eq!(output.coordinates, vec![point(0.0, 0.0), point(1.0, 0.0)]);
}

#[test]
fn straighten_zero_length_bond_uses_the_documented_rdkit_normalization() {
    let coordinates = vec![point(2.0, -3.0), point(2.0, -3.0)];
    let output = straighten_depiction(&coordinates, &[(0, 1)], false)
        .expect("zero-length bonds are normalized by the RDKit-compatible policy");
    assert_eq!(output.rotation_radians, 0.0);
    assert_eq!(output.coordinates, coordinates);
}

#[test]
fn straighten_handles_increment_and_half_increment_angle_boundaries() {
    let thirty_degrees = 30_f64.to_radians();
    let exact_increment = straighten_depiction(
        &[
            point(0.0, 0.0),
            point(thirty_degrees.cos(), thirty_degrees.sin()),
        ],
        &[(0, 1)],
        true,
    )
    .expect("valid bond");
    assert!(exact_increment.rotation_radians.abs() < 1e-12);
    let fifteen_degrees = 15_f64.to_radians();
    let half_increment = straighten_depiction(
        &[
            point(0.0, 0.0),
            point(fifteen_degrees.cos(), fifteen_degrees.sin()),
        ],
        &[(0, 1)],
        false,
    )
    .expect("valid bond");
    assert!((half_increment.rotation_radians.abs() - fifteen_degrees).abs() < 1e-12);
}

#[test]
fn straighten_rejects_missing_bond_endpoint() {
    assert_eq!(
        straighten_depiction(&[point(0.0, 0.0)], &[(0, 1)], false),
        Err(GeometryError::BondIndexOutOfBounds { index: 1, len: 1 })
    );
}
