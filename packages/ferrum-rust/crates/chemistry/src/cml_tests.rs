use super::*;

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
