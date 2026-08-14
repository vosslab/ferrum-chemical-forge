//! Worker-safe existing-molecule coordinate preparation.

use std::collections::HashSet;

use ferrum_api::{
    CleanGeometryBuildError, MoleculeCoordinateBuildError, build_clean_geometry_update_v1,
    build_molecule_coordinate_update_v1,
};
use ferrum_chemistry::{ChemistryError as RustChemistryError, NativeChemEngine};
use ferrum_document::{CleanGeometryUpdateV1, DocumentObjectIdV1, MoleculeCoordinateUpdateV1};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyFloat, PyInt, PyString, PyTuple};

use crate::binding::{FerrumError, projection_error};
use crate::projection_binding::PySessionDocumentObservationV1;

create_exception!(ferrum_chem, MoleculeCoordinateError, FerrumError);
create_exception!(
    ferrum_chem,
    UnsupportedMoleculeCoordinateError,
    MoleculeCoordinateError
);

const MOLECULE_OPERATION: &str = "prepare_molecule_coordinates_v1";
const CLEAN_OPERATION: &str = "prepare_clean_geometry_v1";

/// Immutable native-handle-free coordinates prepared from one exact observation.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PreparedMoleculeCoordinatesV1",
    skip_from_py_object
)]
pub(crate) struct PyPreparedMoleculeCoordinatesV1 {
    update: MoleculeCoordinateUpdateV1,
}

impl PyPreparedMoleculeCoordinatesV1 {
    pub(crate) fn update(&self) -> &MoleculeCoordinateUpdateV1 {
        &self.update
    }
}

#[pymethods]
impl PyPreparedMoleculeCoordinatesV1 {
    #[getter]
    fn molecule_id(&self) -> String {
        self.update.molecule_id().as_str().to_owned()
    }

    #[getter]
    fn atom_count(&self) -> usize {
        self.update.positions().len()
    }

    #[getter]
    fn source_revision(&self) -> u64 {
        self.update.source_revision()
    }

    #[getter]
    fn source_digest(&self) -> String {
        hex_digest(self.update.source_digest())
    }
}

/// Immutable native-handle-free clean geometry prepared for one exact observation.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PreparedCleanGeometryV1",
    skip_from_py_object
)]
pub(crate) struct PyPreparedCleanGeometryV1 {
    update: CleanGeometryUpdateV1,
}

impl PyPreparedCleanGeometryV1 {
    pub(crate) fn update(&self) -> &CleanGeometryUpdateV1 {
        &self.update
    }
}

#[pymethods]
impl PyPreparedCleanGeometryV1 {
    #[getter]
    fn molecule_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        PyTuple::new(
            py,
            self.update
                .molecules()
                .iter()
                .map(|molecule| molecule.molecule_id().as_str()),
        )
        .map(Bound::unbind)
    }

    #[getter]
    fn atom_counts(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        PyTuple::new(
            py,
            self.update
                .molecules()
                .iter()
                .map(|molecule| molecule.positions().len()),
        )
        .map(Bound::unbind)
    }

    #[getter]
    fn source_revision(&self) -> u64 {
        self.update.source_revision()
    }

    #[getter]
    fn source_digest(&self) -> String {
        hex_digest(self.update.source_digest())
    }
}

enum NativePreparationFailure {
    Load(RustChemistryError),
    Molecule(MoleculeCoordinateBuildError),
    Clean(CleanGeometryBuildError),
}

/// Generate a complete coordinate update without borrowing a document session.
///
/// Python is detached while the packaged native engine runs. The immutable result
/// remains bound to the source observation's revision, digest, and molecule.
#[pyfunction]
pub(crate) fn prepare_molecule_coordinates_v1(
    py: Python<'_>,
    observation: PyRef<'_, PySessionDocumentObservationV1>,
    molecule_id: String,
) -> PyResult<PyPreparedMoleculeCoordinatesV1> {
    let molecule_id = DocumentObjectIdV1::parse(molecule_id)
        .map_err(|error| crate::binding::operation_validation_error(py, error.to_string()))?;
    let observation = observation.observation().clone();
    let library_path = crate::chemistry_binding::packaged_library_path(py, MOLECULE_OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine =
            NativeChemEngine::load(&worker_path).map_err(NativePreparationFailure::Load)?;
        build_molecule_coordinate_update_v1(&engine, &observation, &molecule_id)
            .map_err(NativePreparationFailure::Molecule)
    });
    match result {
        Ok(update) => Ok(PyPreparedMoleculeCoordinatesV1 { update }),
        Err(NativePreparationFailure::Load(error)) => Err(
            crate::chemistry_binding::map_load_error(py, MOLECULE_OPERATION, &library_path, error)?,
        ),
        Err(NativePreparationFailure::Molecule(error)) => Err(map_build_error(
            py,
            MOLECULE_OPERATION,
            &library_path,
            error,
        )?),
        Err(NativePreparationFailure::Clean(_)) => {
            unreachable!("molecule preparation maps Molecule")
        }
    }
}

