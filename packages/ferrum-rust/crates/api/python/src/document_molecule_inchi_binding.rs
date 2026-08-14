//! Frozen exact-revision document-molecule InChI export.

use ferrum_api::{
    DocumentMoleculeInchiError as RustDocumentMoleculeInchiError, PreparedDocumentMoleculeInchiV1,
    export_prepared_document_molecule_inchi_v1, prepare_document_molecule_inchi_v1,
};
use ferrum_chemistry::{ChemistryError as RustChemistryError, NativeChemEngine};
use pyo3::create_exception;
use pyo3::prelude::*;

use crate::binding::FerrumError;
use crate::chemistry_binding::PyInchiModeV1;
use crate::projection_binding::PySessionDocumentObservationV1;

create_exception!(ferrum_chem, DocumentMoleculeInchiError, FerrumError);
create_exception!(
    ferrum_chem,
    UnsupportedDocumentMoleculeInchiError,
    DocumentMoleculeInchiError
);

const OPERATION: &str = "export_document_molecule_inchi_v1";

/// One immutable InChI tied to the exact source document observation.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculeInchiV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentMoleculeInchiV1 {
    #[pyo3(get)]
    inchi: String,
    #[pyo3(get)]
    source_revision: u64,
    #[pyo3(get)]
    source_digest: String,
    #[pyo3(get)]
    molecule_id: String,
    #[pyo3(get)]
    mode: PyInchiModeV1,
}

enum NativeExportFailure {
    Load(RustChemistryError),
    Export(RustDocumentMoleculeInchiError),
}

/// Export one supported durable molecule without mutating its document session.
///
/// Rust validates and freezes the complete graph before locating or loading the
/// packaged adapter. Native execution then runs while Python is detached.
#[pyfunction]
fn export_document_molecule_inchi_v1(
    py: Python<'_>,
    observation: PyRef<'_, PySessionDocumentObservationV1>,
    molecule_id: String,
    mode: PyRef<'_, PyInchiModeV1>,
) -> PyResult<PyDocumentMoleculeInchiV1> {
    let molecule_id = crate::document_error_binding::document_object_id(py, molecule_id)?;
    let selected_mode = (*mode).into_rust();
    let prepared = match prepare_document_molecule_inchi_v1(
        observation.observation(),
        &molecule_id,
        selected_mode,
    ) {
        Ok(prepared) => prepared,
        Err(error) => return Err(map_preparation_error(py, error)?),
    };
    let result_facts = result_facts(&prepared, *mode);
    let library_path = crate::chemistry_binding::packaged_library_path(py, OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine = NativeChemEngine::load(&worker_path).map_err(NativeExportFailure::Load)?;
        export_prepared_document_molecule_inchi_v1(&engine, &prepared)
            .map_err(NativeExportFailure::Export)
    });
    match result {
        Ok(inchi) => Ok(PyDocumentMoleculeInchiV1 {
            inchi,
            source_revision: result_facts.0,
            source_digest: result_facts.1,
            molecule_id: result_facts.2,
            mode: result_facts.3,
        }),
        Err(NativeExportFailure::Load(error)) => Err(crate::chemistry_binding::map_load_error(
            py,
            OPERATION,
            &library_path,
            error,
        )?),
        Err(NativeExportFailure::Export(RustDocumentMoleculeInchiError::Chemistry(error))) => {
            Err(crate::chemistry_binding::map_packaged_operation_error(
                py,
                OPERATION,
                &library_path,
                error,
            )?)
        }
        Err(NativeExportFailure::Export(error)) => Err(map_preparation_error(py, error)?),
    }
}

fn result_facts(
    prepared: &PreparedDocumentMoleculeInchiV1,
    mode: PyInchiModeV1,
) -> (u64, String, String, PyInchiModeV1) {
    (
        prepared.source_revision(),
        hex_digest(prepared.source_digest()),
        prepared.molecule_id().as_str().to_owned(),
        mode,
    )
}

fn map_preparation_error(py: Python<'_>, error: RustDocumentMoleculeInchiError) -> PyResult<PyErr> {
    match error {
        RustDocumentMoleculeInchiError::UnknownMolecule { .. }
        | RustDocumentMoleculeInchiError::UnsupportedMolecule(_) => {
            structured_error(py, UnsupportedDocumentMoleculeInchiError::new_err, error)
        }
        RustDocumentMoleculeInchiError::Document(_)
        | RustDocumentMoleculeInchiError::CoreProjection(_) => {
            structured_error(py, DocumentMoleculeInchiError::new_err, error)
        }
        RustDocumentMoleculeInchiError::Chemistry(error) => {
            crate::chemistry_binding::map_chemistry_error(py, error)
        }
    }
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
        "DocumentMoleculeInchiError",
        module.py().get_type::<DocumentMoleculeInchiError>(),
    )?;
    module.add(
        "UnsupportedDocumentMoleculeInchiError",
        module
            .py()
            .get_type::<UnsupportedDocumentMoleculeInchiError>(),
    )?;
    module.add_class::<PyDocumentMoleculeInchiV1>()?;
    module.add_function(wrap_pyfunction!(export_document_molecule_inchi_v1, module)?)?;
    Ok(())
}
