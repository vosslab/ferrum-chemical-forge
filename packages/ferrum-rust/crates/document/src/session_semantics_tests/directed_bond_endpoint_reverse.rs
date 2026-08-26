//! Durable directed-wedge endpoint reversal behavior.

use super::{
    DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError,
    SessionOperationV1, TypedDocumentError,
};
use crate::{CDML_NAMESPACE, ReverseDirectedBondEndpointsV1, element_name};
use xot::Xot;

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\"><molecule id=\"m\">",
    "<atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/></atom>",
    "<atom id=\"b\" name=\"O\"><point x=\"3\" y=\"2\"/></atom>",
    "<atom id=\"c\" name=\"N\"><point x=\"5\" y=\"2\"/></atom>",
    "<bond id=\"wedge\" start=\"a\" end=\"b\" type=\"w1\" color=\"#123456\" v:keep=\"yes\"><v:opaque/></bond>",
    "<bond id=\"hashed\" start=\"b\" end=\"c\" type=\"h1\" line_width=\"2\"/>",
    "<bond id=\"normal\" start=\"a\" end=\"c\" type=\"n1\"/>",
    "<bond id=\"opaque\" start=\"c\" end=\"a\" type=\"x1\"/>",
    "</molecule><v:root_keep/></cdml>"
);

fn reverse(bond_id: &str) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::ReverseDirectedBondEndpointsV1(
        ReverseDirectedBondEndpointsV1::new(bond_id).expect("fixture bond ID is valid"),
    ))
}

fn bond_facts(
    session: &DocumentSession,
    revision: u64,
    index: usize,
) -> (String, String, String, String, String) {
    let observation = session
        .observe(revision)
        .expect("fixture observation must project");
    let bond = &observation.projection().molecules()[0].bonds()[index];
    (
        bond.document_object_id().as_str().to_owned(),
        bond.source_id()
            .expect("fixture bond ID is durable")
            .to_owned(),
        bond.source_type()
            .expect("fixture bond type is present")
            .to_owned(),
        bond.start()
            .source_id()
            .expect("fixture endpoint is present")
            .to_owned(),
        bond.end()
            .source_id()
            .expect("fixture endpoint is present")
            .to_owned(),
    )
}

fn assert_wedge_extension_is_retained(cdml: &str) {
    let mut tree = Xot::new();
    let document = tree.parse(cdml).expect("accepted CDML must parse");
    let root = tree.document_element(document).expect("CDML has a root");
    let id = tree.add_name("id");
    let molecule = tree
        .children(root)
        .find(|node| {
            element_name(&tree, *node).is_some_and(|(local, namespace)| {
                local == "molecule" && namespace == CDML_NAMESPACE
            })
        })
        .expect("fixture molecule must remain present");
    let wedge = tree
        .children(molecule)
        .find(|node| {
            element_name(&tree, *node)
                .is_some_and(|(local, namespace)| local == "bond" && namespace == CDML_NAMESPACE)
                && tree
                    .get_attribute(*node, id)
                    .is_some_and(|id| id == "wedge")
        })
        .expect("reversed wedge must remain present");
    let vendor_namespace = tree.add_namespace("urn:vendor");
    let keep = tree.add_name_ns("keep", vendor_namespace);
    assert_eq!(tree.get_attribute(wedge, keep), Some("yes"));
    assert!(tree.children(wedge).any(|node| {
        element_name(&tree, node)
            .is_some_and(|(local, namespace)| local == "opaque" && namespace == "urn:vendor")
    }));
}

