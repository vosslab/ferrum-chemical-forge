//! Provisional bounded SDF inspection through an explicit ABI-4 adapter.

use std::path::Path;

use ferrum_chemistry::{ChemistryError, validate_sdf_input};
use serde::Serialize;
use thiserror::Error;

use crate::explicit_adapter::{ExplicitAdapterError, load_explicit_adapter};
use crate::smiles_inspection::{
    MoleculeInspectionFactsV1, SmilesInspectionError, molecule_inspection_facts,
};

/// The single machine-readable schema emitted by `ferrum sdf inspect`.
pub const SDF_INSPECTION_SCHEMA_V1: &str = "ferrum-sdf-inspection-v1";

/// Inspect bounded UTF-8 SDF through one caller-selected ABI-4 adapter.
///
/// Input is validated before adapter loading. Imported records retain source
/// order, titles, and repeated property names, and contain complete owned
/// molecule facts rather than native handles.
pub fn inspect_sdf(
    adapter_path: &Path,
    input: &str,
) -> Result<SdfInspectionV1, SdfInspectionError> {
    validate_sdf_input(input)?;
    let engine = load_explicit_adapter(adapter_path)?;
    let records = engine
        .sdf_to_records(input)?
        .into_iter()
        .map(|record| {
            let molecule = molecule_inspection_facts(record.molecule())?;
            let properties = record
                .properties()
                .iter()
                .map(|property| SdfPropertyInspectionV1 {
                    name: property.name().to_owned(),
                    value: property.value().to_owned(),
                })
                .collect();
            Ok(SdfRecordInspectionV1 {
                title: record.title().to_owned(),
                properties,
                molecule,
            })
        })
        .collect::<Result<Vec<_>, SdfInspectionError>>()?;
    Ok(SdfInspectionV1 {
        schema: SDF_INSPECTION_SCHEMA_V1,
        adapter_abi: ferrum_chemistry::ADAPTER_ABI_VERSION,
        records,
    })
}

/// Immutable JSON payload for provisional bounded SDF inspection.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SdfInspectionV1 {
    schema: &'static str,
    adapter_abi: u32,
    records: Vec<SdfRecordInspectionV1>,
}

/// One imported SDF record in exact source order.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SdfRecordInspectionV1 {
    title: String,
    properties: Vec<SdfPropertyInspectionV1>,
    molecule: MoleculeInspectionFactsV1,
}

/// One imported text property, including repeated names.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SdfPropertyInspectionV1 {
    name: String,
    value: String,
}

/// A rejected adapter, SDF request, or complete-molecule observation.
#[derive(Debug, Error)]
pub enum SdfInspectionError {
    /// The caller did not select a safe explicit adapter path.
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    /// Input or native SDF parsing violated the bounded chemistry contract.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// A native record omitted complete atom-aligned molecule facts.
    #[error(transparent)]
    Molecule(#[from] SmilesInspectionError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_input_is_rejected_before_adapter_loading() {
        let error = inspect_sdf(Path::new("/not/a/loaded/adapter.dylib"), "")
            .expect_err("empty SDF must fail before adapter loading");

        assert!(matches!(
            error,
            SdfInspectionError::Chemistry(ChemistryError::InvalidSdfInput { .. })
        ));
    }
}
