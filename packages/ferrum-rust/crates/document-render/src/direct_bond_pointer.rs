//! Immutable frontend pointer evidence for direct-bond authoring.

use ferrum_document::{
    DirectBondAdmissionRefusalV1, DirectBondGestureErrorV1, DirectBondPoint2V1, DocumentFenceV1,
    DocumentObjectIdV1,
};
use thiserror::Error;

use crate::direct_bond_lifecycle::DirectBondLifecycleGesture;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectBondPointerHitState {
    None,
    UniqueAtom,
    AmbiguousAtom,
}

/// Finite affine mapping from viewport pixels to scene coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectBondViewportToScene {
    m11: f64,
    m12: f64,
    m21: f64,
    m22: f64,
    dx: f64,
    dy: f64,
}

impl DirectBondViewportToScene {
    pub fn new(
        m11: f64,
        m12: f64,
        m21: f64,
        m22: f64,
        dx: f64,
        dy: f64,
    ) -> Result<Self, DirectBondPointerProbeError> {
        let values = [m11, m12, m21, m22, dx, dy];
        if values.iter().any(|value| !value.is_finite())
            || (m11 * m22 - m12 * m21).abs() <= f64::EPSILON
        {
            return Err(DirectBondPointerProbeError::MalformedTransform);
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
    ) -> Result<DirectBondPoint2V1, DirectBondPointerProbeError> {
        let determinant = self.m11 * self.m22 - self.m12 * self.m21;
        let translated_x = point.x() - self.dx;
        let translated_y = point.y() - self.dy;
        DirectBondPoint2V1::new(
            (self.m22 * translated_x - self.m21 * translated_y) / determinant,
            (-self.m12 * translated_x + self.m11 * translated_y) / determinant,
        )
        .map_err(|_| DirectBondPointerProbeError::MalformedTransform)
    }
}

/// One frozen pointer observation for a direct-bond endpoint.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondPointerProbe {
    pub(crate) scene_point: DirectBondPoint2V1,
    pub(crate) viewport_to_scene: DirectBondViewportToScene,
    pub(crate) direct_hit_state: DirectBondPointerHitState,
    pub(crate) direct_atom_object_id: Option<DocumentObjectIdV1>,
}

impl DirectBondPointerProbe {
    pub fn new(
        scene_x: f64,
        scene_y: f64,
        viewport_to_scene: DirectBondViewportToScene,
        direct_hit_state: DirectBondPointerHitState,
        direct_atom_object_id: Option<DocumentObjectIdV1>,
    ) -> Result<Self, DirectBondPointerProbeError> {
        let scene_point = DirectBondPoint2V1::new(scene_x, scene_y)
            .map_err(|_| DirectBondPointerProbeError::NonFiniteScenePoint)?;
        match (direct_hit_state, direct_atom_object_id.is_some()) {
            (DirectBondPointerHitState::UniqueAtom, true)
            | (DirectBondPointerHitState::None | DirectBondPointerHitState::AmbiguousAtom, false) =>
                {}
            _ => return Err(DirectBondPointerProbeError::InvalidHitEvidence),
        }
        Ok(Self {
            scene_point,
            viewport_to_scene,
            direct_hit_state,
            direct_atom_object_id,
        })
    }
}

