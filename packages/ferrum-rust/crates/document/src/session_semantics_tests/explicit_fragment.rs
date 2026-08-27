use super::super::session::PendingCreateExplicitFragmentV1;
use super::super::{
    DocumentExplicitFragmentErrorV1, DocumentObjectIdV1, DocumentSession, DocumentSessionError,
    PersistentId, SessionOperationError, TypedDocument, observe_explicit_fragments_v1,
};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
    "</atom><atom id=\"b\" name=\"O\"><point x=\"10\" y=\"0\"/></atom>",
    "<atom id=\"c\" name=\"N\"><point x=\"20\" y=\"0\"/></atom>",
    "<bond id=\"bc\" type=\"n1\" start=\"b\" end=\"c\"/>",
    "<bond id=\"ab\" type=\"n1\" start=\"a\" end=\"b\"/>",
    "<vendor id=\"ferrum-fragment-v1-0\"/></molecule></cdml>",
);

const RETAINED_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
    "</atom><fragment id=\"supported\" type=\"explicit\"><name>Known</name>",
    "<vertex id=\"a\"/></fragment><fragment id=\"retained\" type=\"implicit\">",
    "<name>Legacy</name><vertex id=\"a\"/></fragment></molecule></cdml>",
);

fn molecule(session: &DocumentSession, revision: u64) -> DocumentObjectIdV1 {
    session
        .observe(revision)
        .expect("fixture projects")
        .projection()
        .molecules()[0]
        .document_object_id()
        .clone()
}

fn ids(values: &[&str]) -> Vec<PersistentId> {
    values
        .iter()
        .map(|value| PersistentId::new((*value).to_owned()).expect("fixture ID is valid"))
        .collect()
}

fn prepare(session: &mut DocumentSession, revision: u64) -> PendingCreateExplicitFragmentV1 {
    session
        .prepare_create_explicit_fragment_v1(
            revision,
            &molecule(session, revision),
            "  useful label  ",
            &ids(&["c"]),
            &ids(&["ab"]),
        )
        .expect("valid fragment prepares")
}

#[test]
fn explicit_fragment_closes_bonds_in_source_order_and_avoids_retained_identity() {
    let mut session = DocumentSession::load(SOURCE).expect("source loads");
    let mut prepared = prepare(&mut session, 0);
    let receipt = prepared.record().clone();
    assert_eq!(receipt.name(), "useful label");
    assert_eq!(receipt.bond_ids(), ids(&["ab"]));
    assert_eq!(receipt.atom_ids(), ids(&["a", "b", "c"]));
    assert_ne!(receipt.fragment_id().as_str(), "ferrum-fragment-v1-0");

    let accepted = session
        .commit_create_explicit_fragment_v1(0, &mut prepared)
        .expect("prepared fragment commits");
    let observed = observe_explicit_fragments_v1(
        &TypedDocument::parse(accepted.observation().snapshot().cdml())
            .expect("committed CDML reopens"),
    )
    .expect("committed explicit fragment observation succeeds");
    assert!(observed.records().iter().any(|record| {
        record.fragment_id() == receipt.fragment_id()
            && record.name() == "useful label"
            && record.bond_ids() == ids(&["ab"])
            && record.atom_ids() == ids(&["a", "b", "c"])
    }));
}

#[test]
fn explicit_fragment_refuses_foreign_members_without_mutation() {
    let mut session = DocumentSession::load(SOURCE).expect("source loads");
    let before = session.snapshot().expect("baseline snapshot");
    assert!(matches!(
        session.prepare_create_explicit_fragment_v1(
            0,
            &molecule(&session, 0),
            "label",
            &ids(&["missing"]),
            &[],
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::ExplicitFragment(
                DocumentExplicitFragmentErrorV1::InvalidMember(_)
            )
        ))
    ));
    assert_eq!(session.snapshot().expect("refusal is inert"), before);
}

#[test]
fn explicit_fragment_receipt_is_one_use_and_history_reopens_its_semantics() {
    let mut session = DocumentSession::load(SOURCE).expect("source loads");
    let mut stale = prepare(&mut session, 0);
    let mut accepted = prepare(&mut session, 0);
    let committed = session
        .commit_create_explicit_fragment_v1(0, &mut accepted)
        .expect("current receipt commits");
    let saved = committed.observation().snapshot().cdml().to_owned();
    let after_commit = session.snapshot().expect("committed snapshot");
    assert!(matches!(
        session.commit_create_explicit_fragment_v1(1, &mut stale),
        Err(DocumentSessionError::RevisionConflict { .. })
    ));
    assert_eq!(
        session.snapshot().expect("stale receipt is inert"),
        after_commit
    );
    assert!(matches!(
        session.commit_create_explicit_fragment_v1(1, &mut accepted),
        Err(DocumentSessionError::PreparedOperationConsumed)
    ));
    let undone = session.undo(1).expect("creation is undoable");
    let redone = session.redo(2).expect("creation is redoable");
    let reopened = DocumentSession::load(&saved).expect("saved CDML reopens");
    let undone_observation = observe_explicit_fragments_v1(
        &TypedDocument::parse(undone.observation().snapshot().cdml()).expect("undo CDML parses"),
    )
    .expect("undo explicit fragment observation succeeds");
    assert!(undone_observation.records().is_empty());
    let redone_observation = observe_explicit_fragments_v1(
        &TypedDocument::parse(redone.observation().snapshot().cdml()).expect("redo CDML parses"),
    )
    .expect("redo explicit fragment observation succeeds");
    let reopened_observation = observe_explicit_fragments_v1(
        &TypedDocument::parse(
            reopened
                .observe(0)
                .expect("reopened observation")
                .snapshot()
                .cdml(),
        )
        .expect("reopened CDML parses"),
    )
    .expect("reopened explicit fragment observation succeeds");
    assert_eq!(redone_observation, reopened_observation,);
}

#[test]
fn explicit_fragment_observation_exposes_supported_records_and_retained_notice() {
    let document = TypedDocument::parse(RETAINED_SOURCE).expect("retained source loads");
    let observation = observe_explicit_fragments_v1(&document)
        .expect("retained explicit fragment observation succeeds");
    assert!(observation.has_retained_fragment_metadata());
    assert!(observation.records().iter().any(|record| {
        record.name() == "Known" && record.bond_ids().is_empty() && record.atom_ids() == ids(&["a"])
    }));
}
