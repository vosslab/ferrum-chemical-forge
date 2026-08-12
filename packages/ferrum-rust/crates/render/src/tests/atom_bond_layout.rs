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
    AtomLabelFontProfile::new(FontFace::telex_regular(), size(10.0), Paint::Foreground)
}

fn atom_bond_metrics() -> DeterministicGlyphMetrics {
    DeterministicGlyphMetrics::new(size(0.5), size(0.8), size(0.2))
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
        RenderRevision::new(1).expect("revision"),
        vec![first, second],
        vec![bond],
        atom_bond_font(),
        size(1.0),
        Paint::Rgb24(Rgb24::new("112233").expect("rgb")),
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
    assert_eq!(line.start(), point(8.25, 0.0));
    assert_eq!(line.end(), point(31.75, 0.0));
    assert_eq!(label.runs()[0].origin(), point(-8.25, 0.0));
    assert_eq!(label.runs()[0].scale(), size(1.0));
    assert_eq!(label.runs()[2].origin(), point(1.75, -1.6));
    assert_eq!(label.runs()[2].scale(), size(0.65));
    assert_eq!(label.runs()[3].origin(), point(5.0, 4.4));
    assert_eq!(label.runs()[3].scale(), size(0.65));
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
        RenderRevision::new(2).expect("revision"),
        vec![visible, hidden],
        vec![unsupported, missing_endpoint],
        atom_bond_font(),
        size(1.0),
        Paint::Foreground,
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
        RenderRevision::new(3).expect("revision"),
        vec![
            coincident_first,
            coincident_second,
            extreme_first,
            extreme_second,
        ],
        vec![coincident, extreme],
        atom_bond_font(),
        size(1.0),
        Paint::Foreground,
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
            RenderRevision::new(10).expect("revision"),
            vec![first, second],
            vec![bond],
            atom_bond_font(),
            size(1.0),
            Paint::Foreground,
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
    assert!(layout.runs().iter().all(|run| {
        run.origin().x() >= layout.bounds().min_x()
            && run.origin().x() <= layout.bounds().max_x()
            && run.origin().y() >= layout.bounds().min_y()
            && run.origin().y() <= layout.bounds().max_y()
    }));
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
            RenderRevision::new(1).expect("revision"),
            vec![first, second],
            vec![],
            atom_bond_font(),
            size(1.0),
            Paint::Foreground,
        )
        .is_err()
    );
}
