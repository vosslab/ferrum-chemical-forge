//! Generic prepared interchange transactions exposed to Python.

use ferrum_document::InterchangeRecordBatchInsertionV1;
use pyo3::prelude::*;

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

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyInterchangeRecordBatchInsertionV1>()?;
    Ok(())
}
