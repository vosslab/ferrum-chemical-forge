use ferrum_core::{Identifier, RecordId, RecordKind};

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

fn atom_bond_font() -> AtomLabelFontProfile {
    AtomLabelFontProfile::new(FontFace::telex_regular(), size(10.0), paint("000000"))
}

fn paint(value: &str) -> Paint {
    Paint::rgb24(Rgb24::new(value).expect("test rgb"))
}

fn atom_bond_metrics() -> VerifiedTelexGlyphMetrics {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    VerifiedTelexGlyphMetrics::new(&environment).expect("pure-Rust parser opens verified Telex")
}

fn atom_target(id: &str, source_order: u32, x: f64, y: f64) -> AtomRenderTarget {
    AtomRenderTarget::new(
        target(RecordKind::Atom, id, source_order),
        point(x, y),
        AtomLabelFacts::new("N", 1, 2).expect("label facts"),
        TargetVisibility::Visible,
    )
    .expect("atom target")
}

fn rendered_bond_lines(style: BondStyle) -> Vec<LineOp> {
    let first = atom_target("line-a1", 1, 0.0, 0.0);
    let second = atom_target("line-a2", 3, 40.0, 0.0);
    let bond = BondRenderTarget::new(
        target(RecordKind::Bond, "line-bond", 2),
        first.target().record_id().clone(),
        second.target().record_id().clone(),
        style,
        TargetVisibility::Visible,
    )
    .expect("bond target");
    let request = AtomBondRenderRequest::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [7; 32]),
        vec![first, second],
        vec![bond],
        atom_bond_font(),
        size(1.0),
        size(10.0),
        paint("112233"),
    )
    .expect("request");
    let plan = build_atom_bond_plan(&request, &atom_bond_metrics()).expect("plan");
    assert!(plan.issues().is_empty());
    plan.batches()[1]
        .operations()
        .iter()
        .map(|operation| match operation {
            RenderOp::Line(line) => line.clone(),
            _ => panic!("bond batch must contain only lines"),
        })
        .collect()
}

fn assert_near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.0e-12,
        "{actual} != {expected}"
    );
}

#[test]
fn atom_bond_request_emits_structured_labels_and_metric_clipped_single_bonds() {
    let first = atom_target("a1", 2, 0.0, 0.0);
    let second = atom_target("a2", 4, 40.0, 0.0);
    let bond = BondRenderTarget::new(
        target(RecordKind::Bond, "b1", 3),
        first.target().record_id().clone(),
        second.target().record_id().clone(),
        BondStyle::NormalSingle,
        TargetVisibility::Visible,
    )
    .expect("bond target");
    let request = AtomBondRenderRequest::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
        vec![first, second],
        vec![bond],
        atom_bond_font(),
        size(1.0),
        size(6.0),
        paint("112233"),
    )
    .expect("request");

    let plan = build_atom_bond_plan(&request, &atom_bond_metrics()).expect("plan");
    assert!(plan.issues().is_empty());
    assert_eq!(plan.batches().len(), 3);
    assert_eq!(plan.batches()[0].target().source_order(), 2);
    assert_eq!(plan.batches()[1].target().source_order(), 3);
    assert_eq!(plan.batches()[2].target().source_order(), 4);

    let RenderOp::Text(label) = &plan.batches()[0].operations()[0] else {
        panic!("first target must render text");
    };
    assert_eq!(label.runs()[0].text(), "N");
    assert_eq!(label.runs()[0].script(), TextScript::Baseline);
    assert_eq!(label.runs()[1].text(), "H");
    assert_eq!(label.runs()[2].text(), "2");
    assert_eq!(label.runs()[2].script(), TextScript::Subscript);
    assert_eq!(label.runs()[3].text(), "+");
    assert_eq!(label.runs()[3].script(), TextScript::Superscript);

    let RenderOp::Line(line) = &plan.batches()[1].operations()[0] else {
        panic!("middle target must render a line");
    };
    assert!(line.start().x() > 0.0);
    assert!(line.end().x() < 40.0);
    assert!(label.runs()[0].origin().x() < 0.0);
    assert_eq!(label.runs()[0].scale(), size(1.0));
    assert!(label.runs()[2].origin().y() < 0.0);
    assert_eq!(label.runs()[2].scale(), size(0.65));
    assert!(label.runs()[3].origin().y() > 0.0);
    assert_eq!(label.runs()[3].scale(), size(0.65));
    assert!(
        label
            .runs()
            .iter()
            .all(|run| run.glyphs().len() == run.text().chars().count())
    );
    assert_eq!(label.z(), 30);
    assert_eq!(line.z(), 10);
}

