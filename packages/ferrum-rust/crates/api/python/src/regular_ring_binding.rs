//! Private prepared regular-ring adapter for the native Qt insertion tool.

use ferrum_document::{
    DetachedRegularRingInsertionV1, PendingCreateMolecule, RegularRingOrientationV1,
    RegularRingSizeV1,
};
use pyo3::prelude::*;

use crate::binding::{
    PyDocumentSession, PySessionOperationResultV1, document_result, operation_validation_error,
    projection_error,
};
use crate::projection_binding::PyPoint3V1;

/// Private one-use regular-ring candidate. It deliberately has no stub promise.
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PreparedRegularRingInsertionV1"
)]
pub(crate) struct PyPreparedRegularRingInsertionV1 {
    pub(crate) pending: PendingCreateMolecule,
    #[pyo3(get)]
    pub(crate) molecule_identifier: String,
    #[pyo3(get)]
    pub(crate) atom_identifiers: Vec<String>,
    #[pyo3(get)]
    pub(crate) bond_identifiers: Vec<String>,
    #[pyo3(get)]
    pub(crate) vertices: Vec<PyPoint3V1>,
}

#[pymethods]
impl PyDocumentSession {
    /// Prepare a private Rust-owned detached regular ring and expose copied preview facts.
    fn prepare_create_regular_ring_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        size: u8,
        center_x: f64,
        center_y: f64,
        side_length: f64,
    ) -> PyResult<PyPreparedRegularRingInsertionV1> {
        let center = match ferrum_document::Point3V1::new(center_x, center_y, 0.0) {
            Ok(center) => center,
            Err(error) => return Err(projection_error(py, error)?),
        };
        let size = RegularRingSizeV1::new(size)
            .map_err(|error| operation_validation_error(py, error.to_string()))?;
        let request = DetachedRegularRingInsertionV1::new(
            size,
            center,
            side_length,
            RegularRingOrientationV1::FlatTop,
        )
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
        let vertices = request
            .vertices()
            .map_err(|error| operation_validation_error(py, error.to_string()))?
            .into_iter()
            .map(|point| PyPoint3V1 {
                x: point.x(),
                y: point.y(),
                z: point.z(),
            })
            .collect();
        let pending = document_result(
            py,
            self.session
                .prepare_create_regular_ring_v1(expected_revision, request),
        )?;
        Ok(PyPreparedRegularRingInsertionV1 {
            molecule_identifier: pending.molecule_identifier().as_str().to_owned(),
            atom_identifiers: pending
                .atom_identifiers()
                .iter()
                .map(|identifier| identifier.as_str().to_owned())
                .collect(),
            bond_identifiers: pending
                .bond_identifiers()
                .iter()
                .map(|identifier| identifier.as_str().to_owned())
                .collect(),
            pending,
            vertices,
        })
    }

    /// Commit one prepared private regular-ring candidate exactly once.
    fn commit_create_regular_ring_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyPreparedRegularRingInsertionV1>,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(
            py,
            self.session
                .commit_create_molecule(expected_revision, &mut prepared.pending),
        )
        .map(Into::into)
    }
}
