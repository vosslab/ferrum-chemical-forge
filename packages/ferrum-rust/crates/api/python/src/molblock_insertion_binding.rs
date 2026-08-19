//! Bounded, worker-safe molblock preparation for document insertion.

use std::path::PathBuf;

use ferrum_chemistry::{
    ChemistryError as RustChemistryError, NativeChemEngine, validate_molblock_input,
};
use ferrum_document::{
    MolblockMoleculeBuildError, build_molblock_molecule_insertion_v1, read_molblock_file_v1,
};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString};

use crate::geometry_binding::PyInsertionPlacementV1;
use crate::smiles_insertion_binding::PyMoleculeInsertionV1;

const OPERATION: &str = "prepare_molblock_molecule_v1";

enum NativePreparationFailure {
    Load(RustChemistryError),
    Build(MolblockMoleculeBuildError),
}

/// Parse and place already-allocated molblock text without touching a session.
#[pyfunction]
fn prepare_molblock_molecule_v1(
    py: Python<'_>,
    molblock: String,
    placement: PyRef<'_, PyInsertionPlacementV1>,
) -> PyResult<PyMoleculeInsertionV1> {
    if let Err(error) = validate_molblock_input(&molblock) {
        return Err(crate::chemistry_binding::map_chemistry_error(py, error)?);
    }
    prepare_source(py, molblock, placement.placement())
}

/// Read, parse, and place one local molblock under the native operation byte limit.
///
/// The path must be an exact built-in string. Rust opens and bounds the file before
/// UTF-8 decoding, validates the molblock before native library loading, and returns
/// only immutable handle-free insertion facts.
#[pyfunction]
fn prepare_molblock_file_v1(
    py: Python<'_>,
    path: &Bound<'_, PyAny>,
    placement: PyRef<'_, PyInsertionPlacementV1>,
) -> PyResult<PyMoleculeInsertionV1> {
    let path = exact_path(path)?;
    let source = match py.detach(move || read_molblock_file_v1(&path)) {
        Ok(source) => source,
        Err(error) => {
            return Err(crate::smiles_insertion_binding::map_molblock_source_error(
                py, error,
            )?);
        }
    };
    if let Err(error) = validate_molblock_input(&source) {
        return Err(crate::chemistry_binding::map_chemistry_error(py, error)?);
    }
    prepare_source(py, source, placement.placement())
}

fn prepare_source(
    py: Python<'_>,
    source: String,
    placement: ferrum_geometry::MoleculePlacementV1,
) -> PyResult<PyMoleculeInsertionV1> {
    let library_path = crate::chemistry_binding::packaged_library_path(py, OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine =
            NativeChemEngine::load(&worker_path).map_err(NativePreparationFailure::Load)?;
        build_molblock_molecule_insertion_v1(&engine, &source, placement)
            .map_err(NativePreparationFailure::Build)
    });
    match result {
        Ok(insertion) => Ok(PyMoleculeInsertionV1::new(insertion)),
        Err(NativePreparationFailure::Load(error)) => Err(
            crate::chemistry_binding::map_load_error(py, OPERATION, &library_path, error)?,
        ),
        Err(NativePreparationFailure::Build(error)) => Err(
            crate::smiles_insertion_binding::map_molblock_build_error(py, &library_path, error)?,
        ),
    }
}

fn exact_path(path: &Bound<'_, PyAny>) -> PyResult<PathBuf> {
    if !path.is_exact_instance_of::<PyString>() {
        return Err(PyTypeError::new_err(
            "molblock file path must be an exact built-in str",
        ));
    }
    Ok(PathBuf::from(path.cast::<PyString>()?.to_str()?))
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(prepare_molblock_molecule_v1, module)?)?;
    module.add_function(wrap_pyfunction!(prepare_molblock_file_v1, module)?)?;
    Ok(())
}
