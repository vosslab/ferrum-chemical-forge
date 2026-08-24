//! Renderer-issued admission for one immutable complete document candidate.
//!
//! The candidate is a validated renderer plan assembled from semantic document
//! facts. The opaque receipt never carries document-session authority; callers
//! may only verify that it still belongs to the exact immutable candidate.

use crate::{DocumentRenderContentV1, DocumentRenderOutcomeV1, DocumentRenderPlanV1, RenderOp};
use ferrum_core::{Identifier, RecordId, RecordKind};
use thiserror::Error;

/// Fixed schema for renderer-issued complete-document admission receipts.
pub const DOCUMENT_RENDER_ADMISSION_SCHEMA_V1: &str = "ferrum-document-render-admission-v1";

/// Document-minted identity for one pending visual transaction.
///
/// The renderer treats this as immutable candidate data.  `ferrum-document`
/// allocates the issuer and sequence; the values carry no session authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentRenderPendingIdentityV1 {
    issuer: u64,
    sequence: u64,
}

impl DocumentRenderPendingIdentityV1 {
    /// Construct one document-owned pending identity.
    #[must_use]
    pub const fn new(issuer: u64, sequence: u64) -> Self {
        Self { issuer, sequence }
    }
}

/// Immutable complete-document input accepted by the renderer admission boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentRenderCandidateV1 {
    plan: DocumentRenderPlanV1,
    pending_identity: DocumentRenderPendingIdentityV1,
}

impl DocumentRenderCandidateV1 {
    /// Construct a candidate from the renderer's complete semantic plan.
    pub fn from_complete_plan(
        plan: DocumentRenderPlanV1,
        pending_identity: DocumentRenderPendingIdentityV1,
    ) -> Result<Self, DocumentRenderAdmissionErrorV1> {
        if plan
            .outcomes()
            .iter()
            .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
        {
            return Err(DocumentRenderAdmissionErrorV1::ExcludedRoots);
        }
        Ok(Self {
            plan,
            pending_identity,
        })
    }

    /// Return the complete immutable plan that defines this candidate.
    #[must_use]
    pub fn plan(&self) -> &DocumentRenderPlanV1 {
        &self.plan
    }
}

/// Opaque renderer proof for one exact admitted complete-document candidate.
///
/// This type deliberately has no public constructor and does not implement
/// `Clone`, serialization, or document-session mutation authority.
#[derive(Debug)]
pub struct AdmittedDocumentRenderCandidateV1 {
    candidate: DocumentRenderCandidateV1,
    renderer_schema: &'static str,
    renderer_generation: u64,
}

impl AdmittedDocumentRenderCandidateV1 {
    /// Verify that this proof belongs to exactly the supplied immutable candidate.
    pub fn verify_candidate_v1(
        &self,
        candidate: &DocumentRenderCandidateV1,
    ) -> Result<(), DocumentRenderAdmissionErrorV1> {
        if self.renderer_schema != DOCUMENT_RENDER_ADMISSION_SCHEMA_V1
            || self.renderer_generation != 1
            || self.candidate != *candidate
        {
            return Err(DocumentRenderAdmissionErrorV1::CandidateMismatch);
        }
        Ok(())
    }

    /// Return the renderer-issued complete plan for the admitted candidate.
    #[must_use]
    pub fn plan(&self) -> &DocumentRenderPlanV1 {
        self.candidate.plan()
    }
}

/// Admit one complete semantic document candidate through the renderer.
pub fn admit_document_render_candidate_v1(
    candidate: &DocumentRenderCandidateV1,
) -> Result<AdmittedDocumentRenderCandidateV1, DocumentRenderAdmissionErrorV1> {
    if candidate
        .plan()
        .outcomes()
        .iter()
        .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(DocumentRenderAdmissionErrorV1::ExcludedRoots);
    }
    Ok(AdmittedDocumentRenderCandidateV1 {
        candidate: candidate.clone(),
        renderer_schema: DOCUMENT_RENDER_ADMISSION_SCHEMA_V1,
        renderer_generation: 1,
    })
}

/// Return the renderer-owned draw operations for one bond in a complete plan.
#[must_use]
pub fn target_operations_for_document_bond_v1(
    plan: &DocumentRenderPlanV1,
    bond_identifier: &str,
) -> Option<Vec<RenderOp>> {
    let identifier = Identifier::new(bond_identifier.to_owned()).ok()?;
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

/// Closed renderer-admission failure before document transaction redemption.
#[derive(Debug, Error)]
pub enum DocumentRenderAdmissionErrorV1 {
    /// A complete artifact would omit one or more semantic document roots.
    #[error("the complete document candidate excludes one or more roots")]
    ExcludedRoots,
    /// The renderer-issued proof does not belong to this immutable candidate.
    #[error("renderer admission proof does not match the document candidate")]
    CandidateMismatch,
}
