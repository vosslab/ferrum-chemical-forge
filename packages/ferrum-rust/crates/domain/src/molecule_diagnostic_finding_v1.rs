//! Closed, bounded chemistry diagnostic facts safe for document reports.

use thiserror::Error;

/// Maximum UTF-8 bytes retained in an optional diagnostic detail.
pub const MAX_MOLECULE_DIAGNOSTIC_DETAIL_BYTES_V1: usize = 256;

/// Stable severity for a report finding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MoleculeDiagnosticSeverityV1 {
    Info,
    Warning,
    Error,
}

/// Closed V1 finding code vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MoleculeDiagnosticCodeV1 {
    TextAtomPresent,
    UnexpandedGroupPresent,
    ZeroOrderBond,
    CompositionUnavailable,
    UnsupportedVertex,
    MissingElement,
    InvalidElement,
    UnsupportedAtomFact,
    UnsupportedBondEndpoint,
    UnsupportedBondStyle,
    UnsupportedBondOrder,
    InconsistentAromaticity,
    IncompleteAuthoredCharge,
    NeutralCapacityNotChecked,
    NeutralCapacityExceeded,
    IdentifierUnavailable,
}

/// Client-actionable bounded recovery vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MoleculeDiagnosticRecoveryV1 {
    None,
    InspectStructure,
    CorrectChemicalFacts,
    ChooseSupportedRepresentation,
    ReduceSelection,
    RetryWithChemistryRuntime,
}

/// Sanitized finding location with an optional durable source identifier.
///
/// A missing source identifier means the scanner can describe the affected
/// semantic subject but cannot safely address a particular authored record.
/// Document adapters validate present identifiers before using them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MoleculeDiagnosticLocationV1 {
    Root,
    Atom { source_identifier: Option<String> },
    Vertex { source_identifier: Option<String> },
    Bond { source_identifier: Option<String> },
}

/// Owned report finding with bounded optional detail.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoleculeDiagnosticFindingV1 {
    severity: MoleculeDiagnosticSeverityV1,
    code: MoleculeDiagnosticCodeV1,
    recovery: MoleculeDiagnosticRecoveryV1,
    location: MoleculeDiagnosticLocationV1,
    detail: Option<String>,
}

impl MoleculeDiagnosticFindingV1 {
    /// Construct a finding after enforcing the public detail budget.
    pub fn new(
        severity: MoleculeDiagnosticSeverityV1,
        code: MoleculeDiagnosticCodeV1,
        recovery: MoleculeDiagnosticRecoveryV1,
        location: MoleculeDiagnosticLocationV1,
        detail: Option<&str>,
    ) -> Result<Self, MoleculeDiagnosticFindingErrorV1> {
        let detail = detail.map(bounded_copy).transpose()?;
        Ok(Self {
            severity,
            code,
            recovery,
            location,
            detail,
        })
    }
    #[must_use]
    pub const fn severity(&self) -> MoleculeDiagnosticSeverityV1 {
        self.severity
    }
    #[must_use]
    pub const fn code(&self) -> MoleculeDiagnosticCodeV1 {
        self.code
    }
    #[must_use]
    pub const fn recovery(&self) -> MoleculeDiagnosticRecoveryV1 {
        self.recovery
    }
    #[must_use]
    pub const fn location(&self) -> &MoleculeDiagnosticLocationV1 {
        &self.location
    }
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

fn bounded_copy(value: &str) -> Result<String, MoleculeDiagnosticFindingErrorV1> {
    if value.len() > MAX_MOLECULE_DIAGNOSTIC_DETAIL_BYTES_V1 {
        return Err(MoleculeDiagnosticFindingErrorV1::DetailTooLong);
    }
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| MoleculeDiagnosticFindingErrorV1::ResourceAllocation)?;
    result.push_str(value);
    Ok(result)
}

/// Construction failure for bounded diagnostics.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum MoleculeDiagnosticFindingErrorV1 {
    #[error("molecule diagnostic detail exceeds the V1 byte limit")]
    DetailTooLong,
    #[error("molecule diagnostic storage could not be reserved")]
    ResourceAllocation,
}

#[cfg(test)]
mod molecule_diagnostic_finding_v1_tests {
    use super::*;
    #[test]
    fn details_are_bounded_without_truncation() {
        assert!(
            MoleculeDiagnosticFindingV1::new(
                MoleculeDiagnosticSeverityV1::Warning,
                MoleculeDiagnosticCodeV1::CompositionUnavailable,
                MoleculeDiagnosticRecoveryV1::InspectStructure,
                MoleculeDiagnosticLocationV1::Root,
                Some(&"x".repeat(MAX_MOLECULE_DIAGNOSTIC_DETAIL_BYTES_V1 + 1)),
            )
            .is_err()
        );
    }
}
