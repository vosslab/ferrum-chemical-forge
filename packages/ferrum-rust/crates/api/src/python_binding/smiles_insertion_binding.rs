//! Frozen, worker-safe SMILES molecule preparation for document insertion.

use ferrum_chemistry::{
    ChemistryError as RustChemistryError, NativeChemEngine, validate_smiles_input,
};
use ferrum_document::{
    DocumentMoleculePreparationErrorV2, MolblockMoleculeBuildError, MolblockSourceErrorV1,
    SmilesMoleculeBuildError, prepare_smiles_molecule_for_document_v2,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::binding::{FerrumError, projection_error};
use super::geometry_binding::PyInsertionPlacementV1;
use super::molecule_insertion_binding::{PyMoleculeInsertionV1, structured_insertion_error};

create_exception!(ferrum_chem, MoleculeInsertionError, FerrumError);
create_exception!(
    ferrum_chem,
    UnsupportedMoleculeInsertionError,
    MoleculeInsertionError
);
create_exception!(ferrum_chem, MolblockInputError, MoleculeInsertionError);

const OPERATION: &str = "prepare_smiles_molecule_v1";

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
        prepare_smiles_molecule_for_document_v2(&engine, &smiles, placement)
            .map_err(NativePreparationFailure::Build)
    });
    match result {
        Ok(prepared) => PyMoleculeInsertionV1::from_prepared(prepared)
            .map_err(|error| MoleculeInsertionError::new_err(error.to_string())),
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
        SmilesMoleculeBuildError::Preparation(error) => map_preparation_error(py, error),
        SmilesMoleculeBuildError::InvalidPreparedSemantics => {
            structured_insertion_error(py, MoleculeInsertionError::new_err, error)
        }
    }
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
        MolblockMoleculeBuildError::Preparation(error) => map_preparation_error(py, error),
    }
}

pub(crate) fn map_preparation_error(
    py: Python<'_>,
    error: DocumentMoleculePreparationErrorV2,
) -> PyResult<PyErr> {
    match error {
        DocumentMoleculePreparationErrorV2::Geometry(error) => Ok(
            super::geometry_binding::geometry_error(py, error.to_string()),
        ),
        DocumentMoleculePreparationErrorV2::Position(error) => projection_error(py, error),
        DocumentMoleculePreparationErrorV2::AromaticityResolutionFailed
        | DocumentMoleculePreparationErrorV2::InvalidStereoReference { .. }
        | DocumentMoleculePreparationErrorV2::InvalidStereoSemantics
        | DocumentMoleculePreparationErrorV2::UnrepresentableTetrahedral { .. }
        | DocumentMoleculePreparationErrorV2::UnrepresentableDoubleBondStereo { .. }
        | DocumentMoleculePreparationErrorV2::UnrepresentableDoubleBondDepiction { .. }
        | DocumentMoleculePreparationErrorV2::UnsupportedStereoClass { .. }
        | DocumentMoleculePreparationErrorV2::UnsupportedAtomFact { .. }
        | DocumentMoleculePreparationErrorV2::UnsupportedBondOrder { .. } => {
            structured_insertion_error(py, UnsupportedMoleculeInsertionError::new_err, error)
        }
        DocumentMoleculePreparationErrorV2::MissingCoordinates
        | DocumentMoleculePreparationErrorV2::CoordinateCountMismatch { .. }
        | DocumentMoleculePreparationErrorV2::NonFiniteCoordinate { .. }
        | DocumentMoleculePreparationErrorV2::Insertion(_) => {
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
    module.add_function(wrap_pyfunction!(prepare_smiles_molecule_v1, module)?)?;
    Ok(())
}
