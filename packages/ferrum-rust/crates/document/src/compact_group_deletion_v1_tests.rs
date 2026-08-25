use super::{
    DocumentObjectIdV1, DocumentSession, DocumentSessionError, SessionOperationError,
    TypedDocument, TypedDocumentError,
};

const ATTACHED_GROUP: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\">",
    "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
    "<compact-group id=\"g\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group>",
    "<bond id=\"b\" start=\"a\" end=\"g\" type=\"n1\"/>",
    "</molecule></cdml>",
);

fn compact_group_address(session: &DocumentSession) -> (DocumentObjectIdV1, DocumentObjectIdV1) {
    let observation = session.observe(0).expect("observation is available");
    let molecule = &observation.projection().molecules()[0];
    (
        molecule.id().expect("renderer-issued molecule ID").clone(),
        molecule.compact_groups()[0]
            .id()
            .clone(),
    )
}

#[test]
fn compact_group_deletion_removes_exact_group_and_bond_then_reopens() {
    let mut session = DocumentSession::load(ATTACHED_GROUP).expect("fixture loads");
    let (molecule_object_id, compact_group_object_id) = compact_group_address(&session);
    let mut pending = session
        .prepare_delete_compact_group_v1(0, &molecule_object_id, &compact_group_object_id)
        .expect("compact deletion prepares");
    assert_eq!(pending.receipt().molecule_id().as_str(), "m");
    assert_eq!(pending.receipt().compact_group_id().as_str(), "g");
    assert_eq!(pending.receipt().exterior_bond_id().as_str(), "b");
    let committed = session
        .commit_delete_compact_group_v1(0, &mut pending)
        .expect("compact deletion commits");
    let cdml = committed.observation().snapshot().cdml();
    assert!(cdml.contains("id=\"a\""));
    assert!(!cdml.contains("compact-group"));
    assert!(!cdml.contains("id=\"b\""));
    TypedDocument::parse(cdml).expect("committed candidate reparses as typed CDML");
    assert!(matches!(
        session.commit_delete_compact_group_v1(1, &mut pending),
        Err(DocumentSessionError::PreparedOperationConsumed)
    ));
    let undone = session.undo(1).expect("undo restores the group");
    assert!(undone.observation().snapshot().cdml().contains("compact-group"));
    let redone = session.redo(2).expect("redo removes the group again");
    assert!(!redone.observation().snapshot().cdml().contains("compact-group"));
}

#[test]
fn compact_group_deletion_refuses_zero_or_multiple_exterior_bonds_atomically() {
    let no_bond = ATTACHED_GROUP.replace("<bond id=\"b\" start=\"a\" end=\"g\" type=\"n1\"/>", "");
    let two_bonds = ATTACHED_GROUP.replace(
        "<bond id=\"b\" start=\"a\" end=\"g\" type=\"n1\"/>",
        concat!(
            "<atom id=\"c\" name=\"C\"><point x=\"40\" y=\"0\"/></atom>",
            "<bond id=\"b\" start=\"a\" end=\"g\" type=\"n1\"/>",
            "<bond id=\"c-g\" start=\"c\" end=\"g\" type=\"n1\"/>"
        ),
    );
    for source in [no_bond, two_bonds] {
        let mut session = DocumentSession::load(&source).expect("fixture loads");
        let (molecule_object_id, compact_group_object_id) = compact_group_address(&session);
        let before = session.snapshot().expect("snapshot works");
        assert!(matches!(
            session.prepare_delete_compact_group_v1(0, &molecule_object_id, &compact_group_object_id),
            Err(DocumentSessionError::Operation(SessionOperationError::Candidate(
                TypedDocumentError::InvalidCompactGroupDeletionTopology(_)
            )))
        ));
        assert_eq!(session.snapshot().expect("snapshot remains unchanged"), before);
    }
}

#[test]
fn compact_group_deletion_refuses_a_foreign_durable_group_without_mutation() {
    let source = ATTACHED_GROUP.replace(
        "</molecule></cdml>",
        concat!(
            "</molecule><molecule id=\"other\">",
            "<atom id=\"other-atom\" name=\"C\"><point x=\"40\" y=\"0\"/></atom>",
            "<compact-group id=\"other-group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"60\" y=\"0\"/></compact-group>",
            "<bond id=\"other-bond\" start=\"other-atom\" end=\"other-group\" type=\"n1\"/>",
            "</molecule></cdml>"
        ),
    );
    let mut session = DocumentSession::load(&source).expect("fixture loads");
    let observation = session.observe(0).expect("observation is available");
    let molecule_object_id = observation.projection().molecules()[0]
        .id()
        .expect("renderer-issued molecule ID")
        .clone();
    let compact_group_object_id = observation.projection().molecules()[1].compact_groups()[0]
        .id()
        .clone();
    let before = session.snapshot().expect("snapshot works");
    assert!(matches!(
        session.prepare_delete_compact_group_v1(0, &molecule_object_id, &compact_group_object_id),
        Err(DocumentSessionError::Operation(
            SessionOperationError::InvalidLiveChemicalTarget(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot remains unchanged"), before);
}

#[test]
fn compact_group_deletion_refuses_raw_source_ids_as_durable_selectors() {
    assert!(DocumentObjectIdV1::parse("m").is_err());
    assert!(DocumentObjectIdV1::parse("g").is_err());
}
