//! Explicit-adapter InChI import, export, and InChIKey observation.

use std::path::Path;

use serde::Serialize;
use thiserror::Error;

use crate::{
    ChemistryError, ExplicitAdapterError, InchiMode, MoleculeInspectionFactsV1,
    SmilesInspectionError, load_explicit_adapter, molecule_inspection_facts, validate_inchi_input,
};

/// Machine-readable schema emitted by InChI inspection.
pub const INCHI_INSPECTION_SCHEMA_V1: &str = "ferrum-inchi-inspection-v1";
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
        adapter_abi: crate::ADAPTER_ABI_VERSION,
        inchi_key,
        molecule: molecule_inspection_facts(&molecule)?,
    })
}
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
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InchiInspectionV1 {
    schema: &'static str,
    adapter_abi: u32,
    inchi_key: String,
    molecule: MoleculeInspectionFactsV1,
}
#[derive(Debug, Error)]
pub enum InchiInspectionError {
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    #[error(transparent)]
    Molecule(#[from] SmilesInspectionError),
}
#[derive(Debug, Error)]
pub enum InchiExportError {
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_inchi_is_rejected_before_adapter_loading() {
        assert!(matches!(
            inspect_inchi(Path::new("/not/a/loaded/adapter.dylib"), "methane"),
            Err(InchiInspectionError::Chemistry(
                ChemistryError::InvalidInchiInput { .. }
            ))
        ));
    }
}
