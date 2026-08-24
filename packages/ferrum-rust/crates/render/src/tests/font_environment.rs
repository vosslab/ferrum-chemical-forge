use crate::{
    AtomLabelFacts, AtomLabelFontProfile, FerrumFontEnvironmentV1, FontFace, GlyphMetrics, Paint,
    PositiveFinite, Rgb24, TextScript, VerifiedTelexGlyphMetrics,
};
use ferrum_render_contract::{TELEX_REGULAR_RESOURCE_ID_V1, TELEX_REGULAR_SHA256_V1};

fn paint() -> Paint {
    Paint::rgb24(Rgb24::new("000000").expect("test rgb"))
}
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

fn size(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("test size is finite and positive")
}

#[test]
fn bundled_telex_asset_matches_its_closed_resource_contract() {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let asset = environment.descriptor(crate::FerrumFontId::TelexRegular);
    assert_eq!(asset.id().resource_id(), TELEX_REGULAR_RESOURCE_ID_V1);
    assert_eq!(asset.bytes(), 38_940);
    assert_eq!(asset.sha256(), TELEX_REGULAR_SHA256_V1);
    assert_eq!(asset.family(), "Telex");
    assert_eq!(asset.postscript_name(), "Telex-Regular");
}

#[test]
fn bundled_telex_font_matches_the_shared_scalar_capability_table() {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    VerifiedTelexGlyphMetrics::new(&environment)
        .expect("bundled Telex must satisfy every shared scalar capability");
}

#[test]
fn verified_metrics_lay_out_the_verified_font_without_a_family_lookup() {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Telex");
    let font = AtomLabelFontProfile::new(FontFace::telex_regular(), size(12.0), paint());
    let label = AtomLabelFacts::new("Cl", 1, 2).expect("chemical label is valid");
    let layout = metrics
        .layout_atom_label(&label, &font)
        .expect("verified Telex lays out the label");
    assert_eq!(layout.runs().len(), 4);
    assert!(layout.bounds().min_x() < 0.0);
    assert!(layout.bounds().max_x() > 0.0);
}

#[test]
fn verified_metrics_center_the_closed_plus_glyph_without_frontend_advances() {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Telex");
    let layout = metrics
        .layout_centered_plus(size(18.0), paint())
        .expect("verified Telex lays out the fixed plus content");
    let operation = layout.operation();
    assert_eq!(operation.face(), &FontFace::telex_regular());
    let [run] = operation.runs() else {
        panic!("fixed plus layout has one exact run");
    };
    assert_eq!(run.text(), "+");
    assert_eq!(run.glyphs().len(), 1);
    assert_ne!(run.glyphs()[0].glyph_index(), 0);
    let bounds = layout.bounds();
    // Telex + has positive x bearing and ink below its baseline.  Centering the
    // true outline (90..557 x -71..542 design units) must not include the
    // operation origin in the rectangle.
    assert_eq!(
        (
            operation.origin().x(),
            operation.origin().y(),
            bounds.min_x(),
            bounds.min_y(),
            bounds.max_x(),
            bounds.max_y(),
        ),
        (
            -5.8229999999999995,
            5.517,
            -4.202999999999999,
            -4.239,
            4.203,
            4.239,
        )
    );
}

#[test]
fn true_ink_metrics_and_atom_label_clipping_envelope_have_distinct_contracts() {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Telex");
    let font = AtomLabelFontProfile::new(FontFace::telex_regular(), size(12.0), paint());

    let ink = metrics
        .measure_text_run("I", font.size(), size(1.0))
        .expect("Telex I is measurable");
    // The exact design outline is x=88..183 at 12 scene units per em.
    assert_eq!(
        (
            ink.x_bearing(),
            ink.y_bearing(),
            ink.width(),
            ink.height(),
            ink.x_advance()
        ),
        (1.056, -8.532, 1.1400000000000001, 8.532, 3.2520000000000002)
    );

    let label = metrics
        .layout_atom_label(
            &AtomLabelFacts::new("I", 0, 0).expect("valid iodine label"),
            &font,
        )
        .expect("iodine label is laid out");
    // Atom/bond clipping is the one explicit envelope that includes the anchor.
    assert_eq!(
        (
            label.bounds().min_x(),
            label.bounds().min_y(),
            label.bounds().max_x(),
            label.bounds().max_y(),
        ),
        (-0.5700000000000001, -8.532, 0.5700000000000001, 0.0)
    );
}

