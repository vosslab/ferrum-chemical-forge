//! Opaque PyO3 seam for Rust-owned multi-point path authoring.

use crate::{
    ApiPresentationPathGestureV1, ApiPresentationPathPreparedV1, ApiPresentationPathPreviewV1,
    PresentationPathRenderCategoryV1, PresentationPathRenderErrorV1,
    PresentationPathRenderRecoveryV1, begin_api_presentation_path_gesture_v1,
    commit_api_presentation_path_gesture_v1, prepare_api_presentation_path_gesture_v1,
    preview_api_presentation_path_gesture_v1,
};
use ferrum_document::{DocumentFenceV1, PresentationGesturePoint2V1, PresentationPathKindV1, PresentationRecordKindV1};
use pyo3::{create_exception, prelude::*};

use super::binding::{PyDocumentSession, PySessionOperationResultV1};
use super::presentation_creation_gesture_binding::digest;

create_exception!(ferrum_chem, PresentationPathGestureError, super::binding::DocumentError);

#[pyclass(frozen, eq, hash, module = "ferrum_chem", name = "PresentationPathKindV1", rename_all = "snake_case", skip_from_py_object)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyPresentationPathKindV1 { Polyline, Polygon }
impl From<PyPresentationPathKindV1> for PresentationPathKindV1 {
    fn from(value: PyPresentationPathKindV1) -> Self { match value { PyPresentationPathKindV1::Polyline => Self::Polyline, PyPresentationPathKindV1::Polygon => Self::Polygon } }
}
#[pyclass(frozen, eq, hash, module = "ferrum_chem", name = "PresentationPathGestureCategoryV1", rename_all = "snake_case", skip_from_py_object)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyPresentationPathGestureCategoryV1 { StaleSnapshot, ForeignSession, MismatchedPreview, ReplayedGesture, InvalidGeometry, RenderPreparation, SessionConflict }
#[pyclass(frozen, eq, hash, module = "ferrum_chem", name = "PresentationPathGestureRecoveryV1", rename_all = "snake_case", skip_from_py_object)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyPresentationPathGestureRecoveryV1 { RefreshAndRestart, ChangeGeometry, DocumentUnchanged }
#[pyclass(unsendable, module = "ferrum_chem", name = "PresentationPathGestureV1")]
pub(crate) struct PyPresentationPathGestureV1 { gesture: ApiPresentationPathGestureV1 }
#[pyclass(frozen, module = "ferrum_chem", name = "PresentationPathOverlayV1")]
pub(crate) struct PyPresentationPathOverlayV1 {
    #[pyo3(get)] kind: Py<PyPresentationPathKindV1>,
    #[pyo3(get)] points: Vec<(f64, f64)>,
    #[pyo3(get)] closed: bool,
    #[pyo3(get)] stroke_color: String,
    #[pyo3(get)] stroke_width: f64,
    #[pyo3(get)] fill_color: Option<String>,
}
#[pyclass(unsendable, module = "ferrum_chem", name = "PresentationPathPreviewV1")]
pub(crate) struct PyPresentationPathPreviewV1 { preview: ApiPresentationPathPreviewV1, #[pyo3(get)] overlay: Py<PyPresentationPathOverlayV1> }
#[pyclass(unsendable, module = "ferrum_chem", name = "PresentationPathPreparedV1")]
pub(crate) struct PyPresentationPathPreparedV1 { prepared: ApiPresentationPathPreparedV1 }
#[pyclass(frozen, module = "ferrum_chem", name = "PresentationPathCommitV1")]
pub(crate) struct PyPresentationPathCommitV1 { #[pyo3(get)] identifier: String, #[pyo3(get)] kind: Py<PyPresentationPathKindV1>, #[pyo3(get)] result: PySessionOperationResultV1 }

#[pymethods]
impl PyDocumentSession {
    fn begin_presentation_path_gesture_v1(&self, py: Python<'_>, expected_revision: u64, expected_digest_hex: String, kind: PyRef<'_, PyPresentationPathKindV1>) -> PyResult<PyPresentationPathGestureV1> {
        begin_api_presentation_path_gesture_v1(&self.session, DocumentFenceV1::new(expected_revision, digest(&expected_digest_hex)?), (*kind).into()).map(|gesture| PyPresentationPathGestureV1 { gesture }).map_err(|error| path_error(py, error))
    }
    fn preview_presentation_path_gesture_v1(&self, py: Python<'_>, gesture: PyRef<'_, PyPresentationPathGestureV1>, points: Vec<(f64, f64)>) -> PyResult<PyPresentationPathPreviewV1> {
        let points = points.into_iter().map(|(x, y)| PresentationGesturePoint2V1::new(x, y).map_err(|_| PresentationPathRenderErrorV1::InvalidGeometry(ferrum_document::PresentationPathGestureErrorV1::DegenerateGeometry))).collect::<Result<Vec<_>, _>>().map_err(|error| path_error(py, error))?;
        preview_api_presentation_path_gesture_v1(&self.session, &gesture.gesture, points).map(|preview| preview_to_python(py, preview)).map_err(|error| path_error(py, error))
    }
    fn prepare_presentation_path_gesture_v1(&mut self, py: Python<'_>, gesture: PyRef<'_, PyPresentationPathGestureV1>, preview: PyRef<'_, PyPresentationPathPreviewV1>) -> PyResult<PyPresentationPathPreparedV1> {
        prepare_api_presentation_path_gesture_v1(&mut self.session, &gesture.gesture, &preview.preview).map(|prepared| PyPresentationPathPreparedV1 { prepared }).map_err(|error| path_error(py, error))
    }
    fn commit_presentation_path_gesture_v1(&mut self, py: Python<'_>, mut prepared: PyRefMut<'_, PyPresentationPathPreparedV1>) -> PyResult<PyPresentationPathCommitV1> {
        commit_api_presentation_path_gesture_v1(&mut self.session, &mut prepared.prepared).map(|commit| {
            let kind = match commit.root().kind() { PresentationRecordKindV1::Polyline => PyPresentationPathKindV1::Polyline, PresentationRecordKindV1::Polygon => PyPresentationPathKindV1::Polygon, _ => unreachable!("path commits are closed") };
            PyPresentationPathCommitV1 { identifier: commit.root().presentation_id().as_str().to_owned(), kind: Py::new(py, kind).expect("kind allocates"), result: commit.result().clone().into() }
        }).map_err(|error| path_error(py, error))
    }
}

fn preview_to_python(py: Python<'_>, preview: ApiPresentationPathPreviewV1) -> PyPresentationPathPreviewV1 {
    let path = preview.path();
    let kind = match path.kind() { PresentationPathKindV1::Polyline => PyPresentationPathKindV1::Polyline, PresentationPathKindV1::Polygon => PyPresentationPathKindV1::Polygon };
    let appearance = preview.appearance();
    let overlay = PyPresentationPathOverlayV1 { kind: Py::new(py, kind).expect("kind allocates"), points: path.points().iter().map(|point| (point.x(), point.y())).collect(), closed: path.kind() == PresentationPathKindV1::Polygon, stroke_color: appearance.stroke_color().to_owned(), stroke_width: appearance.stroke_width(), fill_color: appearance.fill_color().map(str::to_owned) };
    PyPresentationPathPreviewV1 { preview, overlay: Py::new(py, overlay).expect("overlay allocates") }
}
fn path_error(py: Python<'_>, error: PresentationPathRenderErrorV1) -> PyErr {
    let category = match error.category() {
        PresentationPathRenderCategoryV1::StaleSnapshot => PyPresentationPathGestureCategoryV1::StaleSnapshot,
        PresentationPathRenderCategoryV1::ForeignSession => PyPresentationPathGestureCategoryV1::ForeignSession,
        PresentationPathRenderCategoryV1::MismatchedPreview => PyPresentationPathGestureCategoryV1::MismatchedPreview,
        PresentationPathRenderCategoryV1::ReplayedGesture => PyPresentationPathGestureCategoryV1::ReplayedGesture,
        PresentationPathRenderCategoryV1::InvalidGeometry => PyPresentationPathGestureCategoryV1::InvalidGeometry,
        PresentationPathRenderCategoryV1::RenderPreparation => PyPresentationPathGestureCategoryV1::RenderPreparation,
        PresentationPathRenderCategoryV1::SessionConflict => PyPresentationPathGestureCategoryV1::SessionConflict,
    };
    let recovery = match error.recovery() {
        PresentationPathRenderRecoveryV1::RefreshAndRestart => PyPresentationPathGestureRecoveryV1::RefreshAndRestart,
        PresentationPathRenderRecoveryV1::ChangeGeometry => PyPresentationPathGestureRecoveryV1::ChangeGeometry,
        PresentationPathRenderRecoveryV1::DocumentUnchanged => PyPresentationPathGestureRecoveryV1::DocumentUnchanged,
    };
    let exception = PresentationPathGestureError::new_err(error.to_string());
    let value = exception.value(py);
    value.setattr("category", Py::new(py, category).expect("category allocates")).expect("category attaches");
    value.setattr("recovery", Py::new(py, recovery).expect("recovery allocates")).expect("recovery attaches");
    exception
}
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("PresentationPathGestureError", module.py().get_type::<PresentationPathGestureError>())?;
    module.add_class::<PyPresentationPathKindV1>()?;
    module.add_class::<PyPresentationPathGestureCategoryV1>()?;
    module.add_class::<PyPresentationPathGestureRecoveryV1>()?;
    module.add_class::<PyPresentationPathGestureV1>()?;
    module.add_class::<PyPresentationPathOverlayV1>()?;
    module.add_class::<PyPresentationPathPreviewV1>()?;
    module.add_class::<PyPresentationPathPreparedV1>()?;
    module.add_class::<PyPresentationPathCommitV1>()
}
