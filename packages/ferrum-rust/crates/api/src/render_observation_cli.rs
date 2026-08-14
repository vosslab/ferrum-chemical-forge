//! CLI-facing construction of one complete Ferrum render observation.

use ferrum_document::{DocumentSession, DocumentSessionError};
use thiserror::Error;

use crate::{RenderObservationWireV1, observe_render_v1};

/// Build the sole render-observation wire report for one newly loaded CDML document.
///
/// The CLI has no session mutation command, so loading creates the only accepted initial
/// revision, zero. The helper therefore performs one authoritative load followed by one
/// revision-checked observation. A whole-projection suppression is an unsuccessful CLI
/// request because stdout must contain a complete render result rather than an empty plan.
pub(crate) fn render_observation(
    source: &str,
) -> Result<RenderObservationWireV1, RenderObservationCliError> {
    let session = DocumentSession::load(source)?;
    let observation = observe_render_v1(&session, 0)?;
    if observation.suppression().is_some() {
        return Err(RenderObservationCliError::Suppressed);
    }
    Ok(observation.wire())
}

/// Serialize one complete CLI render observation as its canonical JSON object.
pub(crate) fn render_observation_json(source: &str) -> Result<String, RenderObservationCliError> {
    render_observation(source)?
        .to_canonical_json()
        .map_err(RenderObservationCliError::Wire)
}

/// Failure while producing the complete CLI render observation.
#[derive(Debug, Error)]
pub enum RenderObservationCliError {
    /// The supplied CDML could not establish the authoritative document session.
    #[error(transparent)]
    Document(#[from] DocumentSessionError),
    /// The loaded document could not produce a verified Ferrum render observation.
    #[error(transparent)]
    Observation(#[from] crate::RenderObservationError),
    /// The closed render-observation wire object could not be encoded.
    #[error(transparent)]
    Wire(#[from] ferrum_render::RenderError),
    /// Invalid presentation facts suppress the complete render plan.
    #[error("Ferrum V1 suppressed the complete render observation for invalid presentation facts")]
    Suppressed,
}
