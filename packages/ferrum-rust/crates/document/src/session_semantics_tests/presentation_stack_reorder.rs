use super::{
    DocumentObjectIdV1, DocumentSession, DocumentSessionError, SessionOperation, SessionOperationV1,
};
use crate::{
    PresentationRecordKindV1, PresentationRootSelectorV1, PresentationStackOrderV1,
    PresentationStackReorderV1,
};
use ferrum_document_projection::DocumentDirectRootKindV1;

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\"><!--header--><info/><molecule id=\"m\"><atom id=\"m-atom\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
    "<arrow id=\"a\"><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></arrow>",
    "<v:opaque retained=\"yes\"/><text id=\"t\"><point x=\"2\" y=\"2\"/>",
    "<ftext>note</ftext></text><plus id=\"p\"><point x=\"3\" y=\"3\"/></plus>",
    "<!--tail--></cdml>",
);

fn reorder(
    order: PresentationStackOrderV1,
    targets: Vec<PresentationRootSelectorV1>,
) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::ReorderPresentationRoots {
        reorder: PresentationStackReorderV1::new(order, targets).expect("valid reorder intent"),
    })
}

fn presentation_order(session: &DocumentSession, revision: u64) -> Vec<PresentationRootSelectorV1> {
    let observation = session.observe(revision).expect("fixture observation");
    observation
        .projection()
        .presentation_stack()
        .entries()
        .iter()
        .map(|entry| {
            let target = entry.root().target();
            PresentationRootSelectorV1::new(
                target.document_object_id().clone(),
                target.record_kind(),
            )
        })
        .collect()
}

fn direct_root_order(session: &DocumentSession, revision: u64) -> Vec<DocumentObjectIdV1> {
    session
        .observe(revision)
        .expect("fixture observation")
        .projection()
        .direct_roots()
        .iter()
        .map(|root| root.document_object_id().clone())
        .collect()
}

#[test]
fn presentation_stack_modes_preserve_slots_content_and_history() {
    let mut session = DocumentSession::load(SOURCE).expect("fixture loads");
    let baseline = session.snapshot().expect("baseline");
    let [arrow, text, plus] = presentation_order(&session, 0)
        .try_into()
        .expect("fixture has three ordered presentation roots");
    let brought = session
        .apply_document_operation_v1(
            0,
            reorder(
                PresentationStackOrderV1::BringToFront,
                vec![arrow.clone(), text.clone()],
            ),
        )
        .expect("bring succeeds");
    assert_eq!(
        presentation_order(&session, 1),
        [plus.clone(), arrow.clone(), text.clone()]
    );
    let observation = session.observe(1).expect("reordered observation");
    let molecule = observation
        .projection()
        .direct_roots()
        .iter()
        .find(|root| matches!(root.kind(), DocumentDirectRootKindV1::Molecule))
        .expect("fixture has one durable molecule direct root")
        .document_object_id()
        .clone();
    assert_eq!(
        direct_root_order(&session, 1),
        vec![
            molecule,
            plus.document_object_id().clone(),
            arrow.document_object_id().clone(),
            text.document_object_id().clone(),
        ]
    );
    assert!(
        brought
            .observation()
            .snapshot()
            .cdml()
            .contains("retained=\"yes\"")
    );
    let reversed = session
        .apply_document_operation_v1(
            1,
            reorder(
                PresentationStackOrderV1::ReverseSelectedSlots,
                vec![arrow.clone(), text.clone()],
            ),
        )
        .expect("reverse succeeds");
    assert_eq!(
        presentation_order(&session, 2),
        [plus.clone(), text.clone(), arrow.clone()]
    );
    assert_eq!(reversed.observation().snapshot().revision(), 2);
    session.undo(2).expect("undo succeeds");
    assert_eq!(presentation_order(&session, 3), [plus, arrow, text]);
    session.undo(3).expect("second undo succeeds");
    assert_eq!(
        session.snapshot().expect("restored").cdml(),
        baseline.cdml()
    );
}

#[test]
fn invalid_stale_partial_bracket_and_noop_reorders_are_atomic() {
    let mut session = DocumentSession::load(SOURCE).expect("fixture loads");
    let [arrow, text, plus] = presentation_order(&session, 0)
        .try_into()
        .expect("fixture has three ordered presentation roots");
    assert!(
        PresentationStackReorderV1::new(PresentationStackOrderV1::BringToFront, Vec::new(),)
            .is_err()
    );
    assert!(
        PresentationStackReorderV1::new(
            PresentationStackOrderV1::SendToBack,
            vec![arrow.clone(), arrow.clone()],
        )
        .is_err()
    );
    assert!(
        PresentationStackReorderV1::new(
            PresentationStackOrderV1::ReverseSelectedSlots,
            vec![arrow.clone()],
        )
        .is_err()
    );
    let before = session.snapshot().expect("baseline");
    let wrong_kind = reorder(
        PresentationStackOrderV1::BringToFront,
        vec![PresentationRootSelectorV1::new(
            arrow.document_object_id().clone(),
            PresentationRecordKindV1::Text,
        )],
    );
    assert!(matches!(
        session.apply_document_operation_v1(0, wrong_kind),
        Err(DocumentSessionError::Operation(_))
    ));
    assert_eq!(session.snapshot().expect("unchanged"), before);

    let no_change = session
        .apply_document_operation_v1(
            0,
            reorder(PresentationStackOrderV1::BringToFront, vec![plus.clone()]),
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
    let [left, right, bracket_plus] = presentation_order(&bracket, 0)
        .try_into()
        .expect("bracket fixture has three ordered presentation roots");
    let partial = reorder(PresentationStackOrderV1::BringToFront, vec![left.clone()]);
    assert!(matches!(
        bracket.apply_document_operation_v1(0, partial),
        Err(DocumentSessionError::Operation(_))
    ));
    assert_eq!(
        bracket.snapshot().expect("bracket unchanged"),
        bracket_before
    );
    bracket
        .apply_document_operation_v1(
            0,
            reorder(
                PresentationStackOrderV1::BringToFront,
                vec![left.clone(), right.clone()],
            ),
        )
        .expect("complete bracket pair reorders together");
    assert_eq!(presentation_order(&bracket, 1), [bracket_plus, left, right]);

    let changed = session
        .apply_document_operation_v1(
            0,
            reorder(PresentationStackOrderV1::SendToBack, vec![arrow.clone()]),
        )
        .expect("change succeeds");
    let after = session.snapshot().expect("changed snapshot");
    assert!(matches!(
        session.apply_document_operation_v1(
            0,
            reorder(PresentationStackOrderV1::SendToBack, vec![text],),
        ),
        Err(DocumentSessionError::RevisionConflict { .. })
    ));
    assert_eq!(session.snapshot().expect("stale unchanged"), after);
    assert_eq!(changed.observation().snapshot().revision(), 1);
}
