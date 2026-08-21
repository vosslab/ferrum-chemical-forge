//! Bounded, worker-safe SDF batch preparation for document insertion.

use std::path::PathBuf;

use ferrum_chemistry::{
    ChemistryError as RustChemistryError, NativeChemEngine, validate_sdf_input,
};
use ferrum_document::{PendingCreateSdfRecords, SdfRecordBatchInsertionV1};
use ferrum_document::{
    SdfMoleculeBuildError, SdfSourceErrorV1, build_sdf_record_batch_insertion_v1, read_sdf_file_v1,
};
use pyo3::create_exception;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString, PyTuple};

use super::geometry_binding::PyInsertionPlacementV1;
use super::smiles_insertion_binding::{
    MoleculeInsertionError, map_complete_graph_error, structured_insertion_error,
};

create_exception!(ferrum_chem, SdfInputError, MoleculeInsertionError);

const OPERATION: &str = "prepare_sdf_molecules_v1";

/// One immutable, native-handle-free ordered SDF insertion batch.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "SdfMoleculeBatchInsertionV1",
    skip_from_py_object
)]
pub(crate) struct PySdfMoleculeBatchInsertionV1 {
    batch: SdfRecordBatchInsertionV1,
}

impl PySdfMoleculeBatchInsertionV1 {
    pub(crate) fn batch(&self) -> &SdfRecordBatchInsertionV1 {
        &self.batch
    }
}

#[pymethods]
impl PySdfMoleculeBatchInsertionV1 {
    /// Return the number of source-ordered records retained by this batch.
    #[getter]
    fn record_count(&self) -> usize {
        self.batch.records().len()
    }
}

/// Opaque one-use prepared SDF batch insertion.
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PreparedSdfRecordInsertion"
)]
pub(crate) struct PyPreparedSdfRecordInsertion {
    pub(crate) pending: PendingCreateSdfRecords,
    molecule_identifiers: Vec<String>,
    atom_identifiers: Vec<Vec<String>>,
    bond_identifiers: Vec<Vec<String>>,
}

impl PyPreparedSdfRecordInsertion {
    pub(crate) fn new(pending: PendingCreateSdfRecords) -> Self {
        let molecule_identifiers = pending
            .molecule_identifiers()
            .iter()
            .map(|identifier| identifier.as_str().to_owned())
            .collect();
        let atom_identifiers = copied_identifier_groups(pending.atom_identifiers());
        let bond_identifiers = copied_identifier_groups(pending.bond_identifiers());
        Self {
            pending,
            molecule_identifiers,
            atom_identifiers,
            bond_identifiers,
        }
    }
}

#[pymethods]
impl PyPreparedSdfRecordInsertion {
    /// Return durable molecule IDs in exact SDF record order.
    #[getter]
    fn molecule_identifiers(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, &self.molecule_identifiers)?.unbind())
    }

    /// Return record-grouped durable atom IDs in exact source order.
    #[getter]
    fn atom_identifiers(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        nested_tuple(py, &self.atom_identifiers)
    }

    /// Return record-grouped durable bond IDs in exact source order.
    #[getter]
    fn bond_identifiers(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        nested_tuple(py, &self.bond_identifiers)
    }
}

enum NativePreparationFailure {
    Load(RustChemistryError),
    Parse(RustChemistryError),
    Build(SdfMoleculeBuildError),
}

/// Parse and place all already-allocated SDF text without touching a session.
#[pyfunction]
fn prepare_sdf_molecules_v1(
    py: Python<'_>,
    source: String,
    placement: PyRef<'_, PyInsertionPlacementV1>,
) -> PyResult<PySdfMoleculeBatchInsertionV1> {
    if let Err(error) = validate_sdf_input(&source) {
        return Err(super::chemistry_binding::map_chemistry_error(py, error)?);
    }
    prepare_source(py, source, placement.placement())
}

/// Read and prepare all records from one local SDF under the ABI byte ceiling.
#[pyfunction]
fn prepare_sdf_file_v1(
    py: Python<'_>,
    path: &Bound<'_, PyAny>,
    placement: PyRef<'_, PyInsertionPlacementV1>,
) -> PyResult<PySdfMoleculeBatchInsertionV1> {
    let path = exact_path(path)?;
    let source = match py.detach(move || read_sdf_file_v1(&path)) {
        Ok(source) => source,
        Err(error) => return Err(map_source_error(py, error)?),
    };
    if let Err(error) = validate_sdf_input(&source) {
        return Err(super::chemistry_binding::map_chemistry_error(py, error)?);
    }
    prepare_source(py, source, placement.placement())
}

