//! Frozen typed V4 batch payloads with derived generic replay conveniences.

use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::render_primitive_binding::{
    PyEllipseOpV1, PyLineOpV1, PyMaskOpV1, PyPathOpV3, PyRenderOperationV3, PyRenderPointV1,
    PyTextOpV1, double_bond_carrier_mark_operation, ellipse_operation, frozen_tuple,
    line_operation, mask_operation, path_operation, text_operation,
};

#[pyclass(frozen, name = "InkBoundsV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyInkBoundsV1 {
    #[pyo3(get)]
    pub(crate) min_x: f64,
    #[pyo3(get)]
    pub(crate) min_y: f64,
    #[pyo3(get)]
    pub(crate) max_x: f64,
    #[pyo3(get)]
    pub(crate) max_y: f64,
}

#[pyclass(frozen, name = "AtomLabelRenderV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyAtomLabelRenderV1 {
    pub(crate) mask: Option<PyMaskOpV1>,
    #[pyo3(get)]
    pub(crate) text: PyTextOpV1,
    #[pyo3(get)]
    pub(crate) core_element_run_index: u32,
    #[pyo3(get)]
    pub(crate) full_ink_bounds: PyInkBoundsV1,
    #[pyo3(get)]
    pub(crate) core_element_ink_bounds: PyInkBoundsV1,
}

#[pymethods]
impl PyAtomLabelRenderV1 {
    #[getter]
    fn mask(&self, py: Python<'_>) -> PyResult<Option<Py<PyMaskOpV1>>> {
        self.mask
            .as_ref()
            .map(|value| Py::new(py, value.clone()))
            .transpose()
    }
}

macro_rules! closed_operation {
    ($name:ident, $payload:ident, $python_name:literal, [$($variant:ident($type:ty, $method:ident, $generic:ident)),+ $(,)?]) => {
        #[derive(Clone)]
        enum $payload { $($variant($type)),+ }

        #[pyclass(frozen, name = $python_name, skip_from_py_object)]
        #[derive(Clone)]
        pub(crate) struct $name {
            #[pyo3(get)]
            kind: String,
            operation: $payload,
        }

        impl $name {
            $(pub(crate) fn $method(value: $type) -> Self {
                Self { kind: stringify!($method).to_owned(), operation: $payload::$variant(value) }
            })+
            fn replay_operation(&self) -> PyRenderOperationV3 {
                match &self.operation { $($payload::$variant(value) => $generic(value.clone())),+ }
            }
        }

        #[pymethods]
        impl $name {
            #[getter]
            fn operation(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
                match &self.operation { $(
                    $payload::$variant(value) => Ok(Py::new(py, value.clone())?.into_any()),
                )+ }
            }
        }
    };
}

closed_operation!(
    PyAtomDecorationRenderOpV1,
    PyAtomDecorationPayload,
    "AtomDecorationRenderOpV1",
    [
        Text(PyTextOpV1, text, text_operation),
        Line(PyLineOpV1, line, line_operation),
        Ellipse(PyEllipseOpV1, ellipse, ellipse_operation),
    ]
);

closed_operation!(
    PyCompactGroupRenderOpV1,
    PyCompactGroupOperationPayload,
    "CompactGroupRenderOpV1",
    [
        Text(PyTextOpV1, text, text_operation),
        Line(PyLineOpV1, line, line_operation),
        Ellipse(PyEllipseOpV1, ellipse, ellipse_operation),
    ]
);

closed_operation!(
    PyBondRenderOpV1,
    PyBondOperationPayload,
    "BondRenderOpV1",
    [
        Line(PyLineOpV1, line, line_operation),
        Path(PyPathOpV3, path, path_operation),
        DoubleBondCarrierMark(
            PyLineOpV1,
            double_bond_carrier_mark,
            double_bond_carrier_mark_operation
        ),
    ]
);

#[pyclass(frozen, name = "AtomRenderBatchV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyAtomRenderBatchV1 {
    #[pyo3(get)]
    pub(crate) kind: String,
    #[pyo3(get)]
    pub(crate) atom_local_anchor: PyRenderPointV1,
    #[pyo3(get)]
    pub(crate) label: PyAtomLabelRenderV1,
    pub(crate) decorations: Vec<PyAtomDecorationRenderOpV1>,
}

#[pymethods]
impl PyAtomRenderBatchV1 {
    #[getter]
    fn decorations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.decorations)
    }
    #[getter]
    fn operations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        let mut operations = Vec::with_capacity(self.decorations.len() + 2);
        if let Some(mask) = &self.label.mask {
            operations.push(mask_operation(mask.clone()));
        }
        operations.push(text_operation(self.label.text.clone()));
        operations.extend(
            self.decorations
                .iter()
                .map(PyAtomDecorationRenderOpV1::replay_operation),
        );
        frozen_tuple(py, &operations)
    }
}

#[pyclass(frozen, name = "CompactGroupRenderBatchV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyCompactGroupRenderBatchV1 {
    #[pyo3(get)]
    pub(crate) kind: String,
    #[pyo3(get)]
    pub(crate) atom_local_anchor: PyRenderPointV1,
    pub(crate) operations: Vec<PyCompactGroupRenderOpV1>,
}

#[pymethods]
impl PyCompactGroupRenderBatchV1 {
    #[getter]
    fn operations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        let operations = self
            .operations
            .iter()
            .map(PyCompactGroupRenderOpV1::replay_operation)
            .collect::<Vec<_>>();
        frozen_tuple(py, &operations)
    }
    #[getter]
    fn typed_operations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.operations)
    }
}

#[pyclass(frozen, name = "BondRenderBatchV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyBondRenderBatchV1 {
    #[pyo3(get)]
    pub(crate) kind: String,
    pub(crate) operations: Vec<PyBondRenderOpV1>,
}

#[pymethods]
impl PyBondRenderBatchV1 {
    #[getter]
    fn operations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        let operations = self
            .operations
            .iter()
            .map(PyBondRenderOpV1::replay_operation)
            .collect::<Vec<_>>();
        frozen_tuple(py, &operations)
    }
    #[getter]
    fn typed_operations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.operations)
    }
}

#[derive(Clone)]
pub(crate) enum PyRenderBatchContentV4 {
    Atom(Box<PyAtomRenderBatchV1>),
    CompactGroup(PyCompactGroupRenderBatchV1),
    Bond(PyBondRenderBatchV1),
}

impl PyRenderBatchContentV4 {
    pub(crate) fn to_python(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match self {
            Self::Atom(value) => Ok(Py::new(py, value.as_ref().clone())?.into_any()),
            Self::CompactGroup(value) => Ok(Py::new(py, value.clone())?.into_any()),
            Self::Bond(value) => Ok(Py::new(py, value.clone())?.into_any()),
        }
    }
}
