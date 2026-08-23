//! Shared direct-bond planning primitives.

use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentFenceV1 {
    revision: u64,
    digest: [u8; 32],
}

impl DocumentFenceV1 {
    #[must_use]
    pub const fn new(revision: u64, digest: [u8; 32]) -> Self {
        Self { revision, digest }
    }

    #[must_use]
    pub const fn revision(self) -> u64 {
        self.revision
    }

    #[must_use]
    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectBondPoint2V1 {
    x: f64,
    y: f64,
}

impl DirectBondPoint2V1 {
    pub fn new(x: f64, y: f64) -> Result<Self, DirectBondGestureErrorV1> {
        if x.is_finite() && y.is_finite() {
            Ok(Self { x, y })
        } else {
            Err(DirectBondGestureErrorV1::NonFinitePoint)
        }
    }

    #[must_use]
    pub const fn x(self) -> f64 {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> f64 {
        self.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectBondSnapPolicyV1 {
    hex_grid: bool,
    angle_increment_degrees: Option<u16>,
    fixed_length_pt: Option<f64>,
}

impl DirectBondSnapPolicyV1 {
    pub fn new(
        hex_grid: bool,
        angle_increment_degrees: Option<u16>,
        fixed_length_pt: Option<f64>,
    ) -> Result<Self, DirectBondGestureErrorV1> {
        if !matches!(angle_increment_degrees, None | Some(15 | 30 | 45))
            || fixed_length_pt.is_some_and(|value| !value.is_finite() || value <= 0.0)
        {
            return Err(DirectBondGestureErrorV1::InvalidSnapPolicy);
        }
        Ok(Self {
            hex_grid,
            angle_increment_degrees,
            fixed_length_pt,
        })
    }

    #[must_use]
    pub const fn free() -> Self {
        Self {
            hex_grid: false,
            angle_increment_degrees: None,
            fixed_length_pt: None,
        }
    }

    #[must_use]
    pub const fn hex_grid(self) -> bool {
        self.hex_grid
    }

    #[must_use]
    pub const fn angle_increment_degrees(self) -> Option<u16> {
        self.angle_increment_degrees
    }

    #[must_use]
    pub const fn fixed_length_pt(self) -> Option<f64> {
        self.fixed_length_pt
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum DirectBondGestureErrorV1 {
    #[error("direct bond gesture revision is stale")]
    StaleRevision,
    #[error("direct bond gesture digest is stale")]
    StaleDigest,
    #[error("direct bond gesture belongs to a different document session")]
    ForeignSession,
    #[error("direct bond gesture was already redeemed")]
    ReplayedGesture,
    #[error("direct bond gesture start atom is unknown or unsupported")]
    UnknownStartAtom,
    #[error("direct bond gesture end atom is unknown or unsupported")]
    UnknownEndAtom,
    #[error(
        "direct bond gesture accepts normal single, double, or triple bonds, solid-wedge bonds, or hashed-wedge bonds only"
    )]
    UnsupportedPresentation,
    #[error("direct bond gesture cannot join an atom to itself")]
    SelfLoop,
    #[error("direct bond gesture cannot join atoms from different molecules")]
    CrossMolecule,
    #[error("direct bond gesture would duplicate an existing bond")]
    DuplicateBond,
    #[error("direct bond gesture point is not finite")]
    NonFinitePoint,
    #[error("direct bond gesture snapping policy is invalid")]
    InvalidSnapPolicy,
    #[error("direct bond gesture endpoint collapsed onto its start atom")]
    CollapsedEndpoint,
    #[error("direct bond gesture candidate cannot be rendered")]
    UnrenderableCandidate,
    #[error("direct bond gesture candidate exceeds neutral bond capacity")]
    ExceedsChemistryCapacity,
    #[error("direct bond gesture candidate is outside the supported neutral chemistry profile")]
    UnsupportedChemistryAdmission,
    #[error("direct bond gesture commit was rejected by the document session")]
    SessionConflict,
}

/// Closed user-facing refusal taxonomy for direct-bond candidate admission.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum DirectBondAdmissionRefusalV1 {
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
    #[error(
        "direct bond gesture accepts normal single, double, or triple bonds, solid-wedge bonds, or hashed-wedge bonds only"
    )]
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

/// Closed mechanical failures for redeeming an already admitted candidate.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum DirectBondCommitErrorV1 {
    #[error("direct bond admission belongs to a different document session")]
    ForeignSession,
    #[error("direct bond admission receipt was already redeemed")]
    ReplayedReceipt,
    #[error("direct bond admission revision is stale")]
    StaleRevision,
    #[error("direct bond admission digest is stale")]
    StaleDigest,
    #[error("direct bond commit could not allocate a durable identity")]
    IdentityAllocationFailed,
    #[error("direct bond commit could not reserve a provisional token")]
    ProvisionalTokenUnavailable,
    #[error("direct bond commit could not apply its admitted candidate")]
    CandidateApplicationFailed,
    #[error("direct bond session revision space is exhausted")]
    RevisionExhausted,
}
