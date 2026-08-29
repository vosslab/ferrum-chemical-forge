use thiserror::Error;

use crate::adapter_contract::{
    FERRUM_CHEM_MAX_RESPONSE_BYTES, FERRUM_CHEM_SMARTS_MATCH_MAX_ROWS,
    FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES,
};
use crate::{
    Coordinates, ImportedSdfRecord, MolGraph, MoleculeComposition, SdfRecord, SmilesMolecule,
};

/// Explicit molfile syntax selected for one export operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MolblockVersion {
    /// MDL V2000 syntax with its inherent fixed-width limits and precision.
    V2000,
    /// MDL V3000 syntax.
    V3000,
}

/// Closed InChI serialization profile exposed by Ferrum's engine boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InchiMode {
    /// Standard InChI with the `InChI=1S/` prefix.
    Standard,
    /// Non-standard fixed-hydrogen InChI with the `InChI=1/` prefix.
    FixedHydrogen,
}

/// Options that define one kekulization request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KekulizeOptions {
    /// Remove aromatic flags after choosing Kekule orders when true.
    clear_aromatic_flags: bool,
    /// Select a deterministic representation when the engine supports one.
    canonical: bool,
    /// Bound engine backtracking.
    max_backtracks: u32,
}

impl KekulizeOptions {
    /// Create options with an explicit positive backtracking limit.
    pub fn new(
        clear_aromatic_flags: bool,
        canonical: bool,
        max_backtracks: u32,
    ) -> Result<Self, KekulizeOptionsError> {
        if max_backtracks == 0 {
            return Err(KekulizeOptionsError::ZeroMaxBacktracks);
        }
        Ok(Self {
            clear_aromatic_flags,
            canonical,
            max_backtracks,
        })
    }

    /// Report whether assigned Kekule bonds should lose aromatic flags.
    #[must_use]
    pub const fn clear_aromatic_flags(&self) -> bool {
        self.clear_aromatic_flags
    }

    /// Report whether the engine should select a canonical result.
    #[must_use]
    pub const fn canonical(&self) -> bool {
        self.canonical
    }

    /// Return the positive maximum backtracking count.
    #[must_use]
    pub const fn max_backtracks(&self) -> u32 {
        self.max_backtracks
    }
}

impl Default for KekulizeOptions {
    fn default() -> Self {
        Self {
            clear_aromatic_flags: false,
            canonical: true,
            max_backtracks: 100,
        }
    }
}

/// Bounded enumeration policy for one SMARTS match request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SmartsMatchOptions {
    max_matches: u32,
}

impl SmartsMatchOptions {
    /// Largest number of result rows admitted by the ABI-5 matcher.
    pub const MAX_MATCHES: u32 = FERRUM_CHEM_SMARTS_MATCH_MAX_ROWS;

    /// Construct options with one explicit, positive result cap.
    pub fn new(max_matches: u32) -> Result<Self, SmartsMatchOptionsError> {
        if max_matches == 0 {
            return Err(SmartsMatchOptionsError::ZeroMaxMatches);
        }
        if max_matches > Self::MAX_MATCHES {
            return Err(SmartsMatchOptionsError::MaxMatchesTooLarge {
                maximum: Self::MAX_MATCHES,
            });
        }
        Ok(Self { max_matches })
    }

    /// Return the maximum number of query-ordered rows to retain.
    #[must_use]
    pub const fn max_matches(&self) -> u32 {
        self.max_matches
    }
}

/// Owned facts returned by one bounded SMARTS enumeration.
///
/// Every value in a row is an atom position in the caller-provided [`MolGraph`].
/// Rows preserve query-atom order; no native graph or wire index escapes this type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmartsMatchResult {
    rows: Vec<Vec<usize>>,
    truncated: bool,
}

impl SmartsMatchResult {
    pub(crate) fn new(rows: Vec<Vec<usize>>, truncated: bool) -> Self {
        Self { rows, truncated }
    }

