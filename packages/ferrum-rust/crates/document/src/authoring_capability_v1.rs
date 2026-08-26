//! Shared lifetime-bound authority for document authoring receipts.
//!
//! A capability is process-local and identified by allocation identity.  It is
//! deliberately separate from durable document IDs and revision fences: those
//! facts identify document content, while this type authorizes one live gesture
//! receipt to mutate the session that issued it.

use std::sync::{Arc, Mutex};

/// Opaque process-local issuer owned by one [`crate::DocumentSession`].
#[derive(Clone, Debug)]
pub(crate) struct AuthoringCapabilityIssuerV1 {
    identity: Arc<AuthoringCapabilityIssuerIdentityV1>,
}

#[derive(Debug)]
struct AuthoringCapabilityIssuerIdentityV1;

/// Opaque, one-shot authority for a document authoring receipt.
#[derive(Clone, Debug)]
pub struct AuthoringCapabilityV1 {
    state: Arc<AuthoringCapabilityStateV1>,
}

#[derive(Debug)]
struct AuthoringCapabilityStateV1 {
    issuer: AuthoringCapabilityIssuerV1,
    disposition: Mutex<AuthoringCapabilityDispositionV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AuthoringCapabilityDispositionV1 {
    Available,
    Claimed,
    Consumed,
}

/// Admission failure for an authoring receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuthoringCapabilityAccessErrorV1 {
    /// The receipt was issued by a distinct live document session.
    ForeignSession,
    /// The receipt is claimed or was already terminally consumed.
    Consumed,
}

/// Admission failure while pairing one authoring gesture with its preview.
///
/// Pair validation intentionally distinguishes a foreign session from two
/// different same-session receipts.  Public route APIs map this closed error
/// to their own stable error categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuthoringGesturePairAccessErrorV1 {
    /// At least one supplied receipt was issued by another live session.
    ForeignSession,
    /// The receipts are local but do not describe one gesture/preview pair.
    PreviewMismatch,
    /// The shared one-shot receipt was already claimed or consumed.
    Consumed,
}

/// RAII reservation for a capability-backed document mutation.
///
/// Dropping an unsettled claim restores the receipt to `Available`.  Call
/// [`Self::consume`] only after the owning session has appended its accepted
/// transaction.
#[derive(Debug)]
pub(crate) struct AuthoringCapabilityClaimV1 {
    state: Arc<AuthoringCapabilityStateV1>,
    settled: bool,
}

impl AuthoringCapabilityIssuerV1 {
    pub(crate) fn new() -> Self {
        Self {
            identity: Arc::new(AuthoringCapabilityIssuerIdentityV1),
        }
    }

    /// Issue a fresh, one-shot capability for an opaque authoring receipt.
    #[must_use]
    pub(crate) fn issue(&self) -> AuthoringCapabilityV1 {
        AuthoringCapabilityV1 {
            state: Arc::new(AuthoringCapabilityStateV1 {
                issuer: self.clone(),
                disposition: Mutex::new(AuthoringCapabilityDispositionV1::Available),
            }),
        }
    }

    #[must_use]
    pub(crate) fn same_issuer(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }

    /// Validate one opaque gesture/preview pair before a route checks its fence.
    ///
    /// The order is deliberate and shared by every pair-authoring route:
    /// session ownership for both handles, shared capability plus route-owned
    /// semantic equality, one-shot replay status, then the caller's revision
    /// and digest fence check.  The temporary claim is released immediately;
    /// the successful commit retains responsibility for terminal consumption.
    pub(crate) fn validate_gesture_pair_for_prepare_v1(
        &self,
        gesture: &AuthoringCapabilityV1,
        preview: &AuthoringCapabilityV1,
        route_content_matches: bool,
    ) -> Result<(), AuthoringGesturePairAccessErrorV1> {
        if !gesture.belongs_to(self) || !preview.belongs_to(self) {
            return Err(AuthoringGesturePairAccessErrorV1::ForeignSession);
        }
        if !gesture.same_capability(preview) || !route_content_matches {
            return Err(AuthoringGesturePairAccessErrorV1::PreviewMismatch);
        }
        gesture
            .claim_for_commit(self)
            .map(drop)
            .map_err(|error| match error {
                AuthoringCapabilityAccessErrorV1::ForeignSession => {
                    AuthoringGesturePairAccessErrorV1::ForeignSession
                }
                AuthoringCapabilityAccessErrorV1::Consumed => {
                    AuthoringGesturePairAccessErrorV1::Consumed
                }
            })
    }
}

