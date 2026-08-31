//! Shared semantic atom-label/bond alignment corpus at the document/render boundary.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

use ferrum_document::DocumentSession;
use ferrum_render::glyph_bond_raster::{
    GlyphBondRasterBondIdentity, GlyphBondRasterFixtureIdentity, GlyphBondRasterRelation,
    GlyphBondRasterSourceMapping, rasterize_glyph_bond_layers,
};
use ferrum_render::{
    AtomRenderBatchV1, BondRenderOpV1, RenderBatchContentV4, RenderDisplayLayerV1, RenderIssueKind,
    RenderOp, RenderPoint, TextScript,
};
use serde::Deserialize;

const CORPUS: &str = include_str!("fixtures/atom_label_bond_alignment_cases_v1.json");
const SCHEMA: &str = "atom_label_bond_alignment_cases_v1";
const THIRD_LABEL_REFUSAL: &str = "bond final ink intersects a non-endpoint atom label";
const V2_FIXTURE_CATALOG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../measure_stack/fixtures/v2/fixtures.json"
));

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AlignmentCorpus {
    schema: String,
    cases: Vec<AlignmentCase>,
}

#[derive(Debug, Deserialize)]
struct V2FixtureCatalog {
    fixtures: Vec<V2Fixture>,
}

#[derive(Debug, Deserialize)]
struct V2Fixture {
    fixture_id: String,
    fixture_cdml: String,
    graph: V2FixtureGraph,
    expected_relations: Vec<V2FixtureRelation>,
    negative_cases: Vec<V2FixtureRelation>,
}

#[derive(Debug, Deserialize)]
struct V2FixtureGraph {
    atoms: Vec<V2FixtureAtom>,
    bonds: Vec<V2FixtureBond>,
}

#[derive(Debug, Deserialize)]
struct V2FixtureAtom {
    atom_id: String,
}

#[derive(Debug, Deserialize)]
struct V2FixtureBond {
    bond_id: String,
    style: String,
}

