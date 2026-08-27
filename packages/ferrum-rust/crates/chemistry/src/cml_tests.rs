use super::*;
use crate::{BondDirection, Coordinates, InterchangeRecordV1, MolAtom, MolBond, MolGraph, Point2};

const CML1: &str = r#"<?xml version="1.0" encoding="UTF-8"?><cml xmlns="http://www.xml-cml.org/schema"><molecule id="m1"><atomArray><atom><builtin builtin="atomId">a1</builtin><builtin builtin="elementType">C</builtin><builtin builtin="x2">1.5</builtin><builtin builtin="y2">-2</builtin></atom><atom><builtin builtin="atomId">a2</builtin><builtin builtin="elementType">O</builtin><builtin builtin="x2">2</builtin><builtin builtin="y2">-2</builtin><builtin builtin="isotopeNumber">18</builtin></atom></atomArray><bondArray><bond><builtin builtin="atomRef">a1</builtin><builtin builtin="atomRef">a2</builtin><builtin builtin="order">D</builtin></bond></bondArray></molecule></cml>"#;
const CML2: &str = r#"<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule><atomArray><atom id="a1" elementType="C" x2="0" y2="0" formalCharge="0"/><atom id="a2" elementType="O" x2="1" y2="0" isotopeNumber="18"/></atomArray><bondArray><bond atomRefs2="a1 a2" order="2"/></bondArray></molecule></cml>"#;

#[test]
fn cml1_builtin_profile_decodes_to_owned_source_graph() {
    let decoded = decode_cml_bytes_v1(CML1.as_bytes()).expect("CML1 profile is admitted");
    let record = &decoded.records()[0];
    assert_eq!(record.source_molecule_id(), Some("m1"));
    assert_eq!(record.atoms()[0].element().symbol(), "C");
    assert_eq!(record.atoms()[0].x2(), 1.5);
    assert_eq!(record.atoms()[1].isotope(), Some(18));
    assert_eq!(record.bonds()[0].order(), BondOrder::Double);
}

#[test]
fn cml2_attribute_profile_decodes_without_coordinate_conversion() {
    let decoded = decode_cml_bytes_v1(CML2.as_bytes()).expect("CML2 profile is admitted");
    let record = &decoded.records()[0];
    assert_eq!(record.atoms()[1].isotope(), Some(18));
    assert_eq!((record.atoms()[1].x2(), record.atoms()[1].y2()), (1.0, 0.0));
    assert_eq!(record.bonds()[0].order(), BondOrder::Double);
}

#[test]
fn cml1_wedge_and_cml2_hash_preserve_directed_source_endpoints() {
    let cml1_wedge = CML1.replace(
        "<builtin builtin=\"order\">D</builtin>",
        "<builtin builtin=\"order\">S</builtin><builtin builtin=\"stereo\">W</builtin>",
    );
    let cml2_hash = r#"<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule><atomArray><atom id="a1" elementType="C" x2="0" y2="0"/><atom id="a2" elementType="O" x2="1" y2="0"/></atomArray><bondArray><bond atomRefs2="a2 a1" order="1"><stereo>H</stereo></bond></bondArray></molecule></cml>"#;

    let cml1 = decode_cml_bytes_v1(cml1_wedge.as_bytes()).expect("CML1 wedge is admitted");
    let cml2 = decode_cml_bytes_v1(cml2_hash.as_bytes()).expect("CML2 hash is admitted");
    assert_eq!(
        cml1.records()[0].bonds()[0].direction(),
        Some(BondDirection::BeginWedge)
    );
    assert_eq!(
        cml2.records()[0].bonds()[0].direction(),
        Some(BondDirection::BeginDash)
    );
    assert_eq!(
        (
            cml2.records()[0].bonds()[0].start(),
            cml2.records()[0].bonds()[0].end()
        ),
        (1, 0)
    );
    let lowered_wedge = cml1.records()[0].bonds()[0]
        .to_mol_bond()
        .expect("parsed CML wedge lowers through the public model constructor");
    assert_eq!((lowered_wedge.start(), lowered_wedge.end()), (0, 1));
    assert_eq!(lowered_wedge.direction(), BondDirection::BeginWedge);
    let encoded = encode_cml_decoded_document_v1(&cml1).expect("directed source bond encodes");
    assert_eq!(
        decode_cml_bytes_v1(encoded.as_bytes()).expect("encoded wedge reimports"),
        cml1
    );
}

