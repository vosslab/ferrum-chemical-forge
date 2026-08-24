//! Transactional document-owned creation of durable presentation roots.

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentFenceV1, DocumentSession, PersistentId,
    PreparedSessionTransitionV1, RevisionState, SessionOperationResultV1,
};
use crate::DocumentSessionError;
use crate::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityIssuerV1, AuthoringCapabilityV1,
    CurvedTerminalArrowKindV1, GeometricLineWidthV1, PresentationGesturePoint2V1,
    PresentationPathGestureV1, PresentationPathKindV1, PresentationRecordKindV1, Rgb24V1,
    TypedDocument,
};
use thiserror::Error;

/// Closed appearance facts for one new durable geometric presentation root.
///
/// The document service accepts this value rather than caller-controlled XML
/// fragments.  Its closed RGB and finite-width values preserve the document's
/// ownership of CDML structure. ASVS 1.2.1, 2.2.1, 2.2.2.
#[derive(Clone, Debug, PartialEq)]
pub struct PresentationAppearanceV1 {
    stroke_color: Rgb24V1,
    stroke_width: GeometricLineWidthV1,
    fill_color: Option<Rgb24V1>,
}

impl PresentationAppearanceV1 {
    /// Construct closed appearance facts for a new presentation root.
    #[must_use]
    pub const fn new(
        stroke_color: Rgb24V1,
        stroke_width: GeometricLineWidthV1,
        fill_color: Option<Rgb24V1>,
    ) -> Self {
        Self {
            stroke_color,
            stroke_width,
            fill_color,
        }
    }

    #[must_use]
    pub const fn stroke_color(&self) -> &Rgb24V1 {
        &self.stroke_color
    }

    #[must_use]
    pub const fn stroke_width(&self) -> GeometricLineWidthV1 {
        self.stroke_width
    }

    #[must_use]
    pub const fn fill_color(&self) -> Option<&Rgb24V1> {
        self.fill_color.as_ref()
    }
}

/// Closed document-native vocabulary for one new durable presentation root.
///
/// The caller supplies validated geometry and closed appearance facts, never an identifier.
#[derive(Clone, Debug)]
pub enum PresentationCreateRequestV1 {
    Vector {
        kind: PresentationVectorCreateKindV1,
        start: PresentationGesturePoint2V1,
        end: PresentationGesturePoint2V1,
        appearance: PresentationAppearanceV1,
    },
    CurvedTerminalArrow {
        kind: CurvedTerminalArrowKindV1,
        start: PresentationGesturePoint2V1,
        control: PresentationGesturePoint2V1,
        end: PresentationGesturePoint2V1,
    },
    CurvedEquilibriumArrow {
        start: PresentationGesturePoint2V1,
        control: PresentationGesturePoint2V1,
        end: PresentationGesturePoint2V1,
    },
    Path {
        path: PresentationPathGestureV1,
        appearance: PresentationAppearanceV1,
    },
}

impl PresentationCreateRequestV1 {
    #[must_use]
    const fn root_kind(&self) -> PresentationRecordKindV1 {
        match self {
            Self::Vector { kind, .. } => kind.root_kind(),
            Self::CurvedTerminalArrow { .. } | Self::CurvedEquilibriumArrow { .. } => {
                PresentationRecordKindV1::Arrow
            }
            Self::Path { path, .. } => match path.kind() {
                PresentationPathKindV1::Polyline => PresentationRecordKindV1::Polyline,
                PresentationPathKindV1::Polygon => PresentationRecordKindV1::Polygon,
            },
        }
    }

