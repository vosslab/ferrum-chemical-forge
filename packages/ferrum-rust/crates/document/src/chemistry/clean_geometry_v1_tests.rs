use std::cell::Cell;

use crate::{
    CleanGeometryMoleculeV1, CleanGeometryUpdateV1, DocumentSession, SessionOperation,
    SessionOperationV1,
};
use ferrum_chemistry::{
    ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, Point2 as ChemistryPoint2,
    SmilesMolecule,
};
use ferrum_geometry::Point2;

use super::{CleanGeometryBuildError, build_clean_geometry_update_v1};

const HALF_AUTHORED_UNIT_POINTS: f64 = (0.001 * 72.0 / 2.54) / 2.0;

fn assert_authored_close(actual: f64, expected: f64) {
    let floating_point_slack = f64::EPSILON * actual.abs().max(expected.abs()).max(1.0);
    assert!(
        (actual - expected).abs() <= HALF_AUTHORED_UNIT_POINTS + floating_point_slack,
        "expected {expected} points within one half authored unit, got {actual}",
    );
}

struct FixedCleanEngine {
    calls: Cell<usize>,
    truncate_result: bool,
}

impl ChemEngine for FixedCleanEngine {
    fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "smiles_to_molecule",
        })
    }

    fn generate_2d_coordinates(&self, molecule: &MolGraph) -> Result<Coordinates, ChemistryError> {
        self.calls.set(self.calls.get() + 1);
        let mut points = (0..molecule.atoms().len())
            .map(|index| {
                ChemistryPoint2::new(0.0, index as f64 * 2.0).expect("finite generated test point")
            })
            .collect::<Vec<_>>();
        if self.truncate_result {
            points.pop();
        }
        Ok(Coordinates::new(points))
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

fn engine() -> FixedCleanEngine {
    FixedCleanEngine {
        calls: Cell::new(0),
        truncate_result: false,
    }
}

#[test]
fn clean_geometry_commits_multiple_centroid_preserving_layouts_atomically() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\">",
        "<molecule id=\"first\" retained=\"yes\">",
        "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\" z=\"7\"/>",
        "<v:note>keep</v:note></atom>",
        "<atom id=\"b\" name=\"N\"><point x=\"20\" y=\"0\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/></molecule>",
        "<molecule id=\"second\"><atom id=\"c\" name=\"C\">",
        "<point x=\"100\" y=\"20\"/></atom><atom id=\"d\" name=\"O\">",
        "<point x=\"100\" y=\"60\"/></atom>",
        "<bond id=\"cd\" start=\"c\" end=\"d\" type=\"n1\"/></molecule></cdml>",
    );
    let engine = engine();
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let observation = session.observe(0).expect("fixture projects");
    let molecule_ids = observation
        .projection()
        .molecules()
        .iter()
        .map(|molecule| molecule.id().expect("durable molecule").clone())
        .collect::<Vec<_>>();
    let update = build_clean_geometry_update_v1(&engine, &observation, &molecule_ids, 10.0)
        .expect("both layouts prepare");
    assert_eq!(engine.calls.get(), 2);
    let repaired = session
        .apply_document_operation_v1(
            0,
            SessionOperation::V1(SessionOperationV1::SetCleanGeometry { update }),
        )
        .expect("one batch commits");
    let molecules = repaired.observation().projection().molecules();
    let first = molecules[0].atoms();
    let second = molecules[1].atoms();
    assert_authored_close(first[0].position().x(), 10.0);
    assert_authored_close(first[0].position().y(), 5.0);
    assert_authored_close(first[1].position().x(), 10.0);
    assert_authored_close(first[1].position().y(), -5.0);
    assert_eq!(first[0].position().z(), 7.0);
    assert_authored_close(second[0].position().x(), 100.0);
    assert_authored_close(second[0].position().y(), 45.0);
    assert_authored_close(second[1].position().x(), 100.0);
    assert_authored_close(second[1].position().y(), 35.0);
    assert_eq!(repaired.observation().snapshot().revision(), 1);
    let cdml = repaired.observation().snapshot().cdml();
    assert!(cdml.contains("<v:note>keep</v:note>"));
    assert!(cdml.contains("retained=\"yes\""));
    assert_eq!(
        session
            .undo(1)
            .expect("one history entry restores both molecules")
            .observation()
            .snapshot()
            .revision(),
        2
    );
}

