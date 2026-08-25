//! Portable complete-document render admission vocabulary.
//!
//! The document crate owns candidate derivation. The renderer only classifies
//! this detached immutable value and returns a closed refusal vocabulary. No
//! value in this module carries renderer acceptance or session authority.

/// Fixed schema for complete-document render admission candidates.
pub const COMPLETE_DOCUMENT_ADMISSION_SCHEMA_V1: &str = "ferrum-complete-document-admission-v1";

/// Immutable fence for the exact document observation from which a candidate
/// was derived.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompleteDocumentSourceFenceV1 {
    issuer: u64,
    revision: u64,
    digest: [u8; 32],
}

impl CompleteDocumentSourceFenceV1 {
    /// Construct an exact document-issued source fence.
    #[must_use]
    pub const fn new(issuer: u64, revision: u64, digest: [u8; 32]) -> Self {
        Self {
            issuer,
            revision,
            digest,
        }
    }

    /// Return the document issuer identifier.
    #[must_use]
    pub const fn issuer(self) -> u64 {
        self.issuer
    }

    /// Return the immutable document revision.
    #[must_use]
    pub const fn revision(self) -> u64 {
        self.revision
    }

    /// Return the exact observation digest.
    #[must_use]
    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

/// Document-minted identity for one pending transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompleteRenderPendingIdentityV1 {
    issuer: u64,
    sequence: u64,
}

impl CompleteRenderPendingIdentityV1 {
    /// Construct one exact pending-transition identity.
    #[must_use]
    pub const fn new(issuer: u64, sequence: u64) -> Self {
        Self { issuer, sequence }
    }
}

/// Durable identity of a document root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteRenderRootIdentityV1(String);

impl CompleteRenderRootIdentityV1 {
    /// Construct one nonblank durable root identity.
    pub fn new(value: impl Into<String>) -> Result<Self, CandidateDerivationFailureV1> {
        let value = value.into();
        if value.trim().is_empty() || value.chars().any(char::is_control) {
            return Err(CandidateDerivationFailureV1::InvalidRootIdentity);
        }
        Ok(Self(value))
    }

    /// Return the exact document-issued durable identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Closed document-side failures that prevent candidate derivation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateDerivationFailureV1 {
    /// The frozen observation deliberately suppresses complete rendering.
    SuppressedObservation,
    /// The frozen observation has incompatible immutable facts.
    InconsistentObservation,
    /// A root lacks a valid durable identity.
    InvalidRootIdentity,
    /// A mandatory immutable lowering fact is absent.
    MissingRequiredRenderFact,
    /// Candidate construction exceeded the document-owned resource budget.
    ResourceLimit,
}

/// Closed primitive facts preserved for one visual root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompleteRenderPrimitiveV1 {
    /// A verified molecule primitive is available.
    Molecule,
    /// A verified text primitive is available.
    Text,
    /// A verified vector primitive is available.
    Vector,
}

/// Immutable lowering status for one document root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompleteRenderRootLoweringV1 {
    /// The root has a visual primitive suitable for complete rendering.
    Visual(CompleteRenderPrimitiveV1),
    /// The root is valid in the source model but has no visual primitive.
    Nonvisual,
    /// A visual root is missing its required verified primitive.
    MissingRequiredPrimitive,
}

/// One exact direct root in canonical paint order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteRenderRootCandidateV1 {
    identity: CompleteRenderRootIdentityV1,
    paint_order: u32,
    lowering: CompleteRenderRootLoweringV1,
}

impl CompleteRenderRootCandidateV1 {
    /// Construct one immutable direct-root lowering fact.
    #[must_use]
    pub const fn new(
        identity: CompleteRenderRootIdentityV1,
        paint_order: u32,
        lowering: CompleteRenderRootLoweringV1,
    ) -> Self {
        Self {
            identity,
            paint_order,
            lowering,
        }
    }

    /// Return the durable root identity.
    #[must_use]
    pub const fn identity(&self) -> &CompleteRenderRootIdentityV1 {
        &self.identity
    }

    /// Return the canonical paint order.
    #[must_use]
    pub const fn paint_order(&self) -> u32 {
        self.paint_order
    }

