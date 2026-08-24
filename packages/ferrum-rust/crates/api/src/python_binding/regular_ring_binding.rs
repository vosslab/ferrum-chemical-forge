//! Private prepared regular-ring adapter for the native Qt insertion tool.

use ferrum_document::{
    DetachedRegularRingInsertionV1, PendingAdmittedMoleculeInsertionV1, RegularRingOrientationV1,
    RegularRingSizeV1,
};
use pyo3::prelude::*;

use super::binding::{
    PyDocumentSession, PySessionOperationResultV1, document_result, operation_validation_error,
    projection_error,
};
use super::render_binding::{PyRenderPlanV2, plan_from};

/// Private one-use regular-ring candidate. It deliberately has no stub promise.
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "AdmittedRegularRingInsertionV1"
)]
pub(crate) struct PyAdmittedRegularRingInsertionV1 {
    pub(crate) pending: PendingAdmittedMoleculeInsertionV1,
    #[pyo3(get)]
    pub(crate) molecule_identifier: String,
    #[pyo3(get)]
    pub(crate) atom_identifiers: Vec<String>,
    #[pyo3(get)]
    pub(crate) bond_identifiers: Vec<String>,
    #[pyo3(get)]
    pub(crate) render_plan: PyRenderPlanV2,
}

#[pymethods]
impl PyDocumentSession {
    /// Prepare a private Rust-owned detached regular ring and expose copied preview facts.
    fn prepare_admitted_regular_ring_insertion_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        size: u8,
        center_x: f64,
        center_y: f64,
        side_length: f64,
    ) -> PyResult<PyAdmittedRegularRingInsertionV1> {
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
        let pending = document_result(
            py,
            self.session
                .prepare_admitted_regular_ring_insertion_v1(expected_revision, request),
        )?;
        let render_plan = pending.molecule_render_plan_v1().ok_or_else(|| {
            operation_validation_error(py, "Ferrum renderer omitted regular-ring plan".to_owned())
        })?;
        let render_plan = plan_from(py, render_plan)?;
        Ok(PyAdmittedRegularRingInsertionV1 {
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
            render_plan,
        })
    }

    /// Commit one prepared private regular-ring candidate exactly once.
    fn commit_admitted_regular_ring_insertion_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyAdmittedRegularRingInsertionV1>,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(
            py,
            self.session
                .commit_admitted_molecule_insertion_v1(expected_revision, &mut prepared.pending),
        )
        .map(Into::into)
    }
}
