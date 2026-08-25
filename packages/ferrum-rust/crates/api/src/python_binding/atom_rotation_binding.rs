//! Exact Python factories for Rust-owned selected-atom rotation.

use ferrum_document::{
    AtomRotationTargetV1, AtomRotationV1, DocumentObjectIdV1, SessionOperation, SessionOperationV1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyFloat, PyInt, PyTuple};

use super::binding::operation_validation_error;
use super::document_error_binding::document_object_id;

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentAtomRotationTargetV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentAtomRotationTargetV1 {
    target: AtomRotationTargetV1,
    #[pyo3(get)]
    molecule_id: String,
    #[pyo3(get)]
    atom_id: String,
}

#[pymethods]
impl PyDocumentAtomRotationTargetV1 {
    #[staticmethod]
    fn create(py: Python<'_>, molecule_id: String, atom_id: String) -> PyResult<Self> {
        let target = AtomRotationTargetV1::new(molecule_id.clone(), atom_id.clone())
            .map_err(|error| operation_validation_error(py, error.to_string()))?;
        Ok(Self {
            target,
            molecule_id,
            atom_id,
        })
    }
}

pub(crate) fn rotate_atoms(
    py: Python<'_>,
    targets: &Bound<'_, PyTuple>,
    center_x: &Bound<'_, PyAny>,
    center_y: &Bound<'_, PyAny>,
    angle_radians: &Bound<'_, PyAny>,
) -> PyResult<SessionOperation> {
    if !targets.is_exact_instance_of::<PyTuple>() {
        return Err(operation_validation_error(
            py,
            "atom rotation targets must be an exact built-in tuple".to_owned(),
        ));
    }
    let targets = targets
        .iter()
        .map(|value| {
            value
                .extract::<PyRef<'_, PyDocumentAtomRotationTargetV1>>()
                .map(|target| target.target.clone())
                .map_err(Into::into)
        })
        .collect::<PyResult<Vec<_>>>()?;
    let rotation = AtomRotationV1::new(
        targets,
        exact_finite(py, center_x, "center x")?,
        exact_finite(py, center_y, "center y")?,
        exact_finite(py, angle_radians, "angle radians")?,
    )
    .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(SessionOperation::V1(SessionOperationV1::RotateAtoms {
        rotation,
    }))
}

pub(crate) fn live_targets(
    py: Python<'_>,
    values: &Bound<'_, PyTuple>,
) -> PyResult<Vec<(DocumentObjectIdV1, DocumentObjectIdV1)>> {
    values
        .iter()
        .map(|value| {
            let pair = value.cast::<PyTuple>().map_err(|_| {
                operation_validation_error(
                    py,
                    "live atom rotation targets must contain exact built-in pairs".to_owned(),
                )
            })?;
            if pair.len() != 2 {
                return Err(operation_validation_error(
                    py,
                    "live atom rotation targets must contain molecule and atom IDs".to_owned(),
                ));
            }
            let molecule = pair.get_item(0)?.extract::<String>()?;
            let atom = pair.get_item(1)?.extract::<String>()?;
            Ok((
                document_object_id(py, molecule)?,
                document_object_id(py, atom)?,
            ))
        })
        .collect()
}

pub(crate) fn rotation(
    py: Python<'_>,
    targets: Vec<AtomRotationTargetV1>,
    center_x: &Bound<'_, PyAny>,
    center_y: &Bound<'_, PyAny>,
    angle_radians: &Bound<'_, PyAny>,
) -> PyResult<SessionOperation> {
    let rotation = AtomRotationV1::new(
        targets,
        exact_finite(py, center_x, "center x")?,
        exact_finite(py, center_y, "center y")?,
        exact_finite(py, angle_radians, "angle radians")?,
    )
    .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(SessionOperation::V1(SessionOperationV1::RotateAtoms {
        rotation,
    }))
}

fn exact_finite(py: Python<'_>, value: &Bound<'_, PyAny>, field: &str) -> PyResult<f64> {
    if !(value.is_exact_instance_of::<PyFloat>() || value.is_exact_instance_of::<PyInt>())
        || value.is_instance_of::<PyBool>()
    {
        return Err(operation_validation_error(
            py,
            format!("atom rotation {field} must be an exact int or float"),
        ));
    }
    let value = value.extract::<f64>().map_err(|_| {
        operation_validation_error(py, format!("atom rotation {field} is outside finite f64"))
    })?;
    value.is_finite().then_some(value).ok_or_else(|| {
        operation_validation_error(py, format!("atom rotation {field} must be finite"))
    })
}
