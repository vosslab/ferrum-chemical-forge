//! Deterministic OASA-compatible peptide-template SMILES profile.
//!
//! This preserves the legacy template grammar used by the former OASA import
//! route. It is neither canonical SMILES nor a general peptide structure
//! model: proline, modified residues, alternative termini, and protonation
//! choices require another explicitly selected profile.

use thiserror::Error;

use crate::peptide::{PeptideSequence, ResidueCode};

/// Schema carried by a [`LegacyPeptideTemplateSmilesV1`] receipt.
pub const LEGACY_PEPTIDE_TEMPLATE_SMILES_SCHEMA_V1: &str =
    "ferrum-legacy-peptide-template-smiles-v1";

/// Fixed structural-template profile reproduced from the OASA compatibility path.
pub const LEGACY_PEPTIDE_TEMPLATE_SMILES_PROFILE_V1: &str = "oasa-compatibility-v1";

/// The one-letter residues representable by this legacy template profile.
pub const LEGACY_PEPTIDE_TEMPLATE_SMILES_SUPPORTED_ALPHABET_V1: &str = "ACDEFGHIKLMNQRSTVWY";

const N_TERMINUS: &str = "[NH3+][C@@H]";
const NEXT_RESIDUE: &str = "(C(=O)N[C@@H]";
const C_TERMINUS: &str = "(C(=O)[O-])";

/// Build the fixed OASA-compatible structural-SMILES template for `sequence`.
///
/// The input is an already accepted, non-empty N-to-C [`PeptideSequence`].
/// This function borrows it and returns a self-contained receipt; it does not
/// parse external text or claim a general molecular representation.
pub fn build_legacy_peptide_template_smiles_v1(
    sequence: &PeptideSequence,
) -> Result<LegacyPeptideTemplateSmilesV1, LegacyPeptideTemplateSmilesErrorV1> {
    let output_len = legacy_peptide_template_smiles_output_bytes_v1(sequence)?;

    let link_count = sequence.len() - 1;
    let mut smiles = String::new();
    smiles
        .try_reserve(output_len)
        .map_err(|_| LegacyPeptideTemplateSmilesErrorV1::AllocationFailed)?;
    smiles.push_str(N_TERMINUS);
    for (index, residue) in sequence.residues().iter().copied().enumerate() {
        let side_chain = side_chain_v1(residue).ok_or(
            LegacyPeptideTemplateSmilesErrorV1::UnsupportedTemplateResidue {
                position: index + 1,
                residue,
            },
        )?;
        smiles.push_str(side_chain);
        if index < link_count {
            smiles.push_str(NEXT_RESIDUE);
        }
    }
    smiles.push_str(C_TERMINUS);
    for _ in 0..link_count {
        smiles.push(')');
    }

    Ok(LegacyPeptideTemplateSmilesV1 {
        sequence: sequence
            .try_clone()
            .map_err(|_| LegacyPeptideTemplateSmilesErrorV1::AllocationFailed)?,
        smiles,
    })
}

/// Calculate this profile's exact SMILES output length without allocating it.
fn legacy_peptide_template_smiles_output_bytes_v1(
    sequence: &PeptideSequence,
) -> Result<usize, LegacyPeptideTemplateSmilesErrorV1> {
    let mut output_len = N_TERMINUS
        .len()
        .checked_add(C_TERMINUS.len())
        .ok_or(LegacyPeptideTemplateSmilesErrorV1::OutputSizeOverflow)?;
    for (index, residue) in sequence.residues().iter().copied().enumerate() {
        let side_chain = side_chain_v1(residue).ok_or(
            LegacyPeptideTemplateSmilesErrorV1::UnsupportedTemplateResidue {
                position: index + 1,
                residue,
            },
        )?;
        output_len = output_len
            .checked_add(side_chain.len())
            .ok_or(LegacyPeptideTemplateSmilesErrorV1::OutputSizeOverflow)?;
    }
    let link_count = sequence.len() - 1;
    let link_len = NEXT_RESIDUE
        .len()
        .checked_add(1)
        .ok_or(LegacyPeptideTemplateSmilesErrorV1::OutputSizeOverflow)?;
    output_len = output_len
        .checked_add(
            link_count
                .checked_mul(link_len)
                .ok_or(LegacyPeptideTemplateSmilesErrorV1::OutputSizeOverflow)?,
        )
        .ok_or(LegacyPeptideTemplateSmilesErrorV1::OutputSizeOverflow)?;
    Ok(output_len)
}

