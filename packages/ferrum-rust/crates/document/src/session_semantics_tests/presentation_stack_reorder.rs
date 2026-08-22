use super::{DocumentSession, DocumentSessionError, SessionOperation, SessionOperationV1};
use crate::{
    PresentationRecordKindV1, PresentationRootSelectorV1, PresentationStackOrderV1,
    PresentationStackReorderV1,
};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\"><!--header--><info/><molecule id=\"m\"/>",
    "<arrow id=\"a\"><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></arrow>",
    "<v:opaque retained=\"yes\"/><text id=\"t\"><point x=\"2\" y=\"2\"/>",
    "<ftext>note</ftext></text><plus id=\"p\"><point x=\"3\" y=\"3\"/></plus>",
    "<!--tail--></cdml>",
);

fn target(id: &str, kind: PresentationRecordKindV1) -> PresentationRootSelectorV1 {
    PresentationRootSelectorV1::new(id, kind).expect("valid test selector")
}

fn reorder(
    order: PresentationStackOrderV1,
    targets: Vec<PresentationRootSelectorV1>,
) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::ReorderPresentationRoots {
        reorder: PresentationStackReorderV1::new(order, targets).expect("valid reorder intent"),
    })
}

fn presentation_order(session: &DocumentSession, revision: u64) -> Vec<String> {
    session
        .observe(revision)
        .expect("fixture observation")
        .projection()
        .presentation_stack()
        .roots()
        .iter()
        .map(|root| {
            root.target()
                .source_id()
                .expect("durable fixture root")
                .to_owned()
        })
        .collect()
}

#[test]
fn presentation_stack_modes_preserve_slots_content_and_history() {
    let mut session = DocumentSession::load(SOURCE).expect("fixture loads");
    let baseline = session.snapshot().expect("baseline");
    let brought = session
        .submit(
            0,
            reorder(
                PresentationStackOrderV1::BringToFront,
                vec![
                    target("a", PresentationRecordKindV1::Arrow),
                    target("t", PresentationRecordKindV1::Text),
                ],
            ),
        )
        .expect("bring succeeds");
    assert_eq!(presentation_order(&session, 1), ["p", "a", "t"]);
    let observation = session.observe(1).expect("reordered observation");
    assert_eq!(observation.projection().molecules()[0].source_order(), 2);
    assert_eq!(
        observation
            .projection()
            .presentation_stack()
            .roots()
            .iter()
            .map(|root| root.target().source_order())
            .collect::<Vec<_>>(),
        [4, 5, 6],
    );
    assert!(
        brought
            .observation()
            .snapshot()
            .cdml()
            .contains("retained=\"yes\"")
    );
    let reversed = session
        .submit(
            1,
            reorder(
                PresentationStackOrderV1::ReverseSelectedSlots,
                vec![
                    target("a", PresentationRecordKindV1::Arrow),
                    target("t", PresentationRecordKindV1::Text),
                ],
            ),
        )
        .expect("reverse succeeds");
    assert_eq!(presentation_order(&session, 2), ["p", "t", "a"]);
    assert_eq!(reversed.observation().snapshot().revision(), 2);
    session.undo(2).expect("undo succeeds");
    assert_eq!(presentation_order(&session, 3), ["p", "a", "t"]);
    session.undo(3).expect("second undo succeeds");
    assert_eq!(
        session.snapshot().expect("restored").cdml(),
        baseline.cdml()
    );
}

#[test]
fn invalid_stale_partial_bracket_and_noop_reorders_are_atomic() {
    assert!(
        PresentationStackReorderV1::new(PresentationStackOrderV1::BringToFront, Vec::new(),)
            .is_err()
    );
    assert!(
        PresentationStackReorderV1::new(
            PresentationStackOrderV1::SendToBack,
            vec![
                target("a", PresentationRecordKindV1::Arrow),
                target("a", PresentationRecordKindV1::Arrow),
            ],
        )
        .is_err()
    );
    assert!(
        PresentationStackReorderV1::new(
            PresentationStackOrderV1::ReverseSelectedSlots,
            vec![target("a", PresentationRecordKindV1::Arrow)],
        )
        .is_err()
    );
    let mut session = DocumentSession::load(SOURCE).expect("fixture loads");
    let before = session.snapshot().expect("baseline");
    let wrong_kind = reorder(
        PresentationStackOrderV1::BringToFront,
        vec![target("a", PresentationRecordKindV1::Text)],
    );
    assert!(matches!(
        session.submit(0, wrong_kind),
        Err(DocumentSessionError::Operation(_))
    ));
    assert_eq!(session.snapshot().expect("unchanged"), before);

    let no_change = session
        .submit(
            0,
            reorder(
                PresentationStackOrderV1::BringToFront,
                vec![target("p", PresentationRecordKindV1::Plus)],
            ),
        )
        .expect("already-front intent is accepted");
    assert_eq!(no_change.observation().snapshot().revision(), 0);

    let bracket_source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><polyline id=\"left\" bracket_pair=\"left\" bracket_side=\"left\" spline=\"no\">",
        "<point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/><point x=\"1\" y=\"2\"/>",
        "<point x=\"0\" y=\"3\"/></polyline><polyline id=\"right\" bracket_pair=\"left\" ",
        "bracket_side=\"right\" spline=\"no\"><point x=\"4\" y=\"0\"/>",
        "<point x=\"3\" y=\"1\"/><point x=\"3\" y=\"2\"/>",
        "<point x=\"4\" y=\"3\"/></polyline><plus id=\"p\"><point x=\"5\" y=\"5\"/>",
        "</plus></cdml>",
    );
    let mut bracket = DocumentSession::load(bracket_source).expect("bracket fixture loads");
    let bracket_before = bracket.snapshot().expect("bracket baseline");
    let partial = reorder(
        PresentationStackOrderV1::BringToFront,
        vec![target("left", PresentationRecordKindV1::Polyline)],
    );
    assert!(matches!(
        bracket.submit(0, partial),
        Err(DocumentSessionError::Operation(_))
    ));
    assert_eq!(
        bracket.snapshot().expect("bracket unchanged"),
        bracket_before
    );
    bracket
        .submit(
            0,
            reorder(
                PresentationStackOrderV1::BringToFront,
                vec![
                    target("left", PresentationRecordKindV1::Polyline),
                    target("right", PresentationRecordKindV1::Polyline),
                ],
            ),
        )
        .expect("complete bracket pair reorders together");
    assert_eq!(presentation_order(&bracket, 1), ["p", "left", "right"]);

    let changed = session
        .submit(
            0,
            reorder(
                PresentationStackOrderV1::SendToBack,
                vec![target("a", PresentationRecordKindV1::Arrow)],
            ),
        )
        .expect("change succeeds");
    let after = session.snapshot().expect("changed snapshot");
    assert!(matches!(
        session.submit(
            0,
            reorder(
                PresentationStackOrderV1::SendToBack,
                vec![target("t", PresentationRecordKindV1::Text)],
            ),
        ),
        Err(DocumentSessionError::RevisionConflict { .. })
    ));
    assert_eq!(session.snapshot().expect("stale unchanged"), after);
    assert_eq!(changed.observation().snapshot().revision(), 1);
}
