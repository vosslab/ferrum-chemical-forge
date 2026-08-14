//! Bounded strict peptide-template composition for the native Qt compatibility action.
//!
//! This V1 ingress policy is deliberately narrower than a peptide-size claim. It
//! derives its byte cap from the exact legacy template grammar and the native
//! adapter's SMILES envelope before parsing or loading native chemistry.

use ferrum_chemistry::{ChemEngine, NATIVE_SMILES_MAX_INPUT_BYTES, NativeChemEngine};
use ferrum_document::MoleculeInsertionV1;
use ferrum_domain::{
    LegacyPeptideTemplateSmilesErrorV1, PeptideSyntaxError, ResidueCode,
    build_legacy_peptide_template_smiles_v1, parse_one_letter_sequence,
};
use ferrum_geometry::MoleculePlacementV1;
use thiserror::Error;

use crate::{
    SmilesMoleculeBuildError,
    complete_graph_molecule_insertion_v1::{
        build_complete_graph_molecule_insertion_from_validated_facts_v1,
        validate_supported_peptide_template_complete_graph_facts_v1,
    },
};

/// Named native insertion profile; distinct from the pure 19-residue domain profile.
pub const NATIVE_PEPTIDE_TEMPLATE_INSERTION_PROFILE_V1: &str =
    "ferrum-native-peptide-template-insertion-v1";

/// Residues admitted by the actual native V1 insertion route.
pub const NATIVE_PEPTIDE_TEMPLATE_INSERTION_SUPPORTED_ALPHABET_V1: &str = "ACDEFGIKLMNQRSTVY";

/// Fixed bytes outside the profile's worst-case per-residue expansion.
pub const NATIVE_PEPTIDE_TEMPLATE_INSERTION_FIXED_OUTPUT_BYTES_V1: usize = 9;

/// Worst added bytes for one native-profile residue after the first.
///
/// Arginine contributes 17 branch bytes; the fixed link plus close contributes 14.
pub const NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_ADDITIONAL_RESIDUE_BYTES_V1: usize = 31;

/// Maximum submitted UTF-8 bytes for interactive V1 template text.
///
/// Every accepted residue is one ASCII byte. The worst supported template is
/// `FIXED_OUTPUT + residues * MAX_ADDITIONAL`, which must fit the adapter's
/// SMILES-byte ceiling. This is intentionally not a general peptide limit.
pub const NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1: usize =
    (NATIVE_SMILES_MAX_INPUT_BYTES - NATIVE_PEPTIDE_TEMPLATE_INSERTION_FIXED_OUTPUT_BYTES_V1)
        / NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_ADDITIONAL_RESIDUE_BYTES_V1;

/// Fully preflighted strict template request, ready for an already-loaded engine.
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

/// Build a frozen insertion from an already preflighted template request.
pub fn build_supported_peptide_template_molecule_insertion_v1(
    engine: &NativeChemEngine,
    request: &SupportedPeptideTemplateRequestV1,
    placement: MoleculePlacementV1,
) -> Result<MoleculeInsertionV1, PeptideTemplateMoleculeBuildErrorV1> {
    build_native_template_insertion_with_engine(engine, request, placement)
}

pub(crate) fn build_native_template_insertion_with_engine<E: ChemEngine>(
    engine: &E,
    request: &SupportedPeptideTemplateRequestV1,
    placement: MoleculePlacementV1,
) -> Result<MoleculeInsertionV1, PeptideTemplateMoleculeBuildErrorV1> {
    let parsed = engine
        .smiles_to_molecule(request.smiles())
        .map_err(SmilesMoleculeBuildError::from)?;
    let mut graph = parsed.molecule().clone();
    validate_supported_peptide_template_complete_graph_facts_v1(&graph)
        .map_err(SmilesMoleculeBuildError::from)?;
    if graph
        .atoms()
        .iter()
        .any(ferrum_chemistry::MolAtom::is_aromatic)
        || graph
            .bonds()
            .iter()
            .any(ferrum_chemistry::MolBond::is_aromatic)
    {
        let options = ferrum_chemistry::KekulizeOptions::new(true, true, 100)
            .map_err(SmilesMoleculeBuildError::from)?;
        graph = engine
            .kekulize(&graph, options)
            .map_err(SmilesMoleculeBuildError::from)?;
        validate_supported_peptide_template_complete_graph_facts_v1(&graph)
            .map_err(SmilesMoleculeBuildError::from)?;
    }
    build_complete_graph_molecule_insertion_from_validated_facts_v1(&graph, placement)
        .map_err(SmilesMoleculeBuildError::from)
        .map_err(PeptideTemplateMoleculeBuildErrorV1::Build)
}

/// Native-engine stage failure after successful strict template preflight.
#[derive(Debug, Error)]
pub enum PeptideTemplateMoleculeBuildErrorV1 {
    /// The frozen SMILES insertion stage rejected the compiled request.
    #[error(transparent)]
    Build(#[from] SmilesMoleculeBuildError),
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
