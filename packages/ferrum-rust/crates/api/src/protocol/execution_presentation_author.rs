//! Stateless execution of the closed presentation authoring protocol.

use crate::{
    add_api_presentation_path_gesture_point_v1, begin_api_presentation_path_gesture_v1,
    begin_api_presentation_vector_gesture_v1, preview_api_presentation_vector_gesture_v1,
    preview_incremental_api_presentation_path_gesture_v1,
    resolve_api_presentation_vector_gesture_v1,
    resolve_incremental_api_presentation_path_gesture_v1,
};
use ferrum_document::{
    AdmittedSessionTransitionRefusalV1, CreateDirectBondV1, DirectBondAdmissionRefusalV1,
    DirectBondEndpointIntent, DirectBondPoint2V1, DirectBondSnapPolicyV1, DocumentBondOrderV1,
    DocumentBondPresentationV1, DocumentFenceV1, DocumentSession, DocumentSessionError,
    PresentationGesturePoint2V1, PresentationPathKindV1, SessionOperation,
    SessionOperationOutcomeV1, SessionOperationResultV1, SessionOperationTransitionRequestV1,
    SessionOperationV1, TransitionAuthorizationRefusalV1, TransitionAuthorizationV1,
};
use ferrum_document_render::{
    CurvedElectronArrowGestureErrorV1, CurvedEquilibriumArrowGestureErrorV1,
    PresentationPathRenderErrorV1, PresentationVectorGestureErrorV1,
    begin_curved_electron_arrow_gesture_v1, begin_curved_equilibrium_arrow_gesture_v1,
    begin_curved_normal_reaction_arrow_gesture_v1, begin_curved_retro_arrow_gesture_v1,
    preview_curved_electron_arrow_gesture_v1, preview_curved_equilibrium_arrow_gesture_v1,
    preview_curved_normal_reaction_arrow_gesture_v1, preview_curved_retro_arrow_gesture_v1,
    resolve_curved_electron_arrow_gesture_v1, resolve_curved_equilibrium_arrow_gesture_v1,
    resolve_curved_normal_reaction_arrow_gesture_v1, resolve_curved_retro_arrow_gesture_v1,
};

use super::*;

pub(super) fn execute_presentation_author(
    request: PresentationAuthorRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let kind = authoring_kind(&request.authoring);
    if request.expected_revision != 0 {
        return Err(refusal(
            kind,
            ProtocolPresentationAuthorCategoryV1::StaleSnapshot,
            ProtocolPresentationAuthorRecoveryV1::RefreshAndRestart,
            "stateless authoring accepts revision zero only",
        ));
    }
    let mut session = admit_document(&request.document)?;
    let digest = parse_digest_hex(&request.expected_digest_hex)?;
    let fence = DocumentFenceV1::new(0, digest);
    match request.authoring {
        PresentationAuthoringRequestV1::Vector {
            vector_kind,
            start,
            end,
            appearance_policy,
        } => execute_vector(
            &mut session,
            fence,
            vector_kind,
            start,
            end,
            appearance_policy,
        ),
        PresentationAuthoringRequestV1::CurvedTerminalArrow {
            terminal_kind,
            start,
            control,
            end,
        } => execute_terminal_arrow(&mut session, fence, terminal_kind, start, control, end),
        PresentationAuthoringRequestV1::CurvedEquilibriumArrow {
            start,
            control,
            end,
        } => execute_equilibrium_arrow(&mut session, fence, start, control, end),
        PresentationAuthoringRequestV1::Path { path_kind, points } => {
            execute_path(&mut session, fence, path_kind, points)
        }
        PresentationAuthoringRequestV1::DirectBond {
            start,
            end,
            presentation,
            new_atom_element,
            snap,
        } => execute_direct_bond(
            &mut session,
            fence,
            start,
            end,
            presentation,
            new_atom_element,
            snap,
        ),
    }
}

