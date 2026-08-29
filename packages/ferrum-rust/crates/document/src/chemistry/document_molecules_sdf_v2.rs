//! Exact-observation batch SDF export for direct document molecule roots.

use crate::{
    DocumentObjectIdV1, InterchangeRecordMetadataErrorV1, SessionDocumentObservationV1,
    TypedDocument, observe_interchange_record_metadata_v1,
};
use ferrum_chemistry::{
    ChemEngine, ChemistryError, MolblockVersion, NATIVE_SDF_MAX_RECORDS, NativeChemEngine,
    NativeTextOutputLimit, SdfError, SdfProperty, SdfRecord, validate_molblock_title,
};
use thiserror::Error;

const DOCUMENT_MOLECULES_SDF_TEXT_LIMIT: NativeTextOutputLimit =
    NativeTextOutputLimit::ADAPTER_MAXIMUM;

use super::document_molecule_graph_v1::{
    DocumentMoleculeGraphError, document_molecule_coordinate_graph_v1,
};
use super::document_molecule_inspection_v1::{
    DocumentMoleculeInspectionErrorV1, copied_object_id, direct_projection_molecule_v1,
    verify_molecule_observation_v1,
};
use crate::document_direct_root_index_v1::document_direct_root_paint_orders_v1;

/// Stable schema for a batch direct-root SDF receipt.
pub const DOCUMENT_MOLECULES_SDF_SCHEMA_V2: &str = "ferrum-document-molecules-sdf-v2";
/// Coordinate and record-envelope profile used by batch direct-root SDF export.
pub const DOCUMENT_MOLECULES_SDF_PROFILE_V2: &str =
    "document-xy-to-chemistry-x-minus-y-rust-native-sdf-batch-v2";

/// Immutable exact-observation request for two or more direct-root SDF records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculesSdfRequestV2 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_ids: Vec<DocumentObjectIdV1>,
    version: MolblockVersion,
}

impl DocumentMoleculesSdfRequestV2 {
    /// Construct a nonempty, distinct direct-root selection without reordering it.
    pub fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_ids: Vec<DocumentObjectIdV1>,
        version: MolblockVersion,
    ) -> Result<Self, DocumentMoleculesSdfRequestErrorV2> {
        if molecule_ids.len() < 2 {
            return Err(DocumentMoleculesSdfRequestErrorV2::InsufficientSelection);
        }
        if molecule_ids.len() > NATIVE_SDF_MAX_RECORDS {
            return Err(DocumentMoleculesSdfRequestErrorV2::RecordLimit {
                limit: NATIVE_SDF_MAX_RECORDS,
            });
        }
        for (index, molecule_id) in molecule_ids.iter().enumerate() {
            if molecule_ids[..index].contains(molecule_id) {
                return Err(DocumentMoleculesSdfRequestErrorV2::DuplicateMolecule);
            }
        }
        Ok(Self {
            expected_revision,
            expected_digest,
            molecule_ids,
            version,
        })
    }

    /// Return the revision that must still own every selected root.
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }

    /// Return the digest that must still own every selected root.
    #[must_use]
    pub const fn expected_digest(&self) -> &[u8; 32] {
        &self.expected_digest
    }

    /// Return the selected direct-root selectors in request order.
    #[must_use]
    pub fn molecule_ids(&self) -> &[DocumentObjectIdV1] {
        &self.molecule_ids
    }

    /// Return the explicit Molfile syntax for every record in the batch.
    #[must_use]
    pub const fn version(&self) -> MolblockVersion {
        self.version
    }
}

/// Native-handle-free SDF records frozen before loading the native engine.
#[derive(Debug)]
pub struct PreparedDocumentMoleculesSdfV2 {
    source_revision: u64,
    source_digest: [u8; 32],
    version: MolblockVersion,
    records: Vec<PreparedDocumentMoleculesSdfRecordV2>,
}

impl PreparedDocumentMoleculesSdfV2 {
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