    /// Construct validated, caller-relative match facts for a custom chemistry engine.
    ///
    /// This accepts only typed atom positions in the caller-supplied graph; it
    /// exposes no native matcher, adapter, or wire representation.
    pub fn try_from_rows(
        target: &MolGraph,
        options: SmartsMatchOptions,
        rows: Vec<Vec<usize>>,
        truncated: bool,
    ) -> Result<Self, SmartsMatchResultError> {
        if rows.len()
            > usize::try_from(options.max_matches()).expect("SMARTS match row maximum fits usize")
        {
            return Err(SmartsMatchResultError::TooManyRows {
                maximum: options.max_matches(),
            });
        }
        let atom_count = target.atoms().len();
        let mut query_arity = None;
        for row in &rows {
            if row.is_empty() {
                return Err(SmartsMatchResultError::EmptyRow);
            }
            if let Some(expected) = query_arity {
                if row.len() != expected {
                    return Err(SmartsMatchResultError::InconsistentQueryRowArity {
                        expected,
                        observed: row.len(),
                    });
                }
            } else {
                query_arity = Some(row.len());
            }
            let mut positions = row.clone();
            positions.sort_unstable();
            if positions.windows(2).any(|pair| pair[0] == pair[1]) {
                return Err(SmartsMatchResultError::DuplicateTargetPosition);
            }
            if let Some(position) = row.iter().copied().find(|position| *position >= atom_count) {
                return Err(SmartsMatchResultError::TargetPositionOutOfRange {
                    position,
                    atom_count,
                });
            }
        }
        Ok(Self { rows, truncated })
    }

    /// Return query-ordered target atom positions for every retained match row.
    #[must_use]
    pub fn rows(&self) -> &[Vec<usize>] {
        &self.rows
    }

    /// Report whether native enumeration observed a match beyond the requested cap.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }
}

/// Closed validation failures for caller-owned SMARTS match facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SmartsMatchResultError {
    /// A custom engine supplied more rows than its caller requested.
    #[error("SMARTS result exceeds the maximum of {maximum} rows")]
    TooManyRows {
        /// Caller-requested upper bound shared with the engine invocation.
        maximum: u32,
    },
    /// A SMARTS match must bind at least one query atom.
    #[error("SMARTS result contains an empty match row")]
    EmptyRow,
    /// One query row cannot bind two query atoms to the same target atom.
    #[error("SMARTS result contains a duplicate target atom position")]
    DuplicateTargetPosition,
    /// A custom engine supplied an atom position outside the caller's target graph.
    #[error(
        "SMARTS result references target atom position {position}, but the graph has {atom_count} atoms"
    )]
    TargetPositionOutOfRange {
        /// Invalid caller-relative atom position.
        position: usize,
        /// Number of atoms in the caller-provided target graph.
        atom_count: usize,
    },
    /// Match rows must retain one stable query-atom arity for one request.
    #[error("SMARTS result row arity {observed} differs from the prior query arity {expected}")]
    InconsistentQueryRowArity {
        /// Query atom count established by the first retained row.
        expected: usize,
        /// Query atom count in the inconsistent row.
        observed: usize,
    },
}

/// Closed, detail-free reasons why the SMARTS matcher cannot provide a result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SmartsMatchUnavailableReason {
    /// The required native runtime could not be used.
    RuntimeUnavailable,
    /// The native ABI is incompatible with this engine build.
    AbiIncompatible,
    /// The required ABI-5 SMARTS capability is absent.
    CapabilityUnavailable,
    /// The native call did not complete successfully.
    NativeCallFailed,
    /// The native response violated the private FQM1 contract.
    MalformedNativeResponse,
    /// The native matcher rejected the request or target.
    NativeRejected,
}

/// A rejected [`SmartsMatchOptions`] value.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum SmartsMatchOptionsError {
    /// A match request must retain at least one row.
    #[error("max_matches must be positive")]
    ZeroMaxMatches,
    /// The requested cap exceeds the bounded ABI contract.
    #[error("max_matches exceeds the {maximum}-row SMARTS matcher limit")]
    MaxMatchesTooLarge {
        /// Largest supported row cap.
        maximum: u32,
    },
}

/// Safe chemistry operations over immutable, owned Ferrum molecular graphs.
///
/// Implementations may use a native toolkit, a WASM implementation, or another
/// engine. They must never expose that implementation's graph or handle types.
pub trait ChemEngine {
    /// Parse SMILES into a complete owned graph with atom-aligned 2D coordinates.
    fn smiles_to_molecule(&self, smiles: &str) -> Result<SmilesMolecule, ChemistryError>;

    /// Generate an owned, atom-index-aligned 2D depiction.
    ///
    /// The reference request explicitly selects RDKit canonical orientation and
    /// never imports pre-existing graph coordinates into the depiction engine.
    fn generate_2d_coordinates(&self, molecule: &MolGraph) -> Result<Coordinates, ChemistryError>;

    /// Enumerate bounded SMARTS matches against one caller-owned graph.
    fn smarts_match(
        &self,
        _query: &str,
        _target: &MolGraph,
        _options: SmartsMatchOptions,
    ) -> Result<SmartsMatchResult, ChemistryError> {
        Err(ChemistryError::SmartsMatchUnavailable {
            reason: SmartsMatchUnavailableReason::RuntimeUnavailable,
        })
    }

