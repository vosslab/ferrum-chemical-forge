//! Frozen Python DTOs for Rust-owned arrow display geometry.

use ferrum_document::{
    ArrowDisplayGeometryV1, ArrowHeadPositionV1, ArrowHeadShapeV1, ArrowHeadV1, ArrowPathV1,
    ArrowProjectionV1, CurvedTerminalArrowDisplayKindV1,
};
use pyo3::prelude::*;

use super::projection_binding::{PyPoint3V1, PyPresentationStrokeV1, PyPresentationTargetV1};

/// Every ordered point of one supported normal-arrow path.
#[pyclass(frozen, name = "ArrowPathV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyArrowPathV1 {
    #[pyo3(get)]
    pub(crate) points: Vec<PyPoint3V1>,
}
impl From<&ArrowPathV1> for PyArrowPathV1 {
    fn from(value: &ArrowPathV1) -> Self {
        Self {
            points: value
                .points()
                .iter()
                .map(|point| PyPoint3V1 {
                    x: point.x(),
                    y: point.y(),
                    z: point.z(),
                })
                .collect(),
        }
    }
}

/// Explicit normal-arrow head dimensions copied from Rust.
#[pyclass(frozen, name = "ArrowHeadShapeV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyArrowHeadShapeV1 {
    #[pyo3(get)]
    pub(crate) line_inset: f64,
    #[pyo3(get)]
    pub(crate) total_length: f64,
    #[pyo3(get)]
    pub(crate) half_width: f64,
}

impl From<ArrowHeadShapeV1> for PyArrowHeadShapeV1 {
    fn from(value: ArrowHeadShapeV1) -> Self {
        Self {
            line_inset: value.line_inset(),
            total_length: value.total_length(),
            half_width: value.half_width(),
        }
    }
}

/// One backend-derived filled normal-arrow head polygon.
#[pyclass(frozen, name = "ArrowHeadV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyArrowHeadV1 {
    #[pyo3(get)]
    pub(crate) position: String,
    #[pyo3(get)]
    pub(crate) points: Vec<PyPoint3V1>,
}

impl From<&ArrowHeadV1> for PyArrowHeadV1 {
    fn from(value: &ArrowHeadV1) -> Self {
        Self {
            position: match value.position() {
                ArrowHeadPositionV1::Start => "start",
                ArrowHeadPositionV1::End => "end",
            }
            .to_owned(),
            points: value
                .points()
                .iter()
                .map(|point| PyPoint3V1 {
                    x: point.x(),
                    y: point.y(),
                    z: point.z(),
                })
                .collect(),
        }
    }
}

/// Closed Python representation of Rust-issued arrow display geometry.
#[pyclass(frozen, name = "ArrowDisplayGeometryV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyArrowDisplayGeometryV1 {
    #[pyo3(get)]
    pub(crate) kind: String,
    #[pyo3(get)]
    pub(crate) normal: Option<PyNormalArrowDisplayGeometryV1>,
    #[pyo3(get)]
    pub(crate) equilibrium: Option<PyEquilibriumArrowDisplayGeometryV1>,
    #[pyo3(get)]
    pub(crate) curved_equilibrium: Option<PyCurvedEquilibriumArrowDisplayGeometryV1>,
    #[pyo3(get)]
    pub(crate) curved_terminal: Option<PyCurvedTerminalArrowDisplayGeometryV1>,
}

#[pyclass(frozen, name = "NormalArrowDisplayGeometryV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyNormalArrowDisplayGeometryV1 {
    #[pyo3(get)]
    pub(crate) axis_path: PyArrowPathV1,
    #[pyo3(get)]
    pub(crate) head_shape: PyArrowHeadShapeV1,
    #[pyo3(get)]
    pub(crate) start_head: bool,
    #[pyo3(get)]
    pub(crate) end_head: bool,
    #[pyo3(get)]
    pub(crate) heads: Vec<PyArrowHeadV1>,
}

#[pyclass(
    frozen,
    name = "EquilibriumArrowDisplayGeometryV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyEquilibriumArrowDisplayGeometryV1 {
    #[pyo3(get)]
    pub(crate) axes: Vec<PyArrowPathV1>,
    #[pyo3(get)]
    pub(crate) heads: Vec<PyArrowHeadV1>,
}

/// Rust-issued two-lane geometry for one quadratic curved equilibrium arrow.
#[pyclass(
    frozen,
    name = "CurvedEquilibriumArrowDisplayGeometryV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyCurvedEquilibriumArrowDisplayGeometryV1 {
    #[pyo3(get)]
    pub(crate) axes: Vec<PyArrowPathV1>,
    #[pyo3(get)]
    pub(crate) control: PyPoint3V1,
    #[pyo3(get)]
    pub(crate) heads: Vec<PyArrowHeadV1>,
}

