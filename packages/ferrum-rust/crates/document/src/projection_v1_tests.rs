use super::{
    BondEndpointKindV1, DOCUMENT_PROJECTION_SCHEMA_V1, DocumentHaworthPositionV1,
    DocumentObjectIdV1, DocumentProjectionV1, DocumentSession, PersistentId, ProjectionError,
    ProjectionIssueCodeV1, TypedClass, TypedDocument,
};
use serde_json::Value;

const DOCUMENT: &str = concat!(
    r##"<c:cdml xmlns:c="urn:ferrum:cdml" version="26.07">
<c:standard line_width="1.5" font_size="12" font_family="Fira Sans" "##,
    r##"line_color="#AABBCC"><c:bond width="6" wedge-width="5px" double-ratio="0.6"/>"##,
    r##"<c:atom show_hydrogens="yes"/></c:standard>
"##,
    r##"<c:molecule id="m1" name="first"><c:atom id="a1" name="C" charge="-1" explicit_hydrogens="2" show="no" hydrogens="yes"><c:point x="1cm" y="2cm"/><c:font family="Arial" size="13" color="#112233"/><c:ftext> C label </c:ftext></c:atom><c:group id="g1"/><c:atom id="a2" name="O"><c:point x="3" y="4"/></c:atom><c:bond id="b1" start="a1" end="a2" type="n1" line_width="2" bond_width="-8px" color="#445566"/><c:bond id="b2" start="a1" end="g1" type="n1"/></c:molecule>
<c:molecule id="m2"><c:atom id="a3" name="N"><c:point x="0" y="0"/></c:atom></c:molecule>
</c:cdml>"##,
);

fn projected(source: &str) -> DocumentProjectionV1 {
    let session = DocumentSession::load(source).unwrap();
    let snapshot = session.snapshot().unwrap();
    let document = TypedDocument::parse(snapshot.cdml()).unwrap();
    DocumentProjectionV1::from_snapshot(&document, &snapshot).unwrap()
}

#[test]
fn projection_has_stable_wire_ids_order_positions_and_presentation_after_reparse() {
    let original = TypedDocument::parse(DOCUMENT).unwrap();
    let reparsed = TypedDocument::parse(&original.to_xml().unwrap()).unwrap();
    let session = DocumentSession::load(DOCUMENT).unwrap();
    let snapshot = session.snapshot().unwrap();
    let first = DocumentProjectionV1::from_snapshot(&original, &snapshot).unwrap();
    let second = DocumentProjectionV1::from_snapshot(&reparsed, &snapshot).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.schema(), DOCUMENT_PROJECTION_SCHEMA_V1);
    assert_eq!(first.revision(), 0);
    assert_eq!(first.digest(), snapshot.digest());
    assert!(!first.is_dirty());
    assert_eq!(
        first
            .molecules()
            .iter()
            .map(|molecule| molecule.source_id())
            .collect::<Vec<_>>(),
        vec![Some("m1"), Some("m2")]
    );
    let molecule = &first.molecules()[0];
    assert_eq!(molecule.source_order(), 3);
    assert_eq!(
        molecule
            .atoms()
            .iter()
            .map(|atom| atom.source_order())
            .collect::<Vec<_>>(),
        vec![0, 2]
    );
    assert_eq!(
        molecule
            .bonds()
            .iter()
            .map(|bond| bond.source_order())
            .collect::<Vec<_>>(),
        vec![3, 4]
    );
    assert_eq!(molecule.atoms()[0].position().x(), 72.0 / 2.54);
    assert_eq!(
        molecule.atoms()[0]
            .label_font()
            .unwrap()
            .color()
            .unwrap()
            .as_str(),
        "#112233"
    );
    assert_eq!(
        molecule.atoms()[0].label_text().unwrap().as_str(),
        " C label "
    );
    assert_eq!(
        first
            .drawing_standard()
            .unwrap()
            .line_color()
            .unwrap()
            .as_str(),
        "#aabbcc"
    );
    assert_eq!(
        first
            .drawing_standard()
            .unwrap()
            .bond_width()
            .unwrap()
            .value(),
        6.0
    );
    assert_eq!(
        first
            .drawing_standard()
            .unwrap()
            .wedge_width()
            .unwrap()
            .value(),
        5.0
    );
    assert_eq!(
        first
            .drawing_standard()
            .unwrap()
            .double_ratio()
            .unwrap()
            .value(),
        0.6
    );
    assert_eq!(molecule.bonds()[0].bond_width().unwrap().value(), -8.0);
    assert!(
        molecule.atoms()[0]
            .id()
            .unwrap()
            .as_str()
            .starts_with("ferrum-document-object-v1/")
    );
    assert!(
        molecule.atoms()[0]
            .id()
            .unwrap()
            .as_str()
            .contains("/source/")
    );
}

