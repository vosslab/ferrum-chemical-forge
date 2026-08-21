//! Closed two-endpoint planning values for direct normal-bond gestures.

use super::direct_bond_gesture_v1::DirectBondGestureCapabilityV1;
use crate::{
    DirectBondPoint2V1, DirectBondSnapPolicyV1, DocumentBondPresentationV1, DocumentFenceV1,
    DocumentObjectIdV1, PersistentId, SessionOperationResultV1,
};

#[derive(Clone, Debug, PartialEq)]
pub enum DirectBondEndpointIntentV2 {
    ExistingAtom { atom: DocumentObjectIdV1 },
    NewAtomAt { raw_point: DirectBondPoint2V1 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondGestureV2 {
    pub(crate) capability: DirectBondGestureCapabilityV1,
    pub(crate) fence: DocumentFenceV1,
    pub(crate) start: DirectBondEndpointIntentV2,
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
    #[must_use]
    pub const fn presentation(&self) -> DocumentBondPresentationV1 {
        self.presentation
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondAdmissionV2 {
    pub(crate) capability: DirectBondGestureCapabilityV1,
    pub(crate) fence: DocumentFenceV1,
    pub(crate) candidate: DirectBondAdmittedCandidateV2,
    overlay: DirectBondOverlayV2,
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
    },
    NewExisting {
        new_point: DirectBondPoint2V1,
        existing: DocumentObjectIdV1,
        element: String,
        presentation: DocumentBondPresentationV1,
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
        capability: gesture.capability,
        fence: gesture.fence,
        candidate,
        overlay: DirectBondOverlayV2 {
            start,
            end,
            presentation: gesture.presentation,
        },
    }
}
