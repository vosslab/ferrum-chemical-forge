//! Bounded read-only molecule report over one authenticated observation.
//!
//! This module is deliberately private to the operation protocol.  It owns the
//! report graph only until the trusted runtime callback returns; callers receive
//! the protocol DTO assembled in `dto.rs`, never a chemistry capability.

use ferrum_chemistry::{
    AtomicNumber, ChemEngine, ChemistryError, CompositionAggregationError, MolGraph,
    MoleculeComposition,
};
use ferrum_core::Molecule;
use ferrum_domain::{MoleculeDiagnosticRecoveryV1, diagnose_molecule_representation_v1};
use thiserror::Error;

use ferrum_document::{
    DocumentBondCapacityErrorV1, DocumentBondCapacityOutcomeV1, DocumentBondCapacityRequestV1,
    DocumentBondPresentationV1, DocumentDoubleBondCarrierMarkV1, DocumentDoubleBondConfigurationV1,
    DocumentMoleculeInspectionErrorV1, DocumentObjectIdV1, DocumentStereoDepictionReportV1,
    DocumentStereoLigandV1, DocumentStereoSemanticReportV1, DocumentTetrahedralParityV1,
    MoleculeProjectionV1, SessionDocumentObservationV1, TypedDocument,
    direct_projection_molecule_v1, document_direct_root_paint_orders_v1,
    document_molecule_composition_graph_v1, verify_molecule_observation_v1,
};

use super::dto::{
    DocumentMoleculeReportAggregateOmissionReasonSummaryV1,
    DocumentMoleculeReportAggregateOutcomeSummaryV1, DocumentMoleculeReportRequestV1,
    OperationProtocolOutcomeV1,
};
use super::execution::ExecutionFailureV1;
use super::frozen_document_snapshot_v1::FrozenDocumentSnapshotV1;
use super::molecule_report_diagnostics_v1::{
    MoleculeReportDiagnosticsErrorV1, append_composition_unavailable_finding_v1,
    append_graph_finding_v1, collect_report_findings_v1, finding_recovery_summary_v1,
    map_diagnostic_finding_error_v1,
};
use super::runtime::ChemistryRuntimeV1;

/// Stable report schema identifier.
const DOCUMENT_MOLECULE_REPORT_SCHEMA_V1: &str = "ferrum-document-molecule-report-v1";
/// Maximum selected roots admitted by the report core.
const MAX_MOLECULE_REPORT_SELECTORS_V1: usize = 128;
/// Maximum UTF-8 bytes in one durable selector admitted by the report core.
const MAX_MOLECULE_REPORT_SELECTOR_UTF8_BYTES_V1: usize = 2 * 1024;
/// Maximum document-level findings emitted by one report.
const MAX_MOLECULE_REPORT_DOCUMENT_FINDINGS_V1: usize = 1;

/// Exact direct-root selection fenced to one immutable observation.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ParsedDocumentMoleculeReportRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_ids: Vec<DocumentObjectIdV1>,
}

impl ParsedDocumentMoleculeReportRequestV1 {
    /// Build one nonempty, duplicate-free bounded report request.
    pub fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_ids: Vec<DocumentObjectIdV1>,
    ) -> Result<Self, DocumentMoleculeReportRequestErrorV1> {
        if molecule_ids.is_empty() {
            return Err(DocumentMoleculeReportRequestErrorV1::EmptySelection);
        }
        if molecule_ids.len() > MAX_MOLECULE_REPORT_SELECTORS_V1 {
            return Err(DocumentMoleculeReportRequestErrorV1::TooManySelectors);
        }
        for (index, id) in molecule_ids.iter().enumerate() {
            if id.as_str().len() > MAX_MOLECULE_REPORT_SELECTOR_UTF8_BYTES_V1 {
                return Err(DocumentMoleculeReportRequestErrorV1::SelectorTooLong);
            }
            if molecule_ids[..index].contains(id) {
                return Err(DocumentMoleculeReportRequestErrorV1::DuplicateMolecule);
            }
        }
        Ok(Self {
            expected_revision,
            expected_digest,
            molecule_ids,
        })
    }
}