    /// Export one complete owned graph as canonical SMARTS for this engine build.
    fn molecule_to_smarts(&self, _molecule: &MolGraph) -> Result<String, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "molecule_to_smarts",
        })
    }

    /// Export one complete graph as canonical isomeric SMILES.
    fn molecule_to_smiles(
        &self,
        _molecule: &MolGraph,
        _limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "molecule_to_smiles",
        })
    }

    /// Calculate isotope-aware formula, counts, charge, and masses.
    fn molecule_composition(
        &self,
        _molecule: &MolGraph,
    ) -> Result<MoleculeComposition, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "molecule_composition",
        })
    }

    /// Export one coordinate-bearing graph as an explicit molblock version.
    fn molecule_to_molblock(
        &self,
        _molecule: &MolGraph,
        _version: MolblockVersion,
        _limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "molecule_to_molblock",
        })
    }

    /// Export one coordinate-bearing graph with an exact first-line title.
    fn molecule_to_molblock_with_title(
        &self,
        _molecule: &MolGraph,
        _version: MolblockVersion,
        _title: &str,
        _limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "molecule_to_molblock_with_title",
        })
    }

    /// Import one bounded V2000 or V3000 molblock into a complete owned molecule.
    fn molblock_to_molecule(&self, _molblock: &str) -> Result<SmilesMolecule, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "molblock_to_molecule",
        })
    }

    /// Import one standard or non-standard InChI into an owned 2D molecule.
    fn inchi_to_molecule(&self, _inchi: &str) -> Result<SmilesMolecule, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "inchi_to_molecule",
        })
    }

    /// Export one complete graph using a closed InChI mode.
    fn molecule_to_inchi(
        &self,
        _molecule: &MolGraph,
        _mode: InchiMode,
        _limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "molecule_to_inchi",
        })
    }

    /// Derive the official InChIKey for one validated InChI line.
    fn inchi_to_inchi_key(&self, _inchi: &str) -> Result<String, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "inchi_to_inchi_key",
        })
    }

    /// Export ordered coordinate-bearing records through RDKit's SD writer.
    fn records_to_sdf(
        &self,
        _records: &[SdfRecord],
        _version: MolblockVersion,
        _limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "records_to_sdf",
        })
    }

    /// Import ordered SDF records without exposing toolkit-owned state.
    fn sdf_to_records(&self, _input: &str) -> Result<Vec<ImportedSdfRecord>, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "sdf_to_records",
        })
    }

    /// Return a new graph with Kekule orders assigned under `options`.
    fn kekulize(
        &self,
        molecule: &MolGraph,
        options: KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError>;
}

/// A nonzero maximum number of UTF-8 bytes a native text writer may return.
///
/// The caller owns this policy. The native ABI receives it before invoking an
/// allocating RDKit writer and proves a representation-specific upper bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeTextOutputLimit(u64);

impl NativeTextOutputLimit {
    /// Largest text payload representable by the current adapter response envelope.
    pub const ADAPTER_MAXIMUM: Self =
        Self((FERRUM_CHEM_MAX_RESPONSE_BYTES - FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES) as u64);

    /// Creates a caller-owned nonzero text-output budget.
    pub const fn new(bytes: u64) -> Result<Self, NativeTextOutputLimitError> {
        if bytes == 0 {
            Err(NativeTextOutputLimitError::Zero)
        } else if bytes > Self::ADAPTER_MAXIMUM.0 {
            Err(NativeTextOutputLimitError::ExceedsAdapterMaximum {
                maximum: Self::ADAPTER_MAXIMUM.0,
            })
        } else {
            Ok(Self(bytes))
        }
    }

    /// Returns the byte budget passed to the native ABI.
    #[must_use]
    pub const fn bytes(self) -> u64 {
        self.0
    }
}

/// Rejection from [`NativeTextOutputLimit::new`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum NativeTextOutputLimitError {
    /// An unlimited native text writer is not part of Ferrum's contract.
    #[error("native text output limit must be nonzero")]
    Zero,
    /// A text budget cannot exceed the adapter response envelope.
    #[error("native text output limit exceeds the {maximum}-byte adapter maximum")]
    ExceedsAdapterMaximum { maximum: u64 },
}

/// A deliberate placeholder for products compiled without a chemistry engine.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnavailableChemEngine;

