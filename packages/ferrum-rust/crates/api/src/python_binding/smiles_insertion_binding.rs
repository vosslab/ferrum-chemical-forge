//! Frozen, worker-safe SMILES molecule preparation for document insertion.

use ferrum_chemistry::{
    validate_smiles_input, ChemistryError as RustChemistryError, NativeChemEngine,
};
use ferrum_document::MoleculeInsertionV1;
use ferrum_document::{
    build_smiles_molecule_insertion_v1, CompleteGraphMoleculeInsertionError,
    MolblockMoleculeBuildError, MolblockSourceErrorV1, SmilesMoleculeBuildError,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::binding::{projection_error, FerrumError};
use super::geometry_binding::PyInsertionPlacementV1;

create_exception!(ferrum_chem, MoleculeInsertionError, FerrumError);
create_exception!(
    ferrum_chem,
    UnsupportedMoleculeInsertionError,
    MoleculeInsertionError
);
create_exception!(ferrum_chem, MolblockInputError, MoleculeInsertionError);

const OPERATION: &str = "prepare_smiles_molecule_v1";

/// One immutable, native-handle-free molecule ready for a session transaction.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "MoleculeInsertionV1",
    skip_from_py_object
)]
pub(crate) struct PyMoleculeInsertionV1 {
    insertion: MoleculeInsertionV1,
}

impl PyMoleculeInsertionV1 {
    pub(crate) fn new(insertion: MoleculeInsertionV1) -> Self {
        Self { insertion }
    }

    pub(crate) fn insertion(&self) -> &MoleculeInsertionV1 {
        &self.insertion
    }
}

#[pymethods]
impl PyMoleculeInsertionV1 {
    /// Return the number of source-ordered atoms in this complete graph.
    #[getter]
    fn atom_count(&self) -> usize {
        self.insertion.atoms().len()
    }

    /// Return the number of source-ordered bonds in this complete graph.
    #[getter]
    fn bond_count(&self) -> usize {
        self.insertion.bonds().len()
    }
}

enum NativePreparationFailure {
    Load(RustChemistryError),
    Build(SmilesMoleculeBuildError),
}

/// Parse and place a molecule without borrowing or mutating a document session.
///
/// Native chemistry runs while Python is detached. The returned value contains only
/// owned Ferrum facts and can be delivered from a Qt worker to the UI-thread session.
#[pyfunction]
pub(crate) fn prepare_smiles_molecule_v1(
    py: Python<'_>,
    smiles: String,
    placement: PyRef<'_, PyInsertionPlacementV1>,
) -> PyResult<PyMoleculeInsertionV1> {
    if let Err(error) = validate_smiles_input(&smiles) {
        return Err(super::chemistry_binding::map_chemistry_error(py, error)?);
    }
    let placement = placement.placement();
    let library_path = super::chemistry_binding::packaged_library_path(py, OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine =
            NativeChemEngine::load(&worker_path).map_err(NativePreparationFailure::Load)?;
        build_smiles_molecule_insertion_v1(&engine, &smiles, placement)
            .map_err(NativePreparationFailure::Build)
    });
    match result {
        Ok(insertion) => Ok(PyMoleculeInsertionV1::new(insertion)),
        Err(NativePreparationFailure::Load(error)) => Err(
            super::chemistry_binding::map_load_error(py, OPERATION, &library_path, error)?,
        ),
        Err(NativePreparationFailure::Build(error)) => {
            Err(map_build_error(py, &library_path, error)?)
        }
    }
}

pub(crate) fn map_build_error(
    py: Python<'_>,
    library_path: &std::path::Path,
    error: SmilesMoleculeBuildError,
) -> PyResult<PyErr> {
    match error {
        SmilesMoleculeBuildError::Chemistry(error) => {
            super::chemistry_binding::map_packaged_operation_error(
                py,
                OPERATION,
                library_path,
                error,
            )
        }
        SmilesMoleculeBuildError::Geometry(error) => Ok(super::geometry_binding::geometry_error(
            py,
            error.to_string(),
        )),
        SmilesMoleculeBuildError::Position(error) => projection_error(py, error),
        SmilesMoleculeBuildError::MissingCoordinates
        | SmilesMoleculeBuildError::UnsupportedAtomFact { .. }
        | SmilesMoleculeBuildError::UnsupportedBondFact { .. }
        | SmilesMoleculeBuildError::UnsupportedBondOrder { .. } => {
            structured_insertion_error(py, UnsupportedMoleculeInsertionError::new_err, error)
        }
        SmilesMoleculeBuildError::KekulizeOptions(_) | SmilesMoleculeBuildError::Insertion(_) => {
            structured_insertion_error(py, MoleculeInsertionError::new_err, error)
        }
    }
}

