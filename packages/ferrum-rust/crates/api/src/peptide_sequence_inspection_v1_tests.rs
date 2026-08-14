use crate::{
    PEPTIDE_SEQUENCE_INSPECTION_SCHEMA_V1, PeptideSequenceInspectionErrorV1,
    inspect_peptide_sequence_v1,
};

#[test]
fn inspection_preserves_proline_and_ordered_residue_facts() {
    let inspection = inspect_peptide_sequence_v1("AP").expect("canonical sequence");

    assert_eq!(inspection.schema(), PEPTIDE_SEQUENCE_INSPECTION_SCHEMA_V1);
    assert_eq!(inspection.canonical_one_letter_sequence(), "AP");
    assert_eq!(
        inspection.supported_one_letter_alphabet(),
        "ACDEFGHIKLMNPQRSTVWY"
    );
    assert_eq!(inspection.residue_count(), 2);
    assert_eq!(inspection.residues()[0].position(), 1);
    assert_eq!(inspection.residues()[0].one_letter(), 'A');
    assert_eq!(inspection.residues()[0].three_letter(), "Ala");
    assert_eq!(inspection.residues()[1].position(), 2);
    assert_eq!(inspection.residues()[1].one_letter(), 'P');
    assert_eq!(inspection.residues()[1].three_letter(), "Pro");
}

#[test]
fn inspection_maps_first_invalid_unicode_scalar_to_public_error() {
    let error = inspect_peptide_sequence_v1("A\u{03b2}P").expect_err("beta is not a residue");

    assert_eq!(
        error,
        PeptideSequenceInspectionErrorV1::UnsupportedResidue {
            position: 2,
            found: '\u{03b2}',
            supported_one_letter_alphabet: "ACDEFGHIKLMNPQRSTVWY".to_owned(),
        }
    );
}

#[test]
fn inspection_maps_empty_input_to_public_error() {
    assert_eq!(
        inspect_peptide_sequence_v1("").expect_err("empty input is not a peptide"),
        PeptideSequenceInspectionErrorV1::EmptySequence
    );
}
