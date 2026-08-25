//! Typed peptide-sequence facts and strict one-letter syntax validation.
//!
//! This module deliberately represents sequence and terminus intent without
//! choosing atom names, bonding, coordinates, or a native-codec profile.

mod inspection;
/// Closed, immutable peptide molecular semantics consumed by document adapters.
pub mod structure_plan_v1;
mod types;
mod validate;

pub use inspection::{
    PEPTIDE_SEQUENCE_INSPECTION_SCHEMA_V1, PeptideResidueInspectionV1,
    PeptideSequenceInspectionErrorV1, PeptideSequenceInspectionV1, inspect_peptide_sequence_v1,
};
pub use types::{
    PeptideSequence, PeptideSyntaxError, PeptideTerminus, ProtonationIntent, ResidueCode,
    TerminusIntent,
};
pub use validate::parse_one_letter_sequence;

#[cfg(test)]
mod tests;