#[test]
fn alternate_prefixes_do_not_change_projection() {
    let alternate = DOCUMENT.replace("c:", "z:").replace("xmlns:c", "xmlns:z");
    let session = DocumentSession::load(DOCUMENT).unwrap();
    let snapshot = session.snapshot().unwrap();
    let original = TypedDocument::parse(DOCUMENT).unwrap();
    let alternate = TypedDocument::parse(&alternate).unwrap();
    assert_eq!(
        DocumentProjectionV1::from_snapshot(&original, &snapshot).unwrap(),
        DocumentProjectionV1::from_snapshot(&alternate, &snapshot).unwrap()
    );
}

#[test]
fn absent_presentation_facts_remain_absent() {
    let document = TypedDocument::parse(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule><atom name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .unwrap();
    let projection = projected(&document.to_xml().unwrap());
    let atom = &projection.molecules()[0].atoms()[0];
    assert!(projection.drawing_standard().is_none());
    assert!(atom.source_id().is_none());
    assert!(atom.label_font().is_none());
    assert!(atom.label_text().is_none());
    assert!(atom.show().is_none());
    assert!(atom.hydrogens().is_none());
    assert!(atom.id().is_none());
    assert!(
        atom.projection_key()
            .as_str()
            .starts_with("ferrum-projection-local-v1/")
    );
}

#[test]
fn haworth_depth_facts_project_and_survive_typed_round_trip() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"b\" name=\"O\"><point x=\"1\" y=\"0\"/></atom>",
        "<bond start=\"a\" end=\"b\" haworth_position=\"front\"/>",
        "<bond start=\"b\" end=\"a\" haworth_position=\"back\"/></molecule></cdml>",
    );
    let original = projected(source);
    let retained = TypedDocument::parse(source).unwrap();
    let reparsed = projected(&retained.to_xml().unwrap());

    assert_eq!(
        original.molecules()[0]
            .bonds()
            .iter()
            .map(|bond| bond.haworth_position())
            .collect::<Vec<_>>(),
        vec![
            Some(DocumentHaworthPositionV1::Front),
            Some(DocumentHaworthPositionV1::Back),
        ]
    );
    assert_eq!(original, reparsed);
}

#[test]
fn malformed_haworth_depth_is_reported_without_source_coercion() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"b\" name=\"O\"><point x=\"1\" y=\"0\"/></atom>",
        "<bond start=\"a\" end=\"b\" haworth_position=\"side\"/></molecule></cdml>",
    );
    let retained = TypedDocument::parse(source).unwrap();
    let projection = projected(source);
    let reparsed = TypedDocument::parse(&retained.to_xml().unwrap()).unwrap();
    let bond = reparsed
        .root()
        .children_of(TypedClass::Molecule)
        .next()
        .unwrap()
        .children_of(TypedClass::Bond)
        .next()
        .unwrap();

    assert_eq!(
        projection.molecules()[0].bonds()[0].haworth_position(),
        None
    );
    assert_eq!(
        projection
            .issues()
            .iter()
            .filter(|issue| issue.code() == ProjectionIssueCodeV1::InvalidPresentationFact)
            .count(),
        1
    );
    assert_eq!(bond.attribute("haworth_position"), Some("side"));
}

#[test]
fn unsupported_endpoints_and_presentation_facts_become_ordered_issues() {
    let projection = projected(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"O\"><point x=\"1\" y=\"0\"/></atom><group id=\"g\"/><bond start=\"a\" end=\"g\"/><bond start=\"a\" end=\"b\" line_width=\"0\" color=\"blue\"/></molecule></cdml>",
    );
    assert_eq!(projection.molecules()[0].bonds().len(), 2);
    assert_eq!(
        projection.molecules()[0].bonds()[0].end().kind(),
        BondEndpointKindV1::Group
    );
    assert_eq!(
        projection.molecules()[0].bonds()[0].end().source_id(),
        Some("g")
    );
    assert_eq!(
        projection
            .issues()
            .iter()
            .map(|issue| issue.code())
            .collect::<Vec<_>>(),
        vec![
            ProjectionIssueCodeV1::UnsupportedBondEndpoint,
            ProjectionIssueCodeV1::InvalidPresentationFact,
            ProjectionIssueCodeV1::InvalidPresentationFact,
        ]
    );
}

