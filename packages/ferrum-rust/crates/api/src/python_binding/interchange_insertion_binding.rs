//! Generic prepared interchange transactions exposed to Python.

use ferrum_document::{InterchangeRecordBatchInsertionV1, PendingAdmittedInterchangeBatchV1};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

/// One immutable, native-handle-free ordered interchange insertion batch.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "InterchangeRecordBatchInsertionV1",
    skip_from_py_object
)]
pub(crate) struct PyInterchangeRecordBatchInsertionV1 {
    batch: InterchangeRecordBatchInsertionV1,
}

impl PyInterchangeRecordBatchInsertionV1 {
    pub(crate) const fn new(batch: InterchangeRecordBatchInsertionV1) -> Self {
        Self { batch }
    }

    pub(crate) const fn batch(&self) -> &InterchangeRecordBatchInsertionV1 {
        &self.batch
    }
}

#[pymethods]
impl PyInterchangeRecordBatchInsertionV1 {
    /// Return the number of source-ordered records retained by this batch.
    #[getter]
    fn record_count(&self) -> usize {
        self.batch.records().len()
    }
}

/// Opaque one-use prepared interchange-record insertion.
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "AdmittedInterchangeRecordInsertionV1"
)]
pub(crate) struct PyAdmittedInterchangeRecordInsertionV1 {
    pub(crate) pending: PendingAdmittedInterchangeBatchV1,
    molecule_identifiers: Vec<String>,
    atom_identifiers: Vec<Vec<String>>,
    bond_identifiers: Vec<Vec<String>>,
}

impl PyAdmittedInterchangeRecordInsertionV1 {
    pub(crate) fn new(pending: PendingAdmittedInterchangeBatchV1) -> Self {
        let molecule_identifiers = pending
            .molecule_identifiers()
            .iter()
            .map(|identifier| identifier.as_str().to_owned())
            .collect();
        let atom_identifiers = copied_identifier_groups(pending.atom_identifiers());
        let bond_identifiers = copied_identifier_groups(pending.bond_identifiers());
        Self {
            pending,
            molecule_identifiers,
            atom_identifiers,
            bond_identifiers,
        }
    }
}

#[pymethods]
impl PyAdmittedInterchangeRecordInsertionV1 {
    /// Return durable molecule IDs in exact source-record order.
    #[getter]
    fn molecule_identifiers(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, &self.molecule_identifiers)?.unbind())
    }

    /// Return record-grouped durable atom IDs in exact source order.
    #[getter]
    fn atom_identifiers(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        nested_tuple(py, &self.atom_identifiers)
    }

    /// Return record-grouped durable bond IDs in exact source order.
    #[getter]
    fn bond_identifiers(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        nested_tuple(py, &self.bond_identifiers)
    }
}

fn copied_identifier_groups(groups: &[Vec<ferrum_document::PersistentId>]) -> Vec<Vec<String>> {
    groups
        .iter()
        .map(|group| {
            group
                .iter()
                .map(|identifier| identifier.as_str().to_owned())
                .collect()
        })
        .collect()
}

fn nested_tuple(py: Python<'_>, groups: &[Vec<String>]) -> PyResult<Py<PyTuple>> {
    let groups = groups
        .iter()
        .map(|group| PyTuple::new(py, group))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyTuple::new(py, groups)?.unbind())
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyInterchangeRecordBatchInsertionV1>()?;
    module.add_class::<PyAdmittedInterchangeRecordInsertionV1>()?;
    Ok(())
}
