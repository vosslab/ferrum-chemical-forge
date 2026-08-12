use std::collections::BTreeSet;

use crate::haworth::{
    BondDepiction, Face, HaworthError, RingForm, WedgeEdgeRole, layout_single_ring,
};

use super::fixtures::request;

#[test]
fn explicit_pyranose_layout_is_storage_and_cycle_direction_independent() {
    let first =
        layout_single_ring(&request(RingForm::Pyranose, 20.0, false, false)).expect("layout");
    let reordered =
        layout_single_ring(&request(RingForm::Pyranose, 20.0, true, true)).expect("layout");
    assert_eq!(first, reordered);
}

#[test]
fn furanose_layout_has_only_finite_scaled_coordinates() {
    let unit = layout_single_ring(&request(RingForm::Furanose, 1.0, false, false)).expect("layout");
    let scaled =
        layout_single_ring(&request(RingForm::Furanose, 7.5, false, false)).expect("layout");
    for (id, point) in scaled.coordinates() {
        assert!(point.x.is_finite() && point.y.is_finite());
        let unit_point = unit.coordinates().get(id).expect("same identity");
        assert_eq!(point.x, unit_point.x * 7.5);
        assert_eq!(point.y, unit_point.y * 7.5);
    }
}

#[test]
fn front_face_roles_are_unique_and_other_edges_are_back() {
    let depiction =
        layout_single_ring(&request(RingForm::Pyranose, 10.0, false, false)).expect("layout");
    let roles: BTreeSet<_> = depiction
        .bonds()
        .values()
        .filter_map(|bond| match bond {
            BondDepiction::HaworthFront { edge_role, face } => {
                assert_eq!(*face, Face::Front);
                Some(*edge_role)
            }
            BondDepiction::Back { face } => {
                assert_eq!(*face, Face::Back);
                None
            }
        })
        .collect();
    assert_eq!(
        roles,
        BTreeSet::from([
            WedgeEdgeRole::Center,
            WedgeEdgeRole::LeftShoulder,
            WedgeEdgeRole::RightShoulder,
        ])
    );
}

#[test]
fn rejects_nonpositive_and_nonfinite_scale_but_accepts_maximum_finite_scale() {
    for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let error = layout_single_ring(&request(RingForm::Furanose, scale, false, false))
            .expect_err("scale must reject");
        assert_eq!(
            error,
            HaworthError::InvalidSpec("scale must be finite and positive")
        );
    }
    let depiction = layout_single_ring(&request(RingForm::Furanose, f64::MAX, false, false))
        .expect("maximum finite scale remains representable");
    assert!(
        depiction
            .coordinates()
            .values()
            .all(|point| point.x.is_finite() && point.y.is_finite())
    );
}