#[derive(Debug, Deserialize)]
struct V2FixtureRelation {
    relation: String,
    subject_id: String,
    object_id: String,
    expectation: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AlignmentCase {
    name: String,
    cdml: String,
    expected_outcome: ExpectedOutcome,
    atoms: Vec<ExpectedAtom>,
    bonds: Vec<ExpectedBond>,
    #[serde(default)]
    offending_bond: Option<String>,
    checks: SemanticChecks,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedAtom {
    source_id: String,
    core_run: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedBond {
    source_id: String,
    style: String,
    display_layer: ExpectedDisplayLayer,
    operation_shape: ExpectedBondOperationShape,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ExpectedDisplayLayer {
    Ordinary,
    HaworthFrontStroke,
    HaworthFrontWedge,
}

impl ExpectedDisplayLayer {
    const fn render_layer(self) -> RenderDisplayLayerV1 {
        match self {
            Self::Ordinary => RenderDisplayLayerV1::Ordinary,
            Self::HaworthFrontStroke => RenderDisplayLayerV1::HaworthFrontStroke,
            Self::HaworthFrontWedge => RenderDisplayLayerV1::HaworthFrontWedge,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ExpectedBondOperationShape {
    SingleLine,
    ParallelDoubleLines,
    ParallelTripleLines,
    SolidWedgePath,
    HashedWedgeLines,
    DashedLines,
    WavyPath,
    HaworthFrontPath,
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
    assert!(
        !corpus.cases.is_empty(),
        "alignment corpus has semantic cases"
    );
    let case_names = corpus
        .cases
        .iter()
        .map(|case| case.name.as_str())
        .collect::<HashSet<_>>();
    assert_eq!(
        case_names.len(),
        corpus.cases.len(),
        "each semantic alignment case has one unique name"
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

fn bond_operations_match_shape(
    operations: &[BondRenderOpV1],
    expected: ExpectedBondOperationShape,
) -> bool {
    match expected {
        ExpectedBondOperationShape::SingleLine => {
            matches!(operations, [BondRenderOpV1::Line(_)])
        }
        ExpectedBondOperationShape::ParallelDoubleLines => {
            matches!(
                operations,
                [BondRenderOpV1::Line(_), BondRenderOpV1::Line(_)]
            )
        }
        ExpectedBondOperationShape::ParallelTripleLines => {
            matches!(
                operations,
                [
                    BondRenderOpV1::Line(_),
                    BondRenderOpV1::Line(_),
                    BondRenderOpV1::Line(_)
                ]
            )
        }
        ExpectedBondOperationShape::SolidWedgePath
        | ExpectedBondOperationShape::WavyPath
        | ExpectedBondOperationShape::HaworthFrontPath => {
            matches!(operations, [BondRenderOpV1::Path(_)])
        }
        ExpectedBondOperationShape::HashedWedgeLines | ExpectedBondOperationShape::DashedLines => {
            !operations.is_empty()
                && operations
                    .iter()
                    .all(|operation| matches!(operation, BondRenderOpV1::Line(_)))
        }
    }
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

fn glyph_bond_raster_output_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .expect("document crate has repository-root ancestor")
        .join("output_glyph_alignment")
}

fn glyph_bond_raster_mapping(
    observation: &ferrum_document::DocumentRenderObservationV2,
) -> GlyphBondRasterSourceMapping {
    let molecule = &observation.document().projection().molecules()[0];
    let atoms = molecule
        .atoms()
        .iter()
        .map(|atom| {
            (
                atom.document_object_id().as_str().to_owned(),
                atom.source_id()
                    .expect("fixture atom has source ID")
                    .to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let elements = molecule
        .atoms()
        .iter()
        .map(|atom| {
            (
                atom.source_id()
                    .expect("fixture atom has source ID")
                    .to_owned(),
                atom.element()
                    .expect("fixture atom has an element")
                    .to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let bonds = molecule
        .bonds()
        .iter()
        .map(|bond| {
            (
                bond.document_object_id().as_str().to_owned(),
                bond.source_id()
                    .expect("fixture bond has source ID")
                    .to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    GlyphBondRasterSourceMapping::with_atom_elements(atoms, elements, bonds)
}

fn glyph_bond_raster_bonds(
    observation: &ferrum_document::DocumentRenderObservationV2,
    case: &AlignmentCase,
) -> Vec<GlyphBondRasterBondIdentity> {
    observation.document().projection().molecules()[0]
        .bonds()
        .iter()
        .map(|bond| {
            let source_id = bond.source_id().expect("fixture bond has source ID");
            let expected = case
                .bonds
                .iter()
                .find(|expected| expected.source_id == source_id)
                .expect("fixture bond has a measurement style declaration");
            GlyphBondRasterBondIdentity::new(
                source_id,
                bond.start()
                    .source_id()
                    .expect("fixture atom has source ID"),
                bond.end().source_id().expect("fixture atom has source ID"),
                measurement_style(expected),
            )
        })
        .collect()
}

fn measurement_style(expected: &ExpectedBond) -> &'static str {
    match expected.operation_shape {
        ExpectedBondOperationShape::SingleLine => match expected.style.as_str() {
            "b1" => "bold",
            _ => "normal",
        },
        ExpectedBondOperationShape::ParallelDoubleLines => "double",
        ExpectedBondOperationShape::ParallelTripleLines => "triple",
        ExpectedBondOperationShape::SolidWedgePath => "solid-wedge",
        ExpectedBondOperationShape::HashedWedgeLines => "hashed-wedge",
        ExpectedBondOperationShape::DashedLines => "dashed",
        ExpectedBondOperationShape::WavyPath => "wavy",
        ExpectedBondOperationShape::HaworthFrontPath => match expected.display_layer {
            ExpectedDisplayLayer::HaworthFrontStroke => "haworth-front-stroke",
            ExpectedDisplayLayer::HaworthFrontWedge => "haworth-front-wedge",
            ExpectedDisplayLayer::Ordinary => "haworth-front",
        },
    }
}

fn v2_fixture_identity(case: &AlignmentCase) -> GlyphBondRasterFixtureIdentity {
    let catalog: V2FixtureCatalog =
        serde_json::from_str(V2_FIXTURE_CATALOG).expect("V2 fixture catalog has valid JSON");
    let fixture = catalog
        .fixtures
        .iter()
        .find(|fixture| fixture.fixture_id == case.name)
        .unwrap_or_else(|| panic!("{} has a V2 fixture catalog row", case.name));
    assert_eq!(
        fixture.fixture_cdml, case.cdml,
        "{} V2 fixture CDML is the authoritative corpus CDML",
        case.name
    );
    let fixture_atoms = fixture
        .graph
        .atoms
        .iter()
        .map(|atom| atom.atom_id.as_str())
        .collect::<HashSet<_>>();
    let case_atoms = case
        .atoms
        .iter()
        .map(|atom| atom.source_id.as_str())
        .collect::<HashSet<_>>();
    assert_eq!(
        fixture_atoms, case_atoms,
        "{} V2 graph names every source atom",
        case.name
    );
    let fixture_bonds = fixture
        .graph
        .bonds
        .iter()
        .map(|bond| (bond.bond_id.as_str(), bond.style.as_str()))
        .collect::<HashSet<_>>();
    let case_bonds = case
        .bonds
        .iter()
        .map(|bond| (bond.source_id.as_str(), measurement_style(bond)))
        .collect::<HashSet<_>>();
    assert_eq!(
        fixture_bonds, case_bonds,
        "{} V2 graph names every source bond/style",
        case.name
    );
    let relation = |row: &V2FixtureRelation| {
        GlyphBondRasterRelation::new(
            &row.relation,
            &row.subject_id,
            &row.object_id,
            &row.expectation,
        )
    };
    GlyphBondRasterFixtureIdentity::from_cdml(
        &fixture.fixture_id,
        &fixture.fixture_cdml,
        fixture.expected_relations.iter().map(relation).collect(),
        fixture.negative_cases.iter().map(relation).collect(),
    )
}

#[test]
#[ignore = "developer raster handoff; run through the glyph-bond measurement gate"]
fn glyph_bond_raster_handoff_emits_every_renderable_alignment_case() {
    let output_root = glyph_bond_raster_output_root().join("v2");
    std::fs::create_dir_all(&output_root).expect("ignored developer output root is creatable");
    let requested_cases = std::env::var("FERRUM_GLYPH_BOND_RASTER_CASE")
        .ok()
        .map(|case_name| {
            case_name
                .split(',')
                .map(str::to_owned)
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    let mut emitted_cases = 0;
    for case in corpus().cases {
        if case.expected_outcome != ExpectedOutcome::Render {
            continue;
        }
        if !requested_cases.is_empty() && !requested_cases.contains(&case.name) {
            continue;
        }
        let session = DocumentSession::load(&case.cdml)
            .unwrap_or_else(|error| panic!("{} CDML must load: {error}", case.name));
        let observation = session
            .observe_render_v2(0)
            .unwrap_or_else(|error| panic!("{} must resolve V4/V2: {error}", case.name));
        let plan = observation.resolved().molecule_plans()[0].plan();
        let viewport = ferrum_render::RenderViewportV1::new(-200.0, -200.0, 400.0, 400.0)
            .expect("developer diagnostic viewport is finite");
        let mapping = glyph_bond_raster_mapping(&observation);
        let layers = rasterize_glyph_bond_layers(plan, viewport, &mapping)
            .unwrap_or_else(|error| panic!("{} rasterizes: {error}", case.name));
        let case_directory = output_root.join(&case.name);
        let manifest = layers
            .write_measurement_manifest_v2(
                &case_directory,
                &v2_fixture_identity(&case),
                &glyph_bond_raster_bonds(&observation, &case),
            )
            .unwrap_or_else(|error| panic!("{} emits handoff: {error}", case.name));
        assert!(manifest.is_file(), "{} emits raster manifest", case.name);
        assert!(
            layers.normal_composite().nontransparent_pixels() > 0,
            "{} emits composite ink",
            case.name
        );
        emitted_cases += 1;
    }
    assert!(
        emitted_cases > 0,
        "developer raster selection emits a renderable case"
    );
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
        let source_molecule = &observation.document().projection().molecules()[0];

        assert_eq!(
            source_molecule.atoms().len(),
            case.atoms.len(),
            "{} declares every source atom's core run",
            case.name
        );

        let atom_ids = case
            .atoms
            .iter()
            .map(|atom| atom.source_id.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(
            atom_ids.len(),
            case.atoms.len(),
            "{} names each expected atom once",
            case.name
        );
        for expected in &case.atoms {
            let source_atom = source_molecule
                .atoms()
                .iter()
                .find(|atom| atom.source_id() == Some(expected.source_id.as_str()))
                .unwrap_or_else(|| {
                    panic!("{} source atom {} projects", case.name, expected.source_id)
                });
            let batch = plan
                .batches()
                .iter()
                .find(|batch| {
                    batch.target().document_object_id() == source_atom.document_object_id()
                })
                .unwrap_or_else(|| {
                    panic!(
                        "{} source atom {} has a target batch",
                        case.name, expected.source_id
                    )
                });
            let RenderBatchContentV4::Atom(atom) = batch.content() else {
                panic!(
                    "{} source atom {} has atom content",
                    case.name, expected.source_id
                );
            };
            let core_index = usize::try_from(atom.label().core_element_run_index())
                .expect("core index fits usize");
            assert_eq!(
                atom.label().text().runs()[core_index].text(),
                expected.core_run,
                "{} source atom {} retains its expected core run",
                case.name,
                expected.source_id
            );
        }

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
            let expected_core = case
                .checks
                .core_run
                .as_ref()
                .expect("isotope checks name their core element")
                .text
                .as_str();
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
            assert_eq!(label.text().runs()[core].text(), expected_core);
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
        let bond_ids = case
            .bonds
            .iter()
            .map(|bond| bond.source_id.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(
            bond_ids.len(),
            case.bonds.len(),
            "{} names each expected bond once",
            case.name
        );
        assert_eq!(
            bond_batches.len(),
            case.bonds.len(),
            "{} emits exactly its declared target-specific bond batches",
            case.name
        );
        for expected in &case.bonds {
            let source_bond = source_molecule
                .bonds()
                .iter()
                .find(|bond| bond.source_id() == Some(expected.source_id.as_str()))
                .unwrap_or_else(|| {
                    panic!("{} source bond {} projects", case.name, expected.source_id)
                });
            assert_eq!(
                source_bond.source_type(),
                Some(expected.style.as_str()),
                "{} source bond {} retains its expected style token",
                case.name,
                expected.source_id
            );
            let batch = plan
                .batches()
                .iter()
                .find(|batch| {
                    batch.target().document_object_id() == source_bond.document_object_id()
                })
                .unwrap_or_else(|| {
                    panic!(
                        "{} source bond {} has a target batch",
                        case.name, expected.source_id
                    )
                });
            assert_eq!(
                batch.display_layer(),
                expected.display_layer.render_layer(),
                "{} source bond {} has its declared display layer",
                case.name,
                expected.source_id
            );
            let RenderBatchContentV4::Bond(bond) = batch.content() else {
                panic!(
                    "{} source bond {} has bond content",
                    case.name, expected.source_id
                );
            };
            assert!(
                bond_operations_match_shape(bond.operations(), expected.operation_shape),
                "{} source bond {} has its declared operation shape",
                case.name,
                expected.source_id
            );
        }
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
