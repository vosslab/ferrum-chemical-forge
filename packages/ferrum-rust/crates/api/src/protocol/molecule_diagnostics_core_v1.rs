//! Bounded, runtime-free structure diagnostics for authenticated direct roots.

use ferrum_document::{
    DocumentBondCapacityErrorV1, DocumentBondCapacityRequestV1, DocumentMoleculeInspectionErrorV1,
    DocumentObjectIdV1, SessionDocumentObservationV1, TypedDocument, direct_projection_molecule_v1,
    document_direct_root_paint_orders_v1, document_molecule_composition_graph_v1,
    inspect_document_bond_capacity_v1, verify_molecule_observation_v1,
};
use ferrum_domain::diagnose_molecule_representation_v1;
use thiserror::Error;

use super::dto::{
    DocumentMoleculeDiagnosticRecordSummaryV1, DocumentMoleculeDiagnosticsRequestV1,
    DocumentMoleculeDiagnosticsSummaryV1, OperationProtocolOutcomeV1,
};
use super::execution::{ExecutionFailureV1, hex_digest};
use super::frozen_document_snapshot_v1::FrozenDocumentSnapshotV1;
use super::molecule_report_diagnostics_v1::{
    MoleculeReportDiagnosticsErrorV1, append_graph_finding_v1, collect_report_findings_v1,
    map_diagnostic_finding_error_v1,
};

const DOCUMENT_MOLECULE_DIAGNOSTICS_SCHEMA_V1: &str = "ferrum-document-molecule-diagnostics-v1";
const MAX_MOLECULE_DIAGNOSTIC_SELECTORS_V1: usize = 128;
const MAX_MOLECULE_DIAGNOSTIC_SELECTOR_UTF8_BYTES_V1: usize = 2 * 1024;

/// Execute one source-fenced diagnostic request without acquiring chemistry runtime.
pub(super) fn execute_document_molecule_diagnostics_v1(
    snapshot: FrozenDocumentSnapshotV1,
    request: DocumentMoleculeDiagnosticsRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let molecule_ids = parse_molecule_ids(request.molecule_ids)?;
    let observation = snapshot.observation();
    let records = collect_records(observation, &molecule_ids).map_err(map_diagnostics_error)?;
    Ok(OperationProtocolOutcomeV1::DocumentMoleculeDiagnostics {
        diagnostics: DocumentMoleculeDiagnosticsSummaryV1 {
            schema: DOCUMENT_MOLECULE_DIAGNOSTICS_SCHEMA_V1.to_owned(),
            source_revision: snapshot.source_revision(),
            source_digest_hex: hex_digest(snapshot.source_digest()),
            records,
        },
    })
}

fn parse_molecule_ids(values: Vec<String>) -> Result<Vec<DocumentObjectIdV1>, ExecutionFailureV1> {
    if values.is_empty() {
        return Err(ExecutionFailureV1::document_invalid(
            "molecule diagnostics requires at least one selected direct root".to_owned(),
        ));
    }
    if values.len() > MAX_MOLECULE_DIAGNOSTIC_SELECTORS_V1 {
        return Err(ExecutionFailureV1::resource_limit(
            "molecule diagnostics selection exceeds the V1 limit".to_owned(),
        ));
    }
    let selector_utf8_bytes = values
        .iter()
        .try_fold(0_usize, |total, value| total.checked_add(value.len()));
    if !matches!(selector_utf8_bytes, Some(total) if total <= MAX_MOLECULE_DIAGNOSTIC_SELECTOR_UTF8_BYTES_V1)
    {
        return Err(ExecutionFailureV1::resource_limit(
            "molecule diagnostics selection exceeds the V1 UTF-8 byte limit".to_owned(),
        ));
    }
    let mut ids = Vec::new();
    ids.try_reserve_exact(values.len()).map_err(|_| {
        ExecutionFailureV1::resource_limit(
            "molecule diagnostics could not reserve selection".to_owned(),
        )
    })?;
    for value in values {
        let id = DocumentObjectIdV1::parse(value).map_err(|_| {
            ExecutionFailureV1::document_invalid(
                "molecule_ids must contain durable direct root identifiers".to_owned(),
            )
        })?;
        if ids.contains(&id) {
            return Err(ExecutionFailureV1::document_invalid(
                "molecule diagnostics selection repeats a durable direct root".to_owned(),
            ));
        }
        ids.push(id);
    }
    Ok(ids)
}

