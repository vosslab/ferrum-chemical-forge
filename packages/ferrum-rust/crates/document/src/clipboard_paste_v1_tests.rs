use super::{
    DocumentClipboardPasteErrorV1, DocumentSession, DocumentSessionError, TypedClass,
    TypedDocument, TypedDocumentError, XmlBudgetError, XmlInputBudgetV1, XmlInputError,
    prepare_document_clipboard_paste_v1,
};

const FRAGMENT: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\" xmlns:vendor=\"urn:vendor\">",
    "<molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/>",
    "<vendor:extension link=\"a\"/></atom>",
    "<atom id=\"b\" name=\"O\"><point x=\"11\" y=\"2\"/></atom>",
    "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/>",
    "<fragment id=\"f\" type=\"linear_form\"><name>linear_form</name>",
    "<bond id=\"ab\"/><vertex id=\"a\"/><vertex id=\"b\"/>",
    "<property name=\"bond_length\" value=\"10\" type=\"IntType\"/></fragment>",
    "</molecule><plus id=\"p\"><point x=\"31\" y=\"42\"/></plus></cdml>",
);

fn budget() -> XmlInputBudgetV1 {
    XmlInputBudgetV1 {
        max_utf8_bytes: 16 * 1024,
        max_elements: 128,
        max_depth: 16,
        max_attributes: 256,
        max_text_bytes: 1024,
    }
}

#[test]
fn paste_remaps_every_declaration_and_exact_reference_then_translates_once() {
    let plan = prepare_document_clipboard_paste_v1(FRAGMENT, budget())
        .expect("closed copied fragment must prepare");
    assert_eq!(plan.roots().len(), 2);
    assert_eq!(plan.declared_id_count(), 6);
    let mut session = DocumentSession::load(concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"><info><vendor id=\"ferrum-import-v1-0\"/>",
        "</info></cdml>",
    ))
    .expect("target must load");
    let before = session.snapshot().expect("baseline snapshot");
    let result = session
        .paste_document_clipboard_v1(0, before.digest(), &plan, 20.0, 20.0)
        .expect("prepared fragment must paste");
    let observation = result.operation_result().observation();
    assert_eq!(observation.snapshot().revision(), 1);
    assert_eq!(result.pasted_roots().len(), 2);
    assert!(
        result
            .pasted_roots()
            .iter()
            .all(|root| !matches!(root.source_id().as_str(), "m" | "p"))
    );
    let projection = observation.projection();
    let molecule = &projection.molecules()[0];
    assert!((molecule.atoms()[0].position().x() - 21.0).abs() < 0.02);
    assert!((molecule.atoms()[0].position().y() - 22.0).abs() < 0.02);
    assert!((molecule.atoms()[1].position().x() - 31.0).abs() < 0.02);
    let super::PresentationRootProjectionV1::Plus { plus } =
        projection.presentation_stack().entries()[0].root()
    else {
        panic!("second pasted root must be a Plus");
    };
    assert!((plus.anchor().x() - 51.0).abs() < 0.02);
    assert!((plus.anchor().y() - 62.0).abs() < 0.02);

    let document = TypedDocument::parse(observation.snapshot().cdml()).expect("result must parse");
    let pasted_molecule_id = result.pasted_roots()[0].source_id().as_str();
    let pasted_molecule = document
        .root()
        .children_of(TypedClass::Molecule)
        .find(|record| record.attribute("id") == Some(pasted_molecule_id))
        .expect("receipt root must resolve");
    let atom_ids = pasted_molecule
        .children_of(TypedClass::Atom)
        .map(|record| record.attribute("id").expect("pasted atom ID"))
        .collect::<Vec<_>>();
    let bond = pasted_molecule
        .children_of(TypedClass::Bond)
        .next()
        .expect("pasted bond");
    assert_eq!(bond.attribute("start"), Some(atom_ids[0]));
    assert_eq!(bond.attribute("end"), Some(atom_ids[1]));
    assert!(observation.snapshot().cdml().contains("link=\"a\""));
    assert!(!observation.snapshot().cdml().contains("<vertex id=\"a\""));
}

