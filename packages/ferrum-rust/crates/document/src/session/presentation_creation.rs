//! Transactional document-owned creation of durable presentation roots.

use super::{DocumentSession, PersistentId, PreparedSessionTransitionV1, RevisionState};
use crate::DocumentSessionError;
use crate::{
    CurvedTerminalArrowKindV1, GeometricLineWidthV1, PresentationGesturePoint2V1,
    PresentationPathGestureV1, PresentationPathKindV1, PresentationRecordKindV1,
    PresentationRootSelectorV1, Rgb24V1, TypedDocument,
};

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
#[derive(Clone, Debug, PartialEq)]
pub enum PresentationCreateRequestV1 {
    StraightNormalArrow {
        start: PresentationGesturePoint2V1,
        end: PresentationGesturePoint2V1,
        style: crate::ArrowGestureStyleV1,
    },
    StraightEquilibriumArrow {
        start: PresentationGesturePoint2V1,
        end: PresentationGesturePoint2V1,
    },
    StandardPlus {
        anchor: PresentationGesturePoint2V1,
    },
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
            Self::StraightNormalArrow { .. } | Self::StraightEquilibriumArrow { .. } => {
                PresentationRecordKindV1::Arrow
            }
            Self::StandardPlus { .. } => PresentationRecordKindV1::Plus,
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

    fn lower_document(
        &self,
        source: &TypedDocument,
        identifier: &PersistentId,
    ) -> Result<TypedDocument, ()> {
        match self {
            Self::StraightNormalArrow { start, end, style } => source
                .with_insert_straight_normal_arrow(
                    identifier,
                    *start,
                    *end,
                    style.start_head(),
                    style.end_head(),
                )
                .map_err(|_| ()),
            Self::StraightEquilibriumArrow { start, end } => source
                .with_insert_straight_equilibrium_arrow(identifier, *start, *end)
                .map_err(|_| ()),
            Self::StandardPlus { anchor } => source
                .with_insert_standard_plus(identifier, *anchor)
                .map_err(|_| ()),
            _ => self.parse_appended_document(source, identifier),
        }
    }

    fn parse_appended_document(
        &self,
        source: &TypedDocument,
        identifier: &PersistentId,
    ) -> Result<TypedDocument, ()> {
        let source = source.to_xml().map_err(|_| ())?;
        let geometry = match self {
            Self::StraightNormalArrow { .. }
            | Self::StraightEquilibriumArrow { .. }
            | Self::StandardPlus { .. } => {
                unreachable!("direct-root cases lower through typed CDML")
            }
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
            return TypedDocument::parse(&format!(
                "{}{}{}",
                &source[..close],
                geometry,
                &source[close..]
            ))
            .map_err(|_| ());
        }
        let close = source
            .rfind("/>")
            .filter(|index| source[index + 2..].trim().is_empty())
            .ok_or(())?;
        TypedDocument::parse(&format!("{}>{}</cdml>", &source[..close], geometry)).map_err(|_| ())
    }
}

/// Closed durable presentation-vector element vocabulary.
#[derive(Clone, Copy, Debug, PartialEq)]
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

impl DocumentSession {
    pub(crate) fn prepare_create_presentation_transition_v1(
        &mut self,
        expected_revision: u64,
        request: PresentationCreateRequestV1,
        kind: crate::CreatedPresentationRootKindV1,
        authorization_claim: crate::AuthoringCapabilityClaimV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let source_digest = self.current_digest_v1();
        let root_kind = request.root_kind();
        let (identifier, effects) = self
            .reserve_generated_ids_for_transition_v1(|sequences, indexed| {
                sequences.reserve_presentation(indexed)
            })
            .map_err(DocumentSessionError::Operation)?;
        let document = request
            .lower_document(self.current_document_v1(), &identifier)
            .map_err(|_| {
                DocumentSessionError::Operation(
                    crate::SessionOperationError::PresentationCreateRequiresTransitionCore,
                )
            })?;
        let document_object_id = document
            .document_object_id_for_source_id_v1(&identifier)
            .map_err(DocumentSessionError::Projection)?
            .ok_or(DocumentSessionError::Operation(
                crate::SessionOperationError::PresentationCreateRequiresTransitionCore,
            ))?;
        let revision = self
            .next_revision_v1()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate =
            RevisionState::from_document(revision, document).map_err(DocumentSessionError::Load)?;
        let transition = self.prepare_changed_session_transition_with_presentation_outcome_v1(
            super::admitted_transition_v1::ChangedSessionTransitionRequestV1::new(
                expected_revision,
                source_digest,
                candidate,
                effects,
            ),
            PresentationRootSelectorV1::new(document_object_id, root_kind),
            kind,
            authorization_claim,
        )?;
        Ok(transition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: &str = r#"<cdml xmlns="urn:ferrum:cdml" version="26.07"/>"#;

    fn point(x: f64, y: f64) -> PresentationGesturePoint2V1 {
        PresentationGesturePoint2V1::new(x, y).expect("finite test point")
    }

    #[test]
    fn generic_presentation_transition_requires_authorization_and_returns_committed_root() {
        let mut session = DocumentSession::load(EMPTY).expect("session");
        assert!(matches!(
            session.prepare_session_operation_transition_v1(
                crate::SessionOperationTransitionRequestV1::new(
                    0,
                    crate::SessionOperation::V1(
                        crate::SessionOperationV1::CreatePresentationVectorV1(
                            crate::CreatePresentationVectorV1::new(
                                PresentationVectorCreateKindV1::Rectangle,
                                point(0.0, 0.0),
                                point(12.0, 8.0),
                                PresentationAppearanceV1::new(
                                    Rgb24V1::new("#000000").expect("color"),
                                    GeometricLineWidthV1::new(1.0).expect("width"),
                                    None,
                                ),
                            ),
                        )
                    ),
                    crate::TransitionAuthorizationV1::None,
                )
            ),
            Err(DocumentSessionError::TransitionAuthorization(
                crate::TransitionAuthorizationRefusalV1::AuthoringCapabilityRequired
            ))
        ));

        let capability = session.issue_authoring_capability_v1();
        let mut prepared = session
            .prepare_session_operation_transition_v1(
                crate::SessionOperationTransitionRequestV1::new(
                    0,
                    crate::SessionOperation::V1(
                        crate::SessionOperationV1::CreatePresentationVectorV1(
                            crate::CreatePresentationVectorV1::new(
                                PresentationVectorCreateKindV1::Rectangle,
                                point(0.0, 0.0),
                                point(12.0, 8.0),
                                PresentationAppearanceV1::new(
                                    Rgb24V1::new("#000000").expect("color"),
                                    GeometricLineWidthV1::new(1.0).expect("width"),
                                    None,
                                ),
                            ),
                        ),
                    ),
                    crate::TransitionAuthorizationV1::authoring_capability(capability),
                ),
            )
            .expect("generic presentation transition");
        let result = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("generic presentation commit");
        let crate::SessionOperationOutcomeV1::CreatedPresentationRootV1(outcome) = result.outcome()
        else {
            panic!("generic presentation result includes its committed root");
        };
        assert!(
            outcome
                .root()
                .document_object_id()
                .as_str()
                .starts_with("ferrum-document-object-v1/")
        );
    }
}
