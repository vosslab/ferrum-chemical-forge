use super::{
    BondEndpointKindV1, DocumentHaworthPositionV1, DocumentProjectionV1, DocumentSession,
    PresentationProjectionIssueCodeV1, PresentationRecordKindV1, ProjectionError,
    ProjectionIssueCodeV1, TypedDocument,
};
use ferrum_document_projection::DocumentDirectRootKindV1;

fn projected(source: &str) -> DocumentProjectionV1 {
    let session = DocumentSession::load(source).unwrap();
    let snapshot = session.snapshot().unwrap();
    crate::projection_adapter::document_projection_from_snapshot_v1(&snapshot).unwrap()
}

#[test]
fn direct_roots_preserve_interleaved_targets_and_sparse_positions() {
    let projection = projected(concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"first\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<plus id=\"plus\"><point x=\"20\" y=\"0\"/></plus><reaction id=\"gap\"/>",
        "<molecule id=\"second\"><atom id=\"b\" name=\"O\"><point x=\"1\" y=\"0\"/></atom></molecule>",
        "<arrow id=\"arrow\" type=\"normal\"><point x=\"0\" y=\"0\"/><point x=\"20\" y=\"0\"/></arrow>",
        "<arrow id=\"rejected\" type=\"normal\"><point x=\"0\" y=\"0\"/></arrow></cdml>",
    ));
    let roots = projection.direct_roots();
    let first = projection.molecules()[0].document_object_id();
    let second = projection.molecules()[1].document_object_id();
    let presentation_roots = projection.presentation_stack().entries();
    let plus = presentation_roots[0].root().target();
    let arrow = presentation_roots[1].root().target();
    let rejected_issue = projection
        .presentation_stack()
        .issues()
        .iter()
        .find(|issue| issue.code() == PresentationProjectionIssueCodeV1::InvalidArrowGeometry)
        .expect("rejected arrow has a projection issue");
    assert_eq!(
        roots[0].document_object_id(),
        first,
        "first direct root targets the first projected molecule"
    );
    assert_eq!(
        roots[1].document_object_id(),
        plus.document_object_id(),
        "second direct root targets the projected plus"
    );
    assert_eq!(
        roots[2].document_object_id(),
        second,
        "third direct root targets the second projected molecule"
    );
    assert_eq!(
        roots[3].document_object_id(),
        arrow.document_object_id(),
        "fourth direct root targets the projected arrow"
    );
    assert_eq!(
        roots[4].document_object_id(),
        rejected_issue.target().document_object_id(),
        "rejected direct root targets its matching projection issue"
    );
    assert_eq!(
        roots
            .iter()
            .map(|root| root.paint_order())
            .collect::<Vec<_>>(),
        vec![0, 1, 3, 4, 5]
    );
    assert_eq!(roots[0].kind(), DocumentDirectRootKindV1::Molecule);
    assert_eq!(
        roots[1].kind(),
        DocumentDirectRootKindV1::Presentation(PresentationRecordKindV1::Plus)
    );
    assert_eq!(roots[2].kind(), DocumentDirectRootKindV1::Molecule);
    assert_eq!(
        roots[3].kind(),
        DocumentDirectRootKindV1::Presentation(PresentationRecordKindV1::Arrow)
    );
    assert_eq!(
        roots[4].kind(),
        DocumentDirectRootKindV1::RejectedPresentation(
            PresentationProjectionIssueCodeV1::InvalidArrowGeometry
        )
    );
}

#[test]
fn haworth_depth_facts_project_and_survive_typed_round_trip() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"b\" name=\"O\"><point x=\"1\" y=\"0\"/></atom>",
        "<bond id=\"bond\" start=\"a\" end=\"b\" haworth_position=\"front\"/></molecule></cdml>",
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
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"b\" name=\"O\"><point x=\"1\" y=\"0\"/></atom>",
        "<bond id=\"bond\" start=\"a\" end=\"b\" haworth_position=\"side\"/></molecule></cdml>",
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
        "<bond id=\"bond\" start=\"anchor\" end=\"group\" type=\"n1\"/>",
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
        "<bond id=\"bond\" start=\"anchor\" end=\"group\" type=\"n1\"/>",
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
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"NaN\" y=\"0\"/></atom></molecule></cdml>",
    )
    .unwrap();
    let snapshot = session.snapshot().unwrap();
    assert!(matches!(
        crate::projection_adapter::document_projection_from_snapshot_v1(&snapshot),
        Err(ProjectionError::NonFiniteCoordinate { axis: "x" })
    ));
}
