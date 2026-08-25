use super::{
    AdmittedSessionTransitionRefusalV1, DocumentBondOrderV1, DocumentBondPresentationV1,
    DocumentDirectedBondDepictionV1, DocumentDoubleBondCarrierMarkDepictionV1,
    DocumentDoubleBondCarrierMarkV1, DocumentDoubleBondConfigurationV1, DocumentDoubleBondStereoV1,
    DocumentSession, DocumentStereoDepictionReportV1, DocumentStereoLigandV1,
    DocumentStereoSemanticReportV1, DocumentTetrahedralParityV1, DocumentTetrahedralStereoV1,
    MoleculeInsertionAtomV1, MoleculeInsertionBondV1, MoleculeInsertionRequestV1,
    MoleculeInsertionV1, MoleculeInsertionV1Error, Point3V1, PreparedDocumentMoleculeV2,
    SessionOperation, SessionOperationOutcomeV1, SessionOperationTransitionRequestV1,
    SessionOperationV1, TransitionAuthorizationV1, TypedDocument,
};
use ferrum_render::{DoubleBondCarrierMarkDirectionV1, RenderOp};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\"><opaque id=\"ferrum-molecule-v1-0\"/>",
    "<opaque id=\"ferrum-atom-v1-0\"/><opaque id=\"ferrum-bond-v1-0\"/></cdml>"
);

fn atom(element: &str, x: f64) -> MoleculeInsertionAtomV1 {
    MoleculeInsertionAtomV1::new(
        element,
        Point3V1::new(x, 20.0, 0.0).expect("finite test position"),
        None,
        None,
        None,
    )
    .expect("valid test atom")
}

fn carbonyl() -> MoleculeInsertionV1 {
    MoleculeInsertionV1::new(
        vec![atom("C", 10.0), atom("O", 30.0)],
        vec![MoleculeInsertionBondV1::new(
            0,
            1,
            DocumentBondOrderV1::Double,
        )],
    )
    .expect("valid test graph")
}

fn request(revision: u64, molecule: MoleculeInsertionV1) -> SessionOperationTransitionRequestV1 {
    SessionOperationTransitionRequestV1::new(
        revision,
        SessionOperation::V1(SessionOperationV1::InsertMoleculeV1(molecule.into())),
        TransitionAuthorizationV1::None,
    )
}

#[test]
fn insertion_graph_rejects_ambiguous_or_impossible_edges() {
    assert_eq!(
        MoleculeInsertionV1::new(
            vec![atom("C", 0.0)],
            vec![MoleculeInsertionBondV1::new(
                0,
                0,
                DocumentBondOrderV1::Single,
            )],
        ),
        Err(MoleculeInsertionV1Error::SelfBond { atom: 0 })
    );
    assert_eq!(
        MoleculeInsertionV1::new(Vec::new(), Vec::new()),
        Err(MoleculeInsertionV1Error::EmptyMolecule)
    );
}

#[test]
fn generic_molecule_insertion_publishes_ids_only_after_commit_and_is_one_history_step() {
    let mut session = DocumentSession::load(SOURCE).expect("source loads");
    let baseline = session.snapshot().expect("baseline snapshot");
    let mut prepared = session
        .prepare_session_operation_transition_v1(request(0, carbonyl()))
        .expect("generic transition prepares");
    assert_eq!(session.snapshot().expect("preparation is inert"), baseline);

    let accepted = session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("generic transition commits");
    let SessionOperationOutcomeV1::MoleculeInsertedV1(outcome) = accepted.outcome() else {
        panic!("commit publishes molecule insertion facts");
    };
    assert_eq!(
        outcome.molecule_identifier().as_str(),
        "ferrum-molecule-v1-1"
    );
    assert_eq!(outcome.atom_identifiers()[0].as_str(), "ferrum-atom-v1-1");
    assert_eq!(outcome.atom_identifiers()[1].as_str(), "ferrum-atom-v1-2");
    assert_eq!(outcome.bond_identifiers()[0].as_str(), "ferrum-bond-v1-1");
    assert_eq!(accepted.observation().snapshot().revision(), 1);
    assert!(
        session
            .undo(1)
            .expect("one insertion undoes")
            .observation()
            .projection()
            .molecules()
            .is_empty()
    );
}

