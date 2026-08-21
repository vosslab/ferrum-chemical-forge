//! Opaque PyO3 seam for Rust-owned ordinary two-point vector authoring.

use crate::{
    begin_api_presentation_vector_gesture_v1, commit_api_presentation_vector_gesture_v1,
    prepare_api_presentation_vector_gesture_v1, preview_api_presentation_vector_gesture_v1,
    ApiPresentationVectorGestureV1, ApiPresentationVectorPreparedV1,
    ApiPresentationVectorPreviewV1, PresentationVectorGestureCategoryV1,
    PresentationVectorGestureErrorV1, PresentationVectorGestureRecoveryV1,
    PresentationVectorKindV1, PresentationVectorOverlayV1,
};
use ferrum_document::{DocumentFenceV1, PresentationGesturePoint2V1, PresentationRecordKindV1};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::binding::{PyDocumentSession, PySessionOperationResultV1};
use super::presentation_creation_gesture_binding::digest;

create_exception!(
    ferrum_chem,
    PresentationVectorGestureError,
    super::binding::DocumentError
);

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "PresentationVectorKindV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyPresentationVectorKindV1 {
    Line,
    Rectangle,
    Square,
    Oval,
    Circle,
}

impl From<PyPresentationVectorKindV1> for PresentationVectorKindV1 {
    fn from(value: PyPresentationVectorKindV1) -> Self {
        match value {
            PyPresentationVectorKindV1::Line => Self::Line,
            PyPresentationVectorKindV1::Rectangle => Self::Rectangle,
            PyPresentationVectorKindV1::Square => Self::Square,
            PyPresentationVectorKindV1::Oval => Self::Oval,
            PyPresentationVectorKindV1::Circle => Self::Circle,
        }
    }
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "PresentationVectorGestureCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyPresentationVectorGestureCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    ReplayedGesture,
    InvalidPoint,
    DegenerateGeometry,
    UnsupportedKind,
    UnrenderableStandard,
    RenderPreparation,
    SessionConflict,
    ResourceExhausted,
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "PresentationVectorGestureRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyPresentationVectorGestureRecoveryV1 {
    DocumentUnchanged,
    RefreshAndRestart,
    ChangeGeometry,
    ChooseSupportedAppearance,
    ReduceRequest,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PresentationVectorGestureV1"
)]
pub(crate) struct PyPresentationVectorGestureV1 {
    gesture: ApiPresentationVectorGestureV1,
}

#[pyclass(frozen, module = "ferrum_chem", name = "PresentationVectorOverlayV1")]
pub(crate) struct PyPresentationVectorOverlayV1 {
    #[pyo3(get)]
    kind: Py<PyPresentationVectorKindV1>,
    #[pyo3(get)]
    pub start_x: f64,
    #[pyo3(get)]
    pub start_y: f64,
    #[pyo3(get)]
    pub end_x: f64,
    #[pyo3(get)]
    pub end_y: f64,
    #[pyo3(get)]
    pub left: f64,
    #[pyo3(get)]
    pub top: f64,
    #[pyo3(get)]
    pub right: f64,
    #[pyo3(get)]
    pub bottom: f64,
    #[pyo3(get)]
    pub stroke_color: String,
    #[pyo3(get)]
    pub stroke_width: f64,
    #[pyo3(get)]
    pub fill_color: Option<String>,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PresentationVectorPreviewV1"
)]
pub(crate) struct PyPresentationVectorPreviewV1 {
    preview: ApiPresentationVectorPreviewV1,
    #[pyo3(get)]
    overlay: Py<PyPresentationVectorOverlayV1>,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PresentationVectorPreparedV1"
)]
pub(crate) struct PyPresentationVectorPreparedV1 {
    prepared: ApiPresentationVectorPreparedV1,
}

#[pyclass(frozen, module = "ferrum_chem", name = "PresentationVectorCommitV1")]
pub(crate) struct PyPresentationVectorCommitV1 {
    #[pyo3(get)]
    identifier: String,
    #[pyo3(get)]
    kind: Py<PyPresentationVectorKindV1>,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}

