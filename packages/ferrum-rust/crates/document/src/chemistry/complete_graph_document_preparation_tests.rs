use crate::{
    DocumentBondOrderV1, DocumentBondPresentationV1, DocumentDirectedBondDepictionV1,
    DocumentDoubleBondCarrierMarkDepictionV1, DocumentDoubleBondCarrierMarkV1,
    DocumentDoubleBondConfigurationV1, DocumentDoubleBondStereoV1,
    DocumentMoleculePreparationErrorV2, DocumentStereoLigandV1, DocumentStereoSemanticReportV1,
    DocumentTetrahedralParityV1, DocumentTetrahedralStereoV1, MoleculeInsertionAtomV1,
    MoleculeInsertionV1, Point3V1, PreparedDocumentMoleculeV2,
    prepare_complete_graph_for_document_v2, prepare_inchi_molecule_for_document_v2,
};
use ferrum_chemistry::{
    AtomChirality, AtomicNumber, BondDirection, BondOrder, BondStereo, ChemEngine, ChemistryError,
    Coordinates, KekulizeOptions, MolAtom, MolBond, MolGraph, Point2 as ChemistryPoint2,
    SmilesMolecule,
};
use ferrum_geometry::{MoleculePlacementV1, Point2};

use super::{
    admit_double_bond_carrier_marks_v2, document_double_bond_carrier_mark_v2,
    document_double_bond_configuration_v2, native_ez_direction_is_carrier_v2,
    require_double_bond_carrier_marks_v2, unsupported_document_atom_fact_v2,
};

fn atom(symbol: &str, aromatic: bool) -> MolAtom {
    MolAtom::new(
        AtomicNumber::from_symbol(symbol).expect("test element must be supported"),
        Some(0),
        None,
        Some(0),
        aromatic,
    )
    .expect("test atom facts must be valid")
}

fn coordinates(points: &[(f64, f64)]) -> Coordinates {
    Coordinates::new(
        points
            .iter()
            .map(|(x, y)| ChemistryPoint2::new(*x, *y).expect("test point must be finite"))
            .collect(),
    )
}

fn placement() -> MoleculePlacementV1 {
    MoleculePlacementV1::new(40.0, Point2::new(100.0, 200.0).expect("finite anchor"))
        .expect("positive placement")
}

struct PassThroughKekulizeEngine;

impl ChemEngine for PassThroughKekulizeEngine {
    fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "smiles_to_molecule",
        })
    }

    fn generate_2d_coordinates(&self, _molecule: &MolGraph) -> Result<Coordinates, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "generate_2d_coordinates",
        })
    }

    fn kekulize(
        &self,
        molecule: &MolGraph,
        _options: KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        Ok(molecule.clone())
    }
}

struct InchiHydrogenEngine;

impl ChemEngine for InchiHydrogenEngine {
    fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "smiles_to_molecule",
        })
    }

    fn generate_2d_coordinates(&self, _molecule: &MolGraph) -> Result<Coordinates, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "generate_2d_coordinates",
        })
    }

    fn inchi_to_molecule(&self, inchi: &str) -> Result<SmilesMolecule, ChemistryError> {
        assert_eq!(inchi, "InChI=1S/CH4/h1H4");
        let graph = MolGraph::new(
            vec![
                MolAtom::new(
                    AtomicNumber::from_symbol("C").expect("test element is supported"),
                    Some(0),
                    None,
                    Some(4),
                    false,
                )
                .expect("test atom is valid"),
            ],
            vec![],
            Some(coordinates(&[(0.0, 0.0)])),
        )
        .expect("test graph is valid");
        SmilesMolecule::new("C", graph).map_err(|error| ChemistryError::MalformedNativeResponse {
            reason: error.to_string(),
        })
    }

    fn kekulize(
        &self,
        molecule: &MolGraph,
        _options: KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        Ok(molecule.clone())
    }
}

