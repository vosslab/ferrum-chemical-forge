//! Opaque PyO3 seam for Rust-owned multi-point path authoring.

use std::cell::RefCell;

use crate::{
    ApiPresentationPathGestureV1, ApiPresentationPathOverlayV1, PresentationPathRenderCategoryV1,
    PresentationPathRenderErrorV1, PresentationPathRenderRecoveryV1,
    add_api_presentation_path_gesture_point_v1, begin_api_presentation_path_gesture_v1,
    cancel_api_presentation_path_gesture_v1, preview_incremental_api_presentation_path_gesture_v1,
    resolve_incremental_api_presentation_path_gesture_v1,
};
use ferrum_document::{
    DocumentFenceV1, PresentationGesturePoint2V1, PresentationPathGestureErrorV1,
    PresentationPathKindV1,
};
use pyo3::{create_exception, prelude::*};

use super::binding::PyDocumentSession;
use super::presentation_creation_gesture_binding::digest;

create_exception!(
    ferrum_chem,
    PresentationPathGestureError,
    super::binding::DocumentError
);

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "PresentationPathKindV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyPresentationPathKindV1 {
    Polyline,
    Polygon,
}
impl From<PyPresentationPathKindV1> for PresentationPathKindV1 {
    fn from(value: PyPresentationPathKindV1) -> Self {
        match value {
            PyPresentationPathKindV1::Polyline => Self::Polyline,
            PyPresentationPathKindV1::Polygon => Self::Polygon,
        }
    }
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "PresentationPathGestureCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyPresentationPathGestureCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    Consumed,
    Cancelled,
    Incomplete,
    InvalidGeometry,
    RenderPreparation,
    SessionConflict,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "PresentationPathGestureRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyPresentationPathGestureRecoveryV1 {
    RefreshAndRestart,
    ChangeGeometry,
    ReduceRequest,
    DocumentUnchanged,
}
#[pyclass(unsendable, module = "ferrum_chem", name = "PresentationPathGestureV1")]
pub(crate) struct PyPresentationPathGestureV1 {
    gesture: Option<ApiPresentationPathGestureV1>,
    kind: PyPresentationPathKindV1,
}
impl PyPresentationPathGestureV1 {
    fn kind(&self) -> PyPresentationPathKindV1 {
        self.kind
    }

