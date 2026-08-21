//! Dedicated PyO3 facade for renderer-backed standard Plus placement.

use super::binding::{PyDocumentSession, PySessionOperationResultV1};
use super::presentation_creation_gesture_binding::{digest, presentation_error};
use crate::{
    ApiPlusGestureV1, ApiPlusPreviewV1, begin_api_plus_gesture_v1, commit_api_plus_gesture_v1,
    preview_api_plus_gesture_v1,
};
use ferrum_document::{DocumentFenceV1, PresentationGesturePoint2V1};
use pyo3::prelude::*;

#[pyclass(unsendable, module = "ferrum_chem", name = "PlusPlacementGestureV1")]
pub(crate) struct PyPlusPlacementGestureV1 {
    gesture: ApiPlusGestureV1,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PlusPlacementOverlayV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyPlusPlacementOverlayV1 {
    #[pyo3(get)]
    origin_x: f64,
    #[pyo3(get)]
    origin_y: f64,
    #[pyo3(get)]
    left: f64,
    #[pyo3(get)]
    top: f64,
    #[pyo3(get)]
    right: f64,
    #[pyo3(get)]
    bottom: f64,
    #[pyo3(get)]
    text: String,
    #[pyo3(get)]
    font_size: f64,
    #[pyo3(get)]
    color: String,
    #[pyo3(get)]
    background: Option<String>,
}
#[pyclass(unsendable, module = "ferrum_chem", name = "PlusPlacementPreviewV1")]
pub(crate) struct PyPlusPlacementPreviewV1 {
    preview: ApiPlusPreviewV1,
    #[pyo3(get)]
    overlay: PyPlusPlacementOverlayV1,
}
#[pyclass(frozen, module = "ferrum_chem", name = "PlusPlacementCommitV1")]
pub(crate) struct PyPlusPlacementCommitV1 {
    #[pyo3(get)]
    identifier: String,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}

#[pymethods]
impl PyDocumentSession {
    fn begin_plus_placement_gesture_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        x: f64,
        y: f64,
    ) -> PyResult<PyPlusPlacementGestureV1> {
        let fence = DocumentFenceV1::new(expected_revision, digest(&expected_digest_hex)?);
        let start = PresentationGesturePoint2V1::new(x, y)
            .map_err(|error| presentation_error(py, error))?;
        begin_api_plus_gesture_v1(&self.session, fence, start)
            .map(|gesture| PyPlusPlacementGestureV1 { gesture })
            .map_err(|error| presentation_error(py, error))
    }
    fn preview_plus_placement_gesture_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyPlusPlacementGestureV1>,
    ) -> PyResult<PyPlusPlacementPreviewV1> {
        preview_api_plus_gesture_v1(&self.session, &gesture.gesture)
            .map(preview)
            .map_err(|error| presentation_error(py, error))
    }
    fn commit_plus_placement_gesture_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyPlusPlacementGestureV1>,
        preview: PyRef<'_, PyPlusPlacementPreviewV1>,
    ) -> PyResult<PyPlusPlacementCommitV1> {
        commit_api_plus_gesture_v1(&mut self.session, &gesture.gesture, &preview.preview)
            .map(|commit| PyPlusPlacementCommitV1 {
                identifier: commit.root().presentation_id().as_str().to_owned(),
                result: commit.result().clone().into(),
            })
            .map_err(|error| presentation_error(py, error))
    }
}
fn preview(value: ApiPlusPreviewV1) -> PyPlusPlacementPreviewV1 {
    let overlay = value.overlay();
    PyPlusPlacementPreviewV1 {
        overlay: PyPlusPlacementOverlayV1 {
            origin_x: overlay.origin_x(),
            origin_y: overlay.origin_y(),
            left: overlay.left(),
            top: overlay.top(),
            right: overlay.right(),
            bottom: overlay.bottom(),
            text: overlay.text().to_owned(),
            font_size: overlay.font_size(),
            color: overlay.color().to_owned(),
            background: overlay.background().map(str::to_owned),
        },
        preview: value,
    }
}
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyPlusPlacementGestureV1>()?;
    module.add_class::<PyPlusPlacementOverlayV1>()?;
    module.add_class::<PyPlusPlacementPreviewV1>()?;
    module.add_class::<PyPlusPlacementCommitV1>()
}
