//! Shared Python values and errors for prepared document molecule insertion.

use ferrum_document::{MoleculeInsertionRequestV1, PreparedDocumentMoleculeV2};
use pyo3::prelude::*;

/// One immutable, native-handle-free molecule ready for a session transaction.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "MoleculeInsertionV1",
    skip_from_py_object
)]
pub(crate) struct PyMoleculeInsertionV1 {
    request: MoleculeInsertionRequestV1,
}

impl PyMoleculeInsertionV1 {
    pub(crate) fn from_prepared(
        prepared: PreparedDocumentMoleculeV2,
    ) -> Result<Self, ferrum_document::DocumentStereoSemanticsErrorV1> {
        Ok(Self {
            request: prepared.into_molecule_insertion_request_v1()?,
        })
    }

    pub(crate) const fn request(&self) -> &MoleculeInsertionRequestV1 {
        &self.request
    }
}

#[pymethods]
impl PyMoleculeInsertionV1 {
    /// Return the number of source-ordered atoms in this complete graph.
    #[getter]
    fn atom_count(&self) -> usize {
        self.request.molecule().atoms().len()
    }

    /// Return the number of source-ordered bonds in this complete graph.
    #[getter]
    fn bond_count(&self) -> usize {
        self.request.molecule().bonds().len()
    }
}

pub(crate) fn structured_insertion_error(
    py: Python<'_>,
    constructor: impl FnOnce(String) -> PyErr,
    error: impl std::fmt::Display,
) -> PyResult<PyErr> {
    let reason = error.to_string();
    let py_error = constructor(reason.clone());
    py_error.value(py).setattr("reason", reason)?;
    Ok(py_error)
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyMoleculeInsertionV1>()?;
    Ok(())
}
