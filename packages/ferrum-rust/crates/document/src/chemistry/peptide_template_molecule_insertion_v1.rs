//! Native peptide-template insertion at the document and chemistry boundary.

use ferrum_chemistry::{ChemEngine, NativeChemEngine};
use ferrum_domain::SupportedPeptideTemplateRequestV1;
use ferrum_geometry::MoleculePlacementV1;
use thiserror::Error;

use super::{
    SmilesMoleculeBuildError, build_complete_graph_molecule_insertion_from_validated_facts_v1,
    validate_supported_peptide_template_complete_graph_facts_v1,
};
use crate::MoleculeInsertionV1;

/// Build a frozen insertion from an already preflighted template request.
pub fn build_supported_peptide_template_molecule_insertion_v1(
    engine: &NativeChemEngine,
    request: &SupportedPeptideTemplateRequestV1,
    placement: MoleculePlacementV1,
) -> Result<MoleculeInsertionV1, PeptideTemplateMoleculeBuildErrorV1> {
    build_native_template_insertion_with_engine(engine, request, placement)
}

/// Build a frozen insertion through an injected chemistry engine.
pub fn build_native_template_insertion_with_engine<E: ChemEngine>(
    engine: &E,
    request: &SupportedPeptideTemplateRequestV1,
    placement: MoleculePlacementV1,
) -> Result<MoleculeInsertionV1, PeptideTemplateMoleculeBuildErrorV1> {
    let parsed = engine
        .smiles_to_molecule(request.smiles())
        .map_err(SmilesMoleculeBuildError::from)?;
    let mut graph = parsed.molecule().clone();
    validate_supported_peptide_template_complete_graph_facts_v1(&graph)
        .map_err(SmilesMoleculeBuildError::from)?;
    if graph
        .atoms()
        .iter()
        .any(ferrum_chemistry::MolAtom::is_aromatic)
        || graph
            .bonds()
            .iter()
            .any(ferrum_chemistry::MolBond::is_aromatic)
    {
        let options = ferrum_chemistry::KekulizeOptions::new(true, true, 100)
            .map_err(SmilesMoleculeBuildError::from)?;
        graph = engine
            .kekulize(&graph, options)
            .map_err(SmilesMoleculeBuildError::from)?;
        validate_supported_peptide_template_complete_graph_facts_v1(&graph)
            .map_err(SmilesMoleculeBuildError::from)?;
    }
    build_complete_graph_molecule_insertion_from_validated_facts_v1(&graph, placement)
        .map_err(SmilesMoleculeBuildError::from)
        .map_err(PeptideTemplateMoleculeBuildErrorV1::Build)
}

/// Native-engine stage failure after successful strict template preflight.
#[derive(Debug, Error)]
pub enum PeptideTemplateMoleculeBuildErrorV1 {
    /// The frozen SMILES insertion stage rejected the compiled request.
    #[error(transparent)]
    Build(#[from] SmilesMoleculeBuildError),
}