fn execute_vector(
    session: &mut DocumentSession,
    fence: DocumentFenceV1,
    vector_kind: ProtocolPresentationVectorKindV1,
    start: PresentationAuthorPointV1,
    end: PresentationAuthorPointV1,
    appearance_policy: ProtocolPresentationVectorAppearancePolicyV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    match appearance_policy {
        ProtocolPresentationVectorAppearancePolicyV1::EffectiveDrawingStandard => {}
    }
    let start = point(start, PresentationAuthoringKindV1::Vector)?;
    let end = point(end, PresentationAuthoringKindV1::Vector)?;
    let kind = match vector_kind {
        ProtocolPresentationVectorKindV1::Line => PresentationVectorKindV1::Line,
        ProtocolPresentationVectorKindV1::Rectangle => PresentationVectorKindV1::Rectangle,
        ProtocolPresentationVectorKindV1::Square => PresentationVectorKindV1::Square,
        ProtocolPresentationVectorKindV1::Oval => PresentationVectorKindV1::Oval,
        ProtocolPresentationVectorKindV1::Circle => PresentationVectorKindV1::Circle,
    };
    let gesture = begin_api_presentation_vector_gesture_v1(session, fence, kind, start)
        .map_err(vector_error)?;
    let preview =
        preview_api_presentation_vector_gesture_v1(session, &gesture, end).map_err(vector_error)?;
    let request = resolve_api_presentation_vector_gesture_v1(session, gesture, preview)
        .map_err(vector_error)?;
    finish_created_presentation_root(PresentationAuthoringKindV1::Vector, session, request)
}

fn execute_terminal_arrow(
    session: &mut DocumentSession,
    fence: DocumentFenceV1,
    terminal_kind: ProtocolCurvedTerminalArrowKindV1,
    start: PresentationAuthorPointV1,
    control: PresentationAuthorPointV1,
    end: PresentationAuthorPointV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let kind = PresentationAuthoringKindV1::CurvedTerminalArrow;
    let start = point(start, kind)?;
    let control = point(control, kind)?;
    let end = point(end, kind)?;
    match terminal_kind {
        ProtocolCurvedTerminalArrowKindV1::Electron => {
            let gesture = begin_curved_electron_arrow_gesture_v1(session, fence, start, control)
                .map_err(terminal_error)?;
            let preview = preview_curved_electron_arrow_gesture_v1(session, &gesture, end)
                .map_err(terminal_error)?;
            let request = resolve_curved_electron_arrow_gesture_v1(session, gesture, preview)
                .map_err(terminal_error)?;
            finish_created_presentation_root(kind, session, request)
        }
        ProtocolCurvedTerminalArrowKindV1::Retro => {
            let gesture = begin_curved_retro_arrow_gesture_v1(session, fence, start, control)
                .map_err(terminal_error)?;
            let preview = preview_curved_retro_arrow_gesture_v1(session, &gesture, end)
                .map_err(terminal_error)?;
            let request = resolve_curved_retro_arrow_gesture_v1(session, gesture, preview)
                .map_err(terminal_error)?;
            finish_created_presentation_root(kind, session, request)
        }
        ProtocolCurvedTerminalArrowKindV1::Normal => {
            let gesture =
                begin_curved_normal_reaction_arrow_gesture_v1(session, fence, start, control)
                    .map_err(terminal_error)?;
            let preview = preview_curved_normal_reaction_arrow_gesture_v1(session, &gesture, end)
                .map_err(terminal_error)?;
            let request =
                resolve_curved_normal_reaction_arrow_gesture_v1(session, gesture, preview)
                    .map_err(terminal_error)?;
            finish_created_presentation_root(kind, session, request)
        }
    }
}

fn execute_equilibrium_arrow(
    session: &mut DocumentSession,
    fence: DocumentFenceV1,
    start: PresentationAuthorPointV1,
    control: PresentationAuthorPointV1,
    end: PresentationAuthorPointV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let kind = PresentationAuthoringKindV1::CurvedEquilibriumArrow;
    let gesture = begin_curved_equilibrium_arrow_gesture_v1(
        session,
        fence,
        point(start, kind)?,
        point(control, kind)?,
    )
    .map_err(equilibrium_error)?;
    let preview = preview_curved_equilibrium_arrow_gesture_v1(session, &gesture, point(end, kind)?)
        .map_err(equilibrium_error)?;
    let request = resolve_curved_equilibrium_arrow_gesture_v1(session, gesture, preview)
        .map_err(equilibrium_error)?;
    finish_created_presentation_root(kind, session, request)
}

