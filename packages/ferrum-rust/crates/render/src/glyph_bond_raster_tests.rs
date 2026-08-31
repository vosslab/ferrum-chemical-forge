//! Focused test coverage for the test-only glyph-bond raster handoff.

use std::collections::BTreeMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_document_projection::DocumentObjectIdV1;

use crate::atom_bond::build_atom_bond_plan;
use crate::glyph_bond_raster::{
    GlyphBondRasterBondIdentity, GlyphBondRasterFixtureIdentity, GlyphBondRasterSourceMapping,
    rasterize_glyph_bond_layers,
};
use crate::render_target::RenderPlanEntryContextV1;
use crate::{
    AtomBondRenderRequest, AtomLabelFacts, AtomLabelFontProfile, AtomRenderTarget,
    BondInkClearance, BondRenderTarget, BondStyle, FerrumFontEnvironment, FontFace,
    MoleculeRenderPlanV4, PositiveFinite, RenderBatchContentV4, RenderPaintV3, RenderPoint,
    RenderProvenance, RenderRevision, RenderTarget, RenderViewportV1, Rgb24, TargetVisibility,
    VerifiedMoleculeLabelGlyphMetrics,
};

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("test point is finite")
}

fn size(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("test extent is positive")
}

fn paint(value: &str) -> RenderPaintV3 {
    RenderPaintV3::authored_rgb24(Rgb24::new(value).expect("test paint is valid"))
}

fn record_id(kind: RecordKind, value: &str) -> RecordId {
    RecordId::new(
        kind,
        Identifier::new(value).expect("test identifier is valid"),
    )
    .expect("test record ID")
}

fn context(
    entropy: u8,
    kind: RecordKind,
    source: &str,
    paint_order: u32,
) -> RenderPlanEntryContextV1 {
    RenderPlanEntryContextV1::new(
        RenderTarget::document_object(DocumentObjectIdV1::from_entropy_bytes([entropy; 16])),
        record_id(kind, source),
        paint_order,
        None,
    )
}

fn metrics() -> VerifiedMoleculeLabelGlyphMetrics {
    let environment =
        FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
    VerifiedMoleculeLabelGlyphMetrics::new(&environment)
        .expect("verified Atkinson Hyperlegible Next opens")
}

fn two_label_plan() -> MoleculeRenderPlanV4 {
    let first = AtomRenderTarget::new(
        context(0x11, RecordKind::Atom, "raster-left", 1),
        point(0.0, 0.0),
        AtomLabelFacts::new("N", Some(15), 1, 1).expect("atom facts"),
        TargetVisibility::Visible,
    )
    .expect("atom target");
    let second = AtomRenderTarget::new(
        context(0x12, RecordKind::Atom, "raster-right", 3),
        point(40.0, 0.0),
        AtomLabelFacts::new("O", None, -1, 0).expect("atom facts"),
        TargetVisibility::Visible,
    )
    .expect("atom target");
    let bond = BondRenderTarget::new(
        context(0x13, RecordKind::Bond, "raster-bond", 2),
        record_id(RecordKind::Atom, "raster-left"),
        record_id(RecordKind::Atom, "raster-right"),
        BondStyle::Double,
        TargetVisibility::Visible,
    )
    .expect("bond target");
    let request = AtomBondRenderRequest::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [0x35; 32]),
        vec![first, second],
        vec![bond],
        AtomLabelFontProfile::new(FontFace::molecule_label(), size(10.0), paint("000000"))
            .with_label_mask(paint("ffffff")),
        size(1.0),
        size(8.0),
        BondInkClearance::new(size(1.25)),
        paint("112233"),
    )
    .expect("render request");
    build_atom_bond_plan(&request, &metrics()).expect("accepted plan")
}

fn viewport() -> RenderViewportV1 {
    RenderViewportV1::new(-200.0, -200.0, 400.0, 400.0).expect("test viewport")
}

