use crate::peptide::{
    PeptideSequence, ResidueCode,
    structure_plan_v1::{
        FerrumPeptideProfileV1, PeptideAtomSiteV1, PeptideAtomStereochemistryV1,
        PeptideBondOrderV1, PeptideBondRoleV1, PeptideFormalChargeV1, PeptideStructurePlanErrorV1,
        build_peptide_structure_plan_v1,
    },
};

#[test]
fn strict_sequence_refusal_precedes_native_graph_planning() {
    assert!(PeptideSequence::parse("Aa").is_err());
}

#[test]
fn single_residue_plan_owns_zwitterionic_termini() {
    let sequence = PeptideSequence::from_residues(vec![ResidueCode::Alanine]).expect("sequence");
    let plan = build_peptide_structure_plan_v1(
        &sequence,
        FerrumPeptideProfileV1::Native17ZwitterionicTermini,
    )
    .expect("native profile");
    assert_eq!(
        plan.profile(),
        FerrumPeptideProfileV1::Native17ZwitterionicTermini
    );
    assert!(
        plan.atoms()
            .iter()
            .any(|atom| atom.id().site() == PeptideAtomSiteV1::AminoNitrogen
                && atom.formal_charge() == PeptideFormalChargeV1::PositiveOne)
    );
    assert!(plan.atoms().iter().any(|atom| atom.id().site()
        == PeptideAtomSiteV1::CarboxylateOxygen
        && atom.formal_charge() == PeptideFormalChargeV1::NegativeOne));
}

#[test]
fn dipeptide_plan_preserves_n_to_c_ownership_and_peptide_link() {
    let sequence =
        PeptideSequence::from_residues(vec![ResidueCode::Alanine, ResidueCode::Cysteine])
            .expect("sequence");
    let plan = build_peptide_structure_plan_v1(
        &sequence,
        FerrumPeptideProfileV1::Native17ZwitterionicTermini,
    )
    .expect("native profile");
    let link = plan
        .bonds()
        .iter()
        .find(|bond| bond.role() == PeptideBondRoleV1::PeptideLink)
        .expect("peptide link");
    assert_eq!(link.order(), PeptideBondOrderV1::Single);
    assert_eq!(link.start().residue().one_based(), 1);
    assert_eq!(link.start().site(), PeptideAtomSiteV1::CarbonylCarbon);
    assert_eq!(link.end().residue().one_based(), 2);
    assert_eq!(link.end().site(), PeptideAtomSiteV1::AminoNitrogen);
}

#[test]
fn threonine_plan_owns_alpha_and_side_chain_stereo_facts() {
    let sequence = PeptideSequence::from_residues(vec![ResidueCode::Threonine]).expect("sequence");
    let plan = build_peptide_structure_plan_v1(
        &sequence,
        FerrumPeptideProfileV1::Native17ZwitterionicTermini,
    )
    .expect("native profile");
    assert!(
        plan.atoms()
            .iter()
            .any(|atom| atom.id().site() == PeptideAtomSiteV1::AlphaCarbon
                && atom.stereochemistry() == PeptideAtomStereochemistryV1::TetrahedralS)
    );
    assert!(
        plan.atoms()
            .iter()
            .any(|atom| atom.id().site() == PeptideAtomSiteV1::SideChain(1)
                && atom.stereochemistry() == PeptideAtomStereochemistryV1::TetrahedralR)
    );
}

#[test]
fn profile_refuses_an_unimplemented_residue_without_fallback() {
    let sequence = PeptideSequence::from_residues(vec![ResidueCode::Histidine]).expect("sequence");
    assert_eq!(
        build_peptide_structure_plan_v1(
            &sequence,
            FerrumPeptideProfileV1::Native17ZwitterionicTermini
        ),
        Err(PeptideStructurePlanErrorV1::UnsupportedResidue {
            position: 1,
            residue: ResidueCode::Histidine,
            profile: "ferrum-native-peptide-structure-v1"
        })
    );
}
