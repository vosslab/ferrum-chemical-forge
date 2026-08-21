//! Frozen Python boundary for Ferrum's closed periodic-picker display catalog.
//!
//! The extension accepts one exact symbol, copies the matched Rust facts, and
//! returns no mutable table, category spelling, alias, or fallback behavior.

use ferrum_domain::catalog::{
    ElementDisplayCategoryV1, ElementDisplayFactsV1, PERIODIC_DISPLAY_CATALOG_PROVENANCE_V1,
    periodic_display_elements_v1, periodic_display_facts_v1 as rust_periodic_display_facts_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::binding::FerrumError;

create_exception!(ferrum_chem, PeriodicDisplayError, FerrumError);
create_exception!(
    ferrum_chem,
    UnknownElementDisplaySymbolError,
    PeriodicDisplayError
);

/// Closed V1 category vocabulary for the periodic-table popup palette.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ElementDisplayCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyElementDisplayCategoryV1 {
    Nonmetal,
    Halogen,
    NobleGas,
    Metalloid,
    Metal,
    TransitionMetal,
    Lanthanide,
    Actinide,
}

/// Immutable display facts copied from Ferrum's periodic-picker catalog.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "ElementDisplayFactsV1",
    skip_from_py_object
)]
struct PyElementDisplayFactsV1 {
    #[pyo3(get)]
    symbol: String,
    #[pyo3(get)]
    category: Py<PyElementDisplayCategoryV1>,
    #[pyo3(get)]
    color: String,
}

/// Immutable provenance for the bounded periodic-picker catalog.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PeriodicDisplayCatalogProvenanceV1",
    skip_from_py_object
)]
struct PyPeriodicDisplayCatalogProvenanceV1 {
    #[pyo3(get)]
    catalog_id: String,
    #[pyo3(get)]
    revision: String,
    #[pyo3(get)]
    source: String,
    #[pyo3(get)]
    scope: String,
}

/// Return one exact immutable display entry for a supported picker symbol.
#[pyfunction]
fn periodic_display_facts_v1(py: Python<'_>, symbol: String) -> PyResult<PyElementDisplayFactsV1> {
    let facts = match rust_periodic_display_facts_v1(&symbol) {
        Ok(facts) => facts,
        Err(error) => return Err(unknown_symbol_error(py, error.to_string(), error.symbol())?),
    };
    facts_to_python(py, facts)
}

fn unknown_symbol_error(py: Python<'_>, message: String, symbol: &str) -> PyResult<PyErr> {
    let error = UnknownElementDisplaySymbolError::new_err(message);
    error.value(py).setattr("symbol", symbol)?;
    Ok(error)
}

/// Return every supported picker entry in its immutable user-visible order.
#[pyfunction]
fn periodic_display_entries_v1(py: Python<'_>) -> PyResult<Py<PyTuple>> {
    let entries = periodic_display_elements_v1()
        .iter()
        .map(|facts| facts_to_python(py, facts))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyTuple::new(py, entries)?.unbind())
}

/// Return immutable provenance for this catalog's explicit display-only scope.
#[pyfunction]
fn periodic_display_catalog_provenance_v1() -> PyPeriodicDisplayCatalogProvenanceV1 {
    PyPeriodicDisplayCatalogProvenanceV1 {
        catalog_id: PERIODIC_DISPLAY_CATALOG_PROVENANCE_V1
            .catalog_id()
            .to_owned(),
        revision: PERIODIC_DISPLAY_CATALOG_PROVENANCE_V1.revision().to_owned(),
        source: PERIODIC_DISPLAY_CATALOG_PROVENANCE_V1.source().to_owned(),
        scope: PERIODIC_DISPLAY_CATALOG_PROVENANCE_V1.scope().to_owned(),
    }
}

fn facts_to_python(
    py: Python<'_>,
    facts: &ElementDisplayFactsV1,
) -> PyResult<PyElementDisplayFactsV1> {
    Ok(PyElementDisplayFactsV1 {
        symbol: facts.symbol().to_owned(),
        category: Py::new(py, category_to_python(facts.category()))?,
        color: facts.color().to_owned(),
    })
}

fn category_to_python(category: ElementDisplayCategoryV1) -> PyElementDisplayCategoryV1 {
    match category {
        ElementDisplayCategoryV1::Nonmetal => PyElementDisplayCategoryV1::Nonmetal,
        ElementDisplayCategoryV1::Halogen => PyElementDisplayCategoryV1::Halogen,
        ElementDisplayCategoryV1::NobleGas => PyElementDisplayCategoryV1::NobleGas,
        ElementDisplayCategoryV1::Metalloid => PyElementDisplayCategoryV1::Metalloid,
        ElementDisplayCategoryV1::Metal => PyElementDisplayCategoryV1::Metal,
        ElementDisplayCategoryV1::TransitionMetal => PyElementDisplayCategoryV1::TransitionMetal,
        ElementDisplayCategoryV1::Lanthanide => PyElementDisplayCategoryV1::Lanthanide,
        ElementDisplayCategoryV1::Actinide => PyElementDisplayCategoryV1::Actinide,
    }
}

/// Register the closed periodic-picker display catalog boundary.
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "PeriodicDisplayError",
        module.py().get_type::<PeriodicDisplayError>(),
    )?;
    module.add(
        "UnknownElementDisplaySymbolError",
        module.py().get_type::<UnknownElementDisplaySymbolError>(),
    )?;
    module.add_class::<PyElementDisplayCategoryV1>()?;
    module.add_class::<PyElementDisplayFactsV1>()?;
    module.add_class::<PyPeriodicDisplayCatalogProvenanceV1>()?;
    module.add_function(wrap_pyfunction!(periodic_display_facts_v1, module)?)?;
    module.add_function(wrap_pyfunction!(periodic_display_entries_v1, module)?)?;
    module.add_function(wrap_pyfunction!(
        periodic_display_catalog_provenance_v1,
        module
    )?)?;
    Ok(())
}
