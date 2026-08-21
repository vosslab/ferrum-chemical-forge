use super::reaction_binding_support::{
    authoring_choices, authoring_error, authoring_fence, gesture_error, reaction_list,
};
use super::*;

#[pymethods]
impl PyDocumentSession {
    /// Return source-ordered immutable reaction membership and renderer facts.
    fn observe_reaction_list_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
    ) -> PyResult<PyReactionListObservationV1> {
        let fence = authoring_fence(&expected_digest_hex, expected_revision)?;
        self.session
            .observe_reaction_list_v1(fence)
            .map_err(|error| authoring_error(py, error))
            .and_then(|value| reaction_list(py, value))
    }
    /// Issue one opaque strict reaction selection from a fresh list observation.
    fn select_reaction_v1(
        &self,
        py: Python<'_>,
        list: PyRef<'_, PyReactionListObservationV1>,
        reaction_id: String,
    ) -> PyResult<PyReactionSelectionV1> {
        self.session
            .select_reaction_v1(&list.value, &reaction_id)
            .map(|value| PyReactionSelectionV1 {
                reaction_id: value.reaction_id().to_owned(),
                value,
            })
            .map_err(|error| authoring_error(py, error))
    }
    /// Refuse foreign or stale opaque reaction selection without changing CDML.
    fn validate_reaction_selection_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PyReactionSelectionV1>,
    ) -> PyResult<()> {
        self.session
            .validate_reaction_selection_v1(&selection.value)
            .map_err(|error| authoring_error(py, error))
    }
    /// Begin a complete membership replacement from one opaque strict selection.
    #[allow(clippy::too_many_arguments)]
    fn begin_reaction_membership_patch_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PyReactionSelectionV1>,
        expected_revision: u64,
        reactants: Vec<String>,
        products: Vec<String>,
        arrow: String,
        conditions: Vec<String>,
        pluses: Vec<String>,
    ) -> PyResult<PyReactionLifecycleGestureV1> {
        let request = ReactionMembershipPatchRequestV1::new(
            expected_revision,
            reactants,
            products,
            arrow,
            conditions,
            pluses,
        )
        .map_err(|error| gesture_error(py, error))?;
        begin_api_reaction_membership_patch_v1(&self.session, &selection.value, request)
            .map(|value| PyReactionLifecycleGestureV1 { value })
            .map_err(|error| gesture_error(py, error))
    }
    /// Begin removal of only one selected strict reaction definition.
    fn begin_reaction_definition_delete_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PyReactionSelectionV1>,
    ) -> PyResult<PyReactionLifecycleGestureV1> {
        begin_api_reaction_definition_delete_v1(&self.session, &selection.value)
            .map(|value| PyReactionLifecycleGestureV1 { value })
            .map_err(|error| gesture_error(py, error))
    }
    /// Renderer-preflight a private lifecycle candidate; no document mutation occurs here.
    fn prepare_reaction_lifecycle_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyReactionLifecycleGestureV1>,
    ) -> PyResult<PyPreparedReactionLifecycleV1> {
        prepare_api_reaction_lifecycle_v1(&mut self.session, &gesture.value)
            .map(|value| PyPreparedReactionLifecycleV1 { value })
            .map_err(|error| gesture_error(py, error))
    }
    /// Commit exactly one prepared lifecycle receipt.
    fn commit_reaction_lifecycle_v1(
        &mut self,
        py: Python<'_>,
        mut prepared: PyRefMut<'_, PyPreparedReactionLifecycleV1>,
    ) -> PyResult<PyReactionLifecycleCommitV1> {
        commit_api_reaction_lifecycle_v1(&mut self.session, &mut prepared.value)
            .map(|commit| PyReactionLifecycleCommitV1 {
                reaction_id: commit.reaction_id().to_owned(),
                result: commit.result().clone().into(),
            })
            .map_err(|error| gesture_error(py, error))
    }
    /// Begin one opaque aggregate translation from an exact strict selection.
    #[pyo3(signature = (selection, press_x, press_y, view_hex_grid=false))]
    fn begin_reaction_translation_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PyReactionSelectionV1>,
        press_x: f64,
        press_y: f64,
        view_hex_grid: bool,
    ) -> PyResult<PyReactionTranslationGestureV1> {
        let snap = if view_hex_grid {
            RenderInteractionSnapV1::with_grid(RenderInteractionGridSnapPolicyV1::ViewHexGrid)
        } else {
            RenderInteractionSnapV1::free()
        };
        begin_api_reaction_translation_v1(&self.session, &selection.value, press_x, press_y, snap)
            .map(|value| PyReactionTranslationGestureV1 { value })
            .map_err(|error| gesture_error(py, error))
    }
    /// Compute a transient Rust-owned translation preview without changing CDML.
    fn preview_reaction_translation_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyReactionTranslationGestureV1>,
        pointer_x: f64,
        pointer_y: f64,
    ) -> PyResult<PyReactionTranslationPreviewV1> {
        preview_api_reaction_translation_v1(&self.session, &gesture.value, pointer_x, pointer_y)
            .map(|value| PyReactionTranslationPreviewV1 { value })
            .map_err(|error| gesture_error(py, error))
    }
    /// Renderer-preflight a selected-reaction translation candidate without mutation.
    fn prepare_reaction_translation_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyReactionTranslationGestureV1>,
        preview: PyRef<'_, PyReactionTranslationPreviewV1>,
    ) -> PyResult<PyPreparedReactionTranslationV1> {
        prepare_api_reaction_translation_v1(&mut self.session, &gesture.value, &preview.value)
            .map(|value| PyPreparedReactionTranslationV1 { value })
            .map_err(|error| gesture_error(py, error))
    }
    /// Commit exactly one renderer-admitted aggregate translation receipt.
    fn commit_reaction_translation_v1(
        &mut self,
        py: Python<'_>,
        mut prepared: PyRefMut<'_, PyPreparedReactionTranslationV1>,
    ) -> PyResult<PyReactionTranslationCommitV1> {
        commit_api_reaction_translation_v1(&mut self.session, &mut prepared.value)
            .map(|commit| PyReactionTranslationCommitV1 {
                reaction_id: commit.reaction_id().to_owned(),
                result: commit.result().clone().into(),
            })
            .map_err(|error| gesture_error(py, error))
    }
    /// Return immutable renderer-observed reaction-member choices for one exact snapshot.
    fn observe_reaction_authoring_choices_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
    ) -> PyResult<PyReactionAuthoringChoicesV1> {
        let fence = authoring_fence(&expected_digest_hex, expected_revision)?;
        self.session
            .observe_reaction_authoring_choices_v1(fence)
            .map_err(|error| authoring_error(py, error))
            .and_then(|value| authoring_choices(py, value))
    }
    /// Refuse a stale or foreign immutable authoring observation without changing CDML.
    fn validate_reaction_authoring_choices_v1(
        &self,
        py: Python<'_>,
        choices: PyRef<'_, PyReactionAuthoringChoicesV1>,
    ) -> PyResult<()> {
        self.session
            .validate_reaction_authoring_choices_v1(&choices.value)
            .map_err(|error| authoring_error(py, error))
    }
    /// Create one reaction only through the renderer-preflighted native bridge.
    // This positional PyO3 API preserves the established public reaction protocol.
    #[allow(clippy::too_many_arguments)]
    fn create_reaction_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        reactants: Vec<String>,
        products: Vec<String>,
        arrow: String,
        conditions: Vec<String>,
        pluses: Vec<String>,
    ) -> PyResult<PyReactionCreateCommitV1> {
        let request = ReactionCreateRequestV1::new(
            expected_revision,
            reactants,
            products,
            arrow,
            conditions,
            pluses,
        )
        .map_err(|error| gesture_error(py, error))?;
        let snapshot = self
            .session
            .snapshot()
            .map_err(|error| ReactionGestureError::new_err(error.to_string()))?;
        let fence = DocumentFenceV1::new(expected_revision, *snapshot.digest());
        let gesture = begin_api_reaction_gesture_v1(&self.session, fence, request)
            .map_err(|error| gesture_error(py, error))?;
        let mut prepared = prepare_api_reaction_gesture_v1(&mut self.session, &gesture)
            .map_err(|error| gesture_error(py, error))?;
        commit_api_reaction_gesture_v1(&mut self.session, &mut prepared)
            .map(|commit| PyReactionCreateCommitV1 {
                reaction_id: commit.reaction_id().to_owned(),
                result: commit.result().clone().into(),
            })
            .map_err(|error| gesture_error(py, error))
    }
}
