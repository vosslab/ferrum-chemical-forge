//! Closed Python Arrow-property changes and bounded operation construction.

use ferrum_document::{
    ArrowLineWidthV1, ArrowPropertiesPatchV1, ArrowPropertyChangeV1, Rgb24V1, SessionOperation,
    SessionOperationV1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyFloat, PyInt, PyTuple};

use crate::binding::operation_validation_error;

/// One exact direct-root Arrow property change accepted by a Rust patch.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentArrowPropertyChangeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentArrowPropertyChangeV1 {
    change: ArrowPropertyChangeV1,
}

#[pymethods]
impl PyDocumentArrowPropertyChangeV1 {
    /// Replace start-head visibility.
    #[staticmethod]
    fn start_head(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        arrow_property_change(py, ArrowPropertyChangeV1::StartHead(exact_bool(py, value)?))
    }

    /// Replace end-head visibility.
    #[staticmethod]
    fn end_head(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        arrow_property_change(py, ArrowPropertyChangeV1::EndHead(exact_bool(py, value)?))
    }

    /// Replace spline interpolation intent.
    #[staticmethod]
    fn spline(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        arrow_property_change(py, ArrowPropertyChangeV1::Spline(exact_bool(py, value)?))
    }

    /// Replace the finite Arrow line width from 0.1 through 20 scene points.
    #[staticmethod]
    fn line_width(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        if !(value.is_exact_instance_of::<PyFloat>() || value.is_exact_instance_of::<PyInt>())
            || value.is_instance_of::<PyBool>()
        {
            return Err(operation_validation_error(
                py,
                "Arrow line width must be an exact int or float from 0.1 through 20".to_owned(),
            ));
        }
        let value = value.extract::<f64>().ok().and_then(ArrowLineWidthV1::new);
        let value = value.ok_or_else(|| {
            operation_validation_error(
                py,
                "Arrow line width must be finite and from 0.1 through 20".to_owned(),
            )
        })?;
        arrow_property_change(py, ArrowPropertyChangeV1::LineWidth(value))
    }

    /// Replace the root-authoritative Arrow line colour.
    #[staticmethod]
    fn color(py: Python<'_>, value: String) -> PyResult<Self> {
        let value = Rgb24V1::new(value).ok_or_else(|| {
            operation_validation_error(py, "Arrow color must be #rgb or #rrggbb".to_owned())
        })?;
        arrow_property_change(py, ArrowPropertyChangeV1::Color(value))
    }
}

pub(crate) fn set_arrow_properties(
    py: Python<'_>,
    arrow_id: String,
    changes: &Bound<'_, PyTuple>,
) -> PyResult<SessionOperation> {
    if !changes.is_exact_instance_of::<PyTuple>() {
        return Err(operation_validation_error(
            py,
            "Arrow-properties changes must be an exact built-in tuple".to_owned(),
        ));
    }
    if changes.len() > 5 {
        return Err(operation_validation_error(
            py,
            "an Arrow-properties patch accepts at most five unique changes".to_owned(),
        ));
    }
    let changes = changes
        .iter()
        .map(|value| {
            value
                .extract::<PyRef<'_, PyDocumentArrowPropertyChangeV1>>()
                .map(|value| value.change.clone())
                .map_err(Into::into)
        })
        .collect::<PyResult<Vec<_>>>()?;
    let patch = ArrowPropertiesPatchV1::new(arrow_id, changes)
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(SessionOperation::V1(
        SessionOperationV1::SetArrowProperties { patch },
    ))
}

fn exact_bool(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<bool> {
    if !value.is_exact_instance_of::<PyBool>() {
        return Err(operation_validation_error(
            py,
            "Arrow boolean properties must be exact bool values".to_owned(),
        ));
    }
    value.extract()
}

fn arrow_property_change(
    py: Python<'_>,
    change: ArrowPropertyChangeV1,
) -> PyResult<PyDocumentArrowPropertyChangeV1> {
    ArrowPropertiesPatchV1::new("validation-arrow", vec![change.clone()])
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(PyDocumentArrowPropertyChangeV1 { change })
}