    /// Return the exact immutable lowering status.
    #[must_use]
    pub const fn lowering(&self) -> CompleteRenderRootLoweringV1 {
        self.lowering
    }
}

/// Exact detached candidate for one complete-document render transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentCompleteRenderCandidateV1 {
    source_fence: CompleteDocumentSourceFenceV1,
    pending_identity: CompleteRenderPendingIdentityV1,
    roots: Vec<CompleteRenderRootCandidateV1>,
}

impl DocumentCompleteRenderCandidateV1 {
    /// Construct a complete candidate with strictly increasing root paint order.
    pub fn new(
        source_fence: CompleteDocumentSourceFenceV1,
        pending_identity: CompleteRenderPendingIdentityV1,
        roots: Vec<CompleteRenderRootCandidateV1>,
    ) -> Result<Self, CandidateDerivationFailureV1> {
        if roots
            .windows(2)
            .any(|pair| pair[0].paint_order() >= pair[1].paint_order())
        {
            return Err(CandidateDerivationFailureV1::InconsistentObservation);
        }
        Ok(Self {
            source_fence,
            pending_identity,
            roots,
        })
    }

    /// Return the immutable source-observation fence.
    #[must_use]
    pub const fn source_fence(&self) -> CompleteDocumentSourceFenceV1 {
        self.source_fence
    }

    /// Return the document-minted pending identity.
    #[must_use]
    pub const fn pending_identity(&self) -> CompleteRenderPendingIdentityV1 {
        self.pending_identity
    }

    /// Return root facts in exact canonical paint order.
    #[must_use]
    pub fn roots(&self) -> &[CompleteRenderRootCandidateV1] {
        &self.roots
    }
}

/// Closed classification of a document root for complete rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompleteRenderRootClassV1 {
    /// A molecule root has a complete visual primitive.
    VisualMolecule,
    /// A text root has a complete visual primitive.
    VisualText,
    /// A vector root has a complete visual primitive.
    VisualVector,
    /// An explicitly allowed, intentionally nonvisual root.
    AllowedNonvisual(AllowedNonvisualRootReasonV1),
    /// A root is not admitted for complete rendering.
    Refused(RefusedRootReasonV1),
}

/// V1 intentionally has no allowed nonvisual root categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllowedNonvisualRootReasonV1 {}

/// Closed reason a root cannot participate in complete rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefusedRootReasonV1 {
    /// The root kind has no V1 renderer mapping.
    UnsupportedRootKind,
    /// The root has invalid visual geometry.
    InvalidGeometry,
    /// The root lacks its required verified layout.
    MissingVerifiedLayout,
    /// The current complete-render profile excludes this root.
    ProfileExcluded,
    /// The root lacks the primitive required for visual rendering.
    MissingRequiredPrimitive,
}

/// Closed refusal returned before any document transition can be redeemed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompleteRenderAdmissionRefusalV1 {
    /// Candidate derivation failed before renderer lowering.
    CandidateDerivation(CandidateDerivationFailureV1),
    /// One root is refused by the complete-render profile.
    RootRefused {
        /// Durable root identity.
        root: CompleteRenderRootIdentityV1,
        /// Always `CompleteRenderRootClassV1::Refused`.
        class: CompleteRenderRootClassV1,
    },
    /// A later exact candidate re-derivation differs from its accepted candidate.
    CandidateMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_rejects_nonincreasing_paint_order() {
        let first = CompleteRenderRootCandidateV1::new(
            CompleteRenderRootIdentityV1::new("root-a").expect("identity"),
            2,
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Molecule),
        );
        let second = CompleteRenderRootCandidateV1::new(
            CompleteRenderRootIdentityV1::new("root-b").expect("identity"),
            2,
            CompleteRenderRootLoweringV1::Visual(CompleteRenderPrimitiveV1::Text),
        );
        assert_eq!(
            DocumentCompleteRenderCandidateV1::new(
                CompleteDocumentSourceFenceV1::new(1, 1, [0; 32]),
                CompleteRenderPendingIdentityV1::new(1, 1),
                vec![first, second],
            ),
            Err(CandidateDerivationFailureV1::InconsistentObservation)
        );
    }
}
