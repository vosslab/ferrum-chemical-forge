//! Authenticated report DTO lowering for bounded molecule diagnostics.

use ferrum_core::Molecule;
use ferrum_document::{DocumentBondCapacityOutcomeV1, DocumentMoleculeCompositionGraphErrorV1};
use ferrum_domain::{
    MoleculeDiagnosticCodeV1, MoleculeDiagnosticFindingErrorV1, MoleculeDiagnosticFindingV1,
    MoleculeDiagnosticLocationV1, MoleculeDiagnosticRecoveryV1, MoleculeDiagnosticSeverityV1,
};
use thiserror::Error;

use super::dto::{
    DocumentMoleculeReportFindingCodeSummaryV1, DocumentMoleculeReportFindingLocationSummaryV1,
    DocumentMoleculeReportFindingRecoverySummaryV1, DocumentMoleculeReportFindingSeveritySummaryV1,
    DocumentMoleculeReportFindingSubjectSummaryV1, DocumentMoleculeReportFindingSummaryV1,
};

/// Maximum report findings retained for one selected molecule.
const MAX_MOLECULE_REPORT_FINDINGS_V1: usize = 64;

/// Public receipt order for representation diagnostics.
///
/// Keep this list explicit when a new report diagnostic category is introduced:
/// the report contract is independent of the scanner's traversal order.
const REPORT_DIAGNOSTIC_CATEGORY_ORDER_V1: [ReportDiagnosticCategoryV1; 3] = [
    ReportDiagnosticCategoryV1::TextAtoms,
    ReportDiagnosticCategoryV1::NeutralCapacity,
    ReportDiagnosticCategoryV1::GroupsAndZeroOrderBonds,
];

/// One report diagnostic category with a stable receipt position.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReportDiagnosticCategoryV1 {
    TextAtoms,
    NeutralCapacity,
    GroupsAndZeroOrderBonds,
}

/// Classify source diagnostics into the deterministic report receipt order.
pub(super) fn collect_report_findings_v1(
    molecule: &Molecule,
    representation_findings: &[MoleculeDiagnosticFindingV1],
    capacity: &DocumentBondCapacityOutcomeV1,
) -> Result<Vec<DocumentMoleculeReportFindingSummaryV1>, MoleculeReportDiagnosticsErrorV1> {
    let mut findings = Vec::new();
    findings
        .try_reserve_exact(MAX_MOLECULE_REPORT_FINDINGS_V1)
        .map_err(|_| MoleculeReportDiagnosticsErrorV1::ResourceAllocation)?;
    for category in REPORT_DIAGNOSTIC_CATEGORY_ORDER_V1 {
        match category {
            ReportDiagnosticCategoryV1::TextAtoms => append_representation_category_v1(
                &mut findings,
                molecule,
                representation_findings,
                |code| code == MoleculeDiagnosticCodeV1::TextAtomPresent,
            )?,
            ReportDiagnosticCategoryV1::NeutralCapacity => {
                append_capacity_finding_v1(&mut findings, molecule, capacity)?;
            }
            ReportDiagnosticCategoryV1::GroupsAndZeroOrderBonds => {
                append_representation_category_v1(
                    &mut findings,
                    molecule,
                    representation_findings,
                    |code| {
                        matches!(
                            code,
                            MoleculeDiagnosticCodeV1::UnexpandedGroupPresent
                                | MoleculeDiagnosticCodeV1::ZeroOrderBond
                        )
                    },
                )?;
            }
        }
    }
    if representation_findings.iter().any(|finding| {
        !matches!(
            finding.code(),
            MoleculeDiagnosticCodeV1::TextAtomPresent
                | MoleculeDiagnosticCodeV1::UnexpandedGroupPresent
                | MoleculeDiagnosticCodeV1::ZeroOrderBond
        )
    }) {
        return Err(MoleculeReportDiagnosticsErrorV1::Finding);
    }
    Ok(findings)
}

