use crate::sugar::legacy_compact_v1::{
    LegacyCompactSugarCodeV1, LegacyCompactSugarRenderRequestV1,
};

#[test]
fn compact_v1_code_serde_revalidates_the_canonical_string() {
    let code = LegacyCompactSugarCodeV1::parse("A2LRDM[2R=CH3]").expect("valid compact v1 code");
    assert_eq!(
        serde_json::to_string(&code).expect("serialize validated code"),
        "\"A2LRDM[2L=H,2R=CH3]\""
    );
    let restored: LegacyCompactSugarCodeV1 =
        serde_json::from_str("\"A2LRDM[2R=CH3]\"").expect("deserialize valid compact v1 code");
    assert_eq!(restored, code);
}

#[test]
fn compact_v1_code_serde_rejects_unvalidated_or_contradictory_payloads() {
    let structural_payload =
        r#"{"body":"ARLRDM","prefix":"Keto","series":"Meso","positions":[],"footnotes":{}}"#;
    assert!(serde_json::from_str::<LegacyCompactSugarCodeV1>(structural_payload).is_err());
    assert!(serde_json::from_str::<LegacyCompactSugarCodeV1>("\"A2LRDM\"").is_err());
}

#[test]
fn compact_v1_render_request_serde_validates_its_nested_code() {
    let invalid_request = r#"{"code":"A2LRDM","ring":"Pyranose","anomer":"Beta"}"#;
    assert!(serde_json::from_str::<LegacyCompactSugarRenderRequestV1>(invalid_request).is_err());
}
