//! Safe explicit-path loading for native chemistry operations.

use std::fs;
use std::path::Path;

use crate::{ChemistryError, NativeChemEngine};
use thiserror::Error;

/// Load exactly one caller-selected native adapter without discovery.
pub fn load_explicit_adapter(
    adapter_path: &Path,
) -> Result<NativeChemEngine, ExplicitAdapterError> {
    if !adapter_path.is_absolute() {
        return Err(ExplicitAdapterError::RelativePath {
            path: adapter_path.display().to_string(),
        });
    }
    let metadata =
        fs::symlink_metadata(adapter_path).map_err(|source| ExplicitAdapterError::Metadata {
            path: adapter_path.display().to_string(),
            source,
        })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(ExplicitAdapterError::UnsafePath {
            path: adapter_path.display().to_string(),
        });
    }
    NativeChemEngine::load(adapter_path).map_err(ExplicitAdapterError::Chemistry)
}

/// A caller-selected adapter path that cannot be loaded without discovery.
#[derive(Debug, Error)]
pub enum ExplicitAdapterError {
    /// The provisional CLI requires an absolute path.
    #[error("adapter path must be absolute: {path}")]
    RelativePath { path: String },
    /// Adapter metadata could not be read before dynamic loading.
    #[error("could not inspect adapter path {path}: {source}")]
    Metadata {
        path: String,
        #[source]
        source: std::io::Error,
    },
    /// The path is symbolic or does not identify a regular file.
    #[error("adapter path must name a regular non-symbolic-link file: {path}")]
    UnsafePath { path: String },
    /// The selected library cannot satisfy the safe chemistry boundary.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}
