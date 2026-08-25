//! Opaque PyO3 seam for Rust-owned direct straight normal-arrow authoring.

use super::binding::PyDocumentSession;
use super::prepared_transition_binding::PySessionOperationTransitionRequestV1;
use super::presentation_render_plan_binding::{
    PyPresentationRenderBoundsV1, PyPresentationVectorOperationV1,
};
use ferrum_document::{
    ArrowGestureStyleV1, DocumentFenceV1, PresentationCreationGestureV1,
    PresentationCreationPreviewV1, PresentationGestureErrorV1, PresentationGestureKindV1,
    PresentationGesturePoint2V1, PresentationGestureSnapPolicyV1, PresentationGestureStyleV1,
};
use ferrum_render::{PresentationPreviewRenderPlanV1, PresentationPreviewRenderRootV1};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
create_exception!(
    ferrum_chem,
    PresentationGestureError,
    super::binding::DocumentError
);
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "PresentationGestureKindV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyPresentationGestureKindV1 {
    StraightNormalArrow,
    StraightEquilibriumArrow,
    Plus,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "PresentationGestureCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyPresentationGestureCategoryV1 {
    StaleRevision,
    StaleDigest,
    ForeignSession,
    ReplayedGesture,
    PreviewMismatch,
    NonFinitePoint,
    CollapsedEndpoint,
    BelowMinimumLength,
    ExceedsGeometryLimit,
    InvalidSnapPolicy,
    InvalidGestureStyle,
    SessionConflict,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "PresentationGestureRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyPresentationGestureRecoveryV1 {
    RefreshAndRestart,
    AdjustEndpoint,
    ChangeToolOrStyle,
    RefreshAndReport,
}
#[pyclass(frozen, module = "ferrum_chem", name = "ArrowGestureStyleV1")]
pub(crate) struct PyArrowGestureStyleV1 {
    style: ArrowGestureStyleV1,
}
#[pymethods]
impl PyArrowGestureStyleV1 {
    #[new]
    #[pyo3(signature=(start_head=false,end_head=true))]
    fn new(start_head: bool, end_head: bool) -> Self {
        Self {
            style: ArrowGestureStyleV1::new(start_head, end_head),
        }
    }
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PresentationGestureSnapPolicyV1"
)]
pub(crate) struct PyPresentationGestureSnapPolicyV1 {
    policy: PresentationGestureSnapPolicyV1,
}
#[pymethods]
impl PyPresentationGestureSnapPolicyV1 {
    #[new]
    #[pyo3(signature=(angle_increment_degrees=None,fixed_length_pt=None))]
    fn new(
        py: Python<'_>,
        angle_increment_degrees: Option<&Bound<'_, PyAny>>,
        fixed_length_pt: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let angle = optional_u16(py, angle_increment_degrees)?;
        let length = optional_u16(py, fixed_length_pt)?;
        PresentationGestureSnapPolicyV1::new(angle, length)
            .map(|policy| Self { policy })
            .map_err(|error| presentation_error(py, error))
    }
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PresentationCreationGestureV1"
)]
pub(crate) struct PyPresentationCreationGestureV1 {
    gesture: Option<PresentationCreationGestureV1>,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PresentationCreationPreviewV1"
)]
pub(crate) struct PyPresentationCreationPreviewV1 {
    preview: PresentationCreationPreviewV1,
    #[pyo3(get)]
    plan: PyPresentationPreviewRenderPlanV1,
}

/// Identifier-free renderer content for one transient Plus preview root.
#[pyclass(frozen, name = "PresentationPreviewPlusV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationPreviewPlusV1 {
    #[pyo3(get)]
    anchor: super::render_binding::PyRenderPointV1,
    #[pyo3(get)]
    operation_origin: super::render_binding::PyRenderPointV1,
    #[pyo3(get)]
    text: String,
    #[pyo3(get)]
    face: String,
    #[pyo3(get)]
    size: f64,
    #[pyo3(get)]
    paint: String,
    #[pyo3(get)]
    z: i32,
    #[pyo3(get)]
    background: Option<String>,
}

/// Identifier-free renderer output for one transient presentation preview root.
#[pyclass(frozen, name = "PresentationPreviewRenderRootV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationPreviewRenderRootV1 {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    bounds: PyPresentationRenderBoundsV1,
    vector_operations: Vec<PyPresentationVectorOperationV1>,
    #[pyo3(get)]
    plus: Option<PyPresentationPreviewPlusV1>,
}

#[pymethods]
impl PyPresentationPreviewRenderRootV1 {
    #[getter]
    fn vector_operations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, self.vector_operations.iter().cloned())?.unbind())
    }
}

/// Immutable, identifier-free renderer plan for one transient presentation preview.
#[pyclass(frozen, name = "PresentationPreviewRenderPlanV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationPreviewRenderPlanV1 {
    #[pyo3(get)]
    schema: &'static str,
    roots: Vec<PyPresentationPreviewRenderRootV1>,
}

