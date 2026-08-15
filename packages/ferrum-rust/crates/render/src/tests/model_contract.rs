use ferrum_core::{Identifier, RecordId, RecordKind};
use serde_json::json;

use crate::*;

fn target(kind: RecordKind, id: &str, source_order: u32) -> RenderTarget {
    let identifier = Identifier::new(id).expect("test source identifier is valid");
    RenderTarget::new(RecordId::from_source(kind, &identifier), source_order)
}

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("test point is finite")
}

fn size(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("test extent is positive and finite")
}

fn paint(value: &str) -> Paint {
    Paint::rgb24(Rgb24::new(value).expect("test rgb"))
}

fn run(text: &str, script: TextScript) -> TextRun {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Telex");
    let scale = match script {
        TextScript::Baseline => size(1.0),
        TextScript::Subscript | TextScript::Superscript => size(0.65),
    };
    let glyphs = metrics
        .v1_glyphs_for_run(text, size(12.0), scale)
        .expect("Telex glyphs");
    TextRun::new(text, script, point(0.0, 0.0), glyphs, scale).expect("run")
}

fn label() -> RenderOp {
    RenderOp::Text(
        TextOp::new(
            point(0.0, 0.0),
            vec![
                run("N", TextScript::Baseline),
                run("2", TextScript::Subscript),
                run("+", TextScript::Superscript),
            ],
            FontFace::telex_regular(),
            size(12.0),
            paint("000000"),
            2,
        )
        .expect("text operation"),
    )
}

fn line() -> RenderOp {
    RenderOp::Line(
        LineOp::new(
            point(1.0, 0.0),
            point(4.0, 0.0),
            size(1.0),
            paint("112233"),
            1,
        )
        .expect("line operation"),
    )
}

fn filled_path() -> RenderOp {
    RenderOp::Path(
        PathOpV2::new(
            vec![
                ScenePathCommandV2::MoveTo(point(1.0, 1.0)),
                ScenePathCommandV2::LineTo(point(5.0, 1.0)),
                ScenePathCommandV2::LineTo(point(3.0, 4.0)),
                ScenePathCommandV2::Close,
            ],
            None,
            Some(paint("112233")),
            2,
        )
        .expect("finite closed path"),
    )
}

fn plan() -> MoleculeRenderPlan {
    MoleculeRenderPlan::new(
        RenderProvenance::new(RenderRevision::new(8).expect("revision"), [8; 32]),
        vec![
            RenderBatch::new(
                target(RecordKind::Atom, "a1", 3),
                BatchSpace::AtomLocal {
                    anchor: point(2.0, 4.0),
                },
                vec![label()],
            )
            .expect("atom batch"),
            RenderBatch::new(
                target(RecordKind::Bond, "b1", 4),
                BatchSpace::Scene,
                vec![line()],
            )
            .expect("bond batch"),
        ],
        vec![],
    )
    .expect("render plan")
}

#[test]
fn plan_json_is_canonical_and_round_trips() {
    let original = plan();
    let first = original.to_canonical_json().expect("serialize");
    let restored = MoleculeRenderPlan::from_json(&first).expect("deserialize");
    let second = restored.to_canonical_json().expect("serialize again");
    assert_eq!(first, second);
    assert_eq!(restored, original);
    assert!(first.starts_with("{\"schema\":\"ferrum-render-plan-v2\""));
    assert!(first.contains("\"paint\":\"112233\""));
}

#[test]
fn scene_path_requires_closed_finite_drawable_painted_geometry() {
    let accepted = filled_path();
    assert!(
        RenderBatch::new(
            target(RecordKind::Bond, "path", 1),
            BatchSpace::Scene,
            vec![accepted],
        )
        .is_ok()
    );
    assert!(
        PathOpV2::new(
            vec![
                ScenePathCommandV2::MoveTo(point(1.0, 1.0)),
                ScenePathCommandV2::LineTo(point(5.0, 1.0)),
            ],
            None,
            Some(paint("112233")),
            0,
        )
        .is_err()
    );
    assert!(
        PathOpV2::new(
            vec![
                ScenePathCommandV2::MoveTo(point(1.0, 1.0)),
                ScenePathCommandV2::LineTo(point(5.0, 1.0)),
            ],
            None,
            None,
            0,
        )
        .is_err()
    );
    assert!(
        PathOpV2::new(
            vec![
                ScenePathCommandV2::MoveTo(point(1.0, 1.0)),
                ScenePathCommandV2::Close,
            ],
            None,
            Some(paint("112233")),
            0,
        )
        .is_err()
    );
    let path_plan = MoleculeRenderPlan::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
        vec![
            RenderBatch::new(
                target(RecordKind::Bond, "path-wire", 1),
                BatchSpace::Scene,
                vec![filled_path()],
            )
            .expect("path batch"),
        ],
        vec![],
    )
    .expect("path plan");
    let mut wire: serde_json::Value =
        serde_json::from_str(&path_plan.to_canonical_json().expect("serialize path plan"))
            .expect("path plan value");
    wire["batches"][0]["operations"][0]["operation"]["commands"][0]["command"]["x"] =
        serde_json::Value::Null;
    assert!(MoleculeRenderPlan::from_json(&wire.to_string()).is_err());
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
    wire["batches"][0]["operations"][0]["operation"]["size"] = json!(0.0);
    assert!(MoleculeRenderPlan::from_json(&wire.to_string()).is_err());

    let mut unknown = serde_json::from_str::<serde_json::Value>(
        &original.to_canonical_json().expect("serialize"),
    )
    .expect("value");
    unknown["batches"][0]["operations"][0]["kind"] = json!("polygon");
    assert!(MoleculeRenderPlan::from_json(&unknown.to_string()).is_err());

    let mut alias = serde_json::from_str::<serde_json::Value>(
        &original.to_canonical_json().expect("serialize"),
    )
    .expect("value");
    alias["batches"][0]["operations"][0]["operation"]
        .as_object_mut()
        .expect("object")
        .remove("z");
    assert!(MoleculeRenderPlan::from_json(&alias.to_string()).is_err());
}