/// Append the graph-refusal fact after one otherwise valid source receipt.
pub(super) fn append_graph_finding_v1(
    findings: &mut Vec<DocumentMoleculeReportFindingSummaryV1>,
    molecule: &Molecule,
    error: &DocumentMoleculeCompositionGraphErrorV1,
) -> Result<(), MoleculeReportDiagnosticsErrorV1> {
    let code = match error {
        DocumentMoleculeCompositionGraphErrorV1::EmptyMolecule
        | DocumentMoleculeCompositionGraphErrorV1::UnsupportedVertex { .. } => {
            MoleculeDiagnosticCodeV1::UnsupportedVertex
        }
        DocumentMoleculeCompositionGraphErrorV1::MissingElement { .. } => {
            MoleculeDiagnosticCodeV1::MissingElement
        }
        DocumentMoleculeCompositionGraphErrorV1::InvalidElement { .. } => {
            MoleculeDiagnosticCodeV1::InvalidElement
        }
        DocumentMoleculeCompositionGraphErrorV1::UnsupportedAtomFact { .. } => {
            MoleculeDiagnosticCodeV1::UnsupportedAtomFact
        }
        DocumentMoleculeCompositionGraphErrorV1::UnsupportedBondEndpoint { .. }
        | DocumentMoleculeCompositionGraphErrorV1::DuplicateAtomIdentity { .. } => {
            MoleculeDiagnosticCodeV1::UnsupportedBondEndpoint
        }
        DocumentMoleculeCompositionGraphErrorV1::UnsupportedBondStyle { .. } => {
            MoleculeDiagnosticCodeV1::UnsupportedBondStyle
        }
        DocumentMoleculeCompositionGraphErrorV1::UnsupportedBondOrder { .. } => {
            MoleculeDiagnosticCodeV1::UnsupportedBondOrder
        }
        DocumentMoleculeCompositionGraphErrorV1::InconsistentAromaticity { .. } => {
            MoleculeDiagnosticCodeV1::InconsistentAromaticity
        }
        DocumentMoleculeCompositionGraphErrorV1::Graph(_) => {
            MoleculeDiagnosticCodeV1::CompositionUnavailable
        }
        DocumentMoleculeCompositionGraphErrorV1::ResourceAllocation => {
            return Err(MoleculeReportDiagnosticsErrorV1::ResourceAllocation);
        }
    };
    append_root_finding_v1(
        findings,
        Some(molecule),
        MoleculeDiagnosticSeverityV1::Warning,
        code,
        MoleculeDiagnosticRecoveryV1::ChooseSupportedRepresentation,
    )
}

/// Append the receipt fact emitted when a selected root has no composition.
pub(super) fn append_composition_unavailable_finding_v1(
    findings: &mut Vec<DocumentMoleculeReportFindingSummaryV1>,
) -> Result<(), MoleculeReportDiagnosticsErrorV1> {
    append_root_finding_v1(
        findings,
        None,
        MoleculeDiagnosticSeverityV1::Warning,
        MoleculeDiagnosticCodeV1::CompositionUnavailable,
        MoleculeDiagnosticRecoveryV1::ChooseSupportedRepresentation,
    )
}

fn append_representation_category_v1(
    findings: &mut Vec<DocumentMoleculeReportFindingSummaryV1>,
    molecule: &Molecule,
    representation_findings: &[MoleculeDiagnosticFindingV1],
    belongs_to_category: impl Fn(MoleculeDiagnosticCodeV1) -> bool,
) -> Result<(), MoleculeReportDiagnosticsErrorV1> {
    for finding in representation_findings
        .iter()
        .filter(|finding| belongs_to_category(finding.code()))
    {
        append_report_finding_v1(findings, Some(molecule), finding.clone())?;
    }
    Ok(())
}

fn append_capacity_finding_v1(
    findings: &mut Vec<DocumentMoleculeReportFindingSummaryV1>,
    molecule: &Molecule,
    capacity: &DocumentBondCapacityOutcomeV1,
) -> Result<(), MoleculeReportDiagnosticsErrorV1> {
    match capacity {
        DocumentBondCapacityOutcomeV1::ExceedsCapacity { .. } => append_root_finding_v1(
            findings,
            Some(molecule),
            MoleculeDiagnosticSeverityV1::Warning,
            MoleculeDiagnosticCodeV1::NeutralCapacityExceeded,
            MoleculeDiagnosticRecoveryV1::CorrectChemicalFacts,
        ),
        DocumentBondCapacityOutcomeV1::NotChecked { reason } => {
            let _ = reason;
            append_root_finding_v1(
                findings,
                Some(molecule),
                MoleculeDiagnosticSeverityV1::Info,
                MoleculeDiagnosticCodeV1::NeutralCapacityNotChecked,
                MoleculeDiagnosticRecoveryV1::ChooseSupportedRepresentation,
            )
        }
        DocumentBondCapacityOutcomeV1::WithinCapacity { .. } => Ok(()),
    }
}

