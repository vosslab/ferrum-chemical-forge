//! Frozen, worker-safe InChI molecule preparation for document insertion.

use ferrum_chemistry::{
    ChemistryError as RustChemistryError, NativeChemEngine, validate_inchi_input,
};
use ferrum_document::{InchiMoleculePreparationErrorV2, prepare_inchi_molecule_for_document_v2};
use pyo3::prelude::*;

use super::geometry_binding::PyInsertionPlacementV1;
use super::molecule_insertion_binding::{PyMoleculeInsertionV1, structured_insertion_error};
use super::smiles_insertion_binding::{MoleculeInsertionError, map_preparation_error};

const OPERATION: &str = "prepare_inchi_molecule_v1";

enum NativePreparationFailure {
    Load(RustChemistryError),
    Build(InchiMoleculePreparationErrorV2),
}

/// Parse and place one InChI molecule without borrowing a document session.
///
/// Native chemistry runs while Python is detached. The result contains only owned
/// Ferrum facts and can be delivered from a Qt worker to the UI-thread session.
#[pyfunction]
fn prepare_inchi_molecule_v1(
    py: Python<'_>,
    inchi: String,
    placement: PyRef<'_, PyInsertionPlacementV1>,
) -> PyResult<PyMoleculeInsertionV1> {
    if let Err(error) = validate_inchi_input(&inchi) {
        return Err(super::chemistry_binding::map_chemistry_error(py, error)?);
    }
    let placement = placement.placement();
    let library_path = super::chemistry_binding::packaged_library_path(py, OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine =
            NativeChemEngine::load(&worker_path).map_err(NativePreparationFailure::Load)?;
        prepare_inchi_molecule_for_document_v2(&engine, &inchi, placement)
            .map_err(NativePreparationFailure::Build)
    });
    match result {
        Ok(prepared) => match PyMoleculeInsertionV1::from_prepared(prepared) {
            Ok(insertion) => Ok(insertion),
            Err(error) => Err(structured_insertion_error(
                py,
                MoleculeInsertionError::new_err,
                error,
            )?),
        },
        Err(NativePreparationFailure::Load(error)) => Err(
            super::chemistry_binding::map_load_error(py, OPERATION, &library_path, error)?,
        ),
        Err(NativePreparationFailure::Build(error)) => {
            Err(map_build_error(py, &library_path, error)?)
        }
    }
}

fn map_build_error(
    py: Python<'_>,
    library_path: &std::path::Path,
    error: InchiMoleculePreparationErrorV2,
) -> PyResult<PyErr> {
    match error {
        InchiMoleculePreparationErrorV2::Chemistry(error) => {
            super::chemistry_binding::map_packaged_operation_error(
                py,
                OPERATION,
                library_path,
                error,
            )
        }
        InchiMoleculePreparationErrorV2::Preparation(error) => map_preparation_error(py, error),
    }
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(prepare_inchi_molecule_v1, module)?)?;
    Ok(())
}
