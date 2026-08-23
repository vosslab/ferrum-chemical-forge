//! Native-Rust-only endpoint values and opaque candidates for neutral direct-bond mutation.
//!
//! This module is a noninteractive document-domain transaction seam. It accepts
//! only explicitly resolved durable atom IDs or finite points for new atoms; it
//! does not accept pointer probes, viewport transforms, hit evidence, snapping
//! decisions, overlays, render plans, or issued operations.

use crate::{
    AuthoringCapabilityV1, DirectBondPoint2V1, DirectBondSnapPolicyV1, DocumentBondPresentationV1,
    DocumentFenceV1, DocumentObjectIdV1, PersistentId, SessionOperationResultV1,
};

/// Explicitly resolved input for native, noninteractive direct-bond mutation.
///
/// This public Rust type is not an interactive gesture value and has no Qt or
/// PyO3 route. V2 gesture values remain crate-private implementation details.
#[derive(Clone, Debug, PartialEq)]
pub enum DirectBondEndpointIntent {
    ExistingAtom { atom: DocumentObjectIdV1 },
    NewAtomAt { raw_point: DirectBondPoint2V1 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondGestureV2 {
    pub(crate) fence: DocumentFenceV1,
    pub(crate) start: DirectBondEndpointIntent,
    pub(crate) presentation: DocumentBondPresentationV1,
    pub(crate) new_atom_element: String,
    pub(crate) snap: DirectBondSnapPolicyV1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondOverlayV2 {
    start: DirectBondPoint2V1,
    end: DirectBondPoint2V1,
    presentation: DocumentBondPresentationV1,
}
impl DirectBondOverlayV2 {
    #[must_use]
    pub const fn start(&self) -> DirectBondPoint2V1 {
        self.start
    }
    #[must_use]
    pub const fn end(&self) -> DirectBondPoint2V1 {
        self.end
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondAdmissionV2 {
    pub(crate) fence: DocumentFenceV1,
    pub(crate) candidate: DirectBondAdmittedCandidateV2,
    overlay: DirectBondOverlayV2,
}

/// Renderer-neutral, fenced CDML transition for one native direct-bond mutation.
///
/// The document crate creates this value from explicitly resolved domain
/// endpoints using its own chemistry and identity rules. It carries one private,
/// process-local capability, so a byte-identical independently loaded session
/// cannot commit it and aliases share one redemption authority. The renderer's
/// narrow, read-only bridge may inspect candidate facts needed to preflight a V3
/// target plan; it cannot construct, alter, or commit the candidate. This
/// candidate is not an interactive gesture receipt.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondMutationCandidate {
    pub(crate) capability: AuthoringCapabilityV1,
    pub(crate) source_fence: DocumentFenceV1,
    pub(crate) start_point: DirectBondPoint2V1,
    pub(crate) end_point: DirectBondPoint2V1,
    pub(crate) candidate_cdml: String,
    pub(crate) candidate_digest: [u8; 32],
    pub(crate) planned_bond: PersistentId,
    pub(crate) planned_end_atom: PersistentId,
    pub(crate) planned_second_created_atom: Option<PersistentId>,
    pub(crate) created_new_atom: bool,
    pub(crate) created_new_molecule: bool,
}
impl DirectBondMutationCandidate {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        capability: AuthoringCapabilityV1,
        source_fence: DocumentFenceV1,
        start_point: DirectBondPoint2V1,
        end_point: DirectBondPoint2V1,
        candidate_cdml: String,
        candidate_digest: [u8; 32],
        planned_bond: PersistentId,
        planned_end_atom: PersistentId,
        planned_second_created_atom: Option<PersistentId>,
        created_new_atom: bool,
        created_new_molecule: bool,
    ) -> Self {
        Self {
            capability,
            source_fence,
            start_point,
            end_point,
            candidate_cdml,
            candidate_digest,
            planned_bond,
            planned_end_atom,
            planned_second_created_atom,
            created_new_atom,
            created_new_molecule,
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn start_point_for_render_bridge(&self) -> DirectBondPoint2V1 {
        self.start_point
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn end_point_for_render_bridge(&self) -> DirectBondPoint2V1 {
        self.end_point
    }
    #[doc(hidden)]
    #[must_use]
    pub fn candidate_cdml_for_render_bridge(&self) -> &str {
        &self.candidate_cdml
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn source_fence_for_render_bridge(&self) -> DocumentFenceV1 {
        self.source_fence
    }

    #[doc(hidden)]
    #[must_use]
    pub fn planned_bond_for_render_bridge(&self) -> &PersistentId {
        &self.planned_bond
    }

    #[doc(hidden)]
    #[must_use]
    pub fn planned_end_atom_for_render_bridge(&self) -> &PersistentId {
        &self.planned_end_atom
    }

    /// The pointer-start atom created by a blank-to-blank gesture, if any.
    ///
    /// The V2 public outcome names this separately because `planned_end_atom`
    /// always remains the pointer-release endpoint.
    #[doc(hidden)]
    #[must_use]
    pub fn planned_second_created_atom_for_render_bridge(&self) -> Option<&PersistentId> {
        self.planned_second_created_atom.as_ref()
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn candidate_digest_for_render_bridge(&self) -> [u8; 32] {
        self.candidate_digest
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn created_new_atom_for_render_bridge(&self) -> bool {
        self.created_new_atom
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn created_new_molecule_for_render_bridge(&self) -> bool {
        self.created_new_molecule
    }
}
impl DirectBondAdmissionV2 {
    #[must_use]
    pub fn overlay(&self) -> &DirectBondOverlayV2 {
        &self.overlay
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum DirectBondAdmittedCandidateV2 {
    ExistingExisting {
        start: DocumentObjectIdV1,
        end: DocumentObjectIdV1,
        presentation: DocumentBondPresentationV1,
    },
    ExistingNew {
        existing: DocumentObjectIdV1,
        new_point: DirectBondPoint2V1,
        element: String,
        presentation: DocumentBondPresentationV1,
        new_atom_is_start: bool,
    },
    NewExisting {
        new_point: DirectBondPoint2V1,
        existing: DocumentObjectIdV1,
        element: String,
        presentation: DocumentBondPresentationV1,
        new_atom_is_start: bool,
    },
    NewNew {
        start: DirectBondPoint2V1,
        end: DirectBondPoint2V1,
        element: String,
        presentation: DocumentBondPresentationV1,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommittedDirectBondGestureV2 {
    bond: PersistentId,
    end_atom: PersistentId,
    second_created_atom: Option<PersistentId>,
    created_new_atom: bool,
    created_new_molecule: bool,
    result: SessionOperationResultV1,
}
impl CommittedDirectBondGestureV2 {
    pub(crate) fn new(
        bond: PersistentId,
        end_atom: PersistentId,
        second_created_atom: Option<PersistentId>,
        created_new_atom: bool,
        created_new_molecule: bool,
        result: SessionOperationResultV1,
    ) -> Self {
        Self {
            bond,
            end_atom,
            second_created_atom,
            created_new_atom,
            created_new_molecule,
            result,
        }
    }
    #[must_use]
    pub fn bond(&self) -> &PersistentId {
        &self.bond
    }
    #[must_use]
    pub fn end_atom(&self) -> &PersistentId {
        &self.end_atom
    }
    #[must_use]
    pub fn second_created_atom(&self) -> Option<&PersistentId> {
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
    #[cfg(test)]
    #[must_use]
    pub fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
}

pub(crate) fn admission(
    gesture: &DirectBondGestureV2,
    candidate: DirectBondAdmittedCandidateV2,
    start: DirectBondPoint2V1,
    end: DirectBondPoint2V1,
) -> DirectBondAdmissionV2 {
    DirectBondAdmissionV2 {
        fence: gesture.fence,
        candidate,
        overlay: DirectBondOverlayV2 {
            start,
            end,
            presentation: gesture.presentation,
        },
    }
}