#[test]
fn paste_is_one_history_step_and_repeated_use_allocates_fresh_roots() {
    let plan = prepare_document_clipboard_paste_v1(FRAGMENT, budget()).expect("valid plan");
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let baseline = session.snapshot().expect("baseline");
    let first = session
        .paste_document_clipboard_v1(0, baseline.digest(), &plan, 20.0, 20.0)
        .expect("first Paste");
    let first_ids = first
        .pasted_roots()
        .iter()
        .map(|root| root.source_id().as_str().to_owned())
        .collect::<Vec<_>>();
    let first_object_ids = first
        .pasted_roots()
        .iter()
        .map(|root| root.object_id().clone())
        .collect::<Vec<_>>();
    let first_snapshot = first.operation_result().observation().snapshot().clone();
    let undone = session.undo(1).expect("Paste must undo");
    assert_eq!(undone.observation().snapshot().cdml(), baseline.cdml());
    let redone = session.redo(2).expect("Paste must redo");
    assert_eq!(
        redone.observation().snapshot().cdml(),
        first_snapshot.cdml()
    );
    for object_id in &first_object_ids {
        assert!(
            session
                .current_document_v1()
                .resolve_document_object_id(object_id)
                .expect("redo document identity index must resolve")
                .is_some(),
            "redo must retain every accepted opaque root selector"
        );
    }
    let reopened =
        DocumentSession::load(first_snapshot.cdml()).expect("serialized Paste must reopen");
    for object_id in &first_object_ids {
        assert!(
            reopened
                .current_document_v1()
                .resolve_document_object_id(object_id)
                .expect("reopened document identity index must resolve")
                .is_some(),
            "reopened Paste must retain every accepted opaque root selector"
        );
    }
    let second = session
        .paste_document_clipboard_v1(
            3,
            redone.observation().snapshot().digest(),
            &plan,
            20.0,
            20.0,
        )
        .expect("same plan may be pasted again");
    let second_ids = second
        .pasted_roots()
        .iter()
        .map(|root| root.source_id().as_str().to_owned())
        .collect::<Vec<_>>();
    let second_object_ids = second
        .pasted_roots()
        .iter()
        .map(|root| root.object_id())
        .collect::<Vec<_>>();
    assert!(
        first_ids
            .iter()
            .all(|identifier| !second_ids.contains(identifier))
    );
    assert!(
        first_object_ids
            .iter()
            .all(|object_id| !second_object_ids.contains(&object_id))
    );
    assert_eq!(
        second
            .operation_result()
            .observation()
            .snapshot()
            .revision(),
        4
    );
}

#[test]
fn preparation_and_authentication_fail_without_mutating_the_target() {
    let mut exact = budget();
    exact.max_utf8_bytes = FRAGMENT.len() - 1;
    assert!(matches!(
        prepare_document_clipboard_paste_v1(FRAGMENT, exact),
        Err(DocumentClipboardPasteErrorV1::Typed(
            TypedDocumentError::XmlInput(XmlInputError::Budget(XmlBudgetError::Utf8Bytes { .. }))
        ))
    ));
    assert!(matches!(
        prepare_document_clipboard_paste_v1(
            "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"><paper id=\"paper\"/></cdml>",
            budget(),
        ),
        Err(DocumentClipboardPasteErrorV1::UnsupportedRoot)
    ));
    assert!(matches!(
        prepare_document_clipboard_paste_v1(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\"/><bond id=\"b\" start=\"a\" end=\"outside\"/></molecule></cdml>",
            budget(),
        ),
        Err(DocumentClipboardPasteErrorV1::InvalidReference { field: "end" })
    ));

    let plan = prepare_document_clipboard_paste_v1(FRAGMENT, budget()).expect("valid plan");
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let before = session.snapshot().expect("baseline");
    assert!(matches!(
        session.paste_document_clipboard_v1(0, &[0_u8; 32], &plan, 20.0, 20.0),
        Err(DocumentSessionError::ClipboardPaste(
            DocumentClipboardPasteErrorV1::DigestMismatch
        ))
    ));
    assert_eq!(session.snapshot().expect("unchanged snapshot"), before);
}

#[test]
fn malformed_owned_reference_refuses_without_changing_opaque_foreign_data_or_session() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\" xmlns:vendor=\"urn:vendor\">",
        "<molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<vendor:data link=\"outside\"/><bond id=\"ab\" start=\"a\" end=\"outside\"/>",
        "</molecule></cdml>",
    );
    let session = DocumentSession::load(concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\" xmlns:vendor=\"urn:vendor\">",
        "<info><vendor:data link=\"outside\"/></info></cdml>",
    ))
    .expect("target with opaque foreign data must load");
    let baseline = session.snapshot().expect("baseline snapshot");

    assert!(matches!(
        prepare_document_clipboard_paste_v1(source, budget()),
        Err(DocumentClipboardPasteErrorV1::InvalidReference { field: "end" })
    ));
    assert!(source.contains("link=\"outside\""));
    assert_eq!(session.snapshot().expect("unchanged snapshot"), baseline);
}