/// Closed terminal-arrow family identity copied exactly from the Rust projection.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "CurvedTerminalArrowDisplayKindV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyCurvedTerminalArrowDisplayKindV1 {
    Electron,
    Retro,
    CurvedNormalReaction,
}

/// One Rust-issued curved terminal-arrow display payload for every family.
#[pyclass(
    frozen,
    name = "CurvedTerminalArrowDisplayGeometryV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyCurvedTerminalArrowDisplayGeometryV1 {
    #[pyo3(get)]
    pub(crate) kind: PyCurvedTerminalArrowDisplayKindV1,
    #[pyo3(get)]
    pub(crate) axis_path: PyArrowPathV1,
    #[pyo3(get)]
    pub(crate) control: PyPoint3V1,
    #[pyo3(get)]
    pub(crate) head_shape: PyArrowHeadShapeV1,
    #[pyo3(get)]
    pub(crate) head: PyArrowHeadV1,
}

impl From<&ArrowDisplayGeometryV1> for PyArrowDisplayGeometryV1 {
    fn from(value: &ArrowDisplayGeometryV1) -> Self {
        match value {
            ArrowDisplayGeometryV1::Normal {
                axis_path,
                head_shape,
                start_head,
                end_head,
                heads,
            } => Self {
                kind: "normal".to_owned(),
                normal: Some(PyNormalArrowDisplayGeometryV1 {
                    axis_path: axis_path.into(),
                    head_shape: (*head_shape).into(),
                    start_head: *start_head,
                    end_head: *end_head,
                    heads: heads.iter().map(Into::into).collect(),
                }),
                equilibrium: None,
                curved_equilibrium: None,
                curved_terminal: None,
            },
            ArrowDisplayGeometryV1::Equilibrium { axes, heads } => Self {
                kind: "equilibrium".to_owned(),
                normal: None,
                equilibrium: Some(PyEquilibriumArrowDisplayGeometryV1 {
                    axes: axes.iter().map(Into::into).collect(),
                    heads: heads.iter().map(Into::into).collect(),
                }),
                curved_equilibrium: None,
                curved_terminal: None,
            },
            ArrowDisplayGeometryV1::CurvedEquilibrium {
                axes,
                control,
                heads,
            } => Self {
                kind: "curved_equilibrium".to_owned(),
                normal: None,
                equilibrium: None,
                curved_equilibrium: Some(PyCurvedEquilibriumArrowDisplayGeometryV1 {
                    axes: axes.iter().map(Into::into).collect(),
                    control: PyPoint3V1 {
                        x: control.x(),
                        y: control.y(),
                        z: control.z(),
                    },
                    heads: heads.iter().map(Into::into).collect(),
                }),
                curved_terminal: None,
            },
            ArrowDisplayGeometryV1::CurvedTerminal {
                terminal_kind,
                axis_path,
                control,
                head_shape,
                head,
            } => Self {
                kind: "curved_terminal".to_owned(),
                normal: None,
                equilibrium: None,
                curved_equilibrium: None,
                curved_terminal: Some(PyCurvedTerminalArrowDisplayGeometryV1 {
                    kind: match terminal_kind {
                        CurvedTerminalArrowDisplayKindV1::Electron => {
                            PyCurvedTerminalArrowDisplayKindV1::Electron
                        }
                        CurvedTerminalArrowDisplayKindV1::Retro => {
                            PyCurvedTerminalArrowDisplayKindV1::Retro
                        }
                        CurvedTerminalArrowDisplayKindV1::CurvedNormalReaction => {
                            PyCurvedTerminalArrowDisplayKindV1::CurvedNormalReaction
                        }
                    },
                    axis_path: axis_path.into(),
                    control: PyPoint3V1 {
                        x: control.x(),
                        y: control.y(),
                        z: control.z(),
                    },
                    head_shape: (*head_shape).into(),
                    head: head.into(),
                }),
            },
        }
    }
}

/// One direct-root Arrow with a closed kind-owned geometry payload.
#[pyclass(frozen, name = "ArrowProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyArrowProjectionV1 {
    #[pyo3(get)]
    pub(crate) target: PyPresentationTargetV1,
    #[pyo3(get)]
    pub(crate) source_path: PyArrowPathV1,
    #[pyo3(get)]
    pub(crate) geometry: PyArrowDisplayGeometryV1,
    #[pyo3(get)]
    pub(crate) stroke: PyPresentationStrokeV1,
}

impl From<&ArrowProjectionV1> for PyArrowProjectionV1 {
    fn from(value: &ArrowProjectionV1) -> Self {
        Self {
            target: value.target().into(),
            source_path: value.source_path().into(),
            geometry: value.geometry().into(),
            stroke: value.stroke().into(),
        }
    }
}
