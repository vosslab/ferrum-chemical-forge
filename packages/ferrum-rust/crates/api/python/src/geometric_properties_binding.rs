//! Closed Python geometric-property changes and bounded operation construction.

use ferrum_document::{
    GeometricLineWidthV1, GeometricPropertiesPatchV1, GeometricPropertyChangeV1, Rgb24V1,
    SessionOperation, SessionOperationV1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyFloat, PyInt, PyString, PyTuple};

use crate::binding::operation_validation_error;

/// One exact geometric presentation property change accepted by a Rust patch.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentGeometricPropertyChangeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentGeometricPropertyChangeV1 {
    change: GeometricPropertyChangeV1,
}

#[pymethods]
impl PyDocumentGeometricPropertyChangeV1 {
    /// Replace the finite line width from 0.1 through 20 scene points.
    #[staticmethod]
    fn line_width(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        if !(value.is_exact_instance_of::<PyFloat>() || value.is_exact_instance_of::<PyInt>())
            || value.is_instance_of::<PyBool>()
        {
            return Err(operation_validation_error(
                py,
                "geometric line width must be an exact int or float from 0.1 through 20".to_owned(),
            ));
        }
        let value = value
            .extract::<f64>()
            .ok()
            .and_then(GeometricLineWidthV1::new)
            .ok_or_else(|| {
                operation_validation_error(
                    py,
                    "geometric line width must be finite and from 0.1 through 20".to_owned(),
                )
            })?;
        geometric_property_change(py, GeometricPropertyChangeV1::LineWidth(value))
    }

    /// Replace the root stroke colour using one six-digit hexadecimal value.
    #[staticmethod]
    fn stroke_color(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let value = exact_six_digit_color(py, value, "geometric stroke color")?;
        geometric_property_change(py, GeometricPropertyChangeV1::StrokeColor(value))
    }

    /// Replace the root fill colour, or author explicit no-fill with `None`.
    #[staticmethod]
    fn fill_color(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let value = if value.is_none() {
            None
        } else {
            Some(exact_six_digit_color(py, value, "geometric fill color")?)
        };
        geometric_property_change(py, GeometricPropertyChangeV1::FillColor(value))
    }
}

pub(crate) fn set_geometric_properties(
    py: Python<'_>,
    presentation_id: String,
    changes: &Bound<'_, PyTuple>,
) -> PyResult<SessionOperation> {
    if !changes.is_exact_instance_of::<PyTuple>() {
        return Err(operation_validation_error(
            py,
            "geometric-properties changes must be an exact built-in tuple".to_owned(),
        ));
    }
    if changes.len() > 3 {
        return Err(operation_validation_error(
            py,
            "a geometric-properties patch accepts at most three unique changes".to_owned(),
        ));
    }
    let changes = changes
        .iter()
        .map(|value| {
            value
                .extract::<PyRef<'_, PyDocumentGeometricPropertyChangeV1>>()
                .map(|value| value.change.clone())
                .map_err(Into::into)
        })
        .collect::<PyResult<Vec<_>>>()?;
    let patch = GeometricPropertiesPatchV1::new(presentation_id, changes)
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(SessionOperation::V1(
        SessionOperationV1::SetGeometricProperties { patch },
    ))
}

fn exact_six_digit_color(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
    description: &'static str,
) -> PyResult<Rgb24V1> {
    if !value.is_exact_instance_of::<PyString>() {
        return Err(operation_validation_error(
            py,
            format!("{description} must be an exact built-in str"),
        ));
    }
    let value = value.extract::<String>()?;
    if value.len() != 7 {
        return Err(operation_validation_error(
            py,
            format!("{description} must use six hexadecimal digits"),
        ));
    }
    Rgb24V1::new(value).ok_or_else(|| {
        operation_validation_error(py, format!("{description} must use six hexadecimal digits"))
    })
}

fn geometric_property_change(
    py: Python<'_>,
    change: GeometricPropertyChangeV1,
) -> PyResult<PyDocumentGeometricPropertyChangeV1> {
    GeometricPropertiesPatchV1::new("validation-geometric", vec![change.clone()])
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(PyDocumentGeometricPropertyChangeV1 { change })
}
