//! Closed Python Wavy-property changes and bounded operation construction.

use ferrum_document::{
    GeometricLineWidthV1, Rgb24V1, SessionOperation, SessionOperationV1, WavyPropertiesPatchV1,
    WavyPropertyChangeV1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyFloat, PyInt, PyString, PyTuple};

use super::binding::operation_validation_error;
use super::document_error_binding::document_object_id;

/// One exact Wavy presentation property change accepted by a Rust patch.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentWavyPropertyChangeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentWavyPropertyChangeV1 {
    change: WavyPropertyChangeV1,
}

#[pymethods]
impl PyDocumentWavyPropertyChangeV1 {
    /// Replace the finite line width from 0.1 through 20 scene points.
    #[staticmethod]
    fn line_width(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        if !(value.is_exact_instance_of::<PyFloat>() || value.is_exact_instance_of::<PyInt>())
            || value.is_instance_of::<PyBool>()
        {
            return Err(operation_validation_error(
                py,
                "Wavy line width must be an exact int or float from 0.1 through 20".to_owned(),
            ));
        }
        let value = value
            .extract::<f64>()
            .ok()
            .and_then(GeometricLineWidthV1::new)
            .ok_or_else(|| {
                operation_validation_error(
                    py,
                    "Wavy line width must be finite and from 0.1 through 20".to_owned(),
                )
            })?;
        wavy_property_change(py, WavyPropertyChangeV1::LineWidth(value))
    }

    /// Replace the visible line colour using one six-digit hexadecimal value.
    #[staticmethod]
    fn line_color(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        if !value.is_exact_instance_of::<PyString>() {
            return Err(operation_validation_error(
                py,
                "Wavy line color must be an exact built-in str".to_owned(),
            ));
        }
        let value = value.extract::<String>()?;
        if value.len() != 7 {
            return Err(operation_validation_error(
                py,
                "Wavy line color must use six hexadecimal digits".to_owned(),
            ));
        }
        let value = Rgb24V1::new(value).ok_or_else(|| {
            operation_validation_error(
                py,
                "Wavy line color must use six hexadecimal digits".to_owned(),
            )
        })?;
        wavy_property_change(py, WavyPropertyChangeV1::LineColor(value))
    }
}

pub(crate) fn set_wavy_properties(
    py: Python<'_>,
    wavy_id: String,
    changes: &Bound<'_, PyTuple>,
) -> PyResult<SessionOperation> {
    let wavy_id = document_object_id(py, wavy_id)?;
    if !changes.is_exact_instance_of::<PyTuple>() {
        return Err(operation_validation_error(
            py,
            "Wavy-properties changes must be an exact built-in tuple".to_owned(),
        ));
    }
    if changes.len() > 2 {
        return Err(operation_validation_error(
            py,
            "a Wavy-properties patch accepts at most two unique changes".to_owned(),
        ));
    }
    let changes = changes
        .iter()
        .map(|value| {
            value
                .extract::<PyRef<'_, PyDocumentWavyPropertyChangeV1>>()
                .map(|value| value.change.clone())
                .map_err(Into::into)
        })
        .collect::<PyResult<Vec<_>>>()?;
    let patch = WavyPropertiesPatchV1::new(wavy_id, changes)
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(SessionOperation::V1(
        SessionOperationV1::SetWavyProperties { patch },
    ))
}

fn wavy_property_change(
    _py: Python<'_>,
    change: WavyPropertyChangeV1,
) -> PyResult<PyDocumentWavyPropertyChangeV1> {
    Ok(PyDocumentWavyPropertyChangeV1 { change })
}