fn prepare_source(
    py: Python<'_>,
    source: String,
    placement: ferrum_geometry::MoleculePlacementV1,
) -> PyResult<PySdfMoleculeBatchInsertionV1> {
    let library_path = super::chemistry_binding::packaged_library_path(py, OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine =
            NativeChemEngine::load(&worker_path).map_err(NativePreparationFailure::Load)?;
        let records = engine
            .sdf_to_records(&source)
            .map_err(NativePreparationFailure::Parse)?;
        build_sdf_record_batch_insertion_v1(&engine, &records, placement)
            .map_err(NativePreparationFailure::Build)
    });
    match result {
        Ok(batch) => Ok(PySdfMoleculeBatchInsertionV1 { batch }),
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
    error: SdfMoleculeBuildError,
) -> PyResult<PyErr> {
    match error {
        SdfMoleculeBuildError::Chemistry(error) => {
            super::chemistry_binding::map_packaged_operation_error(
                py,
                OPERATION,
                library_path,
                error,
            )
        }
        SdfMoleculeBuildError::CompleteGraph(error) => map_complete_graph_error(py, error),
        SdfMoleculeBuildError::Geometry(error) => Ok(super::geometry_binding::geometry_error(
            py,
            error.to_string(),
        )),
        SdfMoleculeBuildError::Position(error) => super::binding::projection_error(py, error),
        SdfMoleculeBuildError::KekulizeOptions(error) => {
            structured_insertion_error(py, MoleculeInsertionError::new_err, error)
        }
        SdfMoleculeBuildError::Insertion(error) => {
            structured_insertion_error(py, MoleculeInsertionError::new_err, error)
        }
        SdfMoleculeBuildError::Metadata(error) => {
            structured_insertion_error(py, MoleculeInsertionError::new_err, error)
        }
    }
}

fn map_source_error(py: Python<'_>, error: SdfSourceErrorV1) -> PyResult<PyErr> {
    let reason = error.to_string();
    let (path, stage, limit, observed_at_least) = match error {
        SdfSourceErrorV1::Read { path, .. } => (path, "read", None, None),
        SdfSourceErrorV1::NonRegularFile { path } => (path, "source_policy", None, None),
        SdfSourceErrorV1::LimitUnrepresentable { maximum_bytes } => {
            (PathBuf::new(), "source_policy", Some(maximum_bytes), None)
        }
        SdfSourceErrorV1::ByteLimitExceeded {
            path,
            limit,
            observed_at_least,
        } => (path, "bytes", Some(limit), Some(observed_at_least)),
        SdfSourceErrorV1::Utf8 { path, .. } => (path, "utf8", None, None),
    };
    let py_error = SdfInputError::new_err(reason.clone());
    let value = py_error.value(py);
    value.setattr("reason", reason)?;
    value.setattr("path", path.to_string_lossy().as_ref())?;
    value.setattr("stage", stage)?;
    value.setattr("limit", limit)?;
    value.setattr("observed_at_least", observed_at_least)?;
    Ok(py_error)
}

fn exact_path(path: &Bound<'_, PyAny>) -> PyResult<PathBuf> {
    if !path.is_exact_instance_of::<PyString>() {
        return Err(PyTypeError::new_err(
            "SDF file path must be an exact built-in str",
        ));
    }
    Ok(PathBuf::from(path.cast::<PyString>()?.to_str()?))
}

fn copied_identifier_groups(groups: &[Vec<ferrum_document::PersistentId>]) -> Vec<Vec<String>> {
    groups
        .iter()
        .map(|group| {
            group
                .iter()
                .map(|identifier| identifier.as_str().to_owned())
                .collect()
        })
        .collect()
}

fn nested_tuple(py: Python<'_>, groups: &[Vec<String>]) -> PyResult<Py<PyTuple>> {
    let groups = groups
        .iter()
        .map(|group| PyTuple::new(py, group))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyTuple::new(py, groups)?.unbind())
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("SdfInputError", module.py().get_type::<SdfInputError>())?;
    module.add_class::<PySdfMoleculeBatchInsertionV1>()?;
    module.add_class::<PyPreparedSdfRecordInsertion>()?;
    module.add_function(wrap_pyfunction!(prepare_sdf_molecules_v1, module)?)?;
    module.add_function(wrap_pyfunction!(prepare_sdf_file_v1, module)?)?;
    Ok(())
}
