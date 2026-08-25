//! Rust-owned InChI parsing and detached document preparation.

use ferrum_chemistry::{ChemEngine, ChemistryError, validate_inchi_input};
use ferrum_geometry::MoleculePlacementV1;
use thiserror::Error;

use super::{
    DocumentMoleculePreparationErrorV2, PreparedDocumentMoleculeV2,
    prepare_complete_graph_for_document_v2,
};

/// Parse, normalize, place, and validate one InChI molecule off-session.
///
/// The chemistry engine remains the sole InChI parser. The returned value is
/// handle-free and cannot mutate a document until a session prepares and commits it
/// against an exact current revision.
pub fn prepare_inchi_molecule_for_document_v2<E: ChemEngine>(
    engine: &E,
    inchi: &str,
    placement: MoleculePlacementV1,
) -> Result<PreparedDocumentMoleculeV2, InchiMoleculePreparationErrorV2> {
    validate_inchi_input(inchi)?;
    let parsed = engine.inchi_to_molecule(inchi)?;
    prepare_complete_graph_for_document_v2(engine, parsed.molecule(), placement).map_err(Into::into)
}

/// Failure while converting one untrusted InChI into persistable molecule facts.
#[derive(Debug, Error)]
pub enum InchiMoleculePreparationErrorV2 {
    /// Input validation, parsing, or native chemistry failed.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// The complete graph cannot be represented by the durable document grammar.
    #[error(transparent)]
    Preparation(#[from] DocumentMoleculePreparationErrorV2),
}
