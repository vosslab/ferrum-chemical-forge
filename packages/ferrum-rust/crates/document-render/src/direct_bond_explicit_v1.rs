//! Explicit-endpoint direct-bond authoring for stateless Rust clients.
//!
//! This boundary accepts semantic document endpoints only. Pointer probing and
//! viewport coordinates remain a UI concern. The document session owns the
//! renderer-admitted pending candidate and resulting mutation; the renderer
//! admits and verifies the complete plan.

use ferrum_document::{
    DirectBondAdmissionRefusalV1, DirectBondEndpointIntent, DirectBondGestureErrorV1,
    DirectBondSnapPolicyV1, DocumentBondPresentationV1, DocumentFenceV1, DocumentSession,
};
use thiserror::Error;

use super::direct_bond_v3_lifecycle::{
    admit_direct_bond_candidate, begin_direct_bond_v3_lifecycle, commit_direct_bond_admission,
};
use super::{CommittedDirectBondGesture, DirectBondCommitError};

/// A direct-bond refusal from semantic admission or one commit receipt.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum DirectBondExplicitErrorV1 {
    #[error("direct-bond admission refused: {0}")]
    Admission(DirectBondAdmissionRefusalV1),
    #[error(transparent)]
    Commit(#[from] DirectBondCommitError),
    #[error("direct-bond session transaction failed")]
    SessionConflict,
}

/// Author one direct bond from explicit endpoint intent in a single lifecycle.
///
/// The caller never receives a capability, candidate, preview, or commit
/// receipt. Failed admission leaves document content, history, and durable
/// generated IDs unchanged; the private admission sequence may advance before
/// the final receipt redemption succeeds.
pub fn author_direct_bond_explicit_v1(
    session: &mut DocumentSession,
    fence: DocumentFenceV1,
    start: DirectBondEndpointIntent,
    end: DirectBondEndpointIntent,
    presentation: DocumentBondPresentationV1,
    new_atom_element: String,
    snap: DirectBondSnapPolicyV1,
) -> Result<CommittedDirectBondGesture, DirectBondExplicitErrorV1> {
    let gesture =
        begin_direct_bond_v3_lifecycle(session, fence, start, presentation, new_atom_element, snap)
            .map_err(admission_refusal)?;
    let mut admission = admit_direct_bond_candidate(session, &gesture, end)
        .map_err(DirectBondExplicitErrorV1::Admission)?;
    commit_direct_bond_admission(session, &mut admission).map_err(DirectBondExplicitErrorV1::from)
}

fn admission_refusal(error: DirectBondGestureErrorV1) -> DirectBondExplicitErrorV1 {
    use DirectBondAdmissionRefusalV1 as Refusal;
    if matches!(error, DirectBondGestureErrorV1::SessionConflict) {
        return DirectBondExplicitErrorV1::SessionConflict;
    }
    let refusal = match error {
        DirectBondGestureErrorV1::StaleRevision => Refusal::StaleRevision,
        DirectBondGestureErrorV1::StaleDigest => Refusal::StaleDigest,
        DirectBondGestureErrorV1::ForeignSession => Refusal::ForeignSession,
        DirectBondGestureErrorV1::ReplayedGesture => Refusal::ReplayedGesture,
        DirectBondGestureErrorV1::UnknownStartAtom => Refusal::UnknownStartAtom,
        DirectBondGestureErrorV1::UnknownEndAtom => Refusal::UnknownEndAtom,
        DirectBondGestureErrorV1::UnsupportedPresentation => Refusal::UnsupportedPresentation,
        DirectBondGestureErrorV1::SelfLoop => Refusal::SelfLoop,
        DirectBondGestureErrorV1::CrossMolecule => Refusal::CrossMolecule,
        DirectBondGestureErrorV1::DuplicateBond => Refusal::DuplicateBond,
        DirectBondGestureErrorV1::NonFinitePoint | DirectBondGestureErrorV1::InvalidSnapPolicy => {
            Refusal::InvalidEndpointInput
        }
        DirectBondGestureErrorV1::CollapsedEndpoint => Refusal::CollapsedEndpoint,
        DirectBondGestureErrorV1::UnrenderableCandidate => Refusal::UnrenderableCandidate,
        DirectBondGestureErrorV1::ExceedsChemistryCapacity => Refusal::ExceedsChemistryCapacity,
        DirectBondGestureErrorV1::UnsupportedChemistryAdmission => {
            Refusal::UnsupportedChemistryAdmission
        }
        DirectBondGestureErrorV1::SessionConflict => unreachable!("handled above"),
    };
    DirectBondExplicitErrorV1::Admission(refusal)
}

#[cfg(test)]
mod tests {
    use ferrum_document::{
        DirectBondPoint2V1, DirectBondSnapPolicyV1, DocumentBondOrderV1, DocumentFenceV1,
        DocumentSession,
    };

    use super::{
        DirectBondEndpointIntent, DocumentBondPresentationV1, author_direct_bond_explicit_v1,
    };

    #[test]
    fn explicit_endpoints_preflight_and_commit_one_native_bond() {
        let mut session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"/>")
                .expect("empty document loads");
        let snapshot = session.snapshot().expect("snapshot");
        let endpoint = |x, y| DirectBondEndpointIntent::NewAtomAt {
            raw_point: DirectBondPoint2V1::new(x, y).expect("finite endpoint"),
        };
        let committed = author_direct_bond_explicit_v1(
            &mut session,
            DocumentFenceV1::new(snapshot.revision(), *snapshot.digest()),
            endpoint(0.0, 0.0),
            endpoint(40.0, 0.0),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            "C".to_owned(),
            DirectBondSnapPolicyV1::free(),
        )
        .expect("explicit endpoint authoring commits");
        assert!(committed.created_new_molecule());
        assert!(committed.created_new_atom());
        assert!(
            committed
                .result()
                .observation()
                .snapshot()
                .cdml()
                .contains(committed.bond().as_str())
        );
    }
}