    fn take_gesture(&mut self) -> Option<ApiPresentationPathGestureV1> {
        self.gesture.take()
    }
}
#[pyclass(frozen, module = "ferrum_chem", name = "PresentationPathProgressV1")]
pub(crate) struct PyPresentationPathProgressV1 {
    #[pyo3(get)]
    accepted_point_count: usize,
    #[pyo3(get)]
    minimum_point_count: usize,
    #[pyo3(get)]
    can_complete: bool,
}
#[pyclass(
    frozen,
    unsendable,
    module = "ferrum_chem",
    name = "PresentationPathOverlayV1"
)]
pub(crate) struct PyPresentationPathOverlayV1 {
    overlay: RefCell<Option<ApiPresentationPathOverlayV1>>,
    #[pyo3(get)]
    kind: Py<PyPresentationPathKindV1>,
    #[pyo3(get)]
    accepted_points: Vec<(f64, f64)>,
    #[pyo3(get)]
    hover: Option<(f64, f64)>,
    #[pyo3(get)]
    points: Vec<(f64, f64)>,
    #[pyo3(get)]
    closed: bool,
    #[pyo3(get)]
    stroke_color: String,
    #[pyo3(get)]
    stroke_width: f64,
    #[pyo3(get)]
    fill_color: Option<String>,
}
impl PyPresentationPathOverlayV1 {
    fn take_overlay(&self) -> Option<ApiPresentationPathOverlayV1> {
        self.overlay.borrow_mut().take()
    }
}
#[pymethods]
impl PyDocumentSession {
    fn begin_presentation_path_gesture_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        kind: PyRef<'_, PyPresentationPathKindV1>,
    ) -> PyResult<PyPresentationPathGestureV1> {
        begin_api_presentation_path_gesture_v1(
            &self.session,
            DocumentFenceV1::new(expected_revision, digest(&expected_digest_hex)?),
            (*kind).into(),
        )
        .map(|gesture| PyPresentationPathGestureV1 {
            gesture: Some(gesture),
            kind: *kind,
        })
        .map_err(|error| path_error(py, error))
    }
    fn add_presentation_path_gesture_point_v1(
        &self,
        py: Python<'_>,
        mut gesture: PyRefMut<'_, PyPresentationPathGestureV1>,
        x: f64,
        y: f64,
    ) -> PyResult<PyPresentationPathProgressV1> {
        let point = point_from_python(py, x, y)?;
        let gesture = gesture
            .gesture
            .as_mut()
            .ok_or_else(|| path_error(py, PresentationPathRenderErrorV1::Consumed))?;
        add_api_presentation_path_gesture_point_v1(&self.session, gesture, point)
            .map(progress_to_python)
            .map_err(|error| path_error(py, error))
    }
    fn preview_presentation_path_gesture_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyPresentationPathGestureV1>,
        hover: Option<(f64, f64)>,
    ) -> PyResult<PyPresentationPathOverlayV1> {
        let hover = hover
            .map(|(x, y)| point_from_python(py, x, y))
            .transpose()?;
        let kind = gesture.kind();
        let gesture = gesture
            .gesture
            .as_ref()
            .ok_or_else(|| path_error(py, PresentationPathRenderErrorV1::Consumed))?;
        preview_incremental_api_presentation_path_gesture_v1(&self.session, gesture, hover)
            .map(|overlay| overlay_to_python(py, kind, overlay))
            .map_err(|error| path_error(py, error))
    }
    fn resolve_presentation_path_gesture_v1(
        &self,
        py: Python<'_>,
        mut gesture: PyRefMut<'_, PyPresentationPathGestureV1>,
        overlay: PyRef<'_, PyPresentationPathOverlayV1>,
    ) -> PyResult<super::prepared_transition_binding::PySessionOperationTransitionRequestV1> {
        resolve_incremental_api_presentation_path_gesture_v1(
            &self.session,
            gesture
                .take_gesture()
                .ok_or_else(|| path_error(py, PresentationPathRenderErrorV1::Consumed))?,
            overlay
                .take_overlay()
                .ok_or_else(|| path_error(py, PresentationPathRenderErrorV1::Consumed))?,
        )
        .map(
            super::prepared_transition_binding::PySessionOperationTransitionRequestV1::from_request,
        )
        .map_err(|error| path_error(py, error))
    }
    fn cancel_presentation_path_gesture_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyPresentationPathGestureV1>,
    ) -> PyResult<()> {
        let gesture = gesture
            .gesture
            .as_ref()
            .ok_or_else(|| path_error(py, PresentationPathRenderErrorV1::Consumed))?;
        cancel_api_presentation_path_gesture_v1(&self.session, gesture)
            .map_err(|error| path_error(py, error))
    }
}

fn point_from_python(py: Python<'_>, x: f64, y: f64) -> PyResult<PresentationGesturePoint2V1> {
    PresentationGesturePoint2V1::new(x, y).map_err(|_| {
        path_error(
            py,
            PresentationPathRenderErrorV1::InvalidGeometry(
                PresentationPathGestureErrorV1::DegenerateGeometry,
            ),
        )
    })
}

fn progress_to_python(progress: crate::PresentationPathProgressV1) -> PyPresentationPathProgressV1 {
    PyPresentationPathProgressV1 {
        accepted_point_count: progress.accepted_points(),
        minimum_point_count: progress.minimum_points(),
        can_complete: progress.can_prepare(),
    }
}

