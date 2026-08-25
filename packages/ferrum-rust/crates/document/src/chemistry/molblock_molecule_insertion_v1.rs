//! Rust-owned molblock parsing and conversion into one document insertion.

use crate::DocumentMoleculePreparationErrorV2;
use ferrum_chemistry::{ChemEngine, ChemistryError};
use ferrum_geometry::MoleculePlacementV1;
use thiserror::Error;

use super::{PreparedDocumentMoleculeV2, prepare_complete_graph_for_document_v2};

/// Parse, normalize, place, and validate one V2000 or V3000 molecule off-session.
///
/// The chemistry engine remains the only molblock parser. The returned value is
/// handle-free and cannot mutate a document until a session prepares and commits it
/// against an exact current revision.
pub fn prepare_molblock_molecule_for_document_v2<E: ChemEngine>(
    engine: &E,
    molblock: &str,
    placement: MoleculePlacementV1,
) -> Result<PreparedDocumentMoleculeV2, MolblockMoleculeBuildError> {
    let parsed = engine.molblock_to_molecule(molblock)?;
    prepare_complete_graph_for_document_v2(engine, parsed.molecule(), placement).map_err(Into::into)
}

/// Failure while converting one untrusted molblock into persistable molecule facts.
#[derive(Debug, Error)]
pub enum MolblockMoleculeBuildError {
    /// Input validation, parsing, or native chemistry failed.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// The complete graph cannot be represented by the durable CDML request grammar.
    #[error(transparent)]
    Preparation(#[from] DocumentMoleculePreparationErrorV2),
}

#[cfg(test)]
#[path = "molblock_molecule_insertion_v1_tests.rs"]
mod tests;
