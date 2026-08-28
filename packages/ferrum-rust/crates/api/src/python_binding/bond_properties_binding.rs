//! Closed Python bond-property change values for Rust document operations.

use ferrum_document::{
    BondPropertiesPatchV1, BondPropertyChangeV1, NonZeroFiniteV1, PositiveFiniteV1, Rgb24V1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool};

use super::binding::operation_validation_error;
use super::document_session_binding::PyDocumentBondPresentationV1;

/// One exact bond-property change accepted by a complete Rust patch.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentBondPropertyChangeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentBondPropertyChangeV1 {
    pub(crate) change: BondPropertyChangeV1,
}

#[pymethods]
impl PyDocumentBondPropertyChangeV1 {
    /// Replace the complete closed bond presentation.
    #[staticmethod]
    fn presentation(
        py: Python<'_>,
        value: PyRef<'_, PyDocumentBondPresentationV1>,
    ) -> PyResult<Self> {
        bond_property_change(py, BondPropertyChangeV1::Presentation((*value).into()))
    }

    /// Replace or clear the explicit centered-double-bond fact.
    #[staticmethod]
    fn center(py: Python<'_>, value: Option<bool>) -> PyResult<Self> {
        bond_property_change(py, BondPropertyChangeV1::Center(value))
    }

    /// Replace or clear the authored positive line width.
    #[staticmethod]
    fn line_width(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let value = optional_positive(py, value, "bond line width")?;
        bond_property_change(py, BondPropertyChangeV1::LineWidth(value))
    }

    /// Replace or clear signed, nonzero parallel-lane spacing.
    #[staticmethod]
    fn bond_width(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let value = match optional_scalar(py, value, "bond width")? {
            Some(value) => NonZeroFiniteV1::new(value).map(Some).ok_or_else(|| {
                operation_validation_error(py, "bond width must be finite and nonzero".to_owned())
            })?,
            None => None,
        };
        bond_property_change(py, BondPropertyChangeV1::BondWidth(value))
    }

    /// Replace or clear the authored positive wedge width.
    #[staticmethod]
    fn wedge_width(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let value = optional_positive(py, value, "bond wedge width")?;
        bond_property_change(py, BondPropertyChangeV1::WedgeWidth(value))
    }

    /// Replace or clear the authored line colour.
    #[staticmethod]
    fn color(py: Python<'_>, value: Option<String>) -> PyResult<Self> {
        let value = match value {
            Some(value) => Rgb24V1::new(value).map(Some).ok_or_else(|| {
                operation_validation_error(py, "bond color must be #rgb or #rrggbb".to_owned())
            })?,
            None => None,
        };
        bond_property_change(py, BondPropertyChangeV1::Color(value))
    }
}

fn optional_positive(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
    label: &str,
) -> PyResult<Option<PositiveFiniteV1>> {
    match optional_scalar(py, value, label)? {
        Some(value) => PositiveFiniteV1::new(value).map(Some).ok_or_else(|| {
            operation_validation_error(py, format!("{label} must be positive and finite"))
        }),
        None => Ok(None),
    }
}

fn optional_scalar(py: Python<'_>, value: &Bound<'_, PyAny>, label: &str) -> PyResult<Option<f64>> {
    if value.is_none() {
        return Ok(None);
    }
    if value.is_instance_of::<PyBool>() {
        return Err(operation_validation_error(
            py,
            format!("{label} must be a number or None, not bool"),
        ));
    }
    value.extract::<f64>().map(Some)
}

fn bond_property_change(
    py: Python<'_>,
    change: BondPropertyChangeV1,
) -> PyResult<PyDocumentBondPropertyChangeV1> {
    BondPropertiesPatchV1::new("validation-bond", vec![change.clone()])
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(PyDocumentBondPropertyChangeV1 { change })
}