#[test]
fn generic_molecule_transition_refusals_leave_state_and_id_allocation_unchanged() {
    let mut owner = DocumentSession::create_empty_document_v1().expect("owner creates");
    let mut foreign = DocumentSession::create_empty_document_v1().expect("foreign creates");
    let mut prepared = owner
        .prepare_session_operation_transition_v1(request(0, carbonyl()))
        .expect("transition prepares");
    let baseline = owner.snapshot().expect("baseline snapshot");
    assert_eq!(
        foreign.commit_session_operation_transition_v1(&mut prepared),
        Err(AdmittedSessionTransitionRefusalV1::ForeignSession)
    );
    assert_eq!(
        owner.snapshot().expect("foreign refusal is inert"),
        baseline
    );
    owner
        .retire_session_operation_transition_v1(&mut prepared)
        .expect("transition retires");
    assert_eq!(
        owner.commit_session_operation_transition_v1(&mut prepared),
        Err(AdmittedSessionTransitionRefusalV1::Replayed)
    );
    let mut fresh = owner
        .prepare_session_operation_transition_v1(request(0, carbonyl()))
        .expect("equivalent request prepares after retirement");
    let accepted = owner
        .commit_session_operation_transition_v1(&mut fresh)
        .expect("fresh transition commits");
    let SessionOperationOutcomeV1::MoleculeInsertedV1(outcome) = accepted.outcome() else {
        panic!("committed outcome is molecule insertion");
    };
    assert_eq!(
        outcome.molecule_identifier().as_str(),
        "ferrum-molecule-v1-0"
    );
}

#[test]
fn stereo_cdml_round_trip_v2_uses_one_generic_molecule_transition() {
    let molecule = MoleculeInsertionV1::new(
        vec![
            MoleculeInsertionAtomV1::new(
                "C",
                Point3V1::new(0.0, 0.0, 0.0).expect("finite center"),
                None,
                None,
                Some(1),
            )
            .expect("valid center"),
            atom("Cl", 20.0),
            atom("Br", 40.0),
            atom("F", 60.0),
        ],
        vec![
            MoleculeInsertionBondV1::new_with_presentation(
                0,
                1,
                DocumentBondPresentationV1::SolidWedge,
            ),
            MoleculeInsertionBondV1::new(0, 2, DocumentBondOrderV1::Single),
            MoleculeInsertionBondV1::new(0, 3, DocumentBondOrderV1::Single),
        ],
    )
    .expect("valid tetrahedral topology");
    let report = DocumentStereoSemanticReportV1::new(
        vec![
            DocumentTetrahedralStereoV1::new(
                0,
                [
                    DocumentStereoLigandV1::Atom(1),
                    DocumentStereoLigandV1::Atom(2),
                    DocumentStereoLigandV1::Atom(3),
                    DocumentStereoLigandV1::ExplicitHydrogen,
                ],
                DocumentTetrahedralParityV1::Clockwise,
            )
            .expect("admitted tetrahedral semantics"),
        ],
        vec![],
    );
    let depictions = DocumentStereoDepictionReportV1::new(
        vec![
            DocumentDirectedBondDepictionV1::new(0, 0, 1, DocumentBondPresentationV1::SolidWedge)
                .expect("matching wedge depiction"),
        ],
        Vec::new(),
    );
    let prepared = PreparedDocumentMoleculeV2::with_stereo_reports(
        molecule,
        Some(report),
        Some(depictions.clone()),
    )
    .expect("graph-relative tetrahedral facts are admitted");
    let expected_semantics = prepared
        .stereo_semantics()
        .expect("prepared molecule retains semantics")
        .clone();
    let request = prepared
        .into_molecule_insertion_request_v1()
        .expect("prepared semantics remain valid for generic insertion");
    let mut session = DocumentSession::load(SOURCE).expect("source loads");
    let accepted = session
        .apply_document_operation_v1(
            0,
            SessionOperation::V1(SessionOperationV1::InsertMoleculeV1(request)),
        )
        .expect("one generic operation commits");
    assert_eq!(accepted.observation().snapshot().revision(), 1);
    let saved = session.snapshot().expect("saved snapshot");
    let reopened = DocumentSession::load(saved.cdml()).expect("canonical CDML reopens");
    let observation = reopened.observe(0).expect("reopened session observes");
    let molecule_id = observation.projection().molecules()[0]
        .id()
        .expect("inserted molecule has one durable ID");
    let typed = TypedDocument::parse(saved.cdml()).expect("saved CDML types");
    let semantics = typed
        .molecule_stereo_semantics_v1(molecule_id)
        .expect("canonical semantics decode");
    let semantics = semantics.expect("inserted molecule retains semantics");
    assert_eq!(semantics, expected_semantics);
    assert_eq!(
        observation
            .molecule_stereo_depictions_v1(molecule_id)
            .expect("canonical depictions decode"),
        Some(depictions)
    );
}

