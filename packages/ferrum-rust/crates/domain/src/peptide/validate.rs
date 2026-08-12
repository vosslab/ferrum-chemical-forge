//! Strict parser for canonical peptide one-letter syntax.

use crate::peptide::{PeptideSequence, PeptideSyntaxError, ResidueCode};

/// Parse canonical uppercase one-letter peptide syntax.
///
/// Validation reports the first invalid Unicode scalar at a one-based position.
/// It deliberately does not normalize case, omit whitespace, or interpret
/// aliases: a caller must choose a canonical sequence before this boundary.
pub fn parse_one_letter_sequence(input: &str) -> Result<PeptideSequence, PeptideSyntaxError> {
    if input.is_empty() {
        return Err(PeptideSyntaxError::EmptySequence);
    }

    let mut residues = Vec::with_capacity(input.chars().count());
    for (offset, code) in input.chars().enumerate() {
        let residue =
            ResidueCode::from_one_letter(code).ok_or(PeptideSyntaxError::UnsupportedResidue {
                position: offset + 1,
                found: code,
                supported_alphabet: ResidueCode::SUPPORTED_ONE_LETTER_ALPHABET,
            })?;
        residues.push(residue);
    }
    PeptideSequence::from_residues(residues)
}
