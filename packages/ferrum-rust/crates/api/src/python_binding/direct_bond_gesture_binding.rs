//! Opaque PyO3 seam for revision-fenced direct normal-bond gestures.

use ferrum_document::{
    CommittedDirectBondGestureV1, DirectBondEndIntentV1, DirectBondGestureErrorV1,
    DirectBondGestureV1, DirectBondPoint2V1, DirectBondPreviewV1, DirectBondSnapPolicyV1,
    DocumentFenceV1,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::{
    binding::{PyDocumentBondPresentationV1, PyDocumentSession, PySessionOperationResultV1},
    document_error_binding::{RevisionConflictError, document_object_id},
};

create_exception!(
    ferrum_chem,
    DirectBondGestureError,
    super::binding::DocumentError
);

/// Closed direct-bond outcome vocabulary for controller decisions.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondGestureCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyDirectBondGestureCategoryV1 {
    StaleRevision,
    StaleDigest,
    ForeignSession,
    UnknownStartAtom,
    UnknownEndAtom,
    UnsupportedPresentation,
    SelfLoop,
    CrossMolecule,
    DuplicateBond,
    NonFinitePoint,
    InvalidSnapPolicy,
    CollapsedEndpoint,
    PreviewMismatch,
    UnrenderableCandidate,
    SessionConflict,
}

/// Closed recovery disposition paired with every direct-bond category.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondGestureRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyDirectBondGestureRecoveryV1 {
    RefreshAndRestart,
    AdjustEndpoint,
    CorrectInput,
    ChangePresentation,
    ReportConflict,
}

/// Frozen captured snapping input; frontend code cannot supply a mutable mapping.
#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondSnapPolicyV1")]
pub(crate) struct PyDirectBondSnapPolicyV1 {
    policy: DirectBondSnapPolicyV1,
}

#[pymethods]
impl PyDirectBondSnapPolicyV1 {
    #[new]
    #[pyo3(signature = (hex_grid=false, angle_increment_degrees=None, fixed_length_pt=None))]
    fn new(
        py: Python<'_>,
        hex_grid: bool,
        angle_increment_degrees: Option<u16>,
        fixed_length_pt: Option<f64>,
    ) -> PyResult<Self> {
        DirectBondSnapPolicyV1::new(hex_grid, angle_increment_degrees, fixed_length_pt)
            .map(|policy| Self { policy })
            .map_err(|error| direct_error(py, error))
    }
}

/// Frozen endpoint input for preview planning.
#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondEndIntentV1")]
pub(crate) struct PyDirectBondEndIntentV1 {
    intent: DirectBondEndIntentV1,
}

#[pymethods]
impl PyDirectBondEndIntentV1 {
    #[staticmethod]
    fn existing_atom(py: Python<'_>, object_id: String) -> PyResult<Self> {
        Ok(Self {
            intent: DirectBondEndIntentV1::ExistingAtom {
                atom: document_object_id(py, object_id)?,
            },
        })
    }
    #[staticmethod]
    fn new_atom_at(py: Python<'_>, x: f64, y: f64) -> PyResult<Self> {
        Ok(Self {
            intent: DirectBondEndIntentV1::NewAtomAt {
                raw_point: DirectBondPoint2V1::new(x, y)
                    .map_err(|error| direct_error(py, error))?,
            },
        })
    }
}

/// Opaque uncommitted gesture owned by the Python call site.
#[pyclass(unsendable, module = "ferrum_chem", name = "DirectBondGestureV1")]
pub(crate) struct PyDirectBondGestureV1 {
    gesture: DirectBondGestureV1,
}

/// Frozen scalar overlay geometry supplied by Rust.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DirectBondOverlayV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDirectBondOverlayV1 {
    #[pyo3(get)]
    start_x: f64,
    #[pyo3(get)]
    start_y: f64,
    #[pyo3(get)]
    end_x: f64,
    #[pyo3(get)]
    end_y: f64,
    #[pyo3(get)]
    presentation: String,
    #[pyo3(get)]
    endpoint_is_new: bool,
}

/// Opaque preview retaining its exact checked gesture.
#[pyclass(unsendable, module = "ferrum_chem", name = "DirectBondPreviewV1")]
pub(crate) struct PyDirectBondPreviewV1 {
    preview: DirectBondPreviewV1,
    #[pyo3(get)]
    overlay: PyDirectBondOverlayV1,
}

/// A normal user refusal that callers may display without parsing an exception.
#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondPreviewRefusalV1")]
pub(crate) struct PyDirectBondPreviewRefusalV1 {
    #[pyo3(get)]
    category: Py<PyDirectBondGestureCategoryV1>,
    #[pyo3(get)]
    recovery: Py<PyDirectBondGestureRecoveryV1>,
}