#[test]
fn generic_stereo_request_refuses_a_self_ligand_before_session_mutation() {
    let molecule = MoleculeInsertionV1::new(
        vec![
            MoleculeInsertionAtomV1::new(
                "C",
                Point3V1::new(0.0, 0.0, 0.0).expect("finite center"),
                None,
                None,
                Some(1),
            )
            .expect("valid center"),
            atom("Cl", 20.0),
            atom("Br", 40.0),
            atom("F", 60.0),
        ],
        vec![
            MoleculeInsertionBondV1::new(0, 1, DocumentBondOrderV1::Single),
            MoleculeInsertionBondV1::new(0, 2, DocumentBondOrderV1::Single),
            MoleculeInsertionBondV1::new(0, 3, DocumentBondOrderV1::Single),
        ],
    )
    .expect("valid topology");
    let report = DocumentStereoSemanticReportV1::new(
        vec![
            DocumentTetrahedralStereoV1::new(
                0,
                [
                    DocumentStereoLigandV1::Atom(0),
                    DocumentStereoLigandV1::Atom(1),
                    DocumentStereoLigandV1::Atom(2),
                    DocumentStereoLigandV1::ExplicitHydrogen,
                ],
                DocumentTetrahedralParityV1::Clockwise,
            )
            .expect("local descriptor shape is valid"),
        ],
        vec![],
    );
    assert!(MoleculeInsertionRequestV1::with_stereo_semantics(molecule, report).is_err());
}

#[test]
fn generic_stereo_request_keeps_ez_carrier_depiction_separate_from_semantics() {
    let molecule = MoleculeInsertionV1::new(
        vec![
            atom("C", 0.0),
            atom("C", 20.0),
            atom("F", -20.0),
            atom("Cl", 40.0),
        ],
        vec![
            MoleculeInsertionBondV1::new(0, 1, DocumentBondOrderV1::Double),
            MoleculeInsertionBondV1::new(0, 2, DocumentBondOrderV1::Single),
            MoleculeInsertionBondV1::new(1, 3, DocumentBondOrderV1::Single),
        ],
    )
    .expect("valid E/Z graph");
    let semantics = DocumentStereoSemanticReportV1::new(
        Vec::new(),
        vec![
            DocumentDoubleBondStereoV1::new(0, 2, 3, DocumentDoubleBondConfigurationV1::E)
                .expect("valid E/Z descriptor"),
        ],
    );
    let depictions = DocumentStereoDepictionReportV1::new(
        Vec::new(),
        vec![DocumentDoubleBondCarrierMarkDepictionV1::new(
            0,
            1,
            DocumentDoubleBondCarrierMarkV1::Up,
        )],
    );

    let request = MoleculeInsertionRequestV1::with_stereo_reports(
        molecule,
        Some(semantics),
        Some(depictions),
    )
    .expect("admitted carrier mark references the selected E/Z ligand");

    assert_eq!(
        request
            .stereo_semantics()
            .expect("chemical facts")
            .double_bonds()
            .len(),
        1
    );
    assert_eq!(
        request
            .stereo_depictions()
            .expect("drawing facts")
            .double_bond_carrier_marks()[0]
            .mark(),
        DocumentDoubleBondCarrierMarkV1::Up
    );
}

#[test]
fn generic_stereo_request_admits_two_ez_descriptors_that_share_one_carrier() {
    let molecule = MoleculeInsertionV1::new(
        vec![
            atom("C", 0.0),
            atom("C", 20.0),
            atom("C", 40.0),
            atom("C", 60.0),
            atom("F", -20.0),
            atom("Cl", 80.0),
        ],
        vec![
            MoleculeInsertionBondV1::new(0, 1, DocumentBondOrderV1::Double),
            MoleculeInsertionBondV1::new(1, 2, DocumentBondOrderV1::Single),
            MoleculeInsertionBondV1::new(2, 3, DocumentBondOrderV1::Double),
            MoleculeInsertionBondV1::new(0, 4, DocumentBondOrderV1::Single),
            MoleculeInsertionBondV1::new(3, 5, DocumentBondOrderV1::Single),
        ],
    )
    .expect("valid conjugated E/Z graph");
    let semantics = DocumentStereoSemanticReportV1::new(
        Vec::new(),
        vec![
            DocumentDoubleBondStereoV1::new(0, 4, 2, DocumentDoubleBondConfigurationV1::E)
                .expect("first E/Z descriptor is locally valid"),
            DocumentDoubleBondStereoV1::new(2, 1, 5, DocumentDoubleBondConfigurationV1::Z)
                .expect("second E/Z descriptor is locally valid"),
        ],
    );
    let depictions = DocumentStereoDepictionReportV1::new(
        Vec::new(),
        vec![
            DocumentDoubleBondCarrierMarkDepictionV1::new(
                0,
                1,
                DocumentDoubleBondCarrierMarkV1::Up,
            ),
            DocumentDoubleBondCarrierMarkDepictionV1::new(
                2,
                1,
                DocumentDoubleBondCarrierMarkV1::Down,
            ),
        ],
    );

    let request = MoleculeInsertionRequestV1::with_stereo_reports(
        molecule,
        Some(semantics),
        Some(depictions),
    )
    .expect("one carrier may depict two distinct E/Z descriptors");

    let marks = request
        .stereo_depictions()
        .expect("drawing facts remain admitted")
        .double_bond_carrier_marks();
    assert_eq!(marks.len(), 2);
    assert_eq!(marks[0].double_bond_index(), 0);
    assert_eq!(marks[0].carrier_bond_index(), 1);
    assert_eq!(marks[1].double_bond_index(), 2);
    assert_eq!(marks[1].carrier_bond_index(), 1);
}

