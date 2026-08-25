use xot::{Node, Xot};

use super::{
    DocumentSession, INTERCHANGE_IMPORT_NAMESPACE_V1, InterchangePropertyInsertionV1,
    InterchangeRecordBatchInsertionV1, InterchangeRecordInsertionV1,
    InterchangeRecordInsertionV1Error, MoleculeInsertionAtomV1, MoleculeInsertionV1, Point3V1,
    SessionOperation, SessionOperationOutcomeV1, SessionOperationTransitionRequestV1,
    SessionOperationV1, TransitionAuthorizationV1, element_name,
};

fn molecule(element: &str, x: f64) -> MoleculeInsertionV1 {
    MoleculeInsertionV1::new(
        vec![
            MoleculeInsertionAtomV1::new(
                element,
                Point3V1::new(x, 2.0, 0.0).expect("finite point"),
                None,
                None,
                None,
            )
            .expect("valid atom"),
        ],
        Vec::new(),
    )
    .expect("valid molecule")
}

fn record(
    element: &str,
    x: f64,
    title: &str,
    properties: &[(&str, &str)],
) -> InterchangeRecordInsertionV1 {
    InterchangeRecordInsertionV1::new(
        molecule(element, x).into(),
        title,
        properties
            .iter()
            .map(|(name, value)| {
                InterchangePropertyInsertionV1::new(*name, *value).expect("valid property")
            })
            .collect(),
    )
    .expect("valid record")
}

fn request(
    revision: u64,
    batch: InterchangeRecordBatchInsertionV1,
) -> SessionOperationTransitionRequestV1 {
    SessionOperationTransitionRequestV1::new(
        revision,
        SessionOperation::V1(SessionOperationV1::InsertInterchangeRecordBatchV1(batch)),
        TransitionAuthorizationV1::None,
    )
}

#[test]
fn generic_interchange_batch_commits_source_ordered_ids_and_lossless_metadata_once() {
    let batch = InterchangeRecordBatchInsertionV1::new(vec![
        record(
            "C",
            1.0,
            "Alpha",
            &[("NOTE", "first\nline"), ("NOTE", "second")],
        ),
        record("N", 5.0, "", &[("EMPTY", "")]),
    ])
    .expect("nonempty batch");
    let mut session = DocumentSession::create_empty_document_v1().expect("session creates");
    let mut prepared = session
        .prepare_session_operation_transition_v1(request(0, batch))
        .expect("batch prepares");
    let accepted = session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("batch commits");
    let SessionOperationOutcomeV1::InterchangeRecordBatchInsertedV1(outcome) = accepted.outcome()
    else {
        panic!("commit publishes batch outcome");
    };
    assert_eq!(outcome.records().len(), 2);
    assert_eq!(
        outcome.records()[0].molecule_identifier().as_str(),
        "ferrum-molecule-v1-0"
    );
    assert_eq!(
        outcome.records()[1].molecule_identifier().as_str(),
        "ferrum-molecule-v1-1"
    );
    assert_eq!(
        outcome.records()[0].atom_identifiers()[0].as_str(),
        "ferrum-atom-v1-0"
    );
    assert_eq!(
        outcome.records()[1].atom_identifiers()[0].as_str(),
        "ferrum-atom-v1-1"
    );
    assert_eq!(accepted.observation().snapshot().revision(), 1);
    assert_interchange_metadata(accepted.observation().snapshot().cdml());
    assert!(
        session
            .undo(1)
            .expect("batch is one history entry")
            .observation()
            .projection()
            .molecules()
            .is_empty()
    );
}

#[test]
fn generic_interchange_batch_rejects_invalid_and_stale_requests_without_mutation() {
    assert_eq!(
        InterchangeRecordBatchInsertionV1::new(Vec::new()),
        Err(InterchangeRecordInsertionV1Error::EmptyBatch)
    );
    let batch = InterchangeRecordBatchInsertionV1::new(vec![record("O", 1.0, "Oxygen", &[])])
        .expect("nonempty batch");
    let mut session = DocumentSession::create_empty_document_v1().expect("session creates");
    let mut first = session
        .prepare_session_operation_transition_v1(request(0, batch.clone()))
        .expect("first batch prepares");
    session
        .commit_session_operation_transition_v1(&mut first)
        .expect("first batch commits");
    let before = session.snapshot().expect("committed snapshot");
    assert!(
        session
            .prepare_session_operation_transition_v1(request(0, batch))
            .is_err()
    );
    assert_eq!(session.snapshot().expect("stale request is inert"), before);
}

fn assert_interchange_metadata(source: &str) {
    let mut tree = Xot::new();
    let document = tree.parse(source).expect("accepted XML parses");
    let root = tree.document_element(document).expect("document root");
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
        .expect("metadata record");
    let encoding = tree.add_name("encoding");
    let title_name = tree.add_name("title");
    let property_name = tree.add_name("name");
    let property_value = tree.add_name("value");
    assert_eq!(tree.get_attribute(record, encoding), Some("utf8-hex-v1"));
    assert_eq!(tree.get_attribute(record, title_name), Some(title));
    let actual = tree
        .children(record)
        .filter(|node| {
            element_name(tree, *node).is_some_and(|(local, namespace)| {
                local == "property" && namespace == INTERCHANGE_IMPORT_NAMESPACE_V1
            })
        })
        .map(|node| {
            (
                tree.get_attribute(node, property_name)
                    .expect("property name"),
                tree.get_attribute(node, property_value)
                    .expect("property value"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}