#[test]
fn directed_wedge_reversal_preserves_identity_and_history_for_w1_and_h1() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let wedge_before = bond_facts(&session, 0, 0);
    let hashed_before = bond_facts(&session, 0, 1);

    let wedge = session
        .apply_document_operation_v1(0, reverse("wedge"))
        .expect("wedge reversal must commit");
    assert_eq!(wedge.observation().snapshot().revision(), 1);
    assert_eq!(
        bond_facts(&session, 1, 0),
        (
            wedge_before.0.clone(),
            wedge_before.1.clone(),
            wedge_before.2.clone(),
            "b".to_owned(),
            "a".to_owned()
        )
    );
    assert_wedge_extension_is_retained(wedge.observation().snapshot().cdml());
    assert_eq!(bond_facts(&session, 1, 1), hashed_before);

    let hashed = session
        .apply_document_operation_v1(1, reverse("hashed"))
        .expect("hashed wedge reversal must commit");
    assert_eq!(hashed.observation().snapshot().revision(), 2);
    assert_eq!(
        bond_facts(&session, 2, 0),
        (
            wedge_before.0.clone(),
            wedge_before.1.clone(),
            wedge_before.2.clone(),
            "b".to_owned(),
            "a".to_owned()
        )
    );
    assert_eq!(
        bond_facts(&session, 2, 1),
        (
            hashed_before.0.clone(),
            hashed_before.1.clone(),
            hashed_before.2.clone(),
            "c".to_owned(),
            "b".to_owned()
        )
    );

    let undone = session.undo(2).expect("one reversal must undo once");
    assert_eq!(undone.observation().snapshot().revision(), 3);
    assert_eq!(bond_facts(&session, 3, 1), hashed_before);
    let redone = session.redo(3).expect("one reversal must redo once");
    assert_eq!(redone.observation().snapshot().revision(), 4);
    assert_eq!(
        bond_facts(&session, 4, 1),
        (
            hashed_before.0,
            hashed_before.1,
            hashed_before.2,
            "c".to_owned(),
            "b".to_owned()
        )
    );
    let reopened = DocumentSession::load(redone.observation().snapshot().cdml())
        .expect("accepted CDML must reopen");
    assert_eq!(bond_facts(&reopened, 0, 0).1, "wedge");
    assert_eq!(bond_facts(&reopened, 0, 0).2, "w1");
    assert_eq!(bond_facts(&reopened, 0, 0).3, "b");
    assert_eq!(bond_facts(&reopened, 0, 0).4, "a");
    assert_eq!(bond_facts(&reopened, 0, 1).1, "hashed");
    assert_eq!(bond_facts(&reopened, 0, 1).2, "h1");
    assert_eq!(bond_facts(&reopened, 0, 1).3, "c");
    assert_eq!(bond_facts(&reopened, 0, 1).4, "b");
}

#[test]
fn directed_wedge_reversal_refusals_are_atomic() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot must serialize");
    for bond_id in ["missing", "normal", "opaque"] {
        let result = session.apply_document_operation_v1(0, reverse(bond_id));
        match bond_id {
            "missing" => assert!(matches!(
                result,
                Err(DocumentSessionError::Operation(
                    SessionOperationError::UnknownBond(_)
                ))
            )),
            _ => assert!(matches!(
                result,
                Err(DocumentSessionError::Operation(
                    SessionOperationError::Candidate(
                        TypedDocumentError::UnsupportedDirectedBondEndpointReversal(_)
                    )
                ))
            )),
        }
        assert_eq!(session.snapshot().expect("snapshot must serialize"), before);
    }
    assert!(matches!(
        ReverseDirectedBondEndpointsV1::new(""),
        Err(crate::ReverseDirectedBondEndpointsV1Error::InvalidSourceBondId)
    ));
}

#[test]
fn directed_wedge_invalid_endpoints_and_stale_revisions_are_atomic() {
    let sources = [
        SOURCE.replace("start=\"a\" end=\"b\"", "start=\"a\" end=\"a\""),
        SOURCE.replace("start=\"a\" end=\"b\"", "start=\"a\""),
        SOURCE.replace("start=\"a\" end=\"b\"", "start=\"a\" end=\"\""),
        SOURCE
            .replace(
                "</molecule><v:root_keep/>",
                concat!(
                    "</molecule><molecule id=\"foreign\"><atom id=\"foreign_atom\" name=\"C\">",
                    "<point x=\"7\" y=\"2\"/></atom></molecule><v:root_keep/>"
                ),
            )
            .replace("start=\"a\" end=\"b\"", "start=\"a\" end=\"foreign_atom\""),
    ];
    for source in sources {
        let mut session = DocumentSession::load(&source).expect("retained source must load");
        let before = session.snapshot().expect("snapshot must serialize");
        assert!(matches!(
            session.apply_document_operation_v1(0, reverse("wedge")),
            Err(DocumentSessionError::Operation(
                SessionOperationError::Candidate(TypedDocumentError::InvalidBondEndpoint(_))
            ))
        ));
        let after = session.snapshot().expect("snapshot must serialize");
        assert_eq!(after.revision(), before.revision());
        assert_eq!(after.digest(), before.digest());
        assert_eq!(after, before);
    }

    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    session
        .apply_document_operation_v1(0, reverse("wedge"))
        .expect("first reversal must commit");
    let before = session.snapshot().expect("snapshot must serialize");
    assert!(matches!(
        session.apply_document_operation_v1(0, reverse("hashed")),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(session.snapshot().expect("snapshot must serialize"), before);
}
