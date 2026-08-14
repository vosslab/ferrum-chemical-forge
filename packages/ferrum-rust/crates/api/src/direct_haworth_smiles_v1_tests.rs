use ferrum_chemistry::{
    AtomicNumber, BondOrder, ChemEngine, ChemistryError, Coordinates, MolAtom, MolBond, MolGraph,
    Point2, SmilesMolecule,
};
use ferrum_geometry::{MoleculePlacementV1, Point2 as PlacementPoint};
use std::collections::BTreeSet;

use super::direct_haworth_smiles_v1::{
    DirectHaworthFromSmilesBuildErrorV1, build_direct_haworth_from_smiles_with_engine_for_test,
};

#[test]
fn public_builder_requires_the_concrete_native_engine() {
    let _: fn(
        &ferrum_chemistry::NativeChemEngine,
        &str,
        MoleculePlacementV1,
    ) -> Result<
        super::PreparedDirectHaworthFromSmilesV1,
        DirectHaworthFromSmilesBuildErrorV1,
    > = super::build_direct_haworth_from_smiles_v1;
}

struct FixedEngine(SmilesMolecule);

impl ChemEngine for FixedEngine {
    fn smiles_to_molecule(&self, _: &str) -> Result<SmilesMolecule, ChemistryError> {
        Ok(self.0.clone())
    }
    fn generate_2d_coordinates(&self, _: &MolGraph) -> Result<Coordinates, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "unused",
        })
    }
    fn kekulize(
        &self,
        _: &MolGraph,
        _: ferrum_chemistry::KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "unused",
        })
    }
}

fn profile(
    first: usize,
    second: usize,
    first_attachment: usize,
    second_attachment: usize,
) -> MolGraph {
    let bridge = first + second;
    let atoms = (0..=bridge)
        .map(|index| {
            MolAtom::new(
                AtomicNumber::from_symbol(if index == 0 || index == first || index == bridge {
                    "O"
                } else {
                    "C"
                })
                .expect("element"),
                Some(0),
                None,
                Some(0),
                false,
            )
            .expect("atom")
        })
        .collect();
    let mut bonds: Vec<_> = (0..first)
        .map(|index| MolBond::new(index, (index + 1) % first, BondOrder::Single, false))
        .collect();
    bonds.extend((0..second).map(|index| {
        MolBond::new(
            first + index,
            first + (index + 1) % second,
            BondOrder::Single,
            false,
        )
    }));
    bonds.push(MolBond::new(
        first_attachment,
        bridge,
        BondOrder::Single,
        false,
    ));
    bonds.push(MolBond::new(
        first + second_attachment,
        bridge,
        BondOrder::Single,
        false,
    ));
    MolGraph::new(
        atoms,
        bonds,
        Some(Coordinates::new(vec![
            Point2::new(0.0, 0.0).expect("point");
            bridge + 1
        ])),
    )
    .expect("graph")
}

#[test]
fn accepts_all_closed_ring_forms_and_nonadjacent_bridge_attachment() {
    for (first, second) in [(5, 5), (5, 6), (6, 5), (6, 6)] {
        let graph = profile(first, second, 2, 3);
        let engine = FixedEngine(SmilesMolecule::new("CO", graph).expect("result"));
        let prepared = build_direct_haworth_from_smiles_with_engine_for_test(
            &engine,
            "CO",
            MoleculePlacementV1::new(7.0, PlacementPoint::new(11.0, -3.0).expect("anchor"))
                .expect("placement"),
        )
        .expect("closed profile");
        let atoms = prepared.receipt().atoms_in_canonical_order();
        let centroid = HaworthCentroid::of(atoms);
        assert_eq!(prepared.translation().x, 11.0 - centroid.0);
        assert_eq!(prepared.translation().y, -3.0 - centroid.1);
    }
}

#[test]
fn accepts_bridge_carbons_adjacent_to_and_separated_from_ring_oxygen() {
    for (first_attachment, second_attachment) in [(1, 1), (2, 3)] {
        let engine = FixedEngine(
            SmilesMolecule::new("CO", profile(6, 5, first_attachment, second_attachment))
                .expect("result"),
        );
        build_direct_haworth_from_smiles_with_engine_for_test(
            &engine,
            "CO",
            MoleculePlacementV1::new(5.0, PlacementPoint::new(0.0, 0.0).expect("anchor"))
                .expect("placement"),
        )
        .expect("closed profile regardless of bridge-carbon distance from ring oxygen");
    }
}

