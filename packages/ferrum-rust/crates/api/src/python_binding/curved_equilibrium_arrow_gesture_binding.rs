//! Opaque PyO3 transport for renderer-preflighted quadratic curved equilibrium arrows.

use ferrum_document::{DocumentFenceV1, PresentationGesturePoint2V1};
use ferrum_document_render::{
    CommittedCurvedEquilibriumArrowV1, CurvedEquilibriumArrowGestureCategoryV1,
    CurvedEquilibriumArrowGestureErrorV1, CurvedEquilibriumArrowGestureRecoveryV1,
    CurvedEquilibriumArrowGestureV1, CurvedEquilibriumArrowPreviewV1,
    PreparedCurvedEquilibriumArrowV1, begin_curved_equilibrium_arrow_gesture_v1,
    commit_curved_equilibrium_arrow_gesture_v1, prepare_curved_equilibrium_arrow_gesture_v1,
    preview_curved_equilibrium_arrow_gesture_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::binding::{PyDocumentSession, PySessionOperationResultV1};
use super::presentation_creation_gesture_binding::{
    PyPresentationGestureRootKindV1, PyPresentationGestureRootSelectorV1, digest,
};

create_exception!(
    ferrum_chem,
    CurvedEquilibriumArrowGestureError,
    super::binding::DocumentError
);

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "CurvedEquilibriumArrowGestureCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyCurvedEquilibriumArrowGestureCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    ReplayedGesture,
    InvalidPoint,
    CollapsedSpan,
    ControlTooNearChord,
    ExceedsGeometryLimit,
    RenderPreparation,
    SessionConflict,
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "CurvedEquilibriumArrowGestureRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyCurvedEquilibriumArrowGestureRecoveryV1 {
    RefreshAndRestart,
    ChangeGeometry,
    DocumentUnchanged,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "CurvedEquilibriumArrowGestureV1"
)]
pub(crate) struct PyCurvedEquilibriumArrowGestureV1 {
    gesture: CurvedEquilibriumArrowGestureV1,
}

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "CurvedEquilibriumArrowOverlayV1"
)]
pub(crate) struct PyCurvedEquilibriumArrowOverlayV1 {
    #[pyo3(get)]
    pub start_x: f64,
    #[pyo3(get)]
    pub start_y: f64,
    #[pyo3(get)]
    pub control_x: f64,
    #[pyo3(get)]
    pub control_y: f64,
    #[pyo3(get)]
    pub end_x: f64,
    #[pyo3(get)]
    pub end_y: f64,
    #[pyo3(get)]
    pub lower_axis: Vec<(f64, f64)>,
    #[pyo3(get)]
    pub upper_axis: Vec<(f64, f64)>,
    #[pyo3(get)]
    pub lower_head: Vec<(f64, f64)>,
    #[pyo3(get)]
    pub upper_head: Vec<(f64, f64)>,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "CurvedEquilibriumArrowPreviewV1"
)]
pub(crate) struct PyCurvedEquilibriumArrowPreviewV1 {
    preview: CurvedEquilibriumArrowPreviewV1,
    #[pyo3(get)]
    overlay: Py<PyCurvedEquilibriumArrowOverlayV1>,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PreparedCurvedEquilibriumArrowV1"
)]
pub(crate) struct PyPreparedCurvedEquilibriumArrowV1 {
    prepared: PreparedCurvedEquilibriumArrowV1,
}

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "CurvedEquilibriumArrowCommitV1"
)]
pub(crate) struct PyCurvedEquilibriumArrowCommitV1 {
    #[pyo3(get)]
    root: Py<PyPresentationGestureRootSelectorV1>,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}