#[pymethods]
impl PyPresentationPreviewRenderPlanV1 {
    #[getter]
    fn roots(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, self.roots.iter().cloned())?.unbind())
    }
}

impl From<&PresentationPreviewRenderPlanV1> for PyPresentationPreviewRenderPlanV1 {
    fn from(value: &PresentationPreviewRenderPlanV1) -> Self {
        Self {
            schema: value.schema(),
            roots: value.roots().iter().map(preview_root_from).collect(),
        }
    }
}
#[pymethods]
impl PyDocumentSession {
    #[allow(clippy::too_many_arguments)]
    fn begin_presentation_creation_gesture_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        kind: PyRef<'_, PyPresentationGestureKindV1>,
        start_x: f64,
        start_y: f64,
        style: Option<PyRef<'_, PyArrowGestureStyleV1>>,
        snap: PyRef<'_, PyPresentationGestureSnapPolicyV1>,
    ) -> PyResult<PyPresentationCreationGestureV1> {
        let fence = DocumentFenceV1::new(expected_revision, digest(&expected_digest_hex)?);
        let start = PresentationGesturePoint2V1::new(start_x, start_y)
            .map_err(|error| presentation_error(py, error))?;
        let kind = match *kind {
            PyPresentationGestureKindV1::StraightNormalArrow => {
                PresentationGestureKindV1::StraightNormalArrow
            }
            PyPresentationGestureKindV1::StraightEquilibriumArrow => {
                PresentationGestureKindV1::StraightEquilibriumArrow
            }
            PyPresentationGestureKindV1::Plus => PresentationGestureKindV1::Plus,
        };
        let style = match kind {
            PresentationGestureKindV1::StraightNormalArrow => style
                .map(|value| PresentationGestureStyleV1::Normal(value.style))
                .ok_or_else(|| {
                    presentation_error(py, PresentationGestureErrorV1::InvalidGestureStyle)
                })?,
            PresentationGestureKindV1::StraightEquilibriumArrow => {
                if style.is_some() {
                    return Err(presentation_error(
                        py,
                        PresentationGestureErrorV1::InvalidGestureStyle,
                    ));
                }
                PresentationGestureStyleV1::Equilibrium
            }
            PresentationGestureKindV1::Plus => PresentationGestureStyleV1::Plus,
        };
        self.session
            .begin_presentation_creation_gesture_v1(fence, kind, start, style, snap.policy)
            .map(|gesture| PyPresentationCreationGestureV1 {
                gesture: Some(gesture),
            })
            .map_err(|error| presentation_error(py, error))
    }
    fn preview_presentation_creation_gesture_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyPresentationCreationGestureV1>,
        end_x: f64,
        end_y: f64,
    ) -> PyResult<PyPresentationCreationPreviewV1> {
        let end = PresentationGesturePoint2V1::new(end_x, end_y)
            .map_err(|error| presentation_error(py, error))?;
        self.session
            .preview_presentation_creation_gesture_v1(
                gesture.gesture.as_ref().ok_or_else(|| {
                    presentation_error(py, PresentationGestureErrorV1::ReplayedGesture)
                })?,
                end,
            )
            .map_err(|error| presentation_error(py, error))
            .and_then(|value| preview(py, value))
    }
    fn resolve_presentation_creation_gesture_v1(
        &mut self,
        py: Python<'_>,
        mut gesture: PyRefMut<'_, PyPresentationCreationGestureV1>,
        preview: PyRef<'_, PyPresentationCreationPreviewV1>,
    ) -> PyResult<PySessionOperationTransitionRequestV1> {
        let request = self
            .session
            .resolve_presentation_creation_gesture_v1(
                gesture.gesture.as_ref().ok_or_else(|| {
                    presentation_error(py, PresentationGestureErrorV1::ReplayedGesture)
                })?,
                &preview.preview,
            )
            .map_err(|error| presentation_error(py, error))?;
        gesture.gesture = None;
        Ok(PySessionOperationTransitionRequestV1::from_request(request))
    }
}
fn optional_u16(py: Python<'_>, value: Option<&Bound<'_, PyAny>>) -> PyResult<Option<u16>> {
    match value {
        None => Ok(None),
        Some(value) if value.is_instance_of::<pyo3::types::PyBool>() => Err(presentation_error(
            py,
            PresentationGestureErrorV1::InvalidSnapPolicy,
        )),
        Some(value) => value.extract::<u16>().map(Some),
    }
}
fn preview(
    _py: Python<'_>,
    value: PresentationCreationPreviewV1,
) -> PyResult<PyPresentationCreationPreviewV1> {
    Ok(PyPresentationCreationPreviewV1 {
        plan: value.plan().into(),
        preview: value,
    })
}

