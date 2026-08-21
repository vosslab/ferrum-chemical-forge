//! Bounded, worker-safe SDF batch preparation for document insertion.

use ferrum_chemistry::{
    ChemistryError as RustChemistryError, NativeChemEngine, interchange_record_from_sdf_v1,
    validate_sdf_input,
};
use ferrum_document::{InterchangeRecordBuildErrorV1, build_interchange_record_batch_insertion_v1};
use pyo3::prelude::*;

use super::geometry_binding::PyInsertionPlacementV1;
use super::interchange_insertion_binding::PyInterchangeRecordBatchInsertionV1;
use super::smiles_insertion_binding::{
    MoleculeInsertionError, map_complete_graph_error, structured_insertion_error,
};

const OPERATION: &str = "prepare_sdf_molecules_v1";

enum NativePreparationFailure {
    Load(RustChemistryError),
    Parse(RustChemistryError),
    Build(InterchangeRecordBuildErrorV1),
}

/// Parse and place all already-allocated SDF text without touching a session.
#[pyfunction]
fn prepare_sdf_molecules_v1(
    py: Python<'_>,
    source: String,
    placement: PyRef<'_, PyInsertionPlacementV1>,
) -> PyResult<PyInterchangeRecordBatchInsertionV1> {
    if let Err(error) = validate_sdf_input(&source) {
        return Err(super::chemistry_binding::map_chemistry_error(py, error)?);
    }
    prepare_source(py, source, placement.placement())
}

fn prepare_source(
    py: Python<'_>,
    source: String,
    placement: ferrum_geometry::MoleculePlacementV1,
) -> PyResult<PyInterchangeRecordBatchInsertionV1> {
    let library_path = super::chemistry_binding::packaged_library_path(py, OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine =
            NativeChemEngine::load(&worker_path).map_err(NativePreparationFailure::Load)?;
        let records = engine
            .sdf_to_records(&source)
            .map_err(NativePreparationFailure::Parse)?
            .into_iter()
            .map(interchange_record_from_sdf_v1)
            .collect::<Vec<_>>();
        build_interchange_record_batch_insertion_v1(&engine, &records, placement)
            .map_err(NativePreparationFailure::Build)
    });
    match result {
        Ok(batch) => Ok(PyInterchangeRecordBatchInsertionV1::new(batch)),
        Err(NativePreparationFailure::Load(error)) => Err(
            super::chemistry_binding::map_load_error(py, OPERATION, &library_path, error)?,
        ),
        Err(NativePreparationFailure::Parse(error)) => {
            Err(super::chemistry_binding::map_packaged_operation_error(
                py,
                OPERATION,
                &library_path,
                error,
            )?)
        }
        Err(NativePreparationFailure::Build(error)) => {
            Err(map_build_error(py, &library_path, error)?)
        }
    }
}

fn map_build_error(
    py: Python<'_>,
    library_path: &std::path::Path,
    error: InterchangeRecordBuildErrorV1,
) -> PyResult<PyErr> {
    match error {
        InterchangeRecordBuildErrorV1::Chemistry(error) => {
            super::chemistry_binding::map_packaged_operation_error(
                py,
                OPERATION,
                library_path,
                error,
            )
        }
        InterchangeRecordBuildErrorV1::CompleteGraph(error) => map_complete_graph_error(py, error),
        InterchangeRecordBuildErrorV1::Geometry(error) => Ok(
            super::geometry_binding::geometry_error(py, error.to_string()),
        ),
        InterchangeRecordBuildErrorV1::Position(error) => {
            super::binding::projection_error(py, error)
        }
        InterchangeRecordBuildErrorV1::KekulizeOptions(error) => {
            structured_insertion_error(py, MoleculeInsertionError::new_err, error)
        }
        InterchangeRecordBuildErrorV1::Insertion(error) => {
            structured_insertion_error(py, MoleculeInsertionError::new_err, error)
        }
        InterchangeRecordBuildErrorV1::Metadata(error) => {
            structured_insertion_error(py, MoleculeInsertionError::new_err, error)
        }
    }
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(prepare_sdf_molecules_v1, module)?)?;
    Ok(())
}
