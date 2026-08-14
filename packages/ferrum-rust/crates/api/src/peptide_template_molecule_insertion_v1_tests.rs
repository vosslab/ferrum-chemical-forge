use ferrum_chemistry::{
    AtomicNumber, BondOrder, ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolAtom,
    MolBond, MolGraph, NativeChemEngine, Point2 as ChemistryPoint2, SmilesMolecule,
};
use ferrum_geometry::{MoleculePlacementV1, Point2};

use super::{
    NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1,
    NATIVE_PEPTIDE_TEMPLATE_INSERTION_PROFILE_V1,
    NATIVE_PEPTIDE_TEMPLATE_INSERTION_SUPPORTED_ALPHABET_V1, PeptideTemplateInsertionErrorV1,
    build_native_template_insertion_with_engine, compile_supported_peptide_template_request_v1,
};

#[test]
fn strict_preflight_preserves_first_unicode_scalar_failure() {
    assert!(matches!(
        compile_supported_peptide_template_request_v1("Aé"),
        Err(PeptideTemplateInsertionErrorV1::Syntax(
            ferrum_domain::PeptideSyntaxError::UnsupportedResidue {
                position: 2,
                found: 'é',
                ..
            }
        ))
    ));
}

#[test]
fn public_template_lowering_requires_the_closed_native_engine_boundary() {
    let _: fn(
        &NativeChemEngine,
        &super::SupportedPeptideTemplateRequestV1,
        MoleculePlacementV1,
    ) -> Result<_, _> = super::build_supported_peptide_template_molecule_insertion_v1;
}

struct FixedEngine(SmilesMolecule);

impl ChemEngine for FixedEngine {
    fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
        Ok(self.0.clone())
    }

    fn generate_2d_coordinates(&self, _molecule: &MolGraph) -> Result<Coordinates, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "unused",
        })
    }

    fn kekulize(
        &self,
        _molecule: &MolGraph,
        _options: KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "unused",
        })
    }
}

#[test]
fn compiled_template_builds_a_frozen_insertion_at_the_requested_placement() {
    let atom = MolAtom::new(
        AtomicNumber::from_symbol("C").expect("carbon"),
        Some(0),
        None,
        Some(0),
        false,
    )
    .expect("valid atom");
    let graph = MolGraph::new(
        vec![atom.clone(), atom],
        vec![MolBond::new(0, 1, BondOrder::Single, false)],
        Some(Coordinates::new(vec![
            ChemistryPoint2::new(0.0, 0.0).expect("finite"),
            ChemistryPoint2::new(2.0, 0.0).expect("finite"),
        ])),
    )
    .expect("valid graph");
    let engine = FixedEngine(SmilesMolecule::new("CC", graph).expect("valid result"));
    let request = compile_supported_peptide_template_request_v1("ANKLE").expect("supported");
    let placement =
        MoleculePlacementV1::new(40.0, Point2::new(100.0, 200.0).expect("finite anchor"))
            .expect("valid placement");
    let insertion = build_native_template_insertion_with_engine(&engine, &request, placement)
        .expect("complete graph is representable");
    assert_eq!(insertion.atoms()[0].position().x(), 80.0);
    assert_eq!(insertion.atoms()[1].position().x(), 120.0);
}

#[test]
fn strict_preflight_rejects_empty_lowercase_space_and_native_profile_residues() {
    for input in ["", "a", "A A"] {
        assert!(matches!(
            compile_supported_peptide_template_request_v1(input),
            Err(PeptideTemplateInsertionErrorV1::Syntax(_))
        ));
    }
    for (input, position, residue) in [("AH", 2, 'H'), ("AP", 2, 'P'), ("AW", 2, 'W')] {
        assert!(matches!(
            compile_supported_peptide_template_request_v1(input),
            Err(PeptideTemplateInsertionErrorV1::NativeProfile {
                position: found_position,
                residue: found_residue,
                profile: NATIVE_PEPTIDE_TEMPLATE_INSERTION_PROFILE_V1,
                supported_alphabet: NATIVE_PEPTIDE_TEMPLATE_INSERTION_SUPPORTED_ALPHABET_V1,
            }) if found_position == position && found_residue.one_letter() == residue
        ));
    }
}

#[test]
fn derived_byte_boundary_admits_worst_case_and_rejects_one_more_byte() {
    let accepted = "R".repeat(NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1);
    let request = compile_supported_peptide_template_request_v1(&accepted)
        .expect("worst-case derived boundary must fit the native SMILES envelope");
    assert!(request.smiles().len() <= ferrum_chemistry::NATIVE_SMILES_MAX_INPUT_BYTES);
    let rejected = "R".repeat(NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1 + 1);
    assert!(matches!(
        compile_supported_peptide_template_request_v1(&rejected),
        Err(PeptideTemplateInsertionErrorV1::ResourceAdmission { .. })
    ));
}
