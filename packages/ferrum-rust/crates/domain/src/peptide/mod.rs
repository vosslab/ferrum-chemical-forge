//! Typed peptide-sequence facts and strict one-letter syntax validation.
//!
//! This module deliberately represents sequence and terminus intent without
//! choosing atom names, bonding, coordinates, or a native-codec profile.

mod legacy_template_smiles_v1;
mod types;
mod validate;

pub use legacy_template_smiles_v1::{
    LEGACY_PEPTIDE_TEMPLATE_SMILES_PROFILE_V1, LEGACY_PEPTIDE_TEMPLATE_SMILES_SCHEMA_V1,
    LEGACY_PEPTIDE_TEMPLATE_SMILES_SUPPORTED_ALPHABET_V1, LegacyPeptideTemplateSmilesErrorV1,
    LegacyPeptideTemplateSmilesV1, build_legacy_peptide_template_smiles_v1,
};
pub use types::{
    PeptideSequence, PeptideSyntaxError, PeptideTerminus, ProtonationIntent, ResidueCode,
    TerminusIntent,
};
pub use validate::parse_one_letter_sequence;

#[cfg(test)]
mod tests;