#[test]
fn clean_geometry_validates_every_target_before_native_generation() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"good\"><atom id=\"a\" name=\"C\">",
        "<point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"O\">",
        "<point x=\"10\" y=\"0\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/></molecule>",
        "<molecule id=\"bad\"><atom id=\"c\" name=\"C\">",
        "<point x=\"20\" y=\"0\"/></atom><atom id=\"d\" name=\"O\">",
        "<point x=\"30\" y=\"0\"/></atom>",
        "<bond id=\"cd\" start=\"c\" end=\"d\" type=\"w1\"/></molecule></cdml>",
    );
    let engine = engine();
    let session = DocumentSession::load(source).expect("fixture loads");
    let observation = session.observe(0).expect("fixture projects");
    let molecule_ids = observation
        .projection()
        .molecules()
        .iter()
        .map(|molecule| molecule.id().expect("durable molecule").clone())
        .collect::<Vec<_>>();
    assert!(matches!(
        build_clean_geometry_update_v1(&engine, &observation, &molecule_ids, 10.0),
        Err(CleanGeometryBuildError::Target { .. })
    ));
    assert_eq!(engine.calls.get(), 0);
}

#[test]
fn clean_geometry_rejects_invalid_envelopes_before_native_generation() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\">",
        "<point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    );
    let engine = engine();
    let session = DocumentSession::load(source).expect("fixture loads");
    let observation = session.observe(0).expect("fixture projects");
    let molecule_id = observation.projection().molecules()[0]
        .id()
        .expect("durable molecule")
        .clone();
    assert!(matches!(
        build_clean_geometry_update_v1(&engine, &observation, &[], 10.0),
        Err(CleanGeometryBuildError::EmptyMolecules)
    ));
    assert!(matches!(
        build_clean_geometry_update_v1(
            &engine,
            &observation,
            &[molecule_id.clone(), molecule_id.clone()],
            10.0,
        ),
        Err(CleanGeometryBuildError::DuplicateMolecule)
    ));
    assert!(matches!(
        build_clean_geometry_update_v1(&engine, &observation, &[molecule_id], 0.0),
        Err(CleanGeometryBuildError::InvalidTargetSpacing)
    ));
    assert_eq!(engine.calls.get(), 0);
}

#[test]
fn clean_geometry_rejects_a_malformed_native_coordinate_count() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\">",
        "<point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"O\">",
        "<point x=\"10\" y=\"0\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/></molecule></cdml>",
    );
    let engine = FixedCleanEngine {
        calls: Cell::new(0),
        truncate_result: true,
    };
    let session = DocumentSession::load(source).expect("fixture loads");
    let observation = session.observe(0).expect("fixture projects");
    let molecule_id = observation.projection().molecules()[0]
        .id()
        .expect("durable molecule")
        .clone();

    assert!(matches!(
        build_clean_geometry_update_v1(&engine, &observation, &[molecule_id], 10.0),
        Err(CleanGeometryBuildError::GeneratedAtomCountMismatch {
            expected: 2,
            actual: 1,
            ..
        })
    ));
    assert_eq!(engine.calls.get(), 1);
}

