//! Private renderer ownership for directed direct-bond V3 authoring.
//!
//! Normal, Solid wedge, and Hashed wedge bonds share one pointer-tip to
//! pointer-base candidate lifecycle: begin, admit immutable target operations,
//! then redeem one opaque renderer receipt.

use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_document::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityV1, DirectBondAdmissionRefusalV1,
    DirectBondCommitErrorV1 as DocumentDirectBondCommitErrorV1, DirectBondEndpointIntent,
    DirectBondGestureErrorV1, DirectBondMutationCandidate, DirectBondSnapPolicyV1,
    DocumentBondPresentationV1, DocumentFenceV1, DocumentSession,
};
use ferrum_render::{
    DocumentRenderContentV1, DocumentRenderOutcomeV1, DocumentRenderPlanV1, RenderOp,
    compose_document_render_plan_v1, document_observation_from_accepted_operation_v1,
};
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
    receipt: Option<DirectBondReceipt>,
    overlay: DirectBondOverlay,
}
impl DirectBondAdmission {
    #[must_use]
    pub const fn overlay(&self) -> &DirectBondOverlay {
        &self.overlay
    }
}

#[derive(Debug)]
struct DirectBondReceipt {
    capability: AuthoringCapabilityV1,
    source_fence: DocumentFenceV1,
    candidate_digest: [u8; 32],
    expected_candidate_revision: u64,
    planned_bond: String,
    planned_end_atom: String,
    planned_second_created_atom: Option<String>,
    target_operations: Vec<RenderOp>,
    plan: DocumentRenderPlanV1,
    candidate: DirectBondMutationCandidate,
}

#[derive(Clone, Debug)]
pub struct CommittedDirectBondGesture {
    bond: ferrum_document::PersistentId,
    end_atom: ferrum_document::PersistentId,
    second_created_atom: Option<ferrum_document::PersistentId>,
    created_new_atom: bool,
    created_new_molecule: bool,
    result: ferrum_document::SessionOperationResultV1,
}
impl CommittedDirectBondGesture {
    #[must_use]
    pub fn bond(&self) -> &ferrum_document::PersistentId {
        &self.bond
    }
    #[must_use]
    pub fn end_atom(&self) -> &ferrum_document::PersistentId {
        &self.end_atom
    }
    #[must_use]
    pub fn second_created_atom(&self) -> Option<&ferrum_document::PersistentId> {
        self.second_created_atom.as_ref()
    }
    #[must_use]
    pub const fn created_new_atom(&self) -> bool {
        self.created_new_atom
    }
    #[must_use]
    pub const fn created_new_molecule(&self) -> bool {
        self.created_new_molecule
    }
    #[must_use]
    pub fn result(&self) -> &ferrum_document::SessionOperationResultV1 {
        &self.result
    }
}

/// Closed public classifications for direct-bond V2 receipt redemption.
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

/// The V1 commit values provide the V3 direct-bond commit taxonomy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectBondCommitRecoveryV1 {
    RefreshAndRestart,
    ChangePresentation,
    ReportConflict,
}

/// A typed renderer-receipt or document-bridge commit failure.
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