#[test]
fn all_bond_endpoint_cases_are_retained_in_source_encounter_order() {
    let projection = projected(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><bond start=\"a\" end=\"missing\"/><atom id=\"b\" name=\"O\"><font size=\"0\"/><point x=\"1\" y=\"0\"/></atom><group id=\"g\"/><text id=\"t\"/><query id=\"q\"/><bond start=\"g\" end=\"t\"/><bond start=\"q\"/><bond end=\"a\"/></molecule></cdml>",
    );
    let bonds = projection.molecules()[0].bonds();
    assert_eq!(bonds.len(), 4);
    assert_eq!(bonds[0].end().kind(), BondEndpointKindV1::Unknown);
    assert_eq!(bonds[1].start().kind(), BondEndpointKindV1::Group);
    assert_eq!(bonds[1].end().kind(), BondEndpointKindV1::MoleculeText);
    assert_eq!(bonds[2].start().kind(), BondEndpointKindV1::Query);
    assert_eq!(bonds[2].end().kind(), BondEndpointKindV1::Missing);
    assert_eq!(bonds[3].start().kind(), BondEndpointKindV1::Missing);
    assert_eq!(
        projection
            .issues()
            .iter()
            .map(|issue| issue.code())
            .collect::<Vec<_>>(),
        vec![
            ProjectionIssueCodeV1::UnknownBondEndpoint,
            ProjectionIssueCodeV1::InvalidPresentationFact,
            ProjectionIssueCodeV1::UnsupportedBondEndpoint,
            ProjectionIssueCodeV1::UnsupportedBondEndpoint,
            ProjectionIssueCodeV1::UnsupportedBondEndpoint,
            ProjectionIssueCodeV1::MissingBondEndpoint,
            ProjectionIssueCodeV1::MissingBondEndpoint,
        ]
    );
}