fn append_root_finding_v1(
    findings: &mut Vec<DocumentMoleculeReportFindingSummaryV1>,
    molecule: Option<&Molecule>,
    severity: MoleculeDiagnosticSeverityV1,
    code: MoleculeDiagnosticCodeV1,
    recovery: MoleculeDiagnosticRecoveryV1,
) -> Result<(), MoleculeReportDiagnosticsErrorV1> {
    let finding = MoleculeDiagnosticFindingV1::new(
        severity,
        code,
        recovery,
        MoleculeDiagnosticLocationV1::Root,
        None,
    )
    .map_err(map_diagnostic_finding_error_v1)?;
    append_report_finding_v1(findings, molecule, finding)
}

fn append_report_finding_v1(
    findings: &mut Vec<DocumentMoleculeReportFindingSummaryV1>,
    molecule: Option<&Molecule>,
    finding: MoleculeDiagnosticFindingV1,
) -> Result<(), MoleculeReportDiagnosticsErrorV1> {
    let summary =
        authenticated_report_finding_summary_v1(molecule, &finding).map_err(map_mapping_error)?;
    if findings.len() == MAX_MOLECULE_REPORT_FINDINGS_V1 {
        return Err(MoleculeReportDiagnosticsErrorV1::FindingLimit);
    }
    findings
        .try_reserve(1)
        .map_err(|_| MoleculeReportDiagnosticsErrorV1::ResourceAllocation)?;
    findings.push(summary);
    Ok(())
}

pub(super) fn map_diagnostic_finding_error_v1(
    error: MoleculeDiagnosticFindingErrorV1,
) -> MoleculeReportDiagnosticsErrorV1 {
    if matches!(error, MoleculeDiagnosticFindingErrorV1::ResourceAllocation) {
        MoleculeReportDiagnosticsErrorV1::ResourceAllocation
    } else {
        MoleculeReportDiagnosticsErrorV1::Finding
    }
}

fn map_mapping_error(
    error: MoleculeReportDiagnosticMappingErrorV1,
) -> MoleculeReportDiagnosticsErrorV1 {
    if matches!(
        error,
        MoleculeReportDiagnosticMappingErrorV1::ResourceAllocation
    ) {
        MoleculeReportDiagnosticsErrorV1::ResourceAllocation
    } else {
        MoleculeReportDiagnosticsErrorV1::Finding
    }
}

/// Failure while classifying or retaining report diagnostics.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(super) enum MoleculeReportDiagnosticsErrorV1 {
    #[error("molecule report diagnostic construction was refused")]
    Finding,
    #[error("molecule report exceeded the bounded finding limit")]
    FindingLimit,
    #[error("molecule report diagnostic storage could not be reserved")]
    ResourceAllocation,
}