impl ChemEngine for UnavailableChemEngine {
    fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "smiles_to_molecule",
        })
    }

    fn generate_2d_coordinates(&self, _molecule: &MolGraph) -> Result<Coordinates, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "generate_2d_coordinates",
        })
    }

    fn molecule_to_smarts(&self, _molecule: &MolGraph) -> Result<String, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "molecule_to_smarts",
        })
    }

    fn molecule_to_molblock(
        &self,
        _molecule: &MolGraph,
        _version: MolblockVersion,
        _limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "molecule_to_molblock",
        })
    }

    fn molblock_to_molecule(&self, _molblock: &str) -> Result<SmilesMolecule, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "molblock_to_molecule",
        })
    }

    fn inchi_to_molecule(&self, _inchi: &str) -> Result<SmilesMolecule, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "inchi_to_molecule",
        })
    }

    fn molecule_to_inchi(
        &self,
        _molecule: &MolGraph,
        _mode: InchiMode,
        _limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "molecule_to_inchi",
        })
    }

    fn inchi_to_inchi_key(&self, _inchi: &str) -> Result<String, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "inchi_to_inchi_key",
        })
    }

    fn records_to_sdf(
        &self,
        _records: &[SdfRecord],
        _version: MolblockVersion,
        _limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "records_to_sdf",
        })
    }

    fn kekulize(
        &self,
        _molecule: &MolGraph,
        _options: KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "kekulize",
        })
    }
}

/// A chemistry operation failure returned by an engine implementation.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ChemistryError {
    /// SMARTS matching failed without exposing native, loader, wire, or diagnostic detail.
    #[error("SMARTS matching is unavailable: {reason:?}")]
    SmartsMatchUnavailable {
        /// Closed reason suitable for public recovery routing.
        reason: SmartsMatchUnavailableReason,
    },
    /// This build has no implementation for the requested operation.
    #[error("chemistry operation is unavailable: {operation}")]
    OperationUnavailable {
        /// Stable operation name.
        operation: &'static str,
    },
    /// A checked owned result could not be allocated.
    #[error("chemistry operation exhausted memory while producing {operation}")]
    ResourceExhausted {
        /// Stable operation whose owned result could not be completed.
        operation: &'static str,
    },
    /// The graph is valid but cannot be kekulized under the supplied options.
    #[error("kekulization failed: {reason}")]
    KekulizationFailed {
        /// Engine-independent explanation suitable for users and logs.
        reason: String,
    },
    /// The graph is valid but the native depiction engine rejected it.
    #[error("2D coordinate generation failed: {reason}")]
    CoordinateGenerationFailed {
        /// Engine-independent explanation suitable for users and logs.
        reason: String,
    },
    /// A supported chemistry codec could not serialize the supplied graph.
    #[error("{codec} export failed: {reason}")]
    CodecFailed {
        /// Stable codec name.
        codec: &'static str,
        /// Engine-independent explanation suitable for users and logs.
        reason: String,
    },
    /// A native writer refused before allocation because its proven output
    /// upper bound exceeds the caller-owned text budget.
    #[error("{codec} output exceeds the requested native text limit")]
    TextOutputLimitExceeded {
        /// Stable codec name.
        codec: &'static str,
        /// The explicit limit used when the native operation was called.
        maximum: Option<u64>,
    },
    /// SMILES text violates the ABI-4 input contract before a native call.
    #[error("SMILES input is invalid: {reason}")]
    InvalidSmilesInput {
        /// Stable description of the rejected input invariant.
        reason: String,
    },
    /// SDF text violates the ABI-4 input contract before a native call.
    #[error("SDF input is invalid: {reason}")]
    InvalidSdfInput {
        /// Stable description of the rejected input invariant.
        reason: String,
    },
    /// Molblock text violates the ABI-4 input contract before a native call.
    #[error("molblock input is invalid: {reason}")]
    InvalidMolblockInput {
        /// Stable description of the rejected input invariant.
        reason: String,
    },
    /// InChI text violates the ABI-4 input contract before a native call.
    #[error("InChI input is invalid: {reason}")]
    InvalidInchiInput {
        /// Stable description of the rejected input invariant.
        reason: String,
    },
    /// The adapter could not be loaded or did not complete the C ABI call.
    #[error("Ferrum chemistry adapter boundary failure: {reason}")]
    NativeBoundary {
        /// Stable, actionable detail from the FFI ownership boundary.
        reason: String,
    },
    /// The adapter returned a response that cannot be decoded as the Ferrum wire format.
    #[error("Ferrum chemistry adapter returned a malformed response: {reason}")]
    MalformedNativeResponse {
        /// Stable description of the violated response invariant.
        reason: String,
    },
    /// The adapter response ended before its declared structure was complete.
    #[error("Ferrum chemistry adapter returned a truncated response")]
    TruncatedNativeResponse,
    /// The adapter response contained bytes after its complete declared structure.
    #[error("Ferrum chemistry adapter returned trailing response bytes")]
    TrailingNativeResponse,
    /// The adapter rejected an otherwise well-formed request without a Kekulize failure.
    #[error("Ferrum chemistry adapter rejected the request ({status}): {reason}")]
    NativeRejected {
        /// Protocol result status returned by the adapter.
        status: u32,
        /// Adapter-supplied diagnostic text.
        reason: String,
    },
    /// The graph uses a fact not representable by this version of the adapter protocol.
    #[error("molecular graph cannot be represented by the Ferrum chemistry adapter: {reason}")]
    UnsupportedNativeRequest {
        /// Description of the incompatible fact.
        reason: String,
    },
}

