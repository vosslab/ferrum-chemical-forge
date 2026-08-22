use super::{
    InterchangeRecordMetadataErrorV1, TypedDocument, observe_interchange_record_metadata_v1,
};

#[test]
fn exact_metadata_recovers_blank_title_and_repeated_ordered_properties() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
        "</atom><ferrum-interchange:interchange-record ",
        "xmlns:ferrum-interchange=\"urn:ferrum-chemical-forge:interchange-import:v1\" ",
        "encoding=\"utf8-hex-v1\" title=\"\">",
        "<ferrum-interchange:property name=\"4e4f5445\" value=\"6669727374\"/>",
        "<ferrum-interchange:property name=\"4e4f5445\" value=\"7365636f6e64\"/>",
        "</ferrum-interchange:interchange-record><vendor:keep xmlns:vendor=\"urn:vendor\"/>",
        "</molecule></cdml>",
    );
    let document = TypedDocument::parse(source).expect("metadata source is valid CDML");

    let metadata = observe_interchange_record_metadata_v1(&document, "m")
        .expect("closed metadata decodes")
        .expect("metadata is present");

    assert_eq!(metadata.title(), "");
    assert_eq!(
        metadata
            .properties()
            .iter()
            .map(|property| (property.name(), property.value()))
            .collect::<Vec<_>>(),
        [("NOTE", "first"), ("NOTE", "second")],
    );
    assert_eq!(
        observe_interchange_record_metadata_v1(&document, "foreign"),
        Err(InterchangeRecordMetadataErrorV1::UnknownDirectMolecule),
    );
}

#[test]
fn authoritative_metadata_refuses_ambiguous_or_malformed_structure() {
    let duplicate = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><f:interchange-record xmlns:f=\"",
        "urn:ferrum-chemical-forge:interchange-import:v1\" encoding=\"utf8-hex-v1\" title=\"\"/>",
        "<f:interchange-record xmlns:f=\"urn:ferrum-chemical-forge:interchange-import:v1\" ",
        "encoding=\"utf8-hex-v1\" title=\"\"/></molecule></cdml>",
    );
    let document = TypedDocument::parse(duplicate).expect("duplicate source remains valid XML");
    assert_eq!(
        observe_interchange_record_metadata_v1(&document, "m"),
        Err(InterchangeRecordMetadataErrorV1::DuplicateMetadata),
    );

    let malformed = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><f:interchange-record xmlns:f=\"",
        "urn:ferrum-chemical-forge:interchange-import:v1\" encoding=\"utf8-hex-v1\" ",
        "title=\"0\"/></molecule></cdml>",
    );
    let document = TypedDocument::parse(malformed).expect("malformed metadata remains valid XML");
    assert_eq!(
        observe_interchange_record_metadata_v1(&document, "m"),
        Err(InterchangeRecordMetadataErrorV1::InvalidHex),
    );

    for (name, value, expected) in [
        (
            "6261643e6e616d65",
            "76616c7565",
            InterchangeRecordMetadataErrorV1::InvalidPropertyName,
        ),
        (
            "6669656c64",
            "24242424",
            InterchangeRecordMetadataErrorV1::InvalidPropertyValue,
        ),
    ] {
        let source = format!(
            concat!(
                "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><f:interchange-record xmlns:f=\"",
                "urn:ferrum-chemical-forge:interchange-import:v1\" encoding=\"utf8-hex-v1\" ",
                "title=\"\"><f:property name=\"{}\" value=\"{}\"/>",
                "</f:interchange-record></molecule></cdml>",
            ),
            name, value,
        );
        let document = TypedDocument::parse(&source).expect("closed invalid metadata is XML");
        assert_eq!(
            observe_interchange_record_metadata_v1(&document, "m"),
            Err(expected),
        );
    }
}
