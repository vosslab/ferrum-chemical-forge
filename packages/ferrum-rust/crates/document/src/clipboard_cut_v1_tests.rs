use super::{
    DocumentClipboardCutErrorV1, DocumentClipboardSelectionV1, DocumentObjectIdV1, DocumentSession,
    prepare_document_clipboard_cut_v1,
};

const SOURCE: &str = concat!(
    "<cdml version=\"26.07\"><plus id=\"p\"><point x=\"30\" y=\"40\"/></plus>",
    "<molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
    "</atom><atom id=\"b\" name=\"N\"><point x=\"10\" y=\"0\"/></atom>",
    "<atom id=\"c\" name=\"O\"><point x=\"20\" y=\"0\"/></atom>",
    "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/>",
    "<bond id=\"bc\" start=\"b\" end=\"c\" type=\"n1\"/>",
    "<fragment id=\"linear\" type=\"linear_form\"><name>linear_form</name>",
    "<bond id=\"ab\"/><bond id=\"bc\"/><vertex id=\"a\"/><vertex id=\"b\"/>",
    "<vertex id=\"c\"/><property name=\"bond_length\" value=\"10\" type=\"IntType\"/>",
    "</fragment></molecule></cdml>",
);

fn atom(observation: &super::SessionDocumentObservationV1, index: usize) -> DocumentObjectIdV1 {
    observation.projection().molecules()[0].atoms()[index]
        .id()
        .expect("fixture atom must have durable identity")
        .clone()
}

#[test]
fn structural_cut_is_one_reversible_topology_edit() {
    let mut session = DocumentSession::load(SOURCE).expect("fixture must load");
    let observation = session.observe(0).expect("fixture must project");
    let before = observation.snapshot().clone();
    let plan = prepare_document_clipboard_cut_v1(
        &observation,
        DocumentClipboardSelectionV1::new(vec![atom(&observation, 1)])
            .expect("selection must validate"),
    )
    .expect("middle atom must prepare for Cut");
    let result = session
        .cut_document_clipboard_v1(0, before.digest(), &plan)
        .expect("prepared Cut must commit");
    let changed = result.observation();
    let molecule = &changed.projection().molecules()[0];
    let remaining = molecule
        .atoms()
        .iter()
        .map(|atom| atom.source_id().expect("fixture source ID"))
        .collect::<Vec<_>>();

    assert_eq!(
        (
            changed.snapshot().revision(),
            remaining,
            molecule.bonds().len(),
            changed.snapshot().cdml().contains("linear_form"),
        ),
        (1, vec!["a", "c"], 0, false),
    );
    let restored = session.undo(1).expect("Cut must undo");
    let restored = restored.observation();
    let molecule = &restored.projection().molecules()[0];
    assert_eq!(
        (
            restored.snapshot().revision(),
            molecule.atoms().len(),
            molecule.bonds().len(),
            restored.snapshot().cdml().contains("linear_form"),
        ),
        (2, 3, 2, true),
    );
}

#[test]
fn mixed_complete_root_copy_fallback_is_not_a_cut_deletion() {
    let session = DocumentSession::load(SOURCE).expect("fixture must load");
    let observation = session.observe(0).expect("fixture must project");
    let plus = observation.projection().presentation_stack().roots()[0]
        .target()
        .id()
        .expect("fixture plus must have durable identity")
        .clone();
    let selection = DocumentClipboardSelectionV1::new(vec![plus, atom(&observation, 0)])
        .expect("selection must validate");

    assert!(matches!(
        prepare_document_clipboard_cut_v1(&observation, selection),
        Err(DocumentClipboardCutErrorV1::UnsupportedTopLevelSelection)
    ));
}
