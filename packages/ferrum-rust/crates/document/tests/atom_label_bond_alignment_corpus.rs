//! Shared semantic atom-label/bond alignment corpus at the document/render boundary.

use std::collections::HashMap;

use ferrum_document::DocumentSession;
use ferrum_render::{
    AtomRenderBatchV1, BondRenderOpV1, RenderBatchContentV4, RenderIssueKind, RenderOp,
    RenderPoint, TextScript,
};
use serde::Deserialize;

const CORPUS: &str = include_str!("fixtures/atom_label_bond_alignment_cases_v1.json");
const SCHEMA: &str = "atom_label_bond_alignment_cases_v1";
const THIRD_LABEL_REFUSAL: &str = "bond final ink intersects a non-endpoint atom label";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AlignmentCorpus {
    schema: String,
    cases: Vec<AlignmentCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AlignmentCase {
    name: String,
    cdml: String,
    expected_outcome: ExpectedOutcome,
    #[serde(default)]
    offending_bond: Option<String>,
    checks: SemanticChecks,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ExpectedOutcome {
    Render,
    UnrenderableTarget,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SemanticChecks {
    finite_geometry: bool,
    ordered_operations: bool,
    positive_bond_content: Option<bool>,
    full_ink_clearance: bool,
    require_mask: Option<bool>,
    core_run: Option<CoreRunCheck>,
    leading_superscript: Option<String>,
    runs: Option<Vec<LabelRunCheck>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoreRunCheck {
    text: String,
    index: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LabelRunCheck {
    text: String,
    script: TextScript,
}

fn corpus() -> AlignmentCorpus {
    let corpus: AlignmentCorpus = serde_json::from_str(CORPUS).expect("corpus has closed schema");
    assert_eq!(corpus.schema, SCHEMA, "corpus schema tag is exact");
    assert_eq!(
        corpus.cases.len(),
        12,
        "approved corpus has no silent row loss"
    );
    corpus
}

fn operation_z(operation: &RenderOp) -> i32 {
    match operation {
        RenderOp::Text(value) => value.z(),
        RenderOp::Line(value) => value.z(),
        RenderOp::Mask(value) => value.z(),
        RenderOp::Ellipse(value) => value.z(),
        RenderOp::Path(value) => value.z(),
        RenderOp::DoubleBondCarrierMark(value) => value.z(),
    }
}

fn all_label_batches<'a>(
    batches: impl Iterator<Item = &'a ferrum_render::RenderBatchV4>,
) -> impl Iterator<Item = &'a AtomRenderBatchV1> {
    batches.filter_map(|batch| match batch.content() {
        RenderBatchContentV4::Atom(atom) => Some(atom.as_ref()),
        RenderBatchContentV4::CompactGroup(_) | RenderBatchContentV4::Bond(_) => None,
    })
}

fn atom_anchors_by_document_object_id(
    observation: &ferrum_document::DocumentRenderObservationV2,
    batches: &[ferrum_render::RenderBatchV4],
    case_name: &str,
) -> HashMap<ferrum_document::DocumentObjectIdV1, RenderPoint> {
    let mut anchors = HashMap::new();
    for source_atom in observation.document().projection().molecules()[0].atoms() {
        let batch = batches
            .iter()
            .find(|batch| batch.target().document_object_id() == source_atom.document_object_id())
            .unwrap_or_else(|| panic!("{case_name} source atom has a render batch"));
        let RenderBatchContentV4::Atom(atom) = batch.content() else {
            panic!("{case_name} source atom target has atom content");
        };
        let anchor = atom.atom_local_anchor();
        assert_eq!(
            anchor.x(),
            source_atom.position().x(),
            "{case_name} source x reaches anchor"
        );
        assert_eq!(
            anchor.y(),
            source_atom.position().y(),
            "{case_name} source y reaches anchor"
        );
        let core_center = atom
            .label()
            .core_element_ink_bounds()
            .center()
            .expect("validated core bounds have a center");
        assert_eq!(core_center.x(), 0.0, "{case_name} core local x is centered");
        assert_eq!(core_center.y(), 0.0, "{case_name} core local y is centered");
        anchors.insert(source_atom.document_object_id().clone(), anchor);
    }
    anchors
}

#[test]
fn atom_label_bond_alignment_cases_are_closed_and_schema_tagged() {
    let parsed = corpus();
    assert!(parsed.cases.iter().all(|case| !case.name.trim().is_empty()));

    let unknown_field = CORPUS.replacen(
        "\"schema\": \"atom_label_bond_alignment_cases_v1\",",
        "\"schema\": \"atom_label_bond_alignment_cases_v1\", \"unexpected\": true,",
        1,
    );
    assert!(serde_json::from_str::<AlignmentCorpus>(&unknown_field).is_err());
}

#[test]
fn authoritative_v4_observation_consumes_every_alignment_case() {
    for case in corpus().cases {
        let session = DocumentSession::load(&case.cdml)
            .unwrap_or_else(|error| panic!("{} CDML must load: {error}", case.name));
        let observation = session
            .observe_render_v2(0)
            .unwrap_or_else(|error| panic!("{} must resolve V4/V2: {error}", case.name));
        let resolved = observation.resolved();
        assert_eq!(
            resolved.molecule_plans().len(),
            1,
            "{} has one molecule",
            case.name
        );
        let plan = resolved.molecule_plans()[0].plan();
        let anchors = atom_anchors_by_document_object_id(&observation, plan.batches(), &case.name);

        if case.checks.finite_geometry {
            for label in all_label_batches(plan.batches().iter()).map(|atom| atom.label()) {
                let full = label.full_ink_bounds();
                let core = label.core_element_ink_bounds();
                for value in [
                    full.min_x(),
                    full.min_y(),
                    full.max_x(),
                    full.max_y(),
                    core.min_x(),
                    core.min_y(),
                    core.max_x(),
                    core.max_y(),
                ] {
                    assert!(value.is_finite(), "{} label geometry is finite", case.name);
                }
                assert!(full.contains(core), "{} full ink owns core ink", case.name);
            }
        }

        if case.checks.ordered_operations {
            assert!(
                plan.batches()
                    .windows(2)
                    .all(|pair| pair[0].paint_order() < pair[1].paint_order()),
                "{} batches retain source paint order",
                case.name
            );
            for batch in plan.batches() {
                let operations = batch.operations();
                assert!(
                    operations
                        .windows(2)
                        .all(|pair| operation_z(&pair[0]) < operation_z(&pair[1])),
                    "{} batch operations retain strict paint order",
                    case.name
                );
            }
        }

        if case.checks.full_ink_clearance {
            assert!(
                all_label_batches(plan.batches().iter()).all(|atom| {
                    let label = atom.label();
                    label.bond_ink_clearance().get() > 0.0
                        && label
                            .full_ink_bounds()
                            .contains(label.core_element_ink_bounds())
                }),
                "{} publishes positive clearance and complete label ink",
                case.name
            );
        }

        if case.checks.require_mask == Some(true) {
            assert!(
                all_label_batches(plan.batches().iter()).any(|atom| atom.label().mask().is_some()),
                "{} publishes a Rust-owned atom label mask",
                case.name
            );
        }

        if let Some(expected) = case.checks.core_run.as_ref() {
            let matching = all_label_batches(plan.batches().iter()).find(|atom| {
                let label = atom.label();
                usize::try_from(label.core_element_run_index())
                    .ok()
                    .and_then(|index| label.text().runs().get(index))
                    .is_some_and(|run| run.text() == expected.text)
            });
            let atom = matching.unwrap_or_else(|| panic!("{} has expected core run", case.name));
            assert_eq!(
                atom.label().core_element_run_index(),
                expected.index,
                "{} core index",
                case.name
            );
        }

        if let Some(expected) = case.checks.leading_superscript.as_deref() {
            let atom = all_label_batches(plan.batches().iter())
                .find(|atom| {
                    atom.label()
                        .text()
                        .runs()
                        .first()
                        .is_some_and(|run| run.text() == expected)
                })
                .unwrap_or_else(|| panic!("{} has isotope run", case.name));
            let label = atom.label();
            assert_eq!(label.text().runs()[0].script(), TextScript::Superscript);
            let core =
                usize::try_from(label.core_element_run_index()).expect("core index fits usize");
            assert_eq!(label.text().runs()[core].text(), "C");
            assert_eq!(label.text().runs()[core].script(), TextScript::Baseline);
        }

        if let Some(expected_runs) = case.checks.runs.as_ref() {
            let atom = all_label_batches(plan.batches().iter())
                .find(|atom| {
                    atom.label()
                        .text()
                        .runs()
                        .iter()
                        .map(|run| run.text())
                        .eq(expected_runs.iter().map(|run| run.text.as_str()))
                })
                .unwrap_or_else(|| panic!("{} has the expected ordered label runs", case.name));
            assert!(
                atom.label()
                    .text()
                    .runs()
                    .iter()
                    .map(|run| run.script())
                    .eq(expected_runs.iter().map(|run| run.script)),
                "{} preserves semantic script order",
                case.name
            );
        }

        let bond_batches = plan
            .batches()
            .iter()
            .filter_map(|batch| match batch.content() {
                RenderBatchContentV4::Bond(bond) => Some(bond),
                RenderBatchContentV4::Atom(_) | RenderBatchContentV4::CompactGroup(_) => None,
            })
            .collect::<Vec<_>>();
        if case.checks.positive_bond_content == Some(true) {
            assert!(
                bond_batches
                    .iter()
                    .all(|bond| !bond.operations().is_empty()),
                "{} successful bonds have emitted final content",
                case.name
            );
            assert!(
                bond_batches
                    .iter()
                    .flat_map(|bond| bond.operations())
                    .all(|operation| {
                        matches!(
                            operation,
                            BondRenderOpV1::Line(_)
                                | BondRenderOpV1::Path(_)
                                | BondRenderOpV1::DoubleBondCarrierMark(_)
                        )
                    }),
                "{} bond content remains closed scene-space geometry",
                case.name
            );
            assert!(
                bond_batches.iter().all(|bond| {
                    let axis = bond.attachment_axis();
                    anchors.values().any(|anchor| *anchor == axis.start())
                        && anchors.values().any(|anchor| *anchor == axis.end())
                }),
                "{} joins source position -> atom anchor -> core center -> bond axis",
                case.name
            );
        }

        match case.expected_outcome {
            ExpectedOutcome::Render => {
                assert!(
                    plan.issues().is_empty(),
                    "{} has no excluded target",
                    case.name
                );
                assert!(
                    !bond_batches.is_empty(),
                    "{} has visible bond content",
                    case.name
                );
            }
            ExpectedOutcome::UnrenderableTarget => {
                let source_id = case.offending_bond.as_deref().expect("refusal names bond");
                let projected_bond = observation.document().projection().molecules()[0]
                    .bonds()
                    .iter()
                    .find(|bond| bond.source_id() == Some(source_id))
                    .unwrap_or_else(|| panic!("{} source bond projects", case.name));
                assert!(
                    !plan.batches().iter().any(|batch| {
                        matches!(batch.content(), RenderBatchContentV4::Bond(_))
                            && batch.target().document_object_id()
                                == projected_bond.document_object_id()
                    }),
                    "{} omits the refused bond batch",
                    case.name
                );
                let issue = plan
                    .issues()
                    .iter()
                    .find(|issue| {
                        issue.target().document_object_id() == projected_bond.document_object_id()
                    })
                    .unwrap_or_else(|| panic!("{} emits target-specific refusal", case.name));
                assert!(
                    matches!(issue.kind(), RenderIssueKind::UnrenderableTarget { reason }
                    if reason == THIRD_LABEL_REFUSAL || source_id == "coincident")
                );
                if source_id == "crossing" {
                    assert!(
                        matches!(issue.kind(), RenderIssueKind::UnrenderableTarget { reason }
                        if reason == THIRD_LABEL_REFUSAL)
                    );
                }
            }
        }
    }
}
