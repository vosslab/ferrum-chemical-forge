//! Exact Python factory for implemented Rust geometry repair.

use ferrum_document::{
    GeometryRepairKindV1, GeometryRepairV1, SessionOperation, SessionOperationV1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyFloat, PyInt, PyString, PyTuple};

use super::binding::operation_validation_error;

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DocumentGeometryRepairKindV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyDocumentGeometryRepairKindV1 {
    SnapToHexGrid,
    StraightenBonds,
    NormalizeBondLengths,
    NormalizeBondAngles,
    NormalizeRings,
}

impl From<PyDocumentGeometryRepairKindV1> for GeometryRepairKindV1 {
    fn from(value: PyDocumentGeometryRepairKindV1) -> Self {
        match value {
            PyDocumentGeometryRepairKindV1::SnapToHexGrid => Self::SnapToHexGrid,
            PyDocumentGeometryRepairKindV1::StraightenBonds => Self::StraightenBonds,
            PyDocumentGeometryRepairKindV1::NormalizeBondLengths => Self::NormalizeBondLengths,
            PyDocumentGeometryRepairKindV1::NormalizeBondAngles => Self::NormalizeBondAngles,
            PyDocumentGeometryRepairKindV1::NormalizeRings => Self::NormalizeRings,
        }
    }
}

pub(crate) fn repair_geometry(
    py: Python<'_>,
    molecule_ids: &Bound<'_, PyTuple>,
    kind: PyRef<'_, PyDocumentGeometryRepairKindV1>,
    target_spacing_points: &Bound<'_, PyAny>,
) -> PyResult<SessionOperation> {
    if !molecule_ids.is_exact_instance_of::<PyTuple>() {
        return Err(operation_validation_error(
            py,
            "geometry repair molecule IDs must be an exact built-in tuple".to_owned(),
        ));
    }
    let molecule_ids = molecule_ids
        .iter()
        .map(|value| {
            if !value.is_exact_instance_of::<PyString>() {
                return Err(operation_validation_error(
                    py,
                    "geometry repair molecule IDs must contain exact strings".to_owned(),
                ));
            }
            value.extract::<String>()
        })
        .collect::<PyResult<Vec<_>>>()?;
    let repair = GeometryRepairV1::new(
        molecule_ids,
        (*kind).into(),
        exact_positive_finite(py, target_spacing_points)?,
    )
    .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(SessionOperation::V1(SessionOperationV1::RepairGeometry {
        repair,
    }))
}

fn exact_positive_finite(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<f64> {
    if !(value.is_exact_instance_of::<PyFloat>() || value.is_exact_instance_of::<PyInt>())
        || value.is_instance_of::<PyBool>()
    {
        return Err(operation_validation_error(
            py,
            "geometry repair target spacing must be an exact int or float".to_owned(),
        ));
    }
    let value = value.extract::<f64>().map_err(|_| {
        operation_validation_error(
            py,
            "geometry repair target spacing is outside finite f64".to_owned(),
        )
    })?;
    (value.is_finite() && value > 0.0)
        .then_some(value)
        .ok_or_else(|| {
            operation_validation_error(
                py,
                "geometry repair target spacing must be finite and greater than zero".to_owned(),
            )
        })
}
