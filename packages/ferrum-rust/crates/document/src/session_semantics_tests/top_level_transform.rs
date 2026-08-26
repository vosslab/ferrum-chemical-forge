use crate::{
    DocumentObjectIdV1, DocumentSession, DocumentSessionError, PresentationRootProjectionV1,
    SessionOperation, SessionOperationError, SessionOperationV1, TopLevelRootKindV1,
    TopLevelRootLayoutTransformModeV1, TopLevelRootLayoutTransformV1, TopLevelRootSelectorV1,
    TopLevelRootTranslationV1, TopLevelTransformModeV1, TopLevelTransformV1,
    TopLevelTransformV1Error, TransitionAuthorizationV1, TypedDocumentError,
};
use ferrum_geometry::{HexGrid, Point2};

const MIXED_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\">",
    "<point x=\"1\" y=\"2\"/><mark type=\"plus\" x=\"1.5\" y=\"2.5\"/>",
    "</atom></molecule><plus id=\"p\" retained=\"yes\"><point x=\"5\" y=\"7\"/>",
    "<opaque kept=\"yes\"/></plus></cdml>",
);

const AUTHORED_HALF_UNIT_POINTS: f64 = (0.001 * 72.0 / 2.54) / 2.0;

fn assert_authored_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= AUTHORED_HALF_UNIT_POINTS);
}

fn selector(
    document_object_id: DocumentObjectIdV1,
    kind: TopLevelRootKindV1,
) -> TopLevelRootSelectorV1 {
    TopLevelRootSelectorV1::new(document_object_id, kind)
}

fn molecule_selector(session: &DocumentSession, index: usize) -> TopLevelRootSelectorV1 {
    let revision = session.snapshot().expect("fixture snapshot").revision();
    let document_object_id = session
        .observe(revision)
        .expect("fixture observation")
        .projection()
        .molecules()[index]
        .id()
        .expect("fixture molecule is durable")
        .clone();
    selector(document_object_id, TopLevelRootKindV1::Molecule)
}

fn presentation_selector(
    session: &DocumentSession,
    index: usize,
    kind: TopLevelRootKindV1,
) -> TopLevelRootSelectorV1 {
    let revision = session.snapshot().expect("fixture snapshot").revision();
    let document_object_id = session
        .observe(revision)
        .expect("fixture observation")
        .projection()
        .presentation_stack()
        .entries()[index]
        .root()
        .target()
        .document_object_id()
        .clone();
    selector(document_object_id, kind)
}

fn mixed_targets(session: &DocumentSession) -> Vec<TopLevelRootSelectorV1> {
    vec![
        molecule_selector(session, 0),
        presentation_selector(session, 0, TopLevelRootKindV1::Plus),
    ]
}

fn opaque_selector(entropy_byte: u8, kind: TopLevelRootKindV1) -> TopLevelRootSelectorV1 {
    selector(
        DocumentObjectIdV1::from_entropy_bytes([entropy_byte; 16]),
        kind,
    )
}

fn operation(
    targets: Vec<TopLevelRootSelectorV1>,
    transform: TopLevelTransformModeV1,
) -> SessionOperation {
    match transform {
        TopLevelTransformModeV1::Translate { dx, dy } => {
            SessionOperation::V1(SessionOperationV1::TranslateTopLevelRootsV1(
                TopLevelRootTranslationV1::new(targets, dx, dy).expect("fixture translation"),
            ))
        }
        TopLevelTransformModeV1::AlignTop => {
            layout_operation(targets, TopLevelRootLayoutTransformModeV1::AlignTop)
        }
        TopLevelTransformModeV1::AlignBottom => {
            layout_operation(targets, TopLevelRootLayoutTransformModeV1::AlignBottom)
        }
        TopLevelTransformModeV1::AlignLeft => {
            layout_operation(targets, TopLevelRootLayoutTransformModeV1::AlignLeft)
        }
        TopLevelTransformModeV1::AlignRight => {
            layout_operation(targets, TopLevelRootLayoutTransformModeV1::AlignRight)
        }
        TopLevelTransformModeV1::AlignCenterX => {
            layout_operation(targets, TopLevelRootLayoutTransformModeV1::AlignCenterX)
        }
        TopLevelTransformModeV1::AlignCenterY => {
            layout_operation(targets, TopLevelRootLayoutTransformModeV1::AlignCenterY)
        }
        TopLevelTransformModeV1::Scale { scale_x, scale_y } => layout_operation(
            targets,
            TopLevelRootLayoutTransformModeV1::Scale { scale_x, scale_y },
        ),
        TopLevelTransformModeV1::MirrorVertical => {
            layout_operation(targets, TopLevelRootLayoutTransformModeV1::MirrorVertical)
        }
        TopLevelTransformModeV1::MirrorHorizontal => {
            layout_operation(targets, TopLevelRootLayoutTransformModeV1::MirrorHorizontal)
        }
    }
}

