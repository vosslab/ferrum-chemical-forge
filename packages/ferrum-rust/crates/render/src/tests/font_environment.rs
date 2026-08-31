use crate::glyph_metrics::GlyphMetrics;
use crate::{
    AtomLabelFacts, AtomLabelFontProfile, FerrumFontEnvironment, FontFace, PositiveFinite,
    RenderPaintV3, Rgb24, TextScript, VerifiedMoleculeLabelGlyphMetrics,
};
use ferrum_render_contract::{MOLECULE_LABEL_RESOURCE_ID, MOLECULE_LABEL_SHA256};
use std::sync::atomic::{AtomicU64, Ordering};

fn paint() -> RenderPaintV3 {
    RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test rgb"))
}

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

fn size(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("test size is finite and positive")
}

#[test]
fn bundled_molecule_label_asset_matches_its_closed_resource_contract() {
    let environment =
        FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
    let asset = environment.molecule_label();
    assert_eq!(asset.resource_id(), MOLECULE_LABEL_RESOURCE_ID);
    assert_eq!(asset.bytes(), 65_068);
    assert_eq!(asset.sha256(), MOLECULE_LABEL_SHA256);
    assert_eq!(asset.family(), "Atkinson Hyperlegible Next");
    assert_eq!(asset.postscript_name(), "AtkinsonHyperlegibleNext-Regular");
}

#[test]
fn bundled_molecule_label_font_matches_the_shared_scalar_capability_table() {
    let environment =
        FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
    VerifiedMoleculeLabelGlyphMetrics::new(&environment)
        .expect("bundled Atkinson Hyperlegible Next must satisfy every shared scalar capability");
}

#[test]
fn verified_metrics_lay_out_the_verified_font_without_a_family_lookup() {
    let environment =
        FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
    let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Atkinson Hyperlegible Next");
    let font = AtomLabelFontProfile::new(FontFace::molecule_label(), size(12.0), paint());
    let label = AtomLabelFacts::new("Cl", None, 1, 2).expect("chemical label is valid");
    let layout = metrics
        .layout_atom_label(&label, &font)
        .expect("verified Atkinson Hyperlegible Next lays out the label");
    assert_eq!(layout.runs().len(), 4);
    assert!(layout.bounds().min_x() < 0.0);
    assert!(layout.bounds().max_x() > 0.0);
}

#[test]
fn verified_metrics_center_the_closed_plus_glyph_without_frontend_advances() {
    let environment =
        FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
    let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Atkinson Hyperlegible Next");
    let layout = metrics
        .layout_centered_plus(size(18.0), paint())
        .expect("verified Atkinson Hyperlegible Next lays out the fixed plus content");
    let operation = layout.operation();
    assert_eq!(operation.face(), &FontFace::molecule_label());
    let [run] = operation.runs() else {
        panic!("fixed plus layout has one exact run");
    };
    assert_eq!(run.text(), "+");
    assert_eq!(run.glyphs().len(), 1);
    assert_ne!(run.glyphs()[0].glyph_index(), 0);
    let bounds = layout.bounds();
    // Atkinson Hyperlegible Next + has positive x bearing and ink below its baseline. Centering the
    // true outline (55..551 x -248..248 design units) must not include the
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
            -5.454,
            4.4639999999999995,
            -4.4639999999999995,
            -4.4639999999999995,
            4.4639999999999995,
            4.4639999999999995,
        )
    );
}

