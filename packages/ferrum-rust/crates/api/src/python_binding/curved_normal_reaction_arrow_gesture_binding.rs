//! Opaque PyO3 transport for renderer-preflighted quadratic curved-normal reaction arrows.

use ferrum_document::{DocumentFenceV1, PresentationGesturePoint2V1};
use ferrum_document_render::{
    CurvedNormalReactionArrowGestureCategoryV1, CurvedNormalReactionArrowGestureErrorV1,
    CurvedNormalReactionArrowGestureRecoveryV1, CurvedNormalReactionArrowGestureV1,
    CurvedNormalReactionArrowPreviewV1, begin_curved_normal_reaction_arrow_gesture_v1,
    preview_curved_normal_reaction_arrow_gesture_v1,
    resolve_curved_normal_reaction_arrow_gesture_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::binding::PyDocumentSession;
use super::presentation_creation_gesture_binding::{PyPresentationPreviewRenderPlanV1, digest};

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
    gesture: Option<CurvedNormalReactionArrowGestureV1>,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "CurvedNormalReactionArrowPreviewV1"
)]
pub(crate) struct PyCurvedNormalReactionArrowPreviewV1 {
    preview: Option<CurvedNormalReactionArrowPreviewV1>,
    #[pyo3(get)]
    plan: PyPresentationPreviewRenderPlanV1,
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
        .map(|gesture| PyCurvedNormalReactionArrowGestureV1 {
            gesture: Some(gesture),
        })
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
            gesture.gesture.as_ref().ok_or_else(|| {
                normal_error(py, CurvedNormalReactionArrowGestureErrorV1::ReplayedGesture)
            })?,
            point(end_x, end_y, py)?,
        )
        .map(|preview| preview_to_python(py, preview))
        .map_err(|error| normal_error(py, error))
    }

    fn resolve_curved_normal_reaction_arrow_gesture_v1(
        &self,
        py: Python<'_>,
        mut gesture: PyRefMut<'_, PyCurvedNormalReactionArrowGestureV1>,
        mut preview: PyRefMut<'_, PyCurvedNormalReactionArrowPreviewV1>,
    ) -> PyResult<super::prepared_transition_binding::PySessionOperationTransitionRequestV1> {
        resolve_curved_normal_reaction_arrow_gesture_v1(
            &self.session,
            gesture.gesture.take().ok_or_else(|| {
                normal_error(py, CurvedNormalReactionArrowGestureErrorV1::ReplayedGesture)
            })?,
            preview.preview.take().ok_or_else(|| {
                normal_error(py, CurvedNormalReactionArrowGestureErrorV1::ReplayedGesture)
            })?,
        )
        .map(
            super::prepared_transition_binding::PySessionOperationTransitionRequestV1::from_request,
        )
        .map_err(|error| normal_error(py, error))
    }
}

fn point(x: f64, y: f64, py: Python<'_>) -> PyResult<PresentationGesturePoint2V1> {
    PresentationGesturePoint2V1::new(x, y)
        .map_err(|_| normal_error(py, CurvedNormalReactionArrowGestureErrorV1::InvalidPoint))
}

fn preview_to_python(
    _py: Python<'_>,
    preview: CurvedNormalReactionArrowPreviewV1,
) -> PyCurvedNormalReactionArrowPreviewV1 {
    PyCurvedNormalReactionArrowPreviewV1 {
        plan: preview.plan().into(),
        preview: Some(preview),
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
    module.add_class::<PyCurvedNormalReactionArrowPreviewV1>()?;
    Ok(())
}