/// Source facts that remain useful even when optional chemistry cannot run.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DocumentMoleculeReportSourceV1 {
    molecule_id: DocumentObjectIdV1,
    document_paint_order: u32,
    authored_name: Option<String>,
    atom_count: usize,
    bond_count: usize,
    authored_elements: Vec<DocumentMoleculeReportElementCountV1>,
    authored_charge: Option<i64>,
    stereo_semantics: Option<DocumentStereoSemanticReportV1>,
    stereo_depiction: Option<DocumentStereoDepictionReportV1>,
}
/// One canonical authored-element count retained from an authenticated root.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DocumentMoleculeReportElementCountV1 {
    symbol: String,
    atom_count: usize,
}
impl DocumentMoleculeReportElementCountV1 {
    #[must_use]
    fn symbol(&self) -> &str {
        &self.symbol
    }
    #[must_use]
    const fn atom_count(&self) -> usize {
        self.atom_count
    }
}
impl DocumentMoleculeReportSourceV1 {
    #[must_use]
    fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
    #[must_use]
    const fn document_paint_order(&self) -> u32 {
        self.document_paint_order
    }
    #[must_use]
    fn authored_name(&self) -> Option<&str> {
        self.authored_name.as_deref()
    }
    #[must_use]
    const fn atom_count(&self) -> usize {
        self.atom_count
    }
    #[must_use]
    const fn bond_count(&self) -> usize {
        self.bond_count
    }
    #[must_use]
    fn authored_elements(&self) -> &[DocumentMoleculeReportElementCountV1] {
        &self.authored_elements
    }
    #[must_use]
    const fn authored_charge(&self) -> Option<i64> {
        self.authored_charge
    }
    #[must_use]
    fn stereo_semantics(&self) -> Option<&DocumentStereoSemanticReportV1> {
        self.stereo_semantics.as_ref()
    }
    #[must_use]
    fn stereo_depiction(&self) -> Option<&DocumentStereoDepictionReportV1> {
        self.stereo_depiction.as_ref()
    }
}

/// One fully prepared report input. This capability and its graphs never cross this module.
struct PreparedDocumentMoleculeReportV1 {
    source_revision: u64,
    source_digest: [u8; 32],
    records: Vec<PreparedReportRecordV1>,
}
struct PreparedReportRecordV1 {
    source: DocumentMoleculeReportSourceV1,
    graph: Option<MolGraph>,
    capacity: DocumentBondCapacityOutcomeV1,
    findings: Vec<super::dto::DocumentMoleculeReportFindingSummaryV1>,
}
/// One source-ordered report record.
#[derive(Clone, Debug, PartialEq)]
struct DocumentMoleculeReportRecordV1 {
    source: DocumentMoleculeReportSourceV1,
    composition: Option<MoleculeComposition>,
    neutral_bond_capacity: DocumentBondCapacityOutcomeV1,
    findings: Vec<super::dto::DocumentMoleculeReportFindingSummaryV1>,
}
impl DocumentMoleculeReportRecordV1 {
    #[must_use]
    const fn source(&self) -> &DocumentMoleculeReportSourceV1 {
        &self.source
    }
    #[must_use]
    const fn composition(&self) -> Option<&MoleculeComposition> {
        self.composition.as_ref()
    }
    #[must_use]
    const fn neutral_bond_capacity(&self) -> &DocumentBondCapacityOutcomeV1 {
        &self.neutral_bond_capacity
    }
    #[must_use]
    fn findings(&self) -> &[super::dto::DocumentMoleculeReportFindingSummaryV1] {
        &self.findings
    }
}

/// Immutable report receipt. It carries no CDML, toolkit graph, or mutation handle.
#[derive(Clone, Debug, PartialEq)]
struct DocumentMoleculeReportV1 {
    schema: &'static str,
    source_revision: u64,
    source_digest: [u8; 32],
    records: Vec<DocumentMoleculeReportRecordV1>,
    combined_composition: Option<MoleculeComposition>,
    document_findings: Vec<DocumentMoleculeReportDocumentFindingV1>,
}
impl DocumentMoleculeReportV1 {
    #[must_use]
    const fn schema(&self) -> &'static str {
        self.schema
    }
    #[cfg(test)]
    #[must_use]
    const fn source_revision(&self) -> u64 {
        self.source_revision
    }
    #[cfg(test)]
    #[must_use]
    const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }
    #[must_use]
    fn records(&self) -> &[DocumentMoleculeReportRecordV1] {
        &self.records
    }
    #[must_use]
    const fn combined_composition(&self) -> Option<&MoleculeComposition> {
        self.combined_composition.as_ref()
    }
    /// Return bounded document-level facts that apply to the selected collection.
    #[must_use]
    fn document_findings(&self) -> &[DocumentMoleculeReportDocumentFindingV1] {
        &self.document_findings
    }
}