fn execute_path(
    session: &mut DocumentSession,
    fence: DocumentFenceV1,
    path_kind: ProtocolPresentationPathKindV1,
    points: Vec<PresentationAuthorPointV1>,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let kind = PresentationAuthoringKindV1::Path;
    let path_kind = match path_kind {
        ProtocolPresentationPathKindV1::Polyline => PresentationPathKindV1::Polyline,
        ProtocolPresentationPathKindV1::Polygon => PresentationPathKindV1::Polygon,
    };
    let mut gesture =
        begin_api_presentation_path_gesture_v1(session, fence, path_kind).map_err(path_error)?;
    for value in points {
        add_api_presentation_path_gesture_point_v1(session, &mut gesture, point(value, kind)?)
            .map_err(path_error)?;
    }
    let overlay = preview_incremental_api_presentation_path_gesture_v1(session, &gesture, None)
        .map_err(path_error)?;
    let request = resolve_incremental_api_presentation_path_gesture_v1(session, gesture, overlay)
        .map_err(path_error)?;
    finish_created_presentation_root(kind, session, request)
}

#[allow(clippy::too_many_arguments)]
fn execute_direct_bond(
    session: &mut DocumentSession,
    fence: DocumentFenceV1,
    start: ProtocolDirectBondEndpointV1,
    end: ProtocolDirectBondEndpointV1,
    presentation: ProtocolDirectBondPresentationV1,
    new_atom_element: String,
    snap: ProtocolDirectBondSnapV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let kind = PresentationAuthoringKindV1::DirectBond;
    let start = direct_bond_endpoint(session, start, kind)?;
    let end = direct_bond_endpoint(session, end, kind)?;
    let presentation = direct_bond_presentation(presentation);
    let snap = DirectBondSnapPolicyV1::new(
        snap.hex_grid,
        snap.angle_increment_degrees,
        snap.fixed_length_pt,
    )
    .map_err(|_| {
        refusal(
            kind,
            ProtocolPresentationAuthorCategoryV1::InvalidEndpoint,
            ProtocolPresentationAuthorRecoveryV1::AdjustEndpoint,
            "direct-bond snap policy is invalid",
        )
    })?;
    let operation =
        CreateDirectBondV1::new(fence, start, end, presentation, new_atom_element, snap)
            .map_err(direct_bond_admission_error)?;
    let capability = session.issue_authoring_capability_v1();
    let mut prepared = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            fence.revision(),
            SessionOperation::V1(SessionOperationV1::CreateDirectBondV1(operation)),
            TransitionAuthorizationV1::authoring_capability(capability),
        ))
        .map_err(direct_bond_prepare_error)?;
    let result = session
        .commit_session_operation_transition_v1(&mut prepared)
        .map_err(direct_bond_commit_error)?;
    let SessionOperationOutcomeV1::DirectBondV1(committed) = result.outcome() else {
        return Err(refusal(
            kind,
            ProtocolPresentationAuthorCategoryV1::SessionConflict,
            ProtocolPresentationAuthorRecoveryV1::RefreshAndRestart,
            "generic direct-bond transition returned an unexpected outcome",
        ));
    };
    let direct_bond = PresentationAuthorDirectBondOutcomeV1 {
        bond_document_object_id: committed.bond_document_object_id().as_str().to_owned(),
        end_atom_document_object_id: committed.end_atom_document_object_id().as_str().to_owned(),
        second_created_atom_document_object_id: committed
            .second_created_atom_document_object_id()
            .map(|identifier| identifier.as_str().to_owned()),
        created_new_atom: committed.created_new_atom(),
        created_new_molecule: committed.created_new_molecule(),
    };
    finish(
        kind,
        committed.bond_document_object_id().as_str().to_owned(),
        "bond".to_owned(),
        &result,
        Some(direct_bond),
    )
}

fn finish(
    authoring_kind: PresentationAuthoringKindV1,
    identifier: String,
    root_kind: String,
    result: &SessionOperationResultV1,
    direct_bond: Option<PresentationAuthorDirectBondOutcomeV1>,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let snapshot = result.observation().snapshot();
    Ok(OperationProtocolOutcomeV1::PresentationAuthor {
        authoring_kind,
        document: snapshot.cdml().to_owned(),
        identifier,
        root_kind,
        committed_revision: snapshot.revision(),
        document_fence: DocumentRequestFenceV1 {
            expected_revision: 0,
            expected_digest_hex: hex_digest(snapshot.digest()),
        },
        direct_bond,
    })
}

