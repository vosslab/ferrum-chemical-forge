//! Pure complete-document admission lowering.

use ferrum_render_contract::{
    CompleteRenderAdmissionRefusalV1, CompleteRenderPrimitiveV1, CompleteRenderRootClassV1,
    CompleteRenderRootIdentityV1, CompleteRenderRootLoweringV1, DocumentCompleteRenderCandidateV1,
    RefusedRootReasonV1,
};

use crate::{
    DocumentPrecommitOverlayV1, DocumentRenderPlanCompositionError, DocumentRenderPlanV1,
    RenderError, ResolvedDocumentRenderV1, compose_document_render_plan_v1,
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
    realization: Option<DocumentRenderPlanV1>,
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
        let realization = self.realization.as_ref().ok_or_else(|| {
            RenderError::InvalidRequest("accepted render has no precommit realization".to_owned())
        })?;
        super::document_precommit_overlay_v1::build_document_precommit_overlay_v1(
            realization,
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
    /// The candidate failed complete-render admission.
    Candidate(CompleteRenderAdmissionRefusalV1),
    /// The accepted projection could not be composed into one renderer plan.
    Realization(DocumentRenderPlanCompositionError),
}

/// Purely lower and classify one detached complete-document candidate.
///
/// This function has no document-session callback, no mutation capability, and
/// no route-local text classification. It only returns immutable presentation
/// facts or a shared closed refusal.
pub fn admit_complete_document_render_v1(
    candidate: &DocumentCompleteRenderCandidateV1,
) -> Result<AcceptedCompleteRenderV1, CompleteRenderAdmissionRefusalV1> {
    let mut roots = Vec::with_capacity(candidate.roots().len());
    for root in candidate.roots() {
        let class = classify_root(root.lowering());
        if matches!(class, CompleteRenderRootClassV1::Refused(_)) {
            return Err(CompleteRenderAdmissionRefusalV1::RootRefused {
                root: root.identity().clone(),
                class,
            });
        }
        roots.push(AcceptedCompleteRenderRootV1 {
            identity: root.identity().clone(),
            paint_order: root.paint_order(),
            class,
        });
    }
    Ok(AcceptedCompleteRenderV1 {
        presentation: AcceptedCompleteRenderPresentationV1 { roots },
        realization: None,
        renderer_schema: COMPLETE_DOCUMENT_RENDERER_SCHEMA_V1,
        renderer_generation: 1,
    })
}

/// Admit one complete candidate and retain its renderer-private realization for
/// typed precommit-overlay selection.
pub fn admit_complete_document_render_with_resolved_v1(
    candidate: &DocumentCompleteRenderCandidateV1,
    resolved: &ResolvedDocumentRenderV1,
) -> Result<AcceptedCompleteRenderV1, CompleteDocumentAdmissionErrorV1> {
    let mut accepted = admit_complete_document_render_v1(candidate)
        .map_err(CompleteDocumentAdmissionErrorV1::Candidate)?;
    accepted.realization = Some(
        compose_document_render_plan_v1(resolved)
            .map_err(CompleteDocumentAdmissionErrorV1::Realization)?,
    );
    Ok(accepted)
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
