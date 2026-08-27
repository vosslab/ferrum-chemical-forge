use ferrum_geometry::Point2;

use super::{
    DocumentSession, DocumentUserTemplateErrorV1, TypedClass, TypedDocument, XmlInputBudgetV1,
    prepare_document_user_template_v1,
};

const TEMPLATE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" ",
    "xmlns:vendor=\"urn:vendor\" version=\"26.07\">",
    "<standard line_width=\"9\"/>",
    "<paper id=\"template-paper\" type=\"A4\"/>",
    "<molecule id=\"source-molecule\" name=\"  Example molecule  \">",
    "<atom id=\"source-a\" name=\"C\"><point x=\"0\" y=\"2\"/></atom>",
    "<atom id=\"source-b\" name=\"O\"><point x=\"10\" y=\"4\"/></atom>",
    "<bond id=\"source-bond\" start=\"source-a\" end=\"source-b\" type=\"n1\"/>",
    "<vendor:data id=\"source-opaque\" link=\"source-a\"/>",
    "</molecule></cdml>",
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
fn inspection_keeps_application_context_out_of_the_template_plan() {
    let plan = prepare_document_user_template_v1(TEMPLATE, budget())
        .expect("one eligible molecule with optional context must prepare");

    assert_eq!(plan.display_name(), Some("Example molecule"));
    assert_eq!(plan.atom_centroid(), Point2::new(5.0, 3.0).unwrap());
}

#[test]
fn insertion_places_only_the_molecule_with_fresh_ids_as_one_history_step() {
    let plan = prepare_document_user_template_v1(TEMPLATE, budget()).expect("eligible template");
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let baseline = session.snapshot().expect("baseline");
    let result = session
        .insert_document_user_template_v1(
            baseline.revision(),
            baseline.digest(),
            &plan,
            Point2::new(100.0, 50.0).unwrap(),
        )
        .expect("template insertion");
    let inserted = result.operation_result().observation();
    let molecule = inserted
        .projection()
        .molecules()
        .first()
        .expect("inserted molecule projection");
    let atom_a = molecule.atoms()[0].position();
    let atom_b = molecule.atoms()[1].position();
    assert!(((atom_a.x() + atom_b.x()) / 2.0 - 100.0).abs() < 0.02);
    assert!(((atom_a.y() + atom_b.y()) / 2.0 - 50.0).abs() < 0.02);
    assert!((atom_b.x() - atom_a.x() - 10.0).abs() < 0.02);
    assert!((atom_b.y() - atom_a.y() - 2.0).abs() < 0.02);

    let inserted_document =
        TypedDocument::parse(inserted.snapshot().cdml()).expect("inserted CDML must parse");
    assert_eq!(
        inserted_document
            .root()
            .children_of(TypedClass::Molecule)
            .count(),
        1
    );
    assert_eq!(
        inserted_document
            .root()
            .children_of(TypedClass::Paper)
            .count(),
        0
    );
    assert_eq!(
        inserted_document
            .root()
            .children_of(TypedClass::Standard)
            .count(),
        0
    );
    assert_ne!(
        result.inserted_molecule().source_id().as_str(),
        "source-molecule"
    );
    let inserted_molecule = inserted_document
        .root()
        .children_of(TypedClass::Molecule)
        .next()
        .unwrap();
    let atom_ids = inserted_molecule
        .children_of(TypedClass::Atom)
        .map(|atom| atom.attribute("id").unwrap())
        .collect::<Vec<_>>();
    let bond = inserted_molecule
        .children_of(TypedClass::Bond)
        .next()
        .unwrap();
    assert_eq!(bond.attribute("start"), Some(atom_ids[0]));
    assert_eq!(bond.attribute("end"), Some(atom_ids[1]));
    assert!(
        atom_ids
            .iter()
            .all(|identifier| !matches!(*identifier, "source-a" | "source-b"))
    );
    assert!(inserted.snapshot().cdml().contains("id=\"source-opaque\""));
    assert!(inserted.snapshot().cdml().contains("link=\"source-a\""));

    let inserted_object_id = result.inserted_molecule().object_id().clone();
    assert!(
        session
            .current_document_v1()
            .resolve_document_object_id(&inserted_object_id)
            .expect("inserted object ID resolves without an identity failure")
            .is_some(),
        "the insertion receipt must identify the installed molecule"
    );
    let inserted_cdml = inserted.snapshot().cdml().to_owned();
    let undone = session.undo(1).expect("template insertion must undo");
    assert_eq!(undone.observation().snapshot().cdml(), baseline.cdml());
    assert!(
        session
            .current_document_v1()
            .resolve_document_object_id(&inserted_object_id)
            .expect("inserted object ID resolves without an identity failure")
            .is_none(),
        "undo must remove the inserted molecule from the current document"
    );
    let redone = session.redo(2).expect("template insertion must redo");
    assert_eq!(redone.observation().snapshot().cdml(), inserted_cdml);
    assert!(
        session
            .current_document_v1()
            .resolve_document_object_id(&inserted_object_id)
            .expect("inserted object ID resolves without an identity failure")
            .is_some(),
        "redo must restore the receipt's durable object ID"
    );
    let reopened = DocumentSession::load(&inserted_cdml).expect("serialized insertion must reopen");
    assert!(
        reopened
            .current_document_v1()
            .resolve_document_object_id(&inserted_object_id)
            .expect("inserted object ID resolves without an identity failure")
            .is_some(),
        "reopened insertion must retain the receipt's durable object ID"
    );
}

