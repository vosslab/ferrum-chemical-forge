use crate::peptide::{
    LEGACY_PEPTIDE_TEMPLATE_SMILES_PROFILE_V1, LEGACY_PEPTIDE_TEMPLATE_SMILES_SCHEMA_V1,
    LEGACY_PEPTIDE_TEMPLATE_SMILES_SUPPORTED_ALPHABET_V1, LegacyPeptideTemplateSmilesErrorV1,
    PeptideSequence, PeptideSyntaxError, ResidueCode, build_legacy_peptide_template_smiles_v1,
};

#[test]
fn builds_the_oasa_compatible_profile_for_a_named_multi_residue_sequence() {
    let sequence = PeptideSequence::parse("ANKLE").expect("strict peptide sequence");
    let receipt = build_legacy_peptide_template_smiles_v1(&sequence).expect("supported template");

    assert_eq!(receipt.schema(), LEGACY_PEPTIDE_TEMPLATE_SMILES_SCHEMA_V1);
    assert_eq!(receipt.profile(), LEGACY_PEPTIDE_TEMPLATE_SMILES_PROFILE_V1);
    assert_eq!(receipt.sequence(), &sequence);
    assert_eq!(
        receipt.smiles(),
        concat!(
            "[NH3+][C@@H](C)(C(=O)N[C@@H](CC(=O)N)(C(=O)N[C@@H](CCCC[NH3+])",
            "(C(=O)N[C@@H](CC(C)C)(C(=O)N[C@@H](CCC(=O)[O-])(C(=O)[O-])))))"
        )
    );
}

#[test]
fn preserves_the_distinct_histidine_side_chain_profile() {
    let sequence = PeptideSequence::parse("H").expect("strict peptide sequence");
    let receipt = build_legacy_peptide_template_smiles_v1(&sequence).expect("histidine template");

    assert_eq!(receipt.smiles(), "[NH3+][C@@H](CC1=C[NH]C=N1)(C(=O)[O-])");
    assert_eq!(
        receipt.supported_alphabet(),
        LEGACY_PEPTIDE_TEMPLATE_SMILES_SUPPORTED_ALPHABET_V1
    );
}

#[test]
fn rejects_proline_with_its_typed_one_based_position() {
    let sequence = PeptideSequence::parse("APD").expect("strict peptide sequence");

    assert_eq!(
        build_legacy_peptide_template_smiles_v1(&sequence)
            .expect_err("proline cannot use the generic legacy template"),
        LegacyPeptideTemplateSmilesErrorV1::UnsupportedTemplateResidue {
            position: 2,
            residue: ResidueCode::Proline,
        }
    );
}

#[test]
fn strict_parser_failure_is_not_reclassified_as_a_template_profile_error() {
    assert_eq!(
        PeptideSequence::parse("A\u{03b2}D").expect_err("invalid syntax must fail at parsing"),
        PeptideSyntaxError::UnsupportedResidue {
            position: 2,
            found: '\u{03b2}',
            supported_alphabet: ResidueCode::SUPPORTED_ONE_LETTER_ALPHABET,
        }
    );
}
