//! Frozen Python boundary for CDML's recognized physical paper sizes.

use ferrum_document::{
    PaperDimensionsMmV1, PaperSizeV1, paper_size_catalog_v1 as rust_paper_size_catalog_v1,
};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

/// Positive fixed dimensions in millimetres, before paper orientation.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PaperDimensionsMmV1",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyPaperDimensionsMmV1 {
    #[pyo3(get)]
    width: f64,
    #[pyo3(get)]
    height: f64,
}

impl From<PaperDimensionsMmV1> for PyPaperDimensionsMmV1 {
    fn from(value: PaperDimensionsMmV1) -> Self {
        Self {
            width: value.width(),
            height: value.height(),
        }
    }
}

/// One exact CDML paper-size name and its optional fixed dimensions.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PaperSizeV1",
    skip_from_py_object
)]
struct PyPaperSizeV1 {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    dimensions: Option<PyPaperDimensionsMmV1>,
}

impl From<&PaperSizeV1> for PyPaperSizeV1 {
    fn from(value: &PaperSizeV1) -> Self {
        Self {
            name: value.name().to_owned(),
            dimensions: value.dimensions().map(Into::into),
        }
    }
}

/// Return immutable copies of every exact recognized CDML paper-size entry.
#[pyfunction]
fn paper_size_catalog_v1(py: Python<'_>) -> PyResult<Py<PyTuple>> {
    let entries = rust_paper_size_catalog_v1()
        .iter()
        .map(PyPaperSizeV1::from)
        .collect::<Vec<_>>();
    Ok(PyTuple::new(py, entries)?.unbind())
}

/// Register the closed CDML paper-size boundary.
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyPaperDimensionsMmV1>()?;
    module.add_class::<PyPaperSizeV1>()?;
    module.add_function(wrap_pyfunction!(paper_size_catalog_v1, module)?)?;
    Ok(())
}
