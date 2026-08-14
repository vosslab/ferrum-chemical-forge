//! Provisional explicit-adapter V2000/V3000 molblock export.

use std::path::Path;

use ferrum_chemistry::{ChemistryError, MolblockVersion};
use thiserror::Error;

use crate::explicit_adapter::{ExplicitAdapterError, load_explicit_adapter};

/// Parse one SMILES value and export its complete graph and coordinates.
pub fn molblock_from_smiles(
    adapter_path: &Path,
    smiles: &str,
    version: MolblockVersion,
) -> Result<String, MolblockExportError> {
    let engine = load_explicit_adapter(adapter_path)?;
    let molecule = engine.smiles_to_molecule(smiles)?;
    engine
        .molecule_to_molblock(molecule.molecule(), version)
        .map_err(MolblockExportError::Chemistry)
}

/// A rejected adapter, SMILES input, or explicit molblock serialization.
#[derive(Debug, Error)]
pub enum MolblockExportError {
    /// The caller did not select a safe explicit adapter path.
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    /// Native parsing or coordinate-bearing molblock export failed.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}
