//! Opaque PyO3 seam for revision-fenced direct normal-bond gestures.

use ferrum_document::{
    CommittedDirectBondGestureV1, CommittedDirectBondGestureV2, DirectBondAdmissionV1,
    DirectBondAdmissionV2, DirectBondCommitErrorV1, DirectBondEndIntentV1,
    DirectBondEndpointIntentV2, DirectBondGestureV1, DirectBondGestureV2, DirectBondPoint2V1,
    DirectBondPreviewV1, DirectBondSnapPolicyV1,
};
pub(super) use ferrum_document::{
    DirectBondAdmissionRefusalV1, DirectBondGestureErrorV1, DocumentFenceV1,
};
use pyo3::create_exception;
use pyo3::prelude::*;

pub(super) use super::{
    binding::PyDocumentBondPresentationV1, document_error_binding::document_object_id,
};
use super::{binding::PySessionOperationResultV1, document_error_binding::RevisionConflictError};

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
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondGestureCategoryV1 {
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
    ExceedsChemistryCapacity,
    UnsupportedChemistryAdmission,
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
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondGestureRecoveryV1 {
    RefreshAndRestart,
    AdjustEndpoint,
    CorrectInput,
    ChangePresentation,
    ReportConflict,
}

/// Closed category for one pure candidate-admission refusal.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondAdmissionCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondAdmissionCategoryV1 {
    ForeignSession,
    StaleRevision,
    StaleDigest,
    UnknownStartAtom,
    UnknownEndAtom,
    UnsupportedPresentation,
    InvalidEndpointInput,
    CollapsedEndpoint,
    SelfLoop,
    CrossMolecule,
    DuplicateBond,
    ExceedsChemistryCapacity,
    UnsupportedChemistryAdmission,
    UnrenderableCandidate,
}

/// Closed category for one receipt-redemption failure.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondCommitCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondCommitCategoryV1 {
    ForeignSession,
    StaleRevision,
    StaleDigest,
    IdentityAllocationFailed,
    ProvisionalTokenUnavailable,
    CandidateApplicationFailed,
    RevisionExhausted,
}

/// Frozen captured snapping input; frontend code cannot supply a mutable mapping.
#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondSnapPolicyV1")]
pub(super) struct PyDirectBondSnapPolicyV1 {
    pub(super) policy: DirectBondSnapPolicyV1,
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
pub(super) struct PyDirectBondEndIntentV1 {
    pub(super) intent: DirectBondEndIntentV1,
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
pub(super) struct PyDirectBondGestureV1 {
    pub(super) gesture: DirectBondGestureV1,
}

/// Frozen scalar overlay geometry supplied by Rust.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DirectBondOverlayV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(super) struct PyDirectBondOverlayV1 {
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
pub(super) struct PyDirectBondPreviewV1 {
    pub(super) preview: DirectBondPreviewV1,
    #[pyo3(get)]
    overlay: PyDirectBondOverlayV1,
}

/// Opaque admitted candidate receipt. Python sees only copied overlay scalars.
#[pyclass(unsendable, module = "ferrum_chem", name = "DirectBondAdmissionV1")]
pub(super) struct PyDirectBondAdmissionV1 {
    pub(super) admission: DirectBondAdmissionV1,
    #[pyo3(get)]
    overlay: PyDirectBondOverlayV1,
}

/// A normal user refusal that callers may display without parsing an exception.
#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondPreviewRefusalV1")]
pub(super) struct PyDirectBondPreviewRefusalV1 {
    #[pyo3(get)]
    pub(super) category: Py<PyDirectBondGestureCategoryV1>,
    #[pyo3(get)]
    pub(super) recovery: Py<PyDirectBondGestureRecoveryV1>,
}

/// A semantic admission refusal that a controller may display directly.
#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondAdmissionRefusalV1")]
pub(super) struct PyDirectBondAdmissionRefusalV1 {
    #[pyo3(get)]
    pub(super) category: Py<PyDirectBondAdmissionCategoryV1>,
    #[pyo3(get)]
    pub(super) recovery: Py<PyDirectBondGestureRecoveryV1>,
}

/// Frozen commit receipt containing the new IDs and authoritative observation.
#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondCommitV1")]
pub(super) struct PyDirectBondCommitV1 {
    #[pyo3(get)]
    bond_identifier: String,
    #[pyo3(get)]
    end_atom_identifier: String,
    #[pyo3(get)]
    created_new_atom: bool,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}

/// Frozen endpoint vocabulary for the two-endpoint direct-bond contract.
#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondEndpointIntentV2")]
pub(super) struct PyDirectBondEndpointIntentV2 {
    pub(super) intent: DirectBondEndpointIntentV2,
}

#[pymethods]
impl PyDirectBondEndpointIntentV2 {
    #[staticmethod]
    fn existing_atom(py: Python<'_>, object_id: String) -> PyResult<Self> {
        Ok(Self {
            intent: DirectBondEndpointIntentV2::ExistingAtom {
                atom: document_object_id(py, object_id)?,
            },
        })
    }
    #[staticmethod]
    fn new_atom_at(py: Python<'_>, x: f64, y: f64) -> PyResult<Self> {
        Ok(Self {
            intent: DirectBondEndpointIntentV2::NewAtomAt {
                raw_point: DirectBondPoint2V1::new(x, y)
                    .map_err(|error| direct_error(py, error))?,
            },
        })
    }
}

#[pyclass(unsendable, module = "ferrum_chem", name = "DirectBondGestureV2")]
pub(super) struct PyDirectBondGestureV2 {
    pub(super) gesture: DirectBondGestureV2,
}

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DirectBondOverlayV2",
    skip_from_py_object
)]
#[derive(Clone)]
pub(super) struct PyDirectBondOverlayV2 {
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
}

