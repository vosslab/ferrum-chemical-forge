use ferrum_chemistry::{
    AtomicNumber, BondOrder, ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolAtom,
    MolBond, MolGraph, Point2 as ChemistryPoint2, SmilesMolecule,
};
use ferrum_geometry::{MoleculePlacementV1, Point2};

use super::{MolblockMoleculeBuildError, prepare_molblock_molecule_for_document_v2};

struct FixedMolblockEngine {
    parsed: SmilesMolecule,
}

impl ChemEngine for FixedMolblockEngine {
    fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "smiles_to_molecule",
        })
    }

    fn molblock_to_molecule(&self, _molblock: &str) -> Result<SmilesMolecule, ChemistryError> {
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
        Ok(self.parsed.molecule().clone())
    }
}

fn carbon() -> MolAtom {
    MolAtom::new(
        AtomicNumber::from_symbol("C").expect("carbon is supported"),
        Some(0),
        None,
        Some(0),
        false,
    )
    .expect("carbon facts are valid")
}

fn engine(order: BondOrder) -> FixedMolblockEngine {
    let coordinates = Coordinates::new(vec![
        ChemistryPoint2::new(0.0, 0.0).expect("finite point"),
        ChemistryPoint2::new(2.0, 0.0).expect("finite point"),
    ]);
    let graph = MolGraph::new(
        vec![carbon(), carbon()],
        vec![MolBond::new(0, 1, order, false)],
        Some(coordinates),
    )
    .expect("test graph is structurally valid");
    FixedMolblockEngine {
        parsed: SmilesMolecule::new("CC", graph).expect("canonical result is valid"),
    }
}

fn placement() -> MoleculePlacementV1 {
    MoleculePlacementV1::new(40.0, Point2::new(100.0, 200.0).expect("finite anchor"))
        .expect("positive placement")
}

#[test]
fn complete_molblock_graph_is_placed_without_document_mutation() {
    let insertion = prepare_molblock_molecule_for_document_v2(
        &engine(BondOrder::Single),
        "accepted by fixed engine",
        placement(),
    )
    .expect("representable molblock graph must convert");

    assert_eq!(
        insertion.molecule_insertion().atoms()[0].position().x(),
        80.0
    );
    assert_eq!(
        insertion.molecule_insertion().atoms()[1].position().x(),
        120.0
    );
}

#[test]
fn unsupported_molblock_fact_is_rejected_before_a_candidate_exists() {
    let result = prepare_molblock_molecule_for_document_v2(
        &engine(BondOrder::Quadruple),
        "accepted by fixed engine",
        placement(),
    );

    assert!(matches!(
        result,
        Err(MolblockMoleculeBuildError::Preparation(_))
    ));
}
