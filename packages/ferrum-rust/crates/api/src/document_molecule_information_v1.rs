//! Exact-observation source facts plus native perceived composition.

use ferrum_chemistry::{
    ChemEngine, ChemistryError, CompositionAggregationError, MolGraph, MoleculeComposition,
};
use ferrum_document::{DocumentObjectIdV1, SessionDocumentObservationV1, TypedDocument};
use thiserror::Error;

use crate::document_molecule_composition_graph_v1::{
    DocumentMoleculeCompositionGraphErrorV1, document_molecule_composition_graph_v1,
};
use crate::document_molecule_inspection_v1::{
    DocumentMoleculeInspectionErrorV1, DocumentMoleculeInspectionV1,
    build_document_molecule_inspection_v1, direct_projection_molecule_v1,
    verify_molecule_observation_v1,
};

/// Stable schema identifier for complete molecule-information receipts.
pub const DOCUMENT_MOLECULE_INFORMATION_SCHEMA_V1: &str = "ferrum-document-molecule-information-v1";

/// Exact observation and unique durable direct-root selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeInformationRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_ids: Vec<DocumentObjectIdV1>,
}

impl DocumentMoleculeInformationRequestV1 {
    /// Construct a nonempty selection without silently deduplicating it.
    pub fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_ids: Vec<DocumentObjectIdV1>,
    ) -> Result<Self, DocumentMoleculeInformationRequestErrorV1> {
        if molecule_ids.is_empty() {
            return Err(DocumentMoleculeInformationRequestErrorV1::EmptySelection);
        }
        for (index, molecule_id) in molecule_ids.iter().enumerate() {
            if molecule_ids[..index].contains(molecule_id) {
                return Err(DocumentMoleculeInformationRequestErrorV1::DuplicateMolecule);
            }
        }
        Ok(Self {
            expected_revision,
            expected_digest,
            molecule_ids,
        })
    }

    /// Return the frozen source revision.
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }

    /// Return the frozen source digest.
    #[must_use]
    pub const fn expected_digest(&self) -> &[u8; 32] {
        &self.expected_digest
    }

    /// Return selected durable roots; preparation normalizes document order.
    #[must_use]
    pub fn molecule_ids(&self) -> &[DocumentObjectIdV1] {
        &self.molecule_ids
    }
}

/// A provenance-authenticated, graph-owning operation prepared before native loading.
#[derive(Debug)]
pub struct PreparedDocumentMoleculeInformationV1 {
    source_revision: u64,
    source_digest: [u8; 32],
    records: Vec<PreparedRecordV1>,
}

impl PreparedDocumentMoleculeInformationV1 {
    /// Return the frozen source revision.
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Return the frozen source digest.
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Return the number of exact roots frozen for execution.
    #[must_use]
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[derive(Debug)]
struct PreparedRecordV1 {
    source_facts: DocumentMoleculeInspectionV1,
    graph: MolGraph,
}

/// One source-fact receipt paired with the engine's perceived composition.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentMoleculeInformationRecordV1 {
    source_facts: DocumentMoleculeInspectionV1,
    composition: MoleculeComposition,
}

impl DocumentMoleculeInformationRecordV1 {
    /// Return retained authored facts and durable source identity.
    #[must_use]
    pub const fn source_facts(&self) -> &DocumentMoleculeInspectionV1 {
        &self.source_facts
    }

    /// Return isotope- and charge-aware perceived composition.
    #[must_use]
    pub const fn composition(&self) -> &MoleculeComposition {
        &self.composition
    }
}

/// Complete all-or-nothing molecule-information result.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentMoleculeInformationV1 {
    schema: &'static str,
    source_revision: u64,
    source_digest: [u8; 32],
    records: Vec<DocumentMoleculeInformationRecordV1>,
    combined_selection: Option<MoleculeComposition>,
}

impl DocumentMoleculeInformationV1 {
    /// Return this receipt's stable schema identifier.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Return the frozen source revision.
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Return the frozen source digest.
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Return records in direct document-root order.
    #[must_use]
    pub fn records(&self) -> &[DocumentMoleculeInformationRecordV1] {
        &self.records
    }

    /// Return an aggregate only when two or more roots were selected.
    #[must_use]
    pub const fn combined_selection(&self) -> Option<&MoleculeComposition> {
        self.combined_selection.as_ref()
    }
}