#[pyclass(unsendable, module = "ferrum_chem", name = "DirectBondAdmissionV2")]
pub(super) struct PyDirectBondAdmissionV2 {
    pub(super) admission: DirectBondAdmissionV2,
    #[pyo3(get)]
    overlay: PyDirectBondOverlayV2,
}

#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondCommitV2")]
pub(super) struct PyDirectBondCommitV2 {
    #[pyo3(get)]
    bond_identifier: String,
    #[pyo3(get)]
    end_atom_identifier: String,
    #[pyo3(get)]
    created_new_atom: bool,
    #[pyo3(get)]
    second_created_atom_identifier: Option<String>,
    #[pyo3(get)]
    created_new_molecule: bool,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}

pub(super) fn preview_binding(preview: DirectBondPreviewV1) -> PyDirectBondPreviewV1 {
    let overlay = overlay_binding(preview.overlay());
    PyDirectBondPreviewV1 { overlay, preview }
}

pub(super) fn admission_binding(admission: DirectBondAdmissionV1) -> PyDirectBondAdmissionV1 {
    let overlay = overlay_binding(admission.overlay());
    PyDirectBondAdmissionV1 { admission, overlay }
}

pub(super) fn overlay_binding(
    overlay: &ferrum_document::DirectBondOverlayV1,
) -> PyDirectBondOverlayV1 {
    PyDirectBondOverlayV1 {
        start_x: overlay.start().x(),
        start_y: overlay.start().y(),
        end_x: overlay.end().x(),
        end_y: overlay.end().y(),
        presentation: presentation_name(overlay.presentation()).to_owned(),
        endpoint_is_new: overlay.endpoint_is_new(),
    }
}

pub(super) fn commit_binding(value: CommittedDirectBondGestureV1) -> PyDirectBondCommitV1 {
    let created_new_atom = matches!(value, CommittedDirectBondGestureV1::NewEndpoint { .. });
    PyDirectBondCommitV1 {
        bond_identifier: value.bond().as_str().to_owned(),
        end_atom_identifier: value.end_atom().as_str().to_owned(),
        created_new_atom,
        result: value.result().clone().into(),
    }
}

pub(super) fn admission_v2_binding(admission: DirectBondAdmissionV2) -> PyDirectBondAdmissionV2 {
    let overlay = admission.overlay();
    PyDirectBondAdmissionV2 {
        overlay: PyDirectBondOverlayV2 {
            start_x: overlay.start().x(),
            start_y: overlay.start().y(),
            end_x: overlay.end().x(),
            end_y: overlay.end().y(),
            presentation: presentation_name(overlay.presentation()).to_owned(),
        },
        admission,
    }
}