#[test]
fn inbound_text_runs_reject_forged_or_non_scalar_telex_layouts() {
    let original = plan().to_canonical_json().expect("serialize");
    for text in ["e\u{301}", "\u{1F600}"] {
        let mut wire: serde_json::Value = serde_json::from_str(&original).expect("value");
        wire["batches"][0]["operations"][0]["operation"]["runs"][0]["text"] = json!(text);
        assert!(MoleculeRenderPlan::from_json(&wire.to_string()).is_err());
    }
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)
        .expect("pure-Rust parser opens verified Telex");
    let fi_glyphs = metrics
        .v1_glyphs_for_run("fi", size(12.0), size(1.0))
        .expect("Telex has scalar glyphs for the adversary text");
    let mut semantic_adversary: serde_json::Value = serde_json::from_str(&original).expect("value");
    semantic_adversary["batches"][0]["operations"][0]["operation"]["runs"][0]["text"] = json!("fi");
    semantic_adversary["batches"][0]["operations"][0]["operation"]["runs"][0]["glyphs"] =
        serde_json::to_value(fi_glyphs).expect("glyph placements serialize");
    assert!(MoleculeRenderPlan::from_json(&semantic_adversary.to_string()).is_err());
    let mut forged: serde_json::Value = serde_json::from_str(&original).expect("value");
    forged["batches"][0]["operations"][0]["operation"]["runs"][0]["glyphs"][0]["glyph_index"] =
        json!(999_999_u32);
    assert!(MoleculeRenderPlan::from_json(&forged.to_string()).is_err());
}

#[test]
fn coordinate_space_target_and_operation_grammar_is_closed() {
    assert!(
        RenderBatch::new(
            target(RecordKind::Bond, "b1", 1),
            BatchSpace::AtomLocal {
                anchor: point(0.0, 0.0)
            },
            vec![label()]
        )
        .is_err()
    );
    assert!(
        RenderBatch::new(
            target(RecordKind::Atom, "a1", 1),
            BatchSpace::Scene,
            vec![line()]
        )
        .is_err()
    );
    assert!(
        RenderBatch::new(
            target(RecordKind::Atom, "a1", 1),
            BatchSpace::AtomLocal {
                anchor: point(0.0, 0.0)
            },
            vec![line()]
        )
        .is_ok()
    );
    assert!(
        RenderBatch::new(
            target(RecordKind::Bond, "b1", 1),
            BatchSpace::Scene,
            vec![label()]
        )
        .is_err()
    );
}

#[test]
fn duplicate_durable_targets_are_rejected_even_when_projection_order_differs() {
    let atom = target(RecordKind::Atom, "a1", 1);
    let first = RenderBatch::new(
        atom.clone(),
        BatchSpace::AtomLocal {
            anchor: point(0.0, 0.0),
        },
        vec![label()],
    )
    .expect("batch");
    let second = RenderBatch::new(
        RenderTarget::new(atom.record_id().clone(), 9),
        BatchSpace::AtomLocal {
            anchor: point(1.0, 0.0),
        },
        vec![label()],
    )
    .expect("batch");
    assert!(
        MoleculeRenderPlan::new(
            RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
            vec![first, second],
            vec![]
        )
        .is_err()
    );
}

