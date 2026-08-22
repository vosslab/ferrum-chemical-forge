use xot::{Node, Xot};

use super::{
    DocumentSession, DocumentSessionError, INTERCHANGE_IMPORT_NAMESPACE_V1,
    InterchangePropertyInsertionV1, InterchangeRecordBatchInsertionV1,
    InterchangeRecordInsertionV1, InterchangeRecordInsertionV1Error, MoleculeInsertionAtomV1,
    MoleculeInsertionV1, Point3V1, element_name,
};

fn molecule(element: &str, x: f64) -> MoleculeInsertionV1 {
    MoleculeInsertionV1::new(
        vec![
            MoleculeInsertionAtomV1::new(
                element,
                Point3V1::new(x, 2.0, 0.0).expect("test position is finite"),
                None,
                None,
                None,
            )
            .expect("test atom is valid"),
        ],
        Vec::new(),
    )
    .expect("test molecule is valid")
}

fn record(
    element: &str,
    x: f64,
    title: &str,
    properties: &[(&str, &str)],
) -> InterchangeRecordInsertionV1 {
    let properties = properties
        .iter()
        .map(|(name, value)| {
            InterchangePropertyInsertionV1::new(*name, *value).expect("test property is valid")
        })
        .collect();
    InterchangeRecordInsertionV1::new(molecule(element, x), title, properties)
        .expect("test record is valid")
}

#[test]
fn interchange_batch_commits_every_record_and_exact_ordered_metadata_once() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\"><opaque id=\"keep\">",
        "<foreign payload=\"retained\"/></opaque></cdml>"
    );
    let batch = InterchangeRecordBatchInsertionV1::new(vec![
        record(
            "C",
            1.0,
            "Alpha",
            &[("NOTE", "first\nline"), ("NOTE", "second")],
        ),
        record("N", 5.0, "", &[("EMPTY", "")]),
    ])
    .expect("batch is nonempty");
    let mut session = DocumentSession::load(source).expect("fixture must load");
    let baseline = session.snapshot().expect("baseline must snapshot");

    let mut pending = session
        .prepare_create_interchange_records_v1(0, &batch)
        .expect("complete batch must prepare");
    assert_eq!(
        session.snapshot().expect("prepare must not mutate"),
        baseline
    );
    assert_eq!(
        pending
            .molecule_identifiers()
            .iter()
            .map(|identifier| identifier.as_str())
            .collect::<Vec<_>>(),
        ["ferrum-molecule-v1-0", "ferrum-molecule-v1-1"]
    );
    assert_eq!(
        pending.atom_identifiers()[0][0].as_str(),
        "ferrum-atom-v1-0"
    );
    assert_eq!(
        pending.atom_identifiers()[1][0].as_str(),
        "ferrum-atom-v1-1"
    );

    let accepted = session
        .commit_create_interchange_records_v1(0, &mut pending)
        .expect("prepared batch must commit");
    let observation = accepted.observation();
    assert_eq!(observation.snapshot().revision(), 1);
    assert_eq!(observation.projection().molecules().len(), 2);
    assert!(
        observation
            .snapshot()
            .cdml()
            .contains("payload=\"retained\"")
    );
    assert_interchange_metadata(observation.snapshot().cdml());

    let undone = session.undo(1).expect("batch must be one undo step");
    assert!(undone.observation().projection().molecules().is_empty());
    let redone = session.redo(2).expect("batch must be one redo step");
    assert_eq!(redone.observation().projection().molecules().len(), 2);
    assert_interchange_metadata(redone.observation().snapshot().cdml());
}