fn layout_operation(
    targets: Vec<TopLevelRootSelectorV1>,
    mode: TopLevelRootLayoutTransformModeV1,
) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::ApplyTopLevelRootLayoutTransformV1(
        TopLevelRootLayoutTransformV1::new(targets, mode).expect("fixture layout transform"),
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
            SessionOperation::V1(SessionOperationV1::TranslateTopLevelRootsV1(
                TopLevelRootTranslationV1::new(targets, dx, dy).expect("fixture translation"),
            )),
            TransitionAuthorizationV1::authoring_capability(capability),
        ))
        .expect("interaction translation prepares");
    session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("interaction translation commits")
}

#[test]
fn direct_layouts_use_explicit_none_and_refuse_authoring_capabilities_before_mutation() {
    let mut session = DocumentSession::load(MIXED_SOURCE).expect("fixture loads");
    let targets = mixed_targets(&session);
    let before = session.snapshot().expect("snapshot");
    let capability = session.issue_authoring_capability_v1();
    assert!(matches!(
        session.prepare_session_operation_transition_v1(
            crate::SessionOperationTransitionRequestV1::new(
                before.revision(),
                layout_operation(
                    targets.clone(),
                    TopLevelRootLayoutTransformModeV1::Scale {
                        scale_x: 2.0,
                        scale_y: 1.0,
                    },
                ),
                TransitionAuthorizationV1::authoring_capability(capability),
            )
        ),
        Err(DocumentSessionError::TransitionAuthorization(
            crate::TransitionAuthorizationRefusalV1::UnexpectedAuthoringCapability
        ))
    ));
    assert_eq!(session.snapshot().expect("unchanged snapshot"), before);

    for operation in [
        layout_operation(
            targets.clone(),
            TopLevelRootLayoutTransformModeV1::Scale {
                scale_x: 2.0,
                scale_y: 1.0,
            },
        ),
        layout_operation(
            targets.clone(),
            TopLevelRootLayoutTransformModeV1::MirrorHorizontal,
        ),
        layout_operation(targets.clone(), TopLevelRootLayoutTransformModeV1::AlignTop),
    ] {
        let revision = session.snapshot().expect("current snapshot").revision();
        let mut prepared = session
            .prepare_session_operation_transition_v1(
                crate::SessionOperationTransitionRequestV1::new(
                    revision,
                    operation,
                    TransitionAuthorizationV1::None,
                ),
            )
            .expect("capability-free layout transition prepares");
        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("capability-free layout transition commits");
    }
    assert_eq!(session.snapshot().expect("changed snapshot").revision(), 3);

    assert!(matches!(
        session.prepare_session_operation_transition_v1(
            crate::SessionOperationTransitionRequestV1::new(
                3,
                SessionOperation::V1(SessionOperationV1::TranslateTopLevelRootsV1(
                    TopLevelRootTranslationV1::new(targets, 1.0, 0.0).expect("translation request"),
                )),
                TransitionAuthorizationV1::None,
            )
        ),
        Err(DocumentSessionError::TransitionAuthorization(
            crate::TransitionAuthorizationRefusalV1::AuthoringCapabilityRequired
        ))
    ));
    assert_eq!(
        session
            .snapshot()
            .expect("translation refusal is unchanged")
            .revision(),
        3
    );
}

