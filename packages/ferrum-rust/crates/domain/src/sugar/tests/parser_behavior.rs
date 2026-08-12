use crate::sugar::legacy_compact_v1::{
    Anomer, FootnoteFamily, FootnoteKey, LegacyCompactSugarCodeV1,
    LegacyCompactSugarRenderRequestV1, RingForm, SugarPrefix, SugarSeries,
};

#[test]
fn parses_a_typed_d_aldose() {
    let code = LegacyCompactSugarCodeV1::parse("ARLRDM").expect("valid compact aldose code");
    assert_eq!(code.prefix(), SugarPrefix::Aldo);
    assert_eq!(code.series(), SugarSeries::D);
    assert_eq!(code.positions().len(), 6);
    assert_eq!(code.canonical_code(), "ARLRDM");
}

#[test]
fn normalizes_a_missing_side_to_hydrogen() {
    let code = LegacyCompactSugarCodeV1::parse("A2LRDM[2R=CH3]").expect("valid side declaration");
    assert_eq!(
        code.footnotes().get(&FootnoteKey {
            position: 2,
            family: FootnoteFamily::Left,
        }),
        Some(&"H".to_owned())
    );
    assert_eq!(code.canonical_code(), "A2LRDM[2L=H,2R=CH3]");
}

#[test]
fn parses_a_meso_ketose_without_inventing_a_series_marker() {
    let code = LegacyCompactSugarCodeV1::parse("MKp").expect("valid meso ketose code");
    assert_eq!(code.prefix(), SugarPrefix::Keto);
    assert_eq!(code.series(), SugarSeries::Meso);
}

#[test]
fn retains_parenthesized_footnote_values() {
    let code = LegacyCompactSugarCodeV1::parse("c23[2C=C3(EPO3),3C=CH2]")
        .expect("valid carbon-state values");
    assert_eq!(code.series(), SugarSeries::Meso);
    assert_eq!(code.canonical_code(), "c23[2C=C3(EPO3),3C=CH2]");
}

#[test]
fn keeps_rendering_request_as_a_non_codec_contract() {
    let code = LegacyCompactSugarCodeV1::parse("ARLRDM").expect("valid compact aldose code");
    let request = LegacyCompactSugarRenderRequestV1::new(code, RingForm::Pyranose, Anomer::Beta);
    assert_eq!(request.ring, RingForm::Pyranose);
    assert_eq!(request.anomer, Anomer::Beta);
}