#[pymethods]
impl PyDocumentSession {
    fn begin_presentation_vector_gesture_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        kind: PyRef<'_, PyPresentationVectorKindV1>,
        start_x: f64,
        start_y: f64,
    ) -> PyResult<PyPresentationVectorGestureV1> {
        let fence = DocumentFenceV1::new(expected_revision, digest(&expected_digest_hex)?);
        let point = PresentationGesturePoint2V1::new(start_x, start_y)
            .map_err(|_| vector_error(py, PresentationVectorGestureErrorV1::InvalidPoint))?;
        begin_api_presentation_vector_gesture_v1(&self.session, fence, (*kind).into(), point)
            .map(|gesture| PyPresentationVectorGestureV1 { gesture })
            .map_err(|error| vector_error(py, error))
    }

    fn preview_presentation_vector_gesture_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyPresentationVectorGestureV1>,
        end_x: f64,
        end_y: f64,
    ) -> PyResult<PyPresentationVectorPreviewV1> {
        let point = PresentationGesturePoint2V1::new(end_x, end_y)
            .map_err(|_| vector_error(py, PresentationVectorGestureErrorV1::InvalidPoint))?;
        preview_api_presentation_vector_gesture_v1(&self.session, &gesture.gesture, point)
            .map(|preview| preview_to_python(py, preview))
            .map_err(|error| vector_error(py, error))
    }

    fn prepare_presentation_vector_gesture_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyPresentationVectorGestureV1>,
        preview: PyRef<'_, PyPresentationVectorPreviewV1>,
    ) -> PyResult<PyPresentationVectorPreparedV1> {
        prepare_api_presentation_vector_gesture_v1(
            &mut self.session,
            &gesture.gesture,
            &preview.preview,
        )
        .map(|prepared| PyPresentationVectorPreparedV1 { prepared })
        .map_err(|error| vector_error(py, error))
    }

    fn commit_presentation_vector_gesture_v1(
        &mut self,
        py: Python<'_>,
        mut prepared: PyRefMut<'_, PyPresentationVectorPreparedV1>,
    ) -> PyResult<PyPresentationVectorCommitV1> {
        commit_api_presentation_vector_gesture_v1(&mut self.session, &mut prepared.prepared)
            .map(|commit| {
                let kind = match commit.root().kind() {
                    PresentationRecordKindV1::Polyline => PyPresentationVectorKindV1::Line,
                    PresentationRecordKindV1::Rectangle => PyPresentationVectorKindV1::Rectangle,
                    PresentationRecordKindV1::Square => PyPresentationVectorKindV1::Square,
                    PresentationRecordKindV1::Oval => PyPresentationVectorKindV1::Oval,
                    PresentationRecordKindV1::Circle => PyPresentationVectorKindV1::Circle,
                    _ => unreachable!("vector commit only emits a vector root"),
                };
                PyPresentationVectorCommitV1 {
                    identifier: commit.root().presentation_id().as_str().to_owned(),
                    kind: Py::new(py, kind).expect("kind allocates"),
                    result: commit.result().clone().into(),
                }
            })
            .map_err(|error| vector_error(py, error))
    }
}

fn preview_to_python(
    py: Python<'_>,
    preview: ApiPresentationVectorPreviewV1,
) -> PyPresentationVectorPreviewV1 {
    let (kind, start_x, start_y, end_x, end_y, left, top, right, bottom) = match preview.overlay() {
        PresentationVectorOverlayV1::Line { start, end, .. } => (
            PyPresentationVectorKindV1::Line,
            start.x(),
            start.y(),
            end.x(),
            end.y(),
            start.x().min(end.x()),
            start.y().min(end.y()),
            start.x().max(end.x()),
            start.y().max(end.y()),
        ),
        PresentationVectorOverlayV1::Box {
            kind,
            left,
            top,
            right,
            bottom,
            ..
        } => (
            kind_to_python(*kind),
            *left,
            *top,
            *right,
            *bottom,
            *left,
            *top,
            *right,
            *bottom,
        ),
    };
    let (stroke_color, stroke_width, fill_color) = {
        let appearance = preview.overlay().appearance();
        (
            appearance.stroke_color().to_owned(),
            appearance.stroke_width(),
            appearance.fill_color().map(str::to_owned),
        )
    };
    PyPresentationVectorPreviewV1 {
        preview,
        overlay: Py::new(
            py,
            PyPresentationVectorOverlayV1 {
                kind: Py::new(py, kind).expect("kind allocates"),
                start_x,
                start_y,
                end_x,
                end_y,
                left,
                top,
                right,
                bottom,
                stroke_color,
                stroke_width,
                fill_color,
            },
        )
        .expect("overlay allocates"),
    }
}

