use crate::peptide::{
    PeptideSequence, PeptideSyntaxError, PeptideTerminus, ProtonationIntent, ResidueCode,
    TerminusIntent, parse_one_letter_sequence,
};

#[test]
fn standard_residue_alphabet_round_trips_in_canonical_order() {
    let sequence = parse_one_letter_sequence(ResidueCode::SUPPORTED_ONE_LETTER_ALPHABET)
        .expect("standard alphabet must parse");
    assert_eq!(sequence.len(), 20);
    assert_eq!(sequence.to_one_letter_string(), "ACDEFGHIKLMNPQRSTVWY");
    assert_eq!(sequence.residues()[10], ResidueCode::Methionine);
    assert_eq!(sequence.residues()[12], ResidueCode::Proline);
    assert_eq!(ResidueCode::Proline.three_letter(), "Pro");
}

#[test]
fn parser_reports_the_first_invalid_one_based_position_and_alphabet() {
    let error = parse_one_letter_sequence("ACxZ").expect_err("lowercase code must not normalize");
    assert_eq!(
        error,
        PeptideSyntaxError::UnsupportedResidue {
            position: 3,
            found: 'x',
            supported_alphabet: "ACDEFGHIKLMNPQRSTVWY",
        }
    );
}

#[test]
fn parser_counts_unicode_scalars_when_reporting_invalid_input() {
    let error = parse_one_letter_sequence("ACβD").expect_err("noncanonical character must fail");
    assert_eq!(
        error,
        PeptideSyntaxError::UnsupportedResidue {
            position: 3,
            found: 'β',
            supported_alphabet: ResidueCode::SUPPORTED_ONE_LETTER_ALPHABET,
        }
    );
}

#[test]
fn empty_input_is_not_a_sequence() {
    assert_eq!(
        PeptideSequence::parse("").expect_err("empty input must fail"),
        PeptideSyntaxError::EmptySequence
    );
}

#[test]
fn terminus_intent_remains_explicit_without_assigning_a_structure() {
    let intent = TerminusIntent::new(PeptideTerminus::Amino, ProtonationIntent::Protonated);
    assert_eq!(intent.terminus, PeptideTerminus::Amino);
    assert_eq!(intent.protonation, ProtonationIntent::Protonated);
}
