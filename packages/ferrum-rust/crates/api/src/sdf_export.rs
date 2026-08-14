//! Provisional explicit-adapter single-record SDF export.

use std::path::Path;

use ferrum_chemistry::{ChemistryError, MolblockVersion, SdfError, SdfProperty, SdfRecord};
use thiserror::Error;

use crate::explicit_adapter::{ExplicitAdapterError, load_explicit_adapter};

/// Parse one SMILES value and export one ordered SDF record.
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

/// A rejected adapter, property, molecule, or SDF writer operation.
#[derive(Debug, Error)]
pub enum SdfExportError {
    /// The caller did not select a safe explicit adapter path.
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    /// A property argument lacked the documented `NAME=VALUE` boundary.
    #[error("SDF property must use NAME=VALUE syntax: {argument}")]
    InvalidPropertyArgument {
        /// The exact rejected argument.
        argument: String,
    },
    /// A title or property would be silently omitted or structurally ambiguous.
    #[error(transparent)]
    Record(#[from] SdfError),
    /// Native parsing or ordered SDF export failed.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}
