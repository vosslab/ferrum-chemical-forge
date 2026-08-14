//! Peptide domain values with explicit, non-structural meaning.

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The 20 genetically encoded proteinogenic amino-acid residue codes.
///
/// This is a residue catalog, not an assertion about a particular molecular
/// representation or protonation microstate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ResidueCode {
    /// Alanine (`A`, `Ala`).
    Alanine,
    /// Arginine (`R`, `Arg`).
    Arginine,
    /// Asparagine (`N`, `Asn`).
    Asparagine,
    /// Aspartic acid (`D`, `Asp`).
    AsparticAcid,
    /// Cysteine (`C`, `Cys`).
    Cysteine,
    /// Glutamic acid (`E`, `Glu`).
    GlutamicAcid,
    /// Glutamine (`Q`, `Gln`).
    Glutamine,
    /// Glycine (`G`, `Gly`).
    Glycine,
    /// Histidine (`H`, `His`).
    Histidine,
    /// Isoleucine (`I`, `Ile`).
    Isoleucine,
    /// Leucine (`L`, `Leu`).
    Leucine,
    /// Lysine (`K`, `Lys`).
    Lysine,
    /// Methionine (`M`, `Met`).
    Methionine,
    /// Phenylalanine (`F`, `Phe`).
    Phenylalanine,
    /// Proline (`P`, `Pro`).
    Proline,
    /// Serine (`S`, `Ser`).
    Serine,
    /// Threonine (`T`, `Thr`).
    Threonine,
    /// Tryptophan (`W`, `Trp`).
    Tryptophan,
    /// Tyrosine (`Y`, `Tyr`).
    Tyrosine,
    /// Valine (`V`, `Val`).
    Valine,
}

impl ResidueCode {
    /// The deterministic accepted one-letter alphabet in canonical order.
    pub const SUPPORTED_ONE_LETTER_ALPHABET: &str = "ACDEFGHIKLMNPQRSTVWY";

    /// Return this residue's canonical uppercase one-letter code.
    #[must_use]
    pub const fn one_letter(self) -> char {
        match self {
            Self::Alanine => 'A',
            Self::Arginine => 'R',
            Self::Asparagine => 'N',
            Self::AsparticAcid => 'D',
            Self::Cysteine => 'C',
            Self::GlutamicAcid => 'E',
            Self::Glutamine => 'Q',
            Self::Glycine => 'G',
            Self::Histidine => 'H',
            Self::Isoleucine => 'I',
            Self::Leucine => 'L',
            Self::Lysine => 'K',
            Self::Methionine => 'M',
            Self::Phenylalanine => 'F',
            Self::Proline => 'P',
            Self::Serine => 'S',
            Self::Threonine => 'T',
            Self::Tryptophan => 'W',
            Self::Tyrosine => 'Y',
            Self::Valine => 'V',
        }
    }

    /// Return this residue's canonical three-letter code.
    #[must_use]
    pub const fn three_letter(self) -> &'static str {
        match self {
            Self::Alanine => "Ala",
            Self::Arginine => "Arg",
            Self::Asparagine => "Asn",
            Self::AsparticAcid => "Asp",
            Self::Cysteine => "Cys",
            Self::GlutamicAcid => "Glu",
            Self::Glutamine => "Gln",
            Self::Glycine => "Gly",
            Self::Histidine => "His",
            Self::Isoleucine => "Ile",
            Self::Leucine => "Leu",
            Self::Lysine => "Lys",
            Self::Methionine => "Met",
            Self::Phenylalanine => "Phe",
            Self::Proline => "Pro",
            Self::Serine => "Ser",
            Self::Threonine => "Thr",
            Self::Tryptophan => "Trp",
            Self::Tyrosine => "Tyr",
            Self::Valine => "Val",
        }
    }

    /// Map one canonical uppercase one-letter code to its residue.
    #[must_use]
    pub const fn from_one_letter(code: char) -> Option<Self> {
        match code {
            'A' => Some(Self::Alanine),
            'R' => Some(Self::Arginine),
            'N' => Some(Self::Asparagine),
            'D' => Some(Self::AsparticAcid),
            'C' => Some(Self::Cysteine),
            'E' => Some(Self::GlutamicAcid),
            'Q' => Some(Self::Glutamine),
            'G' => Some(Self::Glycine),
            'H' => Some(Self::Histidine),
            'I' => Some(Self::Isoleucine),
            'L' => Some(Self::Leucine),
            'K' => Some(Self::Lysine),
            'M' => Some(Self::Methionine),
            'F' => Some(Self::Phenylalanine),
            'P' => Some(Self::Proline),
            'S' => Some(Self::Serine),
            'T' => Some(Self::Threonine),
            'W' => Some(Self::Tryptophan),
            'Y' => Some(Self::Tyrosine),
            'V' => Some(Self::Valine),
            _ => None,
        }
    }
}

