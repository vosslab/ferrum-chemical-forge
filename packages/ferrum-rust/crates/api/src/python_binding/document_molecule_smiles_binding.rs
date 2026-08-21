//! Private canonical document-SMILES operation for bundled Ferrum.
//!
//! This discoverable entry point deliberately remains absent from the wheel
//! stub, CLI, serde, and wire contracts.

use std::path::PathBuf;

use ferrum_chemistry::{ChemistryError as RustChemistryError, NativeChemEngine};
use ferrum_document::DocumentObjectIdV1;
use ferrum_document::artifact_publication_v1::ArtifactPublicationDurabilityV1;
use ferrum_document::{
    DocumentMoleculeSmilesErrorV1, DocumentMoleculeSmilesPublicationErrorV1,
    DocumentMoleculeSmilesRequestV1, DocumentMoleculeSmilesV1,
    export_prepared_document_molecule_smiles_v1, prepare_document_molecule_smiles_v1,
    publish_document_molecule_smiles_v1 as publish_smiles_receipt,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyString;

use super::binding::FerrumError;
use super::projection_binding::PySessionDocumentObservationV1;

create_exception!(ferrum_chem, DocumentMoleculeSmilesError, FerrumError);

const OPERATION: &str = "export_document_molecule_smiles_v1";
const DIGEST_REASON: &str = "expected digest must be exactly 64 lowercase hexadecimal characters";
const DIGEST_TEXT_REASON: &str = "expected digest must be valid UTF-8 text";
const RESOURCE_REASON: &str = "document SMILES publication could not reserve output storage";
const SELECTOR_TEXT_REASON: &str = "molecule selector must be valid UTF-8 text";

/// One immutable canonical SMILES tied to its exact source observation.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculeSmilesV1",
    skip_from_py_object
)]
struct PyDocumentMoleculeSmilesV1 {
    receipt: DocumentMoleculeSmilesV1,
}

#[pymethods]
impl PyDocumentMoleculeSmilesV1 {
    #[getter]
    fn schema(&self) -> &'static str {
        self.receipt.schema()
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
    fn profile(&self) -> &'static str {
        self.receipt.profile()
    }

    #[getter]
    fn smiles(&self) -> &str {
        self.receipt.smiles()
    }
}

/// Result of publishing one exact SMILES receipt.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculeSmilesPublicationV1",
    skip_from_py_object
)]
struct PyDocumentMoleculeSmilesPublicationV1 {
    #[pyo3(get)]
    directory_entry_confirmed: bool,
}

enum NativeExportFailure {
    Load(RustChemistryError),
    Export(DocumentMoleculeSmilesErrorV1),
}

/// Export one exact supported direct-root graph as canonical isomeric SMILES.
///
/// Experimental internal-to-Ferrum operation. Rust authenticates and owns
/// the complete graph before the packaged adapter path is resolved or loaded.
#[pyfunction]
fn export_document_molecule_smiles_v1(
    py: Python<'_>,
    observation: PyRef<'_, PySessionDocumentObservationV1>,
    expected_revision: u64,
    expected_digest: &Bound<'_, PyString>,
    molecule_id: &Bound<'_, PyString>,
) -> PyResult<PyDocumentMoleculeSmilesV1> {
    let expected_digest = expected_digest
        .to_str()
        .map_err(|_| smiles_error(py, DIGEST_TEXT_REASON))?;
    let expected_digest = parse_digest(py, expected_digest)?;
    let molecule_id = molecule_id
        .to_str()
        .map_err(|_| smiles_error(py, SELECTOR_TEXT_REASON))?;
    let molecule_id = copied(py, molecule_id)?;
    let molecule_id = DocumentObjectIdV1::parse(molecule_id)
        .map_err(|error| smiles_error(py, error.to_string()))?;
    let request =
        DocumentMoleculeSmilesRequestV1::new(expected_revision, expected_digest, molecule_id);
    let prepared = prepare_document_molecule_smiles_v1(observation.observation(), &request)
        .map_err(|error| smiles_error(py, error.to_string()))?;
    let library_path = super::chemistry_binding::packaged_library_path(py, OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine = NativeChemEngine::load(&worker_path).map_err(NativeExportFailure::Load)?;
        export_prepared_document_molecule_smiles_v1(&engine, prepared)
            .map_err(NativeExportFailure::Export)
    });
    let result = match result {
        Ok(result) => result,
        Err(NativeExportFailure::Load(error)) => {
            return Err(super::chemistry_binding::map_load_error(
                py,
                OPERATION,
                &library_path,
                error,
            )?);
        }
        Err(NativeExportFailure::Export(DocumentMoleculeSmilesErrorV1::Chemistry(error))) => {
            return Err(super::chemistry_binding::map_packaged_operation_error(
                py,
                OPERATION,
                &library_path,
                error,
            )?);
        }
        Err(NativeExportFailure::Export(error)) => {
            return Err(smiles_error(py, error.to_string()));
        }
    };
    Ok(PyDocumentMoleculeSmilesV1 { receipt: result })
}

/// Safely publish one frozen canonical SMILES receipt to a concrete file.
///
/// Experimental internal-to-Ferrum operation. The file contains exactly
/// one UTF-8 SMILES line and this call never mutates a document session.
#[pyfunction]
fn publish_document_molecule_smiles_v1(
    py: Python<'_>,
    receipt: PyRef<'_, PyDocumentMoleculeSmilesV1>,
    destination: PathBuf,
) -> PyResult<PyDocumentMoleculeSmilesPublicationV1> {
    let outcome = match publish_smiles_receipt(&receipt.receipt, destination) {
        Ok(outcome) => outcome,
        Err(DocumentMoleculeSmilesPublicationErrorV1::ResourceAllocation { destination }) => {
            return Err(super::document_error_binding::publication_resource_error(
                py,
                destination,
                RESOURCE_REASON,
            )?);
        }
        Err(DocumentMoleculeSmilesPublicationErrorV1::Publication(error)) => {
            return Err(super::document_error_binding::map_artifact_publication_error(py, error)?);
        }
    };
    Ok(PyDocumentMoleculeSmilesPublicationV1 {
        directory_entry_confirmed: outcome.durability()
            == ArtifactPublicationDurabilityV1::Confirmed,
    })
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(smiles_error(py, DIGEST_REASON));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        digest[index] = (hex_value(pair[0]) << 4) | hex_value(pair[1]);
    }
    Ok(digest)
}

const fn hex_value(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}

fn hex_digest(py: Python<'_>, digest: &[u8; 32]) -> PyResult<String> {
    let mut value = String::new();
    value
        .try_reserve_exact(64)
        .map_err(|_| smiles_error(py, RESOURCE_REASON))?;
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
        .map_err(|_| smiles_error(py, RESOURCE_REASON))?;
    result.push_str(value);
    Ok(result)
}

fn smiles_error(py: Python<'_>, reason: impl Into<String>) -> PyErr {
    let reason = reason.into();
    let error = DocumentMoleculeSmilesError::new_err(reason.clone());
    if let Err(attribute_error) = error.value(py).setattr("reason", reason) {
        return attribute_error;
    }
    error
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentMoleculeSmilesError",
        module.py().get_type::<DocumentMoleculeSmilesError>(),
    )?;
    module.add_class::<PyDocumentMoleculeSmilesV1>()?;
    module.add_class::<PyDocumentMoleculeSmilesPublicationV1>()?;
    module.add_function(wrap_pyfunction!(
        export_document_molecule_smiles_v1,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        publish_document_molecule_smiles_v1,
        module
    )?)?;
    Ok(())
}