/// Authenticate source state and freeze all graphs before resolving an engine.
pub fn prepare_document_molecule_information_v1(
    observation: &SessionDocumentObservationV1,
    request: &DocumentMoleculeInformationRequestV1,
) -> Result<PreparedDocumentMoleculeInformationV1, DocumentMoleculeInformationErrorV1> {
    verify_molecule_observation_v1(
        observation,
        request.expected_revision,
        &request.expected_digest,
    )?;
    let snapshot = observation.snapshot();
    let projection = observation.projection();
    let document = TypedDocument::parse(snapshot.cdml())
        .map_err(DocumentMoleculeInspectionErrorV1::Document)?;
    let mut records = Vec::new();
    records
        .try_reserve_exact(request.molecule_ids.len())
        .map_err(|_| DocumentMoleculeInformationErrorV1::ResourceAllocation)?;
    for molecule_id in &request.molecule_ids {
        let root = direct_projection_molecule_v1(projection, molecule_id)?;
        let root_source_id = root
            .source_id()
            .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
        let molecule = document
            .core_molecule(molecule_id)
            .map_err(DocumentMoleculeInspectionErrorV1::CoreProjection)?
            .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
        if molecule.source_id().map(ferrum_core::Identifier::as_str) != Some(root_source_id) {
            return Err(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch.into());
        }
        let source_facts = build_document_molecule_inspection_v1(
            snapshot.revision(),
            snapshot.digest(),
            molecule_id,
            root,
            &molecule,
        )?;
        let graph = document_molecule_composition_graph_v1(&molecule)?;
        records.push(PreparedRecordV1 {
            source_facts,
            graph,
        });
    }
    records.sort_by_key(|record| record.source_facts.document_root_order());
    Ok(PreparedDocumentMoleculeInformationV1 {
        source_revision: snapshot.revision(),
        source_digest: *snapshot.digest(),
        records,
    })
}

/// Execute one fully prepared all-or-nothing operation with an explicit engine.
pub fn execute_prepared_document_molecule_information_v1(
    engine: &impl ChemEngine,
    prepared: PreparedDocumentMoleculeInformationV1,
) -> Result<DocumentMoleculeInformationV1, DocumentMoleculeInformationErrorV1> {
    let mut records = Vec::new();
    records
        .try_reserve_exact(prepared.records.len())
        .map_err(|_| DocumentMoleculeInformationErrorV1::ResourceAllocation)?;
    for record in prepared.records {
        let composition = engine.molecule_composition(&record.graph)?;
        records.push(DocumentMoleculeInformationRecordV1 {
            source_facts: record.source_facts,
            composition,
        });
    }
    let combined_selection = if records.len() >= 2 {
        let mut compositions = Vec::new();
        compositions
            .try_reserve_exact(records.len())
            .map_err(|_| DocumentMoleculeInformationErrorV1::ResourceAllocation)?;
        compositions.extend(records.iter().map(|record| &record.composition));
        Some(MoleculeComposition::combine(&compositions)?)
    } else {
        None
    };
    Ok(DocumentMoleculeInformationV1 {
        schema: DOCUMENT_MOLECULE_INFORMATION_SCHEMA_V1,
        source_revision: prepared.source_revision,
        source_digest: prepared.source_digest,
        records,
        combined_selection,
    })
}

/// Rejected request shape before any document or engine work.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DocumentMoleculeInformationRequestErrorV1 {
    /// At least one durable root must be selected.
    #[error("molecule information requires at least one selected molecule")]
    EmptySelection,
    /// Duplicate selectors are ambiguous input rather than extra records.
    #[error("molecule information selection repeats a durable molecule")]
    DuplicateMolecule,
}

/// Failure while preparing or executing exact molecule information.
#[derive(Debug, Error)]
pub enum DocumentMoleculeInformationErrorV1 {
    /// Observation, selection, or retained source facts were rejected.
    #[error(transparent)]
    Inspection(#[from] DocumentMoleculeInspectionErrorV1),
    /// A retained source graph could not cross the closed composition boundary.
    #[error(transparent)]
    Graph(#[from] DocumentMoleculeCompositionGraphErrorV1),
    /// The selected chemistry engine was unavailable or rejected the graph.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// Successful per-root receipts could not form one checked aggregate.
    #[error(transparent)]
    Aggregate(#[from] CompositionAggregationError),
    /// Owned operation or result storage could not be allocated completely.
    #[error("molecule information could not reserve owned storage")]
    ResourceAllocation,
}
