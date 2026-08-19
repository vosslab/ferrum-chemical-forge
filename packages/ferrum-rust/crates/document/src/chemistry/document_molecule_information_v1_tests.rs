use std::cell::RefCell;

use crate::DocumentSession;
use ferrum_chemistry::{
    ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, MoleculeComposition,
    MoleculeCompositionEntry, SmilesMolecule,
};

use super::{
    DOCUMENT_MOLECULE_INFORMATION_SCHEMA_V1, DocumentMoleculeCompositionGraphErrorV1,
    DocumentMoleculeInformationErrorV1, DocumentMoleculeInformationRequestErrorV1,
    DocumentMoleculeInformationRequestV1, execute_prepared_document_molecule_information_v1,
    prepare_document_molecule_information_v1,
};

#[derive(Default)]
struct RecordingCompositionEngine {
    requests: RefCell<Vec<MolGraph>>,
    fail_after: Option<usize>,
}

impl ChemEngine for RecordingCompositionEngine {
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

    fn molecule_composition(
        &self,
        molecule: &MolGraph,
    ) -> Result<MoleculeComposition, ChemistryError> {
        let call_index = self.requests.borrow().len();
        self.requests.borrow_mut().push(molecule.clone());
        if self.fail_after == Some(call_index) {
            return Err(ChemistryError::OperationUnavailable {
                operation: "molecule_composition",
            });
        }
        let mut entries = Vec::new();
        for atom in molecule.atoms() {
            let key = (atom.atomic_number(), atom.isotope());
            if let Some(entry) = entries.iter_mut().find(
                |entry: &&mut (ferrum_chemistry::CompositionElementKey, u64)| {
                    entry.0.atomic_number() == key.0 && entry.0.isotope() == key.1
                },
            ) {
                entry.1 += 1;
            } else {
                entries.push((
                    ferrum_chemistry::CompositionElementKey::new(key.0, key.1),
                    1_u64,
                ));
            }
        }
        let charge = molecule
            .atoms()
            .iter()
            .map(|atom| i64::from(atom.formal_charge().unwrap_or(0)))
            .sum();
        let entries = entries
            .into_iter()
            .map(|(key, count)| {
                let contribution = f64::from(key.atomic_number().get()) * count as f64;
                MoleculeCompositionEntry::new(key, count, contribution).expect("fixture entry")
            })
            .collect();
        let exact_mass = molecule
            .atoms()
            .iter()
            .map(|atom| f64::from(atom.atomic_number().get()))
            .sum();
        MoleculeComposition::from_entries(charge, exact_mass, entries).map_err(|error| {
            ChemistryError::MalformedNativeResponse {
                reason: error.to_string(),
            }
        })
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

fn observation(source: &str) -> crate::SessionDocumentObservationV1 {
    DocumentSession::load(source)
        .expect("source loads")
        .observe(0)
        .expect("source projects")
}

fn request(
    observation: &crate::SessionDocumentObservationV1,
    indices: &[usize],
) -> DocumentMoleculeInformationRequestV1 {
    let ids = indices
        .iter()
        .map(|index| {
            observation.projection().molecules()[*index]
                .id()
                .expect("durable root")
                .clone()
        })
        .collect();
    DocumentMoleculeInformationRequestV1::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        ids,
    )
    .expect("request")
}

#[test]
fn request_refuses_empty_or_duplicate_root_input() {
    assert_eq!(
        DocumentMoleculeInformationRequestV1::new(0, [0; 32], Vec::new()),
        Err(DocumentMoleculeInformationRequestErrorV1::EmptySelection)
    );
    let observation = observation(
        "<cdml version=\"26.07\"><molecule id=\"m1\"><atom id=\"a1\" name=\"C\">\
         <point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    );
    let id = observation.projection().molecules()[0]
        .id()
        .expect("durable")
        .clone();
    assert_eq!(
        DocumentMoleculeInformationRequestV1::new(0, [0; 32], vec![id.clone(), id]),
        Err(DocumentMoleculeInformationRequestErrorV1::DuplicateMolecule)
    );
}

#[test]
fn multi_root_receipt_is_document_ordered_and_combined_without_mutation() {
    let source = concat!(
        "<cdml version=\"26.07\">",
        "<molecule id=\"carbon\" name=\"Carbon\">",
        "<atom id=\"c1\" name=\"C\" charge=\"1\"><point x=\"0\" y=\"0\"/></atom>",
        "</molecule><molecule id=\"oxygen\" name=\"Oxygen\">",
        "<atom id=\"o1\" name=\"O\"><point x=\"2\" y=\"3\"/></atom>",
        "</molecule></cdml>"
    );
    let observation = observation(source);
    let before = observation.clone();
    let prepared =
        prepare_document_molecule_information_v1(&observation, &request(&observation, &[1, 0]))
            .expect("both roots prepare");
    assert_eq!(prepared.record_count(), 2);
    assert_eq!(prepared.source_digest(), observation.snapshot().digest());

    let engine = RecordingCompositionEngine::default();
    let information = execute_prepared_document_molecule_information_v1(&engine, prepared)
        .expect("both roots execute");

    assert_eq!(
        information.schema(),
        DOCUMENT_MOLECULE_INFORMATION_SCHEMA_V1
    );
    assert_eq!(
        information
            .records()
            .iter()
            .map(|record| record.source_facts().source_id())
            .collect::<Vec<_>>(),
        vec!["carbon", "oxygen"]
    );
    assert_eq!(information.records()[0].composition().formula(), "C+");
    assert_eq!(information.records()[1].composition().formula(), "O");
    let combined = information
        .combined_selection()
        .expect("multiple roots combine");
    assert_eq!(combined.formula(), "CO+");
    assert_eq!(combined.net_formal_charge(), 1);
    assert_eq!(combined.element_counts().len(), 2);
    assert_eq!(observation, before);
}

#[test]
fn composition_graph_accepts_closed_drawing_styles_and_aromatic_order() {
    for bond_type in [
        "n1", "w1", "h1", "a1", "b1", "d1", "o1", "s1", "q1", "n2", "n3", "n4",
    ] {
        let source = format!(
            concat!(
                "<cdml version=\"26.07\"><molecule id=\"m1\">",
                "<atom id=\"a1\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
                "<atom id=\"a2\" name=\"C\"><point x=\"1\" y=\"0\"/></atom>",
                "<bond id=\"b1\" start=\"a1\" end=\"a2\" type=\"{}\"/>",
                "</molecule></cdml>"
            ),
            bond_type
        );
        let observation = observation(&source);
        let prepared =
            prepare_document_molecule_information_v1(&observation, &request(&observation, &[0]))
                .unwrap_or_else(|error| panic!("{bond_type} must prepare: {error}"));
        let engine = RecordingCompositionEngine::default();
        execute_prepared_document_molecule_information_v1(&engine, prepared)
            .unwrap_or_else(|error| panic!("{bond_type} must execute: {error}"));
        let graph = engine.requests.borrow();
        if bond_type == "n4" {
            assert!(graph[0].bonds()[0].is_aromatic());
            assert!(graph[0].atoms().iter().all(|atom| atom.is_aromatic()));
        }
    }
}

#[test]
fn unsupported_source_facts_fail_before_engine_execution() {
    for (atom_fact, bond_type) in [(" valency=\"4\"", "n1"), ("", "x1"), ("", "n9")] {
        let source = format!(
            concat!(
                "<cdml version=\"26.07\"><molecule id=\"m1\">",
                "<atom id=\"a1\" name=\"C\"{}><point x=\"0\" y=\"0\"/></atom>",
                "<atom id=\"a2\" name=\"O\"><point x=\"1\" y=\"0\"/></atom>",
                "<bond id=\"b1\" start=\"a1\" end=\"a2\" type=\"{}\"/>",
                "</molecule></cdml>"
            ),
            atom_fact, bond_type
        );
        let observation = observation(&source);
        assert!(matches!(
            prepare_document_molecule_information_v1(&observation, &request(&observation, &[0])),
            Err(DocumentMoleculeInformationErrorV1::Graph(
                DocumentMoleculeCompositionGraphErrorV1::UnsupportedAtomFact { .. }
                    | DocumentMoleculeCompositionGraphErrorV1::UnsupportedBondStyle { .. }
                    | DocumentMoleculeCompositionGraphErrorV1::UnsupportedBondOrder { .. }
            ))
        ));
    }
}

#[test]
fn engine_failure_returns_no_partial_information_receipt() {
    let source = concat!(
        "<cdml version=\"26.07\">",
        "<molecule id=\"m1\"><atom id=\"a1\" name=\"C\">",
        "<point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"m2\"><atom id=\"a2\" name=\"O\">",
        "<point x=\"1\" y=\"0\"/></atom></molecule></cdml>"
    );
    let observation = observation(source);
    let prepared =
        prepare_document_molecule_information_v1(&observation, &request(&observation, &[0, 1]))
            .expect("prepare");
    let engine = RecordingCompositionEngine {
        requests: RefCell::new(Vec::new()),
        fail_after: Some(1),
    };

    assert!(matches!(
        execute_prepared_document_molecule_information_v1(&engine, prepared),
        Err(DocumentMoleculeInformationErrorV1::Chemistry(
            ChemistryError::OperationUnavailable { .. }
        ))
    ));
}
