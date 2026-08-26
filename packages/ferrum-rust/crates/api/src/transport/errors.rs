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
    /// Whether this error's complete user-facing outcome was already emitted.
    #[must_use]
    pub const fn was_emitted_to_stream(&self) -> bool {
        matches!(self, Self::Verb(error) if error.was_emitted_to_stream())
    }

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