#[pymethods]
impl PyDocumentSession {
    #[allow(clippy::too_many_arguments)]
    fn begin_curved_equilibrium_arrow_gesture_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        start_x: f64,
        start_y: f64,
        control_x: f64,
        control_y: f64,
    ) -> PyResult<PyCurvedEquilibriumArrowGestureV1> {
        let fence = DocumentFenceV1::new(expected_revision, digest(&expected_digest_hex)?);
        begin_curved_equilibrium_arrow_gesture_v1(
            &self.session,
            fence,
            point(start_x, start_y, py)?,
            point(control_x, control_y, py)?,
        )
        .map(|gesture| PyCurvedEquilibriumArrowGestureV1 { gesture })
        .map_err(|error| equilibrium_error(py, error))
    }

    fn preview_curved_equilibrium_arrow_gesture_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyCurvedEquilibriumArrowGestureV1>,
        end_x: f64,
        end_y: f64,
    ) -> PyResult<PyCurvedEquilibriumArrowPreviewV1> {
        preview_curved_equilibrium_arrow_gesture_v1(
            &self.session,
            &gesture.gesture,
            point(end_x, end_y, py)?,
        )
        .map(|preview| preview_to_python(py, preview))
        .map_err(|error| equilibrium_error(py, error))
    }

    fn prepare_curved_equilibrium_arrow_gesture_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyCurvedEquilibriumArrowGestureV1>,
        preview: PyRef<'_, PyCurvedEquilibriumArrowPreviewV1>,
    ) -> PyResult<PyPreparedCurvedEquilibriumArrowV1> {
        prepare_curved_equilibrium_arrow_gesture_v1(
            &mut self.session,
            &gesture.gesture,
            &preview.preview,
        )
        .map(|prepared| PyPreparedCurvedEquilibriumArrowV1 { prepared })
        .map_err(|error| equilibrium_error(py, error))
    }

    fn commit_curved_equilibrium_arrow_gesture_v1(
        &mut self,
        py: Python<'_>,
        mut prepared: PyRefMut<'_, PyPreparedCurvedEquilibriumArrowV1>,
    ) -> PyResult<PyCurvedEquilibriumArrowCommitV1> {
        commit_curved_equilibrium_arrow_gesture_v1(&mut self.session, &mut prepared.prepared)
            .map(|value| commit_to_python(py, value))
            .map_err(|error| equilibrium_error(py, error))
    }
}

fn point(x: f64, y: f64, py: Python<'_>) -> PyResult<PresentationGesturePoint2V1> {
    PresentationGesturePoint2V1::new(x, y)
        .map_err(|_| equilibrium_error(py, CurvedEquilibriumArrowGestureErrorV1::InvalidPoint))
}

fn preview_to_python(
    py: Python<'_>,
    preview: CurvedEquilibriumArrowPreviewV1,
) -> PyCurvedEquilibriumArrowPreviewV1 {
    let overlay = preview.overlay();
    let value = PyCurvedEquilibriumArrowOverlayV1 {
        start_x: overlay.start().x(),
        start_y: overlay.start().y(),
        control_x: overlay.control().x(),
        control_y: overlay.control().y(),
        end_x: overlay.end().x(),
        end_y: overlay.end().y(),
        lower_axis: overlay
            .lower_axis()
            .iter()
            .map(|point| (point.x(), point.y()))
            .collect(),
        upper_axis: overlay
            .upper_axis()
            .iter()
            .map(|point| (point.x(), point.y()))
            .collect(),
        lower_head: overlay
            .lower_head()
            .iter()
            .map(|point| (point.x(), point.y()))
            .collect(),
        upper_head: overlay
            .upper_head()
            .iter()
            .map(|point| (point.x(), point.y()))
            .collect(),
    };
    PyCurvedEquilibriumArrowPreviewV1 {
        preview,
        overlay: Py::new(py, value).expect("overlay allocates"),
    }
}

