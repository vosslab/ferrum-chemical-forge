//! Rust-owned composition of SMILES chemistry, placement, and document mutation.

use crate::{
    DocumentMoleculePreparationErrorV2, DocumentSession, DocumentSessionError,
    PreparedSessionTransitionV1, SessionOperation, SessionOperationTransitionRequestV1,
    SessionOperationV1, TransitionAuthorizationV1,
};
use ferrum_chemistry::{ChemEngine, ChemistryError};
use ferrum_geometry::MoleculePlacementV1;
use thiserror::Error;

use super::{PreparedDocumentMoleculeV2, prepare_complete_graph_for_document_v2};

/// Parse, place, validate, and prepare one complete molecule insertion.
///
/// The chemistry engine remains the sole parser and coordinate authority. This
/// coordinator rejects any graph fact for which this V1 writer has not yet proven
/// an exact CDML encoding, converts the engine's y-up depiction into document
/// coordinates, then delegates all identity allocation and mutation authority to
/// [`DocumentSession`].
pub fn prepare_smiles_molecule_v1<E: ChemEngine>(
    engine: &E,
    session: &mut DocumentSession,
    expected_revision: u64,
    smiles: &str,
    placement: MoleculePlacementV1,
) -> Result<PreparedSessionTransitionV1, SmilesMoleculeInsertionError> {
    let request = prepare_smiles_molecule_for_document_v2(engine, smiles, placement)?
        .into_molecule_insertion_request_v1()
        .map_err(|_| SmilesMoleculeBuildError::InvalidPreparedSemantics)?;
    session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            expected_revision,
            SessionOperation::V1(SessionOperationV1::InsertMoleculeV1(request)),
            TransitionAuthorizationV1::None,
        ))
        .map_err(SmilesMoleculeInsertionError::from)
}

/// Build one complete CDML-representable molecule without touching a session.
///
/// This owned result is safe to compute outside the Qt UI thread. The session can
/// later validate and prepare it at an exact current revision without repeating
/// native chemistry or geometry work.
pub fn prepare_smiles_molecule_for_document_v2<E: ChemEngine>(
    engine: &E,
    smiles: &str,
    placement: MoleculePlacementV1,
) -> Result<PreparedDocumentMoleculeV2, SmilesMoleculeBuildError> {
    let parsed = engine.smiles_to_molecule(smiles)?;
    prepare_complete_graph_for_document_v2(engine, parsed.molecule(), placement).map_err(Into::into)
}

/// Failure while building a complete CDML-representable molecule off-session.
#[derive(Debug, Error)]
pub enum SmilesMoleculeBuildError {
    /// The chemistry engine rejected the request or its native boundary failed.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// The complete graph does not have a durable document representation.
    #[error(transparent)]
    Preparation(#[from] DocumentMoleculePreparationErrorV2),
    /// A previously admitted detached payload unexpectedly failed request lowering.
    #[error("prepared SMILES molecule has invalid document semantics")]
    InvalidPreparedSemantics,
}

/// Failure while attaching a built molecule to one authoritative document revision.
#[derive(Debug, Error)]
pub enum SmilesMoleculeInsertionError {
    /// Chemistry, placement, or lossless-encoding validation failed off-session.
    #[error(transparent)]
    Build(#[from] SmilesMoleculeBuildError),
    /// The document session rejected the prepared insertion.
    #[error(transparent)]
    Document(#[from] DocumentSessionError),
}

#[cfg(test)]
#[path = "smiles_molecule_insertion_v1_tests.rs"]
mod tests;