#[test]
fn repeated_raw_adapter_order_produces_the_same_frozen_receipt() {
    let graph = profile(5, 6, 2, 3);
    let placement = MoleculePlacementV1::new(5.0, PlacementPoint::new(4.0, -9.0).expect("anchor"))
        .expect("placement");
    let first = build_direct_haworth_from_smiles_with_engine_for_test(
        &FixedEngine(SmilesMolecule::new("CO", graph.clone()).expect("result")),
        "CO",
        placement,
    )
    .expect("first receipt");
    let second = build_direct_haworth_from_smiles_with_engine_for_test(
        &FixedEngine(SmilesMolecule::new("CO", graph).expect("result")),
        "CO",
        placement,
    )
    .expect("second receipt");
    assert_eq!(first, second);

    let expected_atoms: BTreeSet<_> = (0..12).map(|index| source_id("atom", index)).collect();
    assert_eq!(
        first
            .receipt()
            .atoms_in_canonical_order()
            .iter()
            .map(|atom| atom.source_atom_identity().clone())
            .collect::<BTreeSet<_>>(),
        expected_atoms
    );
    let expected_bonds: BTreeSet<_> = (0..13).map(|index| source_id("bond", index)).collect();
    assert_eq!(
        first
            .receipt()
            .bonds_in_canonical_order()
            .iter()
            .map(|bond| bond.source_bond_identity().clone())
            .collect::<BTreeSet<_>>(),
        expected_bonds
    );
    let bridge_bond = source_id("bond", 11);
    let fact = first
        .receipt()
        .bonds_in_canonical_order()
        .iter()
        .find(|bond| bond.source_bond_identity() == &bridge_bond)
        .expect("raw first bridge bond remains identifiable");
    assert_eq!(
        fact.endpoints(),
        &[source_id("atom", 2), source_id("atom", 11)],
        "the nonadjacent raw bridge attachment remains the actual bridge carbon"
    );
}

fn source_id(kind: &str, index: usize) -> ferrum_core::RecordId {
    let identifier = ferrum_core::Identifier::new(format!("native-direct-haworth-{kind}-{index}"))
        .expect("identifier");
    ferrum_core::RecordId::from_source(
        if kind == "atom" {
            ferrum_core::RecordKind::Atom
        } else {
            ferrum_core::RecordKind::Bond
        },
        &identifier,
    )
}

struct HaworthCentroid;
impl HaworthCentroid {
    fn of(
        atoms: &[ferrum_domain::haworth::DirectGlycosidicHaworthSelectedAtomFactV1],
    ) -> (f64, f64) {
        let count = atoms.len() as f64;
        (
            atoms.iter().map(|atom| atom.local().x).sum::<f64>() / count,
            atoms.iter().map(|atom| atom.local().y).sum::<f64>() / count,
        )
    }
}

#[test]
fn rejects_extra_topology_before_authoring() {
    let mut graph = profile(6, 6, 1, 1);
    let atoms = graph.atoms().to_vec();
    let mut bonds = graph.bonds().to_vec();
    bonds.push(MolBond::new(1, 3, BondOrder::Single, false));
    graph = MolGraph::new(atoms, bonds, graph.coordinates().cloned()).expect("graph");
    let engine = FixedEngine(SmilesMolecule::new("CO", graph).expect("result"));
    assert!(matches!(
        build_direct_haworth_from_smiles_with_engine_for_test(
            &engine,
            "CO",
            MoleculePlacementV1::new(7.0, PlacementPoint::new(0.0, 0.0).expect("anchor"))
                .expect("placement"),
        ),
        Err(DirectHaworthFromSmilesBuildErrorV1::Profile { .. })
    ));
}

#[test]
fn rejects_returned_unsupported_atom_facts_without_parsing_normalization() {
    let graph = profile(5, 6, 1, 1);
    let mut atoms = graph.atoms().to_vec();
    atoms[2] = MolAtom::new(
        AtomicNumber::from_symbol("C").expect("carbon"),
        Some(1),
        None,
        Some(0),
        false,
    )
    .expect("charged atom fact");
    let graph = MolGraph::new(atoms, graph.bonds().to_vec(), graph.coordinates().cloned())
        .expect("raw graph");
    let engine = FixedEngine(SmilesMolecule::new("CO", graph).expect("result"));
    assert!(matches!(
        build_direct_haworth_from_smiles_with_engine_for_test(
            &engine,
            "CO",
            MoleculePlacementV1::new(5.0, PlacementPoint::new(0.0, 0.0).expect("anchor"))
                .expect("placement"),
        ),
        Err(DirectHaworthFromSmilesBuildErrorV1::UnsupportedAtomFact {
            index: 2,
            fact: "formal charge",
        })
    ));
}

#[test]
fn rejects_returned_unsupported_bond_facts_without_kekulization() {
    let graph = profile(5, 6, 1, 1);
    let mut bonds = graph.bonds().to_vec();
    bonds[0] = MolBond::new(0, 1, BondOrder::Double, false);
    let graph = MolGraph::new(graph.atoms().to_vec(), bonds, graph.coordinates().cloned())
        .expect("raw graph");
    let engine = FixedEngine(SmilesMolecule::new("CO", graph).expect("result"));
    assert!(matches!(
        build_direct_haworth_from_smiles_with_engine_for_test(
            &engine,
            "CO",
            MoleculePlacementV1::new(5.0, PlacementPoint::new(0.0, 0.0).expect("anchor"))
                .expect("placement"),
        ),
        Err(DirectHaworthFromSmilesBuildErrorV1::UnsupportedBondFact {
            index: 0,
            fact: "non-single order",
        })
    ));
}
