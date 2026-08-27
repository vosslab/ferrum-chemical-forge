use ferrum_document_projection::DocumentLocationKindV1;

use super::{
    DocumentObjectIdV1, DocumentSession, PersistentId, Point3V1, SessionOperation,
    SessionOperationV1, TypedClass, TypedDocument, TypedDocumentError,
    projection_identity_v1::projection_document_object_id_from_record_v1,
};

const DOCUMENT_OBJECT_NAMESPACE_V1: &str = "urn:ferrum:document-object:v1";

fn molecule_and_atom_ids(document: &TypedDocument) -> (DocumentObjectIdV1, DocumentObjectIdV1) {
    let molecule = document
        .root()
        .children_of(TypedClass::Molecule)
        .next()
        .expect("fixture has one molecule");
    let atom = molecule
        .children_of(TypedClass::Atom)
        .next()
        .expect("fixture has one atom");
    (
        projection_document_object_id_from_record_v1(molecule)
            .expect("ingress assigns molecule identity"),
        projection_document_object_id_from_record_v1(atom).expect("ingress assigns atom identity"),
    )
}

#[test]
fn typed_ingress_requires_source_ids_with_source_free_locations() {
    let missing = TypedDocument::parse(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .expect_err("an addressable root requires a source id");
    assert!(matches!(
        missing,
        TypedDocumentError::MissingStructuralSourceId { ref location }
            if location.kind() == DocumentLocationKindV1::Structural
                && location.root_ordinal() == 0
                && location.child_path().is_empty()
    ));

    let blank = TypedDocument::parse(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\" \"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .expect_err("a blank addressable source id is refused");
    assert!(matches!(
        blank,
        TypedDocumentError::InvalidStructuralSourceId { ref location }
            if location.kind() == DocumentLocationKindV1::Structural
                && location.root_ordinal() == 0
                && location.child_path().is_empty()
    ));

    let duplicate = TypedDocument::parse(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"same\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><arrow id=\"same\"><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></arrow></cdml>",
    )
    .expect_err("duplicate addressable source ids are refused");
    assert!(matches!(
        duplicate,
        TypedDocumentError::DuplicateStructuralSourceId { ref first, ref duplicate }
            if first.kind() == DocumentLocationKindV1::Structural
                && first.root_ordinal() == 0
                && duplicate.kind() == DocumentLocationKindV1::Presentation
                && duplicate.root_ordinal() == 1
    ));
}

#[test]
fn persisted_document_object_metadata_is_validated_and_unique() {
    let malformed = TypedDocument::parse(&format!(
        "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:f=\"{DOCUMENT_OBJECT_NAMESPACE_V1}\"><molecule id=\"m\" f:id=\"not-an-object-id\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>"
    ))
    .expect_err("malformed persisted metadata is refused");
    assert!(matches!(
        malformed,
        TypedDocumentError::InvalidPersistedDocumentObjectId { ref location }
            if location.kind() == DocumentLocationKindV1::Structural
                && location.root_ordinal() == 0
    ));

    let duplicate_id = "ferrum-document-object-v1/00112233445566778899aabbccddeeff";
    let duplicate = TypedDocument::parse(&format!(
        "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:f=\"{DOCUMENT_OBJECT_NAMESPACE_V1}\"><molecule id=\"m\" f:id=\"{duplicate_id}\"><atom id=\"a\" name=\"C\" f:id=\"{duplicate_id}\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>"
    ))
    .expect_err("duplicate persisted metadata is refused");
    assert!(matches!(
        duplicate,
        TypedDocumentError::DuplicatePersistedDocumentObjectId { ref first, ref duplicate }
            if first.kind() == DocumentLocationKindV1::Structural
                && first.root_ordinal() == 0
                && first.child_path().is_empty()
                && duplicate.child_path() == [0]
    ));
}

#[test]
fn allocated_document_object_metadata_is_opaque_distinct_and_persistent() {
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule-source\"><atom id=\"atom-source\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";
    let document = TypedDocument::parse(source).expect("valid source is normalized");
    let (molecule, atom) = molecule_and_atom_ids(&document);
    assert_ne!(molecule, atom);
    assert!(!molecule.as_str().contains("molecule-source"));
    assert!(!atom.as_str().contains("atom-source"));
    assert_eq!(
        document
            .resolve_document_object_id(&molecule)
            .expect("persisted molecule identity resolves through the identity index")
            .expect("persisted molecule identity is present in the identity index")
            .class(),
        TypedClass::Molecule
    );
    assert_eq!(
        document
            .resolve_document_object_id(&atom)
            .expect("persisted atom identity resolves through the identity index")
            .expect("persisted atom identity is present in the identity index")
            .class(),
        TypedClass::Atom
    );

    let serialized = document.to_xml().expect("normalized document serializes");
    assert!(serialized.contains(DOCUMENT_OBJECT_NAMESPACE_V1));
    let reopened = TypedDocument::parse(&serialized).expect("normalized document reopens");
    assert_eq!(molecule_and_atom_ids(&reopened), (molecule, atom));
}

#[test]
fn document_object_identity_index_returns_none_for_an_absent_valid_selector() {
    let document = TypedDocument::parse(concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:f=\"urn:ferrum:document-object:v1\">",
        "<molecule id=\"molecule-source\" ",
        "f:id=\"ferrum-document-object-v1/00000000000000000000000000000001\">",
        "<atom id=\"atom-source\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "</molecule></cdml>",
    ))
    .expect("document with identifier-free root and point records is valid");
    let indexed =
        DocumentObjectIdV1::parse("ferrum-document-object-v1/00000000000000000000000000000001")
            .expect("indexed selector is syntactically valid");
    let absent =
        DocumentObjectIdV1::parse("ferrum-document-object-v1/00000000000000000000000000000002")
            .expect("absent selector is syntactically valid");

    assert_eq!(
        document
            .resolve_document_object_id(&indexed)
            .expect("indexed selector resolution succeeds")
            .expect("indexed selector resolves")
            .class(),
        TypedClass::Molecule
    );
    assert!(
        document
            .resolve_document_object_id(&absent)
            .expect("absent selector resolution succeeds")
            .is_none()
    );
}

#[test]
fn authored_records_and_history_retain_persisted_document_object_ids() {
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";
    let document = TypedDocument::parse(source).expect("valid source is normalized");
    let (_, initial_atom) = molecule_and_atom_ids(&document);
    let authored = document
        .with_insert_atom(
            &PersistentId::new("m").expect("molecule source identifier"),
            &PersistentId::new("new-atom").expect("new atom source identifier"),
            "O",
            Point3V1::new(1.0, 0.0, 0.0).expect("finite authored position"),
        )
        .expect("typed authoring reparses through durable identity ingress");
    let authored_atom = authored
        .document_object_id_for_source_id_v1(
            &PersistentId::new("new-atom").expect("new atom source identifier"),
        )
        .expect("new authored atom identity lookup succeeds")
        .expect("new authored atom has a persisted identity");
    assert_ne!(authored_atom, initial_atom);
    assert!(!authored_atom.as_str().contains("new-atom"));

    let mut session = DocumentSession::load(source).expect("source loads into a session");
    let baseline = TypedDocument::parse(session.snapshot().expect("baseline snapshot").cdml())
        .expect("baseline snapshot reopens");
    let baseline_ids = molecule_and_atom_ids(&baseline);
    let changed = session
        .apply_document_operation_v1(
            0,
            SessionOperation::V1(SessionOperationV1::SetAtomElement {
                atom_id: "a".to_owned(),
                element: "N".to_owned(),
            }),
        )
        .expect("typed edit commits");
    let changed_revision = changed.observation().snapshot().revision();
    let changed_document = TypedDocument::parse(changed.observation().snapshot().cdml())
        .expect("changed snapshot reopens");
    assert_eq!(molecule_and_atom_ids(&changed_document), baseline_ids);

    let undone = session.undo(changed_revision).expect("undo commits");
    let undone_revision = undone.observation().snapshot().revision();
    let undone_document = TypedDocument::parse(undone.observation().snapshot().cdml())
        .expect("undone snapshot reopens");
    assert_eq!(molecule_and_atom_ids(&undone_document), baseline_ids);

    let redone = session.redo(undone_revision).expect("redo commits");
    let reopened = TypedDocument::parse(redone.observation().snapshot().cdml())
        .expect("redone snapshot reopens");
    assert_eq!(molecule_and_atom_ids(&reopened), baseline_ids);
}
