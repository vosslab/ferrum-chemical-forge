use super::*;
use crate::{BondDirection, BondOrder, CdxmlBondPresentationV1};

const VENDOR_DTD: &str = "https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd";
const BASIC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE CDXML SYSTEM "https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd">
<CDXML CreationProgram="ChemDraw 23.0" BondLength="30"><page HeightPages="1"><fragment id="one"><n id="a" p="0 0"/><n id="b" p="30 0" Element="8"/><n id="c" p="0 30"><t><s>N</s></t></n><b B="a" E="b"/><b B="a" E="c" Display="WedgeBegin"/></fragment></page></CDXML>"#;

fn document(body: &str) -> String {
    format!("<CDXML><page><fragment id=\"f\">{body}</fragment></page></CDXML>")
}

fn two_atoms(extra: &str) -> String {
    document(&format!(
        "<n id=\"a\" p=\"0 0\"/><n id=\"b\" p=\"1 0\"/>{extra}"
    ))
}

fn assert_refusal(input: &[u8], reason: CdxmlRefusalReasonV1) {
    assert_eq!(
        decode_cdxml_bytes_v1(input)
            .expect_err("closed profile refusal")
            .reason(),
        reason
    );
}

fn records_document(record_count: usize) -> String {
    let mut input = String::from("<CDXML><page>");
    for index in 0..record_count {
        input.push_str(&format!(
            "<fragment id=\"f{index}\"><n id=\"a\" p=\"0 0\"/></fragment>"
        ));
    }
    input.push_str("</page></CDXML>");
    input
}

fn atoms_document(atom_count: usize) -> String {
    let mut input = String::from("<CDXML><page><fragment id=\"f\">");
    for index in 0..atom_count {
        input.push_str(&format!("<n id=\"a{index}\" p=\"{index} 0\"/>"));
    }
    input.push_str("</fragment></page></CDXML>");
    input
}

fn bonds_document(bond_count: usize) -> String {
    let mut input = atoms_document(202);
    let end = "</fragment></page></CDXML>";
    input.truncate(input.len() - end.len());
    let mut added = 0;
    for start in 0..202 {
        for finish in (start + 1)..202 {
            input.push_str(&format!("<b B=\"a{start}\" E=\"a{finish}\"/>"));
            added += 1;
            if added == bond_count {
                input.push_str(end);
                return input;
            }
        }
    }
    panic!("requested more distinct bonds than the test graph contains");
}

#[test]
fn cdxml_c1_decodes_producer_prolog_metadata_and_authored_direction() {
    let decoded = decode_cdxml_bytes_v1(BASIC.as_bytes()).expect("bounded CDXML");
    assert_eq!(decoded.records().len(), 1);
    assert_eq!(decoded.records()[0].source_fragment_id(), "one");
    let graph = decoded.records()[0].record().molecule();
    assert_eq!(graph.atoms()[0].atomic_number().symbol(), "C");
    assert_eq!(graph.atoms()[1].atomic_number().symbol(), "O");
    assert_eq!(graph.atoms()[2].atomic_number().symbol(), "N");
    assert_eq!(graph.bonds()[1].direction(), BondDirection::BeginWedge);
    assert_eq!(
        decoded.declared_losses(),
        &[
            CdxmlLossCategoryV1::LexicalSyntax,
            CdxmlLossCategoryV1::DocumentViewMetadata
        ]
    );
}

#[test]
fn cdxml_c1_reports_losses_in_canonical_enum_order() {
    let input = "<CDXML><colortable/><!-- retained lexical detail --><page><fragment id=\"f\"><n id=\"a\" p=\"0 0\"/></fragment></page></CDXML>";
    let decoded = decode_cdxml_bytes_v1(input.as_bytes()).expect("bounded CDXML");
    assert_eq!(
        decoded.declared_losses(),
        &[
            CdxmlLossCategoryV1::LexicalSyntax,
            CdxmlLossCategoryV1::DocumentViewMetadata,
        ]
    );
}