#[test]
fn prepared_v2_refuses_descriptors_that_do_not_match_its_molecule() {
    let insertion = MoleculeInsertionV1::new(
        vec![
            MoleculeInsertionAtomV1::new(
                "C",
                Point3V1::new(0.0, 0.0, 0.0).expect("finite point"),
                None,
                None,
                None,
            )
            .expect("valid atom"),
            MoleculeInsertionAtomV1::new(
                "Cl",
                Point3V1::new(1.0, 0.0, 0.0).expect("finite point"),
                None,
                None,
                None,
            )
            .expect("valid atom"),
        ],
        vec![crate::MoleculeInsertionBondV1::new(
            0,
            1,
            DocumentBondOrderV1::Single,
        )],
    )
    .expect("valid detached insertion");
    let tetrahedral = DocumentTetrahedralStereoV1::new(
        0,
        [
            DocumentStereoLigandV1::Atom(1),
            DocumentStereoLigandV1::Atom(2),
            DocumentStereoLigandV1::Atom(3),
            DocumentStereoLigandV1::ExplicitHydrogen,
        ],
        DocumentTetrahedralParityV1::Clockwise,
    )
    .expect("one explicit hydrogen sentinel is valid");
    let double_bond =
        DocumentDoubleBondStereoV1::new(4, 5, 6, DocumentDoubleBondConfigurationV1::E)
            .expect("distinct E/Z ligands are valid");
    let result = PreparedDocumentMoleculeV2::with_stereo_semantics(
        insertion,
        DocumentStereoSemanticReportV1::new(vec![tetrahedral], vec![double_bond]),
    );
    assert!(
        result.is_err(),
        "out-of-graph facts must not form a payload"
    );
}

#[test]
fn v2_preparation_refuses_residual_aromaticity_before_constructing_a_payload() {
    let graph = MolGraph::new(
        vec![atom("C", true), atom("C", true)],
        vec![MolBond::new(0, 1, BondOrder::Aromatic, true)],
        Some(coordinates(&[(0.0, 0.0), (1.0, 0.0)])),
    )
    .expect("pre-kekulized graph is structurally valid");

    assert!(matches!(
        prepare_complete_graph_for_document_v2(&PassThroughKekulizeEngine, &graph, placement()),
        Err(DocumentMoleculePreparationErrorV2::AromaticityResolutionFailed)
    ));
}

#[test]
fn v2_admission_accepts_native_no_implicit_explicit_hydrogen_fact() {
    assert_eq!(
        unsupported_document_atom_fact_v2(AtomChirality::TetrahedralCw, 0, true, Some(1), None,),
        None
    );
    assert_eq!(
        unsupported_document_atom_fact_v2(AtomChirality::Unspecified, 0, true, None, None),
        Some("no-implicit-hydrogen policy")
    );
}

#[test]
fn inchi_coordinator_prepares_native_explicit_hydrogen_counts_through_v2() {
    let prepared = prepare_inchi_molecule_for_document_v2(
        &InchiHydrogenEngine,
        "InChI=1S/CH4/h1H4",
        placement(),
    )
    .expect("native InChI explicit hydrogen counts must prepare");

    assert_eq!(
        prepared.molecule_insertion().atoms()[0].explicit_hydrogens(),
        Some(4)
    );
}

#[test]
fn v2_admission_maps_parser_trans_double_bond_to_e_semantics() {
    assert_eq!(
        document_double_bond_configuration_v2(BondStereo::Trans),
        Some(DocumentDoubleBondConfigurationV1::E)
    );
    assert_eq!(
        document_double_bond_configuration_v2(BondStereo::Other),
        None
    );
}

#[test]
fn directed_bond_depiction_retains_authored_endpoint_order_and_presentation() {
    let depiction =
        DocumentDirectedBondDepictionV1::new(7, 2, 0, DocumentBondPresentationV1::HashedWedge);

    assert_eq!(depiction.bond_index(), 7);
    assert_eq!(depiction.endpoints(), (2, 0));
    assert_eq!(
        depiction.presentation(),
        DocumentBondPresentationV1::HashedWedge
    );
}