pub(super) fn commit_v2_binding(value: CommittedDirectBondGestureV2) -> PyDirectBondCommitV2 {
    PyDirectBondCommitV2 {
        bond_identifier: value.bond().as_str().to_owned(),
        end_atom_identifier: value.end_atom().as_str().to_owned(),
        created_new_atom: value.created_new_atom(),
        second_created_atom_identifier: value
            .second_created_atom()
            .map(|id| id.as_str().to_owned()),
        created_new_molecule: value.created_new_molecule(),
        result: value.result().clone().into(),
    }
}

pub(super) fn parse_digest(value: &str) -> PyResult<[u8; 32]> {
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
pub(super) fn presentation_name(
    value: ferrum_document::DocumentBondPresentationV1,
) -> &'static str {
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
pub(super) fn category(error: &DirectBondGestureErrorV1) -> PyDirectBondGestureCategoryV1 {
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
        DirectBondGestureErrorV1::ExceedsChemistryCapacity => {
            PyDirectBondGestureCategoryV1::ExceedsChemistryCapacity
        }
        DirectBondGestureErrorV1::UnsupportedChemistryAdmission => {
            PyDirectBondGestureCategoryV1::UnsupportedChemistryAdmission
        }
        DirectBondGestureErrorV1::SessionConflict => PyDirectBondGestureCategoryV1::SessionConflict,
    }
}
pub(super) fn recovery(error: &DirectBondGestureErrorV1) -> PyDirectBondGestureRecoveryV1 {
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
        DirectBondGestureErrorV1::ExceedsChemistryCapacity
        | DirectBondGestureErrorV1::UnsupportedChemistryAdmission => {
            PyDirectBondGestureRecoveryV1::CorrectInput
        }
        DirectBondGestureErrorV1::NonFinitePoint | DirectBondGestureErrorV1::InvalidSnapPolicy => {
            PyDirectBondGestureRecoveryV1::CorrectInput
        }
        DirectBondGestureErrorV1::PreviewMismatch | DirectBondGestureErrorV1::SessionConflict => {
            PyDirectBondGestureRecoveryV1::ReportConflict
        }
    }
}
pub(super) fn direct_error(py: Python<'_>, error: DirectBondGestureErrorV1) -> PyErr {
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

pub(super) fn admission_category(
    error: &DirectBondAdmissionRefusalV1,
) -> PyDirectBondAdmissionCategoryV1 {
    match error {
        DirectBondAdmissionRefusalV1::ForeignSession => {
            PyDirectBondAdmissionCategoryV1::ForeignSession
        }
        DirectBondAdmissionRefusalV1::StaleRevision => {
            PyDirectBondAdmissionCategoryV1::StaleRevision
        }
        DirectBondAdmissionRefusalV1::StaleDigest => PyDirectBondAdmissionCategoryV1::StaleDigest,
        DirectBondAdmissionRefusalV1::UnknownStartAtom => {
            PyDirectBondAdmissionCategoryV1::UnknownStartAtom
        }
        DirectBondAdmissionRefusalV1::UnknownEndAtom => {
            PyDirectBondAdmissionCategoryV1::UnknownEndAtom
        }
        DirectBondAdmissionRefusalV1::UnsupportedPresentation => {
            PyDirectBondAdmissionCategoryV1::UnsupportedPresentation
        }
        DirectBondAdmissionRefusalV1::InvalidEndpointInput => {
            PyDirectBondAdmissionCategoryV1::InvalidEndpointInput
        }
        DirectBondAdmissionRefusalV1::CollapsedEndpoint => {
            PyDirectBondAdmissionCategoryV1::CollapsedEndpoint
        }
        DirectBondAdmissionRefusalV1::SelfLoop => PyDirectBondAdmissionCategoryV1::SelfLoop,
        DirectBondAdmissionRefusalV1::CrossMolecule => {
            PyDirectBondAdmissionCategoryV1::CrossMolecule
        }
        DirectBondAdmissionRefusalV1::DuplicateBond => {
            PyDirectBondAdmissionCategoryV1::DuplicateBond
        }
        DirectBondAdmissionRefusalV1::ExceedsChemistryCapacity => {
            PyDirectBondAdmissionCategoryV1::ExceedsChemistryCapacity
        }
        DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission => {
            PyDirectBondAdmissionCategoryV1::UnsupportedChemistryAdmission
        }
        DirectBondAdmissionRefusalV1::UnrenderableCandidate => {
            PyDirectBondAdmissionCategoryV1::UnrenderableCandidate
        }
    }
}

pub(super) fn admission_recovery(
    error: &DirectBondAdmissionRefusalV1,
) -> PyDirectBondGestureRecoveryV1 {
    match error {
        DirectBondAdmissionRefusalV1::UnknownStartAtom
        | DirectBondAdmissionRefusalV1::UnknownEndAtom => {
            PyDirectBondGestureRecoveryV1::RefreshAndRestart
        }
        DirectBondAdmissionRefusalV1::UnsupportedPresentation => {
            PyDirectBondGestureRecoveryV1::ChangePresentation
        }
        DirectBondAdmissionRefusalV1::InvalidEndpointInput => {
            PyDirectBondGestureRecoveryV1::CorrectInput
        }
        DirectBondAdmissionRefusalV1::CollapsedEndpoint
        | DirectBondAdmissionRefusalV1::SelfLoop
        | DirectBondAdmissionRefusalV1::CrossMolecule
        | DirectBondAdmissionRefusalV1::DuplicateBond => {
            PyDirectBondGestureRecoveryV1::AdjustEndpoint
        }
        DirectBondAdmissionRefusalV1::ExceedsChemistryCapacity
        | DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission
        | DirectBondAdmissionRefusalV1::UnrenderableCandidate => {
            PyDirectBondGestureRecoveryV1::CorrectInput
        }
        DirectBondAdmissionRefusalV1::ForeignSession
        | DirectBondAdmissionRefusalV1::StaleRevision
        | DirectBondAdmissionRefusalV1::StaleDigest => {
            PyDirectBondGestureRecoveryV1::RefreshAndRestart
        }
    }
}

pub(super) fn admission_protocol_error(
    py: Python<'_>,
    error: DirectBondAdmissionRefusalV1,
) -> PyErr {
    let category = admission_category(&error);
    let recovery = admission_recovery(&error);
    let exception = match error {
        DirectBondAdmissionRefusalV1::StaleRevision | DirectBondAdmissionRefusalV1::StaleDigest => {
            RevisionConflictError::new_err(error.to_string())
        }
        DirectBondAdmissionRefusalV1::ForeignSession => {
            DirectBondGestureError::new_err(error.to_string())
        }
        _ => unreachable!("semantic admission refusals are values"),
    };
    let instance = exception.value(py);
    instance
        .setattr(
            "category",
            Py::new(py, category).expect("admission category enum allocates"),
        )
        .expect("admission category attaches");
    instance
        .setattr(
            "recovery",
            Py::new(py, recovery).expect("admission recovery enum allocates"),
        )
        .expect("admission recovery attaches");
    exception
}

pub(super) fn admission_commit_error(py: Python<'_>, error: DirectBondCommitErrorV1) -> PyErr {
    let category = match error {
        DirectBondCommitErrorV1::ForeignSession => PyDirectBondCommitCategoryV1::ForeignSession,
        DirectBondCommitErrorV1::StaleRevision => PyDirectBondCommitCategoryV1::StaleRevision,
        DirectBondCommitErrorV1::StaleDigest => PyDirectBondCommitCategoryV1::StaleDigest,
        DirectBondCommitErrorV1::IdentityAllocationFailed => {
            PyDirectBondCommitCategoryV1::IdentityAllocationFailed
        }
        DirectBondCommitErrorV1::ProvisionalTokenUnavailable => {
            PyDirectBondCommitCategoryV1::ProvisionalTokenUnavailable
        }
        DirectBondCommitErrorV1::CandidateApplicationFailed => {
            PyDirectBondCommitCategoryV1::CandidateApplicationFailed
        }
        DirectBondCommitErrorV1::RevisionExhausted => {
            PyDirectBondCommitCategoryV1::RevisionExhausted
        }
    };
    let exception = match error {
        DirectBondCommitErrorV1::StaleRevision | DirectBondCommitErrorV1::StaleDigest => {
            RevisionConflictError::new_err(error.to_string())
        }
        _ => DirectBondGestureError::new_err(error.to_string()),
    };
    let instance = exception.value(py);
    instance
        .setattr(
            "category",
            Py::new(py, category).expect("commit category allocates"),
        )
        .expect("commit category attaches");
    instance
        .setattr("recovery", "refresh_and_restart")
        .expect("commit recovery attaches");
    exception
}