#[test]
fn source_identity_is_validated_resolvable_and_stable_across_reparse_and_mutation() {
    let original = TypedDocument::parse(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a/\u{03b2}\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .unwrap();
    let first = projected(&original.to_xml().unwrap());
    let id = first.molecules()[0].atoms()[0].id().unwrap().clone();
    assert!(id.as_str().contains("/source/"));
    assert_eq!(DocumentObjectIdV1::parse(id.as_str()), Ok(id.clone()));
    assert!(original.resolve_document_object_id(&id).is_some());
    assert!(DocumentObjectIdV1::parse("ferrum-document-object-v1/zz/source/00").is_err());
    let reparsed = TypedDocument::parse(&original.to_xml().unwrap()).unwrap();
    let reparsed_id = projected(&reparsed.to_xml().unwrap()).molecules()[0].atoms()[0]
        .id()
        .unwrap()
        .clone();
    assert_eq!(id, reparsed_id);
    let changed = reparsed
        .with_atom_element(&PersistentId::new("a/\u{03b2}").unwrap(), "N")
        .unwrap()
        .unwrap();
    let changed_id = projected(&changed.to_xml().unwrap()).molecules()[0].atoms()[0]
        .id()
        .unwrap()
        .clone();
    assert_eq!(id, changed_id);
    assert!(changed.resolve_document_object_id(&changed_id).is_some());
}

#[test]
fn wire_serialization_has_closed_names_nulls_scalar_ids_and_issue_taxonomy() {
    let projection = projected(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><bond start=\"a\"/></molecule></cdml>",
    );
    let wire = serde_json::to_value(projection).unwrap();
    assert_eq!(wire["schema"], "ferrum-document-projection-v1");
    assert_eq!(wire["revision"], 0);
    assert_eq!(wire["is_dirty"], false);
    assert_eq!(wire["drawing_standard"], Value::Null);
    assert!(wire["molecules"][0]["id"].is_string());
    assert_eq!(wire["molecules"][0]["atoms"][0]["label_font"], Value::Null);
    assert_eq!(
        wire["molecules"][0]["bonds"][0]["end"]["source_id"],
        Value::Null
    );
    assert_eq!(
        wire["molecules"][0]["bonds"][0]["end"]["object_id"],
        Value::Null
    );
    assert_eq!(wire["molecules"][0]["bonds"][0]["end"]["kind"], "missing");
    assert_eq!(wire["issues"][0]["code"], "missing_bond_endpoint");
    assert_eq!(
        wire["issues"][0]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        vec!["code", "detail", "path"]
    );
}

#[test]
fn pixel_widths_are_retained_as_positive_presentation_facts() {
    let projection = projected(
        "<cdml xmlns=\"urn:ferrum:cdml\"><standard line_width=\"2.0px\"/><molecule><atom name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    );
    assert_eq!(
        projection
            .drawing_standard()
            .unwrap()
            .line_width()
            .unwrap()
            .value(),
        2.0
    );
}

#[test]
fn malformed_required_positions_fail_without_nonfinite_projection() {
    let session = DocumentSession::load(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule><atom name=\"C\"><point x=\"NaN\" y=\"0\"/></atom></molecule></cdml>",
    )
    .unwrap();
    let snapshot = session.snapshot().unwrap();
    let document = TypedDocument::parse(snapshot.cdml()).unwrap();
    assert!(matches!(
        DocumentProjectionV1::from_snapshot(&document, &snapshot),
        Err(ProjectionError::NonFiniteCoordinate { axis: "x" })
    ));
}

#[test]
fn duplicate_idless_records_have_distinct_projection_keys_and_no_operation_ids() {
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"editable\" name=\"O\"><point x=\"1\" y=\"0\"/></atom><atom name=\"C\"><point x=\"0\" y=\"0\"/></atom><group/><atom name=\"C\"><point x=\"0\" y=\"0\"/></atom><group/></molecule></cdml>";
    let mut session = DocumentSession::load(source).unwrap();
    let projection = session.observe(0).unwrap();
    let molecule = &projection.projection().molecules()[0];
    let atom_keys = molecule
        .atoms()
        .iter()
        .map(|atom| atom.projection_key().as_str())
        .collect::<Vec<_>>();
    assert_ne!(atom_keys[1], atom_keys[2]);
    assert!(molecule.atoms()[1..].iter().all(|atom| atom.id().is_none()));
    assert!(molecule.id().is_some());
    assert!(
        molecule
            .projection_key()
            .as_str()
            .starts_with("ferrum-projection-local-v1/")
    );
    session
        .submit(
            0,
            super::SessionOperation::V1(super::SessionOperationV1::SetAtomElement {
                atom_id: "editable".to_owned(),
                element: "N".to_owned(),
            }),
        )
        .unwrap();
    let after_projection = session.observe(1).unwrap();
    let after_atoms = after_projection.projection().molecules()[0].atoms();
    assert!(after_atoms[1..].iter().all(|atom| atom.id().is_none()));
    assert_ne!(
        after_atoms[1].projection_key().as_str(),
        after_atoms[2].projection_key().as_str()
    );
}

#[test]
fn root_issues_follow_root_encounter_order() {
    let projection = projected(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><bond start=\"a\" end=\"unknown\"/></molecule><standard line_width=\"0\"/></cdml>",
    );
    assert_eq!(
        projection
            .issues()
            .iter()
            .map(|issue| issue.code())
            .collect::<Vec<_>>(),
        vec![
            ProjectionIssueCodeV1::UnknownBondEndpoint,
            ProjectionIssueCodeV1::InvalidPresentationFact,
        ]
    );
}

#[test]
fn session_observation_has_single_state_provenance_and_rejects_stale_reads() {
    let mut session = DocumentSession::load(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .unwrap();
    let before = session.observe(0).unwrap();
    assert_eq!(before.snapshot().revision(), 0);
    assert_eq!(before.snapshot().revision(), before.projection().revision());
    assert_eq!(before.snapshot().digest(), before.projection().digest());
    assert_eq!(before.snapshot().is_dirty(), before.projection().is_dirty());
    assert!(!before.snapshot().is_dirty());
    session
        .submit(
            0,
            super::SessionOperation::V1(super::SessionOperationV1::SetAtomElement {
                atom_id: "a".to_owned(),
                element: "N".to_owned(),
            }),
        )
        .unwrap();
    assert!(matches!(
        session.observe(0),
        Err(super::DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1,
        })
    ));
    let after = session.observe(1).unwrap();
    assert_eq!(after.snapshot().revision(), 1);
    assert_eq!(after.snapshot().revision(), after.projection().revision());
    assert!(after.snapshot().is_dirty());
    assert_ne!(before.snapshot().digest(), after.snapshot().digest());
}
#[test]
fn session_observation_keeps_authored_source_identity_for_direct_molecule_join() {
    let observation = crate::DocumentSession::load(concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"authored-molecule\">",
        "<atom id=\"atom-second\" name=\"O\"><point x=\"2\" y=\"0\"/></atom>",
        "<atom id=\"atom-first\" name=\"C\"><point x=\"1\" y=\"0\"/></atom>",
        "<bond id=\"bond\" start=\"atom-second\" end=\"atom-first\" type=\"n1\"/>",
        "</molecule></cdml>",
    ))
    .unwrap()
    .observe(0)
    .unwrap();

    let root = &observation.projection().molecules()[0];
    assert_ne!(root.id().unwrap().as_str(), "authored-molecule");
    assert_eq!(root.source_id(), Some("authored-molecule"));
}
