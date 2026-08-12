use crate::{
    AtomLabelFacts, AtomLabelFontProfile, CairoGlyphMetrics, FerrumFontEnvironmentV1, FontFace,
    GlyphMetrics, Paint, PositiveFinite,
};

fn size(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("test size is finite and positive")
}

#[test]
fn bundled_telex_asset_matches_its_closed_resource_contract() {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let asset = environment.descriptor(crate::FerrumFontId::TelexRegular);
    assert_eq!(asset.id().resource_id(), "ferrum-telex-regular-v1");
    assert_eq!(asset.bytes(), 38_940);
    assert_eq!(
        asset.sha256(),
        "eeaa2d17d105b6b46e5368ecd990f5b19c50131ff922dbf79bfb9bb45c249871"
    );
    assert!(asset.path().is_absolute());
}

#[test]
fn cairo_metrics_lay_out_the_verified_font_without_a_family_lookup() {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let metrics = CairoGlyphMetrics::new(&environment).expect("Cairo opens verified Telex");
    let font = AtomLabelFontProfile::new(FontFace::telex_regular(), size(12.0), Paint::Foreground);
    let label = AtomLabelFacts::new("Cl", 1, 2).expect("chemical label is valid");
    let layout = metrics
        .layout_atom_label(&label, &font)
        .expect("verified Telex lays out the label");
    assert_eq!(layout.runs().len(), 4);
    assert!(layout.bounds().min_x() < 0.0);
    assert!(layout.bounds().max_x() > 0.0);
}

#[test]
fn wrong_file_cannot_pass_the_asset_verifier() {
    let wrong = std::env::temp_dir().join("ferrum-not-telex.ttf");
    std::fs::write(&wrong, b"not a font").expect("write isolated invalid font fixture");
    let result = FerrumFontEnvironmentV1::load_for_test(&wrong);
    std::fs::remove_file(&wrong).expect("remove isolated invalid font fixture");
    assert!(result.is_err());
}