    /// Return the number of prepared records in canonical direct-root order.
    #[must_use]
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[derive(Debug)]
struct PreparedDocumentMoleculesSdfRecordV2 {
    molecule_id: DocumentObjectIdV1,
    record: SdfRecord,
}

/// One canonical direct-root record retained in a completed batch receipt.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentMoleculesSdfRecordV2 {
    molecule_id: DocumentObjectIdV1,
    title: String,
    properties: Vec<SdfProperty>,
}

impl DocumentMoleculesSdfRecordV2 {
    /// Return the canonical durable direct-root ID.
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }

    /// Return the exact effective record title, including empty text.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Return exact ordered properties retained for this root.
    #[must_use]
    pub fn properties(&self) -> &[SdfProperty] {
        &self.properties
    }
}

/// One complete, all-or-nothing SDF batch bound to one source observation.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentMoleculesSdfV2 {
    schema: &'static str,
    source_revision: u64,
    source_digest: [u8; 32],
    profile: &'static str,
    version: MolblockVersion,
    records: Vec<DocumentMoleculesSdfRecordV2>,
    sdf: String,
}

impl DocumentMoleculesSdfV2 {
    /// Return the stable receipt schema.
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

    /// Return the closed coordinate and native batch profile.
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        self.profile
    }

    /// Return the explicit Molfile syntax used for every record.
    #[must_use]
    pub const fn version(&self) -> MolblockVersion {
        self.version
    }

    /// Return records in canonical direct-root projection order.
    #[must_use]
    pub fn records(&self) -> &[DocumentMoleculesSdfRecordV2] {
        &self.records
    }

    /// Return the number of records represented by the native SDF text.
    #[must_use]
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Return the complete native SDF batch text.
    #[must_use]
    pub fn sdf(&self) -> &str {
        &self.sdf
    }
}