fn overlay_to_python(
    py: Python<'_>,
    kind: PyPresentationPathKindV1,
    overlay: ApiPresentationPathOverlayV1,
) -> PyPresentationPathOverlayV1 {
    let issued = overlay.presentation();
    let accepted_points = issued
        .accepted_points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect::<Vec<_>>();
    let hover = issued.hover().map(|point| (point.x(), point.y()));
    let mut points = accepted_points.clone();
    points.extend(hover);
    let (stroke_color, stroke_width, fill_color) = {
        let appearance = issued.appearance();
        (
            appearance.stroke_color().to_owned(),
            appearance.stroke_width(),
            appearance.fill_color().map(str::to_owned),
        )
    };
    PyPresentationPathOverlayV1 {
        overlay: RefCell::new(Some(overlay)),
        kind: Py::new(py, kind).expect("kind allocates"),
        accepted_points,
        hover,
        points,
        closed: kind == PyPresentationPathKindV1::Polygon,
        stroke_color,
        stroke_width,
        fill_color,
    }
}
fn path_error(py: Python<'_>, error: PresentationPathRenderErrorV1) -> PyErr {
    let category = match &error {
        PresentationPathRenderErrorV1::Cancelled => PyPresentationPathGestureCategoryV1::Cancelled,
        PresentationPathRenderErrorV1::InvalidGeometry(
            PresentationPathGestureErrorV1::InsufficientPoints,
        ) => PyPresentationPathGestureCategoryV1::Incomplete,
        _ => match error.category() {
            PresentationPathRenderCategoryV1::StaleSnapshot => {
                PyPresentationPathGestureCategoryV1::StaleSnapshot
            }
            PresentationPathRenderCategoryV1::ForeignSession => {
                PyPresentationPathGestureCategoryV1::ForeignSession
            }
            PresentationPathRenderCategoryV1::MismatchedPreview => {
                PyPresentationPathGestureCategoryV1::MismatchedPreview
            }
            PresentationPathRenderCategoryV1::Consumed => {
                PyPresentationPathGestureCategoryV1::Consumed
            }
            PresentationPathRenderCategoryV1::Cancelled => {
                PyPresentationPathGestureCategoryV1::Cancelled
            }
            PresentationPathRenderCategoryV1::InvalidGeometry => {
                PyPresentationPathGestureCategoryV1::InvalidGeometry
            }
            PresentationPathRenderCategoryV1::RenderPreparation => {
                PyPresentationPathGestureCategoryV1::RenderPreparation
            }
            PresentationPathRenderCategoryV1::SessionConflict => {
                PyPresentationPathGestureCategoryV1::SessionConflict
            }
        },
    };
    let recovery = match error.recovery() {
        PresentationPathRenderRecoveryV1::RefreshAndRestart => {
            PyPresentationPathGestureRecoveryV1::RefreshAndRestart
        }
        PresentationPathRenderRecoveryV1::ChangeGeometry => {
            PyPresentationPathGestureRecoveryV1::ChangeGeometry
        }
        PresentationPathRenderRecoveryV1::ReduceRequest => {
            PyPresentationPathGestureRecoveryV1::ReduceRequest
        }
        PresentationPathRenderRecoveryV1::DocumentUnchanged => {
            PyPresentationPathGestureRecoveryV1::DocumentUnchanged
        }
    };
    let exception = PresentationPathGestureError::new_err(error.to_string());
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
        "PresentationPathGestureError",
        module.py().get_type::<PresentationPathGestureError>(),
    )?;
    module.add_class::<PyPresentationPathKindV1>()?;
    module.add_class::<PyPresentationPathGestureCategoryV1>()?;
    module.add_class::<PyPresentationPathGestureRecoveryV1>()?;
    module.add_class::<PyPresentationPathGestureV1>()?;
    module.add_class::<PyPresentationPathProgressV1>()?;
    module.add_class::<PyPresentationPathOverlayV1>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_refusal_attaches_reduce_request_recovery() {
        Python::initialize();
        Python::attach(|py| {
            let error = path_error(
                py,
                PresentationPathRenderErrorV1::InvalidGeometry(
                    PresentationPathGestureErrorV1::ResourceExhausted,
                ),
            );
            let recovery = error
                .value(py)
                .getattr("recovery")
                .expect("recovery attaches")
                .extract::<PyRef<'_, PyPresentationPathGestureRecoveryV1>>()
                .expect("closed recovery enum");

            assert!(*recovery == PyPresentationPathGestureRecoveryV1::ReduceRequest);
        });
    }
}