#[test]
fn visible_atom_number_is_a_separate_explicit_text_operation() {
    let number_font =
        AtomLabelFontProfile::new(FontFace::telex_regular(), size(9.0), paint("0000c8"));
    let atom = atom_target("numbered", 1, 10.0, 20.0).with_number_label(
        AtomNumberLabelFacts::new(27, point(8.0, -12.0), number_font)
            .expect("positive number label"),
    );
    let request = AtomBondRenderRequest::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [8; 32]),
        vec![atom],
        vec![],
        atom_bond_font(),
        size(1.0),
        size(6.0),
        paint("000000"),
    )
    .expect("request");

    let plan = build_atom_bond_plan(&request, &atom_bond_metrics()).expect("plan");
    assert!(plan.issues().is_empty());
    let operations = plan.batches()[0].operations();
    assert_eq!(operations.len(), 2);
    let RenderOp::Text(number) = &operations[1] else {
        panic!("second atom-local operation must be the number annotation");
    };
    assert_eq!(number.origin(), point(8.0, -12.0));
    assert_eq!(number.size(), size(9.0));
    assert_eq!(number.paint().color().as_str(), "0000c8");
    assert_eq!(number.z(), 40);
    assert_eq!(number.runs().len(), 1);
    assert_eq!(number.runs()[0].text(), "27");
    assert_eq!(number.runs()[0].script(), TextScript::Baseline);
    assert_eq!(number.runs()[0].glyphs().len(), 2);
}

#[test]
fn atom_marks_lower_to_closed_semantic_primitives_without_toolkit_defaults() {
    let cases = [
        (AtomMarkRenderKind::Plus, vec!["ellipse", "line", "line"]),
        (AtomMarkRenderKind::Radical, vec!["ellipse"]),
        (AtomMarkRenderKind::Biradical, vec!["ellipse", "ellipse"]),
        (AtomMarkRenderKind::Electronpair, vec!["line"]),
        (
            AtomMarkRenderKind::DottedElectronpair,
            vec!["ellipse", "ellipse"],
        ),
        (AtomMarkRenderKind::PzOrbital, vec!["ellipse", "ellipse"]),
    ];
    for (kind, expected) in cases {
        let mark = AtomMarkRenderFacts::new(
            kind,
            point(8.0, -12.0),
            45.0,
            size(10.0),
            true,
            size(1.0),
            paint("112233"),
        )
        .expect("mark facts");
        let atom = atom_target("marked", 1, 10.0, 20.0).with_marks(vec![mark]);
        let request = AtomBondRenderRequest::new(
            RenderProvenance::new(RenderRevision::new(1).expect("revision"), [9; 32]),
            vec![atom],
            vec![],
            atom_bond_font(),
            size(1.0),
            size(6.0),
            paint("000000"),
        )
        .expect("request");
        let plan = build_atom_bond_plan(&request, &atom_bond_metrics()).expect("plan");
        assert!(plan.issues().is_empty());
        let actual = plan.batches()[0].operations()[1..]
            .iter()
            .map(|operation| match operation {
                RenderOp::Line(_) => "line",
                RenderOp::Ellipse(_) => "ellipse",
                RenderOp::Text(_) | RenderOp::Mask(_) => "unexpected",
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{kind:?}");
    }
}

#[test]
fn normal_double_and_triple_bonds_emit_parallel_symmetric_bounded_lines() {
    for (style, expected_offsets) in [
        (BondStyle::Double, &[-5.0, 5.0][..]),
        (BondStyle::Triple, &[-7.0, 0.0, 7.0][..]),
    ] {
        let lines = rendered_bond_lines(style);
        assert_eq!(lines.len(), expected_offsets.len());
        for (index, (line, expected_offset)) in
            lines.iter().zip(expected_offsets.iter()).enumerate()
        {
            assert_near(line.start().y(), *expected_offset);
            assert_near(line.end().y(), *expected_offset);
            assert!(line.start().x() >= 0.0);
            assert!(line.end().x() <= 40.0);
            assert!(line.start().x() < line.end().x());
            assert_eq!(line.width(), size(1.0));
            assert_eq!(line.paint().color().as_str(), "112233");
            assert_eq!(line.z(), 10 + i32::try_from(index).expect("small index"));
        }
    }
}

#[test]
fn opaque_label_masks_have_explicit_paint_and_fixed_molecule_plane_order() {
    let first = atom_target("a1", 1, 0.0, 0.0);
    let request = AtomBondRenderRequest::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [2; 32]),
        vec![first],
        vec![],
        atom_bond_font().with_label_mask(paint("ffffff")),
        size(1.0),
        size(6.0),
        paint("000000"),
    )
    .expect("request");

    let plan = build_atom_bond_plan(&request, &atom_bond_metrics()).expect("plan");
    let operations = plan.batches()[0].operations();
    let RenderOp::Mask(mask) = &operations[0] else {
        panic!("opaque label mask must be explicit");
    };
    let RenderOp::Text(text) = &operations[1] else {
        panic!("label text must follow its mask");
    };
    assert_eq!(mask.z(), 20);
    assert_eq!(text.z(), 30);
    assert_eq!(mask.paint().color().as_str(), "ffffff");
}