pub(crate) fn target_operations_for_direct_bond(
    plan: &DocumentRenderPlanV1,
    bond: &ferrum_document::PersistentId,
) -> Option<Vec<RenderOp>> {
    let identifier = Identifier::new(bond.as_str()).ok()?;
    let target = RecordId::from_source(RecordKind::Bond, &identifier);
    plan.outcomes().iter().find_map(|outcome| match outcome {
        DocumentRenderOutcomeV1::Root(root) => match root.content() {
            DocumentRenderContentV1::Molecule(molecule) => molecule
                .batches()
                .iter()
                .find(|batch| batch.target().record_id() == &target)
                .map(|batch| batch.operations().to_vec()),
            _ => None,
        },
        DocumentRenderOutcomeV1::Exclusion(_) => None,
    })
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
    session: &DocumentSession,
    gesture: &DirectBondGesture,
    end: DirectBondEndpointIntent,
) -> Result<DirectBondAdmission, DirectBondAdmissionRefusalV1> {
    require_available_direct_bond_gesture(session, gesture)?;
    let candidate = session.materialize_direct_bond_mutation(
        gesture.fence,
        gesture.start.clone(),
        end,
        gesture.presentation,
        gesture.new_atom_element.clone(),
        gesture.snap,
    )?;
    let candidate_session = DocumentSession::load(candidate.candidate_cdml_for_render_bridge())
        .map_err(|_| DirectBondAdmissionRefusalV1::UnrenderableCandidate)?;
    let observation = candidate_session
        .observe(0)
        .map_err(|_| DirectBondAdmissionRefusalV1::UnrenderableCandidate)?;
    let rendering = document_observation_from_accepted_operation_v1(&observation)
        .map_err(|_| DirectBondAdmissionRefusalV1::UnrenderableCandidate)?;
    let plan = compose_document_render_plan_v1(&rendering)
        .map_err(|_| DirectBondAdmissionRefusalV1::UnrenderableCandidate)?;
    if plan
        .outcomes()
        .iter()
        .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(DirectBondAdmissionRefusalV1::UnrenderableCandidate);
    }
    let operations =
        target_operations_for_direct_bond(&plan, candidate.planned_bond_for_render_bridge())
            .filter(|operations| !operations.is_empty())
            .ok_or(DirectBondAdmissionRefusalV1::UnrenderableCandidate)?;
    let source_fence = candidate.source_fence_for_render_bridge();
    Ok(DirectBondAdmission {
        overlay: DirectBondOverlay {
            start_x: candidate.start_point_for_render_bridge().x(),
            start_y: candidate.start_point_for_render_bridge().y(),
            end_x: candidate.end_point_for_render_bridge().x(),
            end_y: candidate.end_point_for_render_bridge().y(),
            presentation: gesture.presentation,
            operations: operations.clone(),
        },
        receipt: Some(DirectBondReceipt {
            capability: gesture.capability.clone(),
            source_fence,
            candidate_digest: candidate.candidate_digest_for_render_bridge(),
            expected_candidate_revision: source_fence.revision().saturating_add(1),
            planned_bond: candidate
                .planned_bond_for_render_bridge()
                .as_str()
                .to_owned(),
            planned_end_atom: candidate
                .planned_end_atom_for_render_bridge()
                .as_str()
                .to_owned(),
            planned_second_created_atom: candidate
                .planned_second_created_atom_for_render_bridge()
                .map(|atom| atom.as_str().to_owned()),
            target_operations: operations.clone(),
            plan,
            candidate,
        }),
    })
}

