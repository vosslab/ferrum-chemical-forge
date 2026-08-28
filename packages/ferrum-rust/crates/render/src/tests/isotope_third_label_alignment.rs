use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_document_projection::DocumentObjectIdV1;

use crate::atom_bond::build_atom_bond_plan;
use crate::glyph_metrics::GlyphMetrics;
use crate::render_target::RenderPlanEntryContextV1;
use crate::*;

fn id(kind: RecordKind, value: &str) -> RecordId {
    RecordId::new(kind, Identifier::new(value).expect("identifier")).expect("record ID")
}

fn context(
    entropy: u8,
    kind: RecordKind,
    value: &str,
    paint_order: u32,
) -> RenderPlanEntryContextV1 {
    RenderPlanEntryContextV1::new(
        RenderTarget::document_object(DocumentObjectIdV1::from_entropy_bytes([entropy; 16])),
        id(kind, value),
        paint_order,
        Some(DocumentObjectIdV1::from_entropy_bytes([0xa1; 16])),
    )
}

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("finite point")
}

fn size(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("positive extent")
}

fn paint() -> RenderPaintV3 {
    RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("rgb"))
}

fn font() -> AtomLabelFontProfile {
    AtomLabelFontProfile::new(FontFace::telex_regular(), size(10.0), paint())
}

fn metrics() -> VerifiedTelexGlyphMetrics {
    VerifiedTelexGlyphMetrics::new(&FerrumFontEnvironmentV1::load().expect("Telex"))
        .expect("metrics")
}

fn atom(
    entropy: u8,
    value: &str,
    order: u32,
    x: f64,
    y: f64,
    facts: AtomLabelFacts,
) -> AtomRenderTarget {
    AtomRenderTarget::new(
        context(entropy, RecordKind::Atom, value, order),
        point(x, y),
        facts,
        TargetVisibility::Visible,
    )
    .expect("atom")
}

fn request(
    atoms: Vec<AtomRenderTarget>,
    bonds: Vec<BondRenderTarget>,
    entropy: u8,
) -> AtomBondRenderRequest {
    AtomBondRenderRequest::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [entropy; 32]),
        atoms,
        bonds,
        font(),
        size(1.0),
        size(6.0),
        BondInkClearance::new(size(1.25)),
        paint(),
    )
    .expect("request")
}

#[test]
fn isotope_is_first_superscript_and_structural_element_remains_issued_core_run() {
    assert!(AtomLabelFacts::new("C", Some(0), 0, 0).is_err());
    assert!(AtomLabelFacts::new("C", Some(32_768), 0, 0).is_err());
    let facts = AtomLabelFacts::new("C", Some(13), 1, 3).expect("13CH3+");
    let layout = metrics()
        .layout_atom_label(&facts, &font())
        .expect("layout");
    assert_eq!(layout.core_element_run_index(), 1);
    assert_eq!(layout.runs()[0].text(), "13");
    assert_eq!(layout.runs()[0].script(), TextScript::Superscript);
    assert_eq!(layout.runs()[1].text(), "C");
    assert_eq!(layout.runs()[1].script(), TextScript::Baseline);
    assert!(AtomLabelFacts::new("C", Some(32_767), 0, 0).is_ok());

    let plan = build_atom_bond_plan(
        &request(
            vec![atom(0x31, "isotope", 1, 0.0, 0.0, facts)],
            vec![],
            0x31,
        ),
        &metrics(),
    )
    .expect("plan");
    let RenderBatchContentV4::Atom(batch) = plan.batches()[0].content() else {
        panic!("atom batch")
    };
    assert_eq!(batch.label().core_element_run_index(), 1);
    assert!(
        batch.label().full_ink_bounds().min_x() <= batch.label().core_element_ink_bounds().min_x()
    );
}

#[test]
fn third_label_conflict_discards_only_candidate_bond_and_strict_near_miss_renders() {
    let crossing = BondRenderTarget::new(
        context(0x44, RecordKind::Bond, "crossing", 2),
        id(RecordKind::Atom, "first"),
        id(RecordKind::Atom, "second"),
        BondStyle::NormalSingle,
        TargetVisibility::Visible,
    )
    .expect("bond");
    let plan = build_atom_bond_plan(
        &request(
            vec![
                atom(
                    0x41,
                    "first",
                    1,
                    0.0,
                    0.0,
                    AtomLabelFacts::new("N", None, 1, 2).expect("label"),
                ),
                atom(
                    0x42,
                    "second",
                    3,
                    40.0,
                    0.0,
                    AtomLabelFacts::new("N", None, 1, 2).expect("label"),
                ),
                atom(
                    0x43,
                    "third",
                    4,
                    20.0,
                    0.0,
                    AtomLabelFacts::new("N", None, 1, 2).expect("label"),
                ),
            ],
            vec![crossing],
            0x41,
        ),
        &metrics(),
    )
    .expect("plan");
    assert_eq!(plan.batches().len(), 3);
    assert!(
        matches!(plan.issues()[0].kind(), RenderIssueKind::UnrenderableTarget { reason }
        if reason == "bond final ink intersects a non-endpoint atom label")
    );

    let near = BondRenderTarget::new(
        context(0x54, RecordKind::Bond, "near", 2),
        id(RecordKind::Atom, "near-first"),
        id(RecordKind::Atom, "near-second"),
        BondStyle::NormalSingle,
        TargetVisibility::Visible,
    )
    .expect("bond");
    let plan = build_atom_bond_plan(
        &request(
            vec![
                atom(
                    0x51,
                    "near-first",
                    1,
                    0.0,
                    0.0,
                    AtomLabelFacts::new("N", None, 1, 2).expect("label"),
                ),
                atom(
                    0x52,
                    "near-second",
                    3,
                    40.0,
                    0.0,
                    AtomLabelFacts::new("N", None, 1, 2).expect("label"),
                ),
                atom(
                    0x53,
                    "near-third",
                    4,
                    20.0,
                    20.0,
                    AtomLabelFacts::new("N", None, 1, 2).expect("label"),
                ),
            ],
            vec![near],
            0x51,
        ),
        &metrics(),
    )
    .expect("plan");
    assert_eq!(plan.batches().len(), 4);
    assert!(plan.issues().is_empty());
}