/// Owned facts produced by the fixed legacy peptide-template profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyPeptideTemplateSmilesV1 {
    sequence: PeptideSequence,
    smiles: String,
}

impl LegacyPeptideTemplateSmilesV1 {
    /// Return the stable receipt schema.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        LEGACY_PEPTIDE_TEMPLATE_SMILES_SCHEMA_V1
    }

    /// Return the fixed structural-template profile identifier.
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        LEGACY_PEPTIDE_TEMPLATE_SMILES_PROFILE_V1
    }

    /// Return the accepted input sequence in N-to-C order.
    #[must_use]
    pub const fn sequence(&self) -> &PeptideSequence {
        &self.sequence
    }

    /// Return this profile's deterministic structural-SMILES template.
    #[must_use]
    pub fn smiles(&self) -> &str {
        &self.smiles
    }

    /// Consume this receipt and return its already-reserved template storage.
    #[must_use]
    pub fn into_smiles(self) -> String {
        self.smiles
    }

    /// Return the complete one-letter alphabet supported by this profile.
    #[must_use]
    pub const fn supported_alphabet(&self) -> &'static str {
        LEGACY_PEPTIDE_TEMPLATE_SMILES_SUPPORTED_ALPHABET_V1
    }
}

/// Failure to represent an accepted peptide with the legacy template profile.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum LegacyPeptideTemplateSmilesErrorV1 {
    /// The accepted residue's backbone or side chain has no V1 template.
    #[error(
        "residue {residue} at position {position} is unsupported by legacy peptide-template \
         profile {LEGACY_PEPTIDE_TEMPLATE_SMILES_PROFILE_V1}"
    )]
    UnsupportedTemplateResidue {
        /// One-based residue position in the accepted N-to-C sequence.
        position: usize,
        /// The accepted residue the fixed profile cannot represent.
        residue: ResidueCode,
    },
    /// The required output length cannot be represented on this platform.
    #[error("legacy peptide-template SMILES output size overflow")]
    OutputSizeOverflow,
    /// The bounded template builder could not reserve output storage.
    #[error("legacy peptide-template SMILES could not reserve output storage")]
    AllocationFailed,
}

const fn side_chain_v1(residue: ResidueCode) -> Option<&'static str> {
    match residue {
        ResidueCode::Alanine => Some("(C)"),
        ResidueCode::Cysteine => Some("(CS)"),
        ResidueCode::AsparticAcid => Some("(CC(=O)[O-])"),
        ResidueCode::GlutamicAcid => Some("(CCC(=O)[O-])"),
        ResidueCode::Phenylalanine => Some("(Cc1ccccc1)"),
        ResidueCode::Glycine => Some("([H])"),
        ResidueCode::Histidine => Some("(CC1=C[NH]C=N1)"),
        ResidueCode::Isoleucine => Some("([C@H](CC)C)"),
        ResidueCode::Lysine => Some("(CCCC[NH3+])"),
        ResidueCode::Leucine => Some("(CC(C)C)"),
        ResidueCode::Methionine => Some("(CCSC)"),
        ResidueCode::Asparagine => Some("(CC(=O)N)"),
        ResidueCode::Glutamine => Some("(CCC(=O)N)"),
        ResidueCode::Arginine => Some("(CCCNC(=[NH2+])N)"),
        ResidueCode::Serine => Some("(CO)"),
        ResidueCode::Threonine => Some("([C@H](O)C)"),
        ResidueCode::Valine => Some("(C(C)C)"),
        ResidueCode::Tryptophan => Some("(CC1=CC=C2C(=C1)C(=CN2))"),
        ResidueCode::Tyrosine => Some("(Cc1ccc(O)cc1)"),
        ResidueCode::Proline => None,
    }
}
