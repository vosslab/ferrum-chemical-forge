//! Immutable frontend pointer evidence for direct-bond authoring.

use ferrum_document::{
    DirectBondAdmissionRefusalV1, DirectBondGestureErrorV1, DirectBondPoint2V1,
    DocumentBondPresentationV1, DocumentFenceV1,
};
use thiserror::Error;

use crate::direct_bond_v3_lifecycle::{
    CommittedDirectBondGesture, DirectBondAdmission, DirectBondGesture, DirectBondOverlay,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectBondPointerHitStateV3 {
    None,
    UniqueAtom,
    AmbiguousAtom,
}

/// Finite affine mapping from viewport pixels to scene coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectBondViewportToSceneV3 {
    m11: f64,
    m12: f64,
    m21: f64,
    m22: f64,
    dx: f64,
    dy: f64,
}

impl DirectBondViewportToSceneV3 {
    pub fn new(
        m11: f64,
        m12: f64,
        m21: f64,
        m22: f64,
        dx: f64,
        dy: f64,
    ) -> Result<Self, DirectBondPointerProbeErrorV3> {
        let values = [m11, m12, m21, m22, dx, dy];
        if values.iter().any(|value| !value.is_finite())
            || (m11 * m22 - m12 * m21).abs() <= f64::EPSILON
        {
            return Err(DirectBondPointerProbeErrorV3::MalformedTransform);
        }
        Ok(Self {
            m11,
            m12,
            m21,
            m22,
            dx,
            dy,
        })
    }

    pub(crate) fn viewport_point_for(
        self,
        point: DirectBondPoint2V1,
    ) -> Result<DirectBondPoint2V1, DirectBondPointerProbeErrorV3> {
        let determinant = self.m11 * self.m22 - self.m12 * self.m21;
        let translated_x = point.x() - self.dx;
        let translated_y = point.y() - self.dy;
        DirectBondPoint2V1::new(
            (self.m22 * translated_x - self.m21 * translated_y) / determinant,
            (-self.m12 * translated_x + self.m11 * translated_y) / determinant,
        )
        .map_err(|_| DirectBondPointerProbeErrorV3::MalformedTransform)
    }
}

/// One frozen pointer observation for a direct-bond endpoint.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondPointerProbeV3 {
    pub(crate) scene_point: DirectBondPoint2V1,
    pub(crate) viewport_to_scene: DirectBondViewportToSceneV3,
    pub(crate) direct_hit_state: DirectBondPointerHitStateV3,
    pub(crate) direct_atom_source_id: Option<String>,
}

impl DirectBondPointerProbeV3 {
    pub fn new(
        scene_x: f64,
        scene_y: f64,
        viewport_to_scene: DirectBondViewportToSceneV3,
        direct_hit_state: DirectBondPointerHitStateV3,
        direct_atom_source_id: Option<String>,
    ) -> Result<Self, DirectBondPointerProbeErrorV3> {
        let scene_point = DirectBondPoint2V1::new(scene_x, scene_y)
            .map_err(|_| DirectBondPointerProbeErrorV3::NonFiniteScenePoint)?;
        match (direct_hit_state, direct_atom_source_id.is_some()) {
            (DirectBondPointerHitStateV3::UniqueAtom, true)
            | (
                DirectBondPointerHitStateV3::None | DirectBondPointerHitStateV3::AmbiguousAtom,
                false,
            ) => {}
            _ => return Err(DirectBondPointerProbeErrorV3::InvalidHitEvidence),
        }
        Ok(Self {
            scene_point,
            viewport_to_scene,
            direct_hit_state,
            direct_atom_source_id,
        })
    }
}

