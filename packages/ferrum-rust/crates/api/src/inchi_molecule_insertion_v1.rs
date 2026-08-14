//! Rust-owned InChI parsing and conversion into one document insertion.

use ferrum_chemistry::{
    ChemEngine, ChemistryError, KekulizeOptions, KekulizeOptionsError, validate_inchi_input,
};
use ferrum_document::MoleculeInsertionV1;
use ferrum_geometry::MoleculePlacementV1;
use thiserror::Error;

use crate::complete_graph_molecule_insertion_v1::{
    CompleteGraphMoleculeInsertionError,
    build_complete_graph_molecule_insertion_from_validated_facts_v1,
    validate_supported_inchi_complete_graph_facts_v1,
};

/// Parse, normalize, place, and validate one InChI molecule off-session.
///
/// The chemistry engine remains the sole InChI parser. The returned value is
/// handle-free and cannot mutate a document until a session prepares and commits it
/// against an exact current revision.
pub fn build_inchi_molecule_insertion_v1<E: ChemEngine>(
    engine: &E,
    inchi: &str,
    placement: MoleculePlacementV1,
) -> Result<MoleculeInsertionV1, InchiMoleculeBuildError> {
    validate_inchi_input(inchi)?;
    let parsed = engine.inchi_to_molecule(inchi)?;
    let mut graph = parsed.molecule().clone();
    validate_supported_inchi_complete_graph_facts_v1(&graph)?;
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
    validate_supported_inchi_complete_graph_facts_v1(&graph)?;
    build_complete_graph_molecule_insertion_from_validated_facts_v1(&graph, placement)
        .map_err(Into::into)
}

/// Failure while converting one untrusted InChI into persistable molecule facts.
#[derive(Debug, Error)]
pub enum InchiMoleculeBuildError {
    /// Input validation, parsing, or native chemistry failed.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// The closed Kekule request could not be formed.
    #[error(transparent)]
    KekulizeOptions(#[from] KekulizeOptionsError),
    /// The complete graph cannot be represented by the current CDML insertion grammar.
    #[error(transparent)]
    CompleteGraph(#[from] CompleteGraphMoleculeInsertionError),
}
