//! Python methods for publication and recovery from one document session.

use std::path::PathBuf;

use pyo3::prelude::*;

use crate::binding::{PyDocumentSession, PyPublication, document_result};

#[pymethods]
impl PyDocumentSession {
    /// Publish the current revision and update the saved baseline only if confirmed.
    ///
    /// The path is copied before publication. This operation requires an explicit
    /// revision so an unrelated stale caller cannot silently write session state.
    fn save_atomic(
        &mut self,
        py: Python<'_>,
        path: PathBuf,
        expected_revision: u64,
    ) -> PyResult<PyPublication> {
        document_result(py, self.session.save_atomic(&path, expected_revision)).map(Into::into)
    }

    /// Export the current revision without changing baseline, history, or dirty state.
    fn recovery_export(
        &self,
        py: Python<'_>,
        path: PathBuf,
        expected_revision: u64,
    ) -> PyResult<PyPublication> {
        document_result(py, self.session.recovery_export(&path, expected_revision)).map(Into::into)
    }
}
