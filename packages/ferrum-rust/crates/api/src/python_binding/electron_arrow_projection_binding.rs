//! Frozen Python DTO for Rust-owned curved electron-arrow display geometry.

use pyo3::prelude::*;

use super::projection_binding::{PyArrowHeadShapeV1, PyArrowHeadV1, PyArrowPathV1, PyPoint3V1};

/// Quadratic electron-arrow geometry lowered by Rust for Qt presentation.
#[pyclass(frozen, name = "ElectronArrowDisplayGeometryV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyElectronArrowDisplayGeometryV1 {
    #[pyo3(get)]
    pub(crate) axis_path: PyArrowPathV1,
    #[pyo3(get)]
    pub(crate) control: PyPoint3V1,
    #[pyo3(get)]
    pub(crate) head_shape: PyArrowHeadShapeV1,
    #[pyo3(get)]
    pub(crate) head: PyArrowHeadV1,
}
