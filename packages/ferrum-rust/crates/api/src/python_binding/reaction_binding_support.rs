use super::*;

pub(super) enum ReactionBindingErrorV1 {
    Command(ReactionAuthoringCommandRefusalV1),
    Selection(ReactionMemberSelectionRefusalV1),
    StaleSnapshot,
}

impl From<ReactionAuthoringCommandRefusalV1> for ReactionBindingErrorV1 {
    fn from(value: ReactionAuthoringCommandRefusalV1) -> Self {
        Self::Command(value)
    }
}

impl From<ReactionMemberSelectionRefusalV1> for ReactionBindingErrorV1 {
    fn from(value: ReactionMemberSelectionRefusalV1) -> Self {
        Self::Selection(value)
    }
}

pub(super) fn document_object_id(
    py: Python<'_>,
    value: String,
) -> PyResult<ferrum_document::DocumentObjectIdV1> {
    ferrum_document::DocumentObjectIdV1::parse(value).map_err(|error| {
        command_error_with(
            py,
            PyReactionCommandRefusalCategoryV1::InvalidMembers,
            PyReactionCommandRefusalRecoveryV1::CorrectDocumentObjectIds,
            error.to_string(),
        )
    })
}

pub(super) fn snapshot_fence(
    session: &ferrum_document_render::RenderInteractionSessionV1,
    expected_revision: u64,
    expected_digest_hex: &str,
) -> Result<DocumentFenceV1, ReactionBindingErrorV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| ReactionBindingErrorV1::StaleSnapshot)?;
    if snapshot.revision() != expected_revision
        || hex_digest(snapshot.digest()) != expected_digest_hex
    {
        return Err(ReactionBindingErrorV1::StaleSnapshot);
    }
    Ok(DocumentFenceV1::new(
        snapshot.revision(),
        *snapshot.digest(),
    ))
}

pub(super) fn reaction_targets(
    py: Python<'_>,
    reactants: Vec<String>,
    arrow: String,
    products: Vec<String>,
    reagents: Vec<String>,
    pluses: Vec<String>,
) -> PyResult<DocumentReactionMemberTargetsV1> {
    let parse_many = |values: Vec<String>| {
        values
            .into_iter()
            .map(|value| document_object_id(py, value))
            .collect::<PyResult<Vec<_>>>()
    };
    DocumentReactionMemberTargetsV1::new(
        parse_many(reactants)?,
        document_object_id(py, arrow)?,
        parse_many(products)?,
        parse_many(reagents)?,
        parse_many(pluses)?,
    )
    .map_err(|error| {
        command_error_with(
            py,
            PyReactionCommandRefusalCategoryV1::InvalidMembers,
            PyReactionCommandRefusalRecoveryV1::CorrectDocumentObjectIds,
            error.to_string(),
        )
    })
}

pub(super) fn reaction_list(
    py: Python<'_>,
    value: DocumentReactionListObservationV1,
    fence: DocumentFenceV1,
) -> PyResult<PyReactionListObservationV1> {
    let reactions = value
        .reactions()
        .iter()
        .map(|reaction| {
            let members = reaction
                .members()
                .iter()
                .map(|member| {
                    Py::new(
                        py,
                        PyReactionMemberObservationV1 {
                            document_object_id: member.object_id().as_str().to_owned(),
                            document_paint_order: member.document_paint_order(),
                            role: member.role().local_name().to_owned(),
                            role_ordinal: member.role_ordinal(),
                        },
                    )
                })
                .collect::<PyResult<_>>()?;
            Py::new(
                py,
                PyReactionObservationV1 {
                    document_object_id: reaction.reaction_object_id().as_str().to_owned(),
                    strict: reaction.disposition() == DocumentReactionListDispositionV1::Strict,
                    diagnostics: reaction
                        .diagnostics()
                        .iter()
                        .map(|value| format!("{value:?}").to_lowercase())
                        .collect(),
                    members,
                },
            )
        })
        .collect::<PyResult<_>>()?;
    Ok(PyReactionListObservationV1 {
        value,
        reactions,
        revision: fence.revision(),
        digest: hex_digest(&fence.digest()),
    })
}