/// Authenticate all requested direct roots and prepare every native SDF record.
pub fn prepare_document_molecules_sdf_v2(
    observation: &SessionDocumentObservationV1,
    request: &DocumentMoleculesSdfRequestV2,
) -> Result<PreparedDocumentMoleculesSdfV2, DocumentMoleculesSdfErrorV2> {
    verify_molecule_observation_v1(
        observation,
        request.expected_revision,
        &request.expected_digest,
    )?;
    let snapshot = observation.snapshot();
    let document = TypedDocument::parse(snapshot.cdml())
        .map_err(DocumentMoleculeInspectionErrorV1::Document)?;
    let document_paint_orders = document_direct_root_paint_orders_v1(observation.projection())
        .map_err(|_| DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
    let mut sources = Vec::new();
    sources
        .try_reserve_exact(request.molecule_ids.len())
        .map_err(|_| DocumentMoleculesSdfErrorV2::ResourceAllocation)?;
    for molecule_id in &request.molecule_ids {
        let root = direct_projection_molecule_v1(observation.projection(), molecule_id)?;
        let source_id = root
            .source_id()
            .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
        let molecule = document
            .core_molecule(molecule_id)
            .map_err(DocumentMoleculeInspectionErrorV1::CoreProjection)?
            .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
        if molecule.source_id().as_str() != source_id {
            return Err(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch.into());
        }
        sources.push(ResolvedDocumentMoleculeSdfSourceV2 {
            molecule_id: copied_object_id(molecule_id)?,
            source_id: copy_text(source_id)?,
            document_paint_order: *document_paint_orders
                .get(molecule_id)
                .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?,
            molecule,
        });
    }
    sources.sort_by_key(|source| source.document_paint_order);

    let mut records = Vec::new();
    records
        .try_reserve_exact(sources.len())
        .map_err(|_| DocumentMoleculesSdfErrorV2::ResourceAllocation)?;
    for source in sources {
        let metadata = observe_interchange_record_metadata_v1(&document, &source.source_id)?;
        let (title, metadata_properties) = match metadata {
            Some(metadata) => metadata.into_parts(),
            None => (copy_text(source.molecule.name().unwrap_or(""))?, Vec::new()),
        };
        validate_molblock_title(&title)?;
        let mut properties = Vec::new();
        properties
            .try_reserve_exact(metadata_properties.len())
            .map_err(|_| DocumentMoleculesSdfErrorV2::ResourceAllocation)?;
        for property in metadata_properties {
            let (name, value) = property.into_parts();
            properties.push(SdfProperty::new(name, value)?);
        }
        let molecule = document_molecule_coordinate_graph_v1(&source.molecule)?
            .into_parts()
            .0;
        records.push(PreparedDocumentMoleculesSdfRecordV2 {
            molecule_id: source.molecule_id,
            record: SdfRecord::new(molecule, title, properties)?,
        });
    }
    Ok(PreparedDocumentMoleculesSdfV2 {
        source_revision: snapshot.revision(),
        source_digest: *snapshot.digest(),
        version: request.version,
        records,
    })
}

/// Resolve CLI-facing direct source IDs inside the authenticated document boundary.
///
/// This entry point deliberately accepts source IDs rather than exposing projection-local
/// object-ID construction to an outer transport.  The resulting prepared receipt still
/// retains canonical document IDs and canonical direct-root order.
pub fn prepare_document_molecules_sdf_from_source_ids_v2(
    observation: &SessionDocumentObservationV1,
    expected_revision: u64,
    expected_digest: [u8; 32],
    source_ids: &[String],
    version: MolblockVersion,
) -> Result<PreparedDocumentMoleculesSdfV2, DocumentMoleculesSdfErrorV2> {
    if source_ids.len() < 2 {
        return Err(DocumentMoleculesSdfRequestErrorV2::InsufficientSelection.into());
    }
    if source_ids.len() > NATIVE_SDF_MAX_RECORDS {
        return Err(DocumentMoleculesSdfRequestErrorV2::RecordLimit {
            limit: NATIVE_SDF_MAX_RECORDS,
        }
        .into());
    }
    for (index, source_id) in source_ids.iter().enumerate() {
        if source_ids[..index].contains(source_id) {
            return Err(DocumentMoleculesSdfRequestErrorV2::DuplicateMolecule.into());
        }
    }
    verify_molecule_observation_v1(observation, expected_revision, &expected_digest)?;
    let mut molecule_ids = Vec::new();
    molecule_ids
        .try_reserve_exact(source_ids.len())
        .map_err(|_| DocumentMoleculesSdfErrorV2::ResourceAllocation)?;
    for source_id in source_ids {
        let mut roots = observation
            .projection()
            .molecules()
            .iter()
            .filter(|root| root.source_id() == Some(source_id.as_str()));
        let root = roots
            .next()
            .ok_or(DocumentMoleculesSdfErrorV2::UnknownSourceId)?;
        if roots.next().is_some() {
            return Err(DocumentMoleculesSdfErrorV2::AmbiguousSourceId);
        }
        molecule_ids.push(copied_object_id(root.document_object_id())?);
    }
    let request = DocumentMoleculesSdfRequestV2::new(
        expected_revision,
        expected_digest,
        molecule_ids,
        version,
    )?;
    prepare_document_molecules_sdf_v2(observation, &request)
}

/// Execute one fully prepared batch with exactly one native SDF writer call.
pub fn export_prepared_document_molecules_sdf_v2(
    engine: &NativeChemEngine,
    prepared: PreparedDocumentMoleculesSdfV2,
) -> Result<DocumentMoleculesSdfV2, DocumentMoleculesSdfErrorV2> {
    export_with_engine(engine, prepared)
}

fn export_with_engine(
    engine: &impl ChemEngine,
    prepared: PreparedDocumentMoleculesSdfV2,
) -> Result<DocumentMoleculesSdfV2, DocumentMoleculesSdfErrorV2> {
    let mut native_records = Vec::new();
    native_records
        .try_reserve_exact(prepared.records.len())
        .map_err(|_| DocumentMoleculesSdfErrorV2::ResourceAllocation)?;
    let mut records = Vec::new();
    records
        .try_reserve_exact(prepared.records.len())
        .map_err(|_| DocumentMoleculesSdfErrorV2::ResourceAllocation)?;
    for prepared_record in prepared.records {
        native_records.push(prepared_record.record.clone());
        records.push(DocumentMoleculesSdfRecordV2 {
            molecule_id: prepared_record.molecule_id,
            title: copy_text(prepared_record.record.title())?,
            properties: prepared_record.record.properties().to_vec(),
        });
    }
    let sdf = engine.records_to_sdf(
        &native_records,
        prepared.version,
        DOCUMENT_MOLECULES_SDF_TEXT_LIMIT,
    )?;
    Ok(DocumentMoleculesSdfV2 {
        schema: DOCUMENT_MOLECULES_SDF_SCHEMA_V2,
        source_revision: prepared.source_revision,
        source_digest: prepared.source_digest,
        profile: DOCUMENT_MOLECULES_SDF_PROFILE_V2,
        version: prepared.version,
        records,
        sdf,
    })
}

struct ResolvedDocumentMoleculeSdfSourceV2 {
    molecule_id: DocumentObjectIdV1,
    source_id: String,
    document_paint_order: u32,
    molecule: ferrum_core::Molecule,
}

fn copy_text(value: &str) -> Result<String, DocumentMoleculesSdfErrorV2> {
    let mut copied = String::new();
    copied
        .try_reserve_exact(value.len())
        .map_err(|_| DocumentMoleculesSdfErrorV2::ResourceAllocation)?;
    copied.push_str(value);
    Ok(copied)
}

/// Rejected batch request shape before observation lookup or native execution.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DocumentMoleculesSdfRequestErrorV2 {
    /// Batch SDF export intentionally requires more than the V1 single-root contract.
    #[error("batch SDF export requires at least two selected direct molecules")]
    InsufficientSelection,
    /// Repeated roots would make one all-or-nothing receipt ambiguous.
    #[error("batch SDF export selection repeats a durable molecule")]
    DuplicateMolecule,
    /// The selection exceeds the native adapter's bounded SDF request contract.
    #[error("batch SDF export selection exceeds the native record limit of {limit}")]
    RecordLimit { limit: usize },
}

