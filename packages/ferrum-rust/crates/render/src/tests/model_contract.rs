use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_document_projection::DocumentObjectIdV1;
use serde_json::json;

use crate::render_target::RenderPlanEntryContextV1;
use crate::*;

fn target(byte: u8) -> RenderTarget {
    RenderTarget::document_object(DocumentObjectIdV1::from_entropy_bytes([byte; 16]))
}

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("test point is finite")
}

fn size(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("test extent is positive and finite")
}

fn paint(value: &str) -> RenderPaintV3 {
    RenderPaintV3::authored_rgb24(Rgb24::new(value).expect("test rgb"))
}

fn label() -> AtomLabelRenderV1 {
    RenderBatchV4::test_atom_label_from_facts(
        None,
        AtomLabelFacts::new("N", None, 1, 2).expect("label facts"),
        AtomLabelFontProfile::new(FontFace::telex_regular(), size(12.0), paint("000000")),
        2,
    )
    .expect("verified atom label")
}

fn line() -> LineOp {
    LineOp::new(
        point(1.0, 0.0),
        point(4.0, 0.0),
        size(1.0),
        paint("112233"),
        1,
    )
    .expect("line operation")
}

fn bond_attachment_axis() -> BondAttachmentAxisV1 {
    BondAttachmentAxisV1::new(point(0.0, 0.0), point(5.0, 0.0)).expect("test bond attachment axis")
}

fn filled_path() -> PathOpV3 {
    PathOpV3::new(
        vec![
            ScenePathCommandV3::MoveTo(point(1.0, 1.0)),
            ScenePathCommandV3::LineTo(point(5.0, 1.0)),
            ScenePathCommandV3::LineTo(point(3.0, 4.0)),
            ScenePathCommandV3::Close,
        ],
        None,
        Some(paint("112233")),
        2,
    )
    .expect("finite closed path")
}

fn atom_batch(target: RenderTarget, paint_order: u32, anchor: RenderPoint) -> RenderBatchV4 {
    RenderBatchV4::test_atom_target(
        target,
        paint_order,
        AtomRenderBatchV1::new(anchor, label(), Vec::new()).expect("atom content"),
    )
}

fn bond_batch(
    target: RenderTarget,
    paint_order: u32,
    operations: Vec<BondRenderOpV1>,
) -> RenderBatchV4 {
    RenderBatchV4::bond_target(
        target,
        paint_order,
        BondRenderBatchV1::new(bond_attachment_axis(), operations).expect("bond content"),
    )
}

fn plan() -> MoleculeRenderPlanV4 {
    MoleculeRenderPlanV4::new(
        RenderProvenance::new(RenderRevision::new(8).expect("revision"), [8; 32]),
        vec![
            atom_batch(target(0x11), 3, point(2.0, 4.0)),
            bond_batch(target(0x12), 4, vec![BondRenderOpV1::Line(line())]),
        ],
        vec![],
    )
    .expect("render plan")
}

#[test]
fn plan_json_is_canonical_and_round_trips() {
    let original = plan();
    let first = original.to_canonical_json().expect("serialize");
    let restored = MoleculeRenderPlanV4::from_json(&first).expect("deserialize");
    let second = restored.to_canonical_json().expect("serialize again");
    assert_eq!(first, second);
    assert_eq!(restored, original);
    assert!(first.starts_with("{\"schema\":\"ferrum-render-plan-v4\""));
    assert!(first.contains("\"paint\":{\"kind\":\"authored_rgb24\",\"rgb\":\"112233\"}"));
    assert_eq!(
        original
            .batches()
            .iter()
            .map(RenderBatchV4::paint_order)
            .collect::<Vec<_>>(),
        vec![3, 4]
    );
}

#[test]
fn public_targets_serialize_only_durable_document_identity() {
    let durable_target = target(0x21);
    let wire = serde_json::to_value(&durable_target).expect("target serializes");
    let fields = wire.as_object().expect("target object");
    assert_eq!(fields.len(), 1);
    assert_eq!(
        wire["document_object_id"],
        json!(durable_target.document_object_id().as_str())
    );
    for local_field in [
        "record_id",
        "source_order",
        "paint_order",
        "owner_molecule_object_id",
    ] {
        assert!(fields.get(local_field).is_none(), "no {local_field} field");
    }

    let mut forged = wire;
    forged["record_id"] = json!("atom:source_a1");
    assert!(serde_json::from_value::<RenderTarget>(forged).is_err());
}