#[test]
fn true_ink_metrics_and_atom_label_clipping_envelope_have_distinct_contracts() {
    let environment =
        FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
    let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Atkinson Hyperlegible Next");
    let font = AtomLabelFontProfile::new(FontFace::molecule_label(), size(12.0), paint());

    let ink = metrics
        .measure_text_run("I", font.size(), size(1.0))
        .expect("Atkinson Hyperlegible Next I is measurable");
    // The exact design outline is x=55..337 at 12 scene units per em.
    assert_eq!(
        (
            ink.x_bearing(),
            ink.y_bearing(),
            ink.width(),
            ink.height(),
            ink.x_advance()
        ),
        (0.66, -8.016, 3.3840000000000003, 8.016, 4.704000000000001)
    );

    let label = metrics
        .layout_atom_label(
            &AtomLabelFacts::new("I", None, 0, 0).expect("valid iodine label"),
            &font,
        )
        .expect("iodine label is laid out");
    // Atom/bond clipping uses the exact visible ink envelope, whose structural
    // element has been centered on the atom origin.
    assert_eq!(
        (
            label.bounds().min_x(),
            label.bounds().min_y(),
            label.bounds().max_x(),
            label.bounds().max_y(),
        ),
        (-1.6920000000000002, -4.008, 1.6920000000000002, 4.008)
    );
}

#[test]
fn wrong_file_cannot_pass_the_asset_verifier() {
    let wrong = std::env::temp_dir().join("ferrum-not-molecule_label_font.ttf");
    std::fs::write(&wrong, b"not a font").expect("write isolated invalid font fixture");
    let result = FerrumFontEnvironment::load_for_test(&wrong);
    std::fs::remove_file(&wrong).expect("remove isolated invalid font fixture");
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn symlinked_asset_component_cannot_pass_the_resource_verifier() {
    use std::os::unix::fs::symlink;

    let root = fixture_root().join(format!(
        "ferrum-molecule_label_font-symlink-{}",
        std::process::id()
    ));
    let target = root.join("target");
    let linked = root.join("linked");
    std::fs::create_dir_all(&target).expect("create isolated target directory");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/atkinson_hyperlegible_next/ttf/atkinson_hyperlegible_next_regular.ttf"
        ),
        target.join("atkinson_hyperlegible_next_regular.ttf"),
    )
    .expect("copy verified Atkinson Hyperlegible Next fixture");
    symlink(&target, &linked).expect("create isolated symlink component");

    let result = FerrumFontEnvironment::load_for_test(
        &linked.join("atkinson_hyperlegible_next_regular.ttf"),
    );
    std::fs::remove_dir_all(&root).expect("remove isolated symlink fixture");
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn final_symlink_replacement_between_parent_and_final_open_is_rejected() {
    use std::os::unix::fs::symlink;

    let fixture = TreeFixture::molecule_label_font();
    let font = fixture.font_path();
    let replacement = fixture.root().join("replacement.ttf");
    std::fs::write(&replacement, b"replacement bytes").expect("write replacement fixture");
    let font_for_hook = font.to_owned();
    let result = FerrumFontEnvironment::load_for_test_with_after_parent_open(font, move || {
        std::fs::remove_file(&font_for_hook).expect("remove final fixture before open");
        symlink(&replacement, &font_for_hook).expect("replace final fixture with symlink");
    });
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn parent_replacement_after_descriptor_open_cannot_redirect_final_open() {
    use std::os::unix::fs::symlink;

    let fixture = TreeFixture::molecule_label_font();
    let font = fixture.font_path();
    let parent = font.parent().expect("font has fixture parent").to_owned();
    let moved_parent = fixture.root().join("fonts-retained");
    let redirect = fixture.root().join("redirect");
    std::fs::create_dir(&redirect).expect("create isolated redirect directory");
    std::fs::write(
        redirect.join("atkinson_hyperlegible_next_regular.ttf"),
        b"replacement bytes",
    )
    .expect("write isolated redirect font");
    let result = FerrumFontEnvironment::load_for_test_with_after_parent_open(font, move || {
        std::fs::rename(&parent, &moved_parent).expect("move visible parent after descriptor open");
        symlink(&redirect, &parent).expect("replace visible parent with a symlink");
    });
    assert!(result.is_ok());
}

#[test]
fn final_replacement_after_descriptor_open_cannot_change_same_handle_verification() {
    let fixture = TreeFixture::molecule_label_font();
    let font = fixture.font_path();
    let replacement = fixture.root().join("replacement.ttf");
    std::fs::write(&replacement, b"replacement bytes").expect("write replacement fixture");
    let font_for_hook = font.to_owned();
    let environment = FerrumFontEnvironment::load_for_test_with_after_final_open(font, move || {
        std::fs::rename(&replacement, &font_for_hook)
            .expect("replace visible final entry after descriptor open");
    })
    .expect("the retained final descriptor still reads and verifies the original Atkinson Hyperlegible Next bytes");
    let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
        .expect("pure-Rust parser consumes immutable verified bytes");
    let font = AtomLabelFontProfile::new(FontFace::molecule_label(), size(12.0), paint());
    let label = AtomLabelFacts::new("Cl", None, 1, 2).expect("chemical label is valid");
    assert!(metrics.layout_atom_label(&label, &font).is_ok());
}

#[test]
fn verified_metrics_use_memory_after_the_asset_path_is_replaced() {
    let asset = PathFixture::molecule_label_copy();
    let environment = FerrumFontEnvironment::load_for_test(asset.path())
        .expect("copied Atkinson Hyperlegible Next is verified into immutable resource bytes");
    std::fs::write(asset.path(), b"replacement bytes").expect("replace only the fixture pathname");
    let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
        .expect("pure-Rust parser receives retained verified bytes rather than reopening the path");
    let font = AtomLabelFontProfile::new(FontFace::molecule_label(), size(12.0), paint());
    let label = AtomLabelFacts::new("Cl", None, 1, 2).expect("chemical label is valid");
    assert!(metrics.layout_atom_label(&label, &font).is_ok());
}

#[test]
#[ignore = "developer metric receipt; run through devel/measure_m12_font_metrics.py"]
fn metric_receipt_rows_cover_the_complete_pre_tolerance_corpus() {
    let environment =
        FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
    let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Atkinson Hyperlegible Next");
    let font = AtomLabelFontProfile::new(FontFace::molecule_label(), size(12.0), paint());
    let baseline = metrics
        .baseline_metrics(font.size())
        .expect("Atkinson Hyperlegible Next baseline metrics are finite");
    for (label, facts) in receipt_corpus() {
        let layout = metrics
            .layout_atom_label(&facts, &font)
            .expect("Atkinson Hyperlegible Next lays out each receipt corpus label");
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
                "schema": "ferrum-m12-truetype-design-metric-row",
                "label": label,
                "font": {
                    "id": environment.molecule_label().resource_id(),
                    "bytes": environment.molecule_label().bytes(),
                    "sha256": environment.molecule_label().sha256(),
                    "family": environment.molecule_label().family(),
                    "postscript_name": environment.molecule_label().postscript_name(),
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
                    "metric_backend": "ttf-parser-design",
                },
            })
        );
    }
}

