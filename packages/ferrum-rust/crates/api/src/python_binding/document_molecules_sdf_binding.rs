//! Exact-revision multi-molecule SDF export for Ferrum.

use std::path::PathBuf;

use ferrum_chemistry::{ChemistryError as RustChemistryError, NativeChemEngine};
use ferrum_document::artifact_publication_v1::ArtifactPublicationDurabilityV1;
use ferrum_document::{
    DocumentMoleculesSdfErrorV2, DocumentMoleculesSdfRequestV2, DocumentMoleculesSdfV2,
    DocumentMoleculesSdfPublicationErrorV2, export_prepared_document_molecules_sdf_v2,
    prepare_document_molecules_sdf_v2, publish_document_molecules_sdf_v2 as publish_sdf_receipt,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyInt, PyString, PyTuple};

use super::binding::FerrumError;
use super::chemistry_binding::PyMolblockVersionV1;
use super::projection_binding::PySessionDocumentObservationV1;

create_exception!(ferrum_chem, DocumentMoleculesSdfError, FerrumError);

const OPERATION: &str = "export_document_molecules_sdf_v2";
const DIGEST_REASON: &str = "expected digest must be exactly 64 lowercase hexadecimal characters";
const DIGEST_TEXT_REASON: &str = "expected digest must be valid UTF-8 text";
const SELECTOR_TEXT_REASON: &str = "molecule selectors must be valid UTF-8 text";
const SELECTOR_SHAPE_REASON: &str = "molecule selectors must be an exact nonempty tuple of strings";
const RESOURCE_REASON: &str =
    "document molecules SDF export could not reserve input or result storage";

/// Immutable SDF receipt for one exact Rust-canonical source-ordered selection.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculesSdfV2",
    skip_from_py_object
)]
struct PyDocumentMoleculesSdfV2 {
    receipt: DocumentMoleculesSdfV2,
}

/// Result of safely publishing one exact multi-record SDF receipt.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculesSdfPublicationV2",
    skip_from_py_object
)]
struct PyDocumentMoleculesSdfPublicationV2 {
    #[pyo3(get)]
    directory_entry_confirmed: bool,
}

#[pymethods]
impl PyDocumentMoleculesSdfV2 {
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
    fn molecule_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        PyTuple::new(
            py,
            self.receipt
                .records()
                .iter()
                .map(|record| record.molecule_id().as_str()),
        )
        .map(Bound::unbind)
    }

    #[getter]
    fn record_count(&self) -> usize {
        self.receipt.record_count()
    }

    #[getter]
    fn profile(&self) -> &'static str {
        self.receipt.profile()
    }

    #[getter]
    fn version(&self) -> PyMolblockVersionV1 {
        PyMolblockVersionV1::from_rust(self.receipt.version())
    }

    #[getter]
    fn sdf(&self) -> &str {
        self.receipt.sdf()
    }
}

enum NativeExportFailure {
    Load(RustChemistryError),
    Export(DocumentMoleculesSdfErrorV2),
}

/// Export exact selected document molecules as one immutable SDF receipt.
///
/// Rust authenticates the observation and selector tuple before loading the
/// packaged chemistry engine. The receipt contains no native handle and does
/// not write files.
#[pyfunction]
fn export_document_molecules_sdf_v2(
    py: Python<'_>,
    observation: &Bound<'_, PyAny>,
    expected_revision: &Bound<'_, PyAny>,
    expected_digest: &Bound<'_, PyAny>,
    molecule_ids: &Bound<'_, PyAny>,
    version: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentMoleculesSdfV2> {
    let observation = document_observation(py, observation)?;
    let expected_revision = exact_revision(py, expected_revision)?;
    let expected_digest = exact_string(py, expected_digest, DIGEST_TEXT_REASON)?;
    let expected_digest = expected_digest
        .to_str()
        .map_err(|_| sdf_error(py, DIGEST_TEXT_REASON))?;
    let expected_digest = parse_digest(py, expected_digest)?;
    let molecule_ids = document_object_ids(py, molecule_ids)?;
    let version = molblock_version(py, version)?;
    let request = DocumentMoleculesSdfRequestV2::new(
        expected_revision,
        expected_digest,
        molecule_ids,
        version.into_rust(),
    )
    .map_err(|error| sdf_error(py, error.to_string()))?;
    let prepared = prepare_document_molecules_sdf_v2(observation.observation(), &request)
        .map_err(|error| sdf_error(py, error.to_string()))?;
    let library_path = super::chemistry_binding::packaged_library_path(py, OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine = NativeChemEngine::load(&worker_path).map_err(NativeExportFailure::Load)?;
        export_prepared_document_molecules_sdf_v2(&engine, prepared)
            .map_err(NativeExportFailure::Export)
    });
    let receipt = match result {
        Ok(receipt) => receipt,
        Err(NativeExportFailure::Load(error)) => {
            return Err(super::chemistry_binding::map_load_error(
                py,
                OPERATION,
                &library_path,
                error,
            )?);
        }
        Err(NativeExportFailure::Export(DocumentMoleculesSdfErrorV2::Chemistry(error))) => {
            return Err(super::chemistry_binding::map_packaged_operation_error(
                py,
                OPERATION,
                &library_path,
                error,
            )?);
        }
        Err(NativeExportFailure::Export(error)) => return Err(sdf_error(py, error.to_string())),
    };
    Ok(PyDocumentMoleculesSdfV2 { receipt })
}

