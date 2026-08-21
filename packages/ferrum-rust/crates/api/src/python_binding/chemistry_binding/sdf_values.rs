//! Python-owned SDF value classes used by the ABI-4 chemistry boundary.

use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::PySmilesMoleculeV1;

/// Immutable ordered SDF property prepared for native export.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "SdfPropertyV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(super) struct PySdfPropertyV1 {
    #[pyo3(get)]
    pub(super) name: String,
    #[pyo3(get)]
    pub(super) value: String,
}

/// Immutable ordered record copied from native SDF input.
///
/// This is distinct from `SdfRecordV1` because authored SDF input may contain
/// repeated property names, which import preserves in encounter order.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "ImportedSdfRecordV1",
    skip_from_py_object
)]
pub(super) struct PyImportedSdfRecordV1 {
    #[pyo3(get)]
    pub(super) molecule: Py<PySmilesMoleculeV1>,
    #[pyo3(get)]
    pub(super) title: String,
    #[pyo3(get)]
    pub(super) properties: Py<PyTuple>,
}
