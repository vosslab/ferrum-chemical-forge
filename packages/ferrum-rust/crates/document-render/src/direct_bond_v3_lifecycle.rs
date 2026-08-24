//! Interaction facade for document-owned direct-bond admission.

use ferrum_document::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityV1, CommittedDirectBondGestureV2,
    DirectBondAdmissionRefusalV1, DirectBondCommitErrorV1 as DocumentDirectBondCommitErrorV1,
    DirectBondEndpointIntent, DirectBondGestureErrorV1, DirectBondSnapPolicyV1,
    DocumentBondPresentationV1, DocumentFenceV1, DocumentSession, PendingDirectBondMutationV1,
};
use ferrum_render::RenderOp;
use thiserror::Error;

#[derive(Clone, Debug)]
pub(crate) struct DirectBondGesture {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    start: DirectBondEndpointIntent,
    presentation: DocumentBondPresentationV1,
    new_atom_element: String,
    snap: DirectBondSnapPolicyV1,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DirectBondOverlay {
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    presentation: DocumentBondPresentationV1,
    operations: Vec<RenderOp>,
}
impl DirectBondOverlay {
    #[must_use]
    pub const fn start_x(&self) -> f64 {
        self.start_x
    }
    #[must_use]
    pub const fn start_y(&self) -> f64 {
        self.start_y
    }
    #[must_use]
    pub const fn end_x(&self) -> f64 {
        self.end_x
    }
    #[must_use]
    pub const fn end_y(&self) -> f64 {
        self.end_y
    }
    #[must_use]
    pub const fn presentation(&self) -> DocumentBondPresentationV1 {
        self.presentation
    }
    #[must_use]
    pub fn operations(&self) -> &[RenderOp] {
        &self.operations
    }
}

#[derive(Debug)]
pub(crate) struct DirectBondAdmission {
    pending: PendingDirectBondMutationV1,
    overlay: DirectBondOverlay,
}
impl DirectBondAdmission {
    #[must_use]
    pub const fn overlay(&self) -> &DirectBondOverlay {
        &self.overlay
    }
}

pub type CommittedDirectBondGesture = CommittedDirectBondGestureV2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectBondCommitCategoryV1 {
    ForeignSession,
    ReplayedReceipt,
    UnrenderableCandidate,
    StaleRevision,
    StaleDigest,
    IdentityAllocationFailed,
    ProvisionalTokenUnavailable,
    CandidateApplicationFailed,
    RevisionExhausted,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectBondCommitRecoveryV1 {
    RefreshAndRestart,
    ChangePresentation,
    ReportConflict,
}
#[derive(Clone, Debug, Error, PartialEq)]
#[error("direct-bond commit failed: {category:?}")]
pub struct DirectBondCommitError {
    category: DirectBondCommitCategoryV1,
    recovery: DirectBondCommitRecoveryV1,
}
impl DirectBondCommitError {
    const fn new(
        category: DirectBondCommitCategoryV1,
        recovery: DirectBondCommitRecoveryV1,
    ) -> Self {
        Self { category, recovery }
    }
    #[must_use]
    pub const fn category(&self) -> DirectBondCommitCategoryV1 {
        self.category
    }
    #[must_use]
    pub const fn recovery(&self) -> DirectBondCommitRecoveryV1 {
        self.recovery
    }
}

fn document_commit_error(error: DocumentDirectBondCommitErrorV1) -> DirectBondCommitError {
    use DirectBondCommitCategoryV1 as Category;
    use DirectBondCommitRecoveryV1 as Recovery;
    let (category, recovery) = match error {
        DocumentDirectBondCommitErrorV1::ForeignSession => {
            (Category::ForeignSession, Recovery::RefreshAndRestart)
        }
        DocumentDirectBondCommitErrorV1::ReplayedReceipt => {
            (Category::ReplayedReceipt, Recovery::RefreshAndRestart)
        }
        DocumentDirectBondCommitErrorV1::StaleRevision => {
            (Category::StaleRevision, Recovery::RefreshAndRestart)
        }
        DocumentDirectBondCommitErrorV1::StaleDigest => {
            (Category::StaleDigest, Recovery::RefreshAndRestart)
        }
        DocumentDirectBondCommitErrorV1::IdentityAllocationFailed => {
            (Category::IdentityAllocationFailed, Recovery::ReportConflict)
        }
        DocumentDirectBondCommitErrorV1::ProvisionalTokenUnavailable => (
            Category::ProvisionalTokenUnavailable,
            Recovery::ReportConflict,
        ),
        DocumentDirectBondCommitErrorV1::CandidateApplicationFailed => (
            Category::CandidateApplicationFailed,
            Recovery::RefreshAndRestart,
        ),
        DocumentDirectBondCommitErrorV1::RevisionExhausted => {
            (Category::RevisionExhausted, Recovery::ReportConflict)
        }
    };
    DirectBondCommitError::new(category, recovery)
}

pub(crate) fn begin_direct_bond_v3_lifecycle(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: DirectBondEndpointIntent,
    presentation: DocumentBondPresentationV1,
    new_atom_element: String,
    snap: DirectBondSnapPolicyV1,
) -> Result<DirectBondGesture, DirectBondGestureErrorV1> {
    Ok(DirectBondGesture {
        capability: session.authoring_capability_issuer_v1().issue(),
        fence,
        start,
        presentation,
        new_atom_element,
        snap,
    })
}

pub(crate) fn require_available_direct_bond_gesture(
    session: &DocumentSession,
    gesture: &DirectBondGesture,
) -> Result<(), DirectBondAdmissionRefusalV1> {
    if !gesture
        .capability
        .belongs_to(&session.authoring_capability_issuer_v1())
    {
        return Err(DirectBondAdmissionRefusalV1::ForeignSession);
    }
    match gesture
        .capability
        .claim_for_commit(&session.authoring_capability_issuer_v1())
    {
        Ok(claim) => {
            drop(claim);
            Ok(())
        }
        Err(AuthoringCapabilityAccessErrorV1::ForeignSession) => {
            Err(DirectBondAdmissionRefusalV1::ForeignSession)
        }
        Err(AuthoringCapabilityAccessErrorV1::Replayed) => {
            Err(DirectBondAdmissionRefusalV1::ReplayedGesture)
        }
    }
}

pub(crate) fn admit_direct_bond_candidate(
    session: &mut DocumentSession,
    gesture: &DirectBondGesture,
    end: DirectBondEndpointIntent,
) -> Result<DirectBondAdmission, DirectBondAdmissionRefusalV1> {
    require_available_direct_bond_gesture(session, gesture)?;
    let pending = session.prepare_direct_bond_mutation_v1(
        gesture.capability.clone(),
        gesture.fence,
        gesture.start.clone(),
        end,
        gesture.presentation,
        gesture.new_atom_element.clone(),
        gesture.snap,
    )?;
    let overlay = DirectBondOverlay {
        start_x: pending.start_v1().x(),
        start_y: pending.start_v1().y(),
        end_x: pending.end_v1().x(),
        end_y: pending.end_v1().y(),
        presentation: pending.presentation_v1(),
        operations: pending.renderer_operations_v1().to_vec(),
    };
    Ok(DirectBondAdmission { pending, overlay })
}

pub(crate) fn commit_direct_bond_admission(
    session: &mut DocumentSession,
    admission: &mut DirectBondAdmission,
) -> Result<CommittedDirectBondGesture, DirectBondCommitError> {
    session
        .commit_direct_bond_mutation_v1(&mut admission.pending)
        .map_err(document_commit_error)
}
