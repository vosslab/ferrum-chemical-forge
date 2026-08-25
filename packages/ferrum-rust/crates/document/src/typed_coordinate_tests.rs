use crate::{
    DocumentObjectIdV1, DocumentSession, PresentationRootProjectionV1, SessionOperation,
    SessionOperationV1, TopLevelRootKindV1, TopLevelRootSelectorV1, TopLevelRootTranslationV1,
    TransitionAuthorizationV1,
    typed_coordinate::{canonical_authored_coordinate, parse_coordinate},
};

fn selector(
    document_object_id: DocumentObjectIdV1,
    kind: TopLevelRootKindV1,
) -> TopLevelRootSelectorV1 {
    TopLevelRootSelectorV1::new(document_object_id, kind)
}

fn mixed_targets(session: &DocumentSession) -> Vec<TopLevelRootSelectorV1> {
    let revision = session.snapshot().expect("fixture snapshot").revision();
    let observation = session.observe(revision).expect("fixture observation");
    let projection = observation.projection();
    vec![
        selector(
            projection.molecules()[0]
                .id()
                .expect("fixture molecule is durable")
                .clone(),
            TopLevelRootKindV1::Molecule,
        ),
        selector(
            projection.presentation_stack().entries()[0]
                .root()
                .target()
                .document_object_id()
                .clone(),
            TopLevelRootKindV1::Plus,
        ),
    ]
}

fn translation(targets: Vec<TopLevelRootSelectorV1>, dx: f64, dy: f64) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::TranslateTopLevelRootsV1(
        TopLevelRootTranslationV1::new(targets, dx, dy).expect("fixture translation"),
    ))
}

fn submit_translation(
    session: &mut DocumentSession,
    expected_revision: u64,
    targets: Vec<TopLevelRootSelectorV1>,
    dx: f64,
    dy: f64,
) -> crate::SessionOperationResultV1 {
    let capability = session.issue_authoring_capability_v1();
    let mut prepared = session
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            expected_revision,
            translation(targets, dx, dy),
            TransitionAuthorizationV1::authoring_capability(capability),
        ))
        .expect("translation prepares");
    session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("translation commits")
}

#[test]
fn canonical_coordinates_round_trip_finite_point_space_values() {
    let centimetre_derived = 0.001 * 72.0 / 2.54;
    for points in [
        -f64::MAX,
        -47.0,
        -7.001_574_803_149_606,
        -f64::MIN_POSITIVE,
        -0.0,
        0.0,
        f64::MIN_POSITIVE,
        centimetre_derived,
        centimetre_derived / 2.0,
        7.0,
        47.0,
        7.001_574_803_149_606,
        f64::MAX,
    ] {
        let emitted = canonical_authored_coordinate(points);
        let reparsed = parse_coordinate(&emitted).expect("canonical coordinate reparses");
        let expected = if points == 0.0 { 0.0 } else { points };
        assert_eq!(
            reparsed.to_bits(),
            expected.to_bits(),
            "{points:?} -> {emitted}"
        );
    }
    assert_eq!(canonical_authored_coordinate(-0.0), "0");
    assert_eq!(canonical_authored_coordinate(7.0), "7");
    assert_eq!(canonical_authored_coordinate(47.0), "47");
    assert!(parse_coordinate("NaN").is_err());
    assert!(parse_coordinate("inf").is_err());
    assert!(parse_coordinate("1e308cm").is_err());
}

#[test]
fn shared_translation_preserves_exact_multi_root_geometry_after_reparse() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\">",
        "<point x=\"7\" y=\"7\"/></atom></molecule>",
        "<plus id=\"p\"><point x=\"47\" y=\"47\"/></plus></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let targets = mixed_targets(&session);
    let changed = submit_translation(&mut session, 0, targets, 7.0, 4.0);
    assert_eq!(changed.observation().snapshot().revision(), 1);
    let projection = changed.observation().projection();
    let atom = projection.molecules()[0].atoms()[0].position();
    assert_eq!((atom.x(), atom.y()), (14.0, 11.0));
    let PresentationRootProjectionV1::Plus { plus } =
        projection.presentation_stack().entries()[0].root()
    else {
        panic!("fixture plus remains projected");
    };
    assert_eq!((plus.anchor().x(), plus.anchor().y()), (54.0, 51.0));

    let mut reloaded = DocumentSession::load(changed.observation().snapshot().cdml())
        .expect("canonicalized document reloads");
    let targets = mixed_targets(&reloaded);
    let unchanged = submit_translation(&mut reloaded, 0, targets, 0.0, 0.0);
    assert_eq!(unchanged.observation().snapshot().revision(), 0);
    let reloaded_projection = unchanged.observation().projection();
    assert_eq!(
        (
            reloaded_projection.molecules()[0].atoms()[0].position().x(),
            reloaded_projection.molecules()[0].atoms()[0].position().y(),
        ),
        (14.0, 11.0)
    );
    let PresentationRootProjectionV1::Plus { plus } =
        reloaded_projection.presentation_stack().entries()[0].root()
    else {
        panic!("fixture plus remains projected after reload");
    };
    assert_eq!((plus.anchor().x(), plus.anchor().y()), (54.0, 51.0));
}

#[test]
fn nonfinite_transform_rejection_preserves_the_observation() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><plus id=\"p\">",
        "<point x=\"1.7976931348623157e308\" y=\"7\"/></plus></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("finite fixture loads");
    let before = session.snapshot().expect("pre-operation observation");
    let capability = session.issue_authoring_capability_v1();
    let revision = session.snapshot().expect("fixture snapshot").revision();
    let observation = session.observe(revision).expect("fixture observation");
    let target = selector(
        observation.projection().presentation_stack().entries()[0]
            .root()
            .target()
            .document_object_id()
            .clone(),
        TopLevelRootKindV1::Plus,
    );
    assert!(
        session
            .prepare_session_operation_transition_v1(
                crate::SessionOperationTransitionRequestV1::new(
                    0,
                    translation(vec![target], f64::MAX, 4.0),
                    TransitionAuthorizationV1::authoring_capability(capability),
                )
            )
            .is_err()
    );
    assert_eq!(
        session.snapshot().expect("post-operation observation"),
        before
    );
}