/// Frozen commit receipt containing the new IDs and authoritative observation.
#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondCommitV1")]
pub(crate) struct PyDirectBondCommitV1 {
    #[pyo3(get)]
    bond_identifier: String,
    #[pyo3(get)]
    end_atom_identifier: String,
    #[pyo3(get)]
    created_new_atom: bool,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}

#[pymethods]
impl PyDocumentSession {
    #[allow(clippy::too_many_arguments)]
    fn begin_direct_bond_gesture_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        start_atom_object_id: String,
        presentation: PyRef<'_, PyDocumentBondPresentationV1>,
        new_atom_element: String,
        snap: PyRef<'_, PyDirectBondSnapPolicyV1>,
    ) -> PyResult<PyDirectBondGestureV1> {
        let fence = DocumentFenceV1::new(expected_revision, parse_digest(&expected_digest_hex)?);
        let start_atom = document_object_id(py, start_atom_object_id)?;
        self.session
            .begin_direct_bond_gesture_v1(
                fence,
                start_atom,
                (*presentation).into(),
                new_atom_element,
                snap.policy,
            )
            .map(|gesture| PyDirectBondGestureV1 { gesture })
            .map_err(|error| direct_error(py, error))
    }

    fn preview_direct_bond_gesture_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyDirectBondGestureV1>,
        end: PyRef<'_, PyDirectBondEndIntentV1>,
    ) -> PyResult<Py<PyAny>> {
        match self
            .session
            .preview_direct_bond_gesture_v1(&gesture.gesture, end.intent.clone())
        {
            Ok(preview) => Py::new(py, preview_binding(preview)).map(|value| value.into_any()),
            Err(
                error @ (DirectBondGestureErrorV1::SelfLoop
                | DirectBondGestureErrorV1::CrossMolecule
                | DirectBondGestureErrorV1::DuplicateBond),
            ) => Py::new(
                py,
                PyDirectBondPreviewRefusalV1 {
                    category: Py::new(py, category(&error))?,
                    recovery: Py::new(py, recovery(&error))?,
                },
            )
            .map(|value| value.into_any()),
            Err(error) => Err(direct_error(py, error)),
        }
    }

    fn commit_direct_bond_gesture_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyDirectBondGestureV1>,
        preview: PyRef<'_, PyDirectBondPreviewV1>,
    ) -> PyResult<PyDirectBondCommitV1> {
        self.session
            .commit_direct_bond_gesture_v1(&gesture.gesture, &preview.preview)
            .map(commit_binding)
            .map_err(|error| direct_error(py, error))
    }
}

fn preview_binding(preview: DirectBondPreviewV1) -> PyDirectBondPreviewV1 {
    let overlay = preview.overlay();
    PyDirectBondPreviewV1 {
        overlay: PyDirectBondOverlayV1 {
            start_x: overlay.start().x(),
            start_y: overlay.start().y(),
            end_x: overlay.end().x(),
            end_y: overlay.end().y(),
            presentation: presentation_name(overlay.presentation()).to_owned(),
            endpoint_is_new: overlay.endpoint_is_new(),
        },
        preview,
    }
}

fn commit_binding(value: CommittedDirectBondGestureV1) -> PyDirectBondCommitV1 {
    let created_new_atom = matches!(value, CommittedDirectBondGestureV1::NewEndpoint { .. });
    PyDirectBondCommitV1 {
        bond_identifier: value.bond().as_str().to_owned(),
        end_atom_identifier: value.end_atom().as_str().to_owned(),
        created_new_atom,
        result: value.result().clone().into(),
    }
}