fn finish_created_presentation_root(
    kind: PresentationAuthoringKindV1,
    session: &mut DocumentSession,
    request: SessionOperationTransitionRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let mut prepared = session
        .prepare_session_operation_transition_v1(request)
        .map_err(|error| presentation_prepare_error(kind, error))?;
    let result = session
        .commit_session_operation_transition_v1(&mut prepared)
        .map_err(|error| presentation_commit_error(kind, error))?;
    let SessionOperationOutcomeV1::CreatedPresentationRootV1(outcome) = result.outcome() else {
        return Err(refusal(
            kind,
            ProtocolPresentationAuthorCategoryV1::SessionConflict,
            ProtocolPresentationAuthorRecoveryV1::RefreshAndRestart,
            "generic presentation transition returned an unexpected outcome",
        ));
    };
    finish(
        kind,
        outcome.root().document_object_id().as_str().to_owned(),
        format!("{:?}", outcome.kind()),
        &result,
        None,
    )
}

fn point(
    value: PresentationAuthorPointV1,
    kind: PresentationAuthoringKindV1,
) -> Result<PresentationGesturePoint2V1, ExecutionFailureV1> {
    PresentationGesturePoint2V1::new(value.x, value.y).map_err(|_| {
        refusal(
            kind,
            ProtocolPresentationAuthorCategoryV1::InvalidPoint,
            ProtocolPresentationAuthorRecoveryV1::ChangeGeometry,
            "authoring point must be finite",
        )
    })
}

fn direct_bond_endpoint(
    session: &DocumentSession,
    value: ProtocolDirectBondEndpointV1,
    kind: PresentationAuthoringKindV1,
) -> Result<DirectBondEndpointIntent, ExecutionFailureV1> {
    match value {
        ProtocolDirectBondEndpointV1::ExistingAtom { atom_id } => {
            let observation = session.observe(0).map_err(|_| {
                refusal(
                    kind,
                    ProtocolPresentationAuthorCategoryV1::SessionConflict,
                    ProtocolPresentationAuthorRecoveryV1::RefreshAndRestart,
                    "direct-bond source session is unavailable",
                )
            })?;
            let atom = observation
                .projection()
                .molecules()
                .iter()
                .flat_map(|molecule| molecule.atoms())
                .find(|atom| atom.source_id() == Some(atom_id.as_str()))
                .map(|atom| atom.document_object_id().clone());
            atom.map(|atom| DirectBondEndpointIntent::ExistingAtom { atom })
                .ok_or_else(|| {
                    refusal(
                        kind,
                        ProtocolPresentationAuthorCategoryV1::InvalidEndpoint,
                        ProtocolPresentationAuthorRecoveryV1::AdjustEndpoint,
                        "direct-bond atom endpoint is not available",
                    )
                })
        }
        ProtocolDirectBondEndpointV1::NewAtom { point } => {
            DirectBondPoint2V1::new(point.x, point.y)
                .map(|raw_point| DirectBondEndpointIntent::NewAtomAt { raw_point })
                .map_err(|_| {
                    refusal(
                        kind,
                        ProtocolPresentationAuthorCategoryV1::InvalidEndpoint,
                        ProtocolPresentationAuthorRecoveryV1::AdjustEndpoint,
                        "direct-bond point endpoint must be finite",
                    )
                })
        }
    }
}

fn direct_bond_presentation(value: ProtocolDirectBondPresentationV1) -> DocumentBondPresentationV1 {
    match value {
        ProtocolDirectBondPresentationV1::Normal { order } => {
            DocumentBondPresentationV1::Normal(match order {
                ProtocolDirectBondOrderV1::Single => DocumentBondOrderV1::Single,
                ProtocolDirectBondOrderV1::Double => DocumentBondOrderV1::Double,
                ProtocolDirectBondOrderV1::Triple => DocumentBondOrderV1::Triple,
            })
        }
        ProtocolDirectBondPresentationV1::SolidWedge => DocumentBondPresentationV1::SolidWedge,
        ProtocolDirectBondPresentationV1::HashedWedge => DocumentBondPresentationV1::HashedWedge,
    }
}

fn authoring_kind(value: &PresentationAuthoringRequestV1) -> PresentationAuthoringKindV1 {
    match value {
        PresentationAuthoringRequestV1::Vector { .. } => PresentationAuthoringKindV1::Vector,
        PresentationAuthoringRequestV1::CurvedTerminalArrow { .. } => {
            PresentationAuthoringKindV1::CurvedTerminalArrow
        }
        PresentationAuthoringRequestV1::CurvedEquilibriumArrow { .. } => {
            PresentationAuthoringKindV1::CurvedEquilibriumArrow
        }
        PresentationAuthoringRequestV1::Path { .. } => PresentationAuthoringKindV1::Path,
        PresentationAuthoringRequestV1::DirectBond { .. } => {
            PresentationAuthoringKindV1::DirectBond
        }
    }
}