#[test]
fn private_context_carries_source_kind_and_paint_order_into_batch_construction() {
    let context = RenderPlanEntryContextV1::new(
        target(0x22),
        RecordId::new(
            RecordKind::Atom,
            Identifier::new("source_a1").expect("test source identifier is valid"),
        )
        .expect("test record ID"),
        9,
        Some(DocumentObjectIdV1::from_entropy_bytes([0x23; 16])),
    );
    let batch = RenderBatchV4::atom(
        context,
        AtomRenderBatchV1::new(point(0.0, 0.0), label(), Vec::new()).expect("atom content"),
    )
    .expect("atom source produces atom-local batch");
    assert_eq!(batch.paint_order(), 9);
    assert_eq!(
        batch.target().document_object_id(),
        target(0x22).document_object_id()
    );
}

#[test]
fn scene_path_requires_closed_finite_drawable_painted_geometry() {
    let accepted = filled_path();
    assert!(
        BondRenderBatchV1::new(bond_attachment_axis(), vec![BondRenderOpV1::Path(accepted)])
            .is_ok()
    );
    assert!(
        PathOpV3::new(
            vec![
                ScenePathCommandV3::MoveTo(point(1.0, 1.0)),
                ScenePathCommandV3::LineTo(point(5.0, 1.0)),
            ],
            None,
            Some(paint("112233")),
            0,
        )
        .is_err()
    );
    assert!(
        PathOpV3::new(
            vec![
                ScenePathCommandV3::MoveTo(point(1.0, 1.0)),
                ScenePathCommandV3::LineTo(point(5.0, 1.0)),
            ],
            None,
            None,
            0,
        )
        .is_err()
    );
    assert!(
        PathOpV3::new(
            vec![
                ScenePathCommandV3::MoveTo(point(1.0, 1.0)),
                ScenePathCommandV3::Close,
            ],
            None,
            Some(paint("112233")),
            0,
        )
        .is_err()
    );
    let path_plan = MoleculeRenderPlanV4::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
        vec![bond_batch(
            target(0x32),
            1,
            vec![BondRenderOpV1::Path(filled_path())],
        )],
        vec![],
    )
    .expect("path plan");
    let mut wire: serde_json::Value =
        serde_json::from_str(&path_plan.to_canonical_json().expect("serialize path plan"))
            .expect("path plan value");
    wire["batches"][0]["content"]["content"]["operations"][0]["operation"]["commands"][0]["command"]
        ["x"] = serde_json::Value::Null;
    assert!(MoleculeRenderPlanV4::from_json(&wire.to_string()).is_err());
}

#[test]
fn bond_attachment_axis_requires_finite_distinct_structural_endpoints() {
    assert!(RenderPoint::new(f64::NAN, 0.0).is_err());
    assert!(BondAttachmentAxisV1::new(point(1.0, 2.0), point(1.0, 2.0)).is_err());
    let axis = bond_attachment_axis();
    assert_eq!(axis.start(), point(0.0, 0.0));
    assert_eq!(axis.end(), point(5.0, 0.0));
}

#[test]
fn nonfinite_and_invalid_presentation_values_are_rejected() {
    assert!(RenderPoint::new(f64::NAN, 0.0).is_err());
    assert!(RenderPoint::new(f64::INFINITY, 0.0).is_err());
    assert!(PositiveFinite::new(0.0).is_err());
    assert!(PositiveFinite::new(-1.0).is_err());
    assert!(PositiveFinite::new(f64::NEG_INFINITY).is_err());
    assert!(Rgb24::new("#112233").is_err());
    assert!(Rgb24::new("11223G").is_err());
    assert!(FontFace::new("  ").is_err());
    assert!(TextRun::new("", TextScript::Baseline, point(0.0, 0.0), vec![], size(1.0)).is_err());
    assert!(
        LineOp::new(
            point(1.0, 1.0),
            point(1.0, 1.0),
            size(1.0),
            paint("000000"),
            0
        )
        .is_err()
    );
    assert!(
        EllipseOp::new(
            point(1.0, 1.0),
            size(2.0),
            size(1.0),
            0.0,
            None,
            None,
            None,
            0,
        )
        .is_err()
    );
    assert!(
        EllipseOp::new(
            point(1.0, 1.0),
            size(2.0),
            size(1.0),
            0.0,
            Some(size(1.0)),
            None,
            Some(paint("000000")),
            0,
        )
        .is_err()
    );
}