#[test]
fn public_directed_bonds_admit_ordered_wedge_and_hash_depictions() {
    for (direction, presentation) in [
        (
            BondDirection::BeginWedge,
            DocumentBondPresentationV1::SolidWedge,
        ),
        (
            BondDirection::BeginDash,
            DocumentBondPresentationV1::HashedWedge,
        ),
    ] {
        let graph = MolGraph::new(
            vec![atom("C", false), atom("O", false)],
            vec![
                MolBond::directed(1, 0, BondOrder::Single, false, direction)
                    .expect("public directional bond is valid"),
            ],
            Some(coordinates(&[(0.0, 0.0), (1.0, 0.0)])),
        )
        .expect("valid directional graph");

        let prepared =
            prepare_complete_graph_for_document_v2(&PassThroughKekulizeEngine, &graph, placement())
                .expect("valid directed graph prepares for a document");
        let depiction = prepared
            .stereo_depictions()
            .expect("directed graph retains a depiction")
            .directed_bonds()
            .iter()
            .find(|depiction| depiction.bond_index() == 0)
            .expect("directed source bond has one ordered depiction");
        assert_eq!(depiction.endpoints(), (1, 0));
        assert_eq!(depiction.presentation(), presentation);
    }
}

#[test]
fn v2_admission_accepts_native_ez_carrier_directions_only() {
    let up = MolBond::directed(0, 1, BondOrder::Single, false, BondDirection::EndUpRight)
        .expect("public E/Z up carrier is valid");
    let down = MolBond::directed(1, 2, BondOrder::Single, false, BondDirection::EndDownRight)
        .expect("public E/Z down carrier is valid");
    assert_eq!(up.direction(), BondDirection::EndUpRight);
    assert_eq!(down.direction(), BondDirection::EndDownRight);
    assert!(native_ez_direction_is_carrier_v2(BondDirection::EndUpRight));
    assert!(native_ez_direction_is_carrier_v2(
        BondDirection::EndDownRight
    ));
    assert!(!native_ez_direction_is_carrier_v2(
        BondDirection::BeginWedge
    ));
}

#[test]
fn ez_carrier_marks_preserve_the_f_slash_c_double_c_slash_f_shape() {
    let marks = require_double_bond_carrier_marks_v2(
        1,
        vec![
            DocumentDoubleBondCarrierMarkDepictionV1::new(
                1,
                0,
                document_double_bond_carrier_mark_v2(BondDirection::EndUpRight)
                    .expect("up direction is an E/Z carrier mark"),
            ),
            DocumentDoubleBondCarrierMarkDepictionV1::new(
                1,
                2,
                document_double_bond_carrier_mark_v2(BondDirection::EndDownRight)
                    .expect("down direction is an E/Z carrier mark"),
            ),
        ],
    )
    .expect("F/C=C/F-shaped source facts have native directional carriers");
    assert_eq!(marks[0].mark(), DocumentDoubleBondCarrierMarkV1::Up);
    assert_eq!(marks[1].mark(), DocumentDoubleBondCarrierMarkV1::Down);
}

#[test]
fn ez_carrier_admission_refuses_a_semantic_fact_without_native_direction() {
    let first = MolBond::new(0, 1, BondOrder::Single, false);
    let second = MolBond::new(2, 3, BondOrder::Single, false);
    let marks = admit_double_bond_carrier_marks_v2(1, [(0, &first), (2, &second)])
        .expect("undirected carriers are structurally valid");
    assert!(matches!(
        require_double_bond_carrier_marks_v2(1, marks),
        Err(
            DocumentMoleculePreparationErrorV2::UnrepresentableDoubleBondDepiction {
                bond_index: 1,
            }
        )
    ));
}

