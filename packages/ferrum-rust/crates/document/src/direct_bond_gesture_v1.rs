//! Revision-fenced planning values for one direct normal-bond gesture.

use std::sync::atomic::{AtomicU64, Ordering};

use thiserror::Error;

use crate::{
    DocumentBondPresentationV1, DocumentObjectIdV1, PersistentId, SessionOperationResultV1,
};

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
            || fixed_length_pt.is_some_and(|v| !v.is_finite() || v <= 0.0)
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

#[derive(Clone, Debug, PartialEq)]
pub enum DirectBondEndIntentV1 {
    ExistingAtom { atom: DocumentObjectIdV1 },
    NewAtomAt { raw_point: DirectBondPoint2V1 },
}

/// Private provenance for handles issued by one live document session.
///
/// Document-shaped values cannot forge this token because it never leaves the
/// `ferrum-document` crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DirectBondSessionOriginV1(u64);

/// Private, globally unique capability for one begun direct-bond gesture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DirectBondGestureCapabilityV1 {
    origin: DirectBondSessionOriginV1,
    nonce: u64,
}

impl DirectBondSessionOriginV1 {
    pub(crate) fn issue() -> Self {
        static NEXT_SESSION_ORIGIN: AtomicU64 = AtomicU64::new(1);
        let origin = NEXT_SESSION_ORIGIN
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .expect("direct-bond session-origin space is exhausted");
        Self(origin)
    }

    pub(crate) fn issue_gesture(self) -> DirectBondGestureCapabilityV1 {
        static NEXT_GESTURE_NONCE: AtomicU64 = AtomicU64::new(1);
        let nonce = NEXT_GESTURE_NONCE
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .expect("direct-bond gesture-capability space is exhausted");
        DirectBondGestureCapabilityV1 {
            origin: self,
            nonce,
        }
    }
}

