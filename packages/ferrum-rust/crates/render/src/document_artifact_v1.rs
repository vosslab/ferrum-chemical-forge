//! Renderer-neutral receipts for completed whole-document artifacts.
//!
//! The report is deliberately outside the SVG, PNG, or PDF bytes.  It preserves
//! the exact authenticated observation used for a successful export without
//! asking each format to encode Ferrum-specific provenance metadata.

use crate::{
    DocumentRenderExclusionV1, DocumentRenderOutcomeV1, DocumentRenderPlanV1, RenderProvenance,
    RenderViewportV1,
};

/// Immutable source coverage accompanying a completed whole-document artifact.
///
/// The report retains the plan's exact provenance and complete page rectangle.
/// Its exclusions are exactly the named plan outcomes not painted by the sink,
/// in source order; every other plan outcome was lowered into the artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentRenderReportV1 {
    provenance: RenderProvenance,
    page: RenderViewportV1,
    exclusions: Vec<DocumentRenderExclusionV1>,
}

impl DocumentRenderReportV1 {
    pub(crate) fn from_plan(plan: &DocumentRenderPlanV1) -> Self {
        let exclusions = plan
            .outcomes()
            .iter()
            .filter_map(|outcome| match outcome {
                DocumentRenderOutcomeV1::Root(_) => None,
                DocumentRenderOutcomeV1::Exclusion(exclusion) => Some(exclusion.clone()),
            })
            .collect();
        Self {
            provenance: plan.provenance(),
            page: plan.page(),
            exclusions,
        }
    }

    /// Return the exact authenticated source observation provenance.
    #[must_use]
    pub const fn provenance(&self) -> RenderProvenance {
        self.provenance
    }

    /// Return the complete source page rectangle, including its origin.
    #[must_use]
    pub const fn page(&self) -> RenderViewportV1 {
        self.page
    }

    /// Return each intentionally omitted direct root in source order.
    #[must_use]
    pub fn exclusions(&self) -> &[DocumentRenderExclusionV1] {
        &self.exclusions
    }
}

/// An owned format artifact paired with the source report issued for it.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentRenderArtifactV1<A> {
    artifact: A,
    report: DocumentRenderReportV1,
}

impl<A> DocumentRenderArtifactV1<A> {
    pub(crate) fn from_plan(artifact: A, plan: &DocumentRenderPlanV1) -> Self {
        Self::new(artifact, DocumentRenderReportV1::from_plan(plan))
    }

    pub(crate) const fn new(artifact: A, report: DocumentRenderReportV1) -> Self {
        Self { artifact, report }
    }

    /// Borrow the successfully completed format-specific artifact.
    #[must_use]
    pub const fn artifact(&self) -> &A {
        &self.artifact
    }

    /// Consume this receipt into its completed format-specific artifact.
    #[must_use]
    pub fn into_artifact(self) -> A {
        self.artifact
    }

    /// Borrow the immutable report issued from the source document plan.
    #[must_use]
    pub const fn report(&self) -> &DocumentRenderReportV1 {
        &self.report
    }

    /// Consume this receipt into its artifact and source report.
    #[must_use]
    pub fn into_parts(self) -> (A, DocumentRenderReportV1) {
        (self.artifact, self.report)
    }
}
