use crate::{
    DocumentSession, DocumentSessionError, PresentationRootProjectionV1, SessionOperation,
    SessionOperationError, SessionOperationV1, TopLevelRootKindV1, TopLevelRootSelectorV1,
    TopLevelTransformModeV1, TopLevelTransformV1, TopLevelTransformV1Error, TypedDocumentError,
};

const MIXED_SOURCE: &str = concat!(
    "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\">",
    "<point x=\"1\" y=\"2\"/><mark type=\"plus\" x=\"1.5\" y=\"2.5\"/>",
    "</atom></molecule><plus id=\"p\" retained=\"yes\"><point x=\"5\" y=\"7\"/>",
    "<opaque kept=\"yes\"/></plus></cdml>",
);

const AUTHORED_HALF_UNIT_POINTS: f64 = (0.001 * 72.0 / 2.54) / 2.0;

fn assert_authored_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= AUTHORED_HALF_UNIT_POINTS);
}

fn selector(id: &str, kind: TopLevelRootKindV1) -> TopLevelRootSelectorV1 {
    TopLevelRootSelectorV1::new(id, kind).expect("fixture selector")
}

fn operation(
    targets: Vec<TopLevelRootSelectorV1>,
    transform: TopLevelTransformModeV1,
) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::TransformTopLevelRoots {
        transform: TopLevelTransformV1::new(targets, transform).expect("fixture transform"),
    })
}

#[test]
fn rigid_translation_moves_molecule_and_presentation_and_history_restores_geometry() {
    let mut session = DocumentSession::load(MIXED_SOURCE).expect("fixture loads");
    let result = session
        .submit(
            0,
            operation(
                vec![
                    selector("m", TopLevelRootKindV1::Molecule),
                    selector("p", TopLevelRootKindV1::Plus),
                ],
                TopLevelTransformModeV1::Translate { dx: 3.0, dy: -1.0 },
            ),
        )
        .expect("rigid translation succeeds");
    let projection = result.observation().projection();
    assert_authored_close(projection.molecules()[0].atoms()[0].position().x(), 4.0);
    assert_authored_close(projection.molecules()[0].atoms()[0].position().y(), 1.0);
    let PresentationRootProjectionV1::Plus { plus } = &projection.presentation_stack().roots()[0]
    else {
        panic!("fixture plus remains projected");
    };
    assert_authored_close(plus.anchor().x(), 8.0);
    assert_authored_close(plus.anchor().y(), 6.0);
    assert!(
        result
            .observation()
            .snapshot()
            .cdml()
            .contains("retained=\"yes\"")
    );
    assert!(
        result
            .observation()
            .snapshot()
            .cdml()
            .contains("kept=\"yes\"")
    );

    let undone = session.undo(1).expect("translation is one history entry");
    assert_eq!(
        undone.observation().projection().molecules()[0].atoms()[0]
            .position()
            .x(),
        1.0
    );
    let PresentationRootProjectionV1::Plus { plus } = &undone
        .observation()
        .projection()
        .presentation_stack()
        .roots()[0]
    else {
        panic!("fixture plus remains projected after undo");
    };
    assert_eq!(plus.anchor().x(), 5.0);
}