#[test]
fn wrong_file_cannot_pass_the_asset_verifier() {
    let wrong = std::env::temp_dir().join("ferrum-not-telex.ttf");
    std::fs::write(&wrong, b"not a font").expect("write isolated invalid font fixture");
    let result = FerrumFontEnvironmentV1::load_for_test(&wrong);
    std::fs::remove_file(&wrong).expect("remove isolated invalid font fixture");
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn symlinked_asset_component_cannot_pass_the_resource_verifier() {
    use std::os::unix::fs::symlink;

    let root = fixture_root().join(format!("ferrum-telex-symlink-{}", std::process::id()));
    let target = root.join("target");
    let linked = root.join("linked");
    std::fs::create_dir_all(&target).expect("create isolated target directory");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/Telex-Regular.ttf"
        ),
        target.join("Telex-Regular.ttf"),
    )
    .expect("copy verified Telex fixture");
    symlink(&target, &linked).expect("create isolated symlink component");

    let result = FerrumFontEnvironmentV1::load_for_test(&linked.join("Telex-Regular.ttf"));
    std::fs::remove_dir_all(&root).expect("remove isolated symlink fixture");
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn final_symlink_replacement_between_parent_and_final_open_is_rejected() {
    use std::os::unix::fs::symlink;

    let fixture = TreeFixture::telex();
    let font = fixture.font_path();
    let replacement = fixture.root().join("replacement.ttf");
    std::fs::write(&replacement, b"replacement bytes").expect("write replacement fixture");
    let font_for_hook = font.to_owned();
    let result = FerrumFontEnvironmentV1::load_for_test_with_after_parent_open(font, move || {
        std::fs::remove_file(&font_for_hook).expect("remove final fixture before open");
        symlink(&replacement, &font_for_hook).expect("replace final fixture with symlink");
    });
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn parent_replacement_after_descriptor_open_cannot_redirect_final_open() {
    use std::os::unix::fs::symlink;

    let fixture = TreeFixture::telex();
    let font = fixture.font_path();
    let parent = font.parent().expect("font has fixture parent").to_owned();
    let moved_parent = fixture.root().join("fonts-retained");
    let redirect = fixture.root().join("redirect");
    std::fs::create_dir(&redirect).expect("create isolated redirect directory");
    std::fs::write(redirect.join("Telex-Regular.ttf"), b"replacement bytes")
        .expect("write isolated redirect font");
    let result = FerrumFontEnvironmentV1::load_for_test_with_after_parent_open(font, move || {
        std::fs::rename(&parent, &moved_parent).expect("move visible parent after descriptor open");
        symlink(&redirect, &parent).expect("replace visible parent with a symlink");
    });
    assert!(result.is_ok());
}

#[test]
fn final_replacement_after_descriptor_open_cannot_change_same_handle_verification() {
    let fixture = TreeFixture::telex();
    let font = fixture.font_path();
    let replacement = fixture.root().join("replacement.ttf");
    std::fs::write(&replacement, b"replacement bytes").expect("write replacement fixture");
    let font_for_hook = font.to_owned();
    let environment =
        FerrumFontEnvironmentV1::load_for_test_with_after_final_open(font, move || {
            std::fs::rename(&replacement, &font_for_hook)
                .expect("replace visible final entry after descriptor open");
        })
        .expect("the retained final descriptor still reads and verifies the original Telex bytes");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)
        .expect("pure-Rust parser consumes immutable verified bytes");
    let font = AtomLabelFontProfile::new(FontFace::telex_regular(), size(12.0), paint());
    let label = AtomLabelFacts::new("Cl", 1, 2).expect("chemical label is valid");
    assert!(metrics.layout_atom_label(&label, &font).is_ok());
}

#[test]
fn verified_metrics_use_memory_after_the_asset_path_is_replaced() {
    let asset = PathFixture::telex_copy();
    let environment = FerrumFontEnvironmentV1::load_for_test(asset.path())
        .expect("copied Telex is verified into immutable resource bytes");
    std::fs::write(asset.path(), b"replacement bytes").expect("replace only the fixture pathname");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)
        .expect("pure-Rust parser receives retained verified bytes rather than reopening the path");
    let font = AtomLabelFontProfile::new(FontFace::telex_regular(), size(12.0), paint());
    let label = AtomLabelFacts::new("Cl", 1, 2).expect("chemical label is valid");
    assert!(metrics.layout_atom_label(&label, &font).is_ok());
}

