//! Generic document-owned renderer-admitted state transitions.
//!
//! A changed visible state is prepared as one opaque value containing its exact
//! prospective state, observation, renderer proof, deferred session effects, and
//! result. Only this module may redeem that value into session history.

use super::{
    AuthoringCapabilityIssuerV1, DocumentSession, DocumentSessionError, RendererAdmittedPendingV1,
    RevisionState, SessionDocumentObservationV1, SessionOperation, SessionOperationError,
    SessionOperationResultV1,
};
use crate::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityClaimV1, AuthoringCapabilityV1,
    IndexedDocument,
    session_operation::{
        Candidate, CatalogMoleculePlacementOutcomeV1, CreatedPresentationRootKindV1,
        CreatedPresentationRootOutcomeV1, DirectBondOperationOutcomeV1, SessionOperationOutcomeV1,
        SessionOperationV1,
    },
};

use super::ProvisionalToken;

mod core;
mod history;
mod types;

pub(super) use history::AdmittedHistoryV1;
pub use types::*;

/// Construct the sole mutable timeline retained by a document session.
pub(super) fn initial_admitted_history_v1(initial: RevisionState) -> AdmittedHistoryV1 {
    AdmittedHistoryV1::new(initial, 20)
}

#[cfg(test)]
#[path = "admitted_transition_v1/tests.rs"]
mod tests;
