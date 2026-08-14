//! Closed Python atom-property change values for Rust document operations.

use ferrum_document::{AtomPropertiesPatchV1, AtomPropertyChangeV1, PositiveFiniteV1, Rgb24V1};
use pyo3::prelude::*;

use crate::binding::operation_validation_error;

/// One exact atom-property change accepted by a complete Rust patch.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentAtomPropertyChangeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentAtomPropertyChangeV1 {
    pub(crate) change: AtomPropertyChangeV1,
}

#[pymethods]
impl PyDocumentAtomPropertyChangeV1 {
    /// Replace the authored element spelling.
    #[staticmethod]
    fn element(py: Python<'_>, value: String) -> PyResult<Self> {
        atom_property_change(py, AtomPropertyChangeV1::Element(value))
    }

    /// Replace formal charge; zero clears the optional authored attribute.
    #[staticmethod]
    fn formal_charge(py: Python<'_>, value: i32) -> PyResult<Self> {
        atom_property_change(py, AtomPropertyChangeV1::FormalCharge(value))
    }

    /// Replace or clear authored valence.
    #[staticmethod]
    fn valence(py: Python<'_>, value: Option<u16>) -> PyResult<Self> {
        atom_property_change(py, AtomPropertyChangeV1::Valence(value))
    }

    /// Replace or clear authored isotope mass number.
    #[staticmethod]
    fn isotope(py: Python<'_>, value: Option<u16>) -> PyResult<Self> {
        atom_property_change(py, AtomPropertyChangeV1::Isotope(value))
    }

    /// Replace multiplicity; one clears the optional authored attribute.
    #[staticmethod]
    fn multiplicity(py: Python<'_>, value: u16) -> PyResult<Self> {
        atom_property_change(py, AtomPropertyChangeV1::Multiplicity(value))
    }

    /// Persist explicit atom visibility.
    #[staticmethod]
    fn show(py: Python<'_>, value: bool) -> PyResult<Self> {
        atom_property_change(py, AtomPropertyChangeV1::Show(value))
    }

    /// Persist explicit hydrogen-label visibility.
    #[staticmethod]
    fn show_hydrogens(py: Python<'_>, value: bool) -> PyResult<Self> {
        atom_property_change(py, AtomPropertyChangeV1::ShowHydrogens(value))
    }

    /// Replace the positive direct label-font size.
    #[staticmethod]
    fn font_size(py: Python<'_>, value: f64) -> PyResult<Self> {
        let value = PositiveFiniteV1::new(value).ok_or_else(|| {
            operation_validation_error(py, "atom font size must be positive and finite".to_owned())
        })?;
        atom_property_change(py, AtomPropertyChangeV1::FontSize(value))
    }

    /// Replace the direct label-font colour with canonical RGB.
    #[staticmethod]
    fn label_color(py: Python<'_>, value: String) -> PyResult<Self> {
        let value = Rgb24V1::new(value).ok_or_else(|| {
            operation_validation_error(py, "atom label color must be #rgb or #rrggbb".to_owned())
        })?;
        atom_property_change(py, AtomPropertyChangeV1::LabelColor(value))
    }
}

fn atom_property_change(
    py: Python<'_>,
    change: AtomPropertyChangeV1,
) -> PyResult<PyDocumentAtomPropertyChangeV1> {
    AtomPropertiesPatchV1::new("validation-atom", vec![change.clone()])
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(PyDocumentAtomPropertyChangeV1 { change })
}