#[test]
fn metric_receipt_rows_cover_the_complete_pre_tolerance_corpus() {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Telex");
    let font = AtomLabelFontProfile::new(FontFace::telex_regular(), size(12.0), paint());
    let baseline = metrics
        .baseline_metrics(font.size())
        .expect("Telex baseline metrics are finite");
    for (label, facts) in receipt_corpus() {
        let layout = metrics
            .layout_atom_label(&facts, &font)
            .expect("Telex lays out each receipt corpus label");
        let runs: Vec<serde_json::Value> = layout
            .runs()
            .iter()
            .map(|run| {
                let extents = metrics
                    .measure_text_run(run.text(), font.size(), run.scale())
                    .expect("each rendered receipt run is measurable");
                serde_json::json!({
                    "text": run.text(),
                    "script": match run.script() {
                        TextScript::Baseline => "baseline",
                        TextScript::Subscript => "subscript",
                        TextScript::Superscript => "superscript",
                    },
                    "size": font.size().get(),
                    "scale": run.scale().get(),
                    "origin": {"x": run.origin().x(), "y": run.origin().y()},
                    "glyphs": run.glyphs().iter().map(|glyph| serde_json::json!({
                        "id": glyph.glyph_index(),
                        "origin": {"x": glyph.origin().x(), "y": glyph.origin().y()},
                    })).collect::<Vec<_>>(),
                    "bearing": {"x": extents.x_bearing(), "y": extents.y_bearing()},
                    "width": extents.width(),
                    "height": extents.height(),
                    "advance": {"x": extents.x_advance(), "y": extents.y_advance()},
                })
            })
            .collect();
        let bounds = layout.bounds();
        println!(
            "M12_METRIC_JSONL:{}",
            serde_json::json!({
                "schema": "ferrum-m12-truetype-design-metric-row-v1",
                "label": label,
                "font": {
                    "id": "ferrum-telex-regular-v1",
                    "bytes": environment.descriptor(crate::FerrumFontId::TelexRegular).bytes(),
                    "sha256": environment.descriptor(crate::FerrumFontId::TelexRegular).sha256(),
                    "family": environment.descriptor(crate::FerrumFontId::TelexRegular).family(),
                    "postscript_name": environment.descriptor(crate::FerrumFontId::TelexRegular).postscript_name(),
                    "load_flags": ["no_scale", "no_hinting", "no_bitmap"],
                    "units_per_em": 1000,
                    "kerning": false,
                },
                "baseline": {
                    "ascent": baseline.ascent(),
                    "descent": baseline.descent(),
                    "height": baseline.height(),
                },
                "final_bounds": {
                    "min_x": bounds.min_x(), "min_y": bounds.min_y(),
                    "max_x": bounds.max_x(), "max_y": bounds.max_y(),
                },
                "runs": runs,
                "native_libraries": {
                    "metric_backend": "ttf-parser-design-v1",
                },
            })
        );
    }
}