/// Failure while authenticating, preparing, or exporting a direct-root SDF batch.
#[derive(Debug, Error)]
pub enum DocumentMoleculesSdfErrorV2 {
    /// The source-ID request shape was rejected before document lookup or native execution.
    #[error(transparent)]
    Request(#[from] DocumentMoleculesSdfRequestErrorV2),
    /// Observation provenance or direct-root identity was rejected.
    #[error(transparent)]
    Observation(#[from] DocumentMoleculeInspectionErrorV1),
    /// Persisted interchange metadata was absent from its closed grammar.
    #[error(transparent)]
    Metadata(#[from] InterchangeRecordMetadataErrorV1),
    /// No direct projection root carries one requested CLI source ID.
    #[error("batch SDF export source ID does not identify a direct molecule root")]
    UnknownSourceId,
    /// A source ID matched more than one direct projection root.
    #[error("batch SDF export source ID ambiguously identifies direct molecule roots")]
    AmbiguousSourceId,
    /// Retained graph facts cannot cross the exact native SDF boundary.
    #[error("document molecule cannot cross the native SDF structure boundary: {0}")]
    UnsupportedMolecule(#[from] DocumentMoleculeGraphError),
    /// Persisted record facts cannot be represented by the native SDF contract.
    #[error(transparent)]
    Sdf(#[from] SdfError),
    /// The packaged native writer was unavailable or rejected the complete batch.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// Owned operation or receipt storage could not be allocated completely.
    #[error("batch document SDF export could not reserve owned storage")]
    ResourceAllocation,
}

#[cfg(test)]
#[path = "document_molecules_sdf_v2_tests.rs"]
mod tests;
