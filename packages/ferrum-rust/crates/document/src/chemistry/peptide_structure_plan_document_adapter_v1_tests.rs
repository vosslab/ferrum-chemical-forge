use super::*;
use ferrum_chemistry::{Coordinates, KekulizeOptions, Point2 as ChemistryPoint2, SmilesMolecule};
use ferrum_domain::peptide::structure_plan_v1::{
    FerrumPeptideProfileV1, PeptideAtomSiteV1, PeptideBondRoleV1, PeptideStructurePlanErrorV1,
    build_peptide_structure_plan_v1,
};
use ferrum_domain::{PeptideSequence, ResidueCode};
use ferrum_geometry::Point2;

struct PlanCoordinatesEngine;

impl ChemEngine for PlanCoordinatesEngine {
    fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "smiles_to_molecule",
        })
    }

    fn generate_2d_coordinates(&self, molecule: &MolGraph) -> Result<Coordinates, ChemistryError> {
        Ok(Coordinates::new(
            (0..molecule.atoms().len())
                .map(|index| ChemistryPoint2::new(index as f64, 0.0))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| ChemistryError::MalformedNativeResponse {
                    reason: error.to_string(),
                })?,
        ))
    }

    fn kekulize(
        &self,
        molecule: &MolGraph,
        _options: KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        Ok(molecule.clone())
    }
}

fn placement() -> MoleculePlacementV1 {
    MoleculePlacementV1::new(20.0, Point2::new(100.0, 200.0).expect("finite anchor"))
        .expect("valid placement")
}

#[test]
fn plan_adapter_preserves_peptide_link_and_zwitterionic_termini() {
    let sequence =
        PeptideSequence::from_residues(vec![ResidueCode::Alanine, ResidueCode::Cysteine])
            .expect("valid sequence");
    let plan = build_peptide_structure_plan_v1(
        &sequence,
        FerrumPeptideProfileV1::Native17ZwitterionicTermini,
    )
    .expect("supported plan");
    let prepared =
        prepare_peptide_structure_plan_for_document_v1(&PlanCoordinatesEngine, &plan, placement())
            .expect("typed plan prepares");
    let insertion = prepared.molecule_insertion();
    let link = plan
        .bonds()
        .iter()
        .find(|bond| bond.role() == PeptideBondRoleV1::PeptideLink)
        .expect("plan has peptide link");
    let link_start = plan
        .atoms()
        .iter()
        .position(|atom| atom.id() == link.start())
        .expect("link start");
    let link_end = plan
        .atoms()
        .iter()
        .position(|atom| atom.id() == link.end())
        .expect("link end");

    assert!(insertion.bonds().iter().any(|bond| {
        bond.start() == link_start
            && bond.end() == link_end
            && bond.order() == DocumentBondOrderV1::Single
    }));
    assert!(
        insertion
            .atoms()
            .iter()
            .any(|atom| atom.element() == "N" && atom.formal_charge() == Some(1))
    );
    assert!(
        insertion
            .atoms()
            .iter()
            .any(|atom| atom.element() == "O" && atom.formal_charge() == Some(-1))
    );
}

#[test]
fn plan_adapter_retains_threonine_alpha_and_side_chain_stereo_provenance() {
    let sequence =
        PeptideSequence::from_residues(vec![ResidueCode::Threonine]).expect("valid sequence");
    let plan = build_peptide_structure_plan_v1(
        &sequence,
        FerrumPeptideProfileV1::Native17ZwitterionicTermini,
    )
    .expect("supported plan");
    let prepared =
        prepare_peptide_structure_plan_for_document_v1(&PlanCoordinatesEngine, &plan, placement())
            .expect("typed plan prepares");
    let semantics = prepared
        .stereo_semantics()
        .expect("stereo report is retained");
    let alpha = plan
        .atoms()
        .iter()
        .position(|atom| atom.id().site() == PeptideAtomSiteV1::AlphaCarbon)
        .expect("alpha carbon");
    let beta = plan
        .atoms()
        .iter()
        .position(|atom| atom.id().site() == PeptideAtomSiteV1::SideChain(1))
        .expect("beta carbon");

    assert!(semantics.tetrahedral().iter().any(|stereo| {
        stereo.center() == alpha && stereo.parity() == DocumentTetrahedralParityV1::CounterClockwise
    }));
    assert!(semantics.tetrahedral().iter().any(|stereo| {
        stereo.center() == beta && stereo.parity() == DocumentTetrahedralParityV1::Clockwise
    }));
}

#[test]
fn unsupported_profile_residue_refuses_before_document_candidate_exists() {
    let sequence =
        PeptideSequence::from_residues(vec![ResidueCode::Histidine]).expect("valid sequence");

    assert!(matches!(
        build_peptide_structure_plan_v1(
            &sequence,
            FerrumPeptideProfileV1::Native17ZwitterionicTermini,
        ),
        Err(PeptideStructurePlanErrorV1::UnsupportedResidue { .. })
    ));
}
