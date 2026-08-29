//! Explicit-adapter canonical SMILES and SMARTS export.

use crate::{ChemistryError, ExplicitAdapterError, NativeTextOutputLimit, load_explicit_adapter};
use std::path::Path;
use thiserror::Error;
const SMARTS_CODEC_SMILES_TEXT_LIMIT: NativeTextOutputLimit =
    NativeTextOutputLimit::ADAPTER_MAXIMUM;
pub fn canonical_smiles_from_smiles(
    adapter_path: &Path,
    smiles: &str,
) -> Result<String, CanonicalSmilesError> {
    let engine = load_explicit_adapter(adapter_path)?;
    let molecule = engine.smiles_to_molecule(smiles)?;
    engine
        .molecule_to_smiles(molecule.molecule(), SMARTS_CODEC_SMILES_TEXT_LIMIT)
        .map_err(CanonicalSmilesError::Chemistry)
}
pub fn smarts_from_smiles(adapter_path: &Path, smiles: &str) -> Result<String, SmartsExportError> {
    let engine = load_explicit_adapter(adapter_path)?;
    let molecule = engine.smiles_to_molecule(smiles)?;
    engine
        .molecule_to_smarts(molecule.molecule())
        .map_err(SmartsExportError::Chemistry)
}
#[derive(Debug, Error)]
pub enum CanonicalSmilesError {
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}
#[derive(Debug, Error)]
pub enum SmartsExportError {
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}
