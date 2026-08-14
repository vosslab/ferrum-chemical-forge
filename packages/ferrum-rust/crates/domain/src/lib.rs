//! Higher-level chemistry-domain utilities.
//!
//! This crate owns pure domain plans.  It accepts explicit, already-selected
//! chemistry facts and never guesses a ring, carbohydrate numbering, or
//! stereochemistry from a drawing format.

pub mod catalog;
pub mod haworth;
pub mod linear_form;
pub mod peptide;
pub mod repair;
pub mod sugar;

pub use linear_form::{
    LinearFormAtomV1, LinearFormBondV1, LinearFormGraphV1, LinearFormMetadataShapeV1,
    LinearFormPlanErrorV1, LinearFormPlanV1, LinearFormPointReplacementV1, LinearFormRequestV1,
    plan_linear_form_v1,
};
pub use peptide::{
    LEGACY_PEPTIDE_TEMPLATE_SMILES_PROFILE_V1, LEGACY_PEPTIDE_TEMPLATE_SMILES_SCHEMA_V1,
    LEGACY_PEPTIDE_TEMPLATE_SMILES_SUPPORTED_ALPHABET_V1, LegacyPeptideTemplateSmilesErrorV1,
    LegacyPeptideTemplateSmilesV1, PeptideSequence, PeptideSyntaxError, PeptideTerminus,
    ProtonationIntent, ResidueCode, TerminusIntent, build_legacy_peptide_template_smiles_v1,
    parse_one_letter_sequence,
};
pub use repair::{
    CoordinatePatch, CoordinateReplacement, DepictionBond, DepictionGraph, DepictionVertex,
    PatchPreconditionError, RepairError, RepairKind, RepairOutcome, RepairRequest, plan_repair,
    plan_repair_with_outcome,
};