#[test]
fn cdxml_c1_preserves_fragment_order_orders_and_hash_direction() {
    let input = r#"<CDXML><page><fragment id="first"><n id="a" p="0 0"/><n id="b" p="1 0"/><b B="a" E="b" Order="2"/></fragment><fragment id="second"><n id="a" p="0 0"/><n id="b" p="1 0"/><b B="a" E="b" Order="1" Display="WedgedHashBegin"/></fragment></page></CDXML>"#;
    let decoded = decode_cdxml_bytes_v1(input.as_bytes()).expect("two fragments");
    assert_eq!(
        decoded
            .records()
            .iter()
            .map(CdxmlDecodedRecordV1::source_fragment_id)
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert_eq!(
        decoded.records()[0].record().molecule().bonds()[0].order(),
        BondOrder::Double
    );
    assert_eq!(
        decoded.records()[1].record().molecule().bonds()[0].direction(),
        BondDirection::BeginDash
    );
}

#[test]
fn cdxml_c1_preserves_exact_fixed_single_presentations_in_source_bond_order() {
    let input = two_atoms(
        "<b B=\"a\" E=\"b\" Display=\"Wavy\"/><n id=\"c\" p=\"2 0\"/><b B=\"b\" E=\"c\" Display=\"Bold\"/><n id=\"d\" p=\"3 0\"/><b B=\"c\" E=\"d\" Display=\"Dash\"/>",
    );
    let decoded = decode_cdxml_bytes_v1(input.as_bytes()).expect("closed presentations");
    let record = &decoded.records()[0];
    assert_eq!(
        record.bond_presentations(),
        &[
            Some(CdxmlBondPresentationV1::Wavy),
            Some(CdxmlBondPresentationV1::Bold),
            Some(CdxmlBondPresentationV1::Dashed),
        ]
    );
    assert!(
        record.record().molecule().bonds().iter().all(
            |bond| bond.order() == BondOrder::Single && bond.direction() == BondDirection::None
        )
    );
}

#[test]
fn cdxml_c1_keeps_ordinary_and_stereo_bonds_as_absent_presentations() {
    let input = two_atoms(
        "<b B=\"a\" E=\"b\"/><n id=\"c\" p=\"2 0\"/><b B=\"b\" E=\"c\" Display=\"Solid\"/><n id=\"d\" p=\"3 0\"/><b B=\"c\" E=\"d\" Display=\"WedgeBegin\"/>",
    );
    let decoded = decode_cdxml_bytes_v1(input.as_bytes()).expect("ordinary and stereo");
    assert_eq!(
        decoded.records()[0].bond_presentations(),
        &[None, None, None]
    );
}

#[test]
fn cdxml_c1_refuses_presentation_tokens_on_non_single_bonds_before_other_losses() {
    for display in ["Wavy", "Bold", "Dash"] {
        let input = two_atoms(&format!(
            "<b B=\"a\" E=\"b\" Order=\"2\" Display=\"{display}\"/>"
        ));
        assert_refusal(input.as_bytes(), CdxmlRefusalReasonV1::InvalidScalar);
    }
}

#[test]
fn cdxml_c1_refuses_unrepresented_display_tokens_regardless_of_order() {
    for (display, order) in [
        ("WedgeEnd", "1"),
        ("WedgedHashEnd", "1"),
        ("DashBegin", "1"),
        ("Unknown", "1"),
        ("WedgeEnd", "2"),
    ] {
        let input = two_atoms(&format!(
            "<b B=\"a\" E=\"b\" Order=\"{order}\" Display=\"{display}\"/>"
        ));
        assert_refusal(
            input.as_bytes(),
            CdxmlRefusalReasonV1::UnrepresentedSemanticFact,
        );
    }
}

#[test]
fn cdxml_decoded_record_carrier_refuses_misaligned_or_conflicting_presentations() {
    let ordinary = decode_cdxml_bytes_v1(two_atoms("<b B=\"a\" E=\"b\"/>").as_bytes())
        .expect("ordinary carrier")
        .records()[0]
        .record()
        .clone();
    assert_refused_carrier(ordinary.clone(), Vec::new());
    assert_refused_carrier(
        ordinary,
        vec![
            Some(CdxmlBondPresentationV1::Wavy),
            Some(CdxmlBondPresentationV1::Bold),
        ],
    );

    let double = decode_cdxml_bytes_v1(two_atoms("<b B=\"a\" E=\"b\" Order=\"2\"/>").as_bytes())
        .expect("double graph")
        .records()[0]
        .record()
        .clone();
    assert_refused_carrier(double, vec![Some(CdxmlBondPresentationV1::Dashed)]);

    let directed =
        decode_cdxml_bytes_v1(two_atoms("<b B=\"a\" E=\"b\" Display=\"WedgeBegin\"/>").as_bytes())
            .expect("directed graph")
            .records()[0]
            .record()
            .clone();
    assert_refused_carrier(directed, vec![Some(CdxmlBondPresentationV1::Bold)]);
}

fn assert_refused_carrier(
    record: crate::InterchangeRecordV1,
    presentations: Vec<Option<CdxmlBondPresentationV1>>,
) {
    assert_eq!(
        CdxmlDecodedRecordV1::new("test".to_owned(), record, presentations)
            .expect_err("invalid internal presentation carrier")
            .reason(),
        CdxmlRefusalReasonV1::InternalFailure,
    );
}

#[test]
fn cdxml_c1_accepts_closed_text_style_metadata_and_refuses_unknown_style() {
    let styled = document(
        "<n id=\"a\" p=\"0 0\"><t Justification=\"Center\"><s font=\"1\" size=\"12\" face=\"1\" color=\"2\">N</s></t></n>",
    );
    let decoded = decode_cdxml_bytes_v1(styled.as_bytes()).expect("styled direct label");
    assert_eq!(
        decoded.records()[0].record().molecule().atoms()[0]
            .atomic_number()
            .symbol(),
        "N"
    );
    assert_eq!(
        decoded.declared_losses(),
        &[CdxmlLossCategoryV1::DocumentViewMetadata]
    );
    let unknown = document("<n id=\"a\" p=\"0 0\"><t><s font=\"1\" unknown=\"x\">N</s></t></n>");
    assert_refusal(
        unknown.as_bytes(),
        CdxmlRefusalReasonV1::AttributeUnsupported,
    );
}

#[test]
fn cdxml_c2_preserves_canonical_charge_and_isotope_facts() {
    let input = document(
        "<n id=\"a\" p=\"0 0\" Element=\"8\" Charge=\"-128\" Isotope=\"32767\"/><n id=\"b\" p=\"1 0\" Element=\"7\" Charge=\"127\" Isotope=\"1\"/>",
    );
    let decoded = decode_cdxml_bytes_v1(input.as_bytes()).expect("canonical scalar facts");
    let atoms = decoded.records()[0].record().molecule().atoms();
    assert_eq!(
        (atoms[0].formal_charge(), atoms[0].isotope()),
        (Some(-128), Some(32767))
    );
    assert_eq!(
        (atoms[1].formal_charge(), atoms[1].isotope()),
        (Some(127), Some(1))
    );
}

#[test]
fn cdxml_c2_normalizes_explicit_zero_scalar_facts() {
    let absent = decode_cdxml_bytes_v1(document("<n id=\"a\" p=\"0 0\"/>").as_bytes())
        .expect("absent scalar facts");
    let explicit_zero = decode_cdxml_bytes_v1(
        document("<n id=\"a\" p=\"0 0\" Charge=\"0\" Isotope=\"0\"/>").as_bytes(),
    )
    .expect("explicit neutral scalar facts");
    assert_eq!(
        absent.records()[0].record().molecule(),
        explicit_zero.records()[0].record().molecule()
    );
}

#[test]
fn cdxml_c2_refuses_noncanonical_and_out_of_range_scalar_facts() {
    for attributes in [
        "Charge=\"+1\"",
        "Charge=\"-0\"",
        "Charge=\"01\"",
        "Charge=\" 1\"",
        "Charge=\"1.5\"",
        "Charge=\"1e2\"",
        "Charge=\"128\"",
        "Charge=\"-129\"",
        "Isotope=\"+1\"",
        "Isotope=\"01\"",
        "Isotope=\" 1\"",
        "Isotope=\"1.5\"",
        "Isotope=\"1e2\"",
        "Isotope=\"-1\"",
        "Isotope=\"32768\"",
        "Isotope=\"999999999999999999999999999999999999999\"",
    ] {
        let input = document(&format!("<n id=\"a\" p=\"0 0\" {attributes}/>"));
        assert_refusal(input.as_bytes(), CdxmlRefusalReasonV1::InvalidScalar);
    }
}

#[test]
fn cdxml_c1_refuses_invalid_lexical_and_external_input_forms() {
    let internal_dtd = format!(
        "<!DOCTYPE CDXML SYSTEM \"{VENDOR_DTD}\" [<!ENTITY x \"N\">]>{}",
        document("<n id=\"a\" p=\"0 0\"/>")
    );
    let public_dtd = format!(
        "<!DOCTYPE CDXML PUBLIC \"id\" \"{VENDOR_DTD}\">{}",
        document("<n id=\"a\" p=\"0 0\"/>")
    );
    let after_root = format!(
        "{}<!DOCTYPE CDXML SYSTEM \"{VENDOR_DTD}\">",
        document("<n id=\"a\" p=\"0 0\"/>")
    );
    let declaration_after_root = format!(
        "{}<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
        document("<n id=\"a\" p=\"0 0\"/>")
    );
    let cases = [
        (b"\xff".as_slice(), CdxmlRefusalReasonV1::InvalidUtf8),
        (b"<CDXML>".as_slice(), CdxmlRefusalReasonV1::InvalidXml),
        (public_dtd.as_bytes(), CdxmlRefusalReasonV1::DtdForbidden),
        (internal_dtd.as_bytes(), CdxmlRefusalReasonV1::DtdForbidden),
        (b"<CDXML><xi:include href=\"file.xml\"/></CDXML>".as_slice(), CdxmlRefusalReasonV1::NamespaceUnsupported),
        (b"<?xml-stylesheet href=\"file.css\"?><CDXML><page><fragment id=\"f\"><n id=\"a\" p=\"0 0\"/></fragment></page></CDXML>".as_slice(), CdxmlRefusalReasonV1::UnrepresentedSemanticFact),
        (after_root.as_bytes(), CdxmlRefusalReasonV1::InvalidXml),
        (
            declaration_after_root.as_bytes(),
            CdxmlRefusalReasonV1::InvalidXml,
        ),
    ];
    for (input, reason) in cases {
        assert_refusal(input, reason);
    }
}

#[test]
fn cdxml_c1_validates_element_forms_bond_graph_and_fragment_ownership() {
    let agreement = document("<n id=\"a\" p=\"0 0\" Element=\"7\"><t><s>N</s></t></n>");
    assert_eq!(
        decode_cdxml_bytes_v1(agreement.as_bytes())
            .expect("matching numeric and text element")
            .records()[0]
            .record()
            .molecule()
            .atoms()[0]
            .atomic_number()
            .symbol(),
        "N"
    );
    let nested = document("<fragment id=\"nested\"><n id=\"a\" p=\"0 0\"/></fragment>");
    let cross_fragment = "<CDXML><page><fragment id=\"left\"><n id=\"a\" p=\"0 0\"/></fragment><fragment id=\"right\"><n id=\"b\" p=\"1 0\"/><b B=\"a\" E=\"b\"/></fragment></page></CDXML>";
    let cases = [
        (
            document("<n id=\"a\" p=\"0 0\" Element=\"7\"><t><s>O</s></t></n>"),
            CdxmlRefusalReasonV1::InvalidScalar,
        ),
        (
            two_atoms("<b B=\"a\" E=\"b\" Order=\"4\"/>"),
            CdxmlRefusalReasonV1::UnrepresentedSemanticFact,
        ),
        (
            two_atoms("<b B=\"a\" E=\"b\" Order=\"3\" Display=\"WedgeBegin\"/>"),
            CdxmlRefusalReasonV1::InvalidScalar,
        ),
        (nested, CdxmlRefusalReasonV1::UnrepresentedSemanticFact),
        (
            cross_fragment.to_owned(),
            CdxmlRefusalReasonV1::DanglingBond,
        ),
        (
            two_atoms("<b B=\"a\" E=\"missing\"/>"),
            CdxmlRefusalReasonV1::DanglingBond,
        ),
        (
            two_atoms("<b B=\"a\" E=\"a\"/>"),
            CdxmlRefusalReasonV1::SelfBond,
        ),
        (
            two_atoms("<b B=\"a\" E=\"b\"/><b B=\"b\" E=\"a\"/>"),
            CdxmlRefusalReasonV1::DuplicateBond,
        ),
    ];
    for (input, reason) in cases {
        assert_refusal(input.as_bytes(), reason);
    }
    let triple = two_atoms("<b B=\"a\" E=\"b\" Order=\"3\"/>");
    assert_eq!(
        decode_cdxml_bytes_v1(triple.as_bytes())
            .expect("triple bond")
            .records()[0]
            .record()
            .molecule()
            .bonds()[0]
            .order(),
        BondOrder::Triple
    );
}

#[test]
fn cdxml_c1_refuses_invalid_coordinate_and_bond_identifier_values() {
    let nonfinite = "9".repeat(400);
    let long_id = "b".repeat(129);
    let cases = [
        (
            document("<n id=\"a\" p=\"zero 0\"/>"),
            CdxmlRefusalReasonV1::InvalidCoordinate,
        ),
        (
            document(&format!("<n id=\"a\" p=\"{nonfinite} 0\"/>")),
            CdxmlRefusalReasonV1::CoordinateNotFinite,
        ),
        (
            document("<n id=\"a\" p=\"100001 0\"/>"),
            CdxmlRefusalReasonV1::CoordinateOutOfRange,
        ),
        (
            two_atoms("<b id=\"\" B=\"a\" E=\"b\"/>"),
            CdxmlRefusalReasonV1::InvalidScalar,
        ),
        (
            two_atoms(&format!("<b id=\"{long_id}\" B=\"a\" E=\"b\"/>")),
            CdxmlRefusalReasonV1::IdentifierBytesLimit,
        ),
    ];
    for (input, reason) in cases {
        assert_refusal(input.as_bytes(), reason);
    }
}

#[test]
fn cdxml_c1_accepts_and_refuses_input_byte_boundary() {
    let prefix = "<CDXML>";
    let suffix = "<page><fragment id=\"f\"><n id=\"a\" p=\"0 0\"/></fragment></page></CDXML>";
    let padding =
        " ".repeat(CDXML_SIMPLE_MOLECULE_IMPORT_MAX_SOURCE_BYTES_V1 - prefix.len() - suffix.len());
    let accepted = format!("{prefix}{padding}{suffix}");
    assert_eq!(
        accepted.len(),
        CDXML_SIMPLE_MOLECULE_IMPORT_MAX_SOURCE_BYTES_V1
    );
    assert!(decode_cdxml_bytes_v1(accepted.as_bytes()).is_ok());
    let refused = format!(" {accepted}");
    assert_eq!(
        refused.len(),
        CDXML_SIMPLE_MOLECULE_IMPORT_MAX_SOURCE_BYTES_V1 + 1
    );
    assert_refusal(refused.as_bytes(), CdxmlRefusalReasonV1::InputBytesLimit);
}

#[test]
fn cdxml_c1_accepts_exact_and_refuses_one_over_element_boundary() {
    const ELEMENT_LIMIT: usize = 50_000;
    const STRUCTURAL_ELEMENTS: usize = 4;

    fn complete_document(extra_pages: usize) -> String {
        let mut input = String::from(
            "<CDXML><page><fragment id=\"f\"><n id=\"a\" p=\"0 0\"/></fragment></page>",
        );
        for _ in 0..extra_pages {
            input.push_str("<page/>");
        }
        input.push_str("</CDXML>");
        input
    }

    let accepted = complete_document(ELEMENT_LIMIT - STRUCTURAL_ELEMENTS);
    assert!(decode_cdxml_bytes_v1(accepted.as_bytes()).is_ok());
    let refused = complete_document(ELEMENT_LIMIT + 1 - STRUCTURAL_ELEMENTS);
    assert_refusal(refused.as_bytes(), CdxmlRefusalReasonV1::XmlElementLimit);
}

#[test]
fn cdxml_c1_enforces_attribute_value_and_identifier_boundaries() {
    let id_at_limit = "f".repeat(128);
    let id_over_limit = "f".repeat(129);
    let attribute_at_limit = "x".repeat(1_024);
    let attribute_over_limit = "x".repeat(1_025);
    let accepted_id = format!(
        "<CDXML><page><fragment id=\"{id_at_limit}\"><n id=\"a\" p=\"0 0\"/></fragment></page></CDXML>"
    );
    let accepted_attribute = format!(
        "<CDXML Name=\"{attribute_at_limit}\"><page><fragment id=\"f\"><n id=\"a\" p=\"0 0\"/></fragment></page></CDXML>"
    );
    let accepted_bond_id = two_atoms(&format!("<b id=\"{id_at_limit}\" B=\"a\" E=\"b\"/>"));
    assert!(decode_cdxml_bytes_v1(accepted_id.as_bytes()).is_ok());
    assert!(decode_cdxml_bytes_v1(accepted_attribute.as_bytes()).is_ok());
    assert!(decode_cdxml_bytes_v1(accepted_bond_id.as_bytes()).is_ok());
    let cases = [
        (
            format!(
                "<CDXML><page><fragment id=\"{id_over_limit}\"><n id=\"a\" p=\"0 0\"/></fragment></page></CDXML>"
            ),
            CdxmlRefusalReasonV1::IdentifierBytesLimit,
        ),
        (
            format!(
                "<CDXML Name=\"{attribute_over_limit}\"><page><fragment id=\"f\"><n id=\"a\" p=\"0 0\"/></fragment></page></CDXML>"
            ),
            CdxmlRefusalReasonV1::AttributeValueLimit,
        ),
    ];
    for (input, reason) in cases {
        assert_refusal(input.as_bytes(), reason);
    }
}

#[test]
fn cdxml_c1_accepts_and_refuses_record_boundary() {
    assert!(decode_cdxml_bytes_v1(records_document(1_024).as_bytes()).is_ok());
    assert_refusal(
        records_document(1_025).as_bytes(),
        CdxmlRefusalReasonV1::RecordLimit,
    );
}

#[test]
fn cdxml_c1_accepts_and_refuses_atoms_per_record_boundary() {
    assert!(decode_cdxml_bytes_v1(atoms_document(10_000).as_bytes()).is_ok());
    assert_refusal(
        atoms_document(10_001).as_bytes(),
        CdxmlRefusalReasonV1::AtomsPerRecordLimit,
    );
}

#[test]
fn cdxml_c1_accepts_and_refuses_bonds_per_record_boundary() {
    assert!(decode_cdxml_bytes_v1(bonds_document(20_000).as_bytes()).is_ok());
    assert_refusal(
        bonds_document(20_001).as_bytes(),
        CdxmlRefusalReasonV1::BondsPerRecordLimit,
    );
}
