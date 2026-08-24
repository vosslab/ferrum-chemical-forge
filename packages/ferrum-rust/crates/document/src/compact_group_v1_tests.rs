use super::chemistry::{DocumentMoleculeGraphError, document_molecule_graph_v1};
use super::{
    CompactGroupCatalogKeyV1, CoreProjectionError, DocumentProjectionV1, DocumentSession,
    ProjectionError, TypedDocument, TypedDocumentError,
};
use ferrum_core::{BondOrder, BondStyle, VertexRef};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule\">",
    "<compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"390\">",
    "\n  <point x=\"12\" y=\"-8\"/>\n</compact-group></molecule></cdml>",
);

fn projected(source: &str) -> Result<DocumentProjectionV1, ProjectionError> {
    let session = DocumentSession::load(source).expect("CDML source must load structurally");
    let snapshot = session.snapshot().expect("snapshot must serialize");
    crate::projection_adapter::document_projection_from_snapshot_v1(&snapshot)
}

#[test]
fn typed_compact_group_projects_a_key_derived_label_and_reopens_without_translation() {
    let initial = projected(SOURCE).expect("supported compact group must project");
    let group = &initial.molecules()[0].compact_groups()[0];
    assert_eq!(group.catalog_key(), CompactGroupCatalogKeyV1::Methyl);
    assert_eq!(group.label(), "Me");
    assert_eq!(group.anchor().x(), 12.0);
    assert_eq!(group.anchor().y(), -8.0);
    assert_eq!(group.attachment_index(), 0);
    assert_eq!(group.orientation_degrees(), 30.0);

    let session = DocumentSession::load(SOURCE).expect("source must open in a session");
    let saved = session.snapshot().expect("source snapshot must serialize");
    let reopened = projected(saved.cdml()).expect("saved compact group must reopen");
    assert_eq!(initial, reopened);
    assert_eq!(reopened.molecules()[0].compact_groups()[0].id(), group.id());
}

#[test]
fn compact_group_projection_preserves_two_group_source_order_and_reopen_identities() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule\">",
        "<compact-group id=\"methyl\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"0\" y=\"0\"/></compact-group>",
        "<atom id=\"carbon\" name=\"C\"><point x=\"20\" y=\"0\"/></atom>",
        "<compact-group id=\"nitro\" version=\"1\" catalog-key=\"nitro\" attachment-index=\"0\" orientation-degrees=\"90\"><point x=\"40\" y=\"0\"/></compact-group>",
        "</molecule></cdml>",
    );
    let initial = projected(source).expect("two compact groups must project");
    let groups = initial.molecules()[0].compact_groups();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].source_order(), 0);
    assert_eq!(groups[1].source_order(), 2);
    assert_ne!(groups[0].id(), groups[1].id());

    let session = DocumentSession::load(source).expect("source must open in a session");
    let snapshot = session.snapshot().expect("source must snapshot");
    let reopened = projected(snapshot.cdml()).expect("saved source must reopen");
    assert_eq!(reopened.molecules()[0].compact_groups(), groups);
}

#[test]
fn compact_group_is_a_durable_core_group_endpoint_and_reopens_without_translation() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"root\">",
        "<atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group>",
        "<bond id=\"attachment\" start=\"anchor\" end=\"group\" type=\"n1\"/>",
        "</molecule></cdml>",
    );
    let initial = TypedDocument::parse(source)
        .expect("attached compact group must type")
        .core_projection()
        .expect("attached compact group must form a core graph");
    let molecule = &initial.molecules()[0];
    let anchor = molecule.atoms()[0].identity();
    let group = molecule.groups()[0].identity();
    let bond = &molecule.bonds()[0];
    assert_eq!(molecule.groups().len(), 1);
    assert_eq!(
        molecule.groups()[0].source_id().map(|id| id.as_str()),
        Some("group")
    );
    assert_eq!(bond.start(), &VertexRef::Atom(anchor.clone()));
    assert_eq!(bond.end(), &VertexRef::Group(group.clone()));
    assert_eq!(bond.order(), Some(BondOrder::Single));
    assert_eq!(bond.style(), Some(&BondStyle::Normal));
    assert!(matches!(
        document_molecule_graph_v1(molecule),
        Err(DocumentMoleculeGraphError::UnsupportedVertex {
            kind: "group",
            count: 1,
        })
    ));

    let saved = DocumentSession::load(source)
        .expect("attached compact source must load")
        .snapshot()
        .expect("attached compact source must snapshot");
    let reopened = TypedDocument::parse(saved.cdml())
        .expect("saved compact source must type")
        .core_projection()
        .expect("saved compact source must form the same core graph");
    assert_eq!(reopened, initial);
}

