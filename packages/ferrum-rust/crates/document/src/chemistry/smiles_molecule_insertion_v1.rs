//! Rust-owned composition of SMILES chemistry, placement, and document mutation.

use crate::{
    DocumentSession, DocumentSessionError, MoleculeInsertionV1, MoleculeInsertionV1Error,
    PreparedSessionTransitionV1, ProjectionError, SessionOperation,
    SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
};
use ferrum_chemistry::{
    BondOrder, ChemEngine, ChemistryError, KekulizeOptions, KekulizeOptionsError,
};
use ferrum_geometry::{GeometryError, MoleculePlacementV1};
use thiserror::Error;

use super::complete_graph_molecule_insertion_v1::{
    CompleteGraphMoleculeInsertionError, build_complete_graph_molecule_insertion_v1,
    validate_supported_complete_graph_facts_v1,
};

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
    let insertion = build_smiles_molecule_insertion_v1(engine, smiles, placement)?;
    session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            expected_revision,
            SessionOperation::V1(SessionOperationV1::InsertMoleculeV1(insertion)),
            TransitionAuthorizationV1::None,
        ))
        .map_err(SmilesMoleculeInsertionError::from)
}

/// Build one complete CDML-representable molecule without touching a session.
///
/// This owned result is safe to compute outside the Qt UI thread. The session can
/// later validate and prepare it at an exact current revision without repeating
/// native chemistry or geometry work.
pub fn build_smiles_molecule_insertion_v1<E: ChemEngine>(
    engine: &E,
    smiles: &str,
    placement: MoleculePlacementV1,
) -> Result<MoleculeInsertionV1, SmilesMoleculeBuildError> {
    let parsed = engine.smiles_to_molecule(smiles)?;
    let mut graph = parsed.molecule().clone();
    validate_supported_complete_graph_facts_v1(&graph)?;
    if graph
        .atoms()
        .iter()
        .any(ferrum_chemistry::MolAtom::is_aromatic)
        || graph
            .bonds()
            .iter()
            .any(ferrum_chemistry::MolBond::is_aromatic)
    {
        let options = KekulizeOptions::new(true, true, 100)?;
        graph = engine.kekulize(&graph, options)?;
    }
    build_complete_graph_molecule_insertion_v1(&graph, placement).map_err(Into::into)
}

impl From<CompleteGraphMoleculeInsertionError> for SmilesMoleculeBuildError {
    fn from(error: CompleteGraphMoleculeInsertionError) -> Self {
        match error {
            CompleteGraphMoleculeInsertionError::MissingCoordinates => Self::MissingCoordinates,
            CompleteGraphMoleculeInsertionError::CoordinateCountMismatch { .. }
            | CompleteGraphMoleculeInsertionError::NonFiniteCoordinate { .. } => {
                Self::MissingCoordinates
            }
            CompleteGraphMoleculeInsertionError::Geometry(error) => Self::Geometry(error),
            CompleteGraphMoleculeInsertionError::Position(error) => Self::Position(error),
            CompleteGraphMoleculeInsertionError::Insertion(error) => Self::Insertion(error),
            CompleteGraphMoleculeInsertionError::UnsupportedAtomFact { atom_index, fact } => {
                Self::UnsupportedAtomFact { atom_index, fact }
            }
            CompleteGraphMoleculeInsertionError::UnsupportedBondFact { bond_index, fact } => {
                Self::UnsupportedBondFact { bond_index, fact }
            }
            CompleteGraphMoleculeInsertionError::UnsupportedBondOrder { start, end, order } => {
                Self::UnsupportedBondOrder { start, end, order }
            }
        }
    }
}

/// Failure while building a complete CDML-representable molecule off-session.
#[derive(Debug, Error)]
pub enum SmilesMoleculeBuildError {
    /// The chemistry engine rejected the request or its native boundary failed.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// The explicit Kekule request could not be constructed.
    #[error(transparent)]
    KekulizeOptions(#[from] KekulizeOptionsError),
    /// The chemistry engine omitted the promised complete coordinate set.
    #[error("SMILES molecule has no complete 2D coordinate set")]
    MissingCoordinates,
    /// Placement could not produce finite, nondegenerate document geometry.
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    /// A finite document point unexpectedly failed projection validation.
    #[error(transparent)]
    Position(#[from] ProjectionError),
    /// The closed insertion graph rejected a converted chemistry fact.
    #[error(transparent)]
    Insertion(#[from] MoleculeInsertionV1Error),
    /// One atom carries a fact without a proven V1 insertion encoding.
    #[error(
        "SMILES atom {atom_index} has {fact}, which this V1 insertion writer cannot encode yet"
    )]
    UnsupportedAtomFact {
        atom_index: usize,
        fact: &'static str,
    },
    /// One bond carries a fact without a proven V1 insertion encoding.
    #[error(
        "SMILES bond {bond_index} has {fact}, which this V1 insertion writer cannot encode yet"
    )]
    UnsupportedBondFact {
        bond_index: usize,
        fact: &'static str,
    },
    /// This V1 writer has not established an exact encoding for the bond order.
    #[error(
        "SMILES bond {start}-{end} has {order:?} order, which this V1 insertion writer cannot \
         encode yet"
    )]
    UnsupportedBondOrder {
        start: usize,
        end: usize,
        order: BondOrder,
    },
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
