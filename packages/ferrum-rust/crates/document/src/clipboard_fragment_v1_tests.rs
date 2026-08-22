use super::{
    DocumentClipboardFragmentErrorV1, DocumentClipboardFragmentKindV1,
    DocumentClipboardSelectionV1, DocumentObjectIdV1, DocumentSession, TypedClass, TypedDocument,
    extract_document_clipboard_fragment_v1,
};

const STRUCTURE_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"><molecule id=\"m\" name=\"chain\">",
    "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
    "<vendor:fact xmlns:vendor=\"urn:vendor\" value=\"retained\"/></atom>",
    "<atom id=\"b\" name=\"N\"><point x=\"10\" y=\"0\"/></atom>",
    "<atom id=\"c\" name=\"O\"><point x=\"20\" y=\"0\"/></atom>",
    "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/>",
    "<bond id=\"bc\" start=\"b\" end=\"c\" type=\"n1\"/>",
    "<fragment id=\"linear\" type=\"linear_form\"><name>linear_form</name>",
    "<bond id=\"ab\"/><bond id=\"bc\"/><vertex id=\"a\"/><vertex id=\"b\"/>",
    "<vertex id=\"c\"/><property name=\"bond_length\" value=\"10\" type=\"IntType\"/>",
    "</fragment></molecule></cdml>",
);

const MIXED_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"><plus id=\"p\"><point x=\"30\" y=\"40\"/></plus>",
    "<molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/>",
    "</atom><atom id=\"b\" name=\"O\"><point x=\"3\" y=\"4\"/></atom>",
    "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/></molecule></cdml>",
);

fn durable_atom(
    observation: &super::SessionDocumentObservationV1,
    index: usize,
) -> DocumentObjectIdV1 {
    observation.projection().molecules()[0].atoms()[index]
        .id()
        .expect("fixture atom must have durable identity")
        .clone()
}

fn durable_bond(
    observation: &super::SessionDocumentObservationV1,
    index: usize,
) -> DocumentObjectIdV1 {
    observation.projection().molecules()[0].bonds()[index]
        .id()
        .expect("fixture bond must have durable identity")
        .clone()
}

#[test]
fn selected_bond_closes_endpoints_and_omits_generated_metadata() {
    let session = DocumentSession::load(STRUCTURE_SOURCE).expect("fixture must load");
    let observation = session.observe(0).expect("fixture must project");
    let source_snapshot = observation.snapshot().clone();
    let bond = durable_bond(&observation, 1);
    let result = extract_document_clipboard_fragment_v1(
        &observation,
        DocumentClipboardSelectionV1::new(vec![bond.clone()]).expect("selection must validate"),
    )
    .expect("selected bond must copy");

    assert_eq!(result.kind(), DocumentClipboardFragmentKindV1::Structure);
    assert_eq!(result.selected_objects(), std::slice::from_ref(&bond));
    assert_eq!(result.copied_bonds(), &[bond]);
    assert_eq!(
        result.copied_atoms(),
        &[durable_atom(&observation, 1), durable_atom(&observation, 2)]
    );
    assert_eq!(result.source_revision(), 0);
    assert_eq!(result.source_digest(), observation.snapshot().digest());
    assert_eq!(observation.snapshot(), &source_snapshot);
    assert!(!result.fragment_cdml().contains("id=\"a\""));
    assert!(!result.fragment_cdml().contains("id=\"ab\""));
    assert!(!result.fragment_cdml().contains("linear_form"));
    let fragment = TypedDocument::parse(result.fragment_cdml()).expect("fragment must reparse");
    let molecule = fragment
        .root()
        .children_of(TypedClass::Molecule)
        .next()
        .expect("fragment must retain its molecule");
    assert_eq!(molecule.children_of(TypedClass::Atom).count(), 2);
    assert_eq!(molecule.children_of(TypedClass::Bond).count(), 1);
}

#[test]
fn disconnected_structure_is_rejected_without_changing_the_observation() {
    let session = DocumentSession::load(STRUCTURE_SOURCE).expect("fixture must load");
    let observation = session.observe(0).expect("fixture must project");
    let source_snapshot = observation.snapshot().clone();
    let selection = DocumentClipboardSelectionV1::new(vec![
        durable_atom(&observation, 0),
        durable_atom(&observation, 2),
    ])
    .expect("selection must validate");

    assert!(matches!(
        extract_document_clipboard_fragment_v1(&observation, selection),
        Err(DocumentClipboardFragmentErrorV1::DisconnectedStructure)
    ));
    assert_eq!(observation.snapshot(), &source_snapshot);
}

#[test]
fn mixed_selection_copies_complete_roots_in_document_order() {
    let session = DocumentSession::load(MIXED_SOURCE).expect("fixture must load");
    let observation = session.observe(0).expect("fixture must project");
    let plus = observation.projection().presentation_stack().roots()[0]
        .target()
        .id()
        .expect("fixture plus must have durable identity")
        .clone();
    let atom = durable_atom(&observation, 0);
    let molecule = observation.projection().molecules()[0]
        .id()
        .expect("fixture molecule must have durable identity")
        .clone();
    let result = extract_document_clipboard_fragment_v1(
        &observation,
        DocumentClipboardSelectionV1::new(vec![atom.clone(), plus.clone()])
            .expect("selection must validate"),
    )
    .expect("mixed selection must copy complete roots");

    assert_eq!(result.kind(), DocumentClipboardFragmentKindV1::TopLevel);
    assert_eq!(result.selected_objects(), &[plus.clone(), atom]);
    assert_eq!(result.copied_roots(), &[plus, molecule]);
    let fragment = TypedDocument::parse(result.fragment_cdml()).expect("fragment must reparse");
    let classes = fragment
        .root()
        .typed_children()
        .iter()
        .map(|child| child.record().class())
        .collect::<Vec<_>>();
    assert_eq!(classes, vec![TypedClass::CanvasPlus, TypedClass::Molecule]);
    assert_eq!(
        fragment
            .root()
            .children_of(TypedClass::Molecule)
            .next()
            .expect("whole molecule must remain")
            .children_of(TypedClass::Atom)
            .count(),
        2
    );
}
