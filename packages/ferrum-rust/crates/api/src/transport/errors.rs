use thiserror::Error;

use crate::cli::engine_bundle::EngineBundleErrorV1;
use crate::cli::protocol::ProtocolCliError;
use crate::cli::verbs::VerbCliError;

/// A command-line operation failed after its arguments were accepted.
#[derive(Debug, Error)]
pub enum CliError {
    /// Native chemistry engine bundle installation or lookup failed.
    #[error(transparent)]
    Engine(#[from] EngineBundleErrorV1),
    /// Frozen operation-protocol CLI input, emission, or publication failed.
    #[error(transparent)]
    Protocol(#[from] ProtocolCliError),
    /// User-facing verb input, execution, or publication failed.
    #[error(transparent)]
    Verb(#[from] VerbCliError),
}

impl CliError {
    /// Return the documented process status for this CLI failure.
    #[must_use]
    pub const fn exit_status(&self) -> u8 {
        match self {
            Self::Engine(_) => 1,
            Self::Protocol(error) => error.exit_status(),
            Self::Verb(error) => error.exit_status(),
        }
    }
}
