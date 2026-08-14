//! Explicit-adapter CLI composition for existing-molecule coordinate regeneration.

use std::path::Path;

use ferrum_document::{
    DocumentSession, DocumentSessionError, SessionOperation, SessionOperationV1,
};
use thiserror::Error;

use crate::{
    ExplicitAdapterError, MoleculeCoordinateBuildError, build_molecule_coordinate_update_v1,
    explicit_adapter::load_explicit_adapter,
};

/// Regenerate one source-ID-selected molecule and return the accepted CDML snapshot.
pub(crate) fn generate_molecule_coordinates_cdml(
    adapter_path: &Path,
    source: &str,
    molecule_source_id: &str,
) -> Result<String, MoleculeCoordinateCliError> {
    if molecule_source_id.trim().is_empty() {
        return Err(MoleculeCoordinateCliError::BlankMoleculeId);
    }
    let mut session = DocumentSession::load(source)?;
    let observation = session.observe(0)?;
    let molecule_id = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some(molecule_source_id))
        .and_then(|molecule| molecule.id())
        .cloned()
        .ok_or_else(|| MoleculeCoordinateCliError::UnknownMolecule {
            source_id: molecule_source_id.to_owned(),
        })?;
    let engine = load_explicit_adapter(adapter_path)?;
    let update = build_molecule_coordinate_update_v1(&engine, &observation, &molecule_id)?;
    let result = session.submit(
        0,
        SessionOperation::V1(SessionOperationV1::SetMoleculeAtomPositions { update }),
    )?;
    Ok(result.observation().snapshot().cdml().to_owned())
}

/// Failure while regenerating one CDML molecule through the CLI boundary.
#[derive(Debug, Error)]
pub enum MoleculeCoordinateCliError {
    /// A molecule selector must contain authored source identity text.
    #[error("molecule source ID must be nonblank")]
    BlankMoleculeId,
    /// No durable molecule has the exact requested authored source ID.
    #[error("CDML contains no durable molecule with source ID {source_id:?}")]
    UnknownMolecule { source_id: String },
    /// The document session rejected load, observation, or atomic operation acceptance.
    #[error(transparent)]
    Document(#[from] DocumentSessionError),
    /// The explicit native adapter path was unsafe or could not load.
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    /// Chemistry or placement could not build a complete update.
    #[error(transparent)]
    Build(#[from] MoleculeCoordinateBuildError),
}
