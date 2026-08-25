use super::reaction_binding_support::{
    command_error, document_object_id, reaction_list, reaction_targets, snapshot_fence,
};
use super::*;

#[pymethods]
impl PyDocumentSession {
    fn observe_reaction_list_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
    ) -> PyResult<PyReactionListObservationV1> {
        let fence = snapshot_fence(&self.session, expected_revision, &expected_digest_hex)
            .map_err(|error| command_error(py, error))?;
        self.session
            .observe_reaction_list_v1()
            .map_err(|error| command_error(py, error))
            .and_then(|value| reaction_list(py, value, fence))
    }

    fn select_reaction_v1(
        &self,
        py: Python<'_>,
        list: PyRef<'_, PyReactionListObservationV1>,
        reaction_document_object_id: String,
    ) -> PyResult<PyReactionSelectionV1> {
        let object_id = document_object_id(py, reaction_document_object_id)?;
        let reaction = list
            .value
            .reactions()
            .iter()
            .find(|value| value.reaction_object_id() == &object_id)
            .ok_or_else(|| command_error(py, ReactionMemberSelectionRefusalV1::UnknownReaction))?;
        let observation = reaction.selection_observation().ok_or_else(|| {
            command_error(py, ReactionMemberSelectionRefusalV1::UnresolvedReaction)
        })?;
        self.session
            .select_reaction_members_v1(observation)
            .map(|value| PyReactionSelectionV1 {
                value,
                reaction_document_object_id: object_id.as_str().to_owned(),
            })
            .map_err(|error| command_error(py, error))
    }

    fn validate_reaction_selection_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PyReactionSelectionV1>,
    ) -> PyResult<()> {
        std::ops::Deref::deref(&self.session)
            .validate_reaction_selection_v1(&selection.value)
            .map_err(|error| command_error(py, error))
    }

    #[allow(clippy::too_many_arguments)]
    fn begin_create_reaction_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        reactant_document_object_ids: Vec<String>,
        product_document_object_ids: Vec<String>,
        arrow_document_object_id: String,
        reagent_document_object_ids: Vec<String>,
        plus_document_object_ids: Vec<String>,
    ) -> PyResult<PyCreateReactionCommandV1> {
        let fence = snapshot_fence(&self.session, expected_revision, &expected_digest_hex)
            .map_err(|error| command_error(py, error))?;
        let targets = reaction_targets(
            py,
            reactant_document_object_ids,
            arrow_document_object_id,
            product_document_object_ids,
            reagent_document_object_ids,
            plus_document_object_ids,
        )?;
        self.session
            .begin_create_reaction_v1(self.session.issue_authoring_capability_v1(), fence, targets)
            .map(|value| PyCreateReactionCommandV1 { value: Some(value) })
            .map_err(|error| command_error(py, error))
    }

    #[allow(clippy::too_many_arguments)]
    fn begin_replace_reaction_members_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PyReactionSelectionV1>,
        reactant_document_object_ids: Vec<String>,
        product_document_object_ids: Vec<String>,
        arrow_document_object_id: String,
        reagent_document_object_ids: Vec<String>,
        plus_document_object_ids: Vec<String>,
    ) -> PyResult<PyReplaceReactionMembersCommandV1> {
        let targets = reaction_targets(
            py,
            reactant_document_object_ids,
            arrow_document_object_id,
            product_document_object_ids,
            reagent_document_object_ids,
            plus_document_object_ids,
        )?;
        self.session
            .begin_replace_reaction_members_v1(
                self.session.issue_authoring_capability_v1(),
                selection.value.clone(),
                targets,
            )
            .map(|value| PyReplaceReactionMembersCommandV1 { value: Some(value) })
            .map_err(|error| command_error(py, error))
    }

    fn begin_delete_reaction_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PyReactionSelectionV1>,
    ) -> PyResult<PyDeleteReactionCommandV1> {
        self.session
            .begin_delete_reaction_v1(
                self.session.issue_authoring_capability_v1(),
                selection.value.clone(),
            )
            .map(|value| PyDeleteReactionCommandV1 { value: Some(value) })
            .map_err(|error| command_error(py, error))
    }

    fn resolve_create_reaction_command_v1(
        &self,
        py: Python<'_>,
        mut command: PyRefMut<'_, PyCreateReactionCommandV1>,
    ) -> PyResult<PySessionOperationTransitionRequestV1> {
        let command = command
            .value
            .take()
            .ok_or_else(|| command_error(py, ReactionAuthoringCommandRefusalV1::Replayed))?;
        self.session
            .resolve_create_reaction_command_v1(command)
            .map(PySessionOperationTransitionRequestV1::from_request)
            .map_err(|error| command_error(py, error))
    }

    fn resolve_replace_reaction_members_command_v1(
        &self,
        py: Python<'_>,
        mut command: PyRefMut<'_, PyReplaceReactionMembersCommandV1>,
    ) -> PyResult<PySessionOperationTransitionRequestV1> {
        let command = command
            .value
            .take()
            .ok_or_else(|| command_error(py, ReactionAuthoringCommandRefusalV1::Replayed))?;
        self.session
            .resolve_replace_reaction_members_command_v1(command)
            .map(PySessionOperationTransitionRequestV1::from_request)
            .map_err(|error| command_error(py, error))
    }

    fn resolve_delete_reaction_command_v1(
        &self,
        py: Python<'_>,
        mut command: PyRefMut<'_, PyDeleteReactionCommandV1>,
    ) -> PyResult<PySessionOperationTransitionRequestV1> {
        let command = command
            .value
            .take()
            .ok_or_else(|| command_error(py, ReactionAuthoringCommandRefusalV1::Replayed))?;
        self.session
            .resolve_delete_reaction_command_v1(command)
            .map(PySessionOperationTransitionRequestV1::from_request)
            .map_err(|error| command_error(py, error))
    }
}