#[test]
fn rigid_translation_moves_molecule_and_presentation_and_history_restores_geometry() {
    let mut session = DocumentSession::load(MIXED_SOURCE).expect("fixture loads");
    let targets = mixed_targets(&session);
    let result = submit_translation(&mut session, 0, targets, 3.0, -1.0);
    let projection = result.observation().projection();
    assert_authored_close(projection.molecules()[0].atoms()[0].position().x(), 4.0);
    assert_authored_close(projection.molecules()[0].atoms()[0].position().y(), 1.0);
    let PresentationRootProjectionV1::Plus { plus } =
        projection.presentation_stack().entries()[0].root()
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
        .entries()[0]
        .root()
    else {
        panic!("fixture plus remains projected after undo");
    };
    assert_eq!(plus.anchor().x(), 5.0);
}

#[test]
fn renderer_snap_delta_is_canonical_and_rigid_across_mixed_roots() {
    let mut session = DocumentSession::load(MIXED_SOURCE).expect("fixture loads");
    let grid = HexGrid::new(40.0, Point2::new(0.0, 0.0).expect("finite grid origin"))
        .expect("finite grid");
    let targets = mixed_targets(&session);
    let forward = session
        .snap_top_level_translation_for_renderer_v1(0, targets.clone(), 3.0, -1.0, grid)
        .expect("mixed roots have a snapped delta");
    let reversed = session
        .snap_top_level_translation_for_renderer_v1(
            0,
            vec![
                presentation_selector(&session, 0, TopLevelRootKindV1::Plus),
                molecule_selector(&session, 0),
            ],
            3.0,
            -1.0,
            grid,
        )
        .expect("selector order does not change the snapped delta");
    assert_eq!((forward.dx(), forward.dy()), (reversed.dx(), reversed.dy()));
    assert_eq!((forward.dx(), forward.dy()), (-1.0, -2.0));

    let changed = submit_translation(&mut session, 0, targets, forward.dx(), forward.dy());
    let projection = changed.observation().projection();
    let atom = projection.molecules()[0].atoms()[0].position();
    let PresentationRootProjectionV1::Plus { plus } =
        projection.presentation_stack().entries()[0].root()
    else {
        panic!("fixture plus remains projected");
    };
    assert!((plus.anchor().x() - atom.x() - 4.0).abs() <= 2.0 * AUTHORED_HALF_UNIT_POINTS);
    assert!((plus.anchor().y() - atom.y() - 5.0).abs() <= 2.0 * AUTHORED_HALF_UNIT_POINTS);
    let undone = session.undo(1).expect("rigid move is one history entry");
    assert_eq!(
        undone.observation().projection().molecules()[0].atoms()[0]
            .position()
            .x(),
        1.0
    );
}

#[test]
fn alignment_is_semantic_and_a_zero_translation_is_history_free() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><plus id=\"a\"><point x=\"2\" y=\"4\"/></plus>",
        "<plus id=\"b\"><point x=\"8\" y=\"9\"/></plus></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let aligned = session
        .apply_document_operation_v1(
            0,
            operation(
                vec![
                    presentation_selector(&session, 0, TopLevelRootKindV1::Plus),
                    presentation_selector(&session, 1, TopLevelRootKindV1::Plus),
                ],
                TopLevelTransformModeV1::AlignLeft,
            ),
        )
        .expect("alignment succeeds");
    let anchors = aligned
        .observation()
        .projection()
        .presentation_stack()
        .entries()
        .iter()
        .map(|entry| match entry.root() {
            PresentationRootProjectionV1::Plus { plus } => plus.anchor(),
            _ => panic!("fixture contains only plus roots"),
        })
        .collect::<Vec<_>>();
    assert_authored_close(anchors[0].x(), 2.0);
    assert_authored_close(anchors[1].x(), 2.0);

    let target = presentation_selector(&session, 0, TopLevelRootKindV1::Plus);
    let unchanged = submit_translation(&mut session, 1, vec![target], 0.0, 0.0);
    assert_eq!(unchanged.observation().snapshot().revision(), 1);
}

