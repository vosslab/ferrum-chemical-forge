use super::*;

pub(super) fn authoring_fence(digest: &str, revision: u64) -> PyResult<DocumentFenceV1> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(ReactionAuthoringChoicesError::new_err(
            "expected digest must be exactly 64 lowercase hexadecimal characters",
        ));
    }
    let mut bytes = [0; 32];
    for (index, pair) in digest.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (authoring_hex(pair[0]) << 4) | authoring_hex(pair[1]);
    }
    Ok(DocumentFenceV1::new(revision, bytes))
}
const fn authoring_hex(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}
fn choice_kind(value: ReactionAuthoringChoiceKindV1) -> PyReactionAuthoringChoiceKindV1 {
    match value {
        ReactionAuthoringChoiceKindV1::Molecule => PyReactionAuthoringChoiceKindV1::Molecule,
        ReactionAuthoringChoiceKindV1::Arrow => PyReactionAuthoringChoiceKindV1::Arrow,
        ReactionAuthoringChoiceKindV1::Plus => PyReactionAuthoringChoiceKindV1::Plus,
        ReactionAuthoringChoiceKindV1::ConditionText => {
            PyReactionAuthoringChoiceKindV1::ConditionText
        }
    }
}
fn choice_availability(
    value: ReactionAuthoringChoiceAvailabilityV1,
) -> PyReactionAuthoringChoiceAvailabilityV1 {
    match value {
        ReactionAuthoringChoiceAvailabilityV1::Eligible => {
            PyReactionAuthoringChoiceAvailabilityV1::Eligible
        }
        ReactionAuthoringChoiceAvailabilityV1::AlreadyInReaction => {
            PyReactionAuthoringChoiceAvailabilityV1::AlreadyInReaction
        }
    }
}
fn exclusion_reason(
    value: ReactionAuthoringExclusionReasonV1,
) -> PyReactionAuthoringExclusionReasonV1 {
    match value {
        ReactionAuthoringExclusionReasonV1::DisplayOnly => {
            PyReactionAuthoringExclusionReasonV1::DisplayOnly
        }
        ReactionAuthoringExclusionReasonV1::Unrenderable => {
            PyReactionAuthoringExclusionReasonV1::Unrenderable
        }
        ReactionAuthoringExclusionReasonV1::MissingSemanticIdentity => {
            PyReactionAuthoringExclusionReasonV1::MissingSemanticIdentity
        }
        ReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity => {
            PyReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity
        }
        ReactionAuthoringExclusionReasonV1::KindMismatch => {
            PyReactionAuthoringExclusionReasonV1::KindMismatch
        }
    }
}
fn exclusion_recovery(
    value: ReactionAuthoringExclusionRecoveryV1,
) -> PyReactionAuthoringExclusionRecoveryV1 {
    match value {
        ReactionAuthoringExclusionRecoveryV1::ChooseSupportedMember => {
            PyReactionAuthoringExclusionRecoveryV1::ChooseSupportedMember
        }
        ReactionAuthoringExclusionRecoveryV1::RepairDocument => {
            PyReactionAuthoringExclusionRecoveryV1::RepairDocument
        }
    }
}
fn choice_bounds(value: RenderInteractionBoundsV1) -> PyReactionAuthoringChoiceBoundsV1 {
    PyReactionAuthoringChoiceBoundsV1 {
        left: value.left(),
        top: value.top(),
        right: value.right(),
        bottom: value.bottom(),
    }
}
fn reaction_diagnostic(value: ReactionDefinitionDiagnosticV1) -> PyReactionDiagnosticV1 {
    let reason = match value {
        ReactionDefinitionDiagnosticV1::MissingReactionId => {
            PyReactionDiagnosticReasonV1::MissingReactionId
        }
        ReactionDefinitionDiagnosticV1::EmptyReactionId => {
            PyReactionDiagnosticReasonV1::EmptyReactionId
        }
        ReactionDefinitionDiagnosticV1::UnknownRoleChild => {
            PyReactionDiagnosticReasonV1::UnknownRoleChild
        }
        ReactionDefinitionDiagnosticV1::MissingIdref => PyReactionDiagnosticReasonV1::MissingIdref,
        ReactionDefinitionDiagnosticV1::EmptyIdref => PyReactionDiagnosticReasonV1::EmptyIdref,
        ReactionDefinitionDiagnosticV1::MissingReactants => {
            PyReactionDiagnosticReasonV1::MissingReactants
        }
        ReactionDefinitionDiagnosticV1::MissingProducts => {
            PyReactionDiagnosticReasonV1::MissingProducts
        }
        ReactionDefinitionDiagnosticV1::MissingArrow => PyReactionDiagnosticReasonV1::MissingArrow,
        ReactionDefinitionDiagnosticV1::MultipleArrows => {
            PyReactionDiagnosticReasonV1::MultipleArrows
        }
        ReactionDefinitionDiagnosticV1::MissingTarget => {
            PyReactionDiagnosticReasonV1::MissingTarget
        }
        ReactionDefinitionDiagnosticV1::UnrenderableMember => {
            PyReactionDiagnosticReasonV1::UnrenderableMember
        }
        ReactionDefinitionDiagnosticV1::WrongTargetKind => {
            PyReactionDiagnosticReasonV1::WrongTargetKind
        }
        ReactionDefinitionDiagnosticV1::DuplicateTarget => {
            PyReactionDiagnosticReasonV1::DuplicateTarget
        }
        ReactionDefinitionDiagnosticV1::CrossReactionReuse => {
            PyReactionDiagnosticReasonV1::CrossReactionReuse
        }
    };
    let selector_source = match value {
        ReactionDefinitionDiagnosticV1::UnrenderableMember => {
            PyReactionDiagnosticSelectorSourceV1::RendererAdmission
        }
        _ => PyReactionDiagnosticSelectorSourceV1::DirectCdmlSemanticIndex,
    };
    PyReactionDiagnosticV1 {
        reason,
        recovery: PyReactionDiagnosticRecoveryV1::RepairDocument,
        selector_source,
    }
}
pub(super) fn authoring_choices(
    py: Python<'_>,
    value: ReactionAuthoringChoicesV1,
) -> PyResult<PyReactionAuthoringChoicesV1> {
    let fence = value.fence();
    let choices = value
        .choices()
        .iter()
        .map(|choice| {
            Py::new(
                py,
                PyReactionAuthoringChoiceV1 {
                    identifier: choice.identifier().to_owned(),
                    source_order: choice.source_order(),
                    kind: choice_kind(choice.kind()),
                    availability: choice_availability(choice.availability()),
                    label: choice.label().to_owned(),
                    bounds: choice_bounds(choice.bounds()),
                },
            )
        })
        .collect::<PyResult<_>>()?;
    let exclusions = value
        .exclusions()
        .iter()
        .map(|exclusion| {
            Py::new(
                py,
                PyReactionAuthoringExclusionV1 {
                    diagnostic_key: exclusion.diagnostic_key().to_owned(),
                    reason: exclusion_reason(exclusion.reason()),
                    recovery: exclusion_recovery(exclusion.recovery()),
                    label: exclusion.label().to_owned(),
                },
            )
        })
        .collect::<PyResult<_>>()?;
    Ok(PyReactionAuthoringChoicesV1 {
        value,
        choices,
        exclusions,
        revision: fence.revision(),
        digest: fence
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    })
}
pub(super) fn reaction_list(
    py: Python<'_>,
    value: ReactionListObservationV1,
) -> PyResult<PyReactionListObservationV1> {
    let fence = value.fence();
    let reactions = value
        .reactions()
        .iter()
        .map(|reaction| {
            let diagnostics = reaction
                .diagnostics()
                .iter()
                .map(|diagnostic| Py::new(py, reaction_diagnostic(*diagnostic)))
                .collect::<PyResult<_>>()?;
            let members = reaction
                .members()
                .iter()
                .map(|member| {
                    Py::new(
                        py,
                        PyReactionMemberObservationV1 {
                            identifier: member.identifier().to_owned(),
                            role: member.role().local_name().to_owned(),
                            role_ordinal: member.role_ordinal(),
                            source_order: member.source_order(),
                            bounds: member.bounds().map(choice_bounds),
                        },
                    )
                })
                .collect::<PyResult<_>>()?;
            Py::new(
                py,
                PyReactionObservationV1 {
                    reaction_id: reaction.reaction_id().to_owned(),
                    source_order: reaction.source_order(),
                    disposition: match reaction.disposition() {
                        ReactionDefinitionDispositionV1::Strict => {
                            PyReactionDefinitionDispositionV1::Strict
                        }
                        ReactionDefinitionDispositionV1::DisplayOnly => {
                            PyReactionDefinitionDispositionV1::DisplayOnly
                        }
                    },
                    membership_digest: reaction.membership_digest().to_owned(),
                    union_bounds: reaction.union_bounds().map(choice_bounds),
                    diagnostics,
                    members,
                },
            )
        })
        .collect::<PyResult<_>>()?;
    Ok(PyReactionListObservationV1 {
        value,
        reactions,
        revision: fence.revision(),
        digest: fence
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    })
}
pub(super) fn authoring_error(py: Python<'_>, error: RenderInteractionErrorV1) -> PyErr {
    let category = match error {
        RenderInteractionErrorV1::StaleRevision | RenderInteractionErrorV1::StaleDigest => {
            PyReactionAuthoringChoicesRefusalCategoryV1::StaleSnapshot
        }
        RenderInteractionErrorV1::ForeignSession => {
            PyReactionAuthoringChoicesRefusalCategoryV1::ForeignSession
        }
        _ => PyReactionAuthoringChoicesRefusalCategoryV1::Observation,
    };
    let exception = ReactionAuthoringChoicesError::new_err(error.to_string());
    exception
        .value(py)
        .setattr("category", authoring_refusal_category_member(py, category))
        .expect("category attaches");
    exception
}