#[test]
fn batches_and_operations_require_explicit_stable_order() {
    let atom = RenderBatch::new(
        target(RecordKind::Atom, "a1", 4),
        BatchSpace::AtomLocal {
            anchor: point(0.0, 0.0),
        },
        vec![label()],
    )
    .expect("batch");
    let bond = RenderBatch::new(
        target(RecordKind::Bond, "b1", 3),
        BatchSpace::Scene,
        vec![line()],
    )
    .expect("batch");
    assert!(
        MoleculeRenderPlan::new(
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
        RenderBatch::new(
            target(RecordKind::Bond, "b2", 6),
            BatchSpace::Scene,
            vec![RenderOp::Line(back), RenderOp::Line(front)],
        )
        .is_err()
    );
}

#[test]
fn issues_are_validated_without_creating_partial_batches() {
    let issue = RenderIssue::new(
        target(RecordKind::Bond, "b1", 0),
        RenderIssueKind::UnsupportedFeature {
            feature: "aromatic bond".to_owned(),
        },
    )
    .expect("issue");
    let result = MoleculeRenderPlan::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
        vec![],
        vec![issue],
    )
    .expect("plan");
    assert!(result.batches().is_empty());
    assert_eq!(result.issues().len(), 1);
    assert_eq!(result.issues()[0].target().source_order(), 0);
    assert!(
        RenderIssue::new(
            target(RecordKind::Bond, "b2", 0),
            RenderIssueKind::UnrenderableTarget {
                reason: " ".to_owned()
            }
        )
        .is_err()
    );
}

#[test]
fn exclusions_and_batches_form_a_unique_ordered_target_partition() {
    let batch = RenderBatch::new(
        target(RecordKind::Atom, "a1", 3),
        BatchSpace::AtomLocal {
            anchor: point(0.0, 0.0),
        },
        vec![label()],
    )
    .expect("batch");
    let issue = RenderIssue::new(
        target(RecordKind::Bond, "b1", 4),
        RenderIssueKind::UnsupportedFeature {
            feature: "aromatic bond".to_owned(),
        },
    )
    .expect("issue");
    let plan = MoleculeRenderPlan::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
        vec![batch.clone()],
        vec![issue.clone()],
    )
    .expect("partition");
    let json = plan.to_canonical_json().expect("serialize");
    let restored = MoleculeRenderPlan::from_json(&json).expect("deserialize");
    assert_eq!(restored.issues()[0].target().source_order(), 4);

    let conflicting_issue = RenderIssue::new(
        target(RecordKind::Atom, "a1", 5),
        RenderIssueKind::UnrenderableTarget {
            reason: "missing label facts".to_owned(),
        },
    )
    .expect("issue");
    assert!(
        MoleculeRenderPlan::new(
            RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
            vec![batch.clone()],
            vec![conflicting_issue],
        )
        .is_err()
    );

    let duplicate_order_issue = RenderIssue::new(
        target(RecordKind::Bond, "b2", 3),
        RenderIssueKind::UnsupportedFeature {
            feature: "wedge bond".to_owned(),
        },
    )
    .expect("issue");
    assert!(
        MoleculeRenderPlan::new(
            RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
            vec![batch.clone()],
            vec![duplicate_order_issue],
        )
        .is_err()
    );

    let later = RenderIssue::new(
        target(RecordKind::Bond, "b3", 7),
        RenderIssueKind::UnsupportedFeature {
            feature: "double bond".to_owned(),
        },
    )
    .expect("issue");
    assert!(
        MoleculeRenderPlan::new(
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
        "kind": {"kind": "unsupported_feature", "feature": "aromatic bond"}
    }]);
    assert!(MoleculeRenderPlan::from_json(&wire.to_string()).is_err());

    let mut duplicate_issue: serde_json::Value =
        serde_json::from_str(&original.to_canonical_json().expect("serialize")).expect("value");
    duplicate_issue["issues"] = json!([
        {
            "target": {"record_id": target(RecordKind::Bond, "b3", 7).record_id(), "source_order": 7},
            "kind": {"kind": "unsupported_feature", "feature": "aromatic bond"}
        },
        {
            "target": {"record_id": target(RecordKind::Bond, "b4", 6).record_id(), "source_order": 6},
            "kind": {"kind": "unrenderable_target", "reason": "no geometry"}
        }
    ]);
    assert!(MoleculeRenderPlan::from_json(&duplicate_issue.to_string()).is_err());

    let mut equal_z: serde_json::Value =
        serde_json::from_str(&original.to_canonical_json().expect("serialize")).expect("value");
    let operation = equal_z["batches"][1]["operations"][0].clone();
    equal_z["batches"][1]["operations"] = json!([operation.clone(), operation]);
    assert!(MoleculeRenderPlan::from_json(&equal_z.to_string()).is_err());

    let mut signed_zero: serde_json::Value =
        serde_json::from_str(&original.to_canonical_json().expect("serialize")).expect("value");
    signed_zero["batches"][0]["coordinate_space"]["anchor"]["x"] = json!(-0.0);
    let normalized = MoleculeRenderPlan::from_json(&signed_zero.to_string()).expect("valid zero");
    assert!(
        !normalized
            .to_canonical_json()
            .expect("canonical JSON")
            .contains("-0.0")
    );

    let mut unknown: serde_json::Value =
        serde_json::from_str(&original.to_canonical_json().expect("serialize")).expect("value");
    unknown["issues"] = json!([{
        "target": {"record_id": target(RecordKind::Bond, "b5", 9).record_id(), "source_order": 9},
        "kind": {"kind": "unknown", "feature": "future"}
    }]);
    assert!(MoleculeRenderPlan::from_json(&unknown.to_string()).is_err());
}