#[test]
fn scale_uses_aggregate_center_and_removes_invalid_generated_linear_forms_while_retaining_authored_forms()
 {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
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
        .apply_document_operation_v1(
            0,
            operation(
                mixed_targets(&session),
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
    let PresentationRootProjectionV1::Plus { plus } =
        projection.presentation_stack().entries()[0].root()
    else {
        panic!("plus remains projected");
    };
    assert_authored_close(plus.anchor().x(), 30.0);
    let cdml = scaled.observation().snapshot().cdml();
    assert!(!cdml.contains("id=\"owned\""));
    assert!(cdml.contains("id=\"richer\""));
    assert!(cdml.contains("retained=\"yes\""));

    let unchanged = session
        .apply_document_operation_v1(
            1,
            operation(
                vec![molecule_selector(&session, 0)],
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
fn mirrors_share_one_pivot_and_preserve_valid_generated_linear_form_metadata() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"5\"/></atom>",
        "<atom id=\"b\" name=\"C\"><point x=\"10\" y=\"5\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/>",
        "<fragment id=\"owned\" type=\"linear_form\"><name>linear_form</name>",
        "<bond id=\"ab\"/><vertex id=\"a\"/><vertex id=\"b\"/>",
        "<property name=\"bond_length\" value=\"10\" type=\"IntType\"/></fragment>",
        "</molecule><plus id=\"p\"><point x=\"20\" y=\"15\"/></plus></cdml>",
    );
    let mut horizontal = DocumentSession::load(source).expect("fixture loads");
    let mirrored = horizontal
        .apply_document_operation_v1(
            0,
            operation(
                mixed_targets(&horizontal),
                TopLevelTransformModeV1::MirrorHorizontal,
            ),
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
        .apply_document_operation_v1(
            0,
            operation(
                mixed_targets(&vertical),
                TopLevelTransformModeV1::MirrorVertical,
            ),
        )
        .expect("vertical mirror succeeds");
    assert_authored_close(
        mirrored.observation().projection().molecules()[0].atoms()[0]
            .position()
            .x(),
        20.0,
    );
    assert!(
        mirrored
            .observation()
            .snapshot()
            .cdml()
            .contains("id=\"owned\"")
    );
}

#[test]
fn malformed_later_root_rejects_the_whole_transform() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><plus id=\"good\"><point x=\"2\" y=\"4\"/></plus>",
        "<text id=\"bad\"><point x=\"8\" y=\"9\"/><point x=\"9\" y=\"9\"/>",
        "<ftext>bad geometry</ftext></text></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let before = session.snapshot().expect("snapshot");
    let error = session
        .apply_document_operation_v1(
            0,
            operation(
                vec![
                    presentation_selector(&session, 0, TopLevelRootKindV1::Plus),
                    presentation_selector(&session, 1, TopLevelRootKindV1::Text),
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
fn renderer_snap_refuses_partial_brackets_without_changing_the_source() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><polyline id=\"left\" bracket_pair=\"left\" bracket_side=\"left\" spline=\"no\">",
        "<point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/>",
        "<point x=\"1\" y=\"2\"/><point x=\"0\" y=\"3\"/></polyline>",
        "<polyline id=\"right\" bracket_pair=\"left\" bracket_side=\"right\" spline=\"no\">",
        "<point x=\"4\" y=\"0\"/><point x=\"3\" y=\"1\"/>",
        "<point x=\"3\" y=\"2\"/><point x=\"4\" y=\"3\"/></polyline></cdml>",
    );
    let session = DocumentSession::load(source).expect("bracket fixture loads");
    let before = session.snapshot().expect("source snapshot");
    let grid = HexGrid::new(40.0, Point2::new(0.0, 0.0).expect("finite grid origin"))
        .expect("finite grid");
    let error = session
        .snap_top_level_translation_for_renderer_v1(
            0,
            vec![presentation_selector(
                &session,
                0,
                TopLevelRootKindV1::Polyline,
            )],
            1.0,
            0.0,
            grid,
        )
        .expect_err("partial bracket does not receive a snapped delta");
    assert!(
        matches!(
            error,
            crate::renderer_admission::RendererTranslationSnapRefusalV1::Selection
        ),
        "unexpected renderer snap refusal: {error:?}"
    );
    assert_eq!(
        session.snapshot().expect("source remains unchanged"),
        before
    );
}

#[test]
fn transform_grammar_rejects_unbounded_or_ambiguous_intent() {
    let one = vec![opaque_selector(1, TopLevelRootKindV1::Plus)];
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
            vec![opaque_selector(2, TopLevelRootKindV1::Plus)],
            TopLevelTransformModeV1::Scale {
                scale_x: 0.0,
                scale_y: 1.0,
            }
        ),
        Err(TopLevelTransformV1Error::InvalidScaleFactors)
    );
}