fn kind_to_python(kind: PresentationVectorKindV1) -> PyPresentationVectorKindV1 {
    match kind {
        PresentationVectorKindV1::Line => PyPresentationVectorKindV1::Line,
        PresentationVectorKindV1::Rectangle => PyPresentationVectorKindV1::Rectangle,
        PresentationVectorKindV1::Square => PyPresentationVectorKindV1::Square,
        PresentationVectorKindV1::Oval => PyPresentationVectorKindV1::Oval,
        PresentationVectorKindV1::Circle => PyPresentationVectorKindV1::Circle,
        _ => unreachable!("a new vector kind requires an explicit frozen PyO3 member"),
    }
}

fn vector_error(py: Python<'_>, error: PresentationVectorGestureErrorV1) -> PyErr {
    let category = match error.category() {
        PresentationVectorGestureCategoryV1::StaleSnapshot => {
            PyPresentationVectorGestureCategoryV1::StaleSnapshot
        }
        PresentationVectorGestureCategoryV1::ForeignSession => {
            PyPresentationVectorGestureCategoryV1::ForeignSession
        }
        PresentationVectorGestureCategoryV1::MismatchedPreview => {
            PyPresentationVectorGestureCategoryV1::MismatchedPreview
        }
        PresentationVectorGestureCategoryV1::ReplayedGesture => {
            PyPresentationVectorGestureCategoryV1::ReplayedGesture
        }
        PresentationVectorGestureCategoryV1::InvalidPoint => {
            PyPresentationVectorGestureCategoryV1::InvalidPoint
        }
        PresentationVectorGestureCategoryV1::DegenerateGeometry => {
            PyPresentationVectorGestureCategoryV1::DegenerateGeometry
        }
        PresentationVectorGestureCategoryV1::UnsupportedKind => {
            PyPresentationVectorGestureCategoryV1::UnsupportedKind
        }
        PresentationVectorGestureCategoryV1::UnrenderableStandard => {
            PyPresentationVectorGestureCategoryV1::UnrenderableStandard
        }
        PresentationVectorGestureCategoryV1::RenderPreparation => {
            PyPresentationVectorGestureCategoryV1::RenderPreparation
        }
        PresentationVectorGestureCategoryV1::SessionConflict => {
            PyPresentationVectorGestureCategoryV1::SessionConflict
        }
        PresentationVectorGestureCategoryV1::ResourceExhausted => {
            PyPresentationVectorGestureCategoryV1::ResourceExhausted
        }
        _ => unreachable!("a new vector category requires an explicit frozen PyO3 member"),
    };
    let recovery = match error.recovery() {
        PresentationVectorGestureRecoveryV1::DocumentUnchanged => {
            PyPresentationVectorGestureRecoveryV1::DocumentUnchanged
        }
        PresentationVectorGestureRecoveryV1::RefreshAndRestart => {
            PyPresentationVectorGestureRecoveryV1::RefreshAndRestart
        }
        PresentationVectorGestureRecoveryV1::ChangeGeometry => {
            PyPresentationVectorGestureRecoveryV1::ChangeGeometry
        }
        PresentationVectorGestureRecoveryV1::ChooseSupportedAppearance => {
            PyPresentationVectorGestureRecoveryV1::ChooseSupportedAppearance
        }
        PresentationVectorGestureRecoveryV1::ReduceRequest => {
            PyPresentationVectorGestureRecoveryV1::ReduceRequest
        }
        _ => unreachable!("a new vector recovery requires an explicit frozen PyO3 member"),
    };
    let exception = PresentationVectorGestureError::new_err(error.to_string());
    let value = exception.value(py);
    value
        .setattr(
            "category",
            Py::new(py, category).expect("category allocates"),
        )
        .expect("category attaches");
    value
        .setattr(
            "recovery",
            Py::new(py, recovery).expect("recovery allocates"),
        )
        .expect("recovery attaches");
    exception
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "PresentationVectorGestureError",
        module.py().get_type::<PresentationVectorGestureError>(),
    )?;
    module.add_class::<PyPresentationVectorKindV1>()?;
    module.add_class::<PyPresentationVectorGestureCategoryV1>()?;
    module.add_class::<PyPresentationVectorGestureRecoveryV1>()?;
    module.add_class::<PyPresentationVectorGestureV1>()?;
    module.add_class::<PyPresentationVectorOverlayV1>()?;
    module.add_class::<PyPresentationVectorPreviewV1>()?;
    module.add_class::<PyPresentationVectorPreparedV1>()?;
    module.add_class::<PyPresentationVectorCommitV1>()
}