#[test]
fn complete_graph_preparation_maps_atom_bond_and_coordinate_order() {
    let graph = MolGraph::new(
        vec![atom("C", false), atom("O", false)],
        vec![MolBond::new(0, 1, BondOrder::Double, false)],
        Some(coordinates(&[(0.0, 0.0), (2.0, 0.0)])),
    )
    .expect("valid V2000-shaped owned graph");

    let insertion =
        prepare_complete_graph_for_document_v2(&PassThroughKekulizeEngine, &graph, placement())
            .expect("complete graph must prepare");
    let insertion = insertion.molecule_insertion();

    assert_eq!(insertion.atoms()[0].element(), "C");
    assert_eq!(insertion.atoms()[1].element(), "O");
    assert_eq!(insertion.atoms()[0].position().x(), 80.0);
    assert_eq!(insertion.atoms()[1].position().x(), 120.0);
    assert_eq!(insertion.bonds()[0].start(), 0);
    assert_eq!(insertion.bonds()[0].end(), 1);
    assert_eq!(insertion.bonds()[0].order(), DocumentBondOrderV1::Double);
}

#[test]
fn complete_graph_preparation_maps_triple_bond_in_source_order() {
    let graph = MolGraph::new(
        vec![atom("N", false), atom("C", false), atom("O", false)],
        vec![
            MolBond::new(1, 2, BondOrder::Double, false),
            MolBond::new(0, 1, BondOrder::Triple, false),
        ],
        Some(coordinates(&[(-1.0, 0.0), (0.0, 0.0), (1.0, 0.0)])),
    )
    .expect("valid V3000-shaped owned graph");

    let insertion =
        prepare_complete_graph_for_document_v2(&PassThroughKekulizeEngine, &graph, placement())
            .expect("complete graph must prepare");
    let insertion = insertion.molecule_insertion();

    assert_eq!(insertion.bonds()[0].start(), 1);
    assert_eq!(insertion.bonds()[0].end(), 2);
    assert_eq!(insertion.bonds()[0].order(), DocumentBondOrderV1::Double);
    assert_eq!(insertion.bonds()[1].start(), 0);
    assert_eq!(insertion.bonds()[1].end(), 1);
    assert_eq!(insertion.bonds()[1].order(), DocumentBondOrderV1::Triple);
}

#[test]
fn missing_complete_coordinates_are_rejected_before_candidate_construction() {
    let graph = MolGraph::new(vec![atom("C", false)], vec![], None).expect("valid graph");

    assert!(matches!(
        prepare_complete_graph_for_document_v2(&PassThroughKekulizeEngine, &graph, placement()),
        Err(DocumentMoleculePreparationErrorV2::MissingCoordinates)
    ));
}

#[test]
fn unsupported_quadruple_bond_is_rejected_before_document_conversion() {
    let graph = MolGraph::new(
        vec![atom("C", false), atom("C", false)],
        vec![MolBond::new(0, 1, BondOrder::Quadruple, false)],
        Some(coordinates(&[(0.0, 0.0), (1.0, 0.0)])),
    )
    .expect("structurally valid graph");

    assert!(matches!(
        prepare_complete_graph_for_document_v2(&PassThroughKekulizeEngine, &graph, placement()),
        Err(DocumentMoleculePreparationErrorV2::UnsupportedBondOrder {
            order: BondOrder::Quadruple,
            ..
        })
    ));
}

#[test]
fn unresolved_aromatic_facts_are_rejected_when_native_resolution_fails() {
    let graph = MolGraph::new(
        vec![atom("C", true), atom("C", true)],
        vec![MolBond::new(0, 1, BondOrder::Aromatic, true)],
        Some(coordinates(&[(0.0, 0.0), (1.0, 0.0)])),
    )
    .expect("pre-kekulized graph is structurally valid");

    assert!(matches!(
        prepare_complete_graph_for_document_v2(&PassThroughKekulizeEngine, &graph, placement()),
        Err(DocumentMoleculePreparationErrorV2::AromaticityResolutionFailed)
    ));
}