fn refusal(
    kind: PresentationAuthoringKindV1,
    category: ProtocolPresentationAuthorCategoryV1,
    recovery: ProtocolPresentationAuthorRecoveryV1,
    message: impl Into<String>,
) -> ExecutionFailureV1 {
    ExecutionFailureV1::presentation_author_refusal(kind, category, recovery, message.into())
}

fn vector_error(error: PresentationVectorGestureErrorV1) -> ExecutionFailureV1 {
    use PresentationVectorGestureCategoryV1 as Category;
    let category = match error.category() {
        Category::StaleSnapshot => ProtocolPresentationAuthorCategoryV1::StaleSnapshot,
        Category::ForeignSession => ProtocolPresentationAuthorCategoryV1::ForeignSession,
        Category::MismatchedPreview | Category::Consumed => {
            ProtocolPresentationAuthorCategoryV1::Consumed
        }
        Category::InvalidPoint => ProtocolPresentationAuthorCategoryV1::InvalidPoint,
        Category::DegenerateGeometry => ProtocolPresentationAuthorCategoryV1::DegenerateGeometry,
        Category::UnsupportedKind | Category::UnrenderableStandard => {
            ProtocolPresentationAuthorCategoryV1::UnsupportedPresentation
        }
        Category::RenderPreparation => ProtocolPresentationAuthorCategoryV1::RenderPreparation,
        Category::SessionConflict => ProtocolPresentationAuthorCategoryV1::SessionConflict,
        Category::ResourceExhausted => ProtocolPresentationAuthorCategoryV1::Capacity,
        _ => ProtocolPresentationAuthorCategoryV1::SessionConflict,
    };
    refusal_for_category(
        PresentationAuthoringKindV1::Vector,
        category,
        error.to_string(),
    )
}

fn terminal_error(error: CurvedElectronArrowGestureErrorV1) -> ExecutionFailureV1 {
    let category = match error.category() {
        ferrum_document_render::CurvedElectronArrowGestureCategoryV1::StaleSnapshot => {
            ProtocolPresentationAuthorCategoryV1::StaleSnapshot
        }
        ferrum_document_render::CurvedElectronArrowGestureCategoryV1::ForeignSession => {
            ProtocolPresentationAuthorCategoryV1::ForeignSession
        }
        ferrum_document_render::CurvedElectronArrowGestureCategoryV1::Consumed
        | ferrum_document_render::CurvedElectronArrowGestureCategoryV1::MismatchedPreview => {
            ProtocolPresentationAuthorCategoryV1::Consumed
        }
        ferrum_document_render::CurvedElectronArrowGestureCategoryV1::InvalidPoint => {
            ProtocolPresentationAuthorCategoryV1::InvalidPoint
        }
        ferrum_document_render::CurvedElectronArrowGestureCategoryV1::RenderPreparation => {
            ProtocolPresentationAuthorCategoryV1::RenderPreparation
        }
        ferrum_document_render::CurvedElectronArrowGestureCategoryV1::SessionConflict => {
            ProtocolPresentationAuthorCategoryV1::SessionConflict
        }
        _ => ProtocolPresentationAuthorCategoryV1::DegenerateGeometry,
    };
    refusal_for_category(
        PresentationAuthoringKindV1::CurvedTerminalArrow,
        category,
        error.to_string(),
    )
}

fn equilibrium_error(error: CurvedEquilibriumArrowGestureErrorV1) -> ExecutionFailureV1 {
    let category = match error.category() {
        ferrum_document_render::CurvedEquilibriumArrowGestureCategoryV1::StaleSnapshot => {
            ProtocolPresentationAuthorCategoryV1::StaleSnapshot
        }
        ferrum_document_render::CurvedEquilibriumArrowGestureCategoryV1::ForeignSession => {
            ProtocolPresentationAuthorCategoryV1::ForeignSession
        }
        ferrum_document_render::CurvedEquilibriumArrowGestureCategoryV1::Consumed
        | ferrum_document_render::CurvedEquilibriumArrowGestureCategoryV1::MismatchedPreview => {
            ProtocolPresentationAuthorCategoryV1::Consumed
        }
        ferrum_document_render::CurvedEquilibriumArrowGestureCategoryV1::InvalidPoint => {
            ProtocolPresentationAuthorCategoryV1::InvalidPoint
        }
        ferrum_document_render::CurvedEquilibriumArrowGestureCategoryV1::RenderPreparation => {
            ProtocolPresentationAuthorCategoryV1::RenderPreparation
        }
        ferrum_document_render::CurvedEquilibriumArrowGestureCategoryV1::SessionConflict => {
            ProtocolPresentationAuthorCategoryV1::SessionConflict
        }
        _ => ProtocolPresentationAuthorCategoryV1::DegenerateGeometry,
    };
    refusal_for_category(
        PresentationAuthoringKindV1::CurvedEquilibriumArrow,
        category,
        error.to_string(),
    )
}

