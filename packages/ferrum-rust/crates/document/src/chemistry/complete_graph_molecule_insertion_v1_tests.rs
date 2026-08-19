use crate::MoleculeInsertionBondOrderV1;
use ferrum_chemistry::{
    AtomicNumber, BondOrder, Coordinates, MolAtom, MolBond, MolGraph, Point2 as ChemistryPoint2,
};
use ferrum_geometry::{MoleculePlacementV1, Point2};

use super::{CompleteGraphMoleculeInsertionError, build_complete_graph_molecule_insertion_v1};

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

#[test]
fn v2000_shape_maps_atom_bond_and_coordinate_order_without_parser_specific_state() {
    let graph = MolGraph::new(
        vec![atom("C", false), atom("O", false)],
        vec![MolBond::new(0, 1, BondOrder::Double, false)],
        Some(coordinates(&[(0.0, 0.0), (2.0, 0.0)])),
    )
    .expect("valid V2000-shaped owned graph");

    let insertion = build_complete_graph_molecule_insertion_v1(&graph, placement())
        .expect("closed V2000 graph must convert");

    assert_eq!(insertion.atoms()[0].element(), "C");
    assert_eq!(insertion.atoms()[1].element(), "O");
    assert_eq!(insertion.atoms()[0].position().x(), 80.0);
    assert_eq!(insertion.atoms()[1].position().x(), 120.0);
    assert_eq!(insertion.bonds()[0].start(), 0);
    assert_eq!(insertion.bonds()[0].end(), 1);
    assert_eq!(
        insertion.bonds()[0].order(),
        MoleculeInsertionBondOrderV1::Double
    );
}

#[test]
fn v3000_shape_maps_triple_bond_in_source_order() {
    let graph = MolGraph::new(
        vec![atom("N", false), atom("C", false), atom("O", false)],
        vec![
            MolBond::new(1, 2, BondOrder::Double, false),
            MolBond::new(0, 1, BondOrder::Triple, false),
        ],
        Some(coordinates(&[(-1.0, 0.0), (0.0, 0.0), (1.0, 0.0)])),
    )
    .expect("valid V3000-shaped owned graph");

    let insertion = build_complete_graph_molecule_insertion_v1(&graph, placement())
        .expect("closed V3000 graph must convert");

    assert_eq!(insertion.bonds()[0].start(), 1);
    assert_eq!(insertion.bonds()[0].end(), 2);
    assert_eq!(
        insertion.bonds()[0].order(),
        MoleculeInsertionBondOrderV1::Double
    );
    assert_eq!(insertion.bonds()[1].start(), 0);
    assert_eq!(insertion.bonds()[1].end(), 1);
    assert_eq!(
        insertion.bonds()[1].order(),
        MoleculeInsertionBondOrderV1::Triple
    );
}

#[test]
fn missing_complete_coordinates_are_rejected_before_candidate_construction() {
    let graph = MolGraph::new(vec![atom("C", false)], vec![], None).expect("valid graph");

    assert!(matches!(
        build_complete_graph_molecule_insertion_v1(&graph, placement()),
        Err(CompleteGraphMoleculeInsertionError::MissingCoordinates)
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
        build_complete_graph_molecule_insertion_v1(&graph, placement()),
        Err(CompleteGraphMoleculeInsertionError::UnsupportedBondOrder {
            order: BondOrder::Quadruple,
            ..
        })
    ));
}

#[test]
fn unresolved_aromatic_facts_are_rejected_without_implicit_kekulization() {
    let graph = MolGraph::new(
        vec![atom("C", true), atom("C", true)],
        vec![MolBond::new(0, 1, BondOrder::Aromatic, true)],
        Some(coordinates(&[(0.0, 0.0), (1.0, 0.0)])),
    )
    .expect("pre-kekulized graph is structurally valid");

    assert!(matches!(
        build_complete_graph_molecule_insertion_v1(&graph, placement()),
        Err(CompleteGraphMoleculeInsertionError::UnsupportedAtomFact {
            fact: "unresolved aromaticity",
            ..
        })
    ));
}