#[test]
fn generic_ez_stereo_round_trip_preserves_typed_semantics_and_depiction() {
    let molecule = MoleculeInsertionV1::new(
        vec![
            atom("C", 0.0),
            atom("C", 20.0),
            atom("F", -20.0),
            atom("Cl", 40.0),
        ],
        vec![
            MoleculeInsertionBondV1::new(0, 1, DocumentBondOrderV1::Double),
            MoleculeInsertionBondV1::new(0, 2, DocumentBondOrderV1::Single),
            MoleculeInsertionBondV1::new(1, 3, DocumentBondOrderV1::Single),
        ],
    )
    .expect("valid E/Z graph");
    let semantics = DocumentStereoSemanticReportV1::new(
        Vec::new(),
        vec![
            DocumentDoubleBondStereoV1::new(0, 2, 3, DocumentDoubleBondConfigurationV1::E)
                .expect("valid E/Z descriptor"),
        ],
    );
    let depictions = DocumentStereoDepictionReportV1::new(
        Vec::new(),
        vec![DocumentDoubleBondCarrierMarkDepictionV1::new(
            0,
            1,
            DocumentDoubleBondCarrierMarkV1::Up,
        )],
    );
    let request = MoleculeInsertionRequestV1::with_stereo_reports(
        molecule,
        Some(semantics.clone()),
        Some(depictions.clone()),
    )
    .expect("matching E/Z reports admit");
    let mut session = DocumentSession::load(SOURCE).expect("source loads");
    session
        .apply_document_operation_v1(
            0,
            SessionOperation::V1(SessionOperationV1::InsertMoleculeV1(request)),
        )
        .expect("one generic operation commits");
    let saved = session.snapshot().expect("saved snapshot");
    let reopened = DocumentSession::load(saved.cdml()).expect("canonical CDML reopens");
    let observation = reopened.observe(0).expect("reopened document observes");
    let molecule_id = observation.projection().molecules()[0]
        .id()
        .expect("inserted molecule has one durable ID");
    let typed = TypedDocument::parse(saved.cdml()).expect("saved CDML types");
    assert_eq!(
        typed
            .molecule_stereo_semantics_v1(molecule_id)
            .expect("typed semantic observation"),
        Some(semantics)
    );
    assert_eq!(
        observation
            .molecule_stereo_depictions_v1(molecule_id)
            .expect("snapshot-bound typed depiction observation"),
        Some(depictions)
    );
    let render = reopened
        .observe_render_v1(0)
        .expect("persisted E/Z depiction resolves through the normal render observation");
    let plan = render.resolved().molecule_plans()[0].plan();
    let Some(RenderOp::DoubleBondCarrierMark(mark)) = plan
        .batches()
        .iter()
        .flat_map(|batch| batch.operations())
        .find(|operation| matches!(operation, RenderOp::DoubleBondCarrierMark(_)))
    else {
        panic!("persisted E/Z carrier mark emits its dedicated render operation");
    };
    assert_eq!(mark.direction(), DoubleBondCarrierMarkDirectionV1::Up);
    assert!(
        plan.batches()
            .iter()
            .any(|batch| batch.target().record_id() == mark.central_double_bond())
    );
}