#[test]
fn atom_bond_builder_returns_explicit_issues_for_invisible_unsupported_and_unrenderable_targets() {
    let visible = atom_target("a1", 1, 0.0, 0.0);
    let hidden = AtomRenderTarget::new(
        target(RecordKind::Atom, "a2", 2),
        point(10.0, 0.0),
        AtomLabelFacts::new("O", 0, 0).expect("label facts"),
        TargetVisibility::Hidden {
            reason: "collapsed group".to_owned(),
        },
    )
    .expect("atom target");
    let unsupported = BondRenderTarget::new(
        target(RecordKind::Bond, "b1", 3),
        visible.target().record_id().clone(),
        hidden.target().record_id().clone(),
        BondStyle::Aromatic,
        TargetVisibility::Visible,
    )
    .expect("bond target");
    let missing_endpoint = BondRenderTarget::new(
        target(RecordKind::Bond, "b2", 4),
        visible.target().record_id().clone(),
        target(RecordKind::Atom, "a3", 9).record_id().clone(),
        BondStyle::NormalSingle,
        TargetVisibility::Visible,
    )
    .expect("bond target");
    let request = AtomBondRenderRequest::new(
        RenderProvenance::new(RenderRevision::new(2).expect("revision"), [3; 32]),
        vec![visible, hidden],
        vec![unsupported, missing_endpoint],
        atom_bond_font(),
        size(1.0),
        size(6.0),
        paint("000000"),
    )
    .expect("request");

    let plan = build_atom_bond_plan(&request, &atom_bond_metrics()).expect("plan");
    assert_eq!(plan.batches().len(), 1);
    assert_eq!(plan.issues().len(), 3);
    assert!(matches!(
        plan.issues()[0].kind(),
        RenderIssueKind::UnsupportedFeature { feature } if feature.contains("invisible atom target")
    ));
    assert!(matches!(
        plan.issues()[1].kind(),
        RenderIssueKind::UnsupportedFeature { feature } if feature == "aromatic bond"
    ));
    assert!(matches!(
        plan.issues()[2].kind(),
        RenderIssueKind::UnrenderableTarget { reason } if reason.contains("second bond endpoint")
    ));
}

