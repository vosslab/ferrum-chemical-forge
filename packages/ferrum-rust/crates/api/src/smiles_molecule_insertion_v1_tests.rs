use std::cell::Cell;

use ferrum_chemistry::{
    AtomicNumber, BondOrder, ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolAtom,
    MolBond, MolGraph, Point2 as ChemistryPoint2, SmilesMolecule,
};
use ferrum_document::DocumentSession;
use ferrum_geometry::{MoleculePlacementV1, Point2};

use super::{SmilesMoleculeBuildError, SmilesMoleculeInsertionError, prepare_smiles_molecule_v1};

struct FixedEngine {
    parsed: SmilesMolecule,
    kekulized: Option<MolGraph>,
    kekulize_calls: Cell<u32>,
}

impl ChemEngine for FixedEngine {
    fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
        Ok(self.parsed.clone())
    }

    fn generate_2d_coordinates(&self, _molecule: &MolGraph) -> Result<Coordinates, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "generate_2d_coordinates",
        })
    }

    fn kekulize(
        &self,
        _molecule: &MolGraph,
        _options: KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        self.kekulize_calls.set(self.kekulize_calls.get() + 1);
        self.kekulized
            .clone()
            .ok_or(ChemistryError::OperationUnavailable {
                operation: "kekulize",
            })
    }
}

fn carbon(aromatic: bool) -> MolAtom {
    MolAtom::new(
        AtomicNumber::from_symbol("C").expect("carbon is supported"),
        Some(0),
        None,
        Some(0),
        aromatic,
    )
    .expect("carbon facts are valid")
}

fn graph(order: BondOrder, aromatic: bool, coordinates: bool) -> MolGraph {
    let points = coordinates.then(|| {
        Coordinates::new(vec![
            ChemistryPoint2::new(0.0, 0.0).expect("finite point"),
            ChemistryPoint2::new(2.0, 0.0).expect("finite point"),
        ])
    });
    MolGraph::new(
        vec![carbon(aromatic), carbon(aromatic)],
        vec![MolBond::new(0, 1, order, aromatic)],
        points,
    )
    .expect("test graph is structurally valid")
}

fn engine(parsed: MolGraph, kekulized: Option<MolGraph>) -> FixedEngine {
    FixedEngine {
        parsed: SmilesMolecule::new("C=C", parsed).expect("canonical result is valid"),
        kekulized,
        kekulize_calls: Cell::new(0),
    }
}

fn placement() -> MoleculePlacementV1 {
    MoleculePlacementV1::new(40.0, Point2::new(100.0, 200.0).expect("finite anchor"))
        .expect("positive placement")
}

#[test]
fn aromatic_smiles_is_kekulized_placed_and_prepared_as_one_document_candidate() {
    let engine = engine(
        graph(BondOrder::Aromatic, true, true),
        Some(graph(BondOrder::Double, false, true)),
    );
    let mut session =
        DocumentSession::load("<cdml version=\"1.0\"/>").expect("empty document must load");
    let mut pending = prepare_smiles_molecule_v1(&engine, &mut session, 0, "c:c", placement())
        .expect("representable molecule must prepare");
    assert_eq!(engine.kekulize_calls.get(), 1);
    let accepted = session
        .commit_create_molecule(0, &mut pending)
        .expect("prepared molecule must commit");
    let molecule = &accepted.observation().projection().molecules()[0];
    assert_eq!(molecule.atoms()[0].position().x(), 80.0);
    assert_eq!(molecule.atoms()[1].position().x(), 120.0);
    assert_eq!(molecule.bonds()[0].source_type(), Some("n2"));
    assert!(
        !accepted
            .observation()
            .snapshot()
            .cdml()
            .contains("charge=\"0\"")
    );
}

#[test]
fn unsupported_bond_order_is_rejected_before_document_state_changes() {
    let engine = engine(graph(BondOrder::Quadruple, false, true), None);
    let mut session =
        DocumentSession::load("<cdml version=\"1.0\"/>").expect("empty document must load");
    let result = prepare_smiles_molecule_v1(&engine, &mut session, 0, "C$C", placement());
    assert!(matches!(
        result,
        Err(SmilesMoleculeInsertionError::Build(
            SmilesMoleculeBuildError::UnsupportedBondOrder {
                order: BondOrder::Quadruple,
                ..
            }
        ))
    ));
    assert_eq!(
        session
            .snapshot()
            .expect("snapshot remains available")
            .revision(),
        0
    );
}

#[test]
fn missing_engine_coordinates_are_a_typed_preparation_failure() {
    let engine = engine(graph(BondOrder::Single, false, false), None);
    let mut session =
        DocumentSession::load("<cdml version=\"1.0\"/>").expect("empty document must load");
    assert!(matches!(
        prepare_smiles_molecule_v1(&engine, &mut session, 0, "CC", placement()),
        Err(SmilesMoleculeInsertionError::Build(
            SmilesMoleculeBuildError::MissingCoordinates
        ))
    ));
}
