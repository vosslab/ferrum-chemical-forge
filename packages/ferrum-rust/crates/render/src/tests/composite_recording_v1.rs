//! Behavior checks for private whole-document composite recording.

use ferrum_core::{Identifier, RecordId, RecordKind};

use crate::composite_recording_v1::RecordingSink;
use crate::draw_stream_v1::{
    DrawEllipseV1, DrawMetadataV1, DrawPathCommandV1, DrawPathV1, DrawRootKindV1, DrawSinkV1,
    DrawStyleV1,
};
use crate::*;

fn budget() -> CompositeRecordingBudgetV1 {
    CompositeRecordingBudgetV1 {
        max_roots: 8,
        max_target_groups: 8,
        max_events: 64,
        max_path_commands: 64,
        max_transform_depth: 8,
        max_text_scopes: 8,
        max_copied_string_bytes: 128,
    }
}

fn sink(budget: CompositeRecordingBudgetV1) -> RecordingSink {
    RecordingSink::for_test(
        RenderProvenance::new(RenderRevision::new(0).expect("revision"), [0; 32]),
        budget,
    )
}

fn page() -> RenderViewportV1 {
    RenderViewportV1::new(0.0, 0.0, 1.0, 1.0).expect("page")
}

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("point")
}

fn target() -> RenderTarget {
    RenderTarget::new(
        RecordId::from_source(
            RecordKind::Bond,
            &Identifier::new("bond").expect("identifier"),
        ),
        1,
    )
}

fn open_target(sink: &mut RecordingSink) {
    sink.begin_page(page()).expect("page begin");
    sink.begin_root_with_kind(1, "root", DrawRootKindV1::Molecule)
        .expect("root begin");
    sink.begin_molecule_target_group(&target(), BatchSpace::Scene)
        .expect("target begin");
}

#[test]
fn recording_budget_rejects_each_structural_resource() {
    let mut limits = budget();
    limits.max_roots = 0;
    let mut recorder = sink(limits);
    recorder.begin_page(page()).expect("page begin");
    assert_eq!(
        recorder.begin_root_with_kind(1, "root", DrawRootKindV1::Molecule),
        Err(CompositeRecordingErrorV1::BudgetExceeded {
            resource: CompositeRecordingResourceV1::Roots,
        })
    );

    let mut limits = budget();
    limits.max_target_groups = 0;
    let mut recorder = sink(limits);
    recorder.begin_page(page()).expect("page begin");
    recorder
        .begin_root_with_kind(1, "root", DrawRootKindV1::Molecule)
        .expect("root");
    assert_eq!(
        recorder.begin_molecule_target_group(&target(), BatchSpace::Scene),
        Err(CompositeRecordingErrorV1::BudgetExceeded {
            resource: CompositeRecordingResourceV1::TargetGroups,
        })
    );

    let mut limits = budget();
    limits.max_events = 0;
    assert_eq!(
        sink(limits).begin_page(page()),
        Err(CompositeRecordingErrorV1::BudgetExceeded {
            resource: CompositeRecordingResourceV1::Events,
        })
    );

    let mut limits = budget();
    limits.max_path_commands = 0;
    let mut recorder = sink(limits);
    open_target(&mut recorder);
    let path = DrawPathV1 {
        commands: vec![DrawPathCommandV1::MoveTo(point(0.0, 0.0))],
    };
    assert_eq!(
        recorder.draw_path(
            &path,
            DrawStyleV1 {
                fill: None,
                stroke: None,
                fill_rule: None,
            },
            DrawMetadataV1::MoleculeLine { z: 1 },
        ),
        Err(CompositeRecordingErrorV1::BudgetExceeded {
            resource: CompositeRecordingResourceV1::PathCommands,
        })
    );

    let mut limits = budget();
    limits.max_transform_depth = 0;
    let mut recorder = sink(limits);
    open_target(&mut recorder);
    assert_eq!(
        recorder.save(),
        Err(CompositeRecordingErrorV1::BudgetExceeded {
            resource: CompositeRecordingResourceV1::TransformDepth,
        })
    );

    let mut limits = budget();
    limits.max_text_scopes = 0;
    let mut recorder = sink(limits);
    recorder.begin_page(page()).expect("page begin");
    recorder
        .begin_root_with_kind(1, "root", DrawRootKindV1::Text)
        .expect("root");
    assert_eq!(
        recorder.begin_document_text(),
        Err(CompositeRecordingErrorV1::BudgetExceeded {
            resource: CompositeRecordingResourceV1::TextScopes,
        })
    );

    let mut limits = budget();
    limits.max_copied_string_bytes = 0;
    let mut recorder = sink(limits);
    recorder.begin_page(page()).expect("page begin");
    assert_eq!(
        recorder.begin_root_with_kind(1, "root", DrawRootKindV1::Molecule),
        Err(CompositeRecordingErrorV1::BudgetExceeded {
            resource: CompositeRecordingResourceV1::CopiedStringBytes,
        })
    );

    let mut limits = budget();
    limits.max_copied_string_bytes = "root".len();
    let mut recorder = sink(limits);
    recorder.begin_page(page()).expect("page begin");
    recorder
        .begin_root_with_kind(1, "root", DrawRootKindV1::Molecule)
        .expect("root");
    assert_eq!(
        recorder.begin_molecule_target_group(&target(), BatchSpace::Scene),
        Err(CompositeRecordingErrorV1::BudgetExceeded {
            resource: CompositeRecordingResourceV1::CopiedStringBytes,
        })
    );
}

#[test]
fn recorder_rejects_unbalanced_scope_and_keeps_rotated_ellipse_style() {
    let mut recorder = sink(budget());
    assert_eq!(
        recorder.restore(),
        Err(CompositeRecordingErrorV1::InvalidStream)
    );
    open_target(&mut recorder);
    let paint = Paint::rgb24(Rgb24::new("112233").expect("paint"));
    recorder
        .draw_ellipse(
            DrawEllipseV1 {
                center: point(0.0, 0.0),
                radius_x: PositiveFinite::new(1.0).expect("radius"),
                radius_y: PositiveFinite::new(2.0).expect("radius"),
                rotation_degrees: 30.0,
            },
            DrawStyleV1 {
                fill: Some(&paint),
                stroke: None,
                fill_rule: None,
            },
            DrawMetadataV1::MoleculeEllipse { z: 1 },
        )
        .expect("ellipse");
    let event = recorder.test_events().last().expect("ellipse event");
    assert!(matches!(
        event,
        CompositeRecordingEventV1::Ellipse {
            rotation_degrees,
            style,
            ..
        } if *rotation_degrees == 30.0
            && matches!(
                style.fill().map(CompositeFillV1::rule),
                Some(CompositeFillRuleV1::NonZero)
            )
    ));
}

#[test]
fn recorder_requires_one_completed_page_and_rejects_a_second_page() {
    assert_eq!(
        sink(budget()).finish_for_test(),
        Err(CompositeRecordingErrorV1::InvalidStream)
    );
    let mut recorder = sink(budget());
    recorder.begin_page(page()).expect("first page");
    recorder.finish_page().expect("first page finish");
    assert_eq!(
        recorder.begin_page(page()),
        Err(CompositeRecordingErrorV1::InvalidStream)
    );
}