#[test]
fn clean_geometry_rejects_stale_preparation_without_mutation() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\">",
        "<point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"O\">",
        "<point x=\"10\" y=\"0\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/></molecule></cdml>",
    );
    let engine = engine();
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let observation = session.observe(0).expect("fixture projects");
    let molecule_id = observation.projection().molecules()[0]
        .id()
        .expect("durable molecule")
        .clone();
    let update = build_clean_geometry_update_v1(&engine, &observation, &[molecule_id], 10.0)
        .expect("clean geometry prepares");
    let changed = session
        .apply_document_operation_v1(
            0,
            SessionOperation::V1(SessionOperationV1::SetAtomElement {
                atom_id: "a".to_owned(),
                element: "N".to_owned(),
            }),
        )
        .expect("intervening mutation commits")
        .observation()
        .snapshot()
        .clone();

    assert!(
        session
            .apply_document_operation_v1(
                changed.revision(),
                SessionOperation::V1(SessionOperationV1::SetCleanGeometry { update }),
            )
            .is_err()
    );
    let retained = session
        .observe(changed.revision())
        .expect("stale rejection retains current revision");
    assert_eq!(retained.snapshot().digest(), changed.digest());
    assert_eq!(retained.snapshot().cdml(), changed.cdml());
}

#[test]
fn clean_geometry_equal_authored_coordinates_do_not_create_history() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\">",
        "<point x=\"1.000cm\" y=\"2.000cm\"/></atom>",
        "<atom id=\"b\" name=\"N\"><point x=\"2.000cm\" y=\"2.000cm\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/></molecule></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let observation = session.observe(0).expect("fixture projects");
    let molecule = &observation.projection().molecules()[0];
    let molecule_id = molecule.id().expect("durable molecule").clone();
    let positions = molecule
        .atoms()
        .iter()
        .map(|atom| Point2::new(atom.position().x(), atom.position().y()).expect("finite point"))
        .collect();
    let update = CleanGeometryUpdateV1::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        vec![CleanGeometryMoleculeV1::new(molecule_id, positions).expect("valid target")],
    )
    .expect("valid batch");

    let unchanged = session
        .apply_document_operation_v1(
            0,
            SessionOperation::V1(SessionOperationV1::SetCleanGeometry { update }),
        )
        .expect("equal authored coordinates are a no-op");
    assert_eq!(unchanged.observation().snapshot().revision(), 0);
    assert_eq!(unchanged.observation().snapshot().cdml(), source);
}

#[test]
fn clean_geometry_rejects_a_later_count_mismatch_without_partial_mutation() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"first\"><atom id=\"a\" name=\"C\">",
        "<point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"N\">",
        "<point x=\"10\" y=\"0\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/></molecule>",
        "<molecule id=\"second\"><atom id=\"c\" name=\"C\">",
        "<point x=\"20\" y=\"0\"/></atom><atom id=\"d\" name=\"O\">",
        "<point x=\"30\" y=\"0\"/></atom>",
        "<bond id=\"cd\" start=\"c\" end=\"d\" type=\"n1\"/></molecule></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let observation = session.observe(0).expect("fixture projects");
    let ids = observation
        .projection()
        .molecules()
        .iter()
        .map(|molecule| molecule.id().expect("durable molecule").clone())
        .collect::<Vec<_>>();
    let update = CleanGeometryUpdateV1::new(
        0,
        *observation.snapshot().digest(),
        vec![
            CleanGeometryMoleculeV1::new(
                ids[0].clone(),
                vec![
                    Point2::new(1.0, 1.0).expect("finite point"),
                    Point2::new(2.0, 2.0).expect("finite point"),
                ],
            )
            .expect("valid first target"),
            CleanGeometryMoleculeV1::new(
                ids[1].clone(),
                vec![Point2::new(3.0, 3.0).expect("finite point")],
            )
            .expect("shape validation belongs to the document"),
        ],
    )
    .expect("valid provenance envelope");

    assert!(
        session
            .apply_document_operation_v1(
                0,
                SessionOperation::V1(SessionOperationV1::SetCleanGeometry { update }),
            )
            .is_err()
    );
    let retained = session
        .observe(0)
        .expect("failed batch leaves revision zero");
    assert_eq!(retained.snapshot().revision(), 0);
    assert_eq!(retained.snapshot().cdml(), source);
}
