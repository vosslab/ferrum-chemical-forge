//! Typed PyO3 values for the V3 pointer-probe direct-bond lifecycle.
//!
//! `DirectBondSnapPolicyV1` remains a shared configuration value consumed by
//! V3. `DirectBondCommitCategoryV1` and `DirectBondCommitRecoveryV1` remain
//! the V3 commit-result taxonomy; their V1 names are domain versioning, not a
//! separate interaction lifecycle.

use ferrum_document::DirectBondSnapPolicyV1;
use ferrum_document_render::{
    DirectBondAdmissionCategoryV3, DirectBondAdmissionErrorV3 as RenderDirectBondAdmissionErrorV3,
    DirectBondAdmissionRecoveryV3,
    DirectBondAdmissionRefusalV3 as RenderDirectBondAdmissionRefusalV3, DirectBondGestureV3,
    DirectBondPointerHitStateV3, DirectBondPointerProbeCategoryV3,
    DirectBondPointerProbeErrorV3 as RenderDirectBondPointerProbeErrorV3,
    DirectBondPointerProbeRecoveryV3, DirectBondPointerProbeV3, DirectBondViewportToSceneV3,
};
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
create_exception!(
    ferrum_chem,
    DirectBondGestureError,
    super::binding::DocumentError
);
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
    gesture: Option<DirectBondGestureV3>,
}

impl PyDirectBondGestureV3 {
    pub(super) fn from_renderer_gesture(gesture: DirectBondGestureV3) -> Self {
        Self {
            gesture: Some(gesture),
        }
    }

    pub(super) fn take_for_resolution(&mut self) -> PyResult<DirectBondGestureV3> {
        self.gesture.take().ok_or_else(|| {
            DirectBondGestureError::new_err(
                "direct-bond gesture was already transferred to endpoint resolution",
            )
        })
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