#[test]
fn initial_document_revision_zero_is_a_valid_render_provenance_value() {
    assert_eq!(RenderRevision::new(0).expect("initial revision").get(), 0);
}

#[test]
fn deserialization_rejects_invalid_geometry_unknown_tags_and_defaults() {
    let original = plan();
    let mut wire: serde_json::Value =
        serde_json::from_str(&original.to_canonical_json().expect("serialize")).expect("value");
    wire["batches"][0]["content"]["content"]["label"]["text"]["size"] = json!(0.0);
    assert!(MoleculeRenderPlanV4::from_json(&wire.to_string()).is_err());

    let mut unknown = serde_json::from_str::<serde_json::Value>(
        &original.to_canonical_json().expect("serialize"),
    )
    .expect("value");
    unknown["batches"][0]["content"]["kind"] = json!("polygon");
    assert!(MoleculeRenderPlanV4::from_json(&unknown.to_string()).is_err());

    let mut alias = serde_json::from_str::<serde_json::Value>(
        &original.to_canonical_json().expect("serialize"),
    )
    .expect("value");
    alias["batches"][0]["content"]["content"]["label"]["text"]
        .as_object_mut()
        .expect("object")
        .remove("z");
    assert!(MoleculeRenderPlanV4::from_json(&alias.to_string()).is_err());
}

#[test]
fn inbound_text_runs_reject_forged_or_non_scalar_telex_layouts() {
    let original = plan().to_canonical_json().expect("serialize");
    for text in ["e\u{301}", "\u{1F600}"] {
        let mut wire: serde_json::Value = serde_json::from_str(&original).expect("value");
        wire["batches"][0]["content"]["content"]["label"]["text"]["runs"][0]["text"] = json!(text);
        assert!(MoleculeRenderPlanV4::from_json(&wire.to_string()).is_err());
    }
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Telex");
    let fi_glyphs = metrics
        .v1_glyphs_for_run("fi", size(12.0), size(1.0))
        .expect("Telex has scalar glyphs for the adversary text");
    let mut semantic_adversary: serde_json::Value = serde_json::from_str(&original).expect("value");
    semantic_adversary["batches"][0]["content"]["content"]["label"]["text"]["runs"][0]["text"] =
        json!("fi");
    semantic_adversary["batches"][0]["content"]["content"]["label"]["text"]["runs"][0]["glyphs"] =
        serde_json::to_value(fi_glyphs).expect("glyph placements serialize");
    assert!(MoleculeRenderPlanV4::from_json(&semantic_adversary.to_string()).is_err());
    let mut forged: serde_json::Value = serde_json::from_str(&original).expect("value");
    forged["batches"][0]["content"]["content"]["label"]["text"]["runs"][0]["glyphs"][0]["glyph_index"] =
        json!(999_999_u32);
    assert!(MoleculeRenderPlanV4::from_json(&forged.to_string()).is_err());
}

#[test]
fn coordinate_space_target_and_operation_grammar_is_closed() {
    assert!(
        BondRenderBatchV1::new(bond_attachment_axis(), vec![BondRenderOpV1::Line(line())]).is_ok()
    );
    assert!(
        AtomRenderBatchV1::new(
            point(0.0, 0.0),
            RenderBatchV4::test_atom_label_from_facts(
                None,
                AtomLabelFacts::new("N", None, 1, 2).expect("label facts"),
                AtomLabelFontProfile::new(FontFace::telex_regular(), size(12.0), paint("000000"),),
                0,
            )
            .expect("verified atom label"),
            vec![AtomDecorationRenderOpV1::Line(line())],
        )
        .is_ok()
    );
    assert!(
        BondRenderBatchV1::from_render_operations(
            bond_attachment_axis(),
            vec![RenderOp::Text(label().text().clone(),)],
        )
        .is_err()
    );
}