/// Closed document-level fact vocabulary for a molecule report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DocumentMoleculeReportDocumentFindingCodeV1 {
    /// Ferrum deliberately did not produce a partial combined composition.
    CombinedCompositionNotAttempted,
}

/// Exact reason that a selected collection lacks one combined composition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DocumentMoleculeReportAggregateOmissionReasonV1 {
    /// A combined composition is meaningful only for two or more selected roots.
    FewerThanTwoSelected,
    /// At least one selected root has no complete composition receipt.
    IncompleteRecordComposition,
}

/// One bounded collection-level report finding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DocumentMoleculeReportDocumentFindingV1 {
    code: DocumentMoleculeReportDocumentFindingCodeV1,
    aggregate_omission_reason: DocumentMoleculeReportAggregateOmissionReasonV1,
    recovery: MoleculeDiagnosticRecoveryV1,
}

impl DocumentMoleculeReportDocumentFindingV1 {
    const fn aggregate_omission_reason(&self) -> DocumentMoleculeReportAggregateOmissionReasonV1 {
        self.aggregate_omission_reason
    }
    #[must_use]
    const fn recovery(&self) -> MoleculeDiagnosticRecoveryV1 {
        self.recovery
    }
}

/// Authenticate source state and freeze report inputs before invoking a chemistry engine.
fn prepare_document_molecule_report_v1(
    observation: &SessionDocumentObservationV1,
    request: &ParsedDocumentMoleculeReportRequestV1,
) -> Result<PreparedDocumentMoleculeReportV1, DocumentMoleculeReportErrorV1> {
    let sources = resolve_document_molecule_sources_v1(
        observation,
        request.expected_revision,
        &request.expected_digest,
        &request.molecule_ids,
    )?;
    let snapshot = observation.snapshot();
    let mut records = Vec::new();
    records
        .try_reserve_exact(sources.len())
        .map_err(|_| DocumentMoleculeReportErrorV1::ResourceAllocation)?;
    for source in sources {
        let representation_findings = diagnose_molecule_representation_v1(&source.molecule)
            .map_err(map_diagnostic_finding_error_v1)
            .map_err(map_diagnostics_error)?;
        let capacity = evaluate_capacity(observation, request, &source.source.molecule_id)?;
        let mut findings =
            collect_report_findings_v1(&source.molecule, &representation_findings, &capacity)
                .map_err(map_diagnostics_error)?;
        let graph = match document_molecule_composition_graph_v1(&source.molecule) {
            Ok(graph) => Some(graph),
            Err(error) => {
                append_graph_finding_v1(&mut findings, &source.molecule, &error)
                    .map_err(map_diagnostics_error)?;
                None
            }
        };
        records.push(PreparedReportRecordV1 {
            source: source.source,
            graph,
            capacity,
            findings,
        });
    }
    Ok(PreparedDocumentMoleculeReportV1 {
        source_revision: snapshot.revision(),
        source_digest: *snapshot.digest(),
        records,
    })
}