#[test]
fn cdml_refuses_ez_semantics_without_a_carrier_mark_depiction() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\">",
        "<atom id=\"a0\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"a1\" name=\"C\"><point x=\"1\" y=\"0\"/></atom>",
        "<atom id=\"a2\" name=\"F\"><point x=\"-1\" y=\"0\"/></atom>",
        "<atom id=\"a3\" name=\"Cl\"><point x=\"2\" y=\"0\"/></atom>",
        "<bond id=\"b0\" type=\"n2\" start=\"a0\" end=\"a1\"/>",
        "<bond id=\"b1\" type=\"n1\" start=\"a0\" end=\"a2\"/>",
        "<bond id=\"b2\" type=\"n1\" start=\"a1\" end=\"a3\"/>",
        "<stereoSemantics><doubleBond bondIndex=\"0\" startLigand=\"2\" endLigand=\"3\" configuration=\"E\"/></stereoSemantics>",
        "</molecule></cdml>",
    );

    assert!(TypedDocument::parse(source).is_err());
}

#[test]
fn generic_stereo_request_refuses_non_ligand_ez_carrier_depiction() {
    let molecule = MoleculeInsertionV1::new(
        vec![
            atom("C", 0.0),
            atom("C", 20.0),
            atom("F", -20.0),
            atom("Cl", 40.0),
            atom("Br", 60.0),
        ],
        vec![
            MoleculeInsertionBondV1::new(0, 1, DocumentBondOrderV1::Double),
            MoleculeInsertionBondV1::new(0, 4, DocumentBondOrderV1::Single),
            MoleculeInsertionBondV1::new(0, 2, DocumentBondOrderV1::Single),
            MoleculeInsertionBondV1::new(1, 3, DocumentBondOrderV1::Single),
        ],
    )
    .expect("valid graph with an unrelated carrier");
    let semantics = DocumentStereoSemanticReportV1::new(
        Vec::new(),
        vec![
            DocumentDoubleBondStereoV1::new(0, 2, 3, DocumentDoubleBondConfigurationV1::Z)
                .expect("valid E/Z descriptor"),
        ],
    );
    let depictions = DocumentStereoDepictionReportV1::new(
        Vec::new(),
        vec![DocumentDoubleBondCarrierMarkDepictionV1::new(
            0,
            1,
            DocumentDoubleBondCarrierMarkV1::Down,
        )],
    );

    assert!(
        MoleculeInsertionRequestV1::with_stereo_reports(
            molecule,
            Some(semantics),
            Some(depictions),
        )
        .is_err()
    );
}

#[test]
fn nonadjacent_tetrahedral_ligand_is_refused_before_document_load() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\"><molecule id=\"m\">",
        "<atom id=\"a0\" name=\"C\" explicit_hydrogens=\"1\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"a1\" name=\"C\"><point x=\"1\" y=\"0\"/></atom>",
        "<atom id=\"a2\" name=\"C\"><point x=\"2\" y=\"0\"/></atom>",
        "<atom id=\"a3\" name=\"C\"><point x=\"3\" y=\"0\"/></atom>",
        "<bond id=\"b01\" type=\"n1\" start=\"a0\" end=\"a1\"/>",
        "<bond id=\"b02\" type=\"n1\" start=\"a0\" end=\"a2\"/>",
        "<stereoSemantics><tetrahedral center=\"0\" ligands=\"1,2,3,H\" ",
        "parity=\"clockwise\"/></stereoSemantics></molecule></cdml>"
    );
    let result = DocumentSession::load(source);
    assert!(
        matches!(
            result,
            Err(super::DocumentSessionError::Load(
                super::TypedDocumentError::InvalidStereoSemantics
            ))
        ),
        "unexpected malformed-CDML result: {result:?}"
    );
}

#[test]
fn double_bond_endpoint_ligand_is_refused_before_document_load() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\"><molecule id=\"m\">",
        "<atom id=\"a0\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"a1\" name=\"C\"><point x=\"1\" y=\"0\"/></atom>",
        "<atom id=\"a2\" name=\"F\"><point x=\"-1\" y=\"0\"/></atom>",
        "<atom id=\"a3\" name=\"Cl\"><point x=\"2\" y=\"0\"/></atom>",
        "<bond id=\"b01\" type=\"n2\" start=\"a0\" end=\"a1\"/>",
        "<bond id=\"b02\" type=\"n1\" start=\"a0\" end=\"a2\"/>",
        "<bond id=\"b13\" type=\"n1\" start=\"a1\" end=\"a3\"/>",
        "<stereoSemantics><doubleBond bondIndex=\"0\" startLigand=\"0\" endLigand=\"3\" ",
        "configuration=\"E\"/></stereoSemantics></molecule></cdml>"
    );
    let result = DocumentSession::load(source);
    assert!(matches!(
        result,
        Err(super::DocumentSessionError::Load(
            super::TypedDocumentError::InvalidStereoSemantics
        ))
    ));
}
