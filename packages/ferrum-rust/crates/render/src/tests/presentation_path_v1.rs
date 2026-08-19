use crate::{PathCommandV1, PathKindV1, RenderPoint, lower_authored_control_path_v1};

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("test point must be finite")
}

#[test]
fn zero_control_paths_are_straight_and_retain_their_kind() {
    for kind in [PathKindV1::Polyline, PathKindV1::AuthoredSpline] {
        let path = lower_authored_control_path_v1(kind, point(1.0, 2.0), &[], point(3.0, 4.0))
            .expect("zero-control path lowers");
        assert_eq!(path.kind(), kind);
        assert_eq!(
            path.commands(),
            &[
                PathCommandV1::MoveTo(point(1.0, 2.0)),
                PathCommandV1::LineTo(point(3.0, 4.0)),
            ]
        );
    }
}

#[test]
fn polyline_preserves_each_authored_intermediate_point() {
    let controls = [point(2.0, 3.0), point(4.0, 5.0)];
    let path = lower_authored_control_path_v1(
        PathKindV1::Polyline,
        point(0.0, 0.0),
        &controls,
        point(6.0, 7.0),
    )
    .expect("polyline lowers");
    assert_eq!(
        path.commands(),
        &[
            PathCommandV1::MoveTo(point(0.0, 0.0)),
            PathCommandV1::LineTo(controls[0]),
            PathCommandV1::LineTo(controls[1]),
            PathCommandV1::LineTo(point(6.0, 7.0)),
        ]
    );
}

#[test]
fn one_control_spline_elevates_its_quadratic_to_the_equivalent_cubic() {
    let path = lower_authored_control_path_v1(
        PathKindV1::AuthoredSpline,
        point(0.0, 0.0),
        &[point(3.0, 6.0)],
        point(9.0, 0.0),
    )
    .expect("quadratic lowers");
    assert_eq!(
        path.commands(),
        &[
            PathCommandV1::MoveTo(point(0.0, 0.0)),
            PathCommandV1::CubicTo {
                control_1: point(2.0, 4.0),
                control_2: point(5.0, 4.0),
                end: point(9.0, 0.0),
            },
        ]
    );
}

#[test]
fn two_control_spline_is_one_authored_cubic() {
    let controls = [point(2.0, 4.0), point(6.0, 4.0)];
    let path = lower_authored_control_path_v1(
        PathKindV1::AuthoredSpline,
        point(0.0, 0.0),
        &controls,
        point(8.0, 0.0),
    )
    .expect("cubic lowers");
    assert_eq!(
        path.commands(),
        &[
            PathCommandV1::MoveTo(point(0.0, 0.0)),
            PathCommandV1::CubicTo {
                control_1: controls[0],
                control_2: controls[1],
                end: point(8.0, 0.0),
            },
        ]
    );
}

#[test]
fn midpoint_chain_spline_has_continuous_segment_endpoints() {
    let controls = [point(2.0, 4.0), point(6.0, 8.0), point(10.0, 4.0)];
    let end = point(12.0, 0.0);
    let path =
        lower_authored_control_path_v1(PathKindV1::AuthoredSpline, point(0.0, 0.0), &controls, end)
            .expect("midpoint chain lowers");

    let cubics: Vec<_> = path
        .commands()
        .iter()
        .filter_map(|command| match command {
            PathCommandV1::CubicTo { end, .. } => Some(*end),
            PathCommandV1::MoveTo(_) | PathCommandV1::LineTo(_) | PathCommandV1::Close => None,
        })
        .collect();
    assert_eq!(cubics, vec![point(4.0, 6.0), point(8.0, 6.0), end]);
    assert_eq!(cubics.last(), Some(&end));
}

#[test]
fn nonfinite_authored_input_is_refused_at_the_render_point_boundary() {
    assert!(RenderPoint::new(f64::NAN, 0.0).is_err());
    assert!(RenderPoint::new(0.0, f64::INFINITY).is_err());
}

#[test]
fn finite_extreme_authored_input_still_produces_finite_derived_geometry() {
    let path = lower_authored_control_path_v1(
        PathKindV1::AuthoredSpline,
        point(f64::MAX, 0.0),
        &[point(-f64::MAX, 0.0)],
        point(f64::MAX, 0.0),
    )
    .expect("convex quadratic elevation remains finite");
    assert!(path.commands().iter().all(command_is_finite));
}

fn command_is_finite(command: &PathCommandV1) -> bool {
    match command {
        PathCommandV1::MoveTo(point) | PathCommandV1::LineTo(point) => {
            point.x().is_finite() && point.y().is_finite()
        }
        PathCommandV1::CubicTo {
            control_1,
            control_2,
            end,
        } => [control_1, control_2, end]
            .into_iter()
            .all(|point| point.x().is_finite() && point.y().is_finite()),
        PathCommandV1::Close => true,
    }
}