/// Regenerate every selected bonded molecule at one explicit authored spacing.
#[pyfunction]
pub(crate) fn prepare_clean_geometry_v1(
    py: Python<'_>,
    observation: PyRef<'_, PySessionDocumentObservationV1>,
    molecule_ids: &Bound<'_, PyTuple>,
    target_spacing_points: &Bound<'_, PyAny>,
) -> PyResult<PyPreparedCleanGeometryV1> {
    if !molecule_ids.is_exact_instance_of::<PyTuple>() {
        return Err(crate::binding::operation_validation_error(
            py,
            "clean geometry molecule IDs must be an exact built-in tuple".to_owned(),
        ));
    }
    let maximum_targets = observation.observation().projection().molecules().len();
    if molecule_ids.is_empty() || molecule_ids.len() > maximum_targets {
        return Err(crate::binding::operation_validation_error(
            py,
            "clean geometry requires a nonempty tuple no larger than the observed molecule set"
                .to_owned(),
        ));
    }
    let molecule_ids = molecule_ids
        .iter()
        .map(|value| {
            if !value.is_exact_instance_of::<PyString>() {
                return Err(crate::binding::operation_validation_error(
                    py,
                    "clean geometry molecule IDs must contain exact strings".to_owned(),
                ));
            }
            let value = value.extract::<String>()?;
            DocumentObjectIdV1::parse(value)
                .map_err(|error| crate::binding::operation_validation_error(py, error.to_string()))
        })
        .collect::<PyResult<Vec<_>>>()?;
    let mut unique = HashSet::with_capacity(molecule_ids.len());
    if molecule_ids.iter().any(|id| !unique.insert(id.clone())) {
        return Err(crate::binding::operation_validation_error(
            py,
            "clean geometry molecule IDs must be unique".to_owned(),
        ));
    }
    let target_spacing_points = exact_positive_finite(py, target_spacing_points)?;
    let observation = observation.observation().clone();
    let library_path = crate::chemistry_binding::packaged_library_path(py, CLEAN_OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine =
            NativeChemEngine::load(&worker_path).map_err(NativePreparationFailure::Load)?;
        build_clean_geometry_update_v1(&engine, &observation, &molecule_ids, target_spacing_points)
            .map_err(NativePreparationFailure::Clean)
    });
    match result {
        Ok(update) => Ok(PyPreparedCleanGeometryV1 { update }),
        Err(NativePreparationFailure::Load(error)) => Err(
            crate::chemistry_binding::map_load_error(py, CLEAN_OPERATION, &library_path, error)?,
        ),
        Err(NativePreparationFailure::Clean(error)) => {
            Err(map_clean_build_error(py, &library_path, error)?)
        }
        Err(NativePreparationFailure::Molecule(_)) => {
            unreachable!("clean preparation maps Clean")
        }
    }
}