/// Lower one domain finding after confirming its semantic location belongs to
/// the authenticated selected molecule. Root findings and idless facts do not
/// require a molecule borrow because they cannot claim an authored record.
pub(super) fn authenticated_report_finding_summary_v1(
    molecule: Option<&Molecule>,
    finding: &MoleculeDiagnosticFindingV1,
) -> Result<DocumentMoleculeReportFindingSummaryV1, MoleculeReportDiagnosticMappingErrorV1> {
    let location = match finding.location() {
        MoleculeDiagnosticLocationV1::Root => DocumentMoleculeReportFindingLocationSummaryV1::Root,
        MoleculeDiagnosticLocationV1::Atom { source_identifier } => match source_identifier {
            None => DocumentMoleculeReportFindingLocationSummaryV1::Unaddressable {
                subject: DocumentMoleculeReportFindingSubjectSummaryV1::Atom,
            },
            Some(identifier) => {
                let molecule = molecule
                    .ok_or(MoleculeReportDiagnosticMappingErrorV1::UnauthenticatedLocation)?;
                if !molecule
                    .atoms()
                    .iter()
                    .any(|atom| atom.source_id().as_str() == identifier.as_str())
                {
                    return Err(MoleculeReportDiagnosticMappingErrorV1::UnauthenticatedLocation);
                }
                DocumentMoleculeReportFindingLocationSummaryV1::Atom {
                    identifier: copied(identifier)?,
                }
            }
        },
        MoleculeDiagnosticLocationV1::Vertex { source_identifier } => match source_identifier {
            None => DocumentMoleculeReportFindingLocationSummaryV1::Unaddressable {
                subject: DocumentMoleculeReportFindingSubjectSummaryV1::Vertex,
            },
            Some(identifier) => {
                let molecule = molecule
                    .ok_or(MoleculeReportDiagnosticMappingErrorV1::UnauthenticatedLocation)?;
                let contains_vertex = molecule
                    .texts()
                    .iter()
                    .chain(molecule.groups())
                    .any(|vertex| vertex.source_id().as_str() == identifier.as_str());
                if !contains_vertex {
                    return Err(MoleculeReportDiagnosticMappingErrorV1::UnauthenticatedLocation);
                }
                DocumentMoleculeReportFindingLocationSummaryV1::Vertex {
                    identifier: copied(identifier)?,
                }
            }
        },
        MoleculeDiagnosticLocationV1::Bond { source_identifier } => match source_identifier {
            None => DocumentMoleculeReportFindingLocationSummaryV1::Unaddressable {
                subject: DocumentMoleculeReportFindingSubjectSummaryV1::Bond,
            },
            Some(identifier) => {
                let molecule = molecule
                    .ok_or(MoleculeReportDiagnosticMappingErrorV1::UnauthenticatedLocation)?;
                if !molecule
                    .bonds()
                    .iter()
                    .any(|bond| bond.source_id().as_str() == identifier.as_str())
                {
                    return Err(MoleculeReportDiagnosticMappingErrorV1::UnauthenticatedLocation);
                }
                DocumentMoleculeReportFindingLocationSummaryV1::Bond {
                    identifier: copied(identifier)?,
                }
            }
        },
    };
    Ok(DocumentMoleculeReportFindingSummaryV1 {
        severity: finding_severity_summary_v1(finding.severity()),
        code: finding_code_summary_v1(finding.code()),
        recovery: finding_recovery_summary_v1(finding.recovery()),
        location,
        detail: copied_option(finding.detail())?,
    })
}

pub(super) const fn finding_severity_summary_v1(
    severity: MoleculeDiagnosticSeverityV1,
) -> DocumentMoleculeReportFindingSeveritySummaryV1 {
    match severity {
        MoleculeDiagnosticSeverityV1::Info => DocumentMoleculeReportFindingSeveritySummaryV1::Info,
        MoleculeDiagnosticSeverityV1::Warning => {
            DocumentMoleculeReportFindingSeveritySummaryV1::Warning
        }
        MoleculeDiagnosticSeverityV1::Error => {
            DocumentMoleculeReportFindingSeveritySummaryV1::Error
        }
    }
}