#[test]
fn duplicate_durable_targets_are_rejected_even_when_paint_order_differs() {
    let atom = target(0x51);
    let first = atom_batch(atom.clone(), 1, point(0.0, 0.0));
    let second = atom_batch(atom, 9, point(1.0, 0.0));
    assert!(
        MoleculeRenderPlanV4::new(
            RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
            vec![first, second],
            vec![]
        )
        .is_err()
    );
}

#[test]
fn batches_and_operations_require_explicit_stable_order() {
    let atom = atom_batch(target(0x61), 4, point(0.0, 0.0));
    let bond = bond_batch(target(0x62), 3, vec![BondRenderOpV1::Line(line())]);
    assert!(
        MoleculeRenderPlanV4::new(
            RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
            vec![atom, bond],
            vec![],
        )
        .is_err()
    );

    let back = LineOp::new(
        point(1.0, 0.0),
        point(4.0, 0.0),
        size(1.0),
        paint("000000"),
        2,
    )
    .expect("line");
    let front = LineOp::new(
        point(5.0, 0.0),
        point(8.0, 0.0),
        size(1.0),
        paint("000000"),
        1,
    )
    .expect("line");
    assert!(
        BondRenderBatchV1::new(
            bond_attachment_axis(),
            vec![BondRenderOpV1::Line(back), BondRenderOpV1::Line(front)]
        )
        .is_err()
    );
}

#[test]
fn issues_are_validated_without_creating_partial_batches() {
    let issue = RenderIssue::new(
        target(0x71),
        0,
        RenderIssueKind::UnsupportedFeature {
            feature: "aromatic bond".to_owned(),
        },
    )
    .expect("issue");
    let result = MoleculeRenderPlanV4::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
        vec![],
        vec![issue],
    )
    .expect("plan");
    assert!(result.batches().is_empty());
    assert_eq!(result.issues().len(), 1);
    assert_eq!(result.issues()[0].paint_order(), 0);
    assert!(
        RenderIssue::new(
            target(0x72),
            1,
            RenderIssueKind::UnrenderableTarget {
                reason: " ".to_owned()
            }
        )
        .is_err()
    );
}

#[test]
fn exclusions_and_batches_form_a_unique_ordered_target_partition() {
    let batch = atom_batch(target(0x81), 3, point(0.0, 0.0));
    let later_batch = atom_batch(target(0x85), 5, point(8.0, 0.0));
    let issue = RenderIssue::new(
        target(0x82),
        4,
        RenderIssueKind::UnsupportedFeature {
            feature: "aromatic bond".to_owned(),
        },
    )
    .expect("issue");
    let plan = MoleculeRenderPlanV4::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
        vec![batch.clone(), later_batch],
        vec![issue.clone()],
    )
    .expect("partition");
    let json = plan.to_canonical_json().expect("serialize");
    let restored = MoleculeRenderPlanV4::from_json(&json).expect("deserialize");
    assert_eq!(
        restored
            .batches()
            .iter()
            .map(RenderBatchV4::paint_order)
            .collect::<Vec<_>>(),
        vec![3, 5]
    );
    assert_eq!(restored.issues()[0].paint_order(), 4);

    let conflicting_issue = RenderIssue::new(
        target(0x81),
        5,
        RenderIssueKind::UnrenderableTarget {
            reason: "missing label facts".to_owned(),
        },
    )
    .expect("issue");
    assert!(
        MoleculeRenderPlanV4::new(
            RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
            vec![batch.clone()],
            vec![conflicting_issue],
        )
        .is_err()
    );

    let duplicate_order_issue = RenderIssue::new(
        target(0x83),
        3,
        RenderIssueKind::UnsupportedFeature {
            feature: "wedge bond".to_owned(),
        },
    )
    .expect("issue");
    assert!(
        MoleculeRenderPlanV4::new(
            RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
            vec![batch.clone()],
            vec![duplicate_order_issue],
        )
        .is_err()
    );

    let later = RenderIssue::new(
        target(0x84),
        7,
        RenderIssueKind::UnsupportedFeature {
            feature: "double bond".to_owned(),
        },
    )
    .expect("issue");
    assert!(
        MoleculeRenderPlanV4::new(
            RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
            vec![],
            vec![later, issue],
        )
        .is_err()
    );
}

