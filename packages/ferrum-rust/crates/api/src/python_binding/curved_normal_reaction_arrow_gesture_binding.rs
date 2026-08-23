//! Opaque PyO3 transport for renderer-preflighted quadratic curved-normal reaction arrows.

use ferrum_document::{DocumentFenceV1, PresentationGesturePoint2V1};
use ferrum_document_render::{
    CommittedCurvedNormalReactionArrowV1, CurvedNormalReactionArrowGestureCategoryV1,
    CurvedNormalReactionArrowGestureErrorV1, CurvedNormalReactionArrowGestureRecoveryV1,
    CurvedNormalReactionArrowGestureV1, CurvedNormalReactionArrowPreviewV1,
    PreparedCurvedNormalReactionArrowV1, begin_curved_normal_reaction_arrow_gesture_v1,
    commit_curved_normal_reaction_arrow_gesture_v1,
    prepare_curved_normal_reaction_arrow_gesture_v1,
    preview_curved_normal_reaction_arrow_gesture_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::binding::{PyDocumentSession, PySessionOperationResultV1};
use super::presentation_creation_gesture_binding::{
    PyPresentationGestureRootKindV1, PyPresentationGestureRootSelectorV1, digest,
};

create_exception!(
    ferrum_chem,
    CurvedNormalReactionArrowGestureError,
    super::binding::DocumentError
);

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "CurvedNormalReactionArrowGestureCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyCurvedNormalReactionArrowGestureCategoryV1 {
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
    name = "CurvedNormalReactionArrowGestureRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyCurvedNormalReactionArrowGestureRecoveryV1 {
    RefreshAndRestart,
    ChangeGeometry,
    DocumentUnchanged,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "CurvedNormalReactionArrowGestureV1"
)]
pub(crate) struct PyCurvedNormalReactionArrowGestureV1 {
    gesture: CurvedNormalReactionArrowGestureV1,
}

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "CurvedNormalReactionArrowOverlayV1"
)]
pub(crate) struct PyCurvedNormalReactionArrowOverlayV1 {
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
    name = "CurvedNormalReactionArrowPreviewV1"
)]
pub(crate) struct PyCurvedNormalReactionArrowPreviewV1 {
    preview: CurvedNormalReactionArrowPreviewV1,
    #[pyo3(get)]
    overlay: Py<PyCurvedNormalReactionArrowOverlayV1>,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PreparedCurvedNormalReactionArrowV1"
)]
pub(crate) struct PyPreparedCurvedNormalReactionArrowV1 {
    prepared: PreparedCurvedNormalReactionArrowV1,
}

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "CurvedNormalReactionArrowCommitV1"
)]
pub(crate) struct PyCurvedNormalReactionArrowCommitV1 {
    #[pyo3(get)]
    root: Py<PyPresentationGestureRootSelectorV1>,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}

#[pymethods]
impl PyDocumentSession {
    #[allow(clippy::too_many_arguments)]
    fn begin_curved_normal_reaction_arrow_gesture_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        start_x: f64,
        start_y: f64,
        control_x: f64,
        control_y: f64,
    ) -> PyResult<PyCurvedNormalReactionArrowGestureV1> {
        let fence = DocumentFenceV1::new(expected_revision, digest(&expected_digest_hex)?);
        begin_curved_normal_reaction_arrow_gesture_v1(
            &self.session,
            fence,
            point(start_x, start_y, py)?,
            point(control_x, control_y, py)?,
        )
        .map(|gesture| PyCurvedNormalReactionArrowGestureV1 { gesture })
        .map_err(|error| normal_error(py, error))
    }

    fn preview_curved_normal_reaction_arrow_gesture_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyCurvedNormalReactionArrowGestureV1>,
        end_x: f64,
        end_y: f64,
    ) -> PyResult<PyCurvedNormalReactionArrowPreviewV1> {
        preview_curved_normal_reaction_arrow_gesture_v1(
            &self.session,
            &gesture.gesture,
            point(end_x, end_y, py)?,
        )
        .map(|preview| preview_to_python(py, preview))
        .map_err(|error| normal_error(py, error))
    }

    fn prepare_curved_normal_reaction_arrow_gesture_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyCurvedNormalReactionArrowGestureV1>,
        preview: PyRef<'_, PyCurvedNormalReactionArrowPreviewV1>,
    ) -> PyResult<PyPreparedCurvedNormalReactionArrowV1> {
        prepare_curved_normal_reaction_arrow_gesture_v1(
            &mut self.session,
            &gesture.gesture,
            &preview.preview,
        )
        .map(|prepared| PyPreparedCurvedNormalReactionArrowV1 { prepared })
        .map_err(|error| normal_error(py, error))
    }

    fn commit_curved_normal_reaction_arrow_gesture_v1(
        &mut self,
        py: Python<'_>,
        mut prepared: PyRefMut<'_, PyPreparedCurvedNormalReactionArrowV1>,
    ) -> PyResult<PyCurvedNormalReactionArrowCommitV1> {
        commit_curved_normal_reaction_arrow_gesture_v1(&mut self.session, &mut prepared.prepared)
            .map(|value| commit_to_python(py, value))
            .map_err(|error| normal_error(py, error))
    }
}