pub(crate) fn commit_direct_bond_admission(
    session: &mut DocumentSession,
    admission: &mut DirectBondAdmission,
) -> Result<CommittedDirectBondGesture, DirectBondCommitError> {
    use DirectBondCommitCategoryV1 as Category;
    use DirectBondCommitRecoveryV1 as Recovery;

    let receipt = admission.receipt.take().ok_or_else(|| {
        DirectBondCommitError::new(Category::ReplayedReceipt, Recovery::RefreshAndRestart)
    })?;
    if !receipt
        .capability
        .belongs_to(&session.authoring_capability_issuer_v1())
    {
        admission.receipt = Some(receipt);
        return Err(DirectBondCommitError::new(
            Category::ForeignSession,
            Recovery::RefreshAndRestart,
        ));
    }
    let claim = match receipt
        .capability
        .claim_for_commit(&session.authoring_capability_issuer_v1())
    {
        Ok(claim) => claim,
        Err(AuthoringCapabilityAccessErrorV1::ForeignSession) => {
            unreachable!("owner checked above")
        }
        Err(AuthoringCapabilityAccessErrorV1::Replayed) => {
            admission.receipt = Some(receipt);
            return Err(DirectBondCommitError::new(
                Category::ReplayedReceipt,
                Recovery::RefreshAndRestart,
            ));
        }
    };
    let result = (|| {
        if receipt.candidate.source_fence_for_render_bridge() != receipt.source_fence
            || receipt.candidate.candidate_digest_for_render_bridge() != receipt.candidate_digest
            || receipt.candidate.planned_bond_for_render_bridge().as_str() != receipt.planned_bond
            || receipt
                .candidate
                .planned_end_atom_for_render_bridge()
                .as_str()
                != receipt.planned_end_atom
            || receipt
                .candidate
                .planned_second_created_atom_for_render_bridge()
                .map(|atom| atom.as_str())
                != receipt.planned_second_created_atom.as_deref()
            || target_operations_for_direct_bond(
                &receipt.plan,
                receipt.candidate.planned_bond_for_render_bridge(),
            )
            .as_deref()
                != Some(receipt.target_operations.as_slice())
            || receipt
                .plan
                .outcomes()
                .iter()
                .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
        {
            return Err(DirectBondCommitError::new(
                Category::UnrenderableCandidate,
                Recovery::ChangePresentation,
            ));
        }
        let result = session
            .commit_direct_bond_mutation(&receipt.candidate)
            .map_err(document_commit_error)?;
        debug_assert_eq!(
            result.observation().snapshot().revision(),
            receipt.expected_candidate_revision,
            "a successful direct-bond document commit advances exactly one revision"
        );
        Ok(CommittedDirectBondGesture {
            bond: receipt.candidate.planned_bond_for_render_bridge().clone(),
            end_atom: receipt
                .candidate
                .planned_end_atom_for_render_bridge()
                .clone(),
            second_created_atom: receipt
                .candidate
                .planned_second_created_atom_for_render_bridge()
                .cloned(),
            created_new_atom: receipt.candidate.created_new_atom_for_render_bridge(),
            created_new_molecule: receipt.candidate.created_new_molecule_for_render_bridge(),
            result,
        })
    })();
    match result {
        Ok(committed) => {
            claim.consume();
            Ok(committed)
        }
        Err(error) => {
            admission.receipt = Some(receipt);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document::{DirectBondPoint2V1, DocumentBondOrderV1};

    const EMPTY: &str = "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"></cdml>";

    fn point(x: f64, y: f64) -> DirectBondEndpointIntent {
        DirectBondEndpointIntent::NewAtomAt {
            raw_point: DirectBondPoint2V1::new(x, y).expect("finite point"),
        }
    }

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    fn existing(session: &DocumentSession, source_id: &str) -> DirectBondEndpointIntent {
        let observation = session.observe(0).expect("current observation");
        let atom = observation
            .projection()
            .molecules()
            .iter()
            .flat_map(|molecule| molecule.atoms())
            .find(|atom| atom.source_id() == Some(source_id))
            .expect("source atom projects");
        DirectBondEndpointIntent::ExistingAtom {
            atom: atom.id().expect("source atom has an ID").clone(),
        }
    }

    fn matrix_endpoints(
        form: &str,
    ) -> (
        DocumentSession,
        DirectBondEndpointIntent,
        DirectBondEndpointIntent,
    ) {
        const EXISTING: &str = "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"C\"><point x=\"40\" y=\"0\"/></atom></molecule></cdml>";
        let session = DocumentSession::load(if form == "new_new" { EMPTY } else { EXISTING })
            .expect("matrix session loads");
        let endpoints = match form {
            "existing_existing" => (existing(&session, "a"), existing(&session, "b")),
            "existing_new" => (existing(&session, "a"), point(80.0, 0.0)),
            "new_existing" => (point(-40.0, 0.0), existing(&session, "b")),
            "new_new" => (point(0.0, 0.0), point(40.0, 0.0)),
            _ => unreachable!("closed direct-bond endpoint matrix"),
        };
        (session, endpoints.0, endpoints.1)
    }

    fn durable_target_operations(
        session: &DocumentSession,
        bond: &ferrum_document::PersistentId,
    ) -> Vec<RenderOp> {
        let revision = session.snapshot().expect("committed snapshot").revision();
        let observation = session.observe(revision).expect("committed observation");
        let render_observation = document_observation_from_accepted_operation_v1(&observation)
            .expect("committed observation renders");
        let plan =
            compose_document_render_plan_v1(&render_observation).expect("committed plan composes");
        target_operations_for_direct_bond(&plan, bond).expect("committed bond has a render target")
    }

    #[test]
    fn preflight_matrix_matches_durable_target_operations_without_mutation() {
        for presentation in [
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Double),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Triple),
            DocumentBondPresentationV1::SolidWedge,
            DocumentBondPresentationV1::HashedWedge,
        ] {
            for form in [
                "existing_existing",
                "existing_new",
                "new_existing",
                "new_new",
            ] {
                let (mut session, start, end) = matrix_endpoints(form);
                let before = session.snapshot().expect("snapshot before preflight");
                let gesture = begin_direct_bond_v3_lifecycle(
                    &session,
                    fence(&session),
                    start,
                    presentation,
                    "C".to_owned(),
                    DirectBondSnapPolicyV1::free(),
                )
                .expect("gesture begins");
                let mut admission = admit_direct_bond_candidate(&session, &gesture, end)
                    .expect("candidate preflights");
                let after_preflight = session.snapshot().expect("snapshot after preflight");
                assert_eq!(
                    after_preflight.revision(),
                    before.revision(),
                    "{presentation:?} {form}"
                );
                assert_eq!(
                    after_preflight.digest(),
                    before.digest(),
                    "{presentation:?} {form}"
                );
                let frozen_operations = admission.overlay().operations().to_vec();
                let committed = commit_direct_bond_admission(&mut session, &mut admission)
                    .expect("candidate commits");
                assert_eq!(
                    committed.result().observation().snapshot().revision(),
                    before.revision().saturating_add(1),
                    "{presentation:?} {form}"
                );
                assert_eq!(
                    frozen_operations,
                    durable_target_operations(&session, committed.bond()),
                    "{presentation:?} {form}"
                );
                assert_eq!(
                    committed.second_created_atom().is_some(),
                    form == "new_new",
                    "{presentation:?} {form}"
                );
            }
        }
    }

    #[test]
    fn foreign_receipt_refusal_retains_owner_receipt() {
        let mut owner = DocumentSession::load(EMPTY).expect("owner session");
        let mut foreign = DocumentSession::load(EMPTY).expect("foreign session");
        let gesture = begin_direct_bond_v3_lifecycle(
            &owner,
            fence(&owner),
            point(0.0, 0.0),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            "C".to_owned(),
            DirectBondSnapPolicyV1::free(),
        )
        .expect("gesture begins");
        let mut admission = admit_direct_bond_candidate(&owner, &gesture, point(40.0, 0.0))
            .expect("candidate preflights");
        assert!(matches!(
            commit_direct_bond_admission(&mut foreign, &mut admission),
            Err(DirectBondCommitError {
                category: DirectBondCommitCategoryV1::ForeignSession,
                ..
            })
        ));
        commit_direct_bond_admission(&mut owner, &mut admission).expect("owner still commits");
    }

    #[test]
    fn owner_commit_failure_restores_admission_for_retry() {
        let mut session = DocumentSession::load(EMPTY).expect("owner session");
        let before = session.snapshot().expect("snapshot before commit");
        let gesture = begin_direct_bond_v3_lifecycle(
            &session,
            fence(&session),
            point(0.0, 0.0),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            "C".to_owned(),
            DirectBondSnapPolicyV1::free(),
        )
        .expect("gesture begins");
        let mut admission = admit_direct_bond_candidate(&session, &gesture, point(40.0, 0.0))
            .expect("candidate preflights");
        let target_operations = admission
            .receipt
            .as_ref()
            .expect("admission contains a receipt")
            .target_operations
            .clone();
        admission
            .receipt
            .as_mut()
            .expect("admission retains receipt before redemption")
            .target_operations
            .clear();

        let error = commit_direct_bond_admission(&mut session, &mut admission)
            .expect_err("inconsistent renderer receipt is refused");
        assert_eq!(
            error.category(),
            DirectBondCommitCategoryV1::UnrenderableCandidate
        );
        let after_failure = session.snapshot().expect("snapshot after failure");
        assert_eq!(after_failure.revision(), before.revision());
        assert_eq!(after_failure.digest(), before.digest());

        admission
            .receipt
            .as_mut()
            .expect("failed owner commit restores receipt")
            .target_operations = target_operations;
        let committed = commit_direct_bond_admission(&mut session, &mut admission)
            .expect("restored owner receipt retries successfully");
        assert_eq!(
            committed.result().observation().snapshot().revision(),
            before.revision().saturating_add(1)
        );
    }

    #[test]
    fn sibling_admissions_redeem_once_then_replay_with_closed_categories() {
        let mut session = DocumentSession::load(EMPTY).expect("session loads");
        let before = session.snapshot().expect("snapshot before admissions");
        let gesture = begin_direct_bond_v3_lifecycle(
            &session,
            fence(&session),
            point(0.0, 0.0),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            "C".to_owned(),
            DirectBondSnapPolicyV1::free(),
        )
        .expect("gesture begins");
        let mut first = admit_direct_bond_candidate(&session, &gesture, point(40.0, 0.0))
            .expect("first candidate preflights");
        let mut sibling = admit_direct_bond_candidate(&session, &gesture, point(40.0, 0.0))
            .expect("sibling candidate preflights");
        let after_admissions = session.snapshot().expect("snapshot after admissions");
        assert_eq!(after_admissions.revision(), before.revision());
        assert_eq!(after_admissions.digest(), before.digest());

        commit_direct_bond_admission(&mut session, &mut first).expect("first receipt commits");
        let after_first_commit = session.snapshot().expect("snapshot after first commit");
        let error = commit_direct_bond_admission(&mut session, &mut sibling)
            .expect_err("sibling receipt is replayed");
        assert_eq!(
            error.category(),
            DirectBondCommitCategoryV1::ReplayedReceipt
        );
        assert_eq!(
            error.recovery(),
            DirectBondCommitRecoveryV1::RefreshAndRestart
        );
        let after_sibling_commit = session.snapshot().expect("snapshot after sibling commit");
        assert_eq!(
            after_sibling_commit.revision(),
            after_first_commit.revision()
        );
        assert_eq!(after_sibling_commit.digest(), after_first_commit.digest());

        assert!(matches!(
            admit_direct_bond_candidate(&session, &gesture, point(40.0, 0.0)),
            Err(DirectBondAdmissionRefusalV1::ReplayedGesture)
        ));
    }

    #[test]
    fn document_replayed_receipt_maps_to_closed_renderer_recovery() {
        let error = document_commit_error(DocumentDirectBondCommitErrorV1::ReplayedReceipt);
        assert_eq!(
            error.category(),
            DirectBondCommitCategoryV1::ReplayedReceipt
        );
        assert_eq!(
            error.recovery(),
            DirectBondCommitRecoveryV1::RefreshAndRestart
        );
    }
}
