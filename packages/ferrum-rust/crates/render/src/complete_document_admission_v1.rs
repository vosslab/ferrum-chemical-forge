//! Pure complete-document admission lowering.

use ferrum_render_contract::{
    CompleteRenderPrimitiveV1, CompleteRenderRootClassV1, CompleteRenderRootIdentityV1,
    CompleteRenderRootLoweringV1, DocumentCompleteRenderCandidateV1, RefusedRootReasonV1,
};

use crate::{
    DocumentPrecommitOverlayV1, DocumentRenderContentV1, DocumentRenderOutcomeV1,
    DocumentRenderPlanCompositionError, DocumentRenderPlanV1, RenderError,
    ResolvedDocumentRenderV2, compose_document_render_plan_v1,
};
use ferrum_document_projection::DocumentObjectIdV1;

/// Fixed renderer schema recorded in V1 accepted values.
pub const COMPLETE_DOCUMENT_RENDERER_SCHEMA_V1: &str = "ferrum-complete-document-renderer-v1";

/// Immutable presentation fact for one renderer-admitted complete-render root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedCompleteRenderRootV1 {
    identity: CompleteRenderRootIdentityV1,
    paint_order: u32,
    class: CompleteRenderRootClassV1,
}

impl AcceptedCompleteRenderRootV1 {
    /// Return the durable root identity.
    #[must_use]
    pub const fn identity(&self) -> &CompleteRenderRootIdentityV1 {
        &self.identity
    }

    /// Return the root's canonical paint order.
    #[must_use]
    pub const fn paint_order(&self) -> u32 {
        self.paint_order
    }

    /// Return the accepted visual root class.
    #[must_use]
    pub const fn class(&self) -> CompleteRenderRootClassV1 {
        self.class
    }
}

/// Immutable, deliberately lossy renderer presentation of an accepted candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedCompleteRenderPresentationV1 {
    roots: Vec<AcceptedCompleteRenderRootV1>,
}

impl AcceptedCompleteRenderPresentationV1 {
    /// Return accepted visual roots in exact canonical paint order.
    #[must_use]
    pub fn roots(&self) -> &[AcceptedCompleteRenderRootV1] {
        &self.roots
    }
}

/// Opaque renderer acceptance for one exact complete-document candidate.
///
/// This value has no public construction, verification, candidate, identity,
/// plan, serialization, or document-session authority. Its only observation is
/// a deliberately lossy immutable presentation value.
#[derive(Clone, Debug, PartialEq)]
pub struct AcceptedCompleteRenderV1 {
    presentation: AcceptedCompleteRenderPresentationV1,
    source_omissions: Vec<RenderOmission>,
    realization: DocumentRenderPlanV1,
    renderer_schema: &'static str,
    renderer_generation: u64,
}

impl AcceptedCompleteRenderV1 {
    /// Return an immutable presentation projection without acceptance authority.
    #[must_use]
    pub fn presentation(&self) -> AcceptedCompleteRenderPresentationV1 {
        self.presentation.clone()
    }

    /// Select inert paint facts from this renderer-admitted realization.
    pub fn precommit_overlay_v1(
        &self,
        request: &AcceptedRenderOverlayRequestV1,
    ) -> Result<DocumentPrecommitOverlayV1, RenderError> {
        super::document_precommit_overlay_v1::build_document_precommit_overlay_v1(
            &self.realization,
            request,
        )
    }
}

/// Typed selection of document records whose accepted paint belongs in a
/// precommit overlay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedRenderOverlayRequestV1 {
    targets: Vec<AcceptedRenderOverlayTargetV1>,
}

impl AcceptedRenderOverlayRequestV1 {
    /// Build one nonempty selection of accepted document records.
    pub fn new(targets: Vec<AcceptedRenderOverlayTargetV1>) -> Result<Self, RenderError> {
        if targets.is_empty() {
            return Err(RenderError::InvalidRequest(
                "precommit overlay requires at least one selected target".to_owned(),
            ));
        }
        Ok(Self { targets })
    }

    pub(crate) fn targets(&self) -> &[AcceptedRenderOverlayTargetV1] {
        &self.targets
    }
}

/// One typed document record selected from an accepted renderer realization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedRenderOverlayTargetV1 {
    kind: AcceptedRenderOverlayTargetKindV1,
    document_object_id: DocumentObjectIdV1,
}

impl AcceptedRenderOverlayTargetV1 {
    /// Select one atom record by its persisted durable identity.
    #[must_use]
    pub const fn atom(document_object_id: DocumentObjectIdV1) -> Self {
        Self {
            kind: AcceptedRenderOverlayTargetKindV1::Atom,
            document_object_id,
        }
    }

    /// Select one bond record by its persisted durable identity.
    #[must_use]
    pub const fn bond(document_object_id: DocumentObjectIdV1) -> Self {
        Self {
            kind: AcceptedRenderOverlayTargetKindV1::Bond,
            document_object_id,
        }
    }

    /// Return the closed record class required for this durable selection.
    #[must_use]
    pub const fn kind(&self) -> AcceptedRenderOverlayTargetKindV1 {
        self.kind
    }

    /// Return the persisted document identity selected for the overlay.
    #[must_use]
    pub const fn document_object_id(&self) -> &DocumentObjectIdV1 {
        &self.document_object_id
    }
}

