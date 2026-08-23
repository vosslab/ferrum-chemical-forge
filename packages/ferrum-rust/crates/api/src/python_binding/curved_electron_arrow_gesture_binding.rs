//! Opaque PyO3 transport for renderer-preflighted quadratic electron arrows.

use ferrum_document::{DocumentFenceV1, PresentationGesturePoint2V1};
use ferrum_document_render::{
    CommittedCurvedElectronArrowV1, CurvedElectronArrowGestureCategoryV1,
    CurvedElectronArrowGestureErrorV1, CurvedElectronArrowGestureRecoveryV1,
    CurvedElectronArrowGestureV1, CurvedElectronArrowPreviewV1, PreparedCurvedElectronArrowV1,
    begin_curved_electron_arrow_gesture_v1, commit_curved_electron_arrow_gesture_v1,
    prepare_curved_electron_arrow_gesture_v1, preview_curved_electron_arrow_gesture_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::binding::{PyDocumentSession, PySessionOperationResultV1};
use super::presentation_creation_gesture_binding::{
    PyPresentationGestureRootKindV1, PyPresentationGestureRootSelectorV1, digest,
};

create_exception!(
    ferrum_chem,
    CurvedElectronArrowGestureError,
    super::binding::DocumentError
);

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "CurvedElectronArrowGestureCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyCurvedElectronArrowGestureCategoryV1 {
    ForeignSession,
    StaleSnapshot,
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
    name = "CurvedElectronArrowGestureRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyCurvedElectronArrowGestureRecoveryV1 {
    RefreshAndRestart,
    ChangeGeometry,
    DocumentUnchanged,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "CurvedElectronArrowGestureV1"
)]
pub(crate) struct PyCurvedElectronArrowGestureV1 {
    gesture: CurvedElectronArrowGestureV1,
}

#[pyclass(frozen, module = "ferrum_chem", name = "CurvedElectronArrowOverlayV1")]
pub(crate) struct PyCurvedElectronArrowOverlayV1 {
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
    pub cubic_control_1_x: f64,
    #[pyo3(get)]
    pub cubic_control_1_y: f64,
    #[pyo3(get)]
    pub cubic_control_2_x: f64,
    #[pyo3(get)]
    pub cubic_control_2_y: f64,
    #[pyo3(get)]
    pub head: Vec<(f64, f64)>,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "CurvedElectronArrowPreviewV1"
)]
pub(crate) struct PyCurvedElectronArrowPreviewV1 {
    preview: CurvedElectronArrowPreviewV1,
    #[pyo3(get)]
    overlay: Py<PyCurvedElectronArrowOverlayV1>,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PreparedCurvedElectronArrowV1"
)]
pub(crate) struct PyPreparedCurvedElectronArrowV1 {
    prepared: PreparedCurvedElectronArrowV1,
}

#[pyclass(frozen, module = "ferrum_chem", name = "CurvedElectronArrowCommitV1")]
pub(crate) struct PyCurvedElectronArrowCommitV1 {
    #[pyo3(get)]
    root: Py<PyPresentationGestureRootSelectorV1>,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}

#[pymethods]
impl PyDocumentSession {
    #[allow(clippy::too_many_arguments)]
    fn begin_curved_electron_arrow_gesture_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        start_x: f64,
        start_y: f64,
        control_x: f64,
        control_y: f64,
    ) -> PyResult<PyCurvedElectronArrowGestureV1> {
        let fence = DocumentFenceV1::new(expected_revision, digest(&expected_digest_hex)?);
        let start = point(start_x, start_y, py)?;
        let control = point(control_x, control_y, py)?;
        begin_curved_electron_arrow_gesture_v1(&self.session, fence, start, control)
            .map(|gesture| PyCurvedElectronArrowGestureV1 { gesture })
            .map_err(|error| electron_error(py, error))
    }

    fn preview_curved_electron_arrow_gesture_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyCurvedElectronArrowGestureV1>,
        end_x: f64,
        end_y: f64,
    ) -> PyResult<PyCurvedElectronArrowPreviewV1> {
        preview_curved_electron_arrow_gesture_v1(
            &self.session,
            &gesture.gesture,
            point(end_x, end_y, py)?,
        )
        .map(|preview| preview_to_python(py, preview))
        .map_err(|error| electron_error(py, error))
    }

    fn prepare_curved_electron_arrow_gesture_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyCurvedElectronArrowGestureV1>,
        preview: PyRef<'_, PyCurvedElectronArrowPreviewV1>,
    ) -> PyResult<PyPreparedCurvedElectronArrowV1> {
        prepare_curved_electron_arrow_gesture_v1(
            &mut self.session,
            &gesture.gesture,
            &preview.preview,
        )
        .map(|prepared| PyPreparedCurvedElectronArrowV1 { prepared })
        .map_err(|error| electron_error(py, error))
    }

    fn commit_curved_electron_arrow_gesture_v1(
        &mut self,
        py: Python<'_>,
        mut prepared: PyRefMut<'_, PyPreparedCurvedElectronArrowV1>,
    ) -> PyResult<PyCurvedElectronArrowCommitV1> {
        commit_curved_electron_arrow_gesture_v1(&mut self.session, &mut prepared.prepared)
            .map(|value| commit_to_python(py, value))
            .map_err(|error| electron_error(py, error))
    }
}