#[test]
fn wire_float_zeroes_are_normalized_and_text_controls_are_rejected() {
    let negative = RenderPoint::new(-0.0, -0.0).expect("finite point");
    let positive = RenderPoint::new(0.0, 0.0).expect("finite point");
    assert_eq!(negative, positive);
    assert_eq!(
        serde_json::to_string(&negative).expect("serialize point"),
        serde_json::to_string(&positive).expect("serialize point"),
    );
    assert_eq!(negative.x().to_bits(), 0.0_f64.to_bits());
    assert_eq!(negative.y().to_bits(), 0.0_f64.to_bits());

    for invalid in ["", "  ", "\t", "Ferrum\0Sans", "Ferrum\nSans"] {
        assert!(FontFace::new(invalid).is_err(), "invalid face: {invalid:?}");
        assert!(
            TextRun::new(
                invalid,
                TextScript::Baseline,
                point(0.0, 0.0),
                vec![],
                size(1.0)
            )
            .is_err()
        );
    }
    assert!(FontFace::new("ferrum-telex-regular-v1").is_ok());
    assert!(
        TextRun::new(
            "Cl-",
            TextScript::Baseline,
            point(0.0, 0.0),
            vec![
                GlyphPlacement::new(1, point(0.0, 0.0)).expect("glyph"),
                GlyphPlacement::new(2, point(1.0, 0.0)).expect("glyph"),
                GlyphPlacement::new(3, point(2.0, 0.0)).expect("glyph"),
            ],
            size(1.0),
        )
        .is_ok()
    );
}

#[test]
fn deserialization_rejects_partition_contradictions_and_noncanonical_payloads() {
    let original = plan();
    let mut wire: serde_json::Value =
        serde_json::from_str(&original.to_canonical_json().expect("serialize")).expect("value");
    wire["issues"] = json!([{
        "target": wire["batches"][0]["target"].clone(),
        "paint_order": 5,
        "kind": {"kind": "unsupported_feature", "feature": "aromatic bond"}
    }]);
    assert!(MoleculeRenderPlanV4::from_json(&wire.to_string()).is_err());

    let mut duplicate_issue: serde_json::Value =
        serde_json::from_str(&original.to_canonical_json().expect("serialize")).expect("value");
    duplicate_issue["issues"] = json!([
        {
            "target": {"document_object_id": target(0x91).document_object_id()},
            "paint_order": 7,
            "kind": {"kind": "unsupported_feature", "feature": "aromatic bond"}
        },
        {
            "target": {"document_object_id": target(0x92).document_object_id()},
            "paint_order": 6,
            "kind": {"kind": "unrenderable_target", "reason": "no geometry"}
        }
    ]);
    assert!(MoleculeRenderPlanV4::from_json(&duplicate_issue.to_string()).is_err());

    let mut equal_z: serde_json::Value =
        serde_json::from_str(&original.to_canonical_json().expect("serialize")).expect("value");
    let operation = equal_z["batches"][1]["content"]["content"]["operations"][0].clone();
    equal_z["batches"][1]["content"]["content"]["operations"] =
        json!([operation.clone(), operation]);
    assert!(MoleculeRenderPlanV4::from_json(&equal_z.to_string()).is_err());

    let mut signed_zero: serde_json::Value =
        serde_json::from_str(&original.to_canonical_json().expect("serialize")).expect("value");
    signed_zero["batches"][0]["content"]["content"]["atom_local_anchor"]["x"] = json!(-0.0);
    let normalized = MoleculeRenderPlanV4::from_json(&signed_zero.to_string()).expect("valid zero");
    assert!(
        !normalized
            .to_canonical_json()
            .expect("canonical JSON")
            .contains("-0.0")
    );

    let mut unknown: serde_json::Value =
        serde_json::from_str(&original.to_canonical_json().expect("serialize")).expect("value");
    unknown["issues"] = json!([{
        "target": {"document_object_id": target(0x93).document_object_id()},
        "paint_order": 9,
        "kind": {"kind": "unknown", "feature": "future"}
    }]);
    assert!(MoleculeRenderPlanV4::from_json(&unknown.to_string()).is_err());
}
