//! DTOs for read-only molecular composition reports.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Closed read-only molecule-report request. The runtime and report graph are
/// intentionally absent from this transport contract.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportRequestV1 {
    /// Immutable source facts captured by the initiating live document.
    pub snapshot: DocumentMoleculeReportSnapshotV1,
    pub molecule_ids: Vec<String>,
}

/// Frozen source provenance for a detached molecule-report calculation.
///
/// `revision` fences delivery at the initiating live session. A detached
/// executor re-admits `cdml` into its own temporary session and authenticates
/// that session with `digest_hex`; it never attempts to recreate this revision.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportSnapshotV1 {
    pub cdml: String,
    pub revision: u64,
    pub digest_hex: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportSummaryV1 {
    pub schema: String,
    pub source_revision: u64,
    pub source_digest_hex: String,
    pub records: Vec<DocumentMoleculeReportRecordSummaryV1>,
    /// The one complete aggregate composition or its closed omission reason.
    /// Ferrum never emits a subset aggregate.
    pub aggregate: DocumentMoleculeReportAggregateOutcomeSummaryV1,
}

/// The all-or-none aggregate result for one molecule report.
///
/// This tagged DTO makes a complete composition and an omission reason mutually
/// exclusive at both the Rust and JSON protocol boundaries.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DocumentMoleculeReportAggregateOutcomeSummaryV1 {
    Complete {
        composition: DocumentMoleculeReportCompositionSummaryV1,
    },
    Omitted {
        reason: DocumentMoleculeReportAggregateOmissionReasonSummaryV1,
        recovery: DocumentMoleculeReportFindingRecoverySummaryV1,
    },
}

/// Closed reasons that a molecule-report aggregate is unavailable.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentMoleculeReportAggregateOmissionReasonSummaryV1 {
    FewerThanTwoSelected,
    IncompleteRecordComposition,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportRecordSummaryV1 {
    pub molecule_id: String,
    pub document_paint_order: u32,
    pub authored_name: Option<String>,
    pub atom_count: usize,
    pub bond_count: usize,
    pub authored_charge: Option<i64>,
    pub authored_elements: Vec<DocumentMoleculeReportElementCountSummaryV1>,
    /// Complete engine-derived facts, or `None` when this root cannot produce a
    /// supported composition. `findings` explains that absence.
    pub composition: Option<DocumentMoleculeReportCompositionSummaryV1>,
    pub neutral_bond_capacity: String,
    /// Durable source stereo facts, separate from drawing-only bond presentation.
    pub stereo_semantics: Option<DocumentMoleculeReportStereoSemanticsSummaryV1>,
    /// Durable stereo drawing facts, separate from chemical configuration.
    pub stereo_depiction: Option<DocumentMoleculeReportStereoDepictionSummaryV1>,
    /// Authenticated structured diagnostics in report order.
    pub findings: Vec<DocumentMoleculeReportFindingSummaryV1>,
}

/// Canonical graph-source-indexed stereo facts retained by one direct molecule.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportStereoSemanticsSummaryV1 {
    pub tetrahedral: Vec<DocumentMoleculeReportTetrahedralStereoSummaryV1>,
    pub double_bonds: Vec<DocumentMoleculeReportDoubleBondStereoSummaryV1>,
}

/// Canonical source-indexed drawing facts retained by one direct molecule.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportStereoDepictionSummaryV1 {
    pub directed_bonds: Vec<DocumentMoleculeReportDirectedBondDepictionSummaryV1>,
    pub double_bond_carrier_marks: Vec<DocumentMoleculeReportDoubleBondCarrierMarkSummaryV1>,
}

/// One tetrahedral wedge/hash drawing fact with its authored endpoint direction.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportDirectedBondDepictionSummaryV1 {
    pub bond_index: usize,
    pub start: usize,
    pub end: usize,
    pub presentation: DocumentMoleculeReportDirectedBondPresentationSummaryV1,
}

/// Closed wedge/hash drawing vocabulary.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentMoleculeReportDirectedBondPresentationSummaryV1 {
    SolidWedge,
    HashedWedge,
}

/// One E/Z directional carrier drawing fact.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportDoubleBondCarrierMarkSummaryV1 {
    pub double_bond_index: usize,
    pub carrier_bond_index: usize,
    pub mark: DocumentMoleculeReportDoubleBondCarrierMarkKindSummaryV1,
}

/// Closed E/Z carrier-mark vocabulary.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentMoleculeReportDoubleBondCarrierMarkKindSummaryV1 {
    Up,
    Down,
}

