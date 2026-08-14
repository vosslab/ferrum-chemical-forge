//! Behavior checks for the private renderer stream.

use ferrum_core::{Identifier, RecordId, RecordKind};

use crate::authored_direct_glycosidic_haworth::{
    AuthoredDirectGlycosidicHaworthDrawOpV1, AuthoredDirectGlycosidicHaworthRenderPlanV1,
};
use crate::direct_glycosidic_haworth::DirectGlycosidicHaworthPathCommandV1;
use crate::draw_stream_v1::{
    DrawEllipseV1, DrawLineCapV1, DrawMetadataV1, DrawPathCommandV1, DrawPathV1, DrawRectV1,
    DrawSinkV1, DrawStreamErrorV1, DrawStyleV1, lower_direct_glycosidic_haworth_plan_to_sink_v1,
    lower_document_plan_to_sink_v1, lower_document_render_composite_to_sink_v1,
};
use crate::*;

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("finite test point")
}

fn size(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("positive test extent")
}

fn paint(value: &str) -> Paint {
    Paint::rgb24(Rgb24::new(value).expect("valid test paint"))
}

fn target(kind: RecordKind, source: &str, source_order: u32) -> RenderTarget {
    RenderTarget::new(
        RecordId::from_source(
            kind,
            &Identifier::new(source).expect("valid test identifier"),
        ),
        source_order,
    )
}

fn mixed_plan() -> DocumentRenderPlanV1 {
    let provenance = RenderProvenance::new(RenderRevision::new(1).expect("revision"), [42; 32]);
    let molecule = MoleculeRenderPlan::new(
        provenance,
        vec![
            RenderBatch::new(
                target(RecordKind::Atom, "stream-local", 1),
                BatchSpace::AtomLocal {
                    anchor: point(3.0, 4.0),
                },
                vec![
                    RenderOp::Mask(
                        MaskOp::new(point(-1.0, -1.0), size(2.0), size(2.0), paint("ffffff"), 1)
                            .expect("mask"),
                    ),
                    RenderOp::Ellipse(
                        EllipseOp::new(
                            point(0.0, 0.0),
                            size(1.0),
                            size(2.0),
                            30.0,
                            Some(size(0.5)),
                            Some(paint("112233")),
                            Some(paint("aabbcc")),
                            2,
                        )
                        .expect("ellipse"),
                    ),
                ],
            )
            .expect("local batch"),
            RenderBatch::new(
                target(RecordKind::Bond, "stream-scene", 2),
                BatchSpace::Scene,
                vec![RenderOp::Line(
                    LineOp::new(
                        point(1.0, 2.0),
                        point(5.0, 2.0),
                        size(1.0),
                        paint("445566"),
                        1,
                    )
                    .expect("line"),
                )],
            )
            .expect("scene batch"),
        ],
        vec![],
    )
    .expect("molecule plan");
    let metrics =
        VerifiedTelexGlyphMetrics::new(&FerrumFontEnvironmentV1::load().expect("verified Telex"))
            .expect("metrics");
    let plus = metrics
        .layout_centered_plus(size(10.0), paint("000000"))
        .expect("plus layout");
    let text = DocumentTextOpV1::fixed(
        point(20.0, 10.0),
        plus.operation().clone(),
        plus.bounds(),
        Some(paint("ddeeff")),
    )
    .expect("document text");
    let vector = DocumentVectorRootV1::new(vec![
        DocumentVectorOpV1::path(
            vec![
                PathCommandV1::MoveTo(point(1.0, 1.0)),
                PathCommandV1::CubicTo {
                    control_1: point(2.0, 4.0),
                    control_2: point(4.0, 4.0),
                    end: point(5.0, 1.0),
                },
                PathCommandV1::Close,
            ],
            Some(StrokeV1::new(paint("102030"), size(1.5))),
            Some(paint("aabbcc")),
        )
        .expect("cubic vector path"),
    ])
    .expect("vector root");
    DocumentRenderPlanV1::new(
        provenance,
        RenderViewportV1::new(0.0, 0.0, 100.0, 80.0).expect("page"),
        vec![
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                1,
                DocumentRenderIdentityV1::projection_local("molecule").expect("identity"),
                DocumentRenderContentV1::Molecule(molecule),
            )),
            DocumentRenderOutcomeV1::Exclusion(
                DocumentRenderExclusionV1::new(
                    2,
                    DocumentRenderIdentityV1::projection_local("excluded").expect("identity"),
                    "not_yet_lowered:test",
                )
                .expect("exclusion"),
            ),
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                3,
                DocumentRenderIdentityV1::projection_local("text").expect("identity"),
                DocumentRenderContentV1::Text(text),
            )),
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                4,
                DocumentRenderIdentityV1::projection_local("vector").expect("identity"),
                DocumentRenderContentV1::Vector(vector),
            )),
        ],
    )
    .expect("mixed document plan")
}