pub(super) const fn finding_code_summary_v1(
    code: MoleculeDiagnosticCodeV1,
) -> DocumentMoleculeReportFindingCodeSummaryV1 {
    match code {
        MoleculeDiagnosticCodeV1::TextAtomPresent => {
            DocumentMoleculeReportFindingCodeSummaryV1::TextAtomPresent
        }
        MoleculeDiagnosticCodeV1::UnexpandedGroupPresent => {
            DocumentMoleculeReportFindingCodeSummaryV1::UnexpandedGroupPresent
        }
        MoleculeDiagnosticCodeV1::ZeroOrderBond => {
            DocumentMoleculeReportFindingCodeSummaryV1::ZeroOrderBond
        }
        MoleculeDiagnosticCodeV1::CompositionUnavailable => {
            DocumentMoleculeReportFindingCodeSummaryV1::CompositionUnavailable
        }
        MoleculeDiagnosticCodeV1::UnsupportedVertex => {
            DocumentMoleculeReportFindingCodeSummaryV1::UnsupportedVertex
        }
        MoleculeDiagnosticCodeV1::MissingElement => {
            DocumentMoleculeReportFindingCodeSummaryV1::MissingElement
        }
        MoleculeDiagnosticCodeV1::InvalidElement => {
            DocumentMoleculeReportFindingCodeSummaryV1::InvalidElement
        }
        MoleculeDiagnosticCodeV1::UnsupportedAtomFact => {
            DocumentMoleculeReportFindingCodeSummaryV1::UnsupportedAtomFact
        }
        MoleculeDiagnosticCodeV1::UnsupportedBondEndpoint => {
            DocumentMoleculeReportFindingCodeSummaryV1::UnsupportedBondEndpoint
        }
        MoleculeDiagnosticCodeV1::UnsupportedBondStyle => {
            DocumentMoleculeReportFindingCodeSummaryV1::UnsupportedBondStyle
        }
        MoleculeDiagnosticCodeV1::UnsupportedBondOrder => {
            DocumentMoleculeReportFindingCodeSummaryV1::UnsupportedBondOrder
        }
        MoleculeDiagnosticCodeV1::InconsistentAromaticity => {
            DocumentMoleculeReportFindingCodeSummaryV1::InconsistentAromaticity
        }
        MoleculeDiagnosticCodeV1::NeutralCapacityNotChecked => {
            DocumentMoleculeReportFindingCodeSummaryV1::NeutralCapacityNotChecked
        }
        MoleculeDiagnosticCodeV1::NeutralCapacityExceeded => {
            DocumentMoleculeReportFindingCodeSummaryV1::NeutralCapacityExceeded
        }
        MoleculeDiagnosticCodeV1::IdentifierUnavailable => {
            DocumentMoleculeReportFindingCodeSummaryV1::IdentifierUnavailable
        }
    }
}

pub(super) const fn finding_recovery_summary_v1(
    recovery: MoleculeDiagnosticRecoveryV1,
) -> DocumentMoleculeReportFindingRecoverySummaryV1 {
    match recovery {
        MoleculeDiagnosticRecoveryV1::None => DocumentMoleculeReportFindingRecoverySummaryV1::None,
        MoleculeDiagnosticRecoveryV1::InspectStructure => {
            DocumentMoleculeReportFindingRecoverySummaryV1::InspectStructure
        }
        MoleculeDiagnosticRecoveryV1::CorrectChemicalFacts => {
            DocumentMoleculeReportFindingRecoverySummaryV1::CorrectChemicalFacts
        }
        MoleculeDiagnosticRecoveryV1::ChooseSupportedRepresentation => {
            DocumentMoleculeReportFindingRecoverySummaryV1::ChooseSupportedRepresentation
        }
        MoleculeDiagnosticRecoveryV1::MaterializeCompactGroup => {
            DocumentMoleculeReportFindingRecoverySummaryV1::MaterializeCompactGroup
        }
        MoleculeDiagnosticRecoveryV1::ReduceSelection => {
            DocumentMoleculeReportFindingRecoverySummaryV1::ReduceSelection
        }
        MoleculeDiagnosticRecoveryV1::RetryWithChemistryRuntime => {
            DocumentMoleculeReportFindingRecoverySummaryV1::RetryWithChemistryRuntime
        }
    }
}

fn copied_option(
    value: Option<&str>,
) -> Result<Option<String>, MoleculeReportDiagnosticMappingErrorV1> {
    value.map(copied).transpose()
}

fn copied(value: &str) -> Result<String, MoleculeReportDiagnosticMappingErrorV1> {
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| MoleculeReportDiagnosticMappingErrorV1::ResourceAllocation)?;
    result.push_str(value);
    Ok(result)
}

/// Failure while lowering domain diagnostics into authenticated protocol facts.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(super) enum MoleculeReportDiagnosticMappingErrorV1 {
    #[error("molecule diagnostic location is not authenticated by the selected root")]
    UnauthenticatedLocation,
    #[error("molecule diagnostic DTO storage could not be reserved")]
    ResourceAllocation,
}