/// Evaluate prepared chemistry facets. Unsupported records remain records; engine failure refuses completion.
fn execute_prepared_document_molecule_report_v1(
    engine: &(impl ChemEngine + ?Sized),
    prepared: PreparedDocumentMoleculeReportV1,
) -> Result<DocumentMoleculeReportV1, DocumentMoleculeReportErrorV1> {
    let mut records = Vec::new();
    records
        .try_reserve_exact(prepared.records.len())
        .map_err(|_| DocumentMoleculeReportErrorV1::ResourceAllocation)?;
    for mut record in prepared.records {
        let composition = match record.graph.as_ref() {
            Some(graph) => match engine.molecule_composition(graph) {
                Ok(composition) => Some(composition),
                Err(error) => return Err(map_chemistry_error(error)),
            },
            None => None,
        };
        if composition.is_none() {
            append_composition_unavailable_finding_v1(&mut record.findings)
                .map_err(map_diagnostics_error)?;
        }
        records.push(DocumentMoleculeReportRecordV1 {
            source: record.source,
            composition,
            neutral_bond_capacity: record.capacity,
            findings: record.findings,
        });
    }
    let (combined_composition, document_findings) =
        if records.len() >= 2 && records.iter().all(|record| record.composition.is_some()) {
            let mut values = Vec::new();
            values
                .try_reserve_exact(records.len())
                .map_err(|_| DocumentMoleculeReportErrorV1::ResourceAllocation)?;
            values.extend(
                records
                    .iter()
                    .filter_map(|record| record.composition.as_ref()),
            );
            (
                Some(MoleculeComposition::combine(&values).map_err(map_aggregate_error)?),
                Vec::new(),
            )
        } else {
            let reason = if records.len() < 2 {
                DocumentMoleculeReportAggregateOmissionReasonV1::FewerThanTwoSelected
            } else {
                DocumentMoleculeReportAggregateOmissionReasonV1::IncompleteRecordComposition
            };
            let recovery = match reason {
                DocumentMoleculeReportAggregateOmissionReasonV1::FewerThanTwoSelected => {
                    MoleculeDiagnosticRecoveryV1::None
                }
                DocumentMoleculeReportAggregateOmissionReasonV1::IncompleteRecordComposition => {
                    MoleculeDiagnosticRecoveryV1::ChooseSupportedRepresentation
                }
            };
            let mut document_findings = Vec::new();
            document_findings
                .try_reserve_exact(MAX_MOLECULE_REPORT_DOCUMENT_FINDINGS_V1)
                .map_err(|_| DocumentMoleculeReportErrorV1::ResourceAllocation)?;
            document_findings.push(DocumentMoleculeReportDocumentFindingV1 {
                code: DocumentMoleculeReportDocumentFindingCodeV1::CombinedCompositionNotAttempted,
                aggregate_omission_reason: reason,
                recovery,
            });
            (None, document_findings)
        };
    Ok(DocumentMoleculeReportV1 {
        schema: DOCUMENT_MOLECULE_REPORT_SCHEMA_V1,
        source_revision: prepared.source_revision,
        source_digest: prepared.source_digest,
        records,
        combined_composition,
        document_findings,
    })
}

/// Shared authenticated direct-root resolver. It parses the exact snapshot once and owns results.
fn resolve_document_molecule_sources_v1(
    observation: &SessionDocumentObservationV1,
    expected_revision: u64,
    expected_digest: &[u8; 32],
    molecule_ids: &[DocumentObjectIdV1],
) -> Result<Vec<ResolvedDocumentMoleculeSourceV1>, DocumentMoleculeReportErrorV1> {
    verify_molecule_observation_v1(observation, expected_revision, expected_digest)
        .map_err(map_inspection_error)?;
    let snapshot = observation.snapshot();
    let projection = observation.projection();
    let document = TypedDocument::parse(snapshot.cdml())
        .map_err(DocumentMoleculeInspectionErrorV1::Document)
        .map_err(map_inspection_error)?;
    let document_paint_orders = document_direct_root_paint_orders_v1(projection).map_err(|_| {
        map_inspection_error(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)
    })?;
    let mut sources = Vec::new();
    sources
        .try_reserve_exact(molecule_ids.len())
        .map_err(|_| DocumentMoleculeReportErrorV1::ResourceAllocation)?;
    for molecule_id in molecule_ids {
        let root =
            direct_projection_molecule_v1(projection, molecule_id).map_err(map_inspection_error)?;
        let source_id = root
            .source_id()
            .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)
            .map_err(map_inspection_error)?;
        let molecule = document
            .core_molecule(molecule_id)
            .map_err(DocumentMoleculeInspectionErrorV1::CoreProjection)
            .map_err(map_inspection_error)?
            .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)
            .map_err(map_inspection_error)?;
        let stereo_semantics = document
            .molecule_stereo_semantics_v1(molecule_id)
            .map_err(DocumentMoleculeInspectionErrorV1::Document)
            .map_err(map_inspection_error)?;
        let stereo_depiction = observation
            .molecule_stereo_depictions_v1(molecule_id)
            .map_err(DocumentMoleculeInspectionErrorV1::Document)
            .map_err(map_inspection_error)?;
        if molecule.source_id().as_str() != source_id {
            return Err(map_inspection_error(
                DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch,
            ));
        }
        sources.push(ResolvedDocumentMoleculeSourceV1 {
            source: source_facts(
                molecule_id,
                *document_paint_orders
                    .get(molecule_id)
                    .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)
                    .map_err(map_inspection_error)?,
                root,
                &molecule,
                stereo_semantics,
                stereo_depiction,
            )?,
            molecule,
        });
    }
    sources.sort_by_key(|source| source.source.document_paint_order);
    Ok(sources)
}

