//! Bounded strict peptide-template request planning.
//!
//! This owner validates the native compatibility profile and prepares its
//! deterministic SMILES request.  It intentionally does not load a chemistry
//! library, mutate a document, or choose a placement.

use ferrum_chemistry::NATIVE_SMILES_MAX_INPUT_BYTES;
use thiserror::Error;

use super::{
    LegacyPeptideTemplateSmilesErrorV1, PeptideSyntaxError, ResidueCode,
    build_legacy_peptide_template_smiles_v1, parse_one_letter_sequence,
};

/// Named native insertion profile; distinct from the pure full-alphabet profile.
pub const NATIVE_PEPTIDE_TEMPLATE_INSERTION_PROFILE_V1: &str =
    "ferrum-native-peptide-template-insertion-v1";
/// Residues admitted by the native V1 insertion route.
pub const NATIVE_PEPTIDE_TEMPLATE_INSERTION_SUPPORTED_ALPHABET_V1: &str = "ACDEFGIKLMNQRSTVY";
/// Fixed bytes outside the profile's worst-case per-residue expansion.
pub const NATIVE_PEPTIDE_TEMPLATE_INSERTION_FIXED_OUTPUT_BYTES_V1: usize = 9;
/// Worst added bytes for one native-profile residue after the first.
pub const NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_ADDITIONAL_RESIDUE_BYTES_V1: usize = 31;
/// Maximum submitted UTF-8 bytes for interactive V1 template text.
pub const NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1: usize =
    (NATIVE_SMILES_MAX_INPUT_BYTES - NATIVE_PEPTIDE_TEMPLATE_INSERTION_FIXED_OUTPUT_BYTES_V1)
        / NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_ADDITIONAL_RESIDUE_BYTES_V1;

/// Fully preflighted strict template request, ready for a caller-owned engine.
#[derive(Debug, Eq, PartialEq)]
pub struct SupportedPeptideTemplateRequestV1 {
    smiles: String,
}

impl SupportedPeptideTemplateRequestV1 {
    /// Return the bounded compatibility-template SMILES for native composition.
    #[must_use]
    pub fn smiles(&self) -> &str {
        &self.smiles
    }
}

/// Preflight exact interactive text before any native library lookup or load.
pub fn compile_supported_peptide_template_request_v1(
    sequence_text: &str,
) -> Result<SupportedPeptideTemplateRequestV1, PeptideTemplateInsertionErrorV1> {
    if sequence_text.len() > NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1 {
        return Err(PeptideTemplateInsertionErrorV1::ResourceAdmission {
            submitted_bytes: sequence_text.len(),
            max_submitted_bytes: NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1,
        });
    }
    let sequence = parse_one_letter_sequence(sequence_text)
        .map_err(|error| map_syntax_error(error, sequence_text.len()))?;
    for (offset, residue) in sequence.residues().iter().copied().enumerate() {
        if !native_profile_supports(residue) {
            return Err(PeptideTemplateInsertionErrorV1::NativeProfile {
                position: offset + 1,
                residue,
                profile: NATIVE_PEPTIDE_TEMPLATE_INSERTION_PROFILE_V1,
                supported_alphabet: NATIVE_PEPTIDE_TEMPLATE_INSERTION_SUPPORTED_ALPHABET_V1,
            });
        }
    }
    let template = build_legacy_peptide_template_smiles_v1(&sequence)
        .map_err(|error| map_template_error(error, sequence_text.len()))?;
    if template.smiles().len() > NATIVE_SMILES_MAX_INPUT_BYTES {
        return Err(PeptideTemplateInsertionErrorV1::ResourceAdmission {
            submitted_bytes: sequence_text.len(),
            max_submitted_bytes: NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1,
        });
    }
    Ok(SupportedPeptideTemplateRequestV1 {
        smiles: template.into_smiles(),
    })
}

