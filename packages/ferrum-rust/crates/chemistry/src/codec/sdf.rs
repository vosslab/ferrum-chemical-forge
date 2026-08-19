//! Explicit-adapter single-record SDF export and bounded inspection.

use crate::{
    ChemistryError, ExplicitAdapterError, MolblockVersion, MoleculeInspectionFactsV1, SdfError,
    SdfProperty, SdfRecord, SmilesInspectionError, load_explicit_adapter,
    molecule_inspection_facts, validate_sdf_input,
};
use serde::Serialize;
use std::path::Path;
use thiserror::Error;
pub const SDF_INSPECTION_SCHEMA_V1: &str = "ferrum-sdf-inspection-v1";
pub fn sdf_from_smiles(
    adapter_path: &Path,
    smiles: &str,
    title: &str,
    property_arguments: &[String],
    version: MolblockVersion,
) -> Result<String, SdfExportError> {
    let properties = property_arguments
        .iter()
        .map(|argument| {
            let (name, value) = argument.split_once('=').ok_or_else(|| {
                SdfExportError::InvalidPropertyArgument {
                    argument: argument.clone(),
                }
            })?;
            SdfProperty::new(name, value).map_err(SdfExportError::Record)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let engine = load_explicit_adapter(adapter_path)?;
    let molecule = engine.smiles_to_molecule(smiles)?;
    let record = SdfRecord::new(molecule.molecule().clone(), title, properties)?;
    engine
        .records_to_sdf(&[record], version)
        .map_err(SdfExportError::Chemistry)
}
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
        adapter_abi: crate::ADAPTER_ABI_VERSION,
        records,
    })
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SdfInspectionV1 {
    schema: &'static str,
    adapter_abi: u32,
    records: Vec<SdfRecordInspectionV1>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SdfRecordInspectionV1 {
    title: String,
    properties: Vec<SdfPropertyInspectionV1>,
    molecule: MoleculeInspectionFactsV1,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SdfPropertyInspectionV1 {
    name: String,
    value: String,
}
#[derive(Debug, Error)]
pub enum SdfExportError {
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    #[error("SDF property must use NAME=VALUE syntax: {argument}")]
    InvalidPropertyArgument { argument: String },
    #[error(transparent)]
    Record(#[from] SdfError),
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}
#[derive(Debug, Error)]
pub enum SdfInspectionError {
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    #[error(transparent)]
    Molecule(#[from] SmilesInspectionError),
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_input_is_rejected_before_adapter_loading() {
        assert!(matches!(
            inspect_sdf(Path::new("/not/a/loaded/adapter.dylib"), ""),
            Err(SdfInspectionError::Chemistry(
                ChemistryError::InvalidSdfInput { .. }
            ))
        ));
    }
}
