//! Closed native identifier evaluation for molecule-report records.

use ferrum_chemistry::{ChemEngine, ChemistryError, InchiMode, MolGraph};

use super::molecule_report_core_v1::DocumentMoleculeReportErrorV1;

/// One complete native identifier bundle or its closed per-record omission.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum DocumentMoleculeReportIdentifiersV1 {
    Available {
        canonical_smiles: String,
        standard_inchi: String,
        standard_inchi_key: String,
    },
    Unavailable(DocumentMoleculeReportIdentifierUnavailableReasonV1),
}

/// Closed per-record identifier omission reasons.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DocumentMoleculeReportIdentifierUnavailableReasonV1 {
    UnsupportedMolecule,
    ChemistryUnavailable,
}

#[must_use]
pub(super) const fn unavailable_identifiers_for_missing_graph_v1()
-> DocumentMoleculeReportIdentifiersV1 {
    DocumentMoleculeReportIdentifiersV1::Unavailable(
        DocumentMoleculeReportIdentifierUnavailableReasonV1::UnsupportedMolecule,
    )
}

/// Derive the complete identifier bundle in its explicit dependency order.
///
/// Unsupported graph/export failures are a per-record outcome. Exhaustion is
/// intentionally propagated so the operation's existing resource boundary can
/// refuse the whole request rather than publish a partial bounded result.
pub(super) fn evaluate_identifiers_v1(
    engine: &(impl ChemEngine + ?Sized),
    graph: &MolGraph,
) -> Result<DocumentMoleculeReportIdentifiersV1, DocumentMoleculeReportErrorV1> {
    let canonical_smiles = match engine.molecule_to_smiles(graph) {
        Ok(value) => value,
        Err(error) => return identifier_error_outcome_v1(error),
    };
    let standard_inchi = match engine.molecule_to_inchi(graph, InchiMode::Standard) {
        Ok(value) => value,
        Err(error) => return identifier_error_outcome_v1(error),
    };
    let standard_inchi_key = match engine.inchi_to_inchi_key(&standard_inchi) {
        Ok(value) => value,
        Err(error) => return identifier_error_outcome_v1(error),
    };
    Ok(DocumentMoleculeReportIdentifiersV1::Available {
        canonical_smiles,
        standard_inchi,
        standard_inchi_key,
    })
}

fn identifier_error_outcome_v1(
    error: ChemistryError,
) -> Result<DocumentMoleculeReportIdentifiersV1, DocumentMoleculeReportErrorV1> {
    if matches!(error, ChemistryError::ResourceExhausted { .. }) {
        return Err(DocumentMoleculeReportErrorV1::ResourceAllocation);
    }
    let reason = match error {
        ChemistryError::OperationUnavailable { .. }
        | ChemistryError::NativeBoundary { .. }
        | ChemistryError::MalformedNativeResponse { .. }
        | ChemistryError::TruncatedNativeResponse
        | ChemistryError::TrailingNativeResponse => {
            DocumentMoleculeReportIdentifierUnavailableReasonV1::ChemistryUnavailable
        }
        _ => DocumentMoleculeReportIdentifierUnavailableReasonV1::UnsupportedMolecule,
    };
    Ok(DocumentMoleculeReportIdentifiersV1::Unavailable(reason))
}

pub(super) fn identifier_summary_v1(
    identifiers: &DocumentMoleculeReportIdentifiersV1,
) -> super::dto::DocumentMoleculeReportIdentifiersSummaryV1 {
    match identifiers {
        DocumentMoleculeReportIdentifiersV1::Available {
            canonical_smiles,
            standard_inchi,
            standard_inchi_key,
        } => super::dto::DocumentMoleculeReportIdentifiersSummaryV1::Available {
            canonical_smiles: canonical_smiles.clone(),
            standard_inchi: standard_inchi.clone(),
            standard_inchi_key: standard_inchi_key.clone(),
        },
        DocumentMoleculeReportIdentifiersV1::Unavailable(reason) => {
            super::dto::DocumentMoleculeReportIdentifiersSummaryV1::Unavailable {
                reason: match reason {
                    DocumentMoleculeReportIdentifierUnavailableReasonV1::UnsupportedMolecule => {
                        super::dto::DocumentMoleculeReportIdentifierUnavailableReasonSummaryV1::UnsupportedMolecule
                    }
                    DocumentMoleculeReportIdentifierUnavailableReasonV1::ChemistryUnavailable => {
                        super::dto::DocumentMoleculeReportIdentifierUnavailableReasonSummaryV1::ChemistryUnavailable
                    }
                },
            }
        }
    }
}