/// Closed direct-bond pointer-probe refusal contract.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DirectBondPointerProbeError {
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
pub enum DirectBondPointerProbeCategory {
    NonFiniteScenePoint,
    MalformedTransform,
    InvalidHitEvidence,
    UnknownDirectAtom,
    AmbiguousAtom,
    StaleRevision,
    StaleDigest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectBondPointerProbeRecovery {
    CorrectInput,
    AdjustEndpoint,
    RefreshAndRestart,
}

/// Closed semantic refusal contract after a direct-bond pointer endpoint resolves.
///
/// Pointer evidence failures and document-admission failures deliberately remain
/// separate: a valid pointer can name an endpoint that the document must refuse.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DirectBondAdmissionRefusal {
    #[error("direct bond gesture belongs to a different document session")]
    ForeignSession,
    #[error("direct bond gesture was already redeemed")]
    Consumed,
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
pub enum DirectBondAdmissionCategory {
    ForeignSession,
    Consumed,
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
pub enum DirectBondAdmissionRecovery {
    RefreshAndRestart,
    AdjustEndpoint,
    ChangePresentation,
}

impl From<DirectBondAdmissionRefusalV1> for DirectBondAdmissionRefusal {
    fn from(value: DirectBondAdmissionRefusalV1) -> Self {
        match value {
            DirectBondAdmissionRefusalV1::ForeignSession => Self::ForeignSession,
            DirectBondAdmissionRefusalV1::Consumed => Self::Consumed,
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

impl DirectBondAdmissionRefusal {
    #[must_use]
    pub const fn category(self) -> DirectBondAdmissionCategory {
        match self {
            Self::ForeignSession => DirectBondAdmissionCategory::ForeignSession,
            Self::Consumed => DirectBondAdmissionCategory::Consumed,
            Self::StaleRevision => DirectBondAdmissionCategory::StaleRevision,
            Self::StaleDigest => DirectBondAdmissionCategory::StaleDigest,
            Self::UnknownStartAtom => DirectBondAdmissionCategory::UnknownStartAtom,
            Self::UnknownEndAtom => DirectBondAdmissionCategory::UnknownEndAtom,
            Self::UnsupportedPresentation => DirectBondAdmissionCategory::UnsupportedPresentation,
            Self::InvalidEndpointInput => DirectBondAdmissionCategory::InvalidEndpointInput,
            Self::CollapsedEndpoint => DirectBondAdmissionCategory::CollapsedEndpoint,
            Self::SelfLoop => DirectBondAdmissionCategory::SelfLoop,
            Self::CrossMolecule => DirectBondAdmissionCategory::CrossMolecule,
            Self::DuplicateBond => DirectBondAdmissionCategory::DuplicateBond,
            Self::ExceedsChemistryCapacity => DirectBondAdmissionCategory::ExceedsChemistryCapacity,
            Self::UnsupportedChemistryAdmission => {
                DirectBondAdmissionCategory::UnsupportedChemistryAdmission
            }
            Self::UnrenderableCandidate => DirectBondAdmissionCategory::UnrenderableCandidate,
        }
    }

    #[must_use]
    pub const fn recovery(self) -> DirectBondAdmissionRecovery {
        match self {
            Self::ForeignSession | Self::Consumed | Self::StaleRevision | Self::StaleDigest => {
                DirectBondAdmissionRecovery::RefreshAndRestart
            }
            Self::UnsupportedPresentation | Self::UnrenderableCandidate => {
                DirectBondAdmissionRecovery::ChangePresentation
            }
            Self::UnknownStartAtom
            | Self::UnknownEndAtom
            | Self::InvalidEndpointInput
            | Self::CollapsedEndpoint
            | Self::SelfLoop
            | Self::CrossMolecule
            | Self::DuplicateBond
            | Self::ExceedsChemistryCapacity
            | Self::UnsupportedChemistryAdmission => DirectBondAdmissionRecovery::AdjustEndpoint,
        }
    }
}

/// Direct-bond admission can fail while resolving pointer evidence or while admitting a
/// valid resolved endpoint into the document.
#[derive(Debug, Error)]
pub enum DirectBondAdmissionError {
    #[error(transparent)]
    PointerProbe(#[from] DirectBondPointerProbeError),
    #[error(transparent)]
    Refusal(#[from] DirectBondAdmissionRefusal),
    #[error(transparent)]
    DocumentGesture(DirectBondGestureErrorV1),
}

impl DirectBondPointerProbeError {
    #[must_use]
    pub const fn category(self) -> DirectBondPointerProbeCategory {
        match self {
            Self::NonFiniteScenePoint => DirectBondPointerProbeCategory::NonFiniteScenePoint,
            Self::MalformedTransform => DirectBondPointerProbeCategory::MalformedTransform,
            Self::InvalidHitEvidence => DirectBondPointerProbeCategory::InvalidHitEvidence,
            Self::UnknownDirectAtom => DirectBondPointerProbeCategory::UnknownDirectAtom,
            Self::AmbiguousAtom => DirectBondPointerProbeCategory::AmbiguousAtom,
            Self::StaleRevision => DirectBondPointerProbeCategory::StaleRevision,
            Self::StaleDigest => DirectBondPointerProbeCategory::StaleDigest,
        }
    }

    #[must_use]
    pub const fn recovery(self) -> DirectBondPointerProbeRecovery {
        match self {
            Self::UnknownDirectAtom | Self::AmbiguousAtom => {
                DirectBondPointerProbeRecovery::AdjustEndpoint
            }
            Self::StaleRevision | Self::StaleDigest => {
                DirectBondPointerProbeRecovery::RefreshAndRestart
            }
            Self::NonFiniteScenePoint | Self::MalformedTransform | Self::InvalidHitEvidence => {
                DirectBondPointerProbeRecovery::CorrectInput
            }
        }
    }
}

/// Opaque direct-bond gesture retaining the resolved pointer press.
#[derive(Debug)]
pub struct DirectBondGesture {
    pub(crate) gesture: DirectBondLifecycleGesture,
    pub(crate) fence: DocumentFenceV1,
}
