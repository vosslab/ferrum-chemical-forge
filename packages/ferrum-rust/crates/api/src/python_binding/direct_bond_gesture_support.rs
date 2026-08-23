//! Typed PyO3 values for the V3 pointer-probe direct-bond lifecycle.
//!
//! `DirectBondSnapPolicyV1` remains a shared configuration value consumed by
//! V3. `DirectBondCommitCategoryV1` and `DirectBondCommitRecoveryV1` remain
//! the V3 commit-result taxonomy; their V1 names are domain versioning, not a
//! separate interaction lifecycle.

use ferrum_document::DirectBondSnapPolicyV1;
use ferrum_document_render::{
    CommittedDirectBondGestureV3, DirectBondAdmissionCategoryV3,
    DirectBondAdmissionErrorV3 as RenderDirectBondAdmissionErrorV3, DirectBondAdmissionRecoveryV3,
    DirectBondAdmissionRefusalV3 as RenderDirectBondAdmissionRefusalV3, DirectBondAdmissionV3,
    DirectBondCommitCategoryV1, DirectBondCommitError as RenderDirectBondCommitError,
    DirectBondCommitRecoveryV1, DirectBondGestureV3, DirectBondPointerHitStateV3,
    DirectBondPointerProbeCategoryV3,
    DirectBondPointerProbeErrorV3 as RenderDirectBondPointerProbeErrorV3,
    DirectBondPointerProbeRecoveryV3, DirectBondPointerProbeV3, DirectBondViewportToSceneV3,
};
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::binding::PySessionOperationResultV1;

create_exception!(
    ferrum_chem,
    DirectBondGestureError,
    super::binding::DocumentError
);
create_exception!(ferrum_chem, DirectBondCommitError, DirectBondGestureError);
create_exception!(
    ferrum_chem,
    DirectBondAdmissionRefusalV3,
    DirectBondGestureError
);
create_exception!(
    ferrum_chem,
    DirectBondPointerProbeErrorV3,
    DirectBondGestureError
);

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondPointerProbeCategoryV3",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondPointerProbeCategoryV3 {
    NonFiniteScenePoint,
    MalformedTransform,
    InvalidHitEvidence,
    UnknownDirectAtom,
    AmbiguousAtom,
    StaleRevision,
    StaleDigest,
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondPointerProbeRecoveryV3",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondPointerProbeRecoveryV3 {
    CorrectInput,
    AdjustEndpoint,
    RefreshAndRestart,
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondAdmissionCategoryV3",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondAdmissionCategoryV3 {
    ForeignSession,
    ReplayedGesture,
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

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondAdmissionRecoveryV3",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondAdmissionRecoveryV3 {
    RefreshAndRestart,
    AdjustEndpoint,
    ChangePresentation,
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondPointerHitStateV3",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondPointerHitStateV3 {
    None,
    UniqueAtom,
    AmbiguousAtom,
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondCommitCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondCommitCategoryV1 {
    ForeignSession,
    ReplayedReceipt,
    UnrenderableCandidate,
    StaleRevision,
    StaleDigest,
    IdentityAllocationFailed,
    ProvisionalTokenUnavailable,
    CandidateApplicationFailed,
    RevisionExhausted,
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondCommitRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondCommitRecoveryV1 {
    RefreshAndRestart,
    ChangePresentation,
    ReportConflict,
}

#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondSnapPolicyV1")]
pub(super) struct PyDirectBondSnapPolicyV1 {
    pub(super) policy: DirectBondSnapPolicyV1,
}
#[pymethods]
impl PyDirectBondSnapPolicyV1 {
    #[new]
    #[pyo3(signature = (hex_grid=false, angle_increment_degrees=None, fixed_length_pt=None))]
    fn new(
        hex_grid: bool,
        angle_increment_degrees: Option<u16>,
        fixed_length_pt: Option<f64>,
    ) -> PyResult<Self> {
        DirectBondSnapPolicyV1::new(hex_grid, angle_increment_degrees, fixed_length_pt)
            .map(|policy| Self { policy })
            .map_err(|error| PyValueError::new_err(error.to_string()))
    }
}

#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondViewportToSceneV3")]
pub(super) struct PyDirectBondViewportToSceneV3 {
    transform: DirectBondViewportToSceneV3,
}
#[pymethods]
impl PyDirectBondViewportToSceneV3 {
    #[new]
    fn new(
        py: Python<'_>,
        m11: f64,
        m12: f64,
        m21: f64,
        m22: f64,
        dx: f64,
        dy: f64,
    ) -> PyResult<Self> {
        DirectBondViewportToSceneV3::new(m11, m12, m21, m22, dx, dy)
            .map(|transform| Self { transform })
            .map_err(|error| pointer_probe_error(py, error))
    }
}

#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondPointerProbeV3")]
pub(super) struct PyDirectBondPointerProbeV3 {
    pub(super) probe: DirectBondPointerProbeV3,
}
#[pymethods]
impl PyDirectBondPointerProbeV3 {
    #[new]
    #[pyo3(signature = (scene_x, scene_y, viewport_to_scene, direct_hit_state, direct_atom_id=None))]
    fn new(
        py: Python<'_>,
        scene_x: f64,
        scene_y: f64,
        viewport_to_scene: PyRef<'_, PyDirectBondViewportToSceneV3>,
        direct_hit_state: PyRef<'_, PyDirectBondPointerHitStateV3>,
        direct_atom_id: Option<String>,
    ) -> PyResult<Self> {
        let state = match *direct_hit_state {
            PyDirectBondPointerHitStateV3::None => DirectBondPointerHitStateV3::None,
            PyDirectBondPointerHitStateV3::UniqueAtom => DirectBondPointerHitStateV3::UniqueAtom,
            PyDirectBondPointerHitStateV3::AmbiguousAtom => {
                DirectBondPointerHitStateV3::AmbiguousAtom
            }
        };
        DirectBondPointerProbeV3::new(
            scene_x,
            scene_y,
            viewport_to_scene.transform,
            state,
            direct_atom_id,
        )
        .map(|probe| Self { probe })
        .map_err(|error| pointer_probe_error(py, error))
    }
}

#[pyclass(unsendable, module = "ferrum_chem", name = "DirectBondGestureV3")]
pub(super) struct PyDirectBondGestureV3 {
    pub(super) gesture: DirectBondGestureV3,
}

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DirectBondOverlayV3",
    skip_from_py_object
)]
pub(super) struct PyDirectBondOverlayV3 {
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
    render_operations: Py<PyTuple>,
}
#[pymethods]
impl PyDirectBondOverlayV3 {
    #[getter]
    fn render_operations(&self, py: Python<'_>) -> Py<PyTuple> {
        self.render_operations.clone_ref(py)
    }
}

