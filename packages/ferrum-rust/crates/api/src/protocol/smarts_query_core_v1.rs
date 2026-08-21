//! Private, bounded document SMARTS execution over one owned snapshot.

use ferrum_chemistry::{ChemEngine, SmartsMatchOptions};
use ferrum_document::{SessionDocumentObservationV1, verify_molecule_observation_v1};

use super::{
    document_smarts_snapshot_v1::OwnedDocumentSmartsSnapshotV1,
    dto::{
        DocumentSmartsQueryInputV1, DocumentSmartsQueryMoleculeSummaryV1,
        DocumentSmartsQueryRequestV1, DocumentSmartsQuerySummaryV1,
        DocumentSmartsQueryTraversalSummaryV1, OperationProtocolOutcomeV1,
    },
    execution::ExecutionFailureV1,
    runtime::{ChemistryRuntimeErrorV1, ChemistryRuntimeV1},
};

const MAX_QUERY_BYTES: usize = 8_192;
const MAX_PER_MOLECULE: u32 = 128;
const MAX_TOTAL: u32 = 256;
const DEFAULT_PER_MOLECULE: u32 = 50;
const DEFAULT_TOTAL: u32 = 200;
const SCHEMA: &str = "ferrum-document-molecule-smarts-query-v1";

pub(super) fn execute_document_smarts_query_v1<R: ChemistryRuntimeV1>(
    observation: &SessionDocumentObservationV1,
    request: DocumentSmartsQueryRequestV1,
    runtime: &R,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let digest = parse_digest(&request.document.expected_digest_hex)?;
    verify_molecule_observation_v1(observation, request.document.expected_revision, &digest)
        .map_err(|_| ExecutionFailureV1::document_invalid("stale_document".to_owned()))?;
    let per = request
        .limits
        .max_matches_per_molecule
        .unwrap_or(DEFAULT_PER_MOLECULE);
    let total = request.limits.max_total_matches.unwrap_or(DEFAULT_TOTAL);
    if per == 0 || per > MAX_PER_MOLECULE || total == 0 || total > MAX_TOTAL || per > total {
        return Err(ExecutionFailureV1::document_invalid(
            "match_caps_inconsistent".to_owned(),
        ));
    }
    let snapshot = OwnedDocumentSmartsSnapshotV1::from_accepted_observation_v1(observation)?;
    let query_text = match request.query {
        DocumentSmartsQueryInputV1::Smarts { value } => {
            if value.is_empty() || value.len() > MAX_QUERY_BYTES || value.contains('\0') {
                return Err(ExecutionFailureV1::document_invalid(
                    "query_too_long".to_owned(),
                ));
            }
            value
        }
        DocumentSmartsQueryInputV1::SelectedMolecule { molecule_id } => {
            let target = snapshot
                .selected_target_by_durable_selector(&molecule_id)
                .ok_or_else(|| {
                    ExecutionFailureV1::document_invalid("selected_source_not_molecule".to_owned())
                })?;
            runtime
                .with_engine(|engine| {
                    engine
                        .molecule_to_smarts(target.graph())
                        .map_err(|_| ChemistryRuntimeErrorV1::Unavailable)
                })
                .map_err(map_runtime)?
        }
    };
    if query_text.is_empty() || query_text.len() > MAX_QUERY_BYTES || query_text.contains('\0') {
        return Err(ExecutionFailureV1::chemistry_unavailable(
            "native_runtime_unavailable".to_owned(),
        ));
    }
    let summary = runtime
        .with_engine(|engine| execute_owned_snapshot(engine, &snapshot, &query_text, per, total))
        .map_err(map_runtime)?;
    Ok(OperationProtocolOutcomeV1::DocumentSmartsQuery { query: summary })
}

fn execute_owned_snapshot(
    engine: &dyn ChemEngine,
    snapshot: &OwnedDocumentSmartsSnapshotV1,
    query_text: &str,
    per: u32,
    total: u32,
) -> Result<DocumentSmartsQuerySummaryV1, ChemistryRuntimeErrorV1> {
    let mut molecules = Vec::new();
    let mut remaining = total;
    let mut incomplete = false;
    for target in snapshot.targets() {
        if remaining == 0 {
            incomplete = true;
            break;
        }
        let result = engine
            .smarts_match(
                query_text,
                target.graph(),
                SmartsMatchOptions::new(per.min(remaining))
                    .map_err(|_| ChemistryRuntimeErrorV1::Unavailable)?,
            )
            .map_err(|_| ChemistryRuntimeErrorV1::Unavailable)?;
        let count =
            u32::try_from(result.rows().len()).map_err(|_| ChemistryRuntimeErrorV1::Unavailable)?;
        remaining = remaining
            .checked_sub(count)
            .ok_or(ChemistryRuntimeErrorV1::Unavailable)?;
        if count != 0 {
            molecules.push(DocumentSmartsQueryMoleculeSummaryV1 {
                source_order: target.source_order(),
                match_count: count,
                completeness: if result.truncated() {
                    "truncated"
                } else {
                    "complete"
                }
                .to_owned(),
            });
        }
    }
    Ok(DocumentSmartsQuerySummaryV1 {
        schema: SCHEMA.to_owned(),
        traversal: if incomplete {
            DocumentSmartsQueryTraversalSummaryV1::Incomplete {
                reason: "total_match_budget_reached".to_owned(),
            }
        } else {
            DocumentSmartsQueryTraversalSummaryV1::Complete
        },
        molecules,
    })
}

fn parse_digest(value: &str) -> Result<[u8; 32], ExecutionFailureV1> {
    if value.len() != 64 {
        return Err(ExecutionFailureV1::document_invalid(
            "digest_mismatch".to_owned(),
        ));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        digest[index] = u8::from_str_radix(std::str::from_utf8(pair).unwrap_or(""), 16)
            .map_err(|_| ExecutionFailureV1::document_invalid("digest_mismatch".to_owned()))?;
    }
    Ok(digest)
}

fn map_runtime(error: ChemistryRuntimeErrorV1) -> ExecutionFailureV1 {
    match error {
        ChemistryRuntimeErrorV1::Unavailable | ChemistryRuntimeErrorV1::Chemistry(_) => {
            ExecutionFailureV1::chemistry_unavailable("native_runtime_unavailable".to_owned())
        }
    }
}

#[cfg(test)]
#[path = "smarts_query_core_v1_tests.rs"]
mod tests;