#[test]
fn atom_bond_builder_rejects_coincident_and_extreme_bond_geometry_without_partial_lines() {
    let coincident_first = atom_target("a1", 1, 0.0, 0.0);
    let coincident_second = atom_target("a2", 2, 0.0, 0.0);
    let extreme_first = atom_target("a3", 4, -f64::MAX, 0.0);
    let extreme_second = atom_target("a4", 5, f64::MAX, 0.0);
    let coincident = BondRenderTarget::new(
        target(RecordKind::Bond, "b1", 3),
        coincident_first.target().record_id().clone(),
        coincident_second.target().record_id().clone(),
        BondStyle::NormalSingle,
        TargetVisibility::Visible,
    )
    .expect("bond target");
    let extreme = BondRenderTarget::new(
        target(RecordKind::Bond, "b2", 6),
        extreme_first.target().record_id().clone(),
        extreme_second.target().record_id().clone(),
        BondStyle::NormalSingle,
        TargetVisibility::Visible,
    )
    .expect("bond target");
    let request = AtomBondRenderRequest::new(
        RenderProvenance::new(RenderRevision::new(3).expect("revision"), [4; 32]),
        vec![
            coincident_first,
            coincident_second,
            extreme_first,
            extreme_second,
        ],
        vec![coincident, extreme],
        atom_bond_font(),
        size(1.0),
        size(6.0),
        paint("000000"),
    )
    .expect("request");

    let plan = build_atom_bond_plan(&request, &atom_bond_metrics()).expect("plan");
    assert_eq!(plan.batches().len(), 4);
    assert_eq!(plan.issues().len(), 2);
    for issue in plan.issues() {
        assert!(matches!(
            issue.kind(),
            RenderIssueKind::UnrenderableTarget { reason }
                if reason.contains("coincident") || reason.contains("not representable")
        ));
    }
}

#[test]
fn atom_bond_builder_rejects_touching_or_overlapping_label_clips_in_every_direction() {
    let metrics = atom_bond_metrics();
    for (suffix, x, y) in [("horizontal", 16.5, 0.0), ("diagonal", 6.0, 6.0)] {
        let first = atom_target(&format!("a1-{suffix}"), 1, 0.0, 0.0);
        let second = atom_target(&format!("a2-{suffix}"), 3, x, y);
        let bond = BondRenderTarget::new(
            target(RecordKind::Bond, &format!("b-{suffix}"), 2),
            first.target().record_id().clone(),
            second.target().record_id().clone(),
            BondStyle::NormalSingle,
            TargetVisibility::Visible,
        )
        .expect("bond");
        let request = AtomBondRenderRequest::new(
            RenderProvenance::new(RenderRevision::new(10).expect("revision"), [5; 32]),
            vec![first, second],
            vec![bond],
            atom_bond_font(),
            size(1.0),
            size(6.0),
            paint("000000"),
        )
        .expect("request");
        let plan = build_atom_bond_plan(&request, &metrics).expect("plan");
        assert_eq!(plan.batches().len(), 2, "{suffix}");
        assert!(matches!(
            plan.issues()[0].kind(),
            RenderIssueKind::UnrenderableTarget { reason }
                if reason.contains("no positive visible bond segment")
        ));
    }
}

#[test]
fn deterministic_layout_bounds_describe_the_same_positioned_runs_for_multi_unit_charge() {
    let label = AtomLabelFacts::new("N", -3, 2).expect("facts");
    let layout = atom_bond_metrics()
        .layout_atom_label(&label, &atom_bond_font())
        .expect("layout");
    assert_eq!(layout.runs().len(), 4);
    assert_eq!(layout.runs()[2].text(), "2");
    assert_eq!(layout.runs()[2].script(), TextScript::Subscript);
    assert_eq!(layout.runs()[3].text(), "3-");
    assert_eq!(layout.runs()[3].script(), TextScript::Superscript);
    assert!(layout.bounds().min_x() <= 0.0);
    assert!(layout.bounds().max_x() >= 0.0);
    assert!(layout.bounds().min_y() <= 0.0);
    assert!(layout.bounds().max_y() >= 0.0);
    // GlyphBounds is the atom-anchor clipping envelope.  Superscript and
    // subscript baselines may lie outside their visible outlines, so their run
    // origins are not themselves bounds facts.
}

#[test]
fn atom_bond_request_requires_typed_kinds_unique_order_and_explicit_label_facts() {
    assert!(AtomLabelFacts::new("cl", 0, 0).is_err());
    assert!(
        AtomRenderTarget::new(
            target(RecordKind::Bond, "b1", 1),
            point(0.0, 0.0),
            AtomLabelFacts::new("C", 0, 0).expect("facts"),
            TargetVisibility::Visible,
        )
        .is_err()
    );

    let first = atom_target("a1", 1, 0.0, 0.0);
    let second = atom_target("a2", 1, 2.0, 0.0);
    assert!(
        AtomBondRenderRequest::new(
            RenderProvenance::new(RenderRevision::new(1).expect("revision"), [6; 32]),
            vec![first, second],
            vec![],
            atom_bond_font(),
            size(1.0),
            size(6.0),
            paint("000000"),
        )
        .is_err()
    );
}
