//! Trusted chemistry capability injection for operation protocol V1.
//!
//! Protocol JSON is intentionally path-free.  A caller that needs native
//! chemistry supplies this short-lived capability out-of-band; the engine is
//! loaded for one operation and dropped before the protocol response returns.

use std::path::PathBuf;

use ferrum_chemistry::{ChemEngine, ChemistryError, NativeChemEngine};
use thiserror::Error;

/// Failure while obtaining or using a chemistry execution capability.
#[derive(Debug, Error)]
pub enum ChemistryRuntimeErrorV1 {
    /// No trusted adapter installation is available to this caller.
    #[error("chemistry runtime is unavailable")]
    Unavailable,
    /// A trusted adapter could not be loaded or complete its requested work.
    #[error("chemistry runtime failed: {0}")]
    Chemistry(#[from] ChemistryError),
}

/// One trusted source of a short-lived chemistry engine.
///
/// The generic callback prevents protocol code from retaining a foreign engine
/// handle. Implementations must finish the callback before returning.
pub trait ChemistryRuntimeV1 {
    /// Load or borrow an engine for exactly one owned protocol operation.
    fn with_engine<T>(
        &self,
        operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
    ) -> Result<T, ChemistryRuntimeErrorV1>;
}

/// Runtime used by the path-free convenience executor.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoChemistryRuntimeV1;

impl ChemistryRuntimeV1 for NoChemistryRuntimeV1 {
    fn with_engine<T>(
        &self,
        _operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
    ) -> Result<T, ChemistryRuntimeErrorV1> {
        Err(ChemistryRuntimeErrorV1::Unavailable)
    }
}

/// A runtime whose adapter location came from a trusted, caller-owned locator.
///
/// This Rust-only constructor is deliberately not surfaced in protocol JSON,
/// the public Python API, or a CLI adapter-path option.
#[derive(Clone, Debug)]
pub struct TrustedLibraryChemistryRuntimeV1 {
    library_path: PathBuf,
}

impl TrustedLibraryChemistryRuntimeV1 {
    /// Create a capability from a path already validated by a trusted locator.
    #[must_use]
    pub fn from_trusted_library(library_path: PathBuf) -> Self {
        Self { library_path }
    }
}

impl ChemistryRuntimeV1 for TrustedLibraryChemistryRuntimeV1 {
    fn with_engine<T>(
        &self,
        operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
    ) -> Result<T, ChemistryRuntimeErrorV1> {
        if self.library_path.as_os_str().is_empty() {
            return Err(ChemistryRuntimeErrorV1::Unavailable);
        }
        let engine = NativeChemEngine::load(&self.library_path)?;
        operation(&engine)
    }
}

#[cfg(test)]
mod tests {
    use super::{ChemistryRuntimeErrorV1, ChemistryRuntimeV1, NoChemistryRuntimeV1};
    use crate::protocol::{execute_operation_v1, execute_operation_with_runtime_v1};

    #[test]
    fn absent_runtime_is_a_typed_refusal() {
        let result = NoChemistryRuntimeV1.with_engine(|_| Ok::<_, ChemistryRuntimeErrorV1>(()));
        assert!(matches!(result, Err(ChemistryRuntimeErrorV1::Unavailable)));
    }

    #[test]
    fn existing_path_free_operation_is_unchanged_by_runtime_injection() {
        let request = r#"{
            "schema":"ferrum-operation-request-v1",
            "request_id":"runtime-regression",
            "operation":{"kind":"document.inspect","document":"<cdml/>"}
        }"#;
        let default = execute_operation_v1(request).expect("default execution");
        let injected = execute_operation_with_runtime_v1(request, &NoChemistryRuntimeV1)
            .expect("runtime-aware execution");
        assert_eq!(default, injected);
    }
}
