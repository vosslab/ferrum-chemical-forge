//! Strict, non-structural inspection of one canonical peptide sequence.
//!
//! This is an in-process API over an already allocated UTF-8 string. Future
//! external ingress owners (such as a CLI, Python, clipboard, or document
//! adapter) must set their own text-resource policy before calling it.

use super::{PeptideSyntaxError, ResidueCode, parse_one_letter_sequence};
use serde::Serialize;
use thiserror::Error;

/// Stable schema identifier for a peptide sequence inspection receipt.
pub const PEPTIDE_SEQUENCE_INSPECTION_SCHEMA_V1: &str = "ferrum-peptide-sequence-inspection-v1";

/// Inspect one strict, canonical uppercase peptide sequence without structure.
///
/// The result is owned and has no terminus, molecular, document, native, or
/// source-normalization policy. Invalid positions are one-based Unicode scalar
/// positions in the submitted string.
pub fn inspect_peptide_sequence_v1(
    sequence: &str,
) -> Result<PeptideSequenceInspectionV1, PeptideSequenceInspectionErrorV1> {
    let sequence = parse_one_letter_sequence(sequence).map_err(map_syntax_error)?;
    let residue_count = u64::try_from(sequence.len()).expect("usize fits u64 on Ferrum targets");
    let residues = sequence
        .residues()
        .iter()
        .enumerate()
        .map(|(offset, residue)| PeptideResidueInspectionV1 {
            position: u64::try_from(offset + 1).expect("usize fits u64 on Ferrum targets"),
            one_letter: residue.one_letter(),
            three_letter: residue.three_letter().to_owned(),
        })
        .collect();

    Ok(PeptideSequenceInspectionV1 {
        schema: PEPTIDE_SEQUENCE_INSPECTION_SCHEMA_V1,
        canonical_one_letter_sequence: sequence.to_one_letter_string(),
        supported_one_letter_alphabet: ResidueCode::SUPPORTED_ONE_LETTER_ALPHABET.to_owned(),
        residue_count,
        residues,
    })
}

/// Owned read-only facts about one accepted peptide sequence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PeptideSequenceInspectionV1 {
    schema: &'static str,
    canonical_one_letter_sequence: String,
    supported_one_letter_alphabet: String,
    residue_count: u64,
    residues: Vec<PeptideResidueInspectionV1>,
}

impl PeptideSequenceInspectionV1 {
    /// Return this receipt's stable schema identifier.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Return the accepted canonical one-letter sequence in N-to-C order.
    #[must_use]
    pub fn canonical_one_letter_sequence(&self) -> &str {
        &self.canonical_one_letter_sequence
    }

    /// Return the complete accepted one-letter alphabet.
    #[must_use]
    pub fn supported_one_letter_alphabet(&self) -> &str {
        &self.supported_one_letter_alphabet
    }

    /// Return the number of ordered residue facts.
    #[must_use]
    pub const fn residue_count(&self) -> u64 {
        self.residue_count
    }

    /// Return ordered N-to-C residue facts.
    #[must_use]
    pub fn residues(&self) -> &[PeptideResidueInspectionV1] {
        &self.residues
    }
}

/// One accepted residue in N-to-C sequence order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PeptideResidueInspectionV1 {
    position: u64,
    one_letter: char,
    three_letter: String,
}

impl PeptideResidueInspectionV1 {
    /// Return the one-based N-to-C residue position.
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.position
    }

    /// Return the canonical uppercase one-letter code.
    #[must_use]
    pub const fn one_letter(&self) -> char {
        self.one_letter
    }

    /// Return the canonical three-letter residue code.
    #[must_use]
    pub fn three_letter(&self) -> &str {
        &self.three_letter
    }
}

/// Syntax failures exposed by the peptide inspection contract.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum PeptideSequenceInspectionErrorV1 {
    /// No residue was supplied.
    #[error("peptide sequence must contain at least one residue")]
    EmptySequence,
    /// The first unsupported Unicode scalar in the submitted sequence.
    #[error(
        "unsupported residue {found:?} at position {position}; supported alphabet is \
         {supported_one_letter_alphabet}"
    )]
    UnsupportedResidue {
        /// One-based Unicode scalar position.
        position: u64,
        /// The first unsupported scalar.
        found: char,
        /// The complete accepted alphabet.
        supported_one_letter_alphabet: String,
    },
    /// The bounded strict parser could not reserve its result storage.
    #[error("peptide sequence inspection could not reserve result storage")]
    ResourceAllocation,
}

fn map_syntax_error(error: PeptideSyntaxError) -> PeptideSequenceInspectionErrorV1 {
    match error {
        PeptideSyntaxError::EmptySequence => PeptideSequenceInspectionErrorV1::EmptySequence,
        PeptideSyntaxError::UnsupportedResidue {
            position,
            found,
            supported_alphabet,
        } => PeptideSequenceInspectionErrorV1::UnsupportedResidue {
            position: u64::try_from(position).expect("usize fits u64 on Ferrum targets"),
            found,
            supported_one_letter_alphabet: supported_alphabet.to_owned(),
        },
        PeptideSyntaxError::AllocationFailed => {
            PeptideSequenceInspectionErrorV1::ResourceAllocation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspection_preserves_ordered_residue_facts() {
        let inspection = inspect_peptide_sequence_v1("AP").expect("canonical sequence");
        assert_eq!(inspection.schema(), PEPTIDE_SEQUENCE_INSPECTION_SCHEMA_V1);
        assert_eq!(inspection.canonical_one_letter_sequence(), "AP");
        assert_eq!(inspection.residue_count(), 2);
        assert_eq!(inspection.residues()[1].position(), 2);
        assert_eq!(inspection.residues()[1].three_letter(), "Pro");
    }

    #[test]
    fn inspection_maps_empty_and_first_unicode_failure() {
        assert_eq!(
            inspect_peptide_sequence_v1("").expect_err("empty input"),
            PeptideSequenceInspectionErrorV1::EmptySequence
        );
        assert!(matches!(
            inspect_peptide_sequence_v1("A\u{03b2}P"),
            Err(PeptideSequenceInspectionErrorV1::UnsupportedResidue {
                position: 2,
                found: '\u{03b2}',
                ..
            })
        ));
    }
}
