//! Typed peptide-sequence facts and strict one-letter syntax validation.
//!
//! This module deliberately represents sequence and terminus intent without
//! choosing atom names, bonding, coordinates, or a native-codec profile.

mod types;
mod validate;

pub use types::{
    PeptideSequence, PeptideSyntaxError, PeptideTerminus, ProtonationIntent, ResidueCode,
    TerminusIntent,
};
pub use validate::parse_one_letter_sequence;

#[cfg(test)]
mod tests;