/// One tetrahedral descriptor with exactly four atom positions or an explicit-H sentinel.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportTetrahedralStereoSummaryV1 {
    pub center: usize,
    pub ligands: [DocumentMoleculeReportStereoLigandSummaryV1; 4],
    pub parity: DocumentMoleculeReportTetrahedralParitySummaryV1,
}

/// One ligand in the source-defined tetrahedral order.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DocumentMoleculeReportStereoLigandSummaryV1 {
    Atom { index: usize },
    ExplicitHydrogen,
}

/// Closed tetrahedral parity vocabulary.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentMoleculeReportTetrahedralParitySummaryV1 {
    Clockwise,
    CounterClockwise,
}

/// One E/Z double-bond source descriptor.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportDoubleBondStereoSummaryV1 {
    pub bond_index: usize,
    pub start_ligand: usize,
    pub end_ligand: usize,
    pub configuration: DocumentMoleculeReportDoubleBondConfigurationSummaryV1,
}

/// Closed E/Z configuration vocabulary.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentMoleculeReportDoubleBondConfigurationSummaryV1 {
    E,
    Z,
}

/// One bounded report diagnostic with closed facts and an authenticated location.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportFindingSummaryV1 {
    pub severity: DocumentMoleculeReportFindingSeveritySummaryV1,
    pub code: DocumentMoleculeReportFindingCodeSummaryV1,
    pub recovery: DocumentMoleculeReportFindingRecoverySummaryV1,
    pub location: DocumentMoleculeReportFindingLocationSummaryV1,
    pub detail: Option<String>,
}

/// Closed severity vocabulary for one molecule-report finding.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentMoleculeReportFindingSeveritySummaryV1 {
    Info,
    Warning,
    Error,
}

/// Closed code vocabulary for one molecule-report finding.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentMoleculeReportFindingCodeSummaryV1 {
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
    NeutralCapacityNotChecked,
    NeutralCapacityExceeded,
    IdentifierUnavailable,
}

impl DocumentMoleculeReportFindingCodeSummaryV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TextAtomPresent => "text_atom_present",
            Self::UnexpandedGroupPresent => "unexpanded_group_present",
            Self::ZeroOrderBond => "zero_order_bond",
            Self::CompositionUnavailable => "composition_unavailable",
            Self::UnsupportedVertex => "unsupported_vertex",
            Self::MissingElement => "missing_element",
            Self::InvalidElement => "invalid_element",
            Self::UnsupportedAtomFact => "unsupported_atom_fact",
            Self::UnsupportedBondEndpoint => "unsupported_bond_endpoint",
            Self::UnsupportedBondStyle => "unsupported_bond_style",
            Self::UnsupportedBondOrder => "unsupported_bond_order",
            Self::InconsistentAromaticity => "inconsistent_aromaticity",
            Self::NeutralCapacityNotChecked => "neutral_capacity_not_checked",
            Self::NeutralCapacityExceeded => "neutral_capacity_exceeded",
            Self::IdentifierUnavailable => "identifier_unavailable",
        }
    }
}

/// Closed recovery vocabulary for one molecule-report finding.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentMoleculeReportFindingRecoverySummaryV1 {
    None,
    InspectStructure,
    CorrectChemicalFacts,
    ChooseSupportedRepresentation,
    ReduceSelection,
    RetryWithChemistryRuntime,
}

/// Closed location vocabulary for one authenticated molecule-report finding.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DocumentMoleculeReportFindingLocationSummaryV1 {
    Root,
    Atom {
        identifier: String,
    },
    Vertex {
        identifier: String,
    },
    Bond {
        identifier: String,
    },
    Unaddressable {
        subject: DocumentMoleculeReportFindingSubjectSummaryV1,
    },
}

/// Closed semantic subject vocabulary for an idless finding location.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentMoleculeReportFindingSubjectSummaryV1 {
    Atom,
    Vertex,
    Bond,
}

/// One all-or-none, finite composition receipt mapped from an authenticated
/// chemistry result. No graph, CDML, toolkit, or runtime capability is exposed.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportCompositionSummaryV1 {
    pub formula: String,
    pub net_formal_charge: i64,
    pub average_molecular_weight_da: f64,
    pub monoisotopic_mass_da: f64,
    /// Isotope-aware counts and average-mass percentages in canonical formula order.
    pub elements: Vec<DocumentMoleculeReportCompositionElementSummaryV1>,
}

/// One isotope-aware elemental contribution to a complete composition receipt.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportCompositionElementSummaryV1 {
    pub symbol: String,
    pub isotope: Option<u16>,
    pub atom_count: u64,
    pub average_mass_contribution_da: f64,
    pub mass_percentage: f64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportElementCountSummaryV1 {
    pub symbol: String,
    pub atom_count: usize,
}