fn point(x: f64, y: f64, py: Python<'_>) -> PyResult<PresentationGesturePoint2V1> {
    PresentationGesturePoint2V1::new(x, y)
        .map_err(|_| electron_error(py, CurvedElectronArrowGestureErrorV1::InvalidPoint))
}

fn preview_to_python(
    py: Python<'_>,
    preview: CurvedElectronArrowPreviewV1,
) -> PyCurvedElectronArrowPreviewV1 {
    let overlay = preview.overlay();
    let value = PyCurvedElectronArrowOverlayV1 {
        start_x: overlay.start().x(),
        start_y: overlay.start().y(),
        control_x: overlay.control().x(),
        control_y: overlay.control().y(),
        end_x: overlay.end().x(),
        end_y: overlay.end().y(),
        cubic_control_1_x: overlay.cubic_control_1().x(),
        cubic_control_1_y: overlay.cubic_control_1().y(),
        cubic_control_2_x: overlay.cubic_control_2().x(),
        cubic_control_2_y: overlay.cubic_control_2().y(),
        head: overlay
            .head()
            .iter()
            .map(|point| (point.x(), point.y()))
            .collect(),
    };
    PyCurvedElectronArrowPreviewV1 {
        preview,
        overlay: Py::new(py, value).expect("overlay allocates"),
    }
}

fn commit_to_python(
    py: Python<'_>,
    value: CommittedCurvedElectronArrowV1,
) -> PyCurvedElectronArrowCommitV1 {
    PyCurvedElectronArrowCommitV1 {
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

fn electron_error(py: Python<'_>, error: CurvedElectronArrowGestureErrorV1) -> PyErr {
    let category = match error.category() {
        CurvedElectronArrowGestureCategoryV1::ForeignSession => {
            PyCurvedElectronArrowGestureCategoryV1::ForeignSession
        }
        CurvedElectronArrowGestureCategoryV1::StaleSnapshot => {
            PyCurvedElectronArrowGestureCategoryV1::StaleSnapshot
        }
        CurvedElectronArrowGestureCategoryV1::MismatchedPreview => {
            PyCurvedElectronArrowGestureCategoryV1::MismatchedPreview
        }
        CurvedElectronArrowGestureCategoryV1::ReplayedGesture => {
            PyCurvedElectronArrowGestureCategoryV1::ReplayedGesture
        }
        CurvedElectronArrowGestureCategoryV1::InvalidPoint => {
            PyCurvedElectronArrowGestureCategoryV1::InvalidPoint
        }
        CurvedElectronArrowGestureCategoryV1::CollapsedSpan => {
            PyCurvedElectronArrowGestureCategoryV1::CollapsedSpan
        }
        CurvedElectronArrowGestureCategoryV1::ControlTooNearChord => {
            PyCurvedElectronArrowGestureCategoryV1::ControlTooNearChord
        }
        CurvedElectronArrowGestureCategoryV1::ExceedsGeometryLimit => {
            PyCurvedElectronArrowGestureCategoryV1::ExceedsGeometryLimit
        }
        CurvedElectronArrowGestureCategoryV1::RenderPreparation => {
            PyCurvedElectronArrowGestureCategoryV1::RenderPreparation
        }
        CurvedElectronArrowGestureCategoryV1::SessionConflict => {
            PyCurvedElectronArrowGestureCategoryV1::SessionConflict
        }
    };
    let recovery = match error.recovery() {
        CurvedElectronArrowGestureRecoveryV1::RefreshAndRestart => {
            PyCurvedElectronArrowGestureRecoveryV1::RefreshAndRestart
        }
        CurvedElectronArrowGestureRecoveryV1::ChangeGeometry => {
            PyCurvedElectronArrowGestureRecoveryV1::ChangeGeometry
        }
        CurvedElectronArrowGestureRecoveryV1::DocumentUnchanged => {
            PyCurvedElectronArrowGestureRecoveryV1::DocumentUnchanged
        }
    };
    let exception = CurvedElectronArrowGestureError::new_err(error.to_string());
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
        "CurvedElectronArrowGestureError",
        module.py().get_type::<CurvedElectronArrowGestureError>(),
    )?;
    module.add_class::<PyCurvedElectronArrowGestureCategoryV1>()?;
    module.add_class::<PyCurvedElectronArrowGestureRecoveryV1>()?;
    module.add_class::<PyCurvedElectronArrowGestureV1>()?;
    module.add_class::<PyCurvedElectronArrowOverlayV1>()?;
    module.add_class::<PyCurvedElectronArrowPreviewV1>()?;
    module.add_class::<PyPreparedCurvedElectronArrowV1>()?;
    module.add_class::<PyCurvedElectronArrowCommitV1>()
}
