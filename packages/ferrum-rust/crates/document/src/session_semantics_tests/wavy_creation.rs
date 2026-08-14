//! Prepared Rust-owned Wavy creation behavior.

use super::{DocumentSession, DocumentSessionError, Point3V1};
use crate::{PresentationRootProjectionV1, WAVY_MAX_SEGMENTS_V1};

fn point(x: f64, y: f64) -> Point3V1 {
    Point3V1::new(x, y, 0.0).expect("finite test point")
}

#[test]
fn prepared_wavy_creation_owns_geometry_identity_and_history() {
    let source = "<cdml><opaque id=\"ferrum-presentation-v1-0\"><keep/></opaque></cdml>";
    let mut session = DocumentSession::load(source).expect("source must load");
    let mut pending = session
        .prepare_create_wavy_v1(0, point(0.0, 0.0), point(48.0, 0.0))
        .expect("bounded Wavy gesture must prepare");
    assert_eq!(pending.identifier().as_str(), "ferrum-presentation-v1-1");
    assert_eq!(session.snapshot().expect("snapshot").revision(), 0);

    let result = session
        .commit_create_wavy(0, &mut pending)
        .expect("prepared Wavy must commit");
    let [PresentationRootProjectionV1::Wavy { polyline }] = result
        .observation()
        .projection()
        .presentation_stack()
        .roots()
    else {
        panic!("expected one Wavy root");
    };
    assert_eq!(
        polyline.target().source_id(),
        Some("ferrum-presentation-v1-1")
    );
    assert_eq!(polyline.path().points().len(), 5);
    let first = polyline.path().points().first().unwrap();
    let last = polyline.path().points().last().unwrap();
    assert_eq!((first.x(), first.y()), (0.0, 0.0));
    assert_eq!((last.x(), last.y()), (48.0, 0.0));
    assert_eq!(polyline.stroke().width().value(), 1.5);
    assert_eq!(polyline.stroke().color().as_str(), "#000000");
    assert!(result.observation().snapshot().cdml().contains("<keep"));
    session.undo(1).expect("creation must undo");
    session.redo(2).expect("creation must redo");
}

#[test]
fn invalid_or_stale_wavy_creation_never_mutates_or_consumes_the_next_identity() {
    let mut session = DocumentSession::load("<cdml/>").expect("source must load");
    let before = session.snapshot().expect("snapshot");
    assert!(
        session
            .prepare_create_wavy_v1(0, point(1.0, 1.0), point(1.0, 1.0))
            .is_err()
    );
    let over_bound = 12.0 * (WAVY_MAX_SEGMENTS_V1 as f64 + 0.6);
    assert!(
        session
            .prepare_create_wavy_v1(0, point(0.0, 0.0), point(over_bound, 0.0))
            .is_err()
    );
    assert_eq!(session.snapshot().expect("snapshot"), before);

    let mut pending = session
        .prepare_create_wavy_v1(0, point(0.0, 0.0), point(24.0, 0.0))
        .expect("valid Wavy must prepare");
    assert_eq!(pending.identifier().as_str(), "ferrum-presentation-v1-0");
    assert!(matches!(
        session.commit_create_wavy(1, &mut pending),
        Err(DocumentSessionError::RevisionConflict {
            expected: 1,
            actual: 0
        })
    ));
    session
        .commit_create_wavy(0, &mut pending)
        .expect("pending remains commit-ready after stale caller revision");
}