/// A rejected [`KekulizeOptions`] value.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum KekulizeOptionsError {
    /// A bounded kekulization must allow at least one backtracking step.
    #[error("max_backtracks must be positive")]
    ZeroMaxBacktracks,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AtomicNumber, BondOrder, MolAtom, MolBond};

    fn aromatic_carbon() -> MolAtom {
        MolAtom::new(
            AtomicNumber::try_from(6).expect("carbon is supported"),
            Some(0),
            None,
            None,
            true,
        )
        .expect("valid carbon")
    }

    #[test]
    fn unavailable_engine_does_not_mutate_the_owned_input() {
        let molecule = MolGraph::new(
            vec![aromatic_carbon(), aromatic_carbon()],
            vec![MolBond::new(0, 1, BondOrder::Aromatic, true)],
            None,
        )
        .expect("valid aromatic graph");
        let original = molecule.clone();

        let result = UnavailableChemEngine.kekulize(&molecule, KekulizeOptions::default());

        assert_eq!(molecule, original);
        assert_eq!(
            result,
            Err(ChemistryError::OperationUnavailable {
                operation: "kekulize",
            })
        );
    }

    #[test]
    fn default_options_are_the_reference_request() {
        let options = KekulizeOptions::default();
        assert!(!options.clear_aromatic_flags());
        assert!(options.canonical());
        assert_eq!(options.max_backtracks(), 100);
        assert_eq!(
            KekulizeOptions::new(false, true, 0),
            Err(KekulizeOptionsError::ZeroMaxBacktracks)
        );
    }

    #[test]
    fn smarts_match_options_limit_tracks_generated_abi_limit() {
        assert_eq!(
            SmartsMatchOptions::MAX_MATCHES,
            crate::adapter_contract::FERRUM_CHEM_SMARTS_MATCH_MAX_ROWS
        );
    }

    #[test]
    fn custom_match_rows_are_bound_to_the_requested_target_and_cap() {
        let target = MolGraph::new(
            vec![aromatic_carbon(), aromatic_carbon()],
            vec![MolBond::new(0, 1, BondOrder::Aromatic, true)],
            None,
        )
        .expect("valid target");
        let one = SmartsMatchOptions::new(1).expect("valid cap");

        assert_eq!(
            SmartsMatchResult::try_from_rows(&target, one, vec![vec![0], vec![1]], false),
            Err(SmartsMatchResultError::TooManyRows { maximum: 1 })
        );
        assert_eq!(
            SmartsMatchResult::try_from_rows(&target, one, vec![vec![2]], false),
            Err(SmartsMatchResultError::TargetPositionOutOfRange {
                position: 2,
                atom_count: 2,
            })
        );
        assert_eq!(
            SmartsMatchResult::try_from_rows(&target, one, vec![vec![0, 1], vec![0]], false),
            Err(SmartsMatchResultError::TooManyRows { maximum: 1 })
        );
        let two = SmartsMatchOptions::new(2).expect("valid cap");
        assert_eq!(
            SmartsMatchResult::try_from_rows(&target, two, vec![vec![], vec![0]], false),
            Err(SmartsMatchResultError::EmptyRow)
        );
        assert_eq!(
            SmartsMatchResult::try_from_rows(&target, two, vec![vec![0, 0]], false),
            Err(SmartsMatchResultError::DuplicateTargetPosition)
        );
        assert_eq!(
            SmartsMatchResult::try_from_rows(&target, two, vec![vec![0, 1], vec![0]], false),
            Err(SmartsMatchResultError::InconsistentQueryRowArity {
                expected: 2,
                observed: 1,
            })
        );
    }
}
