//! Exact frozen Python values for Rust-owned bracket pairs.

use ferrum_document::{
    BracketPairProjectionV1, BracketPropertiesPatchV1, BracketPropertyChangeV1, BracketStyleV1,
    GeometricLineWidthV1, PendingCreateBracket, PresentationBracketStyleV1, Rgb24V1,
    SessionOperation, SessionOperationV1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyFloat, PyInt, PyString, PyTuple};

use super::document_error_binding::operation_validation_error;

/// Closed persistent bracket families.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DocumentBracketStyleV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyDocumentBracketStyleV1 {
    Rectangular,
    Round,
}

impl From<PyDocumentBracketStyleV1> for BracketStyleV1 {
    fn from(value: PyDocumentBracketStyleV1) -> Self {
        match value {
            PyDocumentBracketStyleV1::Rectangular => Self::Rectangular,
            PyDocumentBracketStyleV1::Round => Self::Round,
        }
    }
}

/// Exact finite normalized bounds for one bracket-pair request.
#[pyclass(frozen, module = "ferrum_chem", name = "DocumentBracketBoundsV1")]
pub(crate) struct PyDocumentBracketBoundsV1 {
    #[pyo3(get)]
    pub(crate) left: f64,
    #[pyo3(get)]
    pub(crate) top: f64,
    #[pyo3(get)]
    pub(crate) right: f64,
    #[pyo3(get)]
    pub(crate) bottom: f64,
}

#[pymethods]
impl PyDocumentBracketBoundsV1 {
    #[new]
    fn new(py: Python<'_>, left: f64, top: f64, right: f64, bottom: f64) -> PyResult<Self> {
        if ![left, top, right, bottom].into_iter().all(f64::is_finite)
            || left >= right
            || top >= bottom
        {
            return Err(operation_validation_error(
                py,
                "bracket bounds must be finite with strict left-right and top-bottom order"
                    .to_owned(),
            ));
        }
        Ok(Self {
            left,
            top,
            right,
            bottom,
        })
    }
}

/// One exact durable relationship between two bracket polylines.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "BracketPairProjectionV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyBracketPairProjectionV1 {
    #[pyo3(get)]
    pub(crate) pair_id: String,
    #[pyo3(get)]
    pub(crate) member_ids: Vec<String>,
    #[pyo3(get)]
    pub(crate) style: PyDocumentBracketStyleV1,
    #[pyo3(get)]
    pub(crate) line_width: Option<f64>,
    #[pyo3(get)]
    pub(crate) line_color: Option<String>,
}

impl From<&BracketPairProjectionV1> for PyBracketPairProjectionV1 {
    fn from(value: &BracketPairProjectionV1) -> Self {
        Self {
            pair_id: value.pair_id().to_owned(),
            member_ids: value.member_ids().to_vec(),
            style: match value.style() {
                PresentationBracketStyleV1::Rectangular => PyDocumentBracketStyleV1::Rectangular,
                PresentationBracketStyleV1::Round => PyDocumentBracketStyleV1::Round,
            },
            line_width: value.line_width().map(|width| width.value()),
            line_color: value.line_color().map(|color| color.as_str().to_owned()),
        }
    }
}

/// Opaque one-use prepared bracket-pair insertion.
#[pyclass(unsendable, module = "ferrum_chem", name = "PreparedBracketInsertion")]
pub(crate) struct PyPreparedBracketInsertion {
    pub(crate) pending: PendingCreateBracket,
    #[pyo3(get)]
    pub(crate) pair_identifier: String,
    #[pyo3(get)]
    pub(crate) left_identifier: String,
    #[pyo3(get)]
    pub(crate) right_identifier: String,
}

/// One exact common bracket-pair property change accepted by Rust.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentBracketPropertyChangeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentBracketPropertyChangeV1 {
    change: BracketPropertyChangeV1,
}

#[pymethods]
impl PyDocumentBracketPropertyChangeV1 {
    /// Replace both sides' finite line width from 0.1 through 20 scene points.
    #[staticmethod]
    fn line_width(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        if !(value.is_exact_instance_of::<PyFloat>() || value.is_exact_instance_of::<PyInt>())
            || value.is_instance_of::<PyBool>()
        {
            return Err(operation_validation_error(
                py,
                "bracket line width must be an exact int or float from 0.1 through 20".to_owned(),
            ));
        }
        let value = value
            .extract::<f64>()
            .ok()
            .and_then(GeometricLineWidthV1::new)
            .ok_or_else(|| {
                operation_validation_error(
                    py,
                    "bracket line width must be finite and from 0.1 through 20".to_owned(),
                )
            })?;
        validate_change(py, BracketPropertyChangeV1::LineWidth(value))
    }

    /// Replace both sides' visible line colour with six hexadecimal digits.
    #[staticmethod]
    fn line_color(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        if !value.is_exact_instance_of::<PyString>() {
            return Err(operation_validation_error(
                py,
                "bracket line color must be an exact built-in str".to_owned(),
            ));
        }
        let value = value.extract::<String>()?;
        if value.len() != 7 {
            return Err(operation_validation_error(
                py,
                "bracket line color must use six hexadecimal digits".to_owned(),
            ));
        }
        let value = Rgb24V1::new(value).ok_or_else(|| {
            operation_validation_error(
                py,
                "bracket line color must use six hexadecimal digits".to_owned(),
            )
        })?;
        validate_change(py, BracketPropertyChangeV1::LineColor(value))
    }
}

pub(crate) fn set_bracket_properties(
    py: Python<'_>,
    pair_id: String,
    changes: &Bound<'_, PyTuple>,
) -> PyResult<SessionOperation> {
    if !changes.is_exact_instance_of::<PyTuple>() {
        return Err(operation_validation_error(
            py,
            "bracket-properties changes must be an exact built-in tuple".to_owned(),
        ));
    }
    if changes.len() > 2 {
        return Err(operation_validation_error(
            py,
            "a bracket-properties patch accepts at most two unique changes".to_owned(),
        ));
    }
    let changes = changes
        .iter()
        .map(|value| {
            value
                .extract::<PyRef<'_, PyDocumentBracketPropertyChangeV1>>()
                .map(|value| value.change.clone())
                .map_err(Into::into)
        })
        .collect::<PyResult<Vec<_>>>()?;
    let patch = BracketPropertiesPatchV1::new(pair_id, changes)
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(SessionOperation::V1(
        SessionOperationV1::SetBracketProperties { patch },
    ))
}

fn validate_change(
    py: Python<'_>,
    change: BracketPropertyChangeV1,
) -> PyResult<PyDocumentBracketPropertyChangeV1> {
    BracketPropertiesPatchV1::new("validation-bracket", vec![change.clone()])
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(PyDocumentBracketPropertyChangeV1 { change })
}
