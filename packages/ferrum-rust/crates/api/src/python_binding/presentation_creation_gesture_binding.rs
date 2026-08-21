//! Opaque PyO3 seam for Rust-owned direct straight normal-arrow authoring.

use super::binding::{PyDocumentSession, PySessionOperationResultV1};
use ferrum_document::{
    ArrowGestureStyleV1, DocumentFenceV1, PresentationCreationGestureV1,
    PresentationCreationPreviewV1, PresentationGestureErrorV1, PresentationGestureKindV1,
    PresentationGesturePoint2V1, PresentationGestureSnapPolicyV1,
};
use pyo3::create_exception;
use pyo3::prelude::*;
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
    PreviewMismatch,
    NonFinitePoint,
    CollapsedEndpoint,
    BelowMinimumLength,
    ExceedsGeometryLimit,
    InvalidSnapPolicy,
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
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "PresentationGestureRootKindV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyPresentationGestureRootKindV1 {
    Arrow,
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
    gesture: PresentationCreationGestureV1,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PresentationGestureOverlayV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyPresentationGestureOverlayV1 {
    #[pyo3(get)]
    start_x: f64,
    #[pyo3(get)]
    start_y: f64,
    #[pyo3(get)]
    end_x: f64,
    #[pyo3(get)]
    end_y: f64,
    #[pyo3(get)]
    axis_start_x: f64,
    #[pyo3(get)]
    axis_start_y: f64,
    #[pyo3(get)]
    axis_end_x: f64,
    #[pyo3(get)]
    axis_end_y: f64,
    #[pyo3(get)]
    head_vertices: Vec<(f64, f64)>,
    #[pyo3(get)]
    left: f64,
    #[pyo3(get)]
    top: f64,
    #[pyo3(get)]
    right: f64,
    #[pyo3(get)]
    bottom: f64,
    #[pyo3(get)]
    width: f64,
    #[pyo3(get)]
    color: String,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PresentationCreationPreviewV1"
)]
pub(crate) struct PyPresentationCreationPreviewV1 {
    preview: PresentationCreationPreviewV1,
    #[pyo3(get)]
    overlay: PyPresentationGestureOverlayV1,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PresentationGestureRootSelectorV1"
)]
pub(crate) struct PyPresentationGestureRootSelectorV1 {
    #[pyo3(get)]
    identifier: String,
    #[pyo3(get)]
    kind: Py<PyPresentationGestureRootKindV1>,
}
#[pyclass(frozen, module = "ferrum_chem", name = "PresentationGestureCommitV1")]
pub(crate) struct PyPresentationGestureCommitV1 {
    #[pyo3(get)]
    root: Py<PyPresentationGestureRootSelectorV1>,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}
#[pymethods]
impl PyDocumentSession {
    #[allow(clippy::too_many_arguments)]
    fn begin_presentation_creation_gesture_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        _kind: PyRef<'_, PyPresentationGestureKindV1>,
        start_x: f64,
        start_y: f64,
        style: PyRef<'_, PyArrowGestureStyleV1>,
        snap: PyRef<'_, PyPresentationGestureSnapPolicyV1>,
    ) -> PyResult<PyPresentationCreationGestureV1> {
        let fence = DocumentFenceV1::new(expected_revision, digest(&expected_digest_hex)?);
        let start = PresentationGesturePoint2V1::new(start_x, start_y)
            .map_err(|error| presentation_error(py, error))?;
        let kind = PresentationGestureKindV1::StraightNormalArrow;
        self.session
            .begin_presentation_creation_gesture_v1(fence, kind, start, style.style, snap.policy)
            .map(|gesture| PyPresentationCreationGestureV1 { gesture })
            .map_err(|error| presentation_error(py, error))
    }
    fn preview_presentation_creation_gesture_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyPresentationCreationGestureV1>,
        end_x: f64,
        end_y: f64,
    ) -> PyResult<PyPresentationCreationPreviewV1> {
        let end = PresentationGesturePoint2V1::new(end_x, end_y)
            .map_err(|error| presentation_error(py, error))?;
        self.session
            .preview_presentation_creation_gesture_v1(&gesture.gesture, end)
            .map(preview)
            .map_err(|error| presentation_error(py, error))
    }
    fn commit_presentation_creation_gesture_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyPresentationCreationGestureV1>,
        preview: PyRef<'_, PyPresentationCreationPreviewV1>,
    ) -> PyResult<PyPresentationGestureCommitV1> {
        self.session
            .commit_presentation_creation_gesture_v1(&gesture.gesture, &preview.preview)
            .map(|commit| {
                let kind = match commit.root().kind() {
                    ferrum_document::PresentationRecordKindV1::Arrow => {
                        PyPresentationGestureRootKindV1::Arrow
                    }
                    _ => unreachable!("generic presentation gesture creates only an Arrow"),
                };
                PyPresentationGestureCommitV1 {
                    root: Py::new(
                        py,
                        PyPresentationGestureRootSelectorV1 {
                            identifier: commit.root().presentation_id().as_str().to_owned(),
                            kind: Py::new(py, kind).expect("root kind allocates"),
                        },
                    )
                    .expect("root selector allocates"),
                    result: commit.result().clone().into(),
                }
            })
            .map_err(|error| presentation_error(py, error))
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
fn preview(value: PresentationCreationPreviewV1) -> PyPresentationCreationPreviewV1 {
    let overlay = value
        .overlay()
        .expect("generic Python gestures are Arrow-only");
    let bounds = overlay.bounds();
    PyPresentationCreationPreviewV1 {
        overlay: PyPresentationGestureOverlayV1 {
            start_x: overlay.start().x(),
            start_y: overlay.start().y(),
            end_x: overlay.end().x(),
            end_y: overlay.end().y(),
            axis_start_x: overlay.axis_start().x(),
            axis_start_y: overlay.axis_start().y(),
            axis_end_x: overlay.axis_end().x(),
            axis_end_y: overlay.axis_end().y(),
            head_vertices: overlay
                .head_vertices()
                .iter()
                .map(|p| (p.x(), p.y()))
                .collect(),
            left: bounds.left(),
            top: bounds.top(),
            right: bounds.right(),
            bottom: bounds.bottom(),
            width: overlay.width(),
            color: overlay.color().to_owned(),
        },
        preview: value,
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
    module.add_class::<PyPresentationGestureRootKindV1>()?;
    module.add_class::<PyArrowGestureStyleV1>()?;
    module.add_class::<PyPresentationGestureSnapPolicyV1>()?;
    module.add_class::<PyPresentationCreationGestureV1>()?;
    module.add_class::<PyPresentationGestureOverlayV1>()?;
    module.add_class::<PyPresentationCreationPreviewV1>()?;
    module.add_class::<PyPresentationGestureRootSelectorV1>()?;
    module.add_class::<PyPresentationGestureCommitV1>()
}
