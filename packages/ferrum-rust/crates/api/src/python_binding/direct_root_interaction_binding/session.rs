//! Session-owned direct-root interaction methods exposed through PyO3.

use super::{
    super::binding::PyDocumentSession,
    conversion::{
        commit, fence, observation, preview, selection, structure_commit, structure_observation,
        structure_selection,
    },
    dto::*,
    error::{RenderInteractionError, interaction_error},
    query::*,
};
use pyo3::prelude::*;
#[pymethods]
impl PyDocumentSession {
    fn observe_render_interaction_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
    ) -> PyResult<PyObservation> {
        self.session
            .observe_render_interaction_v1(fence(&expected_digest_hex, expected_revision)?)
            .map_err(|error| interaction_error(py, error))
            .and_then(|value| observation(py, value))
    }
    fn select_render_interaction_roots_v1(
        &self,
        py: Python<'_>,
        observation: PyRef<'_, PyObservation>,
        previous: Option<PyRef<'_, PySelection>>,
        query: PyRef<'_, PyQuery>,
    ) -> PyResult<PySelection> {
        self.session
            .select_render_interaction_roots_v1(
                &observation.value,
                previous.as_ref().map(|value| &value.value),
                query.query.clone(),
            )
            .map_err(|error| interaction_error(py, error))
            .and_then(|value| selection(py, value))
    }
    fn render_interaction_selection_contains_point_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PySelection>,
        x: f64,
        y: f64,
    ) -> PyResult<bool> {
        self.session
            .render_interaction_selection_contains_point_v1(&selection.value, x, y)
            .map_err(|error| interaction_error(py, error))
    }
    fn begin_render_interaction_translation_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PySelection>,
        press_x: f64,
        press_y: f64,
        snap: PyRef<'_, PySnap>,
    ) -> PyResult<PyGesture> {
        self.session
            .begin_render_interaction_translation_v1(&selection.value, press_x, press_y, snap.snap)
            .map(|value| PyGesture { value: Some(value) })
            .map_err(|error| interaction_error(py, error))
    }
    fn preview_render_interaction_translation_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyGesture>,
        pointer_x: f64,
        pointer_y: f64,
    ) -> PyResult<PyPreview> {
        let gesture = gesture.value.as_ref().ok_or_else(|| {
            RenderInteractionError::new_err("translation gesture was already prepared")
        })?;
        self.session
            .preview_render_interaction_translation_v1(gesture, pointer_x, pointer_y)
            .map_err(|error| interaction_error(py, error))
            .and_then(|value| preview(py, value))
    }
    fn commit_render_interaction_translation_v1(
        &mut self,
        py: Python<'_>,
        mut gesture: PyRefMut<'_, PyGesture>,
        release_x: f64,
        release_y: f64,
    ) -> PyResult<PyCommit> {
        let gesture = gesture.value.take().ok_or_else(|| {
            RenderInteractionError::new_err("translation gesture was already prepared")
        })?;
        self.session
            .commit_render_interaction_translation_v1(gesture, release_x, release_y)
            .map_err(|error| interaction_error(py, error))
            .and_then(|value| commit(py, value))
    }
    fn observe_structure_interaction_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
    ) -> PyResult<PyStructureObservation> {
        self.session
            .observe_structure_interaction_v1(fence(&expected_digest_hex, expected_revision)?)
            .map_err(|error| interaction_error(py, error))
            .and_then(|value| structure_observation(py, value))
    }
    fn select_structure_interaction_v1(
        &self,
        py: Python<'_>,
        observation: PyRef<'_, PyStructureObservation>,
        previous: Option<PyRef<'_, PyStructureSelection>>,
        query: PyRef<'_, PyStructureQuery>,
    ) -> PyResult<PyStructureSelection> {
        self.session
            .select_structure_interaction_v1(
                &observation.value,
                previous.as_ref().map(|value| &value.value),
                query.query.clone(),
            )
            .map_err(|error| interaction_error(py, error))
            .and_then(|value| structure_selection(py, value))
    }
    fn commit_structure_deletion_v1(
        &mut self,
        py: Python<'_>,
        selection: PyRef<'_, PyStructureSelection>,
    ) -> PyResult<PyStructureCommit> {
        self.session
            .commit_structure_deletion_v1(&selection.value)
            .map_err(|error| interaction_error(py, error))
            .map(structure_commit)
    }
}