/// Safely publish one frozen multi-record SDF receipt to a concrete file.
#[pyfunction]
fn publish_document_molecules_sdf_v2(
    py: Python<'_>,
    receipt: PyRef<'_, PyDocumentMoleculesSdfV2>,
    destination: PathBuf,
) -> PyResult<PyDocumentMoleculesSdfPublicationV2> {
    let outcome = match publish_sdf_receipt(&receipt.receipt, destination) {
        Ok(outcome) => outcome,
        Err(DocumentMoleculesSdfPublicationErrorV2::ResourceAllocation { destination }) => {
            return Err(super::document_error_binding::publication_resource_error(
                py,
                destination,
                RESOURCE_REASON,
            )?);
        }
        Err(DocumentMoleculesSdfPublicationErrorV2::Publication(error)) => {
            return Err(super::document_error_binding::map_artifact_publication_error(py, error)?);
        }
    };
    Ok(PyDocumentMoleculesSdfPublicationV2 {
        directory_entry_confirmed: outcome.durability()
            == ArtifactPublicationDurabilityV1::Confirmed,
    })
}

fn document_observation<'py>(
    py: Python<'py>,
    value: &Bound<'py, PyAny>,
) -> PyResult<PyRef<'py, PySessionDocumentObservationV1>> {
    if !value.is_exact_instance_of::<PySessionDocumentObservationV1>() {
        return Err(sdf_error(
            py,
            "observation must be an exact Ferrum document observation",
        ));
    }
    value
        .extract::<PyRef<'py, PySessionDocumentObservationV1>>()
        .map_err(|_| {
            sdf_error(
                py,
                "observation must be an exact Ferrum document observation",
            )
        })
}

fn exact_revision(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<u64> {
    if !value.is_exact_instance_of::<PyInt>() || value.is_instance_of::<pyo3::types::PyBool>() {
        return Err(sdf_error(
            py,
            "expected revision must be an exact nonnegative integer",
        ));
    }
    value
        .extract::<u64>()
        .map_err(|_| sdf_error(py, "expected revision is outside the supported range"))
}

fn exact_string<'py>(
    py: Python<'py>,
    value: &Bound<'py, PyAny>,
    reason: &str,
) -> PyResult<Bound<'py, PyString>> {
    if !value.is_exact_instance_of::<PyString>() {
        return Err(sdf_error(py, reason));
    }
    value
        .cast::<PyString>()
        .map(Clone::clone)
        .map_err(|_| sdf_error(py, reason))
}

fn molblock_version<'py>(
    py: Python<'py>,
    value: &Bound<'py, PyAny>,
) -> PyResult<PyRef<'py, PyMolblockVersionV1>> {
    if !value.is_exact_instance_of::<PyMolblockVersionV1>() {
        return Err(sdf_error(
            py,
            "version must be an exact Ferrum molblock version",
        ));
    }
    value
        .extract::<PyRef<'py, PyMolblockVersionV1>>()
        .map_err(|_| sdf_error(py, "version must be an exact Ferrum molblock version"))
}

fn document_object_ids(
    py: Python<'_>,
    values: &Bound<'_, PyAny>,
) -> PyResult<Vec<ferrum_document::DocumentObjectIdV1>> {
    if !values.is_exact_instance_of::<PyTuple>() {
        return Err(sdf_error(py, SELECTOR_SHAPE_REASON));
    }
    let values = values.cast::<PyTuple>()?;
    if values.is_empty() {
        return Err(sdf_error(py, SELECTOR_SHAPE_REASON));
    }
    let mut selectors = Vec::new();
    selectors
        .try_reserve_exact(values.len())
        .map_err(|_| sdf_error(py, RESOURCE_REASON))?;
    for value in values.iter() {
        if !value.is_exact_instance_of::<PyString>() {
            return Err(sdf_error(py, SELECTOR_SHAPE_REASON));
        }
        let value = value
            .cast::<PyString>()?
            .to_str()
            .map_err(|_| sdf_error(py, SELECTOR_TEXT_REASON))?;
        let value = copied(py, value)?;
        selectors.push(
            super::document_error_binding::document_object_id(py, value)
                .map_err(|error| sdf_error(py, error.to_string()))?,
        );
    }
    Ok(selectors)
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(sdf_error(py, DIGEST_REASON));
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
        .map_err(|_| sdf_error(py, RESOURCE_REASON))?;
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
        .map_err(|_| sdf_error(py, RESOURCE_REASON))?;
    result.push_str(value);
    Ok(result)
}

fn sdf_error(py: Python<'_>, reason: impl Into<String>) -> PyErr {
    let reason = reason.into();
    let error = DocumentMoleculesSdfError::new_err(reason.clone());
    if let Err(attribute_error) = error.value(py).setattr("reason", reason) {
        return attribute_error;
    }
    error
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentMoleculesSdfError",
        module.py().get_type::<DocumentMoleculesSdfError>(),
    )?;
    module.add_class::<PyDocumentMoleculesSdfV2>()?;
    module.add_class::<PyDocumentMoleculesSdfPublicationV2>()?;
    module.add_function(wrap_pyfunction!(export_document_molecules_sdf_v2, module)?)?;
    module.add_function(wrap_pyfunction!(publish_document_molecules_sdf_v2, module)?)?;
    Ok(())
}
