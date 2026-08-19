use std::cell::RefCell;

use crate::{
    DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError,
    SessionOperationV1,
};
use ferrum_chemistry::{
    ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, Point2 as ChemistryPoint2,
    SmilesMolecule,
};

use super::{MoleculeCoordinateBuildError, build_molecule_coordinate_update_v1};

struct FixedCoordinateEngine {
    coordinates: Coordinates,
    received: RefCell<Option<MolGraph>>,
}

impl ChemEngine for FixedCoordinateEngine {
    fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "smiles_to_molecule",
        })
    }

    fn generate_2d_coordinates(&self, molecule: &MolGraph) -> Result<Coordinates, ChemistryError> {
        self.received.replace(Some(molecule.clone()));
        Ok(self.coordinates.clone())
    }

    fn kekulize(
        &self,
        _molecule: &MolGraph,
        _options: KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "kekulize",
        })
    }
}

fn engine(points: &[(f64, f64)]) -> FixedCoordinateEngine {
    FixedCoordinateEngine {
        coordinates: Coordinates::new(
            points
                .iter()
                .map(|&(x, y)| ChemistryPoint2::new(x, y).expect("finite test coordinate"))
                .collect(),
        ),
        received: RefCell::new(None),
    }
}

fn source(second_element: &str, bond_type: &str) -> String {
    format!(
        concat!(
            "<cdml version=\"1.0\"><molecule id=\"m1\">",
            "<atom id=\"a1\" name=\"C\"><point x=\"10\" y=\"20\" z=\"3\"/></atom>",
            "<atom id=\"a2\" name=\"{}\"><point x=\"50\" y=\"20\"/></atom>",
            "<bond id=\"b1\" start=\"a1\" end=\"a2\" type=\"{}\"/>",
            "</molecule></cdml>"
        ),
        second_element, bond_type
    )
}

#[test]
fn generated_coordinates_preserve_existing_centroid_scale_and_z_in_one_history_entry() {
    let engine = engine(&[(0.0, 0.0), (0.0, 2.0)]);
    let mut session = DocumentSession::load(&source("N", "n1")).expect("source must load");
    let observation = session.observe(0).expect("source must project");
    let molecule_id = observation.projection().molecules()[0]
        .id()
        .expect("durable molecule")
        .clone();
    let update = build_molecule_coordinate_update_v1(&engine, &observation, &molecule_id)
        .expect("ordinary molecule must prepare");
    let received = engine.received.borrow();
    let received = received.as_ref().expect("engine must receive one graph");
    assert!(received.coordinates().is_none());
    assert_eq!(received.atoms()[1].atomic_number().symbol(), "N");

    let result = session
        .submit(
            0,
            SessionOperation::V1(SessionOperationV1::SetMoleculeAtomPositions { update }),
        )
        .expect("current complete update must commit");
    let atoms = result.observation().projection().molecules()[0].atoms();
    assert_eq!(
        (atoms[0].position().x(), atoms[0].position().y()),
        (30.0, 40.0)
    );
    assert_eq!(
        (atoms[1].position().x(), atoms[1].position().y()),
        (30.0, 0.0)
    );
    assert_eq!(atoms[0].position().z(), 3.0);
    assert_eq!(atoms[1].position().z(), 0.0);
    assert_eq!(result.observation().snapshot().revision(), 1);

    let undone = session.undo(1).expect("one atomic history entry must undo");
    let atoms = undone.observation().projection().molecules()[0].atoms();
    assert_eq!(
        (atoms[0].position().x(), atoms[0].position().y()),
        (10.0, 20.0)
    );
    assert_eq!(
        (atoms[1].position().x(), atoms[1].position().y()),
        (50.0, 20.0)
    );
}

#[test]
fn prepared_coordinates_cannot_cross_same_revision_documents() {
    let engine = engine(&[(0.0, 0.0), (2.0, 0.0)]);
    let source_session = DocumentSession::load(&source("N", "n1")).expect("source must load");
    let observation = source_session.observe(0).expect("source must project");
    let molecule_id = observation.projection().molecules()[0]
        .id()
        .expect("durable molecule")
        .clone();
    let update = build_molecule_coordinate_update_v1(&engine, &observation, &molecule_id)
        .expect("source must prepare");
    let mut other = DocumentSession::load(&source("O", "n1")).expect("other source must load");
    let error = other
        .submit(
            0,
            SessionOperation::V1(SessionOperationV1::SetMoleculeAtomPositions { update }),
        )
        .expect_err("equal revision must not bypass digest provenance");
    assert!(matches!(
        error,
        DocumentSessionError::Operation(SessionOperationError::MoleculeCoordinateDigestMismatch)
    ));
    assert_eq!(other.snapshot().expect("snapshot").revision(), 0);
}

#[test]
fn unsupported_drawing_bond_style_is_not_silently_dropped() {
    let engine = engine(&[(0.0, 0.0), (2.0, 0.0)]);
    let session = DocumentSession::load(&source("N", "w1")).expect("source must load");
    let observation = session.observe(0).expect("source must project");
    let molecule_id = observation.projection().molecules()[0]
        .id()
        .expect("durable molecule");
    assert!(matches!(
        build_molecule_coordinate_update_v1(&engine, &observation, molecule_id),
        Err(MoleculeCoordinateBuildError::UnsupportedBondStyle { bond_index: 0 })
    ));
    assert!(engine.received.borrow().is_none());
}