fn path_error(error: PresentationPathRenderErrorV1) -> ExecutionFailureV1 {
    let (category, recovery) = match error {
        PresentationPathRenderErrorV1::InvalidGeometry(_) => {
            let category = ProtocolPresentationAuthorCategoryV1::PathCardinality;
            (
                category,
                recovery_for_presentation_author_category(category),
            )
        }
        PresentationPathRenderErrorV1::ForeignSession => {
            let category = ProtocolPresentationAuthorCategoryV1::ForeignSession;
            (
                category,
                recovery_for_presentation_author_category(category),
            )
        }
        PresentationPathRenderErrorV1::StaleSnapshot => {
            let category = ProtocolPresentationAuthorCategoryV1::StaleSnapshot;
            (
                category,
                recovery_for_presentation_author_category(category),
            )
        }
        PresentationPathRenderErrorV1::MismatchedPreview
        | PresentationPathRenderErrorV1::Consumed => {
            let category = ProtocolPresentationAuthorCategoryV1::Consumed;
            (
                category,
                recovery_for_presentation_author_category(category),
            )
        }
        PresentationPathRenderErrorV1::RenderPreparation => {
            let category = ProtocolPresentationAuthorCategoryV1::RenderPreparation;
            (
                category,
                recovery_for_presentation_author_category(category),
            )
        }
        PresentationPathRenderErrorV1::SessionConflict => {
            let category = ProtocolPresentationAuthorCategoryV1::SessionConflict;
            (
                category,
                recovery_for_presentation_author_category(category),
            )
        }
        PresentationPathRenderErrorV1::Cancelled => (
            ProtocolPresentationAuthorCategoryV1::SessionConflict,
            ProtocolPresentationAuthorRecoveryV1::DocumentUnchanged,
        ),
    };
    refusal(
        PresentationAuthoringKindV1::Path,
        category,
        recovery,
        error.to_string(),
    )
}

fn direct_bond_admission_error(error: DirectBondAdmissionRefusalV1) -> ExecutionFailureV1 {
    refusal_for_category(
        PresentationAuthoringKindV1::DirectBond,
        direct_bond_admission_category(&error),
        error.to_string(),
    )
}

fn direct_bond_prepare_error(error: DocumentSessionError) -> ExecutionFailureV1 {
    let message = error.to_string();
    let category = match error {
        DocumentSessionError::DirectBondAdmission(refusal) => {
            direct_bond_admission_category(&refusal)
        }
        DocumentSessionError::RendererAdmission => {
            ProtocolPresentationAuthorCategoryV1::RenderPreparation
        }
        DocumentSessionError::RevisionConflict { .. } => {
            ProtocolPresentationAuthorCategoryV1::StaleSnapshot
        }
        DocumentSessionError::TransitionAuthorization(
            TransitionAuthorizationRefusalV1::ForeignSession,
        ) => ProtocolPresentationAuthorCategoryV1::ForeignSession,
        DocumentSessionError::TransitionAuthorization(
            TransitionAuthorizationRefusalV1::Consumed,
        ) => ProtocolPresentationAuthorCategoryV1::Consumed,
        _ => ProtocolPresentationAuthorCategoryV1::SessionConflict,
    };
    refusal_for_category(PresentationAuthoringKindV1::DirectBond, category, message)
}

