//! Explicit-adapter InChI import, export, and InChIKey observation.

use std::path::Path;

use ferrum_chemistry::{ChemistryError, InchiMode, validate_inchi_input};
use serde::Serialize;
use thiserror::Error;

use crate::explicit_adapter::{ExplicitAdapterError, load_explicit_adapter};
use crate::smiles_inspection::{
    MoleculeInspectionFactsV1, SmilesInspectionError, molecule_inspection_facts,
};

/// Machine-readable schema emitted by `ferrum inchi inspect`.
pub const INCHI_INSPECTION_SCHEMA_V1: &str = "ferrum-inchi-inspection-v1";

/// Inspect one validated InChI through a caller-selected ABI-4 adapter.
pub fn inspect_inchi(
    adapter_path: &Path,
    inchi: &str,
) -> Result<InchiInspectionV1, InchiInspectionError> {
    validate_inchi_input(inchi)?;
    let engine = load_explicit_adapter(adapter_path)?;
    let inchi_key = engine.inchi_to_inchi_key(inchi)?;
    let molecule = engine.inchi_to_molecule(inchi)?;
    Ok(InchiInspectionV1 {
        schema: INCHI_INSPECTION_SCHEMA_V1,
        adapter_abi: ferrum_chemistry::ADAPTER_ABI_VERSION,
        inchi_key,
        molecule: molecule_inspection_facts(&molecule)?,
    })
}

/// Parse one SMILES value and export its graph through a closed InChI mode.
pub fn inchi_from_smiles(
    adapter_path: &Path,
    smiles: &str,
    mode: InchiMode,
) -> Result<String, InchiExportError> {
    let engine = load_explicit_adapter(adapter_path)?;
    let molecule = engine.smiles_to_molecule(smiles)?;
    engine
        .molecule_to_inchi(molecule.molecule(), mode)
        .map_err(InchiExportError::Chemistry)
}

/// Immutable InChI inspection payload.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InchiInspectionV1 {
    schema: &'static str,
    adapter_abi: u32,
    inchi_key: String,
    molecule: MoleculeInspectionFactsV1,
}

/// A rejected adapter, InChI value, key, or imported molecule observation.
#[derive(Debug, Error)]
pub enum InchiInspectionError {
    /// The caller did not select a safe explicit adapter path.
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    /// Input or native InChI processing violated the chemistry contract.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// The imported molecule omitted complete atom-aligned facts.
    #[error(transparent)]
    Molecule(#[from] SmilesInspectionError),
}

/// A rejected adapter, SMILES value, or InChI serialization.
#[derive(Debug, Error)]
pub enum InchiExportError {
    /// The caller did not select a safe explicit adapter path.
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    /// Native SMILES parsing or InChI export failed.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inchi_is_rejected_before_adapter_loading() {
        let error = inspect_inchi(Path::new("/not/a/loaded/adapter.dylib"), "methane")
            .expect_err("invalid InChI must fail before adapter loading");

        assert!(matches!(
            error,
            InchiInspectionError::Chemistry(ChemistryError::InvalidInchiInput { .. })
        ));
    }
}