fn source_facts(
    molecule_id: &DocumentObjectIdV1,
    document_paint_order: u32,
    root: &MoleculeProjectionV1,
    molecule: &Molecule,
    stereo_semantics: Option<DocumentStereoSemanticReportV1>,
    stereo_depiction: Option<DocumentStereoDepictionReportV1>,
) -> Result<DocumentMoleculeReportSourceV1, DocumentMoleculeReportErrorV1> {
    let mut inventory: Vec<(&str, usize)> = Vec::new();
    inventory
        .try_reserve_exact(molecule.atoms().len())
        .map_err(|_| DocumentMoleculeReportErrorV1::ResourceAllocation)?;
    let mut charge = Some(0_i64);
    for atom in molecule.atoms() {
        if let Some(element) = atom
            .element()
            .and_then(|value| AtomicNumber::from_symbol(value).ok())
        {
            let symbol = element.symbol();
            if let Some((_, count)) = inventory.iter_mut().find(|(present, _)| *present == symbol) {
                *count += 1;
            } else {
                inventory.push((symbol, 1));
            }
        }
        charge = match (charge, atom.formal_charge()) {
            (Some(total), Some(value)) => total.checked_add(i64::from(value)),
            _ => None,
        };
    }
    inventory.sort_unstable_by_key(|(symbol, _)| *symbol);
    let mut authored_elements = Vec::new();
    authored_elements
        .try_reserve_exact(inventory.len())
        .map_err(|_| DocumentMoleculeReportErrorV1::ResourceAllocation)?;
    for (symbol, atom_count) in inventory {
        authored_elements.push(DocumentMoleculeReportElementCountV1 {
            symbol: copied(symbol)?,
            atom_count,
        });
    }
    Ok(DocumentMoleculeReportSourceV1 {
        molecule_id: copied_id(molecule_id)?,
        document_paint_order,
        authored_name: root.name().map(copied).transpose()?,
        atom_count: molecule.atoms().len(),
        bond_count: molecule.bonds().len(),
        authored_elements,
        authored_charge: charge,
        stereo_semantics,
        stereo_depiction,
    })
}

#[derive(Clone, Debug)]
struct ResolvedDocumentMoleculeSourceV1 {
    source: DocumentMoleculeReportSourceV1,
    molecule: Molecule,
}

