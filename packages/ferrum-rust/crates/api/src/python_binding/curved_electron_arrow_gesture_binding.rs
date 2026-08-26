//! Opaque PyO3 transport for renderer-preflighted quadratic electron arrows.

use ferrum_document::{DocumentFenceV1, PresentationGesturePoint2V1};
use ferrum_document_render::{
    CurvedElectronArrowGestureCategoryV1, CurvedElectronArrowGestureErrorV1,
    CurvedElectronArrowGestureRecoveryV1, CurvedElectronArrowGestureV1,
    CurvedElectronArrowPreviewV1, begin_curved_electron_arrow_gesture_v1,
    preview_curved_electron_arrow_gesture_v1, resolve_curved_electron_arrow_gesture_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::binding::PyDocumentSession;
use super::presentation_creation_gesture_binding::{PyPresentationPreviewRenderPlanV1, digest};

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
    Consumed,
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
    gesture: Option<CurvedElectronArrowGestureV1>,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "CurvedElectronArrowPreviewV1"
)]
pub(crate) struct PyCurvedElectronArrowPreviewV1 {
    preview: CurvedElectronArrowPreviewV1,
    #[pyo3(get)]
    plan: PyPresentationPreviewRenderPlanV1,
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
            .map(|gesture| PyCurvedElectronArrowGestureV1 {
                gesture: Some(gesture),
            })
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
            gesture
                .gesture
                .as_ref()
                .ok_or_else(|| electron_error(py, CurvedElectronArrowGestureErrorV1::Consumed))?,
            point(end_x, end_y, py)?,
        )
        .map(|preview| preview_to_python(py, preview))
        .map_err(|error| electron_error(py, error))
    }

    fn resolve_curved_electron_arrow_gesture_v1(
        &self,
        py: Python<'_>,
        mut gesture: PyRefMut<'_, PyCurvedElectronArrowGestureV1>,
        preview: PyRef<'_, PyCurvedElectronArrowPreviewV1>,
    ) -> PyResult<super::prepared_transition_binding::PySessionOperationTransitionRequestV1> {
        resolve_curved_electron_arrow_gesture_v1(
            &self.session,
            gesture
                .gesture
                .take()
                .ok_or_else(|| electron_error(py, CurvedElectronArrowGestureErrorV1::Consumed))?,
            preview.preview.clone(),
        )
        .map(
            super::prepared_transition_binding::PySessionOperationTransitionRequestV1::from_request,
        )
        .map_err(|error| electron_error(py, error))
    }
}

fn point(x: f64, y: f64, py: Python<'_>) -> PyResult<PresentationGesturePoint2V1> {
    PresentationGesturePoint2V1::new(x, y)
        .map_err(|_| electron_error(py, CurvedElectronArrowGestureErrorV1::InvalidPoint))
}

fn preview_to_python(
    _py: Python<'_>,
    preview: CurvedElectronArrowPreviewV1,
) -> PyCurvedElectronArrowPreviewV1 {
    PyCurvedElectronArrowPreviewV1 {
        plan: preview.plan().into(),
        preview,
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
        CurvedElectronArrowGestureCategoryV1::Consumed => {
            PyCurvedElectronArrowGestureCategoryV1::Consumed
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
    module.add_class::<PyCurvedElectronArrowPreviewV1>()?;
    Ok(())
}
