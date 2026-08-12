use thiserror::Error;

use crate::{Coordinates, MolGraph};

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

/// Safe chemistry operations over immutable, owned Ferrum molecular graphs.
///
/// Implementations may use a native toolkit, a WASM implementation, or another
/// engine. They must never expose that implementation's graph or handle types.
pub trait ChemEngine {
    /// Generate an owned, atom-index-aligned 2D depiction.
    ///
    /// The reference request explicitly selects RDKit canonical orientation and
    /// never imports pre-existing graph coordinates into the depiction engine.
    fn generate_2d_coordinates(&self, molecule: &MolGraph) -> Result<Coordinates, ChemistryError>;

    /// Return a new graph with Kekule orders assigned under `options`.
    fn kekulize(
        &self,
        molecule: &MolGraph,
        options: KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError>;
}

/// A deliberate placeholder for products compiled without a chemistry engine.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnavailableChemEngine;

impl ChemEngine for UnavailableChemEngine {
    fn generate_2d_coordinates(&self, _molecule: &MolGraph) -> Result<Coordinates, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "generate_2d_coordinates",
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
    /// This build has no implementation for the requested operation.
    #[error("chemistry operation is unavailable: {operation}")]
    OperationUnavailable {
        /// Stable operation name.
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
    /// SMILES text violates the ABI-3 input contract before a native call.
    #[error("SMILES input is invalid: {reason}")]
    InvalidSmilesInput {
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
}