#[test]
fn truetype_design_metrics_define_the_canonical_v1_corpus_receipt_exactly() {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Telex");
    let font = AtomLabelFontProfile::new(FontFace::telex_regular(), size(12.0), paint());
    let baseline = metrics
        .baseline_metrics(font.size())
        .expect("Telex baseline is measurable");
    assert_eq!(
        (baseline.ascent(), baseline.descent(), baseline.height()),
        (11.34, 3.12, 14.46)
    );

    assert_receipt_label(
        &metrics,
        &font,
        AtomLabelFacts::new("C", 0, 0).expect("valid corpus label"),
        (
            -3.2159999999999997,
            -8.591999999999999,
            3.336000000000001,
            0.10800000000000054,
        ),
        &[ExpectedRun::baseline(
            "C",
            -3.816,
            13,
            7.632,
            6.552000000000001,
            8.7,
        )],
    );
    assert_receipt_label(
        &metrics,
        &font,
        AtomLabelFacts::new("Cl", 0, 0).expect("valid corpus label"),
        (-4.800000000000001, -9.108, 4.368, 0.10800000000000054),
        &[
            ExpectedRun::baseline("Cl", -5.4, 13, 10.8, 9.168000000000001, 9.216000000000001)
                .with_second_glyph(108, 7.632),
        ],
    );
    assert_receipt_label(
        &metrics,
        &font,
        AtomLabelFacts::new("Br", 0, 0).expect("valid corpus label"),
        (-5.022, -8.495999999999999, 5.862, 0.0),
        &[
            ExpectedRun::baseline("Br", -6.042, 12, 12.084, 10.884, 8.495999999999999)
                .with_second_glyph(126, 7.343999999999999),
        ],
    );
    assert_receipt_label(
        &metrics,
        &font,
        AtomLabelFacts::new("H", 0, 2).expect("valid corpus label"),
        (-9.9573, -8.495999999999999, 10.509299999999998, 0.0),
        &[
            ExpectedRun::baseline(
                "H",
                -10.9773,
                24,
                8.687999999999999,
                6.6480000000000015,
                8.495999999999999,
            ),
            ExpectedRun::baseline(
                "H",
                -2.289300000000001,
                24,
                8.687999999999999,
                6.6480000000000015,
                8.495999999999999,
            ),
            ExpectedRun::subscript(
                "2",
                6.398699999999998,
                -2.4960000000000004,
                156,
                4.5786,
                3.7205999999999997,
                5.5847999999999995,
            ),
        ],
    );
    assert_receipt_label(
        &metrics,
        &font,
        AtomLabelFacts::new("N", 1, 3).expect("valid corpus label"),
        (
            -12.312299999999999,
            -8.495999999999999,
            12.660899999999998,
            5.683199999999999,
        ),
        &[
            ExpectedRun::baseline(
                "N",
                -13.3083,
                39,
                8.436,
                6.444000000000001,
                8.495999999999999,
            ),
            ExpectedRun::baseline(
                "H",
                -4.872299999999999,
                24,
                8.687999999999999,
                6.6480000000000015,
                8.495999999999999,
            ),
            ExpectedRun::subscript(
                "3",
                3.8156999999999996,
                -2.4960000000000004,
                157,
                4.5005999999999995,
                3.7128,
                5.6628,
            ),
            ExpectedRun::superscript(
                "+",
                8.316299999999998,
                6.237,
                217,
                4.992,
                3.6426000000000007,
                3.6737999999999995,
            ),
        ],
    );
    assert_receipt_label(
        &metrics,
        &font,
        AtomLabelFacts::new("I", 0, 0).expect("valid corpus label"),
        (-0.5700000000000001, -8.532, 0.5700000000000001, 0.0),
        &[ExpectedRun::baseline(
            "I",
            -1.6260000000000001,
            25,
            3.2520000000000002,
            1.1400000000000001,
            8.532,
        )],
    );
}

#[test]
fn verified_metrics_reject_controls_and_missing_scalars_without_fallback() {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Telex");
    let result = metrics.v1_glyphs_for_run("C\n", size(12.0), size(1.0));
    assert!(result.is_err());
    let result = metrics.v1_glyphs_for_run("\u{10ffff}", size(12.0), size(1.0));
    assert!(result.is_err());
}

struct ExpectedRun {
    text: &'static str,
    script: TextScript,
    scale: f64,
    origin: (f64, f64),
    glyphs: Vec<(u32, f64)>,
    advance: f64,
    width: f64,
    height: f64,
}

impl ExpectedRun {
    fn baseline(
        text: &'static str,
        origin_x: f64,
        glyph: u32,
        advance: f64,
        width: f64,
        height: f64,
    ) -> Self {
        Self {
            text,
            script: TextScript::Baseline,
            scale: 1.0,
            origin: (origin_x, 0.0),
            glyphs: vec![(glyph, 0.0)],
            advance,
            width,
            height,
        }
    }

    fn subscript(
        text: &'static str,
        origin_x: f64,
        origin_y: f64,
        glyph: u32,
        advance: f64,
        width: f64,
        height: f64,
    ) -> Self {
        Self {
            text,
            script: TextScript::Subscript,
            scale: 0.65,
            origin: (origin_x, origin_y),
            glyphs: vec![(glyph, 0.0)],
            advance,
            width,
            height,
        }
    }

    fn superscript(
        text: &'static str,
        origin_x: f64,
        origin_y: f64,
        glyph: u32,
        advance: f64,
        width: f64,
        height: f64,
    ) -> Self {
        Self {
            text,
            script: TextScript::Superscript,
            scale: 0.65,
            origin: (origin_x, origin_y),
            glyphs: vec![(glyph, 0.0)],
            advance,
            width,
            height,
        }
    }

    fn with_second_glyph(mut self, glyph: u32, origin_x: f64) -> Self {
        assert_eq!(self.glyphs.len(), 1);
        self.glyphs.push((glyph, origin_x));
        self
    }
}