impl AuthoringCapabilityV1 {
    /// Return whether two receipt handles share one one-shot authority.
    #[must_use]
    pub(crate) fn same_capability(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }

    /// Return whether this receipt was issued by `issuer`.
    #[must_use]
    pub(crate) fn belongs_to(&self, issuer: &AuthoringCapabilityIssuerV1) -> bool {
        self.state.issuer.same_issuer(issuer)
    }

    /// Reserve this receipt for one owner transaction.
    pub(crate) fn claim_for_commit(
        &self,
        issuer: &AuthoringCapabilityIssuerV1,
    ) -> Result<AuthoringCapabilityClaimV1, AuthoringCapabilityAccessErrorV1> {
        if !self.belongs_to(issuer) {
            return Err(AuthoringCapabilityAccessErrorV1::ForeignSession);
        }
        let mut disposition = self
            .state
            .disposition
            .lock()
            .expect("authoring capability disposition lock is not poisoned");
        if *disposition != AuthoringCapabilityDispositionV1::Available {
            return Err(AuthoringCapabilityAccessErrorV1::Consumed);
        }
        *disposition = AuthoringCapabilityDispositionV1::Claimed;
        Ok(AuthoringCapabilityClaimV1 {
            state: Arc::clone(&self.state),
            settled: false,
        })
    }
}

impl AuthoringCapabilityClaimV1 {
    /// Terminally consume the claimed receipt after its transaction succeeds.
    pub(crate) fn consume(mut self) {
        let mut disposition = self
            .state
            .disposition
            .lock()
            .expect("authoring capability disposition lock is not poisoned");
        debug_assert_eq!(*disposition, AuthoringCapabilityDispositionV1::Claimed);
        *disposition = AuthoringCapabilityDispositionV1::Consumed;
        self.settled = true;
    }
}

impl Drop for AuthoringCapabilityClaimV1 {
    fn drop(&mut self) {
        if !self.settled {
            let mut disposition = self
                .state
                .disposition
                .lock()
                .expect("authoring capability disposition lock is not poisoned");
            if *disposition == AuthoringCapabilityDispositionV1::Claimed {
                *disposition = AuthoringCapabilityDispositionV1::Available;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropped_claim_restores_owner_retry() {
        let issuer = AuthoringCapabilityIssuerV1::new();
        let capability = issuer.issue();
        let claim = capability.claim_for_commit(&issuer).expect("claim");
        drop(claim);
        capability.claim_for_commit(&issuer).expect("owner retry");
    }

    #[test]
    fn foreign_issuer_is_refused_before_claiming() {
        let owner = AuthoringCapabilityIssuerV1::new();
        let foreign = AuthoringCapabilityIssuerV1::new();
        let capability = owner.issue();
        assert!(matches!(
            capability.claim_for_commit(&foreign),
            Err(AuthoringCapabilityAccessErrorV1::ForeignSession)
        ));
        capability
            .claim_for_commit(&owner)
            .expect("owner remains valid");
    }

    #[test]
    fn consumed_receipts_are_terminal() {
        let issuer = AuthoringCapabilityIssuerV1::new();
        let committed = issuer.issue();
        committed
            .claim_for_commit(&issuer)
            .expect("claim")
            .consume();
        assert!(matches!(
            committed.claim_for_commit(&issuer),
            Err(AuthoringCapabilityAccessErrorV1::Consumed)
        ));
    }

    #[test]
    fn pair_validation_preserves_foreign_mismatch_and_replay_precedence() {
        let owner = AuthoringCapabilityIssuerV1::new();
        let foreign = AuthoringCapabilityIssuerV1::new();
        let gesture = owner.issue();
        let local_other = owner.issue();
        let foreign_preview = foreign.issue();

        assert!(matches!(
            owner.validate_gesture_pair_for_prepare_v1(&gesture, &foreign_preview, false),
            Err(AuthoringGesturePairAccessErrorV1::ForeignSession)
        ));
        assert!(matches!(
            owner.validate_gesture_pair_for_prepare_v1(&gesture, &local_other, false),
            Err(AuthoringGesturePairAccessErrorV1::PreviewMismatch)
        ));

        gesture.claim_for_commit(&owner).expect("claim").consume();
        assert!(matches!(
            owner.validate_gesture_pair_for_prepare_v1(&gesture, &gesture, true),
            Err(AuthoringGesturePairAccessErrorV1::Consumed)
        ));
    }
}