fn composite_plan() -> DocumentRenderCompositeV1 {
    let provenance = RenderProvenance::new(RenderRevision::new(9).expect("revision"), [9; 32]);
    let selected_ordinary = target(RecordKind::Bond, "selected-ordinary", 2);
    let retained_bond = target(RecordKind::Bond, "retained-bond", 3);
    let selected_q = target(RecordKind::Bond, "selected-q", 4);
    let retained_issue = target(RecordKind::Bond, "retained-issue", 5);
    let selected_w = target(RecordKind::Bond, "selected-w", 6);
    let metrics =
        VerifiedTelexGlyphMetrics::new(&FerrumFontEnvironmentV1::load().expect("verified Telex"))
            .expect("metrics");
    let label = metrics
        .layout_centered_plus(size(10.0), paint("102030"))
        .expect("atom label")
        .operation()
        .clone();
    let atom = RenderBatch::new(
        target(RecordKind::Atom, "atom", 1),
        BatchSpace::AtomLocal {
            anchor: point(3.0, 4.0),
        },
        vec![
            RenderOp::Mask(
                MaskOp::new(point(-1.0, -1.0), size(2.0), size(2.0), paint("ffffff"), 1)
                    .expect("mask"),
            ),
            RenderOp::Text(label),
        ],
    )
    .expect("atom batch");
    let line = |target: RenderTarget, x: f64| {
        RenderBatch::new(
            target,
            BatchSpace::Scene,
            vec![RenderOp::Line(
                LineOp::new(
                    point(x, 0.0),
                    point(x + 1.0, 0.0),
                    size(1.0),
                    paint("445566"),
                    1,
                )
                .expect("line"),
            )],
        )
        .expect("bond batch")
    };
    let molecule = MoleculeRenderPlan::new(
        provenance,
        vec![
            atom,
            line(selected_ordinary.clone(), 2.0),
            line(retained_bond.clone(), 3.0),
            line(selected_w.clone(), 6.0),
        ],
        vec![
            RenderIssue::new(
                selected_q.clone(),
                RenderIssueKind::UnrenderableTarget {
                    reason: "selected q".to_owned(),
                },
            )
            .expect("selected issue"),
            RenderIssue::new(
                retained_issue,
                RenderIssueKind::UnrenderableTarget {
                    reason: "retained issue".to_owned(),
                },
            )
            .expect("retained issue"),
        ],
    )
    .expect("molecule plan");
    let established = DocumentRenderPlanV1::new(
        provenance,
        RenderViewportV1::new(0.0, 0.0, 20.0, 10.0).expect("page"),
        vec![
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                1,
                DocumentRenderIdentityV1::durable("molecule").expect("root"),
                DocumentRenderContentV1::Molecule(molecule),
            )),
            DocumentRenderOutcomeV1::Exclusion(
                DocumentRenderExclusionV1::new(
                    2,
                    DocumentRenderIdentityV1::projection_local("excluded").expect("identity"),
                    "still intentionally excluded",
                )
                .expect("exclusion"),
            ),
            mixed_plan().outcomes()[2].clone(),
            mixed_plan().outcomes()[3].clone(),
        ],
    )
    .expect("established plan");
    let black = paint("000000");
    let direct = AuthoredDirectGlycosidicHaworthRenderPlanV1::test_plan(
        provenance,
        black,
        vec![
            AuthoredDirectGlycosidicHaworthDrawOpV1::OrdinaryLine {
                bond: selected_ordinary.record_id().clone(),
                authored_child_order: 2,
                endpoints: [point(2.0, 0.0), point(3.0, 0.0)],
                width: size(1.0),
            },
            AuthoredDirectGlycosidicHaworthDrawOpV1::HaworthFrontStroke {
                bond: selected_q.record_id().clone(),
                authored_child_order: 4,
                endpoints: [point(4.0, 0.0), point(5.0, 0.0)],
                width: size(2.0),
            },
            AuthoredDirectGlycosidicHaworthDrawOpV1::RoundedFrontWedge {
                bond: selected_w.record_id().clone(),
                authored_child_order: 6,
                tip: point(6.0, 0.0),
                base: point(7.0, 0.0),
                width: size(2.0),
                commands: vec![
                    DirectGlycosidicHaworthPathCommandV1::MoveTo(point(6.0, 0.0)),
                    DirectGlycosidicHaworthPathCommandV1::LineTo(point(7.0, 1.0)),
                    DirectGlycosidicHaworthPathCommandV1::LineTo(point(7.0, -1.0)),
                    DirectGlycosidicHaworthPathCommandV1::Close,
                ],
            },
        ],
    );
    compose_document_bond_replacement_v1(
        established,
        DocumentRenderIdentityV1::durable("molecule").expect("root"),
        1,
        vec![selected_ordinary, selected_q, selected_w],
        direct,
    )
    .expect("composite")
}