#[test]
fn cml_stereo_refuses_unsupported_duplicate_and_non_single_declarations() {
    let cases = [
        (
            r#"<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule><atomArray><atom id="a1" elementType="C" x2="0" y2="0"/><atom id="a2" elementType="O" x2="1" y2="0"/></atomArray><bondArray><bond atomRefs2="a1 a2" order="1"><stereo>Z</stereo></bond></bondArray></molecule></cml>"#,
            "unsupported stereo scalar",
        ),
        (
            r#"<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule><atomArray><atom id="a1" elementType="C" x2="0" y2="0"/><atom id="a2" elementType="O" x2="1" y2="0"/></atomArray><bondArray><bond atomRefs2="a1 a2" order="1"><stereo>W</stereo><stereo>H</stereo></bond></bondArray></molecule></cml>"#,
            "conflicting stereo declarations",
        ),
        (
            r#"<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule><atomArray><atom id="a1" elementType="C" x2="0" y2="0"/><atom id="a2" elementType="O" x2="1" y2="0"/></atomArray><bondArray><bond atomRefs2="a1 a2" order="2"><stereo>W</stereo></bond></bondArray></molecule></cml>"#,
            "non-single directed bond",
        ),
    ];
    for (input, description) in cases {
        assert_eq!(
            decode_cml_bytes_v1(input.as_bytes())
                .expect_err(description)
                .reason(),
            CmlRefusalReasonV1::InvalidScalar
        );
    }
}

#[test]
fn reversed_cml2_bond_endpoints_refuse_as_a_duplicate() {
    let input = CML2.replace(
        "</bondArray>",
        "<bond atomRefs2=\"a2 a1\" order=\"2\"/></bondArray>",
    );

    assert_eq!(
        decode_cml_bytes_v1(input.as_bytes())
            .expect_err("reversed endpoint pairs must remain duplicate bonds")
            .reason(),
        CmlRefusalReasonV1::DuplicateBond
    );
}

#[test]
fn high_cardinality_distinct_bonds_preserve_source_order() {
    const BOND_COUNT: usize = 4_096;
    let mut input = String::from(
        "<cml xmlns=\"http://www.xml-cml.org/schema/cml2/core\"><molecule><atomArray>\
         <atom id=\"center\" elementType=\"C\" x2=\"0\" y2=\"0\"/>",
    );
    for index in 0..BOND_COUNT {
        input.push_str(&format!(
            "<atom id=\"a{index}\" elementType=\"H\" x2=\"{index}\" y2=\"1\"/>"
        ));
    }
    input.push_str("</atomArray><bondArray>");
    for index in 0..BOND_COUNT {
        input.push_str(&format!(
            "<bond atomRefs2=\"center a{index}\" order=\"1\"/>"
        ));
    }
    input.push_str("</bondArray></molecule></cml>");

    let decoded = decode_cml_bytes_v1(input.as_bytes())
        .expect("bounded distinct endpoint pairs are admitted");
    let record = &decoded.records()[0];
    assert_eq!(record.bonds().len(), BOND_COUNT);
    assert_eq!(record.bonds()[BOND_COUNT - 1].end(), BOND_COUNT);
}

#[test]
fn semantic_and_security_extensions_have_closed_refusals() {
    let oversized_id = "a".repeat(257);
    let cases = [
        (
            CML2.replace("<atomArray>", "<atomArray atomID=\"a1\">"),
            CmlRefusalReasonV1::ArrayAttributeUnsupported,
        ),
        (
            "<!DOCTYPE cml><cml xmlns=\"http://www.xml-cml.org/schema/cml2/core\"></cml>"
                .to_owned(),
            CmlRefusalReasonV1::DtdForbidden,
        ),
        (
            concat!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
                "<cml xmlns=\"http://www.xml-cml.org/schema/cml2/core\"></cml>",
            )
            .to_owned(),
            CmlRefusalReasonV1::InvalidXmlDeclaration,
        ),
        (
            CML2.replace("<molecule>", "<molecule><property/>"),
            CmlRefusalReasonV1::UnrepresentedSemanticFact,
        ),
        (
            CML2.replace("<cml ", "<x:cml "),
            CmlRefusalReasonV1::NamespaceUnsupported,
        ),
        (
            CML2.replace(
                "<atom id=\"a1\" elementType=\"C\" x2=\"0\" y2=\"0\" formalCharge=\"0\"/>",
                "<atom id=\"a1\" elementType=\"C\" x2=\"0\" y2=\"0\" formalCharge=\"0\"><!--note--></atom>",
            ),
            CmlRefusalReasonV1::UnexpectedXmlNode,
        ),
        (
            CML2.replace(
                "<atom id=\"a1\" elementType=\"C\" x2=\"0\" y2=\"0\" formalCharge=\"0\"/>",
                "<atom id=\"a1\" elementType=\"C\" x2=\"0\" y2=\"0\">text</atom>",
            ),
            CmlRefusalReasonV1::UnexpectedXmlText,
        ),
        (
            CML2.replace("id=\"a1\"", &format!("id=\"{oversized_id}\"")),
            CmlRefusalReasonV1::AttributeValueLimit,
        ),
    ];
    for (input, reason) in cases {
        assert_eq!(
            decode_cml_bytes_v1(input.as_bytes())
                .expect_err("closed CML profile must refuse unsupported input")
                .reason(),
            reason
        );
    }
}

