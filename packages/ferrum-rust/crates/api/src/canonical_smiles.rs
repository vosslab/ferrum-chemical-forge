//! Provisional explicit-adapter canonical SMILES export for Rust and CLI callers.

use std::path::Path;

use ferrum_chemistry::ChemistryError;
use thiserror::Error;

use crate::explicit_adapter::{ExplicitAdapterError, load_explicit_adapter};

/// Parse one SMILES value and serialize its complete graph canonically.
///
/// The adapter path follows the same explicit absolute regular-file policy as
/// `ferrum smiles inspect`. The returned printable ASCII string contains no
/// trailing newline.
pub fn canonical_smiles_from_smiles(
    adapter_path: &Path,
    smiles: &str,
) -> Result<String, CanonicalSmilesError> {
    let engine = load_explicit_adapter(adapter_path)?;
    let molecule = engine.smiles_to_molecule(smiles)?;
    engine
        .molecule_to_smiles(molecule.molecule())
        .map_err(CanonicalSmilesError::Chemistry)
}

/// A rejected explicit adapter, SMILES input, or canonical serialization.
#[derive(Debug, Error)]
pub enum CanonicalSmilesError {
    /// The caller did not select a safe explicit adapter path.
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    /// Native parsing or complete-graph SMILES serialization failed.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}
