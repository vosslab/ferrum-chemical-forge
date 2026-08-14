//! Provisional explicit-adapter SMARTS export for Rust and CLI callers.

use std::path::Path;

use ferrum_chemistry::ChemistryError;
use thiserror::Error;

use crate::explicit_adapter::{ExplicitAdapterError, load_explicit_adapter};

/// Parse one SMILES value and export its complete graph with the selected adapter.
///
/// The adapter path follows the same explicit absolute regular-file policy as
/// `ferrum smiles inspect`. The returned string contains no trailing newline.
pub fn smarts_from_smiles(adapter_path: &Path, smiles: &str) -> Result<String, SmartsExportError> {
    let engine = load_explicit_adapter(adapter_path)?;
    let molecule = engine.smiles_to_molecule(smiles)?;
    engine
        .molecule_to_smarts(molecule.molecule())
        .map_err(SmartsExportError::Chemistry)
}

/// A rejected explicit adapter, SMILES input, or SMARTS serialization.
#[derive(Debug, Error)]
pub enum SmartsExportError {
    /// The caller did not select a safe explicit adapter path.
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    /// Native parsing or complete-graph SMARTS export failed.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}