fn target_identity(target: &RenderTarget) -> String {
    target.document_object_id().as_str().to_owned()
}

fn source_mapping(plan: &MoleculeRenderPlanV4) -> GlyphBondRasterSourceMapping {
    let mut atoms = BTreeMap::new();
    let mut bonds = BTreeMap::new();
    for batch in plan.batches() {
        match batch.content() {
            RenderBatchContentV4::Atom(_) => {
                let source = format!("atom_{}", atoms.len());
                atoms.insert(target_identity(batch.target()), source);
            }
            RenderBatchContentV4::Bond(_) => {
                let source = format!("bond_{}", bonds.len());
                bonds.insert(target_identity(batch.target()), source);
            }
            RenderBatchContentV4::CompactGroup(_) => {}
        }
    }
    GlyphBondRasterSourceMapping::new(atoms, bonds)
}

#[test]
fn raster_sink_emits_8x_composite_core_and_final_bond_layers() {
    let plan = two_label_plan();
    let layers = rasterize_glyph_bond_layers(&plan, viewport(), &source_mapping(&plan))
        .expect("raster layers");

    assert_eq!(layers.normal_composite().width(), 3200);
    assert_eq!(layers.normal_composite().height(), 3200);
    assert!(layers.normal_composite().nontransparent_pixels() > 0);
    assert_eq!(layers.target_core_glyph_masks().len(), 2);
    assert_eq!(layers.full_label_masks().len(), 2);
    assert_eq!(layers.final_bond_footprints().len(), 1);
    assert!(
        layers
            .target_core_glyph_masks()
            .values()
            .all(|mask| mask.nontransparent_pixels() > 0)
    );
    assert!(
        layers
            .final_bond_footprints()
            .values()
            .all(|mask| mask.nontransparent_pixels() > 0)
    );
}

#[test]
fn raster_sink_writes_the_closed_measure_stack_final_ink_manifest() {
    let plan = two_label_plan();
    let layers = rasterize_glyph_bond_layers(&plan, viewport(), &source_mapping(&plan))
        .expect("raster layers");
    let atom_ids: Vec<_> = layers.target_core_glyph_masks().keys().cloned().collect();
    let bond_id = layers
        .final_bond_footprints()
        .keys()
        .next()
        .expect("one bond layer")
        .clone();
    let directory = std::env::temp_dir().join(format!(
        "ferrum_glyph_bond_raster_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos(),
    ));
    let manifest = layers
        .write_measurement_manifest_v2(
            &directory,
            &GlyphBondRasterFixtureIdentity::from_cdml(
                "raster_sink_two_label_v1",
                "<cdml fixture=\"raster_sink_two_label_v1\"/>",
                Vec::new(),
                Vec::new(),
            ),
            &[GlyphBondRasterBondIdentity::new(
                bond_id,
                atom_ids[0].clone(),
                atom_ids[1].clone(),
                "double",
            )],
        )
        .expect("measurement manifest");
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest).expect("manifest reads"))
            .expect("manifest is JSON");
    assert_eq!(
        value["schema"],
        serde_json::Value::String("ferrum-measure-stack-raster-layers-v2".to_owned())
    );
    assert!(directory.join("final_composite.png").is_file());
    assert_eq!(value.as_object().expect("manifest object").len(), 10);
    assert_eq!(
        value["capture_profile"]["profile_id"],
        "rust_final_ink_8x_400_square_v1"
    );
    assert_eq!(
        value["capture_profile"]["scene_evaluation"],
        "raw_final_ink"
    );
    assert_eq!(
        value["atom_layers"].as_array().expect("atom array").len(),
        2
    );
    assert_eq!(
        value["bond_layers"].as_array().expect("bond array").len(),
        1
    );
    assert!(
        value["atom_layers"][0]["full_label_layer"]["sha256"]
            .as_str()
            .is_some_and(|digest| digest.len() == 64)
    );
    fs::remove_dir_all(directory).expect("test artifacts remove");
}
