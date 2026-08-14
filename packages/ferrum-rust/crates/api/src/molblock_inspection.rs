//! Provisional single-molblock inspection through an explicit ABI-4 adapter.

use std::path::Path;

use ferrum_chemistry::{ChemistryError, validate_molblock_input};
use serde::Serialize;
use thiserror::Error;

use crate::explicit_adapter::{ExplicitAdapterError, load_explicit_adapter};
use crate::smiles_inspection::{
    MoleculeInspectionFactsV1, SmilesInspectionError, molecule_inspection_facts,
};

/// The machine-readable schema emitted by `ferrum molblock inspect`.
pub const MOLBLOCK_INSPECTION_SCHEMA_V1: &str = "ferrum-molblock-inspection-v1";

/// Inspect one bounded V2000 or V3000 molblock with a caller-selected adapter.
pub fn inspect_molblock(
    adapter_path: &Path,
    input: &str,
) -> Result<MolblockInspectionV1, MolblockInspectionError> {
    validate_molblock_input(input)?;
    let engine = load_explicit_adapter(adapter_path)?;
    let molecule = engine.molblock_to_molecule(input)?;
    Ok(MolblockInspectionV1 {
        schema: MOLBLOCK_INSPECTION_SCHEMA_V1,
        adapter_abi: ferrum_chemistry::ADAPTER_ABI_VERSION,
        molecule: molecule_inspection_facts(&molecule)?,
    })
}

/// Immutable JSON payload for provisional single-molblock inspection.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MolblockInspectionV1 {
    schema: &'static str,
    adapter_abi: u32,
    molecule: MoleculeInspectionFactsV1,
}

/// A rejected adapter, molblock request, or complete-molecule observation.
#[derive(Debug, Error)]
pub enum MolblockInspectionError {
    /// The caller did not select a safe explicit adapter path.
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    /// Input or native molblock parsing violated the bounded chemistry contract.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// The imported molecule omitted complete atom-aligned facts.
    #[error(transparent)]
    Molecule(#[from] SmilesInspectionError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_input_is_rejected_before_adapter_loading() {
        let error = inspect_molblock(Path::new("/not/a/loaded/adapter.dylib"), "")
            .expect_err("empty molblock must fail before adapter loading");

        assert!(matches!(
            error,
            MolblockInspectionError::Chemistry(ChemistryError::InvalidMolblockInput { .. })
        ));
    }
}