fn collect_records(
    observation: &SessionDocumentObservationV1,
    molecule_ids: &[DocumentObjectIdV1],
) -> Result<Vec<DocumentMoleculeDiagnosticRecordSummaryV1>, DocumentMoleculeDiagnosticsErrorV1> {
    let snapshot = observation.snapshot();
    verify_molecule_observation_v1(observation, snapshot.revision(), snapshot.digest())
        .map_err(map_inspection_error)?;
    let document = TypedDocument::parse(snapshot.cdml())
        .map_err(DocumentMoleculeInspectionErrorV1::Document)
        .map_err(map_inspection_error)?;
    let document_paint_orders = document_direct_root_paint_orders_v1(observation.projection())
        .map_err(|_| {
            map_inspection_error(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)
        })?;
    let mut records = Vec::new();
    records
        .try_reserve_exact(molecule_ids.len())
        .map_err(|_| DocumentMoleculeDiagnosticsErrorV1::ResourceAllocation)?;
    for molecule_id in molecule_ids {
        direct_projection_molecule_v1(observation.projection(), molecule_id)
            .map_err(map_inspection_error)?;
        let molecule = document
            .core_molecule(molecule_id)
            .map_err(DocumentMoleculeInspectionErrorV1::CoreProjection)
            .map_err(map_inspection_error)?
            .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)
            .map_err(map_inspection_error)?;
        let representation_findings = diagnose_molecule_representation_v1(&molecule)
            .map_err(map_diagnostic_finding_error_v1)
            .map_err(map_report_diagnostics_error)?;
        let capacity = inspect_capacity(observation, molecule_id)?;
        let mut findings =
            collect_report_findings_v1(&molecule, &representation_findings, &capacity)
                .map_err(map_report_diagnostics_error)?;
        if let Err(error) = document_molecule_composition_graph_v1(&molecule) {
            append_graph_finding_v1(&mut findings, &molecule, &error)
                .map_err(map_report_diagnostics_error)?;
        }
        records.push(DocumentMoleculeDiagnosticRecordSummaryV1 {
            molecule_id: copied(molecule_id.as_str())?,
            document_paint_order: *document_paint_orders
                .get(molecule_id)
                .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)
                .map_err(map_inspection_error)?,
            findings,
        });
    }
    records.sort_by_key(|record| record.document_paint_order);
    Ok(records)
}

fn inspect_capacity(
    observation: &SessionDocumentObservationV1,
    molecule_id: &DocumentObjectIdV1,
) -> Result<ferrum_document::DocumentBondCapacityOutcomeV1, DocumentMoleculeDiagnosticsErrorV1> {
    let snapshot = observation.snapshot();
    let request = DocumentBondCapacityRequestV1::new(
        snapshot.revision(),
        *snapshot.digest(),
        vec![molecule_id.clone()],
    )
    .map_err(|_| DocumentMoleculeDiagnosticsErrorV1::Capacity)?;
    let receipt =
        inspect_document_bond_capacity_v1(observation, &request).map_err(map_capacity_error)?;
    receipt
        .records()
        .first()
        .map(|record| record.outcome().clone())
        .ok_or(DocumentMoleculeDiagnosticsErrorV1::Capacity)
}

fn copied(value: &str) -> Result<String, DocumentMoleculeDiagnosticsErrorV1> {
    let mut copied = String::new();
    copied
        .try_reserve_exact(value.len())
        .map_err(|_| DocumentMoleculeDiagnosticsErrorV1::ResourceAllocation)?;
    copied.push_str(value);
    Ok(copied)
}

fn map_inspection_error(
    error: DocumentMoleculeInspectionErrorV1,
) -> DocumentMoleculeDiagnosticsErrorV1 {
    if matches!(error, DocumentMoleculeInspectionErrorV1::ResourceAllocation) {
        DocumentMoleculeDiagnosticsErrorV1::ResourceAllocation
    } else {
        DocumentMoleculeDiagnosticsErrorV1::Inspection
    }
}

fn map_capacity_error(error: DocumentBondCapacityErrorV1) -> DocumentMoleculeDiagnosticsErrorV1 {
    if matches!(error, DocumentBondCapacityErrorV1::ResourceAllocation) {
        DocumentMoleculeDiagnosticsErrorV1::ResourceAllocation
    } else {
        DocumentMoleculeDiagnosticsErrorV1::Capacity
    }
}

fn map_report_diagnostics_error(
    error: MoleculeReportDiagnosticsErrorV1,
) -> DocumentMoleculeDiagnosticsErrorV1 {
    match error {
        MoleculeReportDiagnosticsErrorV1::FindingLimit => {
            DocumentMoleculeDiagnosticsErrorV1::FindingLimit
        }
        MoleculeReportDiagnosticsErrorV1::ResourceAllocation => {
            DocumentMoleculeDiagnosticsErrorV1::ResourceAllocation
        }
        MoleculeReportDiagnosticsErrorV1::Finding => DocumentMoleculeDiagnosticsErrorV1::Finding,
    }
}

fn map_diagnostics_error(error: DocumentMoleculeDiagnosticsErrorV1) -> ExecutionFailureV1 {
    match error {
        DocumentMoleculeDiagnosticsErrorV1::FindingLimit
        | DocumentMoleculeDiagnosticsErrorV1::ResourceAllocation => {
            ExecutionFailureV1::resource_limit(
                "molecule diagnostics exceeded a bounded resource limit".to_owned(),
            )
        }
        DocumentMoleculeDiagnosticsErrorV1::Inspection
        | DocumentMoleculeDiagnosticsErrorV1::Capacity
        | DocumentMoleculeDiagnosticsErrorV1::Finding => ExecutionFailureV1::document_invalid(
            "molecule diagnostics request was refused".to_owned(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
enum DocumentMoleculeDiagnosticsErrorV1 {
    #[error("molecule diagnostics direct-root resolution was refused")]
    Inspection,
    #[error("molecule diagnostics capacity evaluation was refused")]
    Capacity,
    #[error("molecule diagnostics construction was refused")]
    Finding,
    #[error("molecule diagnostics exceeded the bounded finding limit")]
    FindingLimit,
    #[error("molecule diagnostics could not reserve owned storage")]
    ResourceAllocation,
}