fn point(x: f64, y: f64, py: Python<'_>) -> PyResult<PresentationGesturePoint2V1> {
    PresentationGesturePoint2V1::new(x, y)
        .map_err(|_| normal_error(py, CurvedNormalReactionArrowGestureErrorV1::InvalidPoint))
}

fn preview_to_python(
    py: Python<'_>,
    preview: CurvedNormalReactionArrowPreviewV1,
) -> PyCurvedNormalReactionArrowPreviewV1 {
    let overlay = preview.overlay();
    let value = PyCurvedNormalReactionArrowOverlayV1 {
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
    PyCurvedNormalReactionArrowPreviewV1 {
        preview,
        overlay: Py::new(py, value).expect("overlay allocates"),
    }
}

fn commit_to_python(
    py: Python<'_>,
    value: CommittedCurvedNormalReactionArrowV1,
) -> PyCurvedNormalReactionArrowCommitV1 {
    PyCurvedNormalReactionArrowCommitV1 {
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

fn normal_error(py: Python<'_>, error: CurvedNormalReactionArrowGestureErrorV1) -> PyErr {
    let category = match error.category() {
        CurvedNormalReactionArrowGestureCategoryV1::ForeignSession => {
            PyCurvedNormalReactionArrowGestureCategoryV1::ForeignSession
        }
        CurvedNormalReactionArrowGestureCategoryV1::StaleSnapshot => {
            PyCurvedNormalReactionArrowGestureCategoryV1::StaleSnapshot
        }
        CurvedNormalReactionArrowGestureCategoryV1::MismatchedPreview => {
            PyCurvedNormalReactionArrowGestureCategoryV1::MismatchedPreview
        }
        CurvedNormalReactionArrowGestureCategoryV1::ReplayedGesture => {
            PyCurvedNormalReactionArrowGestureCategoryV1::ReplayedGesture
        }
        CurvedNormalReactionArrowGestureCategoryV1::InvalidPoint => {
            PyCurvedNormalReactionArrowGestureCategoryV1::InvalidPoint
        }
        CurvedNormalReactionArrowGestureCategoryV1::CollapsedSpan => {
            PyCurvedNormalReactionArrowGestureCategoryV1::CollapsedSpan
        }
        CurvedNormalReactionArrowGestureCategoryV1::ControlTooNearChord => {
            PyCurvedNormalReactionArrowGestureCategoryV1::ControlTooNearChord
        }
        CurvedNormalReactionArrowGestureCategoryV1::ExceedsGeometryLimit => {
            PyCurvedNormalReactionArrowGestureCategoryV1::ExceedsGeometryLimit
        }
        CurvedNormalReactionArrowGestureCategoryV1::RenderPreparation => {
            PyCurvedNormalReactionArrowGestureCategoryV1::RenderPreparation
        }
        CurvedNormalReactionArrowGestureCategoryV1::SessionConflict => {
            PyCurvedNormalReactionArrowGestureCategoryV1::SessionConflict
        }
    };
    let recovery = match error.recovery() {
        CurvedNormalReactionArrowGestureRecoveryV1::RefreshAndRestart => {
            PyCurvedNormalReactionArrowGestureRecoveryV1::RefreshAndRestart
        }
        CurvedNormalReactionArrowGestureRecoveryV1::ChangeGeometry => {
            PyCurvedNormalReactionArrowGestureRecoveryV1::ChangeGeometry
        }
        CurvedNormalReactionArrowGestureRecoveryV1::DocumentUnchanged => {
            PyCurvedNormalReactionArrowGestureRecoveryV1::DocumentUnchanged
        }
    };
    let exception = CurvedNormalReactionArrowGestureError::new_err(error.to_string());
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
        "CurvedNormalReactionArrowGestureError",
        module
            .py()
            .get_type::<CurvedNormalReactionArrowGestureError>(),
    )?;
    module.add_class::<PyCurvedNormalReactionArrowGestureCategoryV1>()?;
    module.add_class::<PyCurvedNormalReactionArrowGestureRecoveryV1>()?;
    module.add_class::<PyCurvedNormalReactionArrowGestureV1>()?;
    module.add_class::<PyCurvedNormalReactionArrowOverlayV1>()?;
    module.add_class::<PyCurvedNormalReactionArrowPreviewV1>()?;
    module.add_class::<PyPreparedCurvedNormalReactionArrowV1>()?;
    module.add_class::<PyCurvedNormalReactionArrowCommitV1>()
}
