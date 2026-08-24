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
            .map(|value| PyReactionLifecycleGestureV1 { value: Some(value) })
            .map_err(|error| gesture_error(py, error))
    }
    /// Begin removal of only one selected strict reaction definition.
    fn begin_reaction_definition_delete_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PyReactionSelectionV1>,
    ) -> PyResult<PyReactionLifecycleGestureV1> {
        begin_api_reaction_definition_delete_v1(&self.session, &selection.value)
            .map(|value| PyReactionLifecycleGestureV1 { value: Some(value) })
            .map_err(|error| gesture_error(py, error))
    }
    /// Resolve one lifecycle gesture into an opaque generic transition request.
    fn resolve_reaction_lifecycle_v1(
        &self,
        py: Python<'_>,
        mut gesture: PyRefMut<'_, PyReactionLifecycleGestureV1>,
    ) -> PyResult<PySessionOperationTransitionRequestV1> {
        let gesture = gesture
            .value
            .take()
            .ok_or_else(|| gesture_error(py, ReactionGestureErrorV1::ReplayedGesture))?;
        resolve_api_reaction_lifecycle_v1(&self.session, gesture)
            .map(PySessionOperationTransitionRequestV1::from_request)
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
            .map(|value| PyReactionTranslationGestureV1 { value: Some(value) })
            .map_err(|error| gesture_error(py, error))
    }
    /// Resolve one translation gesture into an opaque generic transition request.
    fn resolve_reaction_translation_v1(
        &self,
        py: Python<'_>,
        mut gesture: PyRefMut<'_, PyReactionTranslationGestureV1>,
        pointer_x: f64,
        pointer_y: f64,
    ) -> PyResult<PySessionOperationTransitionRequestV1> {
        let gesture = gesture
            .value
            .take()
            .ok_or_else(|| gesture_error(py, ReactionGestureErrorV1::ReplayedGesture))?;
        resolve_api_reaction_translation_v1(&self.session, gesture, pointer_x, pointer_y)
            .map(PySessionOperationTransitionRequestV1::from_request)
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
    /// Begin one semantic reaction gesture from a fenced source snapshot.
    #[allow(clippy::too_many_arguments)]
    fn begin_reaction_gesture_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        reactants: Vec<String>,
        products: Vec<String>,
        arrow: String,
        conditions: Vec<String>,
        pluses: Vec<String>,
    ) -> PyResult<PyReactionGestureV1> {
        let request = ReactionCreateRequestV1::new(
            expected_revision,
            reactants,
            products,
            arrow,
            conditions,
            pluses,
        )
        .map_err(|error| gesture_error(py, error))?;
        let fence = authoring_fence(&expected_digest_hex, expected_revision)?;
        begin_api_reaction_gesture_v1(&self.session, fence, request)
            .map(|value| PyReactionGestureV1 { value: Some(value) })
            .map_err(|error| gesture_error(py, error))
    }
    /// Resolve one create gesture into an opaque generic transition request.
    fn resolve_reaction_gesture_v1(
        &self,
        py: Python<'_>,
        mut gesture: PyRefMut<'_, PyReactionGestureV1>,
    ) -> PyResult<PySessionOperationTransitionRequestV1> {
        let gesture = gesture
            .value
            .take()
            .ok_or_else(|| gesture_error(py, ReactionGestureErrorV1::ReplayedGesture))?;
        resolve_api_reaction_gesture_v1(&self.session, gesture)
            .map(PySessionOperationTransitionRequestV1::from_request)
            .map_err(|error| gesture_error(py, error))
    }
}