#[test]
fn alignment_is_semantic_and_a_zero_translation_is_history_free() {
    let source = concat!(
        "<cdml><plus id=\"a\"><point x=\"2\" y=\"4\"/></plus>",
        "<plus id=\"b\"><point x=\"8\" y=\"9\"/></plus></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let aligned = session
        .submit(
            0,
            operation(
                vec![
                    selector("a", TopLevelRootKindV1::Plus),
                    selector("b", TopLevelRootKindV1::Plus),
                ],
                TopLevelTransformModeV1::AlignLeft,
            ),
        )
        .expect("alignment succeeds");
    let anchors = aligned
        .observation()
        .projection()
        .presentation_stack()
        .roots()
        .iter()
        .map(|root| match root {
            PresentationRootProjectionV1::Plus { plus } => plus.anchor(),
            _ => panic!("fixture contains only plus roots"),
        })
        .collect::<Vec<_>>();
    assert_authored_close(anchors[0].x(), 2.0);
    assert_authored_close(anchors[1].x(), 2.0);

    let unchanged = session
        .submit(
            1,
            operation(
                vec![selector("a", TopLevelRootKindV1::Plus)],
                TopLevelTransformModeV1::Translate { dx: 0.0, dy: 0.0 },
            ),
        )
        .expect("zero translation is accepted");
    assert_eq!(unchanged.observation().snapshot().revision(), 1);
}

#[test]
fn scale_uses_aggregate_center_and_retires_only_invalid_owned_metadata() {
    let source = concat!(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"b\" name=\"C\"><point x=\"10\" y=\"0\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/>",
        "<fragment id=\"owned\" type=\"linear_form\"><name>linear_form</name>",
        "<bond id=\"ab\"/><vertex id=\"a\"/><vertex id=\"b\"/>",
        "<property name=\"bond_length\" value=\"10\" type=\"IntType\"/></fragment>",
        "<fragment id=\"richer\" type=\"linear_form\" retained=\"yes\">",
        "<name>linear_form</name><extension/></fragment></molecule>",
        "<plus id=\"p\"><point x=\"20\" y=\"0\"/></plus></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let scaled = session
        .submit(
            0,
            operation(
                vec![
                    selector("m", TopLevelRootKindV1::Molecule),
                    selector("p", TopLevelRootKindV1::Plus),
                ],
                TopLevelTransformModeV1::Scale {
                    scale_x: 2.0,
                    scale_y: 1.0,
                },
            ),
        )
        .expect("scale succeeds");
    let projection = scaled.observation().projection();
    assert_authored_close(projection.molecules()[0].atoms()[0].position().x(), -10.0);
    assert_authored_close(projection.molecules()[0].atoms()[1].position().x(), 10.0);
    let PresentationRootProjectionV1::Plus { plus } = &projection.presentation_stack().roots()[0]
    else {
        panic!("plus remains projected");
    };
    assert_authored_close(plus.anchor().x(), 30.0);
    let cdml = scaled.observation().snapshot().cdml();
    assert!(!cdml.contains("id=\"owned\""));
    assert!(cdml.contains("id=\"richer\""));
    assert!(cdml.contains("retained=\"yes\""));

    let unchanged = session
        .submit(
            1,
            operation(
                vec![selector("m", TopLevelRootKindV1::Molecule)],
                TopLevelTransformModeV1::Scale {
                    scale_x: 1.0,
                    scale_y: 1.0,
                },
            ),
        )
        .expect("identity scale is accepted");
    assert_eq!(unchanged.observation().snapshot().revision(), 1);
}

#[test]
fn mirrors_share_one_pivot_and_metadata_retirement_is_semantic() {
    let source = concat!(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"5\"/></atom>",
        "<atom id=\"b\" name=\"C\"><point x=\"10\" y=\"5\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/>",
        "<fragment id=\"owned\" type=\"linear_form\"><name>linear_form</name>",
        "<bond id=\"ab\"/><vertex id=\"a\"/><vertex id=\"b\"/>",
        "<property name=\"bond_length\" value=\"10\" type=\"IntType\"/></fragment>",
        "</molecule><plus id=\"p\"><point x=\"20\" y=\"15\"/></plus></cdml>",
    );
    let targets = vec![
        selector("m", TopLevelRootKindV1::Molecule),
        selector("p", TopLevelRootKindV1::Plus),
    ];
    let mut horizontal = DocumentSession::load(source).expect("fixture loads");
    let mirrored = horizontal
        .submit(
            0,
            operation(targets.clone(), TopLevelTransformModeV1::MirrorHorizontal),
        )
        .expect("horizontal mirror succeeds");
    assert_authored_close(
        mirrored.observation().projection().molecules()[0].atoms()[0]
            .position()
            .y(),
        15.0,
    );
    assert!(
        mirrored
            .observation()
            .snapshot()
            .cdml()
            .contains("id=\"owned\"")
    );

    let mut vertical = DocumentSession::load(source).expect("fixture loads");
    let mirrored = vertical
        .submit(
            0,
            operation(targets, TopLevelTransformModeV1::MirrorVertical),
        )
        .expect("vertical mirror succeeds");
    assert_authored_close(
        mirrored.observation().projection().molecules()[0].atoms()[0]
            .position()
            .x(),
        20.0,
    );
    assert!(
        !mirrored
            .observation()
            .snapshot()
            .cdml()
            .contains("id=\"owned\"")
    );
}

#[test]
fn malformed_later_root_rejects_the_whole_transform() {
    let source = concat!(
        "<cdml><plus id=\"good\"><point x=\"2\" y=\"4\"/></plus>",
        "<text id=\"bad\"><point x=\"8\" y=\"9\"/><point x=\"9\" y=\"9\"/>",
        "<ftext>bad geometry</ftext></text></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let before = session.snapshot().expect("snapshot");
    let error = session
        .submit(
            0,
            operation(
                vec![
                    selector("good", TopLevelRootKindV1::Plus),
                    selector("bad", TopLevelRootKindV1::Text),
                ],
                TopLevelTransformModeV1::AlignTop,
            ),
        )
        .expect_err("ambiguous later target rejects the request");
    assert!(matches!(
        error,
        DocumentSessionError::Operation(SessionOperationError::Candidate(
            TypedDocumentError::InvalidTopLevelTransformGeometry(_)
        ))
    ));
    let after = session.snapshot().expect("snapshot");
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.digest(), before.digest());
}

#[test]
fn transform_grammar_rejects_unbounded_or_ambiguous_intent() {
    let one = vec![selector("a", TopLevelRootKindV1::Plus)];
    assert_eq!(
        TopLevelTransformV1::new(one.clone(), TopLevelTransformModeV1::AlignLeft),
        Err(TopLevelTransformV1Error::AlignmentNeedsTwoTargets)
    );
    assert_eq!(
        TopLevelTransformV1::new(
            vec![one[0].clone(), one[0].clone()],
            TopLevelTransformModeV1::Translate { dx: 1.0, dy: 2.0 }
        ),
        Err(TopLevelTransformV1Error::DuplicateTarget)
    );
    assert_eq!(
        TopLevelTransformV1::new(
            one,
            TopLevelTransformModeV1::Translate {
                dx: f64::NAN,
                dy: 0.0
            }
        ),
        Err(TopLevelTransformV1Error::NonFiniteTranslation)
    );
    assert_eq!(
        TopLevelTransformV1::new(
            vec![selector("a", TopLevelRootKindV1::Plus)],
            TopLevelTransformModeV1::Scale {
                scale_x: 0.0,
                scale_y: 1.0,
            }
        ),
        Err(TopLevelTransformV1Error::InvalidScaleFactors)
    );
}