#[test]
fn truetype_design_metrics_define_the_canonical_corpus_receipt_exactly() {
    let environment =
        FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
    let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Atkinson Hyperlegible Next");
    let font = AtomLabelFontProfile::new(FontFace::molecule_label(), size(12.0), paint());
    let baseline = metrics
        .baseline_metrics(font.size())
        .expect("Atkinson Hyperlegible Next baseline is measurable");
    assert_eq!(
        (baseline.ascent(), baseline.descent(), baseline.height()),
        (11.808, 3.792, 15.600000000000001)
    );

    assert_receipt_label(
        &metrics,
        &font,
        AtomLabelFacts::new("C", None, 0, 0).expect("valid corpus label"),
        (-3.4080000000000004, -4.152, 3.4080000000000004, 4.152),
        (-3.4080000000000004, -4.152, 3.4080000000000004, 4.152),
        &[ExpectedRun::baseline(
            "C",
            -3.936,
            4.008,
            13,
            7.74,
            6.816000000000001,
            8.304,
        )],
    );
    assert_receipt_label(
        &metrics,
        &font,
        AtomLabelFacts::new("Cl", None, 0, 0).expect("valid corpus label"),
        (-5.058, -4.32, 5.058, 4.32),
        (-5.058, -4.32, 5.058, 4.32),
        &[
            ExpectedRun::baseline("Cl", -5.586, 4.176, 13, 10.884, 10.116, 8.64)
                .with_second_glyph(164, 7.74),
        ],
    );
    assert_receipt_label(
        &metrics,
        &font,
        AtomLabelFacts::new("Br", None, 0, 0).expect("valid corpus label"),
        (-5.232, -4.008, 5.232, 4.008),
        (-5.232, -4.008, 5.232, 4.008),
        &[
            ExpectedRun::baseline("Br", -6.168, 4.008, 12, 11.544, 10.464, 8.016)
                .with_second_glyph(187, 7.356),
        ],
    );
    assert_receipt_label(
        &metrics,
        &font,
        AtomLabelFacts::new("H", None, 0, 2).expect("valid corpus label"),
        (-3.2280000000000006, -4.008, 16.47, 7.0416),
        (-3.228, -4.008, 3.228, 4.008),
        &[
            ExpectedRun::baseline("H", -4.164000000000001, 4.008, 36, 8.328, 6.456, 8.016),
            ExpectedRun::baseline("H", 4.163999999999999, 4.008, 36, 8.328, 6.456, 8.016),
            ExpectedRun::subscript(
                "2",
                12.491999999999997,
                7.0416,
                367,
                4.274400000000001,
                3.7518000000000002,
                5.304,
            ),
        ],
    );
    assert_receipt_label(
        &metrics,
        &font,
        AtomLabelFacts::new("N", None, 1, 3).expect("valid corpus label"),
        (
            -3.2220000000000004,
            -6.355200000000001,
            21.276599999999995,
            7.1352,
        ),
        (-3.222, -4.008, 3.222, 4.008),
        &[
            ExpectedRun::baseline("N", -4.158, 4.008, 57, 8.315999999999999, 6.444, 8.016),
            ExpectedRun::baseline("H", 4.157999999999999, 4.008, 36, 8.328, 6.456, 8.016),
            ExpectedRun::subscript(
                "3",
                12.485999999999997,
                7.0416,
                368,
                4.4928,
                3.6972,
                5.397600000000001,
            ),
            ExpectedRun::superscript(
                "+",
                16.978799999999996,
                -2.4864000000000006,
                298,
                4.7268,
                3.8688000000000002,
                3.8688000000000002,
            ),
        ],
    );
    assert_receipt_label(
        &metrics,
        &font,
        AtomLabelFacts::new("I", None, 0, 0).expect("valid corpus label"),
        (-1.6920000000000002, -4.008, 1.6920000000000002, 4.008),
        (-1.6920000000000002, -4.008, 1.6920000000000002, 4.008),
        &[ExpectedRun::baseline(
            "I",
            -2.3520000000000003,
            4.008,
            38,
            4.704000000000001,
            3.3840000000000003,
            8.016,
        )],
    );
}