#[pyclass(unsendable, module = "ferrum_chem", name = "DirectBondAdmissionV3")]
pub(super) struct PyDirectBondAdmissionV3 {
    pub(super) admission: DirectBondAdmissionV3,
    #[pyo3(get)]
    overlay: Py<PyDirectBondOverlayV3>,
}

#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondCommitV3")]
pub(super) struct PyDirectBondCommitV3 {
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

pub(super) fn admission_v3_binding(
    py: Python<'_>,
    admission: DirectBondAdmissionV3,
) -> PyResult<PyDirectBondAdmissionV3> {
    let overlay = admission.overlay();
    let values = overlay
        .operations()
        .iter()
        .map(|operation| super::render_binding::operation_from(py, operation))
        .collect::<PyResult<Vec<_>>>()?;
    let overlay = Py::new(
        py,
        PyDirectBondOverlayV3 {
            start_x: overlay.start_x(),
            start_y: overlay.start_y(),
            end_x: overlay.end_x(),
            end_y: overlay.end_y(),
            presentation: presentation_name(overlay.presentation()).to_owned(),
            render_operations: super::render_binding::frozen_tuple(py, &values)?,
        },
    )?;
    Ok(PyDirectBondAdmissionV3 { admission, overlay })
}

pub(super) fn commit_v3_binding(value: CommittedDirectBondGestureV3) -> PyDirectBondCommitV3 {
    PyDirectBondCommitV3 {
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

pub(super) fn pointer_probe_error(
    py: Python<'_>,
    error: RenderDirectBondPointerProbeErrorV3,
) -> PyErr {
    let category = match error.category() {
        DirectBondPointerProbeCategoryV3::NonFiniteScenePoint => {
            PyDirectBondPointerProbeCategoryV3::NonFiniteScenePoint
        }
        DirectBondPointerProbeCategoryV3::MalformedTransform => {
            PyDirectBondPointerProbeCategoryV3::MalformedTransform
        }
        DirectBondPointerProbeCategoryV3::InvalidHitEvidence => {
            PyDirectBondPointerProbeCategoryV3::InvalidHitEvidence
        }
        DirectBondPointerProbeCategoryV3::UnknownDirectAtom => {
            PyDirectBondPointerProbeCategoryV3::UnknownDirectAtom
        }
        DirectBondPointerProbeCategoryV3::AmbiguousAtom => {
            PyDirectBondPointerProbeCategoryV3::AmbiguousAtom
        }
        DirectBondPointerProbeCategoryV3::StaleRevision => {
            PyDirectBondPointerProbeCategoryV3::StaleRevision
        }
        DirectBondPointerProbeCategoryV3::StaleDigest => {
            PyDirectBondPointerProbeCategoryV3::StaleDigest
        }
    };
    let recovery = match error.recovery() {
        DirectBondPointerProbeRecoveryV3::CorrectInput => {
            PyDirectBondPointerProbeRecoveryV3::CorrectInput
        }
        DirectBondPointerProbeRecoveryV3::AdjustEndpoint => {
            PyDirectBondPointerProbeRecoveryV3::AdjustEndpoint
        }
        DirectBondPointerProbeRecoveryV3::RefreshAndRestart => {
            PyDirectBondPointerProbeRecoveryV3::RefreshAndRestart
        }
    };
    let exception = DirectBondPointerProbeErrorV3::new_err(error.to_string());
    let value = exception.value(py);
    value
        .setattr("category", Py::new(py, category).expect("closed category"))
        .expect("category attaches");
    value
        .setattr("recovery", Py::new(py, recovery).expect("closed recovery"))
        .expect("recovery attaches");
    exception
}

pub(super) fn admission_error(py: Python<'_>, error: RenderDirectBondAdmissionErrorV3) -> PyErr {
    match error {
        RenderDirectBondAdmissionErrorV3::PointerProbe(error) => pointer_probe_error(py, error),
        RenderDirectBondAdmissionErrorV3::Refusal(error) => admission_refusal_error(py, error),
        RenderDirectBondAdmissionErrorV3::DocumentGesture(error) => {
            DirectBondGestureError::new_err(error.to_string())
        }
    }
}

fn admission_refusal_error(py: Python<'_>, error: RenderDirectBondAdmissionRefusalV3) -> PyErr {
    let category = match error.category() {
        DirectBondAdmissionCategoryV3::ForeignSession => {
            PyDirectBondAdmissionCategoryV3::ForeignSession
        }
        DirectBondAdmissionCategoryV3::ReplayedGesture => {
            PyDirectBondAdmissionCategoryV3::ReplayedGesture
        }
        DirectBondAdmissionCategoryV3::StaleRevision => {
            PyDirectBondAdmissionCategoryV3::StaleRevision
        }
        DirectBondAdmissionCategoryV3::StaleDigest => PyDirectBondAdmissionCategoryV3::StaleDigest,
        DirectBondAdmissionCategoryV3::UnknownStartAtom => {
            PyDirectBondAdmissionCategoryV3::UnknownStartAtom
        }
        DirectBondAdmissionCategoryV3::UnknownEndAtom => {
            PyDirectBondAdmissionCategoryV3::UnknownEndAtom
        }
        DirectBondAdmissionCategoryV3::UnsupportedPresentation => {
            PyDirectBondAdmissionCategoryV3::UnsupportedPresentation
        }
        DirectBondAdmissionCategoryV3::InvalidEndpointInput => {
            PyDirectBondAdmissionCategoryV3::InvalidEndpointInput
        }
        DirectBondAdmissionCategoryV3::CollapsedEndpoint => {
            PyDirectBondAdmissionCategoryV3::CollapsedEndpoint
        }
        DirectBondAdmissionCategoryV3::SelfLoop => PyDirectBondAdmissionCategoryV3::SelfLoop,
        DirectBondAdmissionCategoryV3::CrossMolecule => {
            PyDirectBondAdmissionCategoryV3::CrossMolecule
        }
        DirectBondAdmissionCategoryV3::DuplicateBond => {
            PyDirectBondAdmissionCategoryV3::DuplicateBond
        }
        DirectBondAdmissionCategoryV3::ExceedsChemistryCapacity => {
            PyDirectBondAdmissionCategoryV3::ExceedsChemistryCapacity
        }
        DirectBondAdmissionCategoryV3::UnsupportedChemistryAdmission => {
            PyDirectBondAdmissionCategoryV3::UnsupportedChemistryAdmission
        }
        DirectBondAdmissionCategoryV3::UnrenderableCandidate => {
            PyDirectBondAdmissionCategoryV3::UnrenderableCandidate
        }
    };
    let recovery = match error.recovery() {
        DirectBondAdmissionRecoveryV3::RefreshAndRestart => {
            PyDirectBondAdmissionRecoveryV3::RefreshAndRestart
        }
        DirectBondAdmissionRecoveryV3::AdjustEndpoint => {
            PyDirectBondAdmissionRecoveryV3::AdjustEndpoint
        }
        DirectBondAdmissionRecoveryV3::ChangePresentation => {
            PyDirectBondAdmissionRecoveryV3::ChangePresentation
        }
    };
    let exception = DirectBondAdmissionRefusalV3::new_err(error.to_string());
    let value = exception.value(py);
    value
        .setattr(
            "category",
            Py::new(py, category).expect("closed admission category"),
        )
        .expect("admission category attaches");
    value
        .setattr(
            "recovery",
            Py::new(py, recovery).expect("closed admission recovery"),
        )
        .expect("admission recovery attaches");
    exception
}

pub(super) fn commit_error(py: Python<'_>, error: RenderDirectBondCommitError) -> PyErr {
    let category = match error.category() {
        DirectBondCommitCategoryV1::ForeignSession => PyDirectBondCommitCategoryV1::ForeignSession,
        DirectBondCommitCategoryV1::ReplayedReceipt => {
            PyDirectBondCommitCategoryV1::ReplayedReceipt
        }
        DirectBondCommitCategoryV1::UnrenderableCandidate => {
            PyDirectBondCommitCategoryV1::UnrenderableCandidate
        }
        DirectBondCommitCategoryV1::StaleRevision => PyDirectBondCommitCategoryV1::StaleRevision,
        DirectBondCommitCategoryV1::StaleDigest => PyDirectBondCommitCategoryV1::StaleDigest,
        DirectBondCommitCategoryV1::IdentityAllocationFailed => {
            PyDirectBondCommitCategoryV1::IdentityAllocationFailed
        }
        DirectBondCommitCategoryV1::ProvisionalTokenUnavailable => {
            PyDirectBondCommitCategoryV1::ProvisionalTokenUnavailable
        }
        DirectBondCommitCategoryV1::CandidateApplicationFailed => {
            PyDirectBondCommitCategoryV1::CandidateApplicationFailed
        }
        DirectBondCommitCategoryV1::RevisionExhausted => {
            PyDirectBondCommitCategoryV1::RevisionExhausted
        }
    };
    let recovery = match error.recovery() {
        DirectBondCommitRecoveryV1::RefreshAndRestart => {
            PyDirectBondCommitRecoveryV1::RefreshAndRestart
        }
        DirectBondCommitRecoveryV1::ChangePresentation => {
            PyDirectBondCommitRecoveryV1::ChangePresentation
        }
        DirectBondCommitRecoveryV1::ReportConflict => PyDirectBondCommitRecoveryV1::ReportConflict,
    };
    let exception = DirectBondCommitError::new_err(error.to_string());
    let value = exception.value(py);
    value
        .setattr(
            "category",
            Py::new(py, category).expect("closed commit category"),
        )
        .expect("commit category attaches");
    value
        .setattr(
            "recovery",
            Py::new(py, recovery).expect("closed commit recovery"),
        )
        .expect("commit recovery attaches");
    exception
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
        ferrum_document::DocumentBondPresentationV1::SolidWedge => "solid_wedge",
        ferrum_document::DocumentBondPresentationV1::HashedWedge => "hashed_wedge",
    }
}