impl DirectBondGestureCapabilityV1 {
    pub(crate) fn belongs_to(self, origin: DirectBondSessionOriginV1) -> bool {
        self.origin == origin
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondGestureV1 {
    pub(crate) capability: DirectBondGestureCapabilityV1,
    pub(crate) fence: DocumentFenceV1,
    pub(crate) start_atom: DocumentObjectIdV1,
    pub(crate) start_molecule: PersistentId,
    pub(crate) presentation: DocumentBondPresentationV1,
    pub(crate) new_atom_element: String,
    pub(crate) snap: DirectBondSnapPolicyV1,
    pub(crate) start_point: DirectBondPoint2V1,
}
impl DirectBondGestureV1 {
    #[must_use]
    pub const fn fence(&self) -> DocumentFenceV1 {
        self.fence
    }
    #[must_use]
    pub fn start_atom(&self) -> &DocumentObjectIdV1 {
        &self.start_atom
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DirectBondEndpointV1 {
    ExistingAtom {
        atom: DocumentObjectIdV1,
        point: DirectBondPoint2V1,
    },
    NewAtom {
        point: DirectBondPoint2V1,
        element: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondOverlayV1 {
    start: DirectBondPoint2V1,
    end: DirectBondPoint2V1,
    presentation: DocumentBondPresentationV1,
    endpoint_is_new: bool,
}
impl DirectBondOverlayV1 {
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
    #[must_use]
    pub const fn endpoint_is_new(&self) -> bool {
        self.endpoint_is_new
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondPreviewV1 {
    pub(crate) admission: DirectBondAdmissionV1,
}
impl DirectBondPreviewV1 {
    #[must_use]
    pub fn overlay(&self) -> &DirectBondOverlayV1 {
        self.admission.overlay()
    }
}

/// Opaque, revision-fenced admission for one complete direct normal-bond candidate.
///
/// This receipt deliberately carries no generated identifiers, provisional token, or
/// mutable document state. It can only be redeemed by its issuing session.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondAdmissionV1 {
    pub(crate) capability: DirectBondGestureCapabilityV1,
    pub(crate) fence: DocumentFenceV1,
    pub(crate) candidate: DirectBondAdmittedCandidateV1,
    overlay: DirectBondOverlayV1,
}
impl DirectBondAdmissionV1 {
    #[must_use]
    pub fn overlay(&self) -> &DirectBondOverlayV1 {
        &self.overlay
    }
}

/// Complete semantic direct-bond change held behind an admission receipt.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum DirectBondAdmittedCandidateV1 {
    ExistingEndpoint {
        start_atom: DocumentObjectIdV1,
        end_atom: DocumentObjectIdV1,
        molecule: PersistentId,
        presentation: DocumentBondPresentationV1,
    },
    NewEndpoint {
        start_atom: DocumentObjectIdV1,
        molecule: PersistentId,
        point: DirectBondPoint2V1,
        element: String,
        presentation: DocumentBondPresentationV1,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommittedDirectBondGestureV1 {
    ExistingEndpoint {
        bond: PersistentId,
        end_atom: PersistentId,
        result: SessionOperationResultV1,
    },
    NewEndpoint {
        bond: PersistentId,
        atom: PersistentId,
        result: SessionOperationResultV1,
    },
}
impl CommittedDirectBondGestureV1 {
    #[must_use]
    pub fn bond(&self) -> &PersistentId {
        match self {
            Self::ExistingEndpoint { bond, .. } | Self::NewEndpoint { bond, .. } => bond,
        }
    }
    #[must_use]
    pub fn end_atom(&self) -> &PersistentId {
        match self {
            Self::ExistingEndpoint { end_atom, .. } => end_atom,
            Self::NewEndpoint { atom, .. } => atom,
        }
    }
    #[must_use]
    pub fn result(&self) -> &SessionOperationResultV1 {
        match self {
            Self::ExistingEndpoint { result, .. } | Self::NewEndpoint { result, .. } => result,
        }
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
    #[error("direct bond gesture start atom is unknown or unsupported")]
    UnknownStartAtom,
    #[error("direct bond gesture end atom is unknown or unsupported")]
    UnknownEndAtom,
    #[error("direct bond gesture accepts normal single, double, or triple bonds only")]
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
    #[error("direct bond preview belongs to a different gesture")]
    PreviewMismatch,
    #[error("direct bond gesture candidate cannot be rendered")]
    UnrenderableCandidate,
    #[error("direct bond gesture candidate exceeds neutral bond capacity")]
    ExceedsChemistryCapacity,
    #[error("direct bond gesture candidate is outside the supported neutral chemistry profile")]
    UnsupportedChemistryAdmission,
    #[error("direct bond gesture commit was rejected by the document session")]
    SessionConflict,
}

/// Closed user-facing refusal taxonomy for pure candidate admission.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum DirectBondAdmissionRefusalV1 {
    #[error("direct bond gesture belongs to a different document session")]
    ForeignSession,
    #[error("direct bond gesture revision is stale")]
    StaleRevision,
    #[error("direct bond gesture digest is stale")]
    StaleDigest,
    #[error("direct bond gesture start atom is unknown or unsupported")]
    UnknownStartAtom,
    #[error("direct bond gesture end atom is unknown or unsupported")]
    UnknownEndAtom,
    #[error("direct bond gesture accepts normal single, double, or triple bonds only")]
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

impl From<DirectBondAdmissionRefusalV1> for DirectBondGestureErrorV1 {
    fn from(value: DirectBondAdmissionRefusalV1) -> Self {
        match value {
            DirectBondAdmissionRefusalV1::ForeignSession => Self::ForeignSession,
            DirectBondAdmissionRefusalV1::StaleRevision => Self::StaleRevision,
            DirectBondAdmissionRefusalV1::StaleDigest => Self::StaleDigest,
            DirectBondAdmissionRefusalV1::UnknownStartAtom => Self::UnknownStartAtom,
            DirectBondAdmissionRefusalV1::UnknownEndAtom => Self::UnknownEndAtom,
            DirectBondAdmissionRefusalV1::UnsupportedPresentation => Self::UnsupportedPresentation,
            DirectBondAdmissionRefusalV1::InvalidEndpointInput => Self::NonFinitePoint,
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

impl From<DirectBondCommitErrorV1> for DirectBondGestureErrorV1 {
    fn from(value: DirectBondCommitErrorV1) -> Self {
        match value {
            DirectBondCommitErrorV1::ForeignSession => Self::ForeignSession,
            DirectBondCommitErrorV1::StaleRevision => Self::StaleRevision,
            DirectBondCommitErrorV1::StaleDigest => Self::StaleDigest,
            DirectBondCommitErrorV1::IdentityAllocationFailed
            | DirectBondCommitErrorV1::ProvisionalTokenUnavailable
            | DirectBondCommitErrorV1::CandidateApplicationFailed
            | DirectBondCommitErrorV1::RevisionExhausted => Self::SessionConflict,
        }
    }
}

pub(crate) fn admission(
    gesture: &DirectBondGestureV1,
    candidate: DirectBondAdmittedCandidateV1,
    end: DirectBondPoint2V1,
    endpoint_is_new: bool,
) -> DirectBondAdmissionV1 {
    DirectBondAdmissionV1 {
        capability: gesture.capability,
        fence: gesture.fence,
        candidate,
        overlay: DirectBondOverlayV1 {
            start: gesture.start_point,
            end,
            presentation: gesture.presentation,
            endpoint_is_new,
        },
    }
}

#[cfg(test)]
mod tests {
    use ferrum_core::BondOrder;

    use crate::{
        DocumentBondOrderV1, DocumentBondPresentationV1, DocumentSession, DocumentSnapshot,
    };

    use super::{
        DirectBondEndIntentV1, DirectBondPoint2V1, DirectBondSnapPolicyV1, DocumentFenceV1,
    };

    const SOURCE: &str = concat!(
        "<cdml><molecule id=\"m\">",
        "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"b\" name=\"C\"><point x=\"20\" y=\"0\"/></atom>",
        "</molecule></cdml>"
    );

    fn source(
        session: &DocumentSession,
    ) -> (
        DocumentFenceV1,
        crate::DocumentObjectIdV1,
        crate::DocumentObjectIdV1,
    ) {
        let snapshot = session.snapshot().expect("source snapshot");
        let observation = session
            .observe(snapshot.revision())
            .expect("source observation");
        let atoms = observation.projection().molecules()[0].atoms();
        (
            DocumentFenceV1::new(snapshot.revision(), *snapshot.digest()),
            atoms[0].id().expect("durable first atom").clone(),
            atoms[1].id().expect("durable second atom").clone(),
        )
    }

    #[test]
    fn existing_endpoint_preview_and_commit_are_one_history_transition() {
        let mut session = DocumentSession::load(SOURCE).expect("session loads");
        let (fence, start, end) = source(&session);
        let gesture = session
            .begin_direct_bond_gesture_v1(
                fence,
                start,
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Double),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("gesture begins");
        let preview = session
            .preview_direct_bond_gesture_v1(
                &gesture,
                DirectBondEndIntentV1::ExistingAtom { atom: end },
            )
            .expect("preview is pure");
        assert_eq!(
            session.snapshot().expect("unchanged snapshot").revision(),
            0
        );
        let committed = session
            .commit_direct_bond_gesture_v1(&gesture, &preview)
            .expect("commit succeeds");
        assert_eq!(committed.result().observation().snapshot().revision(), 1);
        assert!(
            committed
                .result()
                .observation()
                .snapshot()
                .cdml()
                .contains("type=\"n2\"")
        );
        assert_eq!(
            session
                .undo(1)
                .expect("undo succeeds")
                .observation()
                .snapshot()
                .revision(),
            2
        );
    }

    #[test]
    fn new_endpoint_commit_uses_preview_coordinates_and_drop_is_noop() {
        let mut session = DocumentSession::load(SOURCE).expect("session loads");
        let (fence, start, _) = source(&session);
        let gesture = session
            .begin_direct_bond_gesture_v1(
                fence,
                start,
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Triple),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("gesture begins");
        let before: DocumentSnapshot = session.snapshot().expect("snapshot before drop");
        drop(gesture.clone());
        assert_eq!(session.snapshot().expect("snapshot after drop"), before);
        let preview = session
            .preview_direct_bond_gesture_v1(
                &gesture,
                DirectBondEndIntentV1::NewAtomAt {
                    raw_point: DirectBondPoint2V1::new(40.0, 5.0).expect("finite point"),
                },
            )
            .expect("new endpoint preview");
        let committed = session
            .commit_direct_bond_gesture_v1(&gesture, &preview)
            .expect("commit succeeds");
        let cdml = committed.result().observation().snapshot().cdml();
        assert!(cdml.contains("x=\"40\" y=\"5\""));
        assert!(cdml.contains("type=\"n3\""));
    }

    #[test]
    fn session_capability_rejects_byte_identical_foreign_handles() {
        let first = DocumentSession::load(SOURCE).expect("first session loads");
        let mut second = DocumentSession::load(SOURCE).expect("second session loads");
        let (first_fence, first_start, first_end) = source(&first);
        let (second_fence, second_start, _) = source(&second);
        let first_gesture = first
            .begin_direct_bond_gesture_v1(
                first_fence,
                first_start,
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("first gesture begins");
        let preview = first
            .preview_direct_bond_gesture_v1(
                &first_gesture,
                DirectBondEndIntentV1::ExistingAtom {
                    atom: first_end.clone(),
                },
            )
            .expect("first preview succeeds");
        let second_gesture = second
            .begin_direct_bond_gesture_v1(
                second_fence,
                second_start,
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("second gesture begins");

        assert_eq!(
            second.preview_direct_bond_gesture_v1(
                &first_gesture,
                DirectBondEndIntentV1::ExistingAtom { atom: first_end },
            ),
            Err(super::DirectBondGestureErrorV1::ForeignSession)
        );
        assert_eq!(
            second.commit_direct_bond_gesture_v1(&second_gesture, &preview),
            Err(super::DirectBondGestureErrorV1::ForeignSession)
        );
        assert_eq!(
            second
                .snapshot()
                .expect("second remains unchanged")
                .revision(),
            0
        );
    }

    #[test]
    fn distinct_identical_gestures_reject_mixed_and_replayed_previews() {
        let mut session = DocumentSession::load(SOURCE).expect("session loads");
        let (fence, start, end) = source(&session);
        let first_gesture = session
            .begin_direct_bond_gesture_v1(
                fence,
                start.clone(),
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("first gesture begins");
        let second_gesture = session
            .begin_direct_bond_gesture_v1(
                fence,
                start,
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("second gesture begins");
        let preview = session
            .preview_direct_bond_gesture_v1(
                &first_gesture,
                DirectBondEndIntentV1::ExistingAtom { atom: end },
            )
            .expect("first preview succeeds");

        assert_eq!(
            session.commit_direct_bond_gesture_v1(&second_gesture, &preview),
            Err(super::DirectBondGestureErrorV1::PreviewMismatch)
        );
        session
            .commit_direct_bond_gesture_v1(&first_gesture, &preview)
            .expect("first preview commits once");
        assert_eq!(
            session.commit_direct_bond_gesture_v1(&first_gesture, &preview),
            Err(super::DirectBondGestureErrorV1::StaleRevision)
        );
    }

    #[test]
    fn receipt_only_admission_is_pure_and_commits_the_admitted_existing_endpoint() {
        let mut session = DocumentSession::load(SOURCE).expect("session loads");
        let mut control = DocumentSession::load(SOURCE).expect("control session loads");
        session.set_next_generated_bond_sequence_for_test(Some(71));
        control.set_next_generated_bond_sequence_for_test(Some(71));
        let (fence, start, end) = source(&session);
        let gesture = session
            .begin_direct_bond_gesture_v1(
                fence,
                start,
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("gesture begins");
        let before = session.snapshot().expect("snapshot before admission");
        let admission = session
            .admit_direct_bond_candidate_v1(
                &gesture,
                DirectBondEndIntentV1::ExistingAtom { atom: end },
            )
            .expect("candidate admits");

        assert_eq!(session.snapshot().expect("admission is pure"), before);
        assert!(!admission.overlay().endpoint_is_new());
        let committed = session
            .commit_direct_bond_admission_v1(&admission)
            .expect("receipt commits once");
        let (control_fence, control_start, control_end) = source(&control);
        let control_gesture = control
            .begin_direct_bond_gesture_v1(
                control_fence,
                control_start,
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("control gesture begins");
        let control_admission = control
            .admit_direct_bond_candidate_v1(
                &control_gesture,
                DirectBondEndIntentV1::ExistingAtom { atom: control_end },
            )
            .expect("control candidate admits");
        let control_committed = control
            .commit_direct_bond_admission_v1(&control_admission)
            .expect("control receipt commits");
        assert_eq!(committed.bond(), control_committed.bond());
        assert_eq!(committed.result().observation().snapshot().revision(), 1);
        assert_eq!(
            session.commit_direct_bond_admission_v1(&admission),
            Err(super::DirectBondCommitErrorV1::StaleRevision)
        );
    }

    #[test]
    fn receipt_only_admission_preserves_the_snapped_new_endpoint() {
        let mut session = DocumentSession::load(SOURCE).expect("session loads");
        let mut control = DocumentSession::load(SOURCE).expect("control session loads");
        session.set_next_generated_atom_sequence_for_test(Some(71));
        session.set_next_generated_bond_sequence_for_test(Some(71));
        control.set_next_generated_atom_sequence_for_test(Some(71));
        control.set_next_generated_bond_sequence_for_test(Some(71));
        let (fence, start, _) = source(&session);
        let snap = DirectBondSnapPolicyV1::new(false, None, Some(20.0)).expect("valid snap");
        let gesture = session
            .begin_direct_bond_gesture_v1(
                fence,
                start,
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                snap,
            )
            .expect("gesture begins");
        let admission = session
            .admit_direct_bond_candidate_v1(
                &gesture,
                DirectBondEndIntentV1::NewAtomAt {
                    raw_point: DirectBondPoint2V1::new(30.0, 0.0).expect("finite point"),
                },
            )
            .expect("new candidate admits");

        assert!(admission.overlay().endpoint_is_new());
        assert_eq!(admission.overlay().end().x(), 20.0);
        let committed = session
            .commit_direct_bond_admission_v1(&admission)
            .expect("receipt commits once");
        let (control_fence, control_start, _) = source(&control);
        let control_gesture = control
            .begin_direct_bond_gesture_v1(
                control_fence,
                control_start,
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::new(false, None, Some(20.0)).expect("valid snap"),
            )
            .expect("control gesture begins");
        let control_admission = control
            .admit_direct_bond_candidate_v1(
                &control_gesture,
                DirectBondEndIntentV1::NewAtomAt {
                    raw_point: DirectBondPoint2V1::new(30.0, 0.0).expect("finite point"),
                },
            )
            .expect("control candidate admits");
        let control_committed = control
            .commit_direct_bond_admission_v1(&control_admission)
            .expect("control receipt commits");
        assert_eq!(committed.bond(), control_committed.bond());
        assert_eq!(committed.end_atom(), control_committed.end_atom());
        assert!(
            committed
                .result()
                .observation()
                .snapshot()
                .cdml()
                .contains("x=\"20\" y=\"0\"")
        );
    }

    #[test]
    fn refused_and_abandoned_admissions_do_not_allocate_identifiers_or_tokens() {
        let mut session = DocumentSession::load(SOURCE).expect("session loads");
        let mut control = DocumentSession::load(SOURCE).expect("control session loads");
        session.set_next_generated_bond_sequence_for_test(Some(71));
        control.set_next_generated_bond_sequence_for_test(Some(71));
        let token_before = session.provisional_token_facts_for_test();
        let (fence, start, end) = source(&session);
        let gesture = session
            .begin_direct_bond_gesture_v1(
                fence,
                start.clone(),
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("gesture begins");
        assert_eq!(
            session.admit_direct_bond_candidate_v1(
                &gesture,
                DirectBondEndIntentV1::ExistingAtom { atom: start },
            ),
            Err(super::DirectBondAdmissionRefusalV1::SelfLoop)
        );
        assert_eq!(session.provisional_token_facts_for_test(), token_before);
        let abandoned = session
            .admit_direct_bond_candidate_v1(
                &gesture,
                DirectBondEndIntentV1::ExistingAtom { atom: end.clone() },
            )
            .expect("candidate admits for abandonment");
        assert_eq!(session.provisional_token_facts_for_test(), token_before);
        drop(abandoned);
        assert_eq!(session.provisional_token_facts_for_test(), token_before);
        let admitted = session
            .admit_direct_bond_candidate_v1(
                &gesture,
                DirectBondEndIntentV1::ExistingAtom { atom: end },
            )
            .expect("post-abandonment candidate admits");
        let committed = session
            .commit_direct_bond_admission_v1(&admitted)
            .expect("post-abandonment receipt commits");
        let (control_fence, control_start, control_end) = source(&control);
        let control_gesture = control
            .begin_direct_bond_gesture_v1(
                control_fence,
                control_start,
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("control gesture begins");
        let control_admission = control
            .admit_direct_bond_candidate_v1(
                &control_gesture,
                DirectBondEndIntentV1::ExistingAtom { atom: control_end },
            )
            .expect("control candidate admits");
        let control_committed = control
            .commit_direct_bond_admission_v1(&control_admission)
            .expect("control receipt commits");
        assert_eq!(committed.bond(), control_committed.bond());
        assert_eq!(
            session.provisional_token_facts_for_test(),
            control.provisional_token_facts_for_test()
        );
    }

    #[test]
    fn normal_orders_preserve_semantic_graph_across_endpoint_history_and_reopen() {
        for (order, expected_order, cdml_order) in [
            (DocumentBondOrderV1::Single, BondOrder::Single, "n1"),
            (DocumentBondOrderV1::Double, BondOrder::Double, "n2"),
            (DocumentBondOrderV1::Triple, BondOrder::Triple, "n3"),
        ] {
            for endpoint_is_new in [false, true] {
                let mut session = DocumentSession::load(SOURCE).expect("session loads");
                let (fence, start, end) = source(&session);
                let gesture = session
                    .begin_direct_bond_gesture_v1(
                        fence,
                        start,
                        DocumentBondPresentationV1::Normal(order),
                        "C".to_owned(),
                        DirectBondSnapPolicyV1::free(),
                    )
                    .expect("normal gesture begins");
                let before = session.snapshot().expect("snapshot before admission");
                let end = if endpoint_is_new {
                    DirectBondEndIntentV1::NewAtomAt {
                        raw_point: DirectBondPoint2V1::new(40.0, 0.0).expect("finite point"),
                    }
                } else {
                    DirectBondEndIntentV1::ExistingAtom { atom: end }
                };
                let admission = session
                    .admit_direct_bond_candidate_v1(&gesture, end)
                    .expect("candidate admits without mutation");
                assert_eq!(session.snapshot().expect("admission stays pure"), before);
                assert_eq!(admission.overlay().endpoint_is_new(), endpoint_is_new);
                assert_eq!(
                    admission.overlay().presentation(),
                    DocumentBondPresentationV1::Normal(order)
                );

                let committed = session
                    .commit_direct_bond_admission_v1(&admission)
                    .expect("admission commits atomically");
                let committed_snapshot = committed.result().observation().snapshot();
                assert_eq!(committed_snapshot.revision(), 1);
                assert!(
                    committed_snapshot
                        .cdml()
                        .contains(&format!("type=\"{cdml_order}\""))
                );

                let undone = session.undo(1).expect("one transition undoes");
                assert_eq!(undone.observation().snapshot().revision(), 2);
                let redone = session.redo(2).expect("one transition redoes");
                let redone_snapshot = redone.observation().snapshot();
                assert_eq!(redone_snapshot.revision(), 3);

                let reopened = DocumentSession::load(redone_snapshot.cdml()).expect("CDML reopens");
                let observation = reopened.observe(0).expect("reopened observation");
                let bonds = observation.projection().molecules()[0].bonds();
                assert_eq!(bonds.len(), 1);
                assert_eq!(bonds[0].order(), Some(expected_order));
            }
        }
    }
}