fn copied(value: &str) -> Result<String, DocumentMoleculeReportErrorV1> {
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| DocumentMoleculeReportErrorV1::ResourceAllocation)?;
    result.push_str(value);
    Ok(result)
}
fn copied_id(
    value: &DocumentObjectIdV1,
) -> Result<DocumentObjectIdV1, DocumentMoleculeReportErrorV1> {
    DocumentObjectIdV1::parse(copied(value.as_str())?)
        .map_err(|_| DocumentMoleculeReportErrorV1::OpaqueIdInvariant)
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
enum DocumentMoleculeReportRequestErrorV1 {
    #[error("molecule report requires at least one selected molecule")]
    EmptySelection,
    #[error("molecule report selection repeats a durable molecule")]
    DuplicateMolecule,
    #[error("molecule report selector exceeds the V1 byte limit")]
    SelectorTooLong,
    #[error("molecule report selection exceeds the V1 limit")]
    TooManySelectors,
}
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
enum DocumentMoleculeReportErrorV1 {
    #[error("molecule report observation or direct-root resolution was refused")]
    Inspection,
    #[error("molecule report capacity evaluation was refused")]
    Capacity,
    #[error("molecule report chemistry completion was refused")]
    Chemistry,
    #[error("molecule report combined composition could not be completed")]
    Aggregate,
    #[error("molecule report diagnostic construction was refused")]
    Finding,
    #[error("molecule report exceeded the bounded finding limit")]
    FindingLimit,
    #[error("molecule report could not reserve owned storage")]
    ResourceAllocation,
    #[error("validated opaque molecule selector could not be reconstructed")]
    OpaqueIdInvariant,
}

fn map_inspection_error(error: DocumentMoleculeInspectionErrorV1) -> DocumentMoleculeReportErrorV1 {
    if matches!(error, DocumentMoleculeInspectionErrorV1::ResourceAllocation) {
        DocumentMoleculeReportErrorV1::ResourceAllocation
    } else {
        DocumentMoleculeReportErrorV1::Inspection
    }
}

fn map_capacity_error(error: DocumentBondCapacityErrorV1) -> DocumentMoleculeReportErrorV1 {
    if matches!(error, DocumentBondCapacityErrorV1::ResourceAllocation) {
        DocumentMoleculeReportErrorV1::ResourceAllocation
    } else {
        DocumentMoleculeReportErrorV1::Capacity
    }
}

fn map_chemistry_error(error: ChemistryError) -> DocumentMoleculeReportErrorV1 {
    if matches!(error, ChemistryError::ResourceExhausted { .. }) {
        DocumentMoleculeReportErrorV1::ResourceAllocation
    } else {
        DocumentMoleculeReportErrorV1::Chemistry
    }
}

fn map_aggregate_error(error: CompositionAggregationError) -> DocumentMoleculeReportErrorV1 {
    if matches!(error, CompositionAggregationError::ResourceExhausted) {
        DocumentMoleculeReportErrorV1::ResourceAllocation
    } else {
        DocumentMoleculeReportErrorV1::Aggregate
    }
}

fn map_diagnostics_error(error: MoleculeReportDiagnosticsErrorV1) -> DocumentMoleculeReportErrorV1 {
    if matches!(error, MoleculeReportDiagnosticsErrorV1::ResourceAllocation) {
        DocumentMoleculeReportErrorV1::ResourceAllocation
    } else if matches!(error, MoleculeReportDiagnosticsErrorV1::FindingLimit) {
        DocumentMoleculeReportErrorV1::FindingLimit
    } else {
        DocumentMoleculeReportErrorV1::Finding
    }
}

/// Execute one request-scoped report.  The only engine borrow is lexically
/// contained in this private module and its graph-owning prepared input is
/// consumed before a DTO can leave the module.
pub(super) fn execute_document_molecule_report_v1<R: ChemistryRuntimeV1>(
    snapshot: FrozenDocumentSnapshotV1,
    request: DocumentMoleculeReportRequestV1,
    runtime: &R,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let ids = request
        .molecule_ids
        .into_iter()
        .map(|value| {
            DocumentObjectIdV1::parse(value).map_err(|_| {
                ExecutionFailureV1::document_invalid(
                    "molecule_ids must contain durable direct root identifiers".to_owned(),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let observation = snapshot.observation();
    let parsed = ParsedDocumentMoleculeReportRequestV1::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        ids,
    )
    .map_err(|error| ExecutionFailureV1::document_invalid(error.to_string()))?;
    let prepared =
        prepare_document_molecule_report_v1(observation, &parsed).map_err(map_report_error)?;
    let report = runtime
        .with_engine(|engine| {
            Ok(
                execute_prepared_document_molecule_report_v1(engine, prepared)
                    .map_err(map_report_error),
            )
        })
        .map_err(|error| match error {
            super::runtime::ChemistryRuntimeErrorV1::Unavailable => {
                ExecutionFailureV1::chemistry_unavailable(
                    "chemistry runtime is unavailable".to_owned(),
                )
            }
            super::runtime::ChemistryRuntimeErrorV1::Chemistry(_) => {
                ExecutionFailureV1::document_invalid(
                    "molecule report chemistry execution was refused".to_owned(),
                )
            }
        })??;
    Ok(OperationProtocolOutcomeV1::DocumentMoleculeReport {
        report: report_summary_with_source_provenance(
            report,
            snapshot.source_revision(),
            *snapshot.source_digest(),
        ),
    })
}

fn evaluate_capacity(
    observation: &SessionDocumentObservationV1,
    request: &ParsedDocumentMoleculeReportRequestV1,
    molecule_id: &DocumentObjectIdV1,
) -> Result<DocumentBondCapacityOutcomeV1, DocumentMoleculeReportErrorV1> {
    let capacity_request = DocumentBondCapacityRequestV1::new(
        request.expected_revision,
        request.expected_digest,
        vec![molecule_id.clone()],
    )
    .map_err(|_| DocumentMoleculeReportErrorV1::Capacity)?;
    let receipt =
        ferrum_document::inspect_document_bond_capacity_v1(observation, &capacity_request)
            .map_err(map_capacity_error)?;
    receipt
        .records()
        .first()
        .map(|record| record.outcome().clone())
        .ok_or(DocumentMoleculeReportErrorV1::Capacity)
}

fn map_report_error(error: DocumentMoleculeReportErrorV1) -> ExecutionFailureV1 {
    match error {
        DocumentMoleculeReportErrorV1::ResourceAllocation
        | DocumentMoleculeReportErrorV1::FindingLimit => {
            ExecutionFailureV1::resource_limit("molecule report exceeded a bounded resource limit")
        }
        DocumentMoleculeReportErrorV1::Chemistry => ExecutionFailureV1::document_invalid(
            "molecule report chemistry execution was refused".to_owned(),
        ),
        _ => ExecutionFailureV1::document_invalid("molecule report request was refused".to_owned()),
    }
}

#[cfg(test)]
fn report_summary(report: DocumentMoleculeReportV1) -> super::dto::DocumentMoleculeReportSummaryV1 {
    let source_revision = report.source_revision();
    let source_digest = *report.source_digest();
    report_summary_with_source_provenance(report, source_revision, source_digest)
}

fn report_summary_with_source_provenance(
    report: DocumentMoleculeReportV1,
    source_revision: u64,
    source_digest: [u8; 32],
) -> super::dto::DocumentMoleculeReportSummaryV1 {
    let aggregate = match report.combined_composition() {
        Some(composition) => DocumentMoleculeReportAggregateOutcomeSummaryV1::Complete {
            composition: composition_summary(composition),
        },
        None => {
            let finding = report
                .document_findings()
                .first()
                .expect("an omitted aggregate always has one checked omission finding");
            DocumentMoleculeReportAggregateOutcomeSummaryV1::Omitted {
                reason: match finding.aggregate_omission_reason() {
                    DocumentMoleculeReportAggregateOmissionReasonV1::FewerThanTwoSelected => {
                        DocumentMoleculeReportAggregateOmissionReasonSummaryV1::FewerThanTwoSelected
                    }
                    DocumentMoleculeReportAggregateOmissionReasonV1::IncompleteRecordComposition => {
                        DocumentMoleculeReportAggregateOmissionReasonSummaryV1::IncompleteRecordComposition
                    }
                },
                recovery: finding_recovery_summary_v1(finding.recovery()),
            }
        }
    };
    let records = report
        .records()
        .iter()
        .map(|record| super::dto::DocumentMoleculeReportRecordSummaryV1 {
            molecule_id: record.source().molecule_id().as_str().to_owned(),
            document_paint_order: record.source().document_paint_order(),
            authored_name: record.source().authored_name().map(str::to_owned),
            atom_count: record.source().atom_count(),
            bond_count: record.source().bond_count(),
            authored_charge: record.source().authored_charge(),
            authored_elements: record
                .source()
                .authored_elements()
                .iter()
                .map(
                    |element| super::dto::DocumentMoleculeReportElementCountSummaryV1 {
                        symbol: element.symbol().to_owned(),
                        atom_count: element.atom_count(),
                    },
                )
                .collect(),
            composition: record.composition().map(composition_summary),
            neutral_bond_capacity: capacity_name(record.neutral_bond_capacity()).to_owned(),
            stereo_semantics: record.source().stereo_semantics().map(stereo_summary),
            stereo_depiction: record
                .source()
                .stereo_depiction()
                .map(stereo_depiction_summary),
            findings: record.findings().to_vec(),
        })
        .collect();
    super::dto::DocumentMoleculeReportSummaryV1 {
        schema: report.schema().to_owned(),
        source_revision,
        source_digest_hex: source_digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        records,
        aggregate,
    }
}

fn stereo_summary(
    report: &DocumentStereoSemanticReportV1,
) -> super::dto::DocumentMoleculeReportStereoSemanticsSummaryV1 {
    super::dto::DocumentMoleculeReportStereoSemanticsSummaryV1 {
        tetrahedral: report
            .tetrahedral()
            .iter()
            .map(|value| super::dto::DocumentMoleculeReportTetrahedralStereoSummaryV1 {
                center: value.center(),
                ligands: std::array::from_fn(|index| match value.ligands()[index] {
                        DocumentStereoLigandV1::Atom(index) => {
                            super::dto::DocumentMoleculeReportStereoLigandSummaryV1::Atom {
                                index,
                            }
                        }
                        DocumentStereoLigandV1::ExplicitHydrogen => {
                            super::dto::DocumentMoleculeReportStereoLigandSummaryV1::ExplicitHydrogen
                        }
                    }),
                parity: match value.parity() {
                    DocumentTetrahedralParityV1::Clockwise => {
                        super::dto::DocumentMoleculeReportTetrahedralParitySummaryV1::Clockwise
                    }
                    DocumentTetrahedralParityV1::CounterClockwise => {
                        super::dto::DocumentMoleculeReportTetrahedralParitySummaryV1::CounterClockwise
                    }
                },
            })
            .collect(),
        double_bonds: report
            .double_bonds()
            .iter()
            .map(|value| super::dto::DocumentMoleculeReportDoubleBondStereoSummaryV1 {
                bond_index: value.bond_index(),
                start_ligand: value.start_ligand(),
                end_ligand: value.end_ligand(),
                configuration: match value.configuration() {
                    DocumentDoubleBondConfigurationV1::E => {
                        super::dto::DocumentMoleculeReportDoubleBondConfigurationSummaryV1::E
                    }
                    DocumentDoubleBondConfigurationV1::Z => {
                        super::dto::DocumentMoleculeReportDoubleBondConfigurationSummaryV1::Z
                    }
                },
            })
            .collect(),
    }
}

fn stereo_depiction_summary(
    report: &DocumentStereoDepictionReportV1,
) -> super::dto::DocumentMoleculeReportStereoDepictionSummaryV1 {
    super::dto::DocumentMoleculeReportStereoDepictionSummaryV1 {
        directed_bonds: report
            .directed_bonds()
            .iter()
            .map(|value| {
                let (start, end) = value.endpoints();
                super::dto::DocumentMoleculeReportDirectedBondDepictionSummaryV1 {
                    bond_index: value.bond_index(),
                    start,
                    end,
                    presentation: match value.presentation() {
                        DocumentBondPresentationV1::SolidWedge => {
                            super::dto::DocumentMoleculeReportDirectedBondPresentationSummaryV1::SolidWedge
                        }
                        DocumentBondPresentationV1::HashedWedge => {
                            super::dto::DocumentMoleculeReportDirectedBondPresentationSummaryV1::HashedWedge
                        }
                        _ => unreachable!("document stereo depictions admit wedges only"),
                    },
                }
            })
            .collect(),
        double_bond_carrier_marks: report
            .double_bond_carrier_marks()
            .iter()
            .map(|value| super::dto::DocumentMoleculeReportDoubleBondCarrierMarkSummaryV1 {
                double_bond_index: value.double_bond_index(),
                carrier_bond_index: value.carrier_bond_index(),
                mark: match value.mark() {
                    DocumentDoubleBondCarrierMarkV1::Up => {
                        super::dto::DocumentMoleculeReportDoubleBondCarrierMarkKindSummaryV1::Up
                    }
                    DocumentDoubleBondCarrierMarkV1::Down => {
                        super::dto::DocumentMoleculeReportDoubleBondCarrierMarkKindSummaryV1::Down
                    }
                },
            })
            .collect(),
    }
}

/// Map a complete engine receipt into finite, plain protocol facts. The
/// chemistry crate admits only positive finite masses and keeps its count and
/// percentage vectors aligned; this boundary deliberately copies those values
/// rather than exposing the receipt or any toolkit representation.
fn composition_summary(
    composition: &MoleculeComposition,
) -> super::dto::DocumentMoleculeReportCompositionSummaryV1 {
    let elements = composition
        .element_counts()
        .iter()
        .zip(composition.mass_percentages())
        .map(|(count, mass)| {
            debug_assert_eq!(count.key(), mass.key());
            super::dto::DocumentMoleculeReportCompositionElementSummaryV1 {
                symbol: count.key().symbol().to_owned(),
                isotope: count.key().isotope(),
                atom_count: count.count(),
                average_mass_contribution_da: mass.average_mass_contribution(),
                mass_percentage: mass.percentage(),
            }
        })
        .collect();
    super::dto::DocumentMoleculeReportCompositionSummaryV1 {
        formula: composition.formula().to_owned(),
        net_formal_charge: composition.net_formal_charge(),
        average_molecular_weight_da: composition.average_molecular_weight(),
        monoisotopic_mass_da: composition.monoisotopic_mass(),
        elements,
    }
}

fn capacity_name(value: &DocumentBondCapacityOutcomeV1) -> &'static str {
    match value {
        DocumentBondCapacityOutcomeV1::WithinCapacity { .. } => "within_capacity",
        DocumentBondCapacityOutcomeV1::ExceedsCapacity { .. } => "exceeds_capacity",
        DocumentBondCapacityOutcomeV1::NotChecked { .. } => "not_checked",
    }
}

#[cfg(test)]
#[path = "molecule_report_core_v1_tests.rs"]
mod tests;