/// Failure owned by the bounded strict peptide-template operation.
#[derive(Debug, Error)]
pub enum PeptideTemplateInsertionErrorV1 {
    /// Strict syntax was empty or contained a first invalid Unicode scalar.
    #[error(transparent)]
    Syntax(#[from] PeptideSyntaxError),
    /// The strict standard sequence contains a residue outside the native profile.
    #[error(
        "residue {residue} at position {position} is unsupported by native peptide-template \
         profile {profile}; supported alphabet is {supported_alphabet}"
    )]
    NativeProfile {
        /// One-based N-to-C position.
        position: usize,
        /// The standard residue excluded by this native profile.
        residue: ResidueCode,
        /// Stable native insertion profile identity.
        profile: &'static str,
        /// Exact alphabet offered by this native insertion profile.
        supported_alphabet: &'static str,
    },
    /// The pure legacy template rejected an unexpected profile member.
    #[error(transparent)]
    UnsupportedProfile(#[from] LegacyPeptideTemplateSmilesErrorV1),
    /// Text or compatible worst-case expansion exceeds this operation's budget.
    #[error(
        "supported peptide-template input has {submitted_bytes} bytes, above the \
         {max_submitted_bytes}-byte V1 admission budget"
    )]
    ResourceAdmission {
        submitted_bytes: usize,
        max_submitted_bytes: usize,
    },
}

fn map_syntax_error(
    error: PeptideSyntaxError,
    submitted_bytes: usize,
) -> PeptideTemplateInsertionErrorV1 {
    match error {
        PeptideSyntaxError::AllocationFailed => {
            PeptideTemplateInsertionErrorV1::ResourceAdmission {
                submitted_bytes,
                max_submitted_bytes: NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1,
            }
        }
        other => PeptideTemplateInsertionErrorV1::Syntax(other),
    }
}

fn map_template_error(
    error: LegacyPeptideTemplateSmilesErrorV1,
    submitted_bytes: usize,
) -> PeptideTemplateInsertionErrorV1 {
    match error {
        LegacyPeptideTemplateSmilesErrorV1::AllocationFailed
        | LegacyPeptideTemplateSmilesErrorV1::OutputSizeOverflow => {
            PeptideTemplateInsertionErrorV1::ResourceAdmission {
                submitted_bytes,
                max_submitted_bytes: NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1,
            }
        }
        other => PeptideTemplateInsertionErrorV1::UnsupportedProfile(other),
    }
}

const fn native_profile_supports(residue: ResidueCode) -> bool {
    matches!(
        residue,
        ResidueCode::Alanine
            | ResidueCode::Cysteine
            | ResidueCode::AsparticAcid
            | ResidueCode::GlutamicAcid
            | ResidueCode::Phenylalanine
            | ResidueCode::Glycine
            | ResidueCode::Isoleucine
            | ResidueCode::Lysine
            | ResidueCode::Leucine
            | ResidueCode::Methionine
            | ResidueCode::Asparagine
            | ResidueCode::Glutamine
            | ResidueCode::Arginine
            | ResidueCode::Serine
            | ResidueCode::Threonine
            | ResidueCode::Valine
            | ResidueCode::Tyrosine
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_first_unicode_scalar_failure() {
        assert!(matches!(
            compile_supported_peptide_template_request_v1("Aé"),
            Err(PeptideTemplateInsertionErrorV1::Syntax(
                PeptideSyntaxError::UnsupportedResidue {
                    position: 2,
                    found: 'é',
                    ..
                }
            ))
        ));
    }

    #[test]
    fn enforces_native_profile_and_derived_byte_limit() {
        assert!(matches!(
            compile_supported_peptide_template_request_v1("AH"),
            Err(PeptideTemplateInsertionErrorV1::NativeProfile { position: 2, .. })
        ));
        let accepted = "R".repeat(NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1);
        assert!(compile_supported_peptide_template_request_v1(&accepted).is_ok());
        let rejected = "R".repeat(NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1 + 1);
        assert!(matches!(
            compile_supported_peptide_template_request_v1(&rejected),
            Err(PeptideTemplateInsertionErrorV1::ResourceAdmission { .. })
        ));
    }
}