fn recording_budget() -> CompositeRecordingBudgetV1 {
    CompositeRecordingBudgetV1 {
        max_roots: 32,
        max_target_groups: 64,
        max_events: 4096,
        max_path_commands: 4096,
        max_transform_depth: 32,
        max_text_scopes: 64,
        max_copied_string_bytes: 65_536,
    }
}

fn presentation_plan(operation: PresentationTextOp, bounds: GlyphBounds) -> DocumentRenderPlanV1 {
    let text = DocumentTextOpV1::presentation(point(10.0, 20.0), operation, bounds, None)
        .expect("presentation text");
    DocumentRenderPlanV1::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [61; 32]),
        RenderViewportV1::new(0.0, 0.0, 120.0, 80.0).expect("page"),
        vec![DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
            1,
            DocumentRenderIdentityV1::projection_local("presentation").expect("identity"),
            DocumentRenderContentV1::Text(text),
        ))],
    )
    .expect("document plan")
}

fn laid_out_presentation(text: &str) -> PresentationTextLayout {
    let environment = FerrumFontEnvironmentV1::load().expect("verified Telex");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment).expect("Telex metrics");
    metrics
        .layout_presentation_text(
            &[PresentationTextSourceRun::new(text, TextScript::Baseline).expect("source")],
            size(18.0),
            paint("123456"),
        )
        .expect("presentation layout")
}

#[test]
fn composite_recording_retains_whole_page_paint_and_authenticated_direct_groups() {
    let recording = record_document_render_composite_v1(&composite_plan(), recording_budget())
        .expect("recording");
    let events = recording.events();
    assert!(events.iter().any(|event| matches!(
        event,
        CompositeRecordingEventV1::Path {
            kind: CompositePaintKindV1::DirectGlycosidicQ1,
            style,
            ..
        } if matches!(style.stroke().map(CompositeStrokeV1::cap), Some(CompositeLineCapV1::Round))
    )));
    assert!(events.windows(2).any(|pair| matches!(
        pair,
        [
            CompositeRecordingEventV1::RootBegin {
                kind: CompositeRootKindV1::Text,
                ..
            },
            CompositeRecordingEventV1::DocumentTextBegin
        ]
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        CompositeRecordingEventV1::FillRect {
            kind: CompositePaintKindV1::DocumentTextBackground,
            ..
        }
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        CompositeRecordingEventV1::Path {
            kind: CompositePaintKindV1::DocumentVectorPath,
            style,
            ..
        }
            if matches!(style.fill().map(CompositeFillV1::rule), Some(CompositeFillRuleV1::EvenOdd))
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        CompositeRecordingEventV1::TargetBegin { direct: true, .. }
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        CompositeRecordingEventV1::FillRect {
            kind: CompositePaintKindV1::MoleculeMask { .. },
            ..
        }
    )));
}

#[test]
fn composite_recording_budget_rejects_a_required_structural_resource() {
    let mut budget = recording_budget();
    budget.max_target_groups = 0;
    assert_eq!(
        record_document_render_composite_v1(&composite_plan(), budget),
        Err(CompositeRecordingErrorV1::BudgetExceeded {
            resource: CompositeRecordingResourceV1::TargetGroups,
        })
    );
}

