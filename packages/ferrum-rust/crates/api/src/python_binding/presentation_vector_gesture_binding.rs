//! Opaque PyO3 seam for Rust-owned ordinary two-point vector authoring.

use crate::{
    ApiPresentationVectorGestureV1, ApiPresentationVectorPreviewV1,
    PresentationVectorGestureCategoryV1, PresentationVectorGestureErrorV1,
    PresentationVectorGestureRecoveryV1, PresentationVectorKindV1, PresentationVectorOverlayV1,
    begin_api_presentation_vector_gesture_v1, preview_api_presentation_vector_gesture_v1,
    resolve_api_presentation_vector_gesture_v1,
};
use ferrum_document::{DocumentFenceV1, PresentationGesturePoint2V1};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::binding::PyDocumentSession;
use super::presentation_creation_gesture_binding::digest;
use super::render_binding::{PyRenderPaintV3, paint_from};

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
    Consumed,
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
    gesture: Option<ApiPresentationVectorGestureV1>,
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
    pub stroke_paint: PyRenderPaintV3,
    #[pyo3(get)]
    pub stroke_width: f64,
    #[pyo3(get)]
    pub fill_paint: Option<PyRenderPaintV3>,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PresentationVectorPreviewV1"
)]
pub(crate) struct PyPresentationVectorPreviewV1 {
    preview: Option<ApiPresentationVectorPreviewV1>,
    #[pyo3(get)]
    overlay: Py<PyPresentationVectorOverlayV1>,
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
            .map(|gesture| PyPresentationVectorGestureV1 {
                gesture: Some(gesture),
            })
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
        let gesture = gesture
            .gesture
            .as_ref()
            .ok_or_else(|| vector_error(py, PresentationVectorGestureErrorV1::Consumed))?;
        preview_api_presentation_vector_gesture_v1(&self.session, gesture, point)
            .map(|preview| preview_to_python(py, preview))
            .map_err(|error| vector_error(py, error))
    }

    fn resolve_presentation_vector_gesture_v1(
        &self,
        py: Python<'_>,
        mut gesture: PyRefMut<'_, PyPresentationVectorGestureV1>,
        mut preview: PyRefMut<'_, PyPresentationVectorPreviewV1>,
    ) -> PyResult<super::prepared_transition_binding::PySessionOperationTransitionRequestV1> {
        let gesture = gesture
            .gesture
            .take()
            .ok_or_else(|| vector_error(py, PresentationVectorGestureErrorV1::Consumed))?;
        let preview = preview
            .preview
            .take()
            .ok_or_else(|| vector_error(py, PresentationVectorGestureErrorV1::Consumed))?;
        resolve_api_presentation_vector_gesture_v1(&self.session, gesture, preview)
            .map(super::prepared_transition_binding::PySessionOperationTransitionRequestV1::from_request)
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
    let (stroke_paint, stroke_width, fill_paint) = {
        let appearance = preview.overlay().appearance();
        (
            paint_from(appearance.stroke_paint()),
            appearance.stroke_width(),
            appearance.fill_paint().map(paint_from),
        )
    };
    PyPresentationVectorPreviewV1 {
        preview: Some(preview),
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
                stroke_paint,
                stroke_width,
                fill_paint,
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
        PresentationVectorGestureCategoryV1::Consumed => {
            PyPresentationVectorGestureCategoryV1::Consumed
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
    Ok(())
}
