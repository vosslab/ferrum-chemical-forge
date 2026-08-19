//! Typed peptide-sequence facts and strict one-letter syntax validation.
//!
//! This module deliberately represents sequence and terminus intent without
//! choosing atom names, bonding, coordinates, or a native-codec profile.

mod inspection;
mod legacy_template_smiles_v1;
mod template_request;
mod types;
mod validate;

pub use inspection::{
    PEPTIDE_SEQUENCE_INSPECTION_SCHEMA_V1, PeptideResidueInspectionV1,
    PeptideSequenceInspectionErrorV1, PeptideSequenceInspectionV1, inspect_peptide_sequence_v1,
};
pub use legacy_template_smiles_v1::{
    LEGACY_PEPTIDE_TEMPLATE_SMILES_PROFILE_V1, LEGACY_PEPTIDE_TEMPLATE_SMILES_SCHEMA_V1,
    LEGACY_PEPTIDE_TEMPLATE_SMILES_SUPPORTED_ALPHABET_V1, LegacyPeptideTemplateSmilesErrorV1,
    LegacyPeptideTemplateSmilesV1, build_legacy_peptide_template_smiles_v1,
};
pub use template_request::{
    NATIVE_PEPTIDE_TEMPLATE_INSERTION_FIXED_OUTPUT_BYTES_V1,
    NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_ADDITIONAL_RESIDUE_BYTES_V1,
    NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1,
    NATIVE_PEPTIDE_TEMPLATE_INSERTION_PROFILE_V1,
    NATIVE_PEPTIDE_TEMPLATE_INSERTION_SUPPORTED_ALPHABET_V1, PeptideTemplateInsertionErrorV1,
    SupportedPeptideTemplateRequestV1, compile_supported_peptide_template_request_v1,
};
pub use types::{
    PeptideSequence, PeptideSyntaxError, PeptideTerminus, ProtonationIntent, ResidueCode,
    TerminusIntent,
};
pub use validate::parse_one_letter_sequence;

#[cfg(test)]
mod tests;