/// Closed target classes supported by the V1 accepted-overlay selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcceptedRenderOverlayTargetKindV1 {
    /// One molecule atom record.
    Atom,
    /// One molecule bond record.
    Bond,
}

/// Failure while constructing an accepted renderer realization.
#[derive(Debug)]
pub enum CompleteDocumentAdmissionErrorV1 {
    /// A source or candidate projection could not be composed into one renderer plan.
    Realization(DocumentRenderPlanCompositionError),
    /// The candidate introduces or replaces one renderer omission.
    NewOmission,
}

/// Purely lower and classify the direct roots of one detached candidate.
///
/// This function has no document-session callback, no mutation capability, and
/// no route-local text classification. It only returns immutable presentation
/// facts or a shared closed refusal.
pub fn classify_document_render_roots_v1(
    candidate: &DocumentCompleteRenderCandidateV1,
) -> AcceptedCompleteRenderPresentationV1 {
    let mut roots = Vec::with_capacity(candidate.roots().len());
    for root in candidate.roots() {
        let class = classify_root(root.lowering());
        roots.push(AcceptedCompleteRenderRootV1 {
            identity: root.identity().clone(),
            paint_order: root.paint_order(),
            class,
        });
    }
    AcceptedCompleteRenderPresentationV1 { roots }
}

/// Admit one exact resolved document and retain its complete renderer realization.
///
/// Direct-root classification is necessary but deliberately insufficient. The
/// renderer compares complete source and candidate omission sets, admitting a
/// candidate only when it introduces no new exclusion, plan issue, or member
/// depiction issue. Existing imported diagnostics may remain or be repaired;
/// the accepted receipt retains only the exact candidate realization for
/// temporary overlay paint.
pub fn admit_complete_document_render_v1(
    candidate: &DocumentCompleteRenderCandidateV1,
    baseline: &ResolvedDocumentRenderV2,
    candidate_resolved: &ResolvedDocumentRenderV2,
) -> Result<AcceptedCompleteRenderV1, CompleteDocumentAdmissionErrorV1> {
    let presentation = classify_document_render_roots_v1(candidate);
    let baseline = compose_document_render_plan_v1(baseline)
        .map_err(CompleteDocumentAdmissionErrorV1::Realization)?;
    let realization = compose_document_render_plan_v1(candidate_resolved)
        .map_err(CompleteDocumentAdmissionErrorV1::Realization)?;
    let source_omissions = omissions(&baseline);
    let candidate_omissions = omissions(&realization);
    candidate_omissions_are_not_new(&source_omissions, &candidate_omissions)
        .then_some(())
        .ok_or(CompleteDocumentAdmissionErrorV1::NewOmission)?;
    Ok(AcceptedCompleteRenderV1 {
        presentation,
        source_omissions,
        realization,
        renderer_schema: COMPLETE_DOCUMENT_RENDERER_SCHEMA_V1,
        renderer_generation: 1,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RenderOmission {
    RootExclusion {
        target: DocumentObjectIdV1,
        feature: String,
    },
    PlanIssue {
        target: DocumentObjectIdV1,
        kind: crate::RenderIssueKind,
    },
    MemberIssue {
        target: DocumentObjectIdV1,
        code: crate::DepictionIssueCodeV1,
        detail: String,
    },
}

fn omissions(plan: &DocumentRenderPlanV1) -> Vec<RenderOmission> {
    let mut omissions = Vec::new();
    for outcome in plan.outcomes() {
        match outcome {
            DocumentRenderOutcomeV1::Exclusion(exclusion) => {
                omissions.push(RenderOmission::RootExclusion {
                    target: exclusion.target().document_object_id().clone(),
                    feature: exclusion.feature().to_owned(),
                });
            }
            DocumentRenderOutcomeV1::Root(root) => {
                let DocumentRenderContentV1::Molecule(content) = root.content() else {
                    continue;
                };
                for issue in content.plan().issues() {
                    omissions.push(RenderOmission::PlanIssue {
                        target: issue.target().document_object_id().clone(),
                        kind: issue.kind().clone(),
                    });
                }
                for issue in content.member_issues() {
                    omissions.push(RenderOmission::MemberIssue {
                        target: issue.target().clone(),
                        code: issue.code(),
                        detail: issue.detail().to_owned(),
                    });
                }
            }
        }
    }
    omissions
}

fn candidate_omissions_are_not_new(
    source_omissions: &[RenderOmission],
    candidate_omissions: &[RenderOmission],
) -> bool {
    candidate_omissions
        .iter()
        .all(|candidate| source_omissions.contains(candidate))
}

fn classify_root(lowering: CompleteRenderRootLoweringV1) -> CompleteRenderRootClassV1 {
    match lowering {
        CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Molecule) => {
            CompleteRenderRootClassV1::VisualMolecule
        }
        CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Text) => {
            CompleteRenderRootClassV1::VisualText
        }
        CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Vector) => {
            CompleteRenderRootClassV1::VisualVector
        }
        CompleteRenderRootLoweringV1::Nonvisual => {
            CompleteRenderRootClassV1::Refused(RefusedRootReasonV1::ProfileExcluded)
        }
        CompleteRenderRootLoweringV1::MissingRequiredPrimitive => {
            CompleteRenderRootClassV1::Refused(RefusedRootReasonV1::MissingRequiredPrimitive)
        }
    }
}