pub(super) fn command_error(py: Python<'_>, error: impl Into<ReactionBindingErrorV1>) -> PyErr {
    match error.into() {
        ReactionBindingErrorV1::StaleSnapshot => command_error_with(
            py,
            PyReactionCommandRefusalCategoryV1::StaleSnapshot,
            PyReactionCommandRefusalRecoveryV1::RefreshAndRestart,
            "reaction command was prepared from a stale document snapshot".to_owned(),
        ),
        ReactionBindingErrorV1::Command(error) => match error {
            ReactionAuthoringCommandRefusalV1::ForeignSession => command_error_with(
                py,
                PyReactionCommandRefusalCategoryV1::ForeignSession,
                PyReactionCommandRefusalRecoveryV1::RefreshAndRestart,
                error.to_string(),
            ),
            ReactionAuthoringCommandRefusalV1::Consumed => command_error_with(
                py,
                PyReactionCommandRefusalCategoryV1::Consumed,
                PyReactionCommandRefusalRecoveryV1::RefreshAndRestart,
                error.to_string(),
            ),
            ReactionAuthoringCommandRefusalV1::StaleRevision
            | ReactionAuthoringCommandRefusalV1::StaleDigest => command_error_with(
                py,
                PyReactionCommandRefusalCategoryV1::StaleSnapshot,
                PyReactionCommandRefusalRecoveryV1::RefreshAndRestart,
                error.to_string(),
            ),
            ReactionAuthoringCommandRefusalV1::InvalidMembers(_) => command_error_with(
                py,
                PyReactionCommandRefusalCategoryV1::InvalidMembers,
                PyReactionCommandRefusalRecoveryV1::CorrectDocumentObjectIds,
                error.to_string(),
            ),
            ReactionAuthoringCommandRefusalV1::InvalidSelection(_) => command_error_with(
                py,
                PyReactionCommandRefusalCategoryV1::InvalidSelection,
                PyReactionCommandRefusalRecoveryV1::RefreshAndRestart,
                error.to_string(),
            ),
        },
        ReactionBindingErrorV1::Selection(error) => command_error_with(
            py,
            PyReactionCommandRefusalCategoryV1::InvalidSelection,
            PyReactionCommandRefusalRecoveryV1::RefreshAndRestart,
            error.to_string(),
        ),
    }
}

fn command_error_with(
    py: Python<'_>,
    category: PyReactionCommandRefusalCategoryV1,
    recovery: PyReactionCommandRefusalRecoveryV1,
    message: String,
) -> PyErr {
    let error = ReactionCommandError::new_err(message.clone());
    let value = error.value(py);
    value.setattr("reason", message).expect("reason attaches");
    value
        .setattr("category", command_category(py, category))
        .expect("category attaches");
    value
        .setattr("recovery", command_recovery(py, recovery))
        .expect("recovery attaches");
    error
}

fn command_category(py: Python<'_>, value: PyReactionCommandRefusalCategoryV1) -> Py<PyAny> {
    let name = match value {
        PyReactionCommandRefusalCategoryV1::StaleSnapshot => "stale_snapshot",
        PyReactionCommandRefusalCategoryV1::ForeignSession => "foreign_session",
        PyReactionCommandRefusalCategoryV1::Consumed => "consumed",
        PyReactionCommandRefusalCategoryV1::InvalidMembers => "invalid_members",
        PyReactionCommandRefusalCategoryV1::InvalidSelection => "invalid_selection",
        PyReactionCommandRefusalCategoryV1::RendererAdmission => "renderer_admission",
        PyReactionCommandRefusalCategoryV1::SessionConflict => "session_conflict",
    };
    py.get_type::<PyReactionCommandRefusalCategoryV1>()
        .getattr(name)
        .expect("registered member")
        .unbind()
}

fn command_recovery(py: Python<'_>, value: PyReactionCommandRefusalRecoveryV1) -> Py<PyAny> {
    let name = match value {
        PyReactionCommandRefusalRecoveryV1::RefreshAndRestart => "refresh_and_restart",
        PyReactionCommandRefusalRecoveryV1::CorrectDocumentObjectIds => {
            "correct_document_object_ids"
        }
        PyReactionCommandRefusalRecoveryV1::RepairDocument => "repair_document",
    };
    py.get_type::<PyReactionCommandRefusalRecoveryV1>()
        .getattr(name)
        .expect("registered member")
        .unbind()
}

fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