fn preview_root_from(value: &PresentationPreviewRenderRootV1) -> PyPresentationPreviewRenderRootV1 {
    match value {
        PresentationPreviewRenderRootV1::Vector { vector, bounds } => {
            PyPresentationPreviewRenderRootV1 {
                kind: "vector".to_owned(),
                bounds: (*bounds).into(),
                vector_operations: vector
                    .operations()
                    .iter()
                    .map(super::presentation_render_plan_binding::vector_operation)
                    .collect(),
                plus: None,
            }
        }
        PresentationPreviewRenderRootV1::Plus {
            anchor,
            operation,
            bounds,
            background,
        } => PyPresentationPreviewRenderRootV1 {
            kind: "plus".to_owned(),
            bounds: (*bounds).into(),
            vector_operations: Vec::new(),
            plus: Some(PyPresentationPreviewPlusV1 {
                anchor: (*anchor).into(),
                operation_origin: operation.origin().into(),
                text: operation.runs().iter().map(|run| run.text()).collect(),
                face: operation.face().as_str().to_owned(),
                size: operation.size().get(),
                paint: operation.paint().color().as_str().to_owned(),
                z: operation.z(),
                background: background
                    .as_ref()
                    .map(|paint| paint.color().as_str().to_owned()),
            }),
        },
    }
}
pub(crate) fn digest(value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|v| v.is_ascii_digit() || matches!(v, b'a'..=b'f'))
    {
        return Err(PresentationGestureError::new_err(
            "expected digest must be exactly 64 lowercase hexadecimal characters",
        ));
    }
    let mut result = [0; 32];
    for (i, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        result[i] = (hex(pair[0]) << 4) | hex(pair[1])
    }
    Ok(result)
}
const fn hex(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}
pub(crate) fn presentation_error(py: Python<'_>, error: PresentationGestureErrorV1) -> PyErr {
    let category = match error.category() {
        ferrum_document::PresentationGestureCategoryV1::StaleRevision => {
            PyPresentationGestureCategoryV1::StaleRevision
        }
        ferrum_document::PresentationGestureCategoryV1::StaleDigest => {
            PyPresentationGestureCategoryV1::StaleDigest
        }
        ferrum_document::PresentationGestureCategoryV1::ForeignSession => {
            PyPresentationGestureCategoryV1::ForeignSession
        }
        ferrum_document::PresentationGestureCategoryV1::ReplayedGesture => {
            PyPresentationGestureCategoryV1::ReplayedGesture
        }
        ferrum_document::PresentationGestureCategoryV1::PreviewMismatch => {
            PyPresentationGestureCategoryV1::PreviewMismatch
        }
        ferrum_document::PresentationGestureCategoryV1::NonFinitePoint => {
            PyPresentationGestureCategoryV1::NonFinitePoint
        }
        ferrum_document::PresentationGestureCategoryV1::CollapsedEndpoint => {
            PyPresentationGestureCategoryV1::CollapsedEndpoint
        }
        ferrum_document::PresentationGestureCategoryV1::BelowMinimumLength => {
            PyPresentationGestureCategoryV1::BelowMinimumLength
        }
        ferrum_document::PresentationGestureCategoryV1::ExceedsGeometryLimit => {
            PyPresentationGestureCategoryV1::ExceedsGeometryLimit
        }
        ferrum_document::PresentationGestureCategoryV1::InvalidSnapPolicy => {
            PyPresentationGestureCategoryV1::InvalidSnapPolicy
        }
        ferrum_document::PresentationGestureCategoryV1::InvalidGestureStyle => {
            PyPresentationGestureCategoryV1::InvalidGestureStyle
        }
        ferrum_document::PresentationGestureCategoryV1::SessionConflict => {
            PyPresentationGestureCategoryV1::SessionConflict
        }
    };
    let recovery = match error.recovery() {
        ferrum_document::PresentationGestureRecoveryV1::RefreshAndRestart => {
            PyPresentationGestureRecoveryV1::RefreshAndRestart
        }
        ferrum_document::PresentationGestureRecoveryV1::AdjustEndpoint => {
            PyPresentationGestureRecoveryV1::AdjustEndpoint
        }
        ferrum_document::PresentationGestureRecoveryV1::ChangeToolOrStyle => {
            PyPresentationGestureRecoveryV1::ChangeToolOrStyle
        }
        ferrum_document::PresentationGestureRecoveryV1::RefreshAndReport => {
            PyPresentationGestureRecoveryV1::RefreshAndReport
        }
    };
    let exception = PresentationGestureError::new_err(error.to_string());
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
        "PresentationGestureError",
        module.py().get_type::<PresentationGestureError>(),
    )?;
    module.add_class::<PyPresentationGestureKindV1>()?;
    module.add_class::<PyPresentationGestureCategoryV1>()?;
    module.add_class::<PyPresentationGestureRecoveryV1>()?;
    module.add_class::<PyArrowGestureStyleV1>()?;
    module.add_class::<PyPresentationGestureSnapPolicyV1>()?;
    module.add_class::<PyPresentationCreationGestureV1>()?;
    module.add_class::<PyPresentationCreationPreviewV1>()?;
    module.add_class::<PyPresentationPreviewPlusV1>()?;
    module.add_class::<PyPresentationPreviewRenderRootV1>()?;
    module.add_class::<PyPresentationPreviewRenderPlanV1>()?;
    Ok(())
}