fn authoring_refusal_category_member(
    py: Python<'_>,
    category: PyReactionAuthoringChoicesRefusalCategoryV1,
) -> Py<PyAny> {
    let name = match category {
        PyReactionAuthoringChoicesRefusalCategoryV1::StaleSnapshot => "stale_snapshot",
        PyReactionAuthoringChoicesRefusalCategoryV1::ForeignSession => "foreign_session",
        PyReactionAuthoringChoicesRefusalCategoryV1::Observation => "observation",
    };
    py.get_type::<PyReactionAuthoringChoicesRefusalCategoryV1>()
        .getattr(name)
        .expect("registered authoring refusal category member")
        .unbind()
}

pub(super) fn gesture_error(py: Python<'_>, error: ReactionGestureErrorV1) -> PyErr {
    reaction_error(py, error.category(), error.to_string())
}
fn reaction_error(py: Python<'_>, category: ReactionGestureCategoryV1, message: String) -> PyErr {
    let category = match category {
        ReactionGestureCategoryV1::StaleSnapshot => PyReactionRefusalCategoryV1::StaleSnapshot,
        ReactionGestureCategoryV1::ForeignSession => PyReactionRefusalCategoryV1::ForeignSession,
        ReactionGestureCategoryV1::ReplayedGesture => PyReactionRefusalCategoryV1::ReplayedGesture,
        ReactionGestureCategoryV1::InvalidRequest => PyReactionRefusalCategoryV1::InvalidRequest,
        ReactionGestureCategoryV1::MissingTarget => PyReactionRefusalCategoryV1::MissingTarget,
        ReactionGestureCategoryV1::WrongTargetKind => PyReactionRefusalCategoryV1::WrongTargetKind,
        ReactionGestureCategoryV1::DuplicateTarget => PyReactionRefusalCategoryV1::DuplicateTarget,
        ReactionGestureCategoryV1::CrossReactionReuse => {
            PyReactionRefusalCategoryV1::CrossReactionReuse
        }
        ReactionGestureCategoryV1::UnrenderableDocument => {
            PyReactionRefusalCategoryV1::UnrenderableDocument
        }
        ReactionGestureCategoryV1::RenderPreparation => {
            PyReactionRefusalCategoryV1::RenderPreparation
        }
        ReactionGestureCategoryV1::SessionConflict => PyReactionRefusalCategoryV1::SessionConflict,
        ReactionGestureCategoryV1::MissingReaction => PyReactionRefusalCategoryV1::MissingReaction,
        ReactionGestureCategoryV1::LegacyDefinitionNotEditable => {
            PyReactionRefusalCategoryV1::LegacyDefinitionNotEditable
        }
        ReactionGestureCategoryV1::MembershipChanged => {
            PyReactionRefusalCategoryV1::MembershipChanged
        }
        ReactionGestureCategoryV1::RendererExclusion => {
            PyReactionRefusalCategoryV1::RendererExclusion
        }
        _ => {
            unreachable!("a new reaction refusal category requires an explicit frozen PyO3 member")
        }
    };
    let recovery = match category {
        PyReactionRefusalCategoryV1::StaleSnapshot
        | PyReactionRefusalCategoryV1::ForeignSession
        | PyReactionRefusalCategoryV1::ReplayedGesture
        | PyReactionRefusalCategoryV1::SessionConflict => {
            PyReactionRefusalRecoveryV1::RefreshAndRestart
        }
        PyReactionRefusalCategoryV1::InvalidRequest
        | PyReactionRefusalCategoryV1::MissingTarget
        | PyReactionRefusalCategoryV1::WrongTargetKind
        | PyReactionRefusalCategoryV1::DuplicateTarget
        | PyReactionRefusalCategoryV1::CrossReactionReuse => {
            PyReactionRefusalRecoveryV1::CorrectSelectors
        }
        PyReactionRefusalCategoryV1::UnrenderableDocument
        | PyReactionRefusalCategoryV1::RenderPreparation => {
            PyReactionRefusalRecoveryV1::ChooseRenderableMembers
        }
        PyReactionRefusalCategoryV1::MissingReaction
        | PyReactionRefusalCategoryV1::MembershipChanged => {
            PyReactionRefusalRecoveryV1::RefreshAndRestart
        }
        PyReactionRefusalCategoryV1::LegacyDefinitionNotEditable => {
            PyReactionRefusalRecoveryV1::RepairLegacyDefinition
        }
        PyReactionRefusalCategoryV1::RendererExclusion => {
            PyReactionRefusalRecoveryV1::ChooseRenderableMembers
        }
    };
    let exception = ReactionGestureError::new_err(message);
    let value = exception.value(py);
    value
        .setattr("category", reaction_refusal_category_member(py, category))
        .expect("category attaches");
    value
        .setattr("recovery", reaction_refusal_recovery_member(py, recovery))
        .expect("recovery attaches");
    exception
}

