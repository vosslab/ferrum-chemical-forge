//! Frozen Python DTOs for Rust-owned semantic arrow projection values.

use ferrum_document::{
    ArrowHeadShapeV1, ArrowPathV1, ArrowProjectionKindV1, ArrowProjectionV1,
    CurvedTerminalArrowKindV1,
};
use pyo3::prelude::*;

use super::projection_binding::{PyPoint3V1, PyPresentationStrokeV1, PyPresentationTargetV1};

/// Ordered authored points for one supported arrow.
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

/// Authored normal-arrow head dimensions.
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

/// Closed semantic policy for the renderer-owned display lowerer.
#[pyclass(frozen, name = "ArrowProjectionKindV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyArrowProjectionKindV1 {
    #[pyo3(get)]
    pub(crate) kind: String,
    #[pyo3(get)]
    pub(crate) head_shape: Option<PyArrowHeadShapeV1>,
    #[pyo3(get)]
    pub(crate) start_head: Option<bool>,
    #[pyo3(get)]
    pub(crate) end_head: Option<bool>,
    #[pyo3(get)]
    pub(crate) terminal_kind: Option<String>,
}

impl From<&ArrowProjectionKindV1> for PyArrowProjectionKindV1 {
    fn from(value: &ArrowProjectionKindV1) -> Self {
        match value {
            ArrowProjectionKindV1::Normal {
                head_shape,
                start_head,
                end_head,
            } => Self {
                kind: "normal".to_owned(),
                head_shape: Some((*head_shape).into()),
                start_head: Some(*start_head),
                end_head: Some(*end_head),
                terminal_kind: None,
            },
            ArrowProjectionKindV1::Equilibrium => Self {
                kind: "equilibrium".to_owned(),
                head_shape: None,
                start_head: None,
                end_head: None,
                terminal_kind: None,
            },
            ArrowProjectionKindV1::CurvedEquilibrium => Self {
                kind: "curved_equilibrium".to_owned(),
                head_shape: None,
                start_head: None,
                end_head: None,
                terminal_kind: None,
            },
            ArrowProjectionKindV1::CurvedTerminal { terminal_kind } => Self {
                kind: "curved_terminal".to_owned(),
                head_shape: None,
                start_head: None,
                end_head: None,
                terminal_kind: Some(
                    match terminal_kind {
                        CurvedTerminalArrowKindV1::Electron => "electron",
                        CurvedTerminalArrowKindV1::Retro => "retro",
                        CurvedTerminalArrowKindV1::Normal => "normal",
                    }
                    .to_owned(),
                ),
            },
        }
    }
}

/// One direct-root arrow containing immutable authored intent only.
#[pyclass(frozen, name = "ArrowProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyArrowProjectionV1 {
    #[pyo3(get)]
    pub(crate) target: PyPresentationTargetV1,
    #[pyo3(get)]
    pub(crate) source_path: PyArrowPathV1,
    #[pyo3(get)]
    pub(crate) kind: PyArrowProjectionKindV1,
    #[pyo3(get)]
    pub(crate) stroke: PyPresentationStrokeV1,
}

impl From<&ArrowProjectionV1> for PyArrowProjectionV1 {
    fn from(value: &ArrowProjectionV1) -> Self {
        Self {
            target: value.target().into(),
            source_path: value.source_path().into(),
            kind: value.kind().into(),
            stroke: value.stroke().into(),
        }
    }
}