#[test]
fn verified_metrics_reject_controls_and_missing_scalars_without_fallback() {
    let environment =
        FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
    let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Atkinson Hyperlegible Next");
    let result = metrics.glyphs_for_run("C\n", size(12.0), size(1.0));
    assert!(result.is_err());
    let result = metrics.glyphs_for_run("\u{10ffff}", size(12.0), size(1.0));
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
        origin_y: f64,
        glyph: u32,
        advance: f64,
        width: f64,
        height: f64,
    ) -> Self {
        Self {
            text,
            script: TextScript::Baseline,
            scale: 1.0,
            origin: (origin_x, origin_y),
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
    metrics: &VerifiedMoleculeLabelGlyphMetrics,
    font: &AtomLabelFontProfile,
    label: AtomLabelFacts,
    expected_bounds: (f64, f64, f64, f64),
    expected_core_bounds: (f64, f64, f64, f64),
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
    let attachment = layout.attachment();
    let core_run = layout
        .runs()
        .first()
        .expect("atom labels contain their structural element run");
    assert_eq!(core_run.text(), label.element());
    assert_eq!(core_run.script(), TextScript::Baseline);
    let core_bounds = attachment.core_element_ink_bounds();
    assert_eq!(
        (
            core_bounds.min_x(),
            core_bounds.min_y(),
            core_bounds.max_x(),
            core_bounds.max_y(),
        ),
        (
            expected_core_bounds.0,
            expected_core_bounds.1,
            expected_core_bounds.2,
            expected_core_bounds.3,
        )
    );
    // The durable attachment rectangle canonicalizes its exact center to
    // positive zero. The receipt above, rather than an uncanonicalized
    // origin-plus-bearing expression, is the public exact-bound contract.
    assert_eq!(
        attachment.core_element_ink_center(),
        crate::RenderPoint::new(0.0, 0.0).expect("origin")
    );
    let full = layout.bounds();
    assert!(core_bounds.min_x() >= full.min_x());
    assert!(core_bounds.min_y() >= full.min_y());
    assert!(core_bounds.max_x() <= full.max_x());
    assert!(core_bounds.max_y() <= full.max_y());
}

fn expected_y_bearing(text: &str) -> f64 {
    match text {
        "C" => -8.16,
        "Cl" => -8.496,
        "Br" | "H" | "N" | "I" => -8.016,
        "2" | "3" => -5.304,
        "+" => -3.8688000000000002,
        _ => panic!("unexpected receipt run"),
    }
}

fn expected_x_bearing(text: &str) -> f64 {
    match text {
        "C" | "Cl" => 0.528,
        "Br" | "H" | "N" => 0.936,
        "2" => 0.2262,
        "3" => 0.3432,
        "+" => 0.42900000000000005,
        "I" => 0.66,
        _ => panic!("unexpected receipt run"),
    }
}

fn receipt_corpus() -> [(&'static str, AtomLabelFacts); 6] {
    [
        (
            "C",
            AtomLabelFacts::new("C", None, 0, 0).expect("valid corpus label"),
        ),
        (
            "Cl",
            AtomLabelFacts::new("Cl", None, 0, 0).expect("valid corpus label"),
        ),
        (
            "Br",
            AtomLabelFacts::new("Br", None, 0, 0).expect("valid corpus label"),
        ),
        (
            "H2",
            AtomLabelFacts::new("H", None, 0, 2).expect("valid corpus label"),
        ),
        (
            "NH3+",
            AtomLabelFacts::new("N", None, 1, 3).expect("valid corpus label"),
        ),
        (
            "I",
            AtomLabelFacts::new("I", None, 0, 0).expect("valid corpus label"),
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
    fn molecule_label_font() -> Self {
        let root = fixture_root().join(format!(
            "ferrum-molecule_label_font-tree-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        let fonts = root.join("package/fonts");
        std::fs::create_dir_all(&fonts).expect("create isolated Atkinson Hyperlegible Next tree");
        let font = fonts.join("atkinson_hyperlegible_next_regular.ttf");
        std::fs::copy(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/fonts/atkinson_hyperlegible_next/ttf/atkinson_hyperlegible_next_regular.ttf"
            ),
            &font,
        )
        .expect("copy verified Atkinson Hyperlegible Next fixture");
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
    fn molecule_label_copy() -> Self {
        let path = fixture_root().join(format!(
            "ferrum-molecule_label_font-copy-{}-{}.ttf",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::copy(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/fonts/atkinson_hyperlegible_next/ttf/atkinson_hyperlegible_next_regular.ttf"
            ),
            &path,
        )
        .expect("copy verified Atkinson Hyperlegible Next fixture");
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