fn presentation_prepare_error(
    kind: PresentationAuthoringKindV1,
    error: DocumentSessionError,
) -> ExecutionFailureV1 {
    let message = error.to_string();
    let category = match error {
        DocumentSessionError::RendererAdmission => {
            ProtocolPresentationAuthorCategoryV1::RenderPreparation
        }
        DocumentSessionError::RevisionConflict { .. } => {
            ProtocolPresentationAuthorCategoryV1::StaleSnapshot
        }
        DocumentSessionError::TransitionAuthorization(
            TransitionAuthorizationRefusalV1::ForeignSession,
        ) => ProtocolPresentationAuthorCategoryV1::ForeignSession,
        DocumentSessionError::TransitionAuthorization(
            TransitionAuthorizationRefusalV1::Consumed,
        ) => ProtocolPresentationAuthorCategoryV1::Consumed,
        _ => ProtocolPresentationAuthorCategoryV1::SessionConflict,
    };
    refusal_for_category(kind, category, message)
}

fn direct_bond_commit_error(error: AdmittedSessionTransitionRefusalV1) -> ExecutionFailureV1 {
    let category = match error {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            ProtocolPresentationAuthorCategoryV1::ForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Consumed => {
            ProtocolPresentationAuthorCategoryV1::Consumed
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            ProtocolPresentationAuthorCategoryV1::RenderPreparation
        }
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            ProtocolPresentationAuthorCategoryV1::StaleSnapshot
        }
        AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            ProtocolPresentationAuthorCategoryV1::SessionConflict
        }
    };
    refusal_for_category(
        PresentationAuthoringKindV1::DirectBond,
        category,
        format!("generic direct-bond transition commit refused: {error:?}"),
    )
}

fn presentation_commit_error(
    kind: PresentationAuthoringKindV1,
    error: AdmittedSessionTransitionRefusalV1,
) -> ExecutionFailureV1 {
    let category = match error {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            ProtocolPresentationAuthorCategoryV1::ForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Consumed => {
            ProtocolPresentationAuthorCategoryV1::Consumed
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            ProtocolPresentationAuthorCategoryV1::RenderPreparation
        }
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            ProtocolPresentationAuthorCategoryV1::StaleSnapshot
        }
        AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            ProtocolPresentationAuthorCategoryV1::SessionConflict
        }
    };
    refusal_for_category(
        kind,
        category,
        format!("generic presentation transition commit refused: {error:?}"),
    )
}

fn direct_bond_admission_category(
    refusal: &DirectBondAdmissionRefusalV1,
) -> ProtocolPresentationAuthorCategoryV1 {
    match refusal {
        DirectBondAdmissionRefusalV1::SelfLoop => ProtocolPresentationAuthorCategoryV1::SelfLoop,
        DirectBondAdmissionRefusalV1::DuplicateBond => {
            ProtocolPresentationAuthorCategoryV1::DuplicateBond
        }
        DirectBondAdmissionRefusalV1::CrossMolecule => {
            ProtocolPresentationAuthorCategoryV1::CrossMolecule
        }
        DirectBondAdmissionRefusalV1::UnsupportedPresentation => {
            ProtocolPresentationAuthorCategoryV1::UnsupportedPresentation
        }
        DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission => {
            ProtocolPresentationAuthorCategoryV1::UnsupportedChemistry
        }
        DirectBondAdmissionRefusalV1::ExceedsChemistryCapacity => {
            ProtocolPresentationAuthorCategoryV1::Capacity
        }
        DirectBondAdmissionRefusalV1::UnrenderableCandidate => {
            ProtocolPresentationAuthorCategoryV1::RenderPreparation
        }
        DirectBondAdmissionRefusalV1::StaleRevision | DirectBondAdmissionRefusalV1::StaleDigest => {
            ProtocolPresentationAuthorCategoryV1::StaleSnapshot
        }
        DirectBondAdmissionRefusalV1::ForeignSession => {
            ProtocolPresentationAuthorCategoryV1::ForeignSession
        }
        DirectBondAdmissionRefusalV1::Consumed => ProtocolPresentationAuthorCategoryV1::Consumed,
        DirectBondAdmissionRefusalV1::UnknownStartAtom
        | DirectBondAdmissionRefusalV1::UnknownEndAtom
        | DirectBondAdmissionRefusalV1::InvalidEndpointInput
        | DirectBondAdmissionRefusalV1::CollapsedEndpoint => {
            ProtocolPresentationAuthorCategoryV1::InvalidEndpoint
        }
    }
}

fn refusal_for_category(
    kind: PresentationAuthoringKindV1,
    category: ProtocolPresentationAuthorCategoryV1,
    message: impl Into<String>,
) -> ExecutionFailureV1 {
    refusal(
        kind,
        category,
        recovery_for_presentation_author_category(category),
        message,
    )
}