pub(crate) fn structured_insertion_error(
    py: Python<'_>,
    constructor: impl FnOnce(String) -> PyErr,
    error: impl std::fmt::Display,
) -> PyResult<PyErr> {
    let reason = error.to_string();
    let py_error = constructor(reason.clone());
    py_error.value(py).setattr("reason", reason)?;
    Ok(py_error)
}

pub(crate) fn map_molblock_build_error(
    py: Python<'_>,
    library_path: &std::path::Path,
    error: MolblockMoleculeBuildError,
) -> PyResult<PyErr> {
    match error {
        MolblockMoleculeBuildError::Chemistry(error) => {
            super::chemistry_binding::map_packaged_operation_error(
                py,
                "prepare_molblock_molecule_v1",
                library_path,
                error,
            )
        }
        MolblockMoleculeBuildError::KekulizeOptions(error) => {
            structured_insertion_error(py, MoleculeInsertionError::new_err, error)
        }
        MolblockMoleculeBuildError::CompleteGraph(error) => map_complete_graph_error(py, error),
    }
}

pub(crate) fn map_complete_graph_error(
    py: Python<'_>,
    error: CompleteGraphMoleculeInsertionError,
) -> PyResult<PyErr> {
    match error {
        CompleteGraphMoleculeInsertionError::Geometry(error) => Ok(
            super::geometry_binding::geometry_error(py, error.to_string()),
        ),
        CompleteGraphMoleculeInsertionError::Position(error) => projection_error(py, error),
        CompleteGraphMoleculeInsertionError::UnsupportedAtomFact { .. }
        | CompleteGraphMoleculeInsertionError::UnsupportedBondFact { .. }
        | CompleteGraphMoleculeInsertionError::UnsupportedBondOrder { .. } => {
            structured_insertion_error(py, UnsupportedMoleculeInsertionError::new_err, error)
        }
        CompleteGraphMoleculeInsertionError::MissingCoordinates
        | CompleteGraphMoleculeInsertionError::CoordinateCountMismatch { .. }
        | CompleteGraphMoleculeInsertionError::NonFiniteCoordinate { .. }
        | CompleteGraphMoleculeInsertionError::Insertion(_) => {
            structured_insertion_error(py, MoleculeInsertionError::new_err, error)
        }
    }
}

pub(crate) fn map_molblock_source_error(
    py: Python<'_>,
    error: MolblockSourceErrorV1,
) -> PyResult<PyErr> {
    let reason = error.to_string();
    let (path, stage, limit, observed_at_least) = match error {
        MolblockSourceErrorV1::Read { path, .. } => (path, "read", None, None),
        MolblockSourceErrorV1::NonRegularFile { path } => (path, "source_policy", None, None),
        MolblockSourceErrorV1::LimitUnrepresentable { maximum_bytes } => (
            std::path::PathBuf::new(),
            "source_policy",
            Some(maximum_bytes),
            None,
        ),
        MolblockSourceErrorV1::ByteLimitExceeded {
            path,
            limit,
            observed_at_least,
        } => (path, "bytes", Some(limit), Some(observed_at_least)),
        MolblockSourceErrorV1::Utf8 { path, .. } => (path, "utf8", None, None),
    };
    let py_error = MolblockInputError::new_err(reason.clone());
    let value = py_error.value(py);
    value.setattr("reason", reason)?;
    value.setattr("path", path.to_string_lossy().as_ref())?;
    value.setattr("stage", stage)?;
    value.setattr("limit", limit)?;
    value.setattr("observed_at_least", observed_at_least)?;
    Ok(py_error)
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "MoleculeInsertionError",
        module.py().get_type::<MoleculeInsertionError>(),
    )?;
    module.add(
        "UnsupportedMoleculeInsertionError",
        module.py().get_type::<UnsupportedMoleculeInsertionError>(),
    )?;
    module.add(
        "MolblockInputError",
        module.py().get_type::<MolblockInputError>(),
    )?;
    module.add_class::<PyMoleculeInsertionV1>()?;
    module.add_function(wrap_pyfunction!(prepare_smiles_molecule_v1, module)?)?;
    Ok(())
}