#[test]
fn eligibility_rejects_content_that_cannot_be_a_detached_molecule_template() {
    let cases = [
        (
            "<cdml xmlns=\"urn:ferrum:cdml\"><info/><molecule id=\"m\"><atom id=\"a\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
            "root",
        ),
        (
            concat!(
                "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"first\"><atom id=\"a\"><point x=\"0\" y=\"0\"/></atom></molecule>",
                "<molecule id=\"second\"><atom id=\"b\"><point x=\"1\" y=\"1\"/></atom></molecule></cdml>"
            ),
            "cardinality",
        ),
        (
            concat!(
                "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\"><point x=\"0\" y=\"0\"/></atom>",
                "<template id=\"template\" atom=\"a\"/></molecule></cdml>"
            ),
            "legacy",
        ),
        (
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\"><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></atom></molecule></cdml>",
            "point",
        ),
        (
            concat!(
                "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\"><point x=\"0\" y=\"0\"/></atom>",
                "<bond id=\"bond\" start=\"a\" end=\"outside\"/></molecule></cdml>"
            ),
            "reference",
        ),
    ];

    for (source, expected) in cases {
        let error = prepare_document_user_template_v1(source, budget())
            .expect_err("ineligible template must fail");
        let matched = match error {
            DocumentUserTemplateErrorV1::UnsupportedRoot => "root",
            DocumentUserTemplateErrorV1::MoleculeCardinality => "cardinality",
            DocumentUserTemplateErrorV1::LegacyTemplateMarker => "legacy",
            DocumentUserTemplateErrorV1::MissingAtom => "atom",
            DocumentUserTemplateErrorV1::AtomPointCardinality => "point",
            DocumentUserTemplateErrorV1::ExternalReference { .. } => "reference",
            other => panic!("unexpected eligibility error: {other}"),
        };
        assert_eq!(matched, expected);
    }
}

#[test]
fn missing_source_id_refuses_without_mutating_the_session() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule>",
        "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "</molecule></cdml>",
    );
    let session = DocumentSession::create_empty_document_v1().expect("empty session");
    let baseline = session.snapshot().expect("baseline snapshot");

    let error = prepare_document_user_template_v1(source, budget())
        .expect_err("a template missing a structural source ID must refuse");

    assert!(matches!(error, DocumentUserTemplateErrorV1::Typed(_)));
    assert_eq!(
        session.snapshot().expect("unchanged snapshot").cdml(),
        baseline.cdml()
    );
}
