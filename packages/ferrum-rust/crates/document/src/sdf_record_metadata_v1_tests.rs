use super::{SdfRecordMetadataErrorV1, TypedDocument, observe_sdf_record_metadata_v1};

#[test]
fn exact_metadata_recovers_blank_title_and_repeated_ordered_properties() {
    let source = concat!(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
        "</atom><ferrum-sdf:sdf-record ",
        "xmlns:ferrum-sdf=\"urn:ferrum-chemical-forge:sdf-import:v1\" ",
        "encoding=\"utf8-hex-v1\" title=\"\">",
        "<ferrum-sdf:property name=\"4e4f5445\" value=\"6669727374\"/>",
        "<ferrum-sdf:property name=\"4e4f5445\" value=\"7365636f6e64\"/>",
        "</ferrum-sdf:sdf-record><vendor:keep xmlns:vendor=\"urn:vendor\"/>",
        "</molecule></cdml>",
    );
    let document = TypedDocument::parse(source).expect("metadata source is valid CDML");

    let metadata = observe_sdf_record_metadata_v1(&document, "m")
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
        observe_sdf_record_metadata_v1(&document, "foreign"),
        Err(SdfRecordMetadataErrorV1::UnknownDirectMolecule),
    );
}

#[test]
fn authoritative_metadata_refuses_ambiguous_or_malformed_structure() {
    let duplicate = concat!(
        "<cdml><molecule id=\"m\"><f:sdf-record xmlns:f=\"",
        "urn:ferrum-chemical-forge:sdf-import:v1\" encoding=\"utf8-hex-v1\" title=\"\"/>",
        "<f:sdf-record xmlns:f=\"urn:ferrum-chemical-forge:sdf-import:v1\" ",
        "encoding=\"utf8-hex-v1\" title=\"\"/></molecule></cdml>",
    );
    let document = TypedDocument::parse(duplicate).expect("duplicate source remains valid XML");
    assert_eq!(
        observe_sdf_record_metadata_v1(&document, "m"),
        Err(SdfRecordMetadataErrorV1::DuplicateMetadata),
    );

    let malformed = concat!(
        "<cdml><molecule id=\"m\"><f:sdf-record xmlns:f=\"",
        "urn:ferrum-chemical-forge:sdf-import:v1\" encoding=\"utf8-hex-v1\" ",
        "title=\"0\"/></molecule></cdml>",
    );
    let document = TypedDocument::parse(malformed).expect("malformed metadata remains valid XML");
    assert_eq!(
        observe_sdf_record_metadata_v1(&document, "m"),
        Err(SdfRecordMetadataErrorV1::InvalidHex),
    );

    for (name, value, expected) in [
        (
            "6261643e6e616d65",
            "76616c7565",
            SdfRecordMetadataErrorV1::InvalidPropertyName,
        ),
        (
            "6669656c64",
            "24242424",
            SdfRecordMetadataErrorV1::InvalidPropertyValue,
        ),
    ] {
        let source = format!(
            concat!(
                "<cdml><molecule id=\"m\"><f:sdf-record xmlns:f=\"",
                "urn:ferrum-chemical-forge:sdf-import:v1\" encoding=\"utf8-hex-v1\" ",
                "title=\"\"><f:property name=\"{}\" value=\"{}\"/>",
                "</f:sdf-record></molecule></cdml>",
            ),
            name, value,
        );
        let document = TypedDocument::parse(&source).expect("closed invalid metadata is XML");
        assert_eq!(
            observe_sdf_record_metadata_v1(&document, "m"),
            Err(expected),
        );
    }
}