const fn recovery_for_presentation_author_category(
    category: ProtocolPresentationAuthorCategoryV1,
) -> ProtocolPresentationAuthorRecoveryV1 {
    match category {
        ProtocolPresentationAuthorCategoryV1::StaleSnapshot
        | ProtocolPresentationAuthorCategoryV1::ForeignSession
        | ProtocolPresentationAuthorCategoryV1::Consumed
        | ProtocolPresentationAuthorCategoryV1::SessionConflict => {
            ProtocolPresentationAuthorRecoveryV1::RefreshAndRestart
        }
        ProtocolPresentationAuthorCategoryV1::InvalidPoint
        | ProtocolPresentationAuthorCategoryV1::DegenerateGeometry
        | ProtocolPresentationAuthorCategoryV1::PathCardinality => {
            ProtocolPresentationAuthorRecoveryV1::ChangeGeometry
        }
        ProtocolPresentationAuthorCategoryV1::UnsupportedPresentation => {
            ProtocolPresentationAuthorRecoveryV1::ChangePresentation
        }
        ProtocolPresentationAuthorCategoryV1::RenderPreparation => {
            ProtocolPresentationAuthorRecoveryV1::DocumentUnchanged
        }
        ProtocolPresentationAuthorCategoryV1::Capacity
        | ProtocolPresentationAuthorCategoryV1::ResourceExhausted => {
            ProtocolPresentationAuthorRecoveryV1::ReportConflict
        }
        ProtocolPresentationAuthorCategoryV1::InvalidEndpoint
        | ProtocolPresentationAuthorCategoryV1::SelfLoop
        | ProtocolPresentationAuthorCategoryV1::DuplicateBond
        | ProtocolPresentationAuthorCategoryV1::CrossMolecule
        | ProtocolPresentationAuthorCategoryV1::UnsupportedChemistry => {
            ProtocolPresentationAuthorRecoveryV1::AdjustEndpoint
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recovery(error: ExecutionFailureV1) -> ProtocolPresentationAuthorRecoveryV1 {
        error
            .presentation_author_refusal
            .expect("renderer refusal is exposed through the protocol")
            .recovery
    }

    #[test]
    fn renderer_authoring_refusals_preserve_actionable_recovery() {
        let geometry = [
            recovery(terminal_error(
                CurvedElectronArrowGestureErrorV1::CollapsedSpan,
            )),
            recovery(equilibrium_error(
                CurvedEquilibriumArrowGestureErrorV1::CollapsedSpan,
            )),
        ];
        assert!(
            geometry
                .iter()
                .all(|value| { *value == ProtocolPresentationAuthorRecoveryV1::ChangeGeometry })
        );

        let restart = [
            recovery(terminal_error(
                CurvedElectronArrowGestureErrorV1::StaleSnapshot,
            )),
            recovery(terminal_error(
                CurvedElectronArrowGestureErrorV1::ForeignSession,
            )),
            recovery(terminal_error(
                CurvedElectronArrowGestureErrorV1::SessionConflict,
            )),
            recovery(equilibrium_error(
                CurvedEquilibriumArrowGestureErrorV1::StaleSnapshot,
            )),
            recovery(equilibrium_error(
                CurvedEquilibriumArrowGestureErrorV1::ForeignSession,
            )),
            recovery(equilibrium_error(
                CurvedEquilibriumArrowGestureErrorV1::SessionConflict,
            )),
            recovery(path_error(PresentationPathRenderErrorV1::StaleSnapshot)),
            recovery(path_error(PresentationPathRenderErrorV1::ForeignSession)),
            recovery(path_error(PresentationPathRenderErrorV1::SessionConflict)),
        ];
        assert!(
            restart
                .iter()
                .all(|value| { *value == ProtocolPresentationAuthorRecoveryV1::RefreshAndRestart })
        );

        let document_unchanged = [
            recovery(terminal_error(
                CurvedElectronArrowGestureErrorV1::RenderPreparation,
            )),
            recovery(equilibrium_error(
                CurvedEquilibriumArrowGestureErrorV1::RenderPreparation,
            )),
            recovery(path_error(PresentationPathRenderErrorV1::RenderPreparation)),
            recovery(path_error(PresentationPathRenderErrorV1::Cancelled)),
        ];
        assert!(
            document_unchanged
                .iter()
                .all(|value| { *value == ProtocolPresentationAuthorRecoveryV1::DocumentUnchanged })
        );
    }
}