fn assert_receipt_label(
    metrics: &VerifiedTelexGlyphMetrics,
    font: &AtomLabelFontProfile,
    label: AtomLabelFacts,
    expected_bounds: (f64, f64, f64, f64),
    expected_runs: &[ExpectedRun],
) {
    let layout = metrics
        .layout_atom_label(&label, font)
        .expect("verified corpus label is laid out");
    assert_eq!(
        (
            layout.bounds().min_x(),
            layout.bounds().min_y(),
            layout.bounds().max_x(),
            layout.bounds().max_y(),
        ),
        expected_bounds
    );
    assert_eq!(layout.runs().len(), expected_runs.len());
    for (actual, expected) in layout.runs().iter().zip(expected_runs) {
        assert_eq!(actual.text(), expected.text);
        assert_eq!(actual.script(), expected.script);
        assert_eq!(actual.scale().get(), expected.scale);
        assert_eq!((actual.origin().x(), actual.origin().y()), expected.origin);
        let glyphs: Vec<(u32, f64, f64)> = actual
            .glyphs()
            .iter()
            .map(|glyph| (glyph.glyph_index(), glyph.origin().x(), glyph.origin().y()))
            .collect();
        let expected_glyphs: Vec<(u32, f64, f64)> = expected
            .glyphs
            .iter()
            .map(|(index, origin_x)| (*index, *origin_x, 0.0))
            .collect();
        assert_eq!(glyphs, expected_glyphs);
        let run = metrics
            .measure_text_run(actual.text(), font.size(), actual.scale())
            .expect("receipt run is measurable");
        assert_eq!(
            (run.x_bearing(), run.y_bearing()),
            (
                expected_x_bearing(actual.text()),
                expected_y_bearing(actual.text()),
            )
        );
        assert_eq!(
            (run.width(), run.height(), run.x_advance(), run.y_advance()),
            (expected.width, expected.height, expected.advance, 0.0)
        );
    }
}

fn expected_y_bearing(text: &str) -> f64 {
    match text {
        "C" => -8.591999999999999,
        "Cl" => -9.108,
        "Br" | "H" | "N" => -8.495999999999999,
        "2" | "3" => -5.5847999999999995,
        "+" => -4.227600000000001,
        "I" => -8.532,
        _ => panic!("unexpected V1 receipt run"),
    }
}

fn expected_x_bearing(text: &str) -> f64 {
    match text {
        "C" | "Cl" => 0.6000000000000001,
        "Br" | "H" => 1.02,
        "N" => 0.996,
        "2" => 0.39000000000000007,
        "3" => 0.3276,
        "+" => 0.7020000000000001,
        "I" => 1.056,
        _ => panic!("unexpected V1 receipt run"),
    }
}

fn receipt_corpus() -> [(&'static str, AtomLabelFacts); 6] {
    [
        (
            "C",
            AtomLabelFacts::new("C", 0, 0).expect("valid corpus label"),
        ),
        (
            "Cl",
            AtomLabelFacts::new("Cl", 0, 0).expect("valid corpus label"),
        ),
        (
            "Br",
            AtomLabelFacts::new("Br", 0, 0).expect("valid corpus label"),
        ),
        (
            "H2",
            AtomLabelFacts::new("H", 0, 2).expect("valid corpus label"),
        ),
        (
            "NH3+",
            AtomLabelFacts::new("N", 1, 3).expect("valid corpus label"),
        ),
        (
            "I",
            AtomLabelFacts::new("I", 0, 0).expect("valid corpus label"),
        ),
    ]
}

struct PathFixture {
    path: std::path::PathBuf,
}

struct TreeFixture {
    root: std::path::PathBuf,
    font: std::path::PathBuf,
}

impl TreeFixture {
    fn telex() -> Self {
        let root = fixture_root().join(format!(
            "ferrum-telex-tree-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        let fonts = root.join("package/fonts");
        std::fs::create_dir_all(&fonts).expect("create isolated Telex tree");
        let font = fonts.join("Telex-Regular.ttf");
        std::fs::copy(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/fonts/Telex-Regular.ttf"
            ),
            &font,
        )
        .expect("copy verified Telex fixture");
        Self { root, font }
    }

    fn root(&self) -> &std::path::Path {
        &self.root
    }

    fn font_path(&self) -> &std::path::Path {
        &self.font
    }
}

impl Drop for TreeFixture {
    fn drop(&mut self) {
        let _result = std::fs::remove_dir_all(&self.root);
    }
}

impl PathFixture {
    fn telex_copy() -> Self {
        let path = fixture_root().join(format!(
            "ferrum-telex-copy-{}-{}.ttf",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::copy(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/fonts/Telex-Regular.ttf"
            ),
            &path,
        )
        .expect("copy verified Telex fixture");
        Self { path }
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

fn fixture_root() -> std::path::PathBuf {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    std::fs::create_dir_all(&root).expect("create workspace target fixture directory");
    root.canonicalize()
        .expect("canonical workspace target fixture directory")
}

impl Drop for PathFixture {
    fn drop(&mut self) {
        let _result = std::fs::remove_file(&self.path);
    }
}
