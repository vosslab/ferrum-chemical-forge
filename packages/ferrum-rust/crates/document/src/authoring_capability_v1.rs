//! Shared lifetime-bound authority for document authoring receipts.
//!
//! A capability is process-local and identified by allocation identity.  It is
//! deliberately separate from durable document IDs and revision fences: those
//! facts identify document content, while this type authorizes one live gesture
//! receipt to mutate the session that issued it.

use std::sync::{Arc, Mutex};

/// Opaque process-local issuer owned by one [`crate::DocumentSession`].
#[derive(Clone, Debug)]
pub struct AuthoringCapabilityIssuerV1 {
    identity: Arc<AuthoringCapabilityIssuerIdentityV1>,
}

#[derive(Debug)]
struct AuthoringCapabilityIssuerIdentityV1;

/// Opaque, cloneable, one-shot authority for a document authoring receipt.
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
pub enum AuthoringCapabilityAccessErrorV1 {
    /// The receipt was issued by a distinct live document session.
    ForeignSession,
    /// The receipt is claimed or was already terminally consumed.
    Replayed,
}

/// RAII reservation for a capability-backed document mutation.
///
/// Dropping an unsettled claim restores the receipt to `Available`.  Call
/// [`Self::consume`] only after the owning session has appended its accepted
/// transaction.
#[derive(Debug)]
pub struct AuthoringCapabilityClaimV1 {
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
    pub fn issue(&self) -> AuthoringCapabilityV1 {
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
}

impl AuthoringCapabilityV1 {
    /// Return whether two receipt handles share one one-shot authority.
    #[must_use]
    pub fn same_capability(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }

    /// Return whether this receipt was issued by `issuer`.
    #[must_use]
    pub fn belongs_to(&self, issuer: &AuthoringCapabilityIssuerV1) -> bool {
        self.state.issuer.same_issuer(issuer)
    }

    /// Reserve this receipt for one owner transaction.
    pub fn claim_for_commit(
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
            return Err(AuthoringCapabilityAccessErrorV1::Replayed);
        }
        *disposition = AuthoringCapabilityDispositionV1::Claimed;
        Ok(AuthoringCapabilityClaimV1 {
            state: Arc::clone(&self.state),
            settled: false,
        })
    }

    /// Mark this receipt terminal without appending a document transaction.
    pub fn consume_without_commit(
        &self,
        issuer: &AuthoringCapabilityIssuerV1,
    ) -> Result<(), AuthoringCapabilityAccessErrorV1> {
        self.claim_for_commit(issuer)?.consume();
        Ok(())
    }
}

impl PartialEq for AuthoringCapabilityV1 {
    fn eq(&self, other: &Self) -> bool {
        self.same_capability(other)
    }
}

impl Eq for AuthoringCapabilityV1 {}

impl AuthoringCapabilityClaimV1 {
    /// Terminally consume the claimed receipt after its transaction succeeds.
    pub fn consume(mut self) {
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
    fn aliases_share_one_capability_and_a_dropped_claim_restores_owner_retry() {
        let issuer = AuthoringCapabilityIssuerV1::new();
        let capability = issuer.issue();
        let alias = capability.clone();
        assert!(capability.same_capability(&alias));
        let claim = capability.claim_for_commit(&issuer).expect("claim");
        assert!(matches!(
            alias.claim_for_commit(&issuer),
            Err(AuthoringCapabilityAccessErrorV1::Replayed)
        ));
        drop(claim);
        alias.claim_for_commit(&issuer).expect("owner retry");
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
    fn consumed_and_cancelled_receipts_are_terminal() {
        let issuer = AuthoringCapabilityIssuerV1::new();
        let committed = issuer.issue();
        committed
            .claim_for_commit(&issuer)
            .expect("claim")
            .consume();
        assert!(matches!(
            committed.claim_for_commit(&issuer),
            Err(AuthoringCapabilityAccessErrorV1::Replayed)
        ));

        let cancelled = issuer.issue();
        cancelled
            .consume_without_commit(&issuer)
            .expect("cancel receipt");
        assert!(matches!(
            cancelled.claim_for_commit(&issuer),
            Err(AuthoringCapabilityAccessErrorV1::Replayed)
        ));
    }

    #[test]
    fn final_alias_retains_authority_until_it_is_dropped() {
        let issuer = AuthoringCapabilityIssuerV1::new();
        let capability = issuer.issue();
        let final_alias = capability.clone();
        drop(capability);
        final_alias
            .claim_for_commit(&issuer)
            .expect("last holder remains usable");
    }
}