#[derive(Default)]
struct RecordingSink {
    events: Vec<String>,
    direct_events: Vec<DirectDrawEvent>,
    empty_path_published: bool,
    refuse_paths: bool,
    refuse_concat: bool,
    refuse_ellipses: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DirectDrawEvent {
    Ordinary(DrawLineCapV1),
    Q1(DrawLineCapV1),
    W1 { fill: bool, stroke: bool },
}

#[derive(Debug)]
enum RecordingError {
    Refused,
}

impl DrawSinkV1 for RecordingSink {
    type Error = RecordingError;

    fn begin_page(&mut self, page: RenderViewportV1) -> Result<(), Self::Error> {
        self.events
            .push(format!("page:{}x{}", page.width(), page.height()));
        Ok(())
    }
    fn begin_root(&mut self, source_order: u32, _: &str) -> Result<(), Self::Error> {
        self.events.push(format!("root:{source_order}"));
        Ok(())
    }
    fn end_root(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_molecule_batch(
        &mut self,
        source_order: u32,
        space: BatchSpace,
    ) -> Result<(), Self::Error> {
        self.events.push(format!("batch:{source_order}:{space:?}"));
        Ok(())
    }
    fn end_molecule_batch(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_document_text(&mut self) -> Result<(), Self::Error> {
        self.events.push("text".to_owned());
        Ok(())
    }
    fn end_document_text(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_text_operation(&mut self, _: i32, _: &Paint) -> Result<(), Self::Error> {
        Ok(())
    }
    fn end_text_operation(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn save(&mut self) -> Result<(), Self::Error> {
        self.events.push("save".to_owned());
        Ok(())
    }
    fn concat_translate(&mut self, anchor: RenderPoint) -> Result<(), Self::Error> {
        if self.refuse_concat {
            return Err(RecordingError::Refused);
        }
        self.events
            .push(format!("translate:{} {}", anchor.x(), anchor.y()));
        Ok(())
    }
    fn restore(&mut self) -> Result<(), Self::Error> {
        self.events.push("restore".to_owned());
        Ok(())
    }
    fn fill_rect(
        &mut self,
        _: DrawRectV1,
        _: &Paint,
        metadata: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        self.events.push(format!("rect:{metadata:?}"));
        Ok(())
    }
    fn draw_path(
        &mut self,
        path: &DrawPathV1,
        style: DrawStyleV1<'_>,
        metadata: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        if self.refuse_paths {
            return Err(RecordingError::Refused);
        }
        self.empty_path_published |= path.commands.is_empty();
        let cubic = path
            .commands
            .iter()
            .any(|command| matches!(command, DrawPathCommandV1::CubicTo { .. }));
        self.events.push(format!(
            "path:{metadata:?}:cubic={cubic}:stroke={}:fill_rule={:?}",
            style.stroke.is_some(),
            style.fill_rule
        ));
        match metadata {
            DrawMetadataV1::DirectGlycosidicOrdinary => self.direct_events.push(
                DirectDrawEvent::Ordinary(style.stroke.expect("ordinary stroke").line_cap),
            ),
            DrawMetadataV1::DirectGlycosidicQ1 => self.direct_events.push(DirectDrawEvent::Q1(
                style.stroke.expect("q1 stroke").line_cap,
            )),
            DrawMetadataV1::DirectGlycosidicW1 => self.direct_events.push(DirectDrawEvent::W1 {
                fill: style.fill.is_some(),
                stroke: style.stroke.is_some(),
            }),
            _ => {}
        }
        Ok(())
    }
    fn draw_ellipse(
        &mut self,
        ellipse: DrawEllipseV1,
        _: DrawStyleV1<'_>,
        metadata: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        if self.refuse_ellipses {
            return Err(RecordingError::Refused);
        }
        self.events.push(format!(
            "ellipse:{metadata:?}:rotation={}",
            ellipse.rotation_degrees
        ));
        Ok(())
    }
    fn finish_page(&mut self) -> Result<(), Self::Error> {
        self.events.push("finish".to_owned());
        Ok(())
    }
}

#[test]
fn private_stream_skips_only_verified_outline_less_whitespace() {
    let layout = laid_out_presentation("L L");
    let run = &layout.operation().runs()[0];
    let space = &run.glyphs()[1];
    let trailing_visible = &run.glyphs()[2];
    assert_eq!(space.glyph_index(), 3);
    assert!(trailing_visible.origin().x() > space.origin().x());

    let mut sink = RecordingSink::default();
    lower_document_plan_to_sink_v1(
        &presentation_plan(layout.operation().clone(), layout.bounds()),
        &mut sink,
    )
    .expect("verified whitespace lowers without an outline");
    assert!(
        !sink.empty_path_published,
        "whitespace never publishes a blank path"
    );
}

#[test]
fn private_stream_rejects_a_visible_scalar_forged_to_the_space_glyph() {
    let layout = laid_out_presentation("L");
    let mut wire = serde_json::to_value(layout.operation()).expect("presentation wire");
    wire["runs"][0]["glyphs"][0]["glyph_index"] = serde_json::Value::from(3);
    let forged: PresentationTextOp = serde_json::from_value(wire).expect("render-level plan");

    let result = lower_document_plan_to_sink_v1(
        &presentation_plan(forged, layout.bounds()),
        &mut RecordingSink::default(),
    );
    assert!(matches!(
        result,
        Err(DrawStreamErrorV1::MissingGlyphOutline { glyph_index: 3 })
    ));
}

#[test]
fn private_stream_keeps_direct_haworth_tiers_and_paint_profiles() {
    let plan = lower_direct_glycosidic_haworth_v1(&super::direct_glycosidic_haworth::request())
        .expect("direct profile");
    let mut sink = RecordingSink::default();
    lower_direct_glycosidic_haworth_plan_to_sink_v1(
        &plan,
        RenderViewportV1::new(-20.0, -20.0, 40.0, 40.0).expect("page"),
        &mut sink,
    )
    .expect("direct stream");
    let first_q = sink
        .direct_events
        .iter()
        .position(|event| matches!(event, DirectDrawEvent::Q1(_)))
        .expect("q1 tier");
    let first_w = sink
        .direct_events
        .iter()
        .position(|event| matches!(event, DirectDrawEvent::W1 { .. }))
        .expect("w1 tier");
    assert!(
        sink.direct_events[..first_q]
            .iter()
            .all(|event| matches!(event, DirectDrawEvent::Ordinary(DrawLineCapV1::Butt)))
    );
    assert!(
        sink.direct_events[first_q..first_w]
            .iter()
            .all(|event| matches!(event, DirectDrawEvent::Q1(DrawLineCapV1::Round)))
    );
    assert!(sink.direct_events[first_w..].iter().all(|event| matches!(
        event,
        DirectDrawEvent::W1 {
            fill: true,
            stroke: false
        }
    )));
}

#[test]
fn composite_stream_replaces_only_selected_bond_outcomes_once() {
    let composite = composite_plan();
    let retained_issue = composite
        .established()
        .outcomes()
        .iter()
        .find_map(|outcome| match outcome {
            DocumentRenderOutcomeV1::Root(root) => match root.content() {
                DocumentRenderContentV1::Molecule(plan) => plan.issues().iter().find(|issue| {
                    issue.target().source_order() == 5
                        && issue.target().record_id().kind() == RecordKind::Bond
                }),
                _ => None,
            },
            DocumentRenderOutcomeV1::Exclusion(_) => None,
        })
        .expect("nonselected issue remains in the established plan");
    assert_eq!(retained_issue.target().source_order(), 5);
    let mut sink = RecordingSink::default();
    lower_document_render_composite_to_sink_v1(&composite, &mut sink).expect("composite stream");

    assert_eq!(sink.events.first(), Some(&"page:20x10".to_owned()));
    assert!(sink.events.iter().any(|event| event == "root:1"));
    assert!(!sink.events.iter().any(|event| event == "root:2"));
    assert!(
        sink.events
            .iter()
            .any(|event| event.starts_with("batch:1:"))
    );
    assert!(
        sink.events
            .iter()
            .any(|event| event.starts_with("batch:3:"))
    );
    assert!(
        !sink
            .events
            .iter()
            .any(|event| event.starts_with("batch:2:"))
    );
    assert!(
        !sink
            .events
            .iter()
            .any(|event| event.starts_with("batch:6:"))
    );

    let direct_start = sink
        .events
        .iter()
        .position(|event| event.contains("DirectGlycosidicOrdinary"))
        .expect("direct ordinary event");
    let retained_bond = sink
        .events
        .iter()
        .position(|event| event.starts_with("batch:3:"))
        .expect("retained bond batch");
    assert!(direct_start < retained_bond);
    assert_eq!(
        sink.direct_events,
        vec![
            DirectDrawEvent::Ordinary(DrawLineCapV1::Butt),
            DirectDrawEvent::Q1(DrawLineCapV1::Round),
            DirectDrawEvent::W1 {
                fill: true,
                stroke: false
            },
        ]
    );
    assert!(
        sink.events
            .iter()
            .any(|event| event.contains("MoleculeMask"))
    );
    assert!(
        sink.events
            .iter()
            .any(|event| event.contains("MoleculeText"))
    );
    assert!(
        sink.events
            .iter()
            .any(|event| event.contains("MoleculeLine"))
    );
    assert_eq!(sink.events.last(), Some(&"finish".to_owned()));
}

#[test]
fn composite_stream_propagates_sink_refusal_without_finishing() {
    let mut sink = RecordingSink {
        refuse_paths: true,
        ..RecordingSink::default()
    };
    let result = lower_document_render_composite_to_sink_v1(&composite_plan(), &mut sink);
    assert!(matches!(
        result,
        Err(DrawStreamErrorV1::Sink(RecordingError::Refused))
    ));
    assert!(!sink.events.iter().any(|event| event == "finish"));
}

#[test]
fn private_stream_preserves_page_order_scopes_profiles_and_exclusions() {
    let mut sink = RecordingSink::default();
    lower_document_plan_to_sink_v1(&mixed_plan(), &mut sink).expect("stream lowering");
    assert_eq!(sink.events[0], "page:100x80");
    assert!(sink.events.iter().any(|event| event == "root:1"));
    assert!(sink.events.iter().any(|event| event == "root:3"));
    assert!(sink.events.iter().any(|event| event == "root:4"));
    assert!(!sink.events.iter().any(|event| event == "root:2"));
    let anchor = sink
        .events
        .iter()
        .position(|event| event == "translate:3 4")
        .expect("atom-local anchor");
    let mask = sink
        .events
        .iter()
        .position(|event| event.contains("MoleculeMask"))
        .expect("opaque molecule mask");
    assert!(anchor < mask);
    let text = sink
        .events
        .iter()
        .position(|event| event == "text")
        .expect("document text");
    let background = sink
        .events
        .iter()
        .position(|event| event.contains("DocumentTextBackground"))
        .expect("document text background");
    let glyph = sink
        .events
        .iter()
        .position(|event| event.contains("MoleculeText"))
        .expect("Telex outline");
    assert!(text < background && background < glyph);
    assert!(
        sink.events
            .iter()
            .any(|event| event.contains("DocumentVectorPath")
                && event.contains("cubic=true")
                && event.contains("stroke=true")
                && event.contains("EvenOdd"))
    );
    assert_eq!(sink.events.last(), Some(&"finish".to_owned()));
}

#[test]
fn private_stream_restores_after_a_body_refusal_without_successful_finish() {
    let mut sink = RecordingSink {
        events: Vec::new(),
        refuse_paths: false,
        refuse_concat: false,
        refuse_ellipses: true,
        ..RecordingSink::default()
    };
    let result = lower_document_plan_to_sink_v1(&mixed_plan(), &mut sink);
    assert!(matches!(
        result,
        Err(DrawStreamErrorV1::Sink(RecordingError::Refused))
    ));
    assert!(sink.events.iter().any(|event| event == "restore"));
    assert!(!sink.events.iter().any(|event| event == "finish"));
}

#[test]
fn private_stream_restores_after_a_translation_refusal_without_successful_finish() {
    let mut sink = RecordingSink {
        events: Vec::new(),
        refuse_paths: false,
        refuse_concat: true,
        refuse_ellipses: false,
        ..RecordingSink::default()
    };
    let result = lower_document_plan_to_sink_v1(&mixed_plan(), &mut sink);
    assert!(matches!(
        result,
        Err(DrawStreamErrorV1::Sink(RecordingError::Refused))
    ));
    let save = sink
        .events
        .iter()
        .position(|event| event == "save")
        .expect("scope save");
    let restore = sink
        .events
        .iter()
        .position(|event| event == "restore")
        .expect("scope restore");
    assert!(save < restore);
    assert!(!sink.events.iter().any(|event| event == "finish"));
}