fn commit_to_python(
    py: Python<'_>,
    value: CommittedCurvedEquilibriumArrowV1,
) -> PyCurvedEquilibriumArrowCommitV1 {
    PyCurvedEquilibriumArrowCommitV1 {
        root: Py::new(
            py,
            PyPresentationGestureRootSelectorV1 {
                identifier: value.root().presentation_id().as_str().to_owned(),
                kind: Py::new(py, PyPresentationGestureRootKindV1::Arrow)
                    .expect("root kind allocates"),
            },
        )
        .expect("root selector allocates"),
        result: value.result().clone().into(),
    }
}

fn equilibrium_error(py: Python<'_>, error: CurvedEquilibriumArrowGestureErrorV1) -> PyErr {
    let category = match error.category() {
        CurvedEquilibriumArrowGestureCategoryV1::StaleSnapshot => {
            PyCurvedEquilibriumArrowGestureCategoryV1::StaleSnapshot
        }
        CurvedEquilibriumArrowGestureCategoryV1::ForeignSession => {
            PyCurvedEquilibriumArrowGestureCategoryV1::ForeignSession
        }
        CurvedEquilibriumArrowGestureCategoryV1::MismatchedPreview => {
            PyCurvedEquilibriumArrowGestureCategoryV1::MismatchedPreview
        }
        CurvedEquilibriumArrowGestureCategoryV1::ReplayedGesture => {
            PyCurvedEquilibriumArrowGestureCategoryV1::ReplayedGesture
        }
        CurvedEquilibriumArrowGestureCategoryV1::InvalidPoint => {
            PyCurvedEquilibriumArrowGestureCategoryV1::InvalidPoint
        }
        CurvedEquilibriumArrowGestureCategoryV1::CollapsedSpan => {
            PyCurvedEquilibriumArrowGestureCategoryV1::CollapsedSpan
        }
        CurvedEquilibriumArrowGestureCategoryV1::ControlTooNearChord => {
            PyCurvedEquilibriumArrowGestureCategoryV1::ControlTooNearChord
        }
        CurvedEquilibriumArrowGestureCategoryV1::ExceedsGeometryLimit => {
            PyCurvedEquilibriumArrowGestureCategoryV1::ExceedsGeometryLimit
        }
        CurvedEquilibriumArrowGestureCategoryV1::RenderPreparation => {
            PyCurvedEquilibriumArrowGestureCategoryV1::RenderPreparation
        }
        CurvedEquilibriumArrowGestureCategoryV1::SessionConflict => {
            PyCurvedEquilibriumArrowGestureCategoryV1::SessionConflict
        }
    };
    let recovery = match error.recovery() {
        CurvedEquilibriumArrowGestureRecoveryV1::RefreshAndRestart => {
            PyCurvedEquilibriumArrowGestureRecoveryV1::RefreshAndRestart
        }
        CurvedEquilibriumArrowGestureRecoveryV1::ChangeGeometry => {
            PyCurvedEquilibriumArrowGestureRecoveryV1::ChangeGeometry
        }
        CurvedEquilibriumArrowGestureRecoveryV1::DocumentUnchanged => {
            PyCurvedEquilibriumArrowGestureRecoveryV1::DocumentUnchanged
        }
    };
    let exception = CurvedEquilibriumArrowGestureError::new_err(error.to_string());
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
        "CurvedEquilibriumArrowGestureError",
        module.py().get_type::<CurvedEquilibriumArrowGestureError>(),
    )?;
    module.add_class::<PyCurvedEquilibriumArrowGestureCategoryV1>()?;
    module.add_class::<PyCurvedEquilibriumArrowGestureRecoveryV1>()?;
    module.add_class::<PyCurvedEquilibriumArrowGestureV1>()?;
    module.add_class::<PyCurvedEquilibriumArrowOverlayV1>()?;
    module.add_class::<PyCurvedEquilibriumArrowPreviewV1>()?;
    module.add_class::<PyPreparedCurvedEquilibriumArrowV1>()?;
    module.add_class::<PyCurvedEquilibriumArrowCommitV1>()
}