#[test]
fn compact_group_core_bridge_refuses_missing_or_unresolved_endpoint_facts() {
    let missing_id = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"root\">",
        "<atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<compact-group version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group>",
        "</molecule></cdml>",
    );
    assert!(matches!(
        TypedDocument::parse(missing_id)
            .expect("missing compact ID remains structurally typed")
            .core_projection(),
        Err(CoreProjectionError::CompactGroup(
            ProjectionError::InvalidCompactGroupField { field: "id", .. }
        ))
    ));

    let unknown_endpoint = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"root\">",
        "<atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<bond start=\"anchor\" end=\"missing\" type=\"n1\"/>",
        "</molecule></cdml>",
    );
    assert!(matches!(
        TypedDocument::parse(unknown_endpoint)
            .expect("unresolved source remains structurally typed")
            .core_projection(),
        Err(CoreProjectionError::UnknownVertex { field: "end", identifier, .. })
            if identifier == "missing"
    ));
}

#[test]
fn compact_group_refuses_undeclared_attributes_at_typed_document_load() {
    let source = SOURCE.replace(
        "version=\"1\"",
        "version=\"1\" local-extension=\"unsupported\"",
    );
    assert!(matches!(
        TypedDocument::parse(&source),
        Err(TypedDocumentError::UndeclaredCompactGroupAttribute { attribute })
            if attribute == "local-extension"
    ));
}

#[test]
fn compact_group_refuses_undeclared_direct_child_at_typed_document_load() {
    let source = SOURCE.replace(
        "\n</compact-group>",
        "<future-private-fact value=\"hidden\"/>\n</compact-group>",
    );
    assert!(matches!(
        TypedDocument::parse(&source),
        Err(TypedDocumentError::UndeclaredCompactGroupContent)
    ));
}

#[test]
fn compact_group_refuses_duplicate_anchor_point_at_typed_document_load() {
    let source = SOURCE.replace(
        "\n</compact-group>",
        "<point x=\"13\" y=\"-8\"/>\n</compact-group>",
    );
    assert!(matches!(
        TypedDocument::parse(&source),
        Err(TypedDocumentError::UndeclaredCompactGroupContent)
    ));
}

#[test]
fn compact_group_refuses_nonwhitespace_text_at_typed_document_load() {
    let source = SOURCE.replace("\n</compact-group>", "private\n</compact-group>");
    assert!(matches!(
        TypedDocument::parse(&source),
        Err(TypedDocumentError::UndeclaredCompactGroupContent)
    ));
}

#[test]
fn invalid_compact_group_forms_refuse_at_the_typed_projection_boundary() {
    for source in [
        SOURCE.replace("catalog-key=\"methyl\"", "catalog-key=\"legacy_me\""),
        SOURCE.replace("attachment-index=\"0\"", "attachment-index=\"2\""),
        SOURCE.replace("orientation-degrees=\"390\"", "orientation-degrees=\"NaN\""),
        SOURCE.replace("version=\"1\"", "version=\"2\""),
    ] {
        assert!(projected(&source).is_err());
    }
}

#[test]
fn legacy_group_records_remain_structurally_retained_without_compact_group_classification() {
    let source = SOURCE.replace(
        concat!(
            "<compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" ",
            "attachment-index=\"0\" orientation-degrees=\"390\">\n  ",
            "<point x=\"12\" y=\"-8\"/>\n</compact-group>",
        ),
            "<group id=\"group\" name=\"Me\" group-type=\"legacy\"><point x=\"12\" y=\"-8\"/></group>",
        )
        .replace(
            "<molecule id=\"molecule\">",
            "<molecule id=\"molecule\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        );
    let document = TypedDocument::parse(&source).expect("legacy group stays structurally valid");
    assert_eq!(document.to_xml().expect("legacy group serializes"), source);
    assert!(
        projected(&source)
            .expect("legacy group is not translated")
            .molecules()[0]
            .compact_groups()
            .is_empty()
    );
}

#[test]
fn legacy_generic_group_endpoint_is_excluded_from_the_core_graph() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"root\">",
        "<atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<group id=\"legacy\" name=\"Me\" group-type=\"legacy\"><point x=\"20\" y=\"0\"/></group>",
        "<bond start=\"anchor\" end=\"legacy\" type=\"n1\"/>",
        "</molecule></cdml>",
    );
    assert!(matches!(
        TypedDocument::parse(source)
            .expect("legacy group remains structurally typed")
            .core_projection(),
        Err(CoreProjectionError::UnknownVertex { field: "end", identifier, .. })
            if identifier == "legacy"
    ));
}