#[test]
fn interchange_batch_grammar_and_stale_revision_fail_without_mutation() {
    assert_eq!(
        InterchangeRecordBatchInsertionV1::new(Vec::new()),
        Err(InterchangeRecordInsertionV1Error::EmptyBatch)
    );
    assert_eq!(
        InterchangePropertyInsertionV1::new("bad\nname", "value"),
        Err(InterchangeRecordInsertionV1Error::InvalidPropertyName)
    );

    let batch = InterchangeRecordBatchInsertionV1::new(vec![record("O", 1.0, "Oxygen", &[])])
        .expect("batch is nonempty");
    let mut session =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("fixture must load");
    let mut first = session
        .prepare_create_interchange_records_v1(0, &batch)
        .expect("initial batch must prepare");
    session
        .commit_create_interchange_records_v1(0, &mut first)
        .expect("initial batch must commit");
    let before = session.snapshot().expect("accepted state must snapshot");
    assert!(matches!(
        session.prepare_create_interchange_records_v1(0, &batch),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(
        session.snapshot().expect("rejection must not mutate"),
        before
    );
}

#[test]
fn refused_and_dropped_interchange_preparation_preserves_ids_and_tokens() {
    let batch = InterchangeRecordBatchInsertionV1::new(vec![record("O", 1.0, "Oxygen", &[])])
        .expect("batch is nonempty");
    let mut session =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("session loads");
    let mut control =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("control session loads");
    let token_before = session.provisional_token_facts_for_test();

    let refused = session
        .prepare_create_interchange_records_v1(0, &batch)
        .expect("valid candidate prepares before an external response refusal");
    assert_eq!(session.provisional_token_facts_for_test(), token_before);
    drop(refused);
    assert_eq!(session.provisional_token_facts_for_test(), token_before);

    let mut accepted = session
        .prepare_create_interchange_records_v1(0, &batch)
        .expect("equivalent candidate prepares after refusal");
    let mut control_pending = control
        .prepare_create_interchange_records_v1(0, &batch)
        .expect("fresh control candidate prepares");
    assert_eq!(
        accepted.molecule_identifiers(),
        control_pending.molecule_identifiers()
    );
    assert_eq!(
        accepted.atom_identifiers(),
        control_pending.atom_identifiers()
    );
    assert_eq!(
        accepted.bond_identifiers(),
        control_pending.bond_identifiers()
    );

    session
        .commit_create_interchange_records_v1(0, &mut accepted)
        .expect("accepted candidate commits");
    control
        .commit_create_interchange_records_v1(0, &mut control_pending)
        .expect("control candidate commits");
    assert_eq!(
        session.provisional_token_facts_for_test(),
        control.provisional_token_facts_for_test()
    );
}

fn assert_interchange_metadata(source: &str) {
    let mut tree = Xot::new();
    let document = tree.parse(source).expect("accepted XML must parse");
    let root = tree
        .document_element(document)
        .expect("document has a root");
    let molecules = tree
        .children(root)
        .filter(|node| element_name(&tree, *node).is_some_and(|(local, _)| local == "molecule"))
        .collect::<Vec<_>>();
    assert_eq!(molecules.len(), 2);
    let name = tree.add_name("name");
    assert_eq!(tree.get_attribute(molecules[0], name), Some("Alpha"));
    assert_eq!(tree.get_attribute(molecules[1], name), None);
    assert_record(
        &mut tree,
        molecules[0],
        "416c706861",
        &[
            ("4e4f5445", "66697273740a6c696e65"),
            ("4e4f5445", "7365636f6e64"),
        ],
    );
    assert_record(&mut tree, molecules[1], "", &[("454d505459", "")]);
}

fn assert_record(tree: &mut Xot, molecule: Node, title: &str, expected: &[(&str, &str)]) {
    let record = tree
        .children(molecule)
        .find(|node| {
            element_name(tree, *node).is_some_and(|(local, namespace)| {
                local == "interchange-record" && namespace == INTERCHANGE_IMPORT_NAMESPACE_V1
            })
        })
        .expect("molecule retains one interchange metadata record");
    let encoding = tree.add_name("encoding");
    let title_name = tree.add_name("title");
    let property_name = tree.add_name("name");
    let property_value = tree.add_name("value");
    assert_eq!(tree.get_attribute(record, encoding), Some("utf8-hex-v1"));
    assert_eq!(tree.get_attribute(record, title_name), Some(title));
    let properties = tree
        .children(record)
        .filter(|node| {
            element_name(tree, *node).is_some_and(|(local, namespace)| {
                local == "property" && namespace == INTERCHANGE_IMPORT_NAMESPACE_V1
            })
        })
        .map(|node| {
            (
                tree.get_attribute(node, property_name)
                    .expect("property has a name"),
                tree.get_attribute(node, property_value)
                    .expect("property has a value"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(properties, expected);
}
