//! Explicit-adapter molblock import, inspection, and export.

use crate::{
    ChemistryError, ExplicitAdapterError, MolblockVersion, MoleculeInspectionFactsV1,
    NativeTextOutputLimit, SmilesInspectionError, load_explicit_adapter, molecule_inspection_facts,
    validate_molblock_input,
};
use serde::Serialize;
use std::path::Path;
use thiserror::Error;
pub const MOLBLOCK_INSPECTION_SCHEMA_V1: &str = "ferrum-molblock-inspection-v1";
const MOLBLOCK_CODEC_TEXT_LIMIT: NativeTextOutputLimit = NativeTextOutputLimit::ADAPTER_MAXIMUM;
pub fn inspect_molblock(
    adapter_path: &Path,
    input: &str,
) -> Result<MolblockInspectionV1, MolblockInspectionError> {
    validate_molblock_input(input)?;
    let engine = load_explicit_adapter(adapter_path)?;
    let molecule = engine.molblock_to_molecule(input)?;
    Ok(MolblockInspectionV1 {
        schema: MOLBLOCK_INSPECTION_SCHEMA_V1,
        adapter_abi: crate::ADAPTER_ABI_VERSION,
        molecule: molecule_inspection_facts(&molecule)?,
    })
}
pub fn molblock_from_smiles(
    adapter_path: &Path,
    smiles: &str,
    version: MolblockVersion,
) -> Result<String, MolblockExportError> {
    let engine = load_explicit_adapter(adapter_path)?;
    let molecule = engine.smiles_to_molecule(smiles)?;
    engine
        .molecule_to_molblock(molecule.molecule(), version, MOLBLOCK_CODEC_TEXT_LIMIT)
        .map_err(MolblockExportError::Chemistry)
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MolblockInspectionV1 {
    schema: &'static str,
    adapter_abi: u32,
    molecule: MoleculeInspectionFactsV1,
}
#[derive(Debug, Error)]
pub enum MolblockInspectionError {
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    #[error(transparent)]
    Molecule(#[from] SmilesInspectionError),
}
#[derive(Debug, Error)]
pub enum MolblockExportError {
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_input_is_rejected_before_adapter_loading() {
        assert!(matches!(
            inspect_molblock(Path::new("/not/a/loaded/adapter.dylib"), ""),
            Err(MolblockInspectionError::Chemistry(
                ChemistryError::InvalidMolblockInput { .. }
            ))
        ));
    }
}
