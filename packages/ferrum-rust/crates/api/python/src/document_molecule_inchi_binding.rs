//! Frozen exact-revision document-molecule InChI export.

use std::path::PathBuf;

use ferrum_api::{
    DocumentMoleculeInchiError as RustDocumentMoleculeInchiError,
    DocumentMoleculeInchiPublicationErrorV1, DocumentMoleculeInchiV1,
    export_prepared_document_molecule_inchi_receipt_v1, prepare_document_molecule_inchi_v1,
    publish_document_molecule_inchi_v1 as publish_inchi_receipt,
};
use ferrum_chemistry::{ChemistryError as RustChemistryError, NativeChemEngine};
use ferrum_document::DocumentObjectIdV1;
use ferrum_document::artifact_publication_v1::ArtifactPublicationDurabilityV1;
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyString;

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
const RESOURCE_REASON: &str = "document InChI publication could not reserve output storage";
const SELECTOR_TEXT_REASON: &str = "molecule selector must be valid UTF-8 text";

/// One immutable InChI tied to the exact source document observation.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculeInchiV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentMoleculeInchiV1 {
    receipt: DocumentMoleculeInchiV1,
}

#[pymethods]
impl PyDocumentMoleculeInchiV1 {
    #[getter]
    fn inchi(&self) -> &str {
        self.receipt.inchi()
    }

    #[getter]
    fn source_revision(&self) -> u64 {
        self.receipt.source_revision()
    }

    #[getter]
    fn source_digest(&self, py: Python<'_>) -> PyResult<String> {
        hex_digest(py, self.receipt.source_digest())
    }

    #[getter]
    fn molecule_id(&self) -> &str {
        self.receipt.molecule_id().as_str()
    }

    #[getter]
    fn mode(&self) -> PyInchiModeV1 {
        PyInchiModeV1::from_rust(self.receipt.mode())
    }
}

/// Result of safely publishing one exact InChI receipt.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculeInchiPublicationV1",
    skip_from_py_object
)]
struct PyDocumentMoleculeInchiPublicationV1 {
    #[pyo3(get)]
    directory_entry_confirmed: bool,
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
    molecule_id: &Bound<'_, PyString>,
    mode: PyRef<'_, PyInchiModeV1>,
) -> PyResult<PyDocumentMoleculeInchiV1> {
    let molecule_id = molecule_id
        .to_str()
        .map_err(|_| inchi_error(py, SELECTOR_TEXT_REASON))?;
    let molecule_id = copied(py, molecule_id)?;
    let molecule_id = DocumentObjectIdV1::parse(molecule_id)
        .map_err(|error| inchi_error(py, error.to_string()))?;
    let selected_mode = (*mode).into_rust();
    let prepared = match prepare_document_molecule_inchi_v1(
        observation.observation(),
        &molecule_id,
        selected_mode,
    ) {
        Ok(prepared) => prepared,
        Err(error) => return Err(map_preparation_error(py, error)?),
    };
    let library_path = crate::chemistry_binding::packaged_library_path(py, OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine = NativeChemEngine::load(&worker_path).map_err(NativeExportFailure::Load)?;
        export_prepared_document_molecule_inchi_receipt_v1(&engine, prepared)
            .map_err(NativeExportFailure::Export)
    });
    match result {
        Ok(receipt) => Ok(PyDocumentMoleculeInchiV1 { receipt }),
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

/// Safely publish one frozen InChI receipt to a concrete file.
#[pyfunction]
fn publish_document_molecule_inchi_v1(
    py: Python<'_>,
    receipt: PyRef<'_, PyDocumentMoleculeInchiV1>,
    destination: PathBuf,
) -> PyResult<PyDocumentMoleculeInchiPublicationV1> {
    let outcome = match publish_inchi_receipt(&receipt.receipt, destination) {
        Ok(outcome) => outcome,
        Err(DocumentMoleculeInchiPublicationErrorV1::ResourceAllocation { destination }) => {
            return Err(crate::document_error_binding::publication_resource_error(
                py,
                destination,
                RESOURCE_REASON,
            )?);
        }
        Err(DocumentMoleculeInchiPublicationErrorV1::Publication(error)) => {
            return Err(crate::document_error_binding::map_artifact_publication_error(py, error)?);
        }
    };
    Ok(PyDocumentMoleculeInchiPublicationV1 {
        directory_entry_confirmed: outcome.durability()
            == ArtifactPublicationDurabilityV1::Confirmed,
    })
}

fn map_preparation_error(py: Python<'_>, error: RustDocumentMoleculeInchiError) -> PyResult<PyErr> {
    match error {
        RustDocumentMoleculeInchiError::UnknownMolecule { .. }
        | RustDocumentMoleculeInchiError::ProjectionRootMismatch
        | RustDocumentMoleculeInchiError::UnsupportedMolecule(_) => {
            structured_error(py, UnsupportedDocumentMoleculeInchiError::new_err, error)
        }
        RustDocumentMoleculeInchiError::Document(_)
        | RustDocumentMoleculeInchiError::CoreProjection(_)
        | RustDocumentMoleculeInchiError::ResourceAllocation => {
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

fn hex_digest(py: Python<'_>, digest: &[u8; 32]) -> PyResult<String> {
    let mut value = String::new();
    value
        .try_reserve_exact(64)
        .map_err(|_| inchi_error(py, RESOURCE_REASON))?;
    for byte in digest {
        value.push(hex_digit(byte >> 4));
        value.push(hex_digit(byte & 0x0f));
    }
    Ok(value)
}

const fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + value - 10) as char,
    }
}

fn copied(py: Python<'_>, value: &str) -> PyResult<String> {
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| inchi_error(py, RESOURCE_REASON))?;
    result.push_str(value);
    Ok(result)
}

fn inchi_error(py: Python<'_>, reason: impl Into<String>) -> PyErr {
    let reason = reason.into();
    let error = DocumentMoleculeInchiError::new_err(reason.clone());
    if let Err(attribute_error) = error.value(py).setattr("reason", reason) {
        return attribute_error;
    }
    error
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
    module.add_class::<PyDocumentMoleculeInchiPublicationV1>()?;
    module.add_function(wrap_pyfunction!(export_document_molecule_inchi_v1, module)?)?;
    module.add_function(wrap_pyfunction!(
        publish_document_molecule_inchi_v1,
        module
    )?)?;
    Ok(())
}