fn parse_digest(value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(DirectBondGestureError::new_err(
            "expected digest must be exactly 64 lowercase hexadecimal characters",
        ));
    }
    let mut digest = [0; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        digest[index] = (hex_value(pair[0]) << 4) | hex_value(pair[1]);
    }
    Ok(digest)
}
const fn hex_value(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}
fn presentation_name(value: ferrum_document::DocumentBondPresentationV1) -> &'static str {
    match value {
        ferrum_document::DocumentBondPresentationV1::Normal(
            ferrum_document::DocumentBondOrderV1::Single,
        ) => "normal_single",
        ferrum_document::DocumentBondPresentationV1::Normal(
            ferrum_document::DocumentBondOrderV1::Double,
        ) => "normal_double",
        ferrum_document::DocumentBondPresentationV1::Normal(
            ferrum_document::DocumentBondOrderV1::Triple,
        ) => "normal_triple",
        _ => "unsupported",
    }
}
fn category(error: &DirectBondGestureErrorV1) -> PyDirectBondGestureCategoryV1 {
    match error {
        DirectBondGestureErrorV1::StaleRevision => PyDirectBondGestureCategoryV1::StaleRevision,
        DirectBondGestureErrorV1::StaleDigest => PyDirectBondGestureCategoryV1::StaleDigest,
        DirectBondGestureErrorV1::ForeignSession => PyDirectBondGestureCategoryV1::ForeignSession,
        DirectBondGestureErrorV1::UnknownStartAtom => {
            PyDirectBondGestureCategoryV1::UnknownStartAtom
        }
        DirectBondGestureErrorV1::UnknownEndAtom => PyDirectBondGestureCategoryV1::UnknownEndAtom,
        DirectBondGestureErrorV1::UnsupportedPresentation => {
            PyDirectBondGestureCategoryV1::UnsupportedPresentation
        }
        DirectBondGestureErrorV1::SelfLoop => PyDirectBondGestureCategoryV1::SelfLoop,
        DirectBondGestureErrorV1::CrossMolecule => PyDirectBondGestureCategoryV1::CrossMolecule,
        DirectBondGestureErrorV1::DuplicateBond => PyDirectBondGestureCategoryV1::DuplicateBond,
        DirectBondGestureErrorV1::NonFinitePoint => PyDirectBondGestureCategoryV1::NonFinitePoint,
        DirectBondGestureErrorV1::InvalidSnapPolicy => {
            PyDirectBondGestureCategoryV1::InvalidSnapPolicy
        }
        DirectBondGestureErrorV1::CollapsedEndpoint => {
            PyDirectBondGestureCategoryV1::CollapsedEndpoint
        }
        DirectBondGestureErrorV1::PreviewMismatch => PyDirectBondGestureCategoryV1::PreviewMismatch,
        DirectBondGestureErrorV1::UnrenderableCandidate => {
            PyDirectBondGestureCategoryV1::UnrenderableCandidate
        }
        DirectBondGestureErrorV1::SessionConflict => PyDirectBondGestureCategoryV1::SessionConflict,
    }
}
fn recovery(error: &DirectBondGestureErrorV1) -> PyDirectBondGestureRecoveryV1 {
    match error {
        DirectBondGestureErrorV1::StaleRevision
        | DirectBondGestureErrorV1::StaleDigest
        | DirectBondGestureErrorV1::ForeignSession
        | DirectBondGestureErrorV1::UnknownStartAtom
        | DirectBondGestureErrorV1::UnknownEndAtom => {
            PyDirectBondGestureRecoveryV1::RefreshAndRestart
        }
        DirectBondGestureErrorV1::SelfLoop
        | DirectBondGestureErrorV1::CrossMolecule
        | DirectBondGestureErrorV1::DuplicateBond
        | DirectBondGestureErrorV1::CollapsedEndpoint => {
            PyDirectBondGestureRecoveryV1::AdjustEndpoint
        }
        DirectBondGestureErrorV1::UnsupportedPresentation
        | DirectBondGestureErrorV1::UnrenderableCandidate => {
            PyDirectBondGestureRecoveryV1::ChangePresentation
        }
        DirectBondGestureErrorV1::NonFinitePoint | DirectBondGestureErrorV1::InvalidSnapPolicy => {
            PyDirectBondGestureRecoveryV1::CorrectInput
        }
        DirectBondGestureErrorV1::PreviewMismatch | DirectBondGestureErrorV1::SessionConflict => {
            PyDirectBondGestureRecoveryV1::ReportConflict
        }
    }
}
fn direct_error(py: Python<'_>, error: DirectBondGestureErrorV1) -> PyErr {
    let category = category(&error);
    let recovery = recovery(&error);
    let exception = match error {
        DirectBondGestureErrorV1::StaleRevision | DirectBondGestureErrorV1::StaleDigest => {
            RevisionConflictError::new_err(error.to_string())
        }
        _ => DirectBondGestureError::new_err(error.to_string()),
    };
    let instance = exception.value(py);
    instance
        .setattr(
            "category",
            Py::new(py, category).expect("category enum allocates"),
        )
        .expect("direct-bond error category attaches");
    instance
        .setattr(
            "recovery",
            Py::new(py, recovery).expect("recovery enum allocates"),
        )
        .expect("direct-bond error recovery attaches");
    exception
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DirectBondGestureError",
        module.py().get_type::<DirectBondGestureError>(),
    )?;
    module.add_class::<PyDirectBondGestureCategoryV1>()?;
    module.add_class::<PyDirectBondGestureRecoveryV1>()?;
    module.add_class::<PyDirectBondSnapPolicyV1>()?;
    module.add_class::<PyDirectBondEndIntentV1>()?;
    module.add_class::<PyDirectBondGestureV1>()?;
    module.add_class::<PyDirectBondOverlayV1>()?;
    module.add_class::<PyDirectBondPreviewV1>()?;
    module.add_class::<PyDirectBondPreviewRefusalV1>()?;
    module.add_class::<PyDirectBondCommitV1>()?;
    Ok(())
}