impl fmt::Display for ResidueCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.one_letter())
    }
}

/// A non-empty, ordered peptide residue sequence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PeptideSequence {
    residues: Vec<ResidueCode>,
}

#[derive(Deserialize)]
struct PeptideSequenceWire {
    residues: Vec<ResidueCode>,
}

impl<'de> Deserialize<'de> for PeptideSequence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = PeptideSequenceWire::deserialize(deserializer)?;
        Self::from_residues(wire.residues).map_err(serde::de::Error::custom)
    }
}

impl PeptideSequence {
    /// Construct a non-empty sequence from already typed standard residues.
    pub fn from_residues(residues: Vec<ResidueCode>) -> Result<Self, PeptideSyntaxError> {
        if residues.is_empty() {
            return Err(PeptideSyntaxError::EmptySequence);
        }
        Ok(Self { residues })
    }

    /// Parse strict, canonical uppercase one-letter peptide syntax.
    pub fn parse(input: &str) -> Result<Self, PeptideSyntaxError> {
        crate::peptide::validate::parse_one_letter_sequence(input)
    }

    /// Fallibly duplicate this sequence for a receipt that requires ownership.
    pub(crate) fn try_clone(&self) -> Result<Self, PeptideSyntaxError> {
        let mut residues = Vec::new();
        residues
            .try_reserve(self.residues.len())
            .map_err(|_| PeptideSyntaxError::AllocationFailed)?;
        residues.extend_from_slice(&self.residues);
        Ok(Self { residues })
    }

    /// Return the sequence in N-to-C order.
    #[must_use]
    pub fn residues(&self) -> &[ResidueCode] {
        &self.residues
    }

    /// Return the number of residues.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.residues.len()
    }

    /// Return whether this sequence has no residues.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.residues.is_empty()
    }

    /// Render canonical one-letter syntax without structural interpretation.
    #[must_use]
    pub fn to_one_letter_string(&self) -> String {
        self.residues
            .iter()
            .map(|residue| residue.one_letter())
            .collect()
    }
}

/// One end of a peptide chain.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PeptideTerminus {
    /// The amino (N) terminus.
    Amino,
    /// The carboxyl (C) terminus.
    Carboxyl,
}

/// Caller-declared protonation intent, before molecular realization.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProtonationIntent {
    /// No protonation state was requested.
    Unspecified,
    /// The caller requests a protonated terminus.
    Protonated,
    /// The caller requests a deprotonated terminus.
    Deprotonated,
    /// The caller requests a neutral terminus.
    Neutral,
}

/// Explicit terminus and protonation intent for a later molecular planner.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TerminusIntent {
    /// The end of the peptide to which the intent applies.
    pub terminus: PeptideTerminus,
    /// The requested protonation fact, intentionally separate from structure.
    pub protonation: ProtonationIntent,
}

impl TerminusIntent {
    /// Pair a chain end with caller-declared protonation intent.
    #[must_use]
    pub const fn new(terminus: PeptideTerminus, protonation: ProtonationIntent) -> Self {
        Self {
            terminus,
            protonation,
        }
    }
}

/// Strict peptide syntax validation failure.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum PeptideSyntaxError {
    /// Empty input cannot represent a peptide sequence.
    #[error("peptide sequence must contain at least one residue")]
    EmptySequence,
    /// An unsupported character occurred at a one-based scalar position.
    #[error(
        "unsupported residue {found:?} at position {position}; supported alphabet is \
         {supported_alphabet}"
    )]
    UnsupportedResidue {
        /// One-based Unicode scalar position in the submitted sequence.
        position: usize,
        /// The first unsupported character.
        found: char,
        /// The complete accepted alphabet, retained on the error for callers.
        supported_alphabet: &'static str,
    },
    /// The strict parser could not reserve its bounded result storage.
    #[error("peptide sequence validation could not reserve result storage")]
    AllocationFailed,
}