    fn append_to(&self, source: &str, identifier: &PersistentId) -> Result<String, ()> {
        let geometry = match self {
            Self::Vector {
                kind,
                start,
                end,
                appearance,
            } => match kind {
                PresentationVectorCreateKindV1::Line => format!(
                    "<polyline id=\"{}\" spline=\"0\" line_color=\"{}\" width=\"{}\"><point x=\"{}\" y=\"{}\" z=\"0\"/><point x=\"{}\" y=\"{}\" z=\"0\"/></polyline>",
                    identifier.as_str(),
                    appearance.stroke_color().as_str(),
                    appearance.stroke_width().value(),
                    start.x(),
                    start.y(),
                    end.x(),
                    end.y()
                ),
                kind => format!(
                    "<{} id=\"{}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" line_color=\"{}\" width=\"{}\" area_color=\"{}\"/>",
                    kind.element_name(),
                    identifier.as_str(),
                    start.x(),
                    start.y(),
                    end.x(),
                    end.y(),
                    appearance.stroke_color().as_str(),
                    appearance.stroke_width().value(),
                    appearance.fill_color().map_or("none", Rgb24V1::as_str)
                ),
            },
            Self::CurvedTerminalArrow {
                kind,
                start,
                control,
                end,
            } => format!(
                "<arrow id=\"{}\" type=\"{}\" width=\"1.0\" color=\"#000000\"><point x=\"{}\" y=\"{}\" z=\"0\"/><point x=\"{}\" y=\"{}\" z=\"0\"/><point x=\"{}\" y=\"{}\" z=\"0\"/></arrow>",
                identifier.as_str(),
                kind.cdml_type(),
                start.x(),
                start.y(),
                control.x(),
                control.y(),
                end.x(),
                end.y()
            ),
            Self::CurvedEquilibriumArrow {
                start,
                control,
                end,
            } => format!(
                "<arrow id=\"{}\" type=\"curved-equilibrium\" width=\"1.0\" color=\"#000000\"><point x=\"{}\" y=\"{}\" z=\"0\"/><point x=\"{}\" y=\"{}\" z=\"0\"/><point x=\"{}\" y=\"{}\" z=\"0\"/></arrow>",
                identifier.as_str(),
                start.x(),
                start.y(),
                control.x(),
                control.y(),
                end.x(),
                end.y()
            ),
            Self::Path { path, appearance } => {
                let points = path
                    .points()
                    .iter()
                    .map(|point| {
                        format!("<point x=\"{}\" y=\"{}\" z=\"0\"/>", point.x(), point.y())
                    })
                    .collect::<String>();
                match (path.kind(), appearance.fill_color()) {
                    (PresentationPathKindV1::Polyline, _) => format!(
                        "<polyline id=\"{}\" spline=\"0\" line_color=\"{}\" width=\"{}\">{points}</polyline>",
                        identifier.as_str(),
                        appearance.stroke_color().as_str(),
                        appearance.stroke_width().value()
                    ),
                    (PresentationPathKindV1::Polygon, Some(area_color)) => format!(
                        "<polygon id=\"{}\" line_color=\"{}\" width=\"{}\" area_color=\"{}\">{points}</polygon>",
                        identifier.as_str(),
                        appearance.stroke_color().as_str(),
                        appearance.stroke_width().value(),
                        area_color.as_str()
                    ),
                    (PresentationPathKindV1::Polygon, None) => format!(
                        "<polygon id=\"{}\" line_color=\"{}\" width=\"{}\">{points}</polygon>",
                        identifier.as_str(),
                        appearance.stroke_color().as_str(),
                        appearance.stroke_width().value()
                    ),
                }
            }
        };
        if let Some(close) = source.rfind("</cdml") {
            return Ok(format!(
                "{}{}{}",
                &source[..close],
                geometry,
                &source[close..]
            ));
        }
        let close = source
            .rfind("/>")
            .filter(|index| source[index + 2..].trim().is_empty())
            .ok_or(())?;
        Ok(format!("{}>{}</cdml>", &source[..close], geometry))
    }
}

/// Closed durable presentation-vector element vocabulary.
#[derive(Clone, Copy, Debug)]
pub enum PresentationVectorCreateKindV1 {
    Line,
    Rectangle,
    Square,
    Oval,
    Circle,
}

impl PresentationVectorCreateKindV1 {
    const fn root_kind(self) -> PresentationRecordKindV1 {
        match self {
            Self::Line => PresentationRecordKindV1::Polyline,
            Self::Rectangle => PresentationRecordKindV1::Rectangle,
            Self::Square => PresentationRecordKindV1::Square,
            Self::Oval => PresentationRecordKindV1::Oval,
            Self::Circle => PresentationRecordKindV1::Circle,
        }
    }