/// Closed direct-bond pointer-probe refusal contract.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DirectBondPointerProbeErrorV3 {
    #[error("direct-bond pointer scene coordinate is not finite")]
    NonFiniteScenePoint,
    #[error("direct-bond viewport-to-scene transform is malformed")]
    MalformedTransform,
    #[error("direct-bond pointer hit evidence is inconsistent")]
    InvalidHitEvidence,
    #[error("direct-bond direct atom identity is unknown, stale, or not an atom")]
    UnknownDirectAtom,
    #[error("direct-bond pointer endpoint is ambiguous")]
    AmbiguousAtom,
    #[error("direct-bond source revision is stale")]
    StaleRevision,
    #[error("direct-bond source digest is stale")]
    StaleDigest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectBondPointerProbeCategoryV3 {
    NonFiniteScenePoint,
    MalformedTransform,
    InvalidHitEvidence,
    UnknownDirectAtom,
    AmbiguousAtom,
    StaleRevision,
    StaleDigest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectBondPointerProbeRecoveryV3 {
    CorrectInput,
    AdjustEndpoint,
    RefreshAndRestart,
}

/// Closed semantic refusal contract after a V3 pointer endpoint has resolved.
///
/// Pointer evidence failures and document-admission failures deliberately remain
/// separate: a valid pointer can name an endpoint that the document must refuse.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DirectBondAdmissionRefusalV3 {
    #[error("direct bond gesture belongs to a different document session")]
    ForeignSession,
    #[error("direct bond gesture was already redeemed")]
    ReplayedGesture,
    #[error("direct bond gesture revision is stale")]
    StaleRevision,
    #[error("direct bond gesture digest is stale")]
    StaleDigest,
    #[error("direct bond gesture start atom is unknown or unsupported")]
    UnknownStartAtom,
    #[error("direct bond gesture end atom is unknown or unsupported")]
    UnknownEndAtom,
    #[error("direct bond gesture presentation is unsupported")]
    UnsupportedPresentation,
    #[error("direct bond gesture endpoint input is invalid")]
    InvalidEndpointInput,
    #[error("direct bond gesture endpoint collapsed onto its start atom")]
    CollapsedEndpoint,
    #[error("direct bond gesture cannot join an atom to itself")]
    SelfLoop,
    #[error("direct bond gesture cannot join atoms from different molecules")]
    CrossMolecule,
    #[error("direct bond gesture would duplicate an existing bond")]
    DuplicateBond,
    #[error("direct bond gesture candidate exceeds neutral bond capacity")]
    ExceedsChemistryCapacity,
    #[error("direct bond gesture candidate is outside the supported neutral chemistry profile")]
    UnsupportedChemistryAdmission,
    #[error("direct bond gesture candidate cannot be rendered")]
    UnrenderableCandidate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectBondAdmissionCategoryV3 {
    ForeignSession,
    ReplayedGesture,
    StaleRevision,
    StaleDigest,
    UnknownStartAtom,
    UnknownEndAtom,
    UnsupportedPresentation,
    InvalidEndpointInput,
    CollapsedEndpoint,
    SelfLoop,
    CrossMolecule,
    DuplicateBond,
    ExceedsChemistryCapacity,
    UnsupportedChemistryAdmission,
    UnrenderableCandidate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectBondAdmissionRecoveryV3 {
    RefreshAndRestart,
    AdjustEndpoint,
    ChangePresentation,
}

impl From<DirectBondAdmissionRefusalV1> for DirectBondAdmissionRefusalV3 {
    fn from(value: DirectBondAdmissionRefusalV1) -> Self {
        match value {
            DirectBondAdmissionRefusalV1::ForeignSession => Self::ForeignSession,
            DirectBondAdmissionRefusalV1::ReplayedGesture => Self::ReplayedGesture,
            DirectBondAdmissionRefusalV1::StaleRevision => Self::StaleRevision,
            DirectBondAdmissionRefusalV1::StaleDigest => Self::StaleDigest,
            DirectBondAdmissionRefusalV1::UnknownStartAtom => Self::UnknownStartAtom,
            DirectBondAdmissionRefusalV1::UnknownEndAtom => Self::UnknownEndAtom,
            DirectBondAdmissionRefusalV1::UnsupportedPresentation => Self::UnsupportedPresentation,
            DirectBondAdmissionRefusalV1::InvalidEndpointInput => Self::InvalidEndpointInput,
            DirectBondAdmissionRefusalV1::CollapsedEndpoint => Self::CollapsedEndpoint,
            DirectBondAdmissionRefusalV1::SelfLoop => Self::SelfLoop,
            DirectBondAdmissionRefusalV1::CrossMolecule => Self::CrossMolecule,
            DirectBondAdmissionRefusalV1::DuplicateBond => Self::DuplicateBond,
            DirectBondAdmissionRefusalV1::ExceedsChemistryCapacity => {
                Self::ExceedsChemistryCapacity
            }
            DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission => {
                Self::UnsupportedChemistryAdmission
            }
            DirectBondAdmissionRefusalV1::UnrenderableCandidate => Self::UnrenderableCandidate,
        }
    }
}

impl DirectBondAdmissionRefusalV3 {
    #[must_use]
    pub const fn category(self) -> DirectBondAdmissionCategoryV3 {
        match self {
            Self::ForeignSession => DirectBondAdmissionCategoryV3::ForeignSession,
            Self::ReplayedGesture => DirectBondAdmissionCategoryV3::ReplayedGesture,
            Self::StaleRevision => DirectBondAdmissionCategoryV3::StaleRevision,
            Self::StaleDigest => DirectBondAdmissionCategoryV3::StaleDigest,
            Self::UnknownStartAtom => DirectBondAdmissionCategoryV3::UnknownStartAtom,
            Self::UnknownEndAtom => DirectBondAdmissionCategoryV3::UnknownEndAtom,
            Self::UnsupportedPresentation => DirectBondAdmissionCategoryV3::UnsupportedPresentation,
            Self::InvalidEndpointInput => DirectBondAdmissionCategoryV3::InvalidEndpointInput,
            Self::CollapsedEndpoint => DirectBondAdmissionCategoryV3::CollapsedEndpoint,
            Self::SelfLoop => DirectBondAdmissionCategoryV3::SelfLoop,
            Self::CrossMolecule => DirectBondAdmissionCategoryV3::CrossMolecule,
            Self::DuplicateBond => DirectBondAdmissionCategoryV3::DuplicateBond,
            Self::ExceedsChemistryCapacity => {
                DirectBondAdmissionCategoryV3::ExceedsChemistryCapacity
            }
            Self::UnsupportedChemistryAdmission => {
                DirectBondAdmissionCategoryV3::UnsupportedChemistryAdmission
            }
            Self::UnrenderableCandidate => DirectBondAdmissionCategoryV3::UnrenderableCandidate,
        }
    }

    #[must_use]
    pub const fn recovery(self) -> DirectBondAdmissionRecoveryV3 {
        match self {
            Self::ForeignSession
            | Self::ReplayedGesture
            | Self::StaleRevision
            | Self::StaleDigest => DirectBondAdmissionRecoveryV3::RefreshAndRestart,
            Self::UnsupportedPresentation | Self::UnrenderableCandidate => {
                DirectBondAdmissionRecoveryV3::ChangePresentation
            }
            Self::UnknownStartAtom
            | Self::UnknownEndAtom
            | Self::InvalidEndpointInput
            | Self::CollapsedEndpoint
            | Self::SelfLoop
            | Self::CrossMolecule
            | Self::DuplicateBond
            | Self::ExceedsChemistryCapacity
            | Self::UnsupportedChemistryAdmission => DirectBondAdmissionRecoveryV3::AdjustEndpoint,
        }
    }
}

/// V3 admission can fail while resolving pointer evidence or while admitting a
/// valid resolved endpoint into the document.
#[derive(Debug, Error)]
pub enum DirectBondAdmissionErrorV3 {
    #[error(transparent)]
    PointerProbe(#[from] DirectBondPointerProbeErrorV3),
    #[error(transparent)]
    Refusal(#[from] DirectBondAdmissionRefusalV3),
    #[error(transparent)]
    DocumentGesture(DirectBondGestureErrorV1),
}

impl DirectBondPointerProbeErrorV3 {
    #[must_use]
    pub const fn category(self) -> DirectBondPointerProbeCategoryV3 {
        match self {
            Self::NonFiniteScenePoint => DirectBondPointerProbeCategoryV3::NonFiniteScenePoint,
            Self::MalformedTransform => DirectBondPointerProbeCategoryV3::MalformedTransform,
            Self::InvalidHitEvidence => DirectBondPointerProbeCategoryV3::InvalidHitEvidence,
            Self::UnknownDirectAtom => DirectBondPointerProbeCategoryV3::UnknownDirectAtom,
            Self::AmbiguousAtom => DirectBondPointerProbeCategoryV3::AmbiguousAtom,
            Self::StaleRevision => DirectBondPointerProbeCategoryV3::StaleRevision,
            Self::StaleDigest => DirectBondPointerProbeCategoryV3::StaleDigest,
        }
    }

    #[must_use]
    pub const fn recovery(self) -> DirectBondPointerProbeRecoveryV3 {
        match self {
            Self::UnknownDirectAtom | Self::AmbiguousAtom => {
                DirectBondPointerProbeRecoveryV3::AdjustEndpoint
            }
            Self::StaleRevision | Self::StaleDigest => {
                DirectBondPointerProbeRecoveryV3::RefreshAndRestart
            }
            Self::NonFiniteScenePoint | Self::MalformedTransform | Self::InvalidHitEvidence => {
                DirectBondPointerProbeRecoveryV3::CorrectInput
            }
        }
    }
}

/// Opaque V3 direct-bond gesture retaining the resolved pointer press.
#[derive(Clone, Debug)]
pub struct DirectBondGestureV3 {
    pub(crate) gesture: DirectBondGesture,
    pub(crate) fence: DocumentFenceV1,
}

/// Opaque V3 direct-bond admission retaining renderer-preflighted operations.
#[derive(Debug)]
pub struct DirectBondAdmissionV3 {
    pub(crate) admission: DirectBondAdmission,
    pub(crate) overlay: DirectBondOverlayV3,
}

/// Renderer-issued V3 overlay facts for one admitted pointer gesture.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondOverlayV3 {
    pub(crate) overlay: DirectBondOverlay,
}

impl DirectBondOverlayV3 {
    #[must_use]
    pub const fn start_x(&self) -> f64 {
        self.overlay.start_x()
    }
    #[must_use]
    pub const fn start_y(&self) -> f64 {
        self.overlay.start_y()
    }
    #[must_use]
    pub const fn end_x(&self) -> f64 {
        self.overlay.end_x()
    }
    #[must_use]
    pub const fn end_y(&self) -> f64 {
        self.overlay.end_y()
    }
    #[must_use]
    pub const fn presentation(&self) -> DocumentBondPresentationV1 {
        self.overlay.presentation()
    }
    #[must_use]
    pub fn operations(&self) -> &[ferrum_render::RenderOp] {
        self.overlay.operations()
    }
}

impl DirectBondAdmissionV3 {
    #[must_use]
    pub const fn overlay(&self) -> &DirectBondOverlayV3 {
        &self.overlay
    }
}

/// Durable V3 outcome for a committed direct-bond pointer gesture.
#[derive(Clone, Debug)]
pub struct CommittedDirectBondGestureV3 {
    pub(crate) committed: CommittedDirectBondGesture,
}

impl CommittedDirectBondGestureV3 {
    #[must_use]
    pub fn bond(&self) -> &ferrum_document::PersistentId {
        self.committed.bond()
    }
    #[must_use]
    pub fn end_atom(&self) -> &ferrum_document::PersistentId {
        self.committed.end_atom()
    }
    #[must_use]
    pub fn second_created_atom(&self) -> Option<&ferrum_document::PersistentId> {
        self.committed.second_created_atom()
    }
    #[must_use]
    pub const fn created_new_atom(&self) -> bool {
        self.committed.created_new_atom()
    }
    #[must_use]
    pub const fn created_new_molecule(&self) -> bool {
        self.committed.created_new_molecule()
    }
    #[must_use]
    pub fn result(&self) -> &ferrum_document::SessionOperationResultV1 {
        self.committed.result()
    }
}