#[test]
fn cml2_atom_and_bond_children_refuse_before_cml1_builtin_parsing() {
    let atom_child = CML2.replace(
        "<atom id=\"a1\" elementType=\"C\" x2=\"0\" y2=\"0\" formalCharge=\"0\"/>",
        "<atom id=\"a1\" elementType=\"C\" x2=\"0\" y2=\"0\" formalCharge=\"0\"><builtin builtin=\"atomId\">a1</builtin></atom>",
    );
    let bond_child = CML2.replace(
        "<bond atomRefs2=\"a1 a2\" order=\"2\"/>",
        "<bond atomRefs2=\"a1 a2\" order=\"2\"><builtin builtin=\"order\">2</builtin></bond>",
    );
    for input in [atom_child, bond_child] {
        assert_eq!(
            decode_cml_bytes_v1(input.as_bytes())
                .expect_err("CML2 atom and bond children are unrepresented semantic facts")
                .reason(),
            CmlRefusalReasonV1::UnrepresentedSemanticFact
        );
    }
}

#[test]
fn invalid_utf8_and_input_budget_refuse_before_graph_admission() {
    assert_eq!(
        decode_cml_bytes_v1(&[0xff])
            .expect_err("UTF-8 is required")
            .reason(),
        CmlRefusalReasonV1::InvalidUtf8
    );
    let oversized = vec![b' '; 1_048_577];
    assert_eq!(
        decode_cml_bytes_v1(&oversized)
            .expect_err("input budget is enforced")
            .reason(),
        CmlRefusalReasonV1::InputBytesLimit
    );
}

#[test]
fn canonical_cml2_round_trips_two_decoded_records_and_preserves_source_ids() {
    let input = r#"<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule id="first"><atomArray><atom id="carbon" elementType="C" x2="0.3333333333333333" y2="-0.1"/><atom id="oxygen" elementType="O" x2="1.25" y2="-0.1" formalCharge="-1" isotopeNumber="18"/></atomArray><bondArray><bond atomRefs2="carbon oxygen" order="2"/></bondArray></molecule><molecule id="second"><atomArray><atom id="nitrogen" elementType="N" x2="-2.5" y2="4.75"/></atomArray></molecule></cml>"#;
    let decoded = decode_cml_bytes_v1(input.as_bytes()).expect("input CML2");
    let encoded = encode_cml_decoded_document_v1(&decoded).expect("canonical CML2 output");
    let round_tripped = decode_cml_bytes_v1(encoded.as_bytes()).expect("encoded CML2 reimports");

    assert!(encoded.starts_with("<cml xmlns=\"http://www.xml-cml.org/schema/cml2/core\">"));
    assert_eq!(round_tripped, decoded);
}

#[test]
fn generic_records_use_local_ids_and_the_document_coordinate_inverse() {
    let atoms = vec![
        MolAtom::new(
            AtomicNumber::try_from(6).expect("carbon"),
            None,
            None,
            None,
            false,
        )
        .expect("carbon atom"),
        MolAtom::new(
            AtomicNumber::try_from(8).expect("oxygen"),
            Some(-1),
            Some(18),
            None,
            false,
        )
        .expect("oxygen atom"),
    ];
    let graph = MolGraph::new(
        atoms,
        vec![MolBond::new(0, 1, BondOrder::Double, false)],
        Some(Coordinates::new(vec![
            Point2::new(10.0, -3.0).expect("finite coordinate"),
            Point2::new(37.5, 6.0).expect("finite coordinate"),
        ])),
    )
    .expect("complete graph");
    let records = vec![InterchangeRecordV1::new(graph, None, Vec::new())];

    let output = encode_cml_interchange_records_v1(&records).expect("representable CML2");
    let decoded = decode_cml_bytes_v1(output.as_bytes()).expect("CML2 output reimports");
    let atoms = decoded.records()[0].atoms();

    assert_eq!((atoms[0].x2(), atoms[0].y2()), (1.0 / 3.0, 0.1));
    assert_eq!((atoms[1].x2(), atoms[1].y2()), (1.25, -0.2));
}

#[test]
fn generic_cml_refuses_metadata_that_has_no_closed_profile_representation() {
    let graph = MolGraph::new(
        vec![
            MolAtom::new(
                AtomicNumber::try_from(6).expect("carbon"),
                None,
                None,
                None,
                false,
            )
            .expect("atom"),
        ],
        Vec::new(),
        Some(Coordinates::new(vec![
            Point2::new(0.0, 0.0).expect("finite coordinate"),
        ])),
    )
    .expect("graph");
    let titled = vec![InterchangeRecordV1::new(
        graph,
        Some("not a CML chemistry fact".to_owned()),
        Vec::new(),
    )];

    assert_eq!(
        encode_cml_interchange_records_v1(&titled)
            .expect_err("titles are not representable")
            .reason(),
        CmlEncoderRefusalReasonV1::TitleUnsupported,
    );
}