    const fn element_name(self) -> &'static str {
        match self {
            Self::Line => "polyline",
            Self::Rectangle => "rect",
            Self::Square => "square",
            Self::Oval => "oval",
            Self::Circle => "circle",
        }
    }
}

/// Opaque one-use, session-bound presentation-root candidate.
///
/// The caller retains this document-session authority until it commits or is dropped.
pub struct PendingCreatePresentationV1 {
    session_issuer: AuthoringCapabilityIssuerV1,
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    transition: PreparedSessionTransitionV1,
    identifier: PersistentId,
    root_kind: PresentationRecordKindV1,
}

impl std::fmt::Debug for PendingCreatePresentationV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreatePresentationV1")
            .field("revision", &self.fence.revision())
            .field("is_resolved", &self.transition.is_consumed_v1())
            .finish()
    }
}

impl PendingCreatePresentationV1 {
    /// Return the canonical generated presentation identity while this receipt is live.
    #[must_use]
    pub const fn identifier(&self) -> &PersistentId {
        &self.identifier
    }

    /// Return the exact direct-root class selected by the closed request vocabulary.
    #[must_use]
    pub const fn root_kind(&self) -> PresentationRecordKindV1 {
        self.root_kind
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PresentationCreateErrorV1 {
    #[error("presentation creation belongs to another document session")]
    ForeignSession,
    #[error("presentation creation snapshot is stale")]
    StaleSnapshot,
    #[error("presentation creation receipt was already consumed")]
    Replayed,
    #[error("presentation creation session transaction failed")]
    SessionConflict,
    #[error("presentation candidate was refused by renderer admission")]
    RendererAdmission,
}

impl DocumentSession {
    /// Reserve one canonical presentation ID and construct its candidate without mutation.
    pub fn prepare_create_presentation_v1(
        &mut self,
        capability: &AuthoringCapabilityV1,
        fence: DocumentFenceV1,
        request: PresentationCreateRequestV1,
    ) -> Result<PendingCreatePresentationV1, PresentationCreateErrorV1> {
        if !capability.belongs_to(&self.authoring_capability_issuer) {
            return Err(PresentationCreateErrorV1::ForeignSession);
        }
        require_fence(self, fence)?;
        let (identifier, effects) = self
            .reserve_generated_ids_for_transition_v1(|sequences, indexed| {
                sequences.reserve_presentation(indexed)
            })
            .map_err(|_| PresentationCreateErrorV1::SessionConflict)?;
        let source = self
            .snapshot()
            .map_err(|_| PresentationCreateErrorV1::SessionConflict)?
            .cdml()
            .to_owned();
        let candidate_cdml = request
            .append_to(&source, &identifier)
            .map_err(|_| PresentationCreateErrorV1::SessionConflict)?;
        let document = TypedDocument::parse(&candidate_cdml)
            .map_err(|_| PresentationCreateErrorV1::SessionConflict)?;
        let revision = self
            .next_revision_v1()
            .ok_or(PresentationCreateErrorV1::SessionConflict)?;
        let candidate = RevisionState::from_document(revision, document)
            .map_err(|_| PresentationCreateErrorV1::SessionConflict)?;
        let transition = self
            .prepare_changed_session_transition_v1(
                fence.revision(),
                fence.digest(),
                candidate,
                effects,
            )
            .map_err(map_prepare_error)?;
        Ok(PendingCreatePresentationV1 {
            session_issuer: self.authoring_capability_issuer.clone(),
            capability: capability.clone(),
            fence,
            transition,
            identifier,
            root_kind: request.root_kind(),
        })
    }

    /// Commit one presentation candidate atomically, installing its ID sequence only on success.
    pub fn commit_create_presentation_v1(
        &mut self,
        pending: &mut PendingCreatePresentationV1,
    ) -> Result<SessionOperationResultV1, PresentationCreateErrorV1> {
        if pending.transition.is_consumed_v1() {
            return Err(PresentationCreateErrorV1::Replayed);
        }
        if !pending
            .session_issuer
            .same_issuer(&self.authoring_capability_issuer)
            || !pending
                .capability
                .belongs_to(&self.authoring_capability_issuer)
        {
            return Err(PresentationCreateErrorV1::ForeignSession);
        }
        let claim = pending
            .capability
            .claim_for_commit(&self.authoring_capability_issuer)
            .map_err(|error| match error {
                AuthoringCapabilityAccessErrorV1::ForeignSession => {
                    PresentationCreateErrorV1::ForeignSession
                }
                AuthoringCapabilityAccessErrorV1::Replayed => PresentationCreateErrorV1::Replayed,
            })?;
        let operation = self
            .commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(map_commit_error)?;
        claim.consume();
        Ok(operation)
    }
}

fn map_prepare_error(error: DocumentSessionError) -> PresentationCreateErrorV1 {
    match error {
        DocumentSessionError::RendererAdmission => PresentationCreateErrorV1::RendererAdmission,
        _ => PresentationCreateErrorV1::SessionConflict,
    }
}

fn map_commit_error(error: AdmittedSessionTransitionRefusalV1) -> PresentationCreateErrorV1 {
    match error {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            PresentationCreateErrorV1::ForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Replayed => PresentationCreateErrorV1::Replayed,
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            PresentationCreateErrorV1::StaleSnapshot
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            PresentationCreateErrorV1::RendererAdmission
        }
        AdmittedSessionTransitionRefusalV1::ProvisionalCapability
        | AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
            PresentationCreateErrorV1::SessionConflict
        }
    }
}

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), PresentationCreateErrorV1> {
    if session.current_revision_v1() != fence.revision()
        || session.current_digest_v1() != fence.digest()
    {
        return Err(PresentationCreateErrorV1::StaleSnapshot);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: &str = r#"<cdml xmlns="urn:ferrum:cdml" version="26.07"/>"#;

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("test snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    fn point(x: f64, y: f64) -> PresentationGesturePoint2V1 {
        PresentationGesturePoint2V1::new(x, y).expect("finite test point")
    }

    fn request() -> PresentationCreateRequestV1 {
        PresentationCreateRequestV1::Vector {
            kind: PresentationVectorCreateKindV1::Rectangle,
            start: point(0.0, 0.0),
            end: point(12.0, 8.0),
            appearance: PresentationAppearanceV1::new(
                Rgb24V1::new("#000000").expect("closed test colour"),
                GeometricLineWidthV1::new(1.0).expect("closed test width"),
                None,
            ),
        }
    }

    #[test]
    fn reserves_canonical_presentation_ids_without_mutating_until_commit() {
        let mut session = DocumentSession::load(
            r#"<cdml xmlns="urn:ferrum:cdml" version="26.07"><opaque id="ferrum-presentation-v1-0"/></cdml>"#,
        )
        .expect("session");
        let before = session.snapshot().expect("before");
        let capability = session.authoring_capability_issuer_v1().issue();
        let mut pending = session
            .prepare_create_presentation_v1(&capability, fence(&session), request())
            .expect("reservation");

        assert_eq!(pending.identifier().as_str(), "ferrum-presentation-v1-1");
        assert_eq!(session.snapshot().expect("unchanged"), before);

        session
            .commit_create_presentation_v1(&mut pending)
            .expect("commit");
        assert!(
            session
                .snapshot()
                .expect("committed")
                .cdml()
                .contains("ferrum-presentation-v1-1")
        );
        assert!(matches!(
            session.commit_create_presentation_v1(&mut pending),
            Err(PresentationCreateErrorV1::Replayed)
        ));
    }

    #[test]
    fn renderer_admission_observation_is_the_exact_committed_candidate() {
        let mut session = DocumentSession::load(EMPTY).expect("session");
        let capability = session.authoring_capability_issuer_v1().issue();
        let mut pending = session
            .prepare_create_presentation_v1(&capability, fence(&session), request())
            .expect("reservation");
        let expected = pending
            .transition
            .metadata_v1()
            .expect("live transition metadata")
            .observation()
            .clone();

        let committed = session
            .commit_create_presentation_v1(&mut pending)
            .expect("commit");

        assert_eq!(committed.observation(), &expected);
    }

    #[test]
    fn foreign_commit_preserves_the_owner_pending_reservation() {
        let mut owner = DocumentSession::load(EMPTY).expect("owner");
        let capability = owner.authoring_capability_issuer_v1().issue();
        let mut pending = owner
            .prepare_create_presentation_v1(&capability, fence(&owner), request())
            .expect("reservation");
        let mut foreign = DocumentSession::load(EMPTY).expect("foreign");

        assert!(matches!(
            foreign.commit_create_presentation_v1(&mut pending),
            Err(PresentationCreateErrorV1::ForeignSession)
        ));
        owner
            .commit_create_presentation_v1(&mut pending)
            .expect("owner retains receipt after foreign refusal");
    }

    #[test]
    fn rejected_appearance_input_cannot_inject_roots_or_presentation_identities() {
        let mut session = DocumentSession::load(EMPTY).expect("session");
        let capability = session.authoring_capability_issuer_v1().issue();
        let before = session.snapshot().expect("before");

        assert!(Rgb24V1::new(r#"#000000\"/><rect id=\"attacker\"/>"#).is_none());
        assert!(Rgb24V1::new(r#"#000000\"><rect id=\"attacker\"/>"#).is_none());
        assert_eq!(
            session.snapshot().expect("rejected input changes nothing"),
            before
        );

        let mut pending = session
            .prepare_create_presentation_v1(&capability, fence(&session), request())
            .expect("closed appearance request");
        assert_eq!(pending.identifier().as_str(), "ferrum-presentation-v1-0");
        session
            .commit_create_presentation_v1(&mut pending)
            .expect("closed request commits");
        let cdml = session.snapshot().expect("committed").cdml().to_owned();
        assert_eq!(cdml.matches("ferrum-presentation-v1-").count(), 1);
        assert!(!cdml.contains("attacker"));
    }

    #[test]
    fn abandoned_presentation_reservation_reuses_its_id_until_a_commit_advances_it() {
        let mut session = DocumentSession::load(EMPTY).expect("session");
        let capability = session.authoring_capability_issuer_v1().issue();
        let abandoned = session
            .prepare_create_presentation_v1(&capability, fence(&session), request())
            .expect("reservation");
        assert_eq!(abandoned.identifier().as_str(), "ferrum-presentation-v1-0");
        drop(abandoned);

        let mut committed = session
            .prepare_create_presentation_v1(&capability, fence(&session), request())
            .expect("replacement reservation");
        assert_eq!(committed.identifier().as_str(), "ferrum-presentation-v1-0");
        session
            .commit_create_presentation_v1(&mut committed)
            .expect("commit installs the reserved sequence");

        let next = session
            .prepare_create_presentation_v1(&capability, fence(&session), request())
            .expect("next reservation");
        assert_eq!(next.identifier().as_str(), "ferrum-presentation-v1-1");
    }

    #[test]
    fn stale_presentation_reservation_does_not_advance_the_committed_id_sequence() {
        let mut session = DocumentSession::load(EMPTY).expect("session");
        let first_capability = session.authoring_capability_issuer_v1().issue();
        let stale_capability = session.authoring_capability_issuer_v1().issue();
        let mut first = session
            .prepare_create_presentation_v1(&first_capability, fence(&session), request())
            .expect("first reservation");
        let mut stale = session
            .prepare_create_presentation_v1(&stale_capability, fence(&session), request())
            .expect("concurrent tentative reservation");
        assert_eq!(first.identifier().as_str(), "ferrum-presentation-v1-0");
        assert_eq!(stale.identifier().as_str(), "ferrum-presentation-v1-0");

        session
            .commit_create_presentation_v1(&mut first)
            .expect("first commit installs its sequence");
        assert_eq!(
            session.commit_create_presentation_v1(&mut stale),
            Err(PresentationCreateErrorV1::StaleSnapshot)
        );
        drop(stale);

        let next_capability = session.authoring_capability_issuer_v1().issue();
        let next = session
            .prepare_create_presentation_v1(&next_capability, fence(&session), request())
            .expect("next reservation after stale refusal");
        assert_eq!(next.identifier().as_str(), "ferrum-presentation-v1-1");
    }
}