fn map_build_error(
    py: Python<'_>,
    operation: &'static str,
    library_path: &std::path::Path,
    error: MoleculeCoordinateBuildError,
) -> PyResult<PyErr> {
    match error {
        MoleculeCoordinateBuildError::Chemistry(error) => {
            crate::chemistry_binding::map_packaged_operation_error(
                py,
                operation,
                library_path,
                error,
            )
        }
        MoleculeCoordinateBuildError::Geometry(error) => Ok(
            crate::geometry_binding::geometry_error(py, error.to_string()),
        ),
        MoleculeCoordinateBuildError::Position(error) => projection_error(py, error),
        MoleculeCoordinateBuildError::UnknownMolecule { .. }
        | MoleculeCoordinateBuildError::EmptyMolecule
        | MoleculeCoordinateBuildError::UnsupportedVertex { .. }
        | MoleculeCoordinateBuildError::MissingElement { .. }
        | MoleculeCoordinateBuildError::InvalidElement { .. }
        | MoleculeCoordinateBuildError::UnsupportedAtomFact { .. }
        | MoleculeCoordinateBuildError::UnsupportedBondEndpoint { .. }
        | MoleculeCoordinateBuildError::UnsupportedBondStyle { .. }
        | MoleculeCoordinateBuildError::UnsupportedBondOrder { .. }
        | MoleculeCoordinateBuildError::NoUsableBondLength => {
            structured_error(py, UnsupportedMoleculeCoordinateError::new_err, error)
        }
        MoleculeCoordinateBuildError::Document(_)
        | MoleculeCoordinateBuildError::CoreProjection(_)
        | MoleculeCoordinateBuildError::DuplicateAtomIdentity { .. }
        | MoleculeCoordinateBuildError::Graph(_)
        | MoleculeCoordinateBuildError::ResourceAllocation
        | MoleculeCoordinateBuildError::Update(_) => {
            structured_error(py, MoleculeCoordinateError::new_err, error)
        }
    }
}

fn map_clean_build_error(
    py: Python<'_>,
    library_path: &std::path::Path,
    error: CleanGeometryBuildError,
) -> PyResult<PyErr> {
    match error {
        CleanGeometryBuildError::InvalidTargetSpacing
        | CleanGeometryBuildError::EmptyMolecules
        | CleanGeometryBuildError::DuplicateMolecule => Err(
            crate::binding::operation_validation_error(py, error.to_string()),
        ),
        CleanGeometryBuildError::Target { source, .. } => {
            map_build_error(py, CLEAN_OPERATION, library_path, source)
        }
        CleanGeometryBuildError::Geometry(error) => Ok(crate::geometry_binding::geometry_error(
            py,
            error.to_string(),
        )),
        CleanGeometryBuildError::UnknownMolecule { .. }
        | CleanGeometryBuildError::UnbondedMolecule { .. } => {
            structured_error(py, UnsupportedMoleculeCoordinateError::new_err, error)
        }
        CleanGeometryBuildError::Document(_)
        | CleanGeometryBuildError::CoreProjection(_)
        | CleanGeometryBuildError::GeneratedAtomCountMismatch { .. }
        | CleanGeometryBuildError::Update(_) => {
            structured_error(py, MoleculeCoordinateError::new_err, error)
        }
    }
}

fn exact_positive_finite(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<f64> {
    if !(value.is_exact_instance_of::<PyFloat>() || value.is_exact_instance_of::<PyInt>())
        || value.is_instance_of::<PyBool>()
    {
        return Err(crate::binding::operation_validation_error(
            py,
            "clean geometry target spacing must be an exact int or float".to_owned(),
        ));
    }
    let value = value.extract::<f64>().map_err(|_| {
        crate::binding::operation_validation_error(
            py,
            "clean geometry target spacing is outside finite f64".to_owned(),
        )
    })?;
    (value.is_finite() && value > 0.0)
        .then_some(value)
        .ok_or_else(|| {
            crate::binding::operation_validation_error(
                py,
                "clean geometry target spacing must be finite and greater than zero".to_owned(),
            )
        })
}

fn structured_error(
    py: Python<'_>,
    constructor: impl FnOnce(String) -> PyErr,
    error: impl std::fmt::Display,
) -> PyResult<PyErr> {
    let reason = error.to_string();
    let py_error = constructor(reason.clone());
    py_error.value(py).setattr("reason", reason)?;
    Ok(py_error)
}

fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "MoleculeCoordinateError",
        module.py().get_type::<MoleculeCoordinateError>(),
    )?;
    module.add(
        "UnsupportedMoleculeCoordinateError",
        module.py().get_type::<UnsupportedMoleculeCoordinateError>(),
    )?;
    module.add_class::<PyPreparedMoleculeCoordinatesV1>()?;
    module.add_class::<PyPreparedCleanGeometryV1>()?;
    module.add_function(wrap_pyfunction!(prepare_molecule_coordinates_v1, module)?)?;
    module.add_function(wrap_pyfunction!(prepare_clean_geometry_v1, module)?)?;
    Ok(())
}
