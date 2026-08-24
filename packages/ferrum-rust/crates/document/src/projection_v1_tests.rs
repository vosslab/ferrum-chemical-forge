use super::{
    BondEndpointKindV1, DocumentHaworthPositionV1, DocumentProjectionV1, DocumentSession,
    ProjectionError, ProjectionIssueCodeV1, TypedDocument,
};

fn projected(source: &str) -> DocumentProjectionV1 {
    let session = DocumentSession::load(source).unwrap();
    let snapshot = session.snapshot().unwrap();
    crate::projection_adapter::document_projection_from_snapshot_v1(&snapshot).unwrap()
}

#[test]
fn molecules_retain_root_source_order() {
    let projection = projected(concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"first\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"second\"><atom id=\"b\" name=\"O\"><point x=\"1\" y=\"0\"/></atom></molecule></cdml>",
    ));
    assert_eq!(projection.molecules()[0].source_id(), Some("first"));
    assert_eq!(projection.molecules()[1].source_id(), Some("second"));
}

#[test]
fn haworth_depth_facts_project_and_survive_typed_round_trip() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"b\" name=\"O\"><point x=\"1\" y=\"0\"/></atom>",
        "<bond start=\"a\" end=\"b\" haworth_position=\"front\"/></molecule></cdml>",
    );
    let original = projected(source);
    let retained = TypedDocument::parse(source).unwrap();
    let reparsed = projected(&retained.to_xml().unwrap());

    assert_eq!(
        original.molecules()[0].bonds()[0].haworth_position(),
        Some(DocumentHaworthPositionV1::Front)
    );
    assert_eq!(
        reparsed.molecules()[0].bonds()[0].haworth_position(),
        Some(DocumentHaworthPositionV1::Front)
    );
}

#[test]
fn malformed_haworth_depth_is_omitted_with_invalid_presentation_issue() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"b\" name=\"O\"><point x=\"1\" y=\"0\"/></atom>",
        "<bond start=\"a\" end=\"b\" haworth_position=\"side\"/></molecule></cdml>",
    );
    let projection = projected(source);
    assert_eq!(
        projection.molecules()[0].bonds()[0].haworth_position(),
        None
    );
    assert!(
        projection
            .issues()
            .iter()
            .any(|issue| issue.code() == ProjectionIssueCodeV1::InvalidPresentationFact)
    );
}

#[test]
fn typed_compact_group_endpoint_projects_as_group() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule\">",
        "<atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group>",
        "<bond start=\"anchor\" end=\"group\" type=\"n1\"/>",
        "</molecule></cdml>",
    );
    let projection = projected(source);
    assert_eq!(
        projection.molecules()[0].bonds()[0].end().kind(),
        BondEndpointKindV1::Group
    );
}

#[test]
fn malformed_compact_group_endpoint_refuses_projection_without_endpoint_guessing() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule\">",
        "<atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<compact-group id=\"group\" version=\"1\" catalog-key=\"not-a-v1-key\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group>",
        "<bond start=\"anchor\" end=\"group\" type=\"n1\"/>",
        "</molecule></cdml>",
    );
    let session = DocumentSession::load(source).expect("malformed compact source must load");
    let snapshot = session
        .snapshot()
        .expect("malformed compact source must snapshot");
    assert!(matches!(
        crate::projection_adapter::document_projection_from_snapshot_v1(&snapshot),
        Err(ProjectionError::CompactGroup { .. })
    ));
}

#[test]
fn malformed_required_positions_fail_without_nonfinite_projection() {
    let session = DocumentSession::load(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule><atom name=\"C\"><point x=\"NaN\" y=\"0\"/></atom></molecule></cdml>",
    )
    .unwrap();
    let snapshot = session.snapshot().unwrap();
    assert!(matches!(
        crate::projection_adapter::document_projection_from_snapshot_v1(&snapshot),
        Err(ProjectionError::NonFiniteCoordinate { axis: "x" })
    ));
}
