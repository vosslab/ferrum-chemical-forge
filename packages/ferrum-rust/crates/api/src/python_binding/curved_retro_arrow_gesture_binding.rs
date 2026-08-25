//! Opaque PyO3 transport for renderer-preflighted quadratic retro arrows.

use ferrum_document::{DocumentFenceV1, PresentationGesturePoint2V1};
use ferrum_document_render::{
    CurvedRetroArrowGestureCategoryV1, CurvedRetroArrowGestureErrorV1,
    CurvedRetroArrowGestureRecoveryV1, CurvedRetroArrowGestureV1, CurvedRetroArrowPreviewV1,
    begin_curved_retro_arrow_gesture_v1, preview_curved_retro_arrow_gesture_v1,
    resolve_curved_retro_arrow_gesture_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::binding::PyDocumentSession;
use super::presentation_creation_gesture_binding::{PyPresentationPreviewRenderPlanV1, digest};

create_exception!(
    ferrum_chem,
    CurvedRetroArrowGestureError,
    super::binding::DocumentError
);

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "CurvedRetroArrowGestureCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyCurvedRetroArrowGestureCategoryV1 {
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
    name = "CurvedRetroArrowGestureRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyCurvedRetroArrowGestureRecoveryV1 {
    RefreshAndRestart,
    ChangeGeometry,
    DocumentUnchanged,
}

#[pyclass(unsendable, module = "ferrum_chem", name = "CurvedRetroArrowGestureV1")]
pub(crate) struct PyCurvedRetroArrowGestureV1 {
    gesture: Option<CurvedRetroArrowGestureV1>,
}

#[pyclass(unsendable, module = "ferrum_chem", name = "CurvedRetroArrowPreviewV1")]
pub(crate) struct PyCurvedRetroArrowPreviewV1 {
    preview: CurvedRetroArrowPreviewV1,
    #[pyo3(get)]
    plan: PyPresentationPreviewRenderPlanV1,
}

#[pymethods]
impl PyDocumentSession {
    #[allow(clippy::too_many_arguments)]
    fn begin_curved_retro_arrow_gesture_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        start_x: f64,
        start_y: f64,
        control_x: f64,
        control_y: f64,
    ) -> PyResult<PyCurvedRetroArrowGestureV1> {
        let fence = DocumentFenceV1::new(expected_revision, digest(&expected_digest_hex)?);
        begin_curved_retro_arrow_gesture_v1(
            &self.session,
            fence,
            point(start_x, start_y, py)?,
            point(control_x, control_y, py)?,
        )
        .map(|gesture| PyCurvedRetroArrowGestureV1 {
            gesture: Some(gesture),
        })
        .map_err(|error| retro_error(py, error))
    }

    fn preview_curved_retro_arrow_gesture_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyCurvedRetroArrowGestureV1>,
        end_x: f64,
        end_y: f64,
    ) -> PyResult<PyCurvedRetroArrowPreviewV1> {
        preview_curved_retro_arrow_gesture_v1(
            &self.session,
            gesture
                .gesture
                .as_ref()
                .ok_or_else(|| retro_error(py, CurvedRetroArrowGestureErrorV1::ReplayedGesture))?,
            point(end_x, end_y, py)?,
        )
        .map(|preview| preview_to_python(py, preview))
        .map_err(|error| retro_error(py, error))
    }

    fn resolve_curved_retro_arrow_gesture_v1(
        &self,
        py: Python<'_>,
        mut gesture: PyRefMut<'_, PyCurvedRetroArrowGestureV1>,
        preview: PyRef<'_, PyCurvedRetroArrowPreviewV1>,
    ) -> PyResult<super::prepared_transition_binding::PySessionOperationTransitionRequestV1> {
        resolve_curved_retro_arrow_gesture_v1(
            &self.session,
            gesture
                .gesture
                .take()
                .ok_or_else(|| retro_error(py, CurvedRetroArrowGestureErrorV1::ReplayedGesture))?,
            preview.preview.clone(),
        )
        .map(
            super::prepared_transition_binding::PySessionOperationTransitionRequestV1::from_request,
        )
        .map_err(|error| retro_error(py, error))
    }
}

fn point(x: f64, y: f64, py: Python<'_>) -> PyResult<PresentationGesturePoint2V1> {
    PresentationGesturePoint2V1::new(x, y)
        .map_err(|_| retro_error(py, CurvedRetroArrowGestureErrorV1::InvalidPoint))
}

fn preview_to_python(
    _py: Python<'_>,
    preview: CurvedRetroArrowPreviewV1,
) -> PyCurvedRetroArrowPreviewV1 {
    PyCurvedRetroArrowPreviewV1 {
        plan: preview.plan().into(),
        preview,
    }
}

fn retro_error(py: Python<'_>, error: CurvedRetroArrowGestureErrorV1) -> PyErr {
    let category = match error.category() {
        CurvedRetroArrowGestureCategoryV1::ForeignSession => {
            PyCurvedRetroArrowGestureCategoryV1::ForeignSession
        }
        CurvedRetroArrowGestureCategoryV1::StaleSnapshot => {
            PyCurvedRetroArrowGestureCategoryV1::StaleSnapshot
        }
        CurvedRetroArrowGestureCategoryV1::MismatchedPreview => {
            PyCurvedRetroArrowGestureCategoryV1::MismatchedPreview
        }
        CurvedRetroArrowGestureCategoryV1::ReplayedGesture => {
            PyCurvedRetroArrowGestureCategoryV1::ReplayedGesture
        }
        CurvedRetroArrowGestureCategoryV1::InvalidPoint => {
            PyCurvedRetroArrowGestureCategoryV1::InvalidPoint
        }
        CurvedRetroArrowGestureCategoryV1::CollapsedSpan => {
            PyCurvedRetroArrowGestureCategoryV1::CollapsedSpan
        }
        CurvedRetroArrowGestureCategoryV1::ControlTooNearChord => {
            PyCurvedRetroArrowGestureCategoryV1::ControlTooNearChord
        }
        CurvedRetroArrowGestureCategoryV1::ExceedsGeometryLimit => {
            PyCurvedRetroArrowGestureCategoryV1::ExceedsGeometryLimit
        }
        CurvedRetroArrowGestureCategoryV1::RenderPreparation => {
            PyCurvedRetroArrowGestureCategoryV1::RenderPreparation
        }
        CurvedRetroArrowGestureCategoryV1::SessionConflict => {
            PyCurvedRetroArrowGestureCategoryV1::SessionConflict
        }
    };
    let recovery = match error.recovery() {
        CurvedRetroArrowGestureRecoveryV1::RefreshAndRestart => {
            PyCurvedRetroArrowGestureRecoveryV1::RefreshAndRestart
        }
        CurvedRetroArrowGestureRecoveryV1::ChangeGeometry => {
            PyCurvedRetroArrowGestureRecoveryV1::ChangeGeometry
        }
        CurvedRetroArrowGestureRecoveryV1::DocumentUnchanged => {
            PyCurvedRetroArrowGestureRecoveryV1::DocumentUnchanged
        }
    };
    let exception = CurvedRetroArrowGestureError::new_err(error.to_string());
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
        "CurvedRetroArrowGestureError",
        module.py().get_type::<CurvedRetroArrowGestureError>(),
    )?;
    module.add_class::<PyCurvedRetroArrowGestureCategoryV1>()?;
    module.add_class::<PyCurvedRetroArrowGestureRecoveryV1>()?;
    module.add_class::<PyCurvedRetroArrowGestureV1>()?;
    module.add_class::<PyCurvedRetroArrowPreviewV1>()?;
    Ok(())
}