fn reaction_refusal_category_member(
    py: Python<'_>,
    category: PyReactionRefusalCategoryV1,
) -> Py<PyAny> {
    let name = match category {
        PyReactionRefusalCategoryV1::StaleSnapshot => "stale_snapshot",
        PyReactionRefusalCategoryV1::ForeignSession => "foreign_session",
        PyReactionRefusalCategoryV1::ReplayedGesture => "replayed_gesture",
        PyReactionRefusalCategoryV1::InvalidRequest => "invalid_request",
        PyReactionRefusalCategoryV1::MissingTarget => "missing_target",
        PyReactionRefusalCategoryV1::WrongTargetKind => "wrong_target_kind",
        PyReactionRefusalCategoryV1::DuplicateTarget => "duplicate_target",
        PyReactionRefusalCategoryV1::CrossReactionReuse => "cross_reaction_reuse",
        PyReactionRefusalCategoryV1::UnrenderableDocument => "unrenderable_document",
        PyReactionRefusalCategoryV1::RenderPreparation => "render_preparation",
        PyReactionRefusalCategoryV1::SessionConflict => "session_conflict",
        PyReactionRefusalCategoryV1::MissingReaction => "missing_reaction",
        PyReactionRefusalCategoryV1::LegacyDefinitionNotEditable => {
            "legacy_definition_not_editable"
        }
        PyReactionRefusalCategoryV1::MembershipChanged => "membership_changed",
        PyReactionRefusalCategoryV1::RendererExclusion => "renderer_exclusion",
    };
    py.get_type::<PyReactionRefusalCategoryV1>()
        .getattr(name)
        .expect("registered reaction refusal category member")
        .unbind()
}

fn reaction_refusal_recovery_member(
    py: Python<'_>,
    recovery: PyReactionRefusalRecoveryV1,
) -> Py<PyAny> {
    let name = match recovery {
        PyReactionRefusalRecoveryV1::RefreshAndRestart => "refresh_and_restart",
        PyReactionRefusalRecoveryV1::CorrectSelectors => "correct_selectors",
        PyReactionRefusalRecoveryV1::ChooseRenderableMembers => "choose_renderable_members",
        PyReactionRefusalRecoveryV1::RepairLegacyDefinition => "repair_legacy_definition",
    };
    py.get_type::<PyReactionRefusalRecoveryV1>()
        .getattr(name)
        .expect("registered reaction refusal recovery member")
        .unbind()
}
