use ferrum_document::{TypedDocument, TypedDocumentError};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule\">",
    "<compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"390\">",
    "\n  <point x=\"12\" y=\"-8\"/>\n</compact-group></molecule></cdml>",
);

#[test]
fn compact_group_refuses_xml_comment_at_typed_document_load() {
    let source = SOURCE.replace(
        "\n</compact-group>",
        "\n  <!-- future compact-group content -->\n</compact-group>",
    );

    assert!(matches!(
        TypedDocument::parse(&source),
        Err(TypedDocumentError::UndeclaredCompactGroupContent)
    ));
}

#[test]
fn compact_group_refuses_processing_instruction_at_typed_document_load() {
    let source = SOURCE.replace(
        "\n</compact-group>",
        "\n  <?ferrum compact-group-extension?>\n</compact-group>",
    );

    assert!(matches!(
        TypedDocument::parse(&source),
        Err(TypedDocumentError::UndeclaredCompactGroupContent)
    ));
}
