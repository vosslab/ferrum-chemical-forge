use crate::sugar::legacy_compact_v1::LegacyCompactSugarCodeV1;

#[test]
fn rejects_missing_marker_footnotes() {
    let error = LegacyCompactSugarCodeV1::parse("A2LRDM").expect_err("marker must be declared");
    assert!(error.to_string().contains("every position marker"));
}

#[test]
fn rejects_mixed_footnote_families() {
    let error = LegacyCompactSugarCodeV1::parse("A2LRDM[2C=CH2,2L=OH]")
        .expect_err("carbon-state and side declarations conflict");
    assert!(error.to_string().contains("cannot mix"));
}

#[test]
fn rejects_a_plain_declaration_at_a_chiral_marker() {
    let error =
        LegacyCompactSugarCodeV1::parse("A2LRDM[2=CH3]").expect_err("chiral marker needs a side");
    assert!(error.to_string().contains("is chiral"));
}
