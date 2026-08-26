//! Typed PyO3 values for the pointer-probe direct-bond lifecycle.
//!
//! `DirectBondSnapPolicyV1` remains a shared configuration value consumed by
//! the gesture. `DirectBondCommitCategoryV1` and `DirectBondCommitRecoveryV1` remain
//! the commit-result taxonomy; their V1 names are domain versioning, not a
//! separate interaction lifecycle.

use ferrum_document::DirectBondSnapPolicyV1;
use ferrum_document_render::{
    DirectBondAdmissionCategory, DirectBondAdmissionError as RenderDirectBondAdmissionError,
    DirectBondAdmissionRecovery, DirectBondAdmissionRefusal as RenderDirectBondAdmissionRefusal,
    DirectBondGesture, DirectBondPointerHitState, DirectBondPointerProbe,
    DirectBondPointerProbeCategory,
    DirectBondPointerProbeError as RenderDirectBondPointerProbeError,
    DirectBondPointerProbeRecovery, DirectBondViewportToScene,
};
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use super::document_error_binding::document_object_id;
create_exception!(
    ferrum_chem,
    DirectBondGestureError,
    super::binding::DocumentError
);
create_exception!(
    ferrum_chem,
    DirectBondAdmissionRefusal,
    DirectBondGestureError
);
create_exception!(
    ferrum_chem,
    DirectBondPointerProbeError,
    DirectBondGestureError
);

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondPointerProbeCategory",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondPointerProbeCategory {
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
    name = "DirectBondPointerProbeRecovery",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondPointerProbeRecovery {
    CorrectInput,
    AdjustEndpoint,
    RefreshAndRestart,
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondAdmissionCategory",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondAdmissionCategory {
    ForeignSession,
    Consumed,
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
    name = "DirectBondAdmissionRecovery",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondAdmissionRecovery {
    RefreshAndRestart,
    AdjustEndpoint,
    ChangePresentation,
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DirectBondPointerHitState",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PyDirectBondPointerHitState {
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

#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondViewportToScene")]
pub(super) struct PyDirectBondViewportToScene {
    transform: DirectBondViewportToScene,
}
#[pymethods]
impl PyDirectBondViewportToScene {
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
        DirectBondViewportToScene::new(m11, m12, m21, m22, dx, dy)
            .map(|transform| Self { transform })
            .map_err(|error| pointer_probe_error(py, error))
    }
}

#[pyclass(frozen, module = "ferrum_chem", name = "DirectBondPointerProbe")]
pub(super) struct PyDirectBondPointerProbe {
    pub(super) probe: DirectBondPointerProbe,
}
#[pymethods]
impl PyDirectBondPointerProbe {
    #[new]
    #[pyo3(signature = (scene_x, scene_y, viewport_to_scene, direct_hit_state, direct_atom_id=None))]
    fn new(
        py: Python<'_>,
        scene_x: f64,
        scene_y: f64,
        viewport_to_scene: PyRef<'_, PyDirectBondViewportToScene>,
        direct_hit_state: PyRef<'_, PyDirectBondPointerHitState>,
        direct_atom_id: Option<String>,
    ) -> PyResult<Self> {
        let state = match *direct_hit_state {
            PyDirectBondPointerHitState::None => DirectBondPointerHitState::None,
            PyDirectBondPointerHitState::UniqueAtom => DirectBondPointerHitState::UniqueAtom,
            PyDirectBondPointerHitState::AmbiguousAtom => DirectBondPointerHitState::AmbiguousAtom,
        };
        let direct_atom_id = direct_atom_id
            .map(|value| document_object_id(py, value))
            .transpose()?;
        DirectBondPointerProbe::new(
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

#[pyclass(unsendable, module = "ferrum_chem", name = "DirectBondGesture")]
pub(super) struct PyDirectBondGesture {
    gesture: Option<DirectBondGesture>,
}

impl PyDirectBondGesture {
    pub(super) fn from_renderer_gesture(gesture: DirectBondGesture) -> Self {
        Self {
            gesture: Some(gesture),
        }
    }

    pub(super) fn take_for_resolution(&mut self) -> PyResult<DirectBondGesture> {
        self.gesture.take().ok_or_else(|| {
            DirectBondGestureError::new_err(
                "direct-bond gesture was already transferred to endpoint resolution",
            )
        })
    }
}

pub(super) fn pointer_probe_error(
    py: Python<'_>,
    error: RenderDirectBondPointerProbeError,
) -> PyErr {
    let category = match error.category() {
        DirectBondPointerProbeCategory::NonFiniteScenePoint => {
            PyDirectBondPointerProbeCategory::NonFiniteScenePoint
        }
        DirectBondPointerProbeCategory::MalformedTransform => {
            PyDirectBondPointerProbeCategory::MalformedTransform
        }
        DirectBondPointerProbeCategory::InvalidHitEvidence => {
            PyDirectBondPointerProbeCategory::InvalidHitEvidence
        }
        DirectBondPointerProbeCategory::UnknownDirectAtom => {
            PyDirectBondPointerProbeCategory::UnknownDirectAtom
        }
        DirectBondPointerProbeCategory::AmbiguousAtom => {
            PyDirectBondPointerProbeCategory::AmbiguousAtom
        }
        DirectBondPointerProbeCategory::StaleRevision => {
            PyDirectBondPointerProbeCategory::StaleRevision
        }
        DirectBondPointerProbeCategory::StaleDigest => {
            PyDirectBondPointerProbeCategory::StaleDigest
        }
    };
    let recovery = match error.recovery() {
        DirectBondPointerProbeRecovery::CorrectInput => {
            PyDirectBondPointerProbeRecovery::CorrectInput
        }
        DirectBondPointerProbeRecovery::AdjustEndpoint => {
            PyDirectBondPointerProbeRecovery::AdjustEndpoint
        }
        DirectBondPointerProbeRecovery::RefreshAndRestart => {
            PyDirectBondPointerProbeRecovery::RefreshAndRestart
        }
    };
    let exception = DirectBondPointerProbeError::new_err(error.to_string());
    let value = exception.value(py);
    value
        .setattr("category", Py::new(py, category).expect("closed category"))
        .expect("category attaches");
    value
        .setattr("recovery", Py::new(py, recovery).expect("closed recovery"))
        .expect("recovery attaches");
    exception
}

pub(super) fn admission_error(py: Python<'_>, error: RenderDirectBondAdmissionError) -> PyErr {
    match error {
        RenderDirectBondAdmissionError::PointerProbe(error) => pointer_probe_error(py, error),
        RenderDirectBondAdmissionError::Refusal(error) => admission_refusal_error(py, error),
        RenderDirectBondAdmissionError::DocumentGesture(error) => {
            DirectBondGestureError::new_err(error.to_string())
        }
    }
}

fn admission_refusal_error(py: Python<'_>, error: RenderDirectBondAdmissionRefusal) -> PyErr {
    let category = match error.category() {
        DirectBondAdmissionCategory::ForeignSession => {
            PyDirectBondAdmissionCategory::ForeignSession
        }
        DirectBondAdmissionCategory::Consumed => PyDirectBondAdmissionCategory::Consumed,
        DirectBondAdmissionCategory::StaleRevision => PyDirectBondAdmissionCategory::StaleRevision,
        DirectBondAdmissionCategory::StaleDigest => PyDirectBondAdmissionCategory::StaleDigest,
        DirectBondAdmissionCategory::UnknownStartAtom => {
            PyDirectBondAdmissionCategory::UnknownStartAtom
        }
        DirectBondAdmissionCategory::UnknownEndAtom => {
            PyDirectBondAdmissionCategory::UnknownEndAtom
        }
        DirectBondAdmissionCategory::UnsupportedPresentation => {
            PyDirectBondAdmissionCategory::UnsupportedPresentation
        }
        DirectBondAdmissionCategory::InvalidEndpointInput => {
            PyDirectBondAdmissionCategory::InvalidEndpointInput
        }
        DirectBondAdmissionCategory::CollapsedEndpoint => {
            PyDirectBondAdmissionCategory::CollapsedEndpoint
        }
        DirectBondAdmissionCategory::SelfLoop => PyDirectBondAdmissionCategory::SelfLoop,
        DirectBondAdmissionCategory::CrossMolecule => PyDirectBondAdmissionCategory::CrossMolecule,
        DirectBondAdmissionCategory::DuplicateBond => PyDirectBondAdmissionCategory::DuplicateBond,
        DirectBondAdmissionCategory::ExceedsChemistryCapacity => {
            PyDirectBondAdmissionCategory::ExceedsChemistryCapacity
        }
        DirectBondAdmissionCategory::UnsupportedChemistryAdmission => {
            PyDirectBondAdmissionCategory::UnsupportedChemistryAdmission
        }
        DirectBondAdmissionCategory::UnrenderableCandidate => {
            PyDirectBondAdmissionCategory::UnrenderableCandidate
        }
    };
    let recovery = match error.recovery() {
        DirectBondAdmissionRecovery::RefreshAndRestart => {
            PyDirectBondAdmissionRecovery::RefreshAndRestart
        }
        DirectBondAdmissionRecovery::AdjustEndpoint => {
            PyDirectBondAdmissionRecovery::AdjustEndpoint
        }
        DirectBondAdmissionRecovery::ChangePresentation => {
            PyDirectBondAdmissionRecovery::ChangePresentation
        }
    };
    let exception = DirectBondAdmissionRefusal::new_err(error.to_string());
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
