//! One frozen selected-molecule text export binding.

use std::path::PathBuf;

use ferrum_chemistry::{ChemistryError as RustChemistryError, NativeChemEngine};
use ferrum_document::{
    DocumentMoleculeExport, DocumentMoleculeExportError as RustDocumentMoleculeExportError,
    DocumentMoleculeExportFormat, DocumentMoleculeExportPublicationError,
    DocumentMoleculeExportRequest, DocumentObjectIdV1, export_prepared_document_molecule,
    prepare_document_molecule_export, publish_document_molecule_export as publish_export_receipt,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyString;

use super::binding::FerrumError;
use super::projection_binding::PySessionDocumentObservationV1;

create_exception!(ferrum_chem, DocumentMoleculeExportError, FerrumError);

const OPERATION: &str = "export_document_molecule";
const DIGEST_REASON: &str = "expected digest must be exactly 64 lowercase hexadecimal characters";
const RESOURCE_REASON: &str = "document molecule export could not reserve output storage";

/// Closed selected-root textual representation vocabulary.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DocumentMoleculeExportFormat",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyDocumentMoleculeExportFormat {
    MolfileV2000,
    MolfileV3000,
    SdfV2000,
    SdfV3000,
    CanonicalSmiles,
    InchiStandard,
    InchiFixedHydrogen,
}

impl PyDocumentMoleculeExportFormat {
    const fn into_rust(self) -> DocumentMoleculeExportFormat {
        match self {
            Self::MolfileV2000 => DocumentMoleculeExportFormat::MolfileV2000,
            Self::MolfileV3000 => DocumentMoleculeExportFormat::MolfileV3000,
            Self::SdfV2000 => DocumentMoleculeExportFormat::SdfV2000,
            Self::SdfV3000 => DocumentMoleculeExportFormat::SdfV3000,
            Self::CanonicalSmiles => DocumentMoleculeExportFormat::CanonicalSmiles,
            Self::InchiStandard => DocumentMoleculeExportFormat::InchiStandard,
            Self::InchiFixedHydrogen => DocumentMoleculeExportFormat::InchiFixedHydrogen,
        }
    }

    const fn from_rust(value: DocumentMoleculeExportFormat) -> Self {
        match value {
            DocumentMoleculeExportFormat::MolfileV2000 => Self::MolfileV2000,
            DocumentMoleculeExportFormat::MolfileV3000 => Self::MolfileV3000,
            DocumentMoleculeExportFormat::SdfV2000 => Self::SdfV2000,
            DocumentMoleculeExportFormat::SdfV3000 => Self::SdfV3000,
            DocumentMoleculeExportFormat::CanonicalSmiles => Self::CanonicalSmiles,
            DocumentMoleculeExportFormat::InchiStandard => Self::InchiStandard,
            DocumentMoleculeExportFormat::InchiFixedHydrogen => Self::InchiFixedHydrogen,
        }
    }
}

/// Immutable text receipt tied to one exact source observation.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculeExport",
    skip_from_py_object
)]
struct PyDocumentMoleculeExport {
    receipt: DocumentMoleculeExport,
}

#[pymethods]
impl PyDocumentMoleculeExport {
    #[getter]
    fn source_revision(&self) -> u64 {
        self.receipt.source_revision()
    }
    #[getter]
    fn source_digest(&self) -> String {
        hex_digest(self.receipt.source_digest())
    }
    #[getter]
    fn molecule_id(&self) -> &str {
        self.receipt.molecule_id().as_str()
    }
    #[getter]
    fn format(&self) -> PyDocumentMoleculeExportFormat {
        PyDocumentMoleculeExportFormat::from_rust(self.receipt.format())
    }
    #[getter]
    fn text(&self) -> &str {
        self.receipt.text()
    }
}

/// Publication acknowledgement for the one shared receipt route.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculeExportPublication",
    skip_from_py_object
)]
struct PyDocumentMoleculeExportPublication {
    #[pyo3(get)]
    directory_entry_confirmed: bool,
}

enum NativeExportFailure {
    Load(RustChemistryError),
    Export(RustDocumentMoleculeExportError),
}

/// Export one exact selected direct root in a closed textual representation.
#[pyfunction]
fn export_document_molecule(
    py: Python<'_>,
    observation: PyRef<'_, PySessionDocumentObservationV1>,
    expected_revision: u64,
    expected_digest: &Bound<'_, PyString>,
    molecule_id: &Bound<'_, PyString>,
    format: PyRef<'_, PyDocumentMoleculeExportFormat>,
) -> PyResult<PyDocumentMoleculeExport> {
    let digest = expected_digest
        .to_str()
        .map_err(|_| export_error(py, DIGEST_REASON))?;
    let molecule_id = molecule_id
        .to_str()
        .map_err(|_| export_error(py, "molecule selector must be valid UTF-8 text"))?;
    let request = DocumentMoleculeExportRequest::new(
        expected_revision,
        parse_digest(py, digest)?,
        DocumentObjectIdV1::parse(molecule_id)
            .map_err(|error| export_error(py, error.to_string()))?,
        format.into_rust(),
    );
    let prepared = prepare_document_molecule_export(observation.observation(), &request)
        .map_err(|error| export_error(py, error.to_string()))?;
    let library_path = super::chemistry_binding::packaged_library_path(py, OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine = NativeChemEngine::load(&worker_path).map_err(NativeExportFailure::Load)?;
        export_prepared_document_molecule(&engine, prepared).map_err(NativeExportFailure::Export)
    });
    match result {
        Ok(receipt) => Ok(PyDocumentMoleculeExport { receipt }),
        Err(NativeExportFailure::Load(error)) => Err(super::chemistry_binding::map_load_error(
            py,
            OPERATION,
            &library_path,
            error,
        )?),
        Err(NativeExportFailure::Export(RustDocumentMoleculeExportError::Chemistry(error))) => {
            Err(super::chemistry_binding::map_packaged_operation_error(
                py,
                OPERATION,
                &library_path,
                error,
            )?)
        }
        Err(NativeExportFailure::Export(error)) => Err(export_error(py, error.to_string())),
    }
}

/// Safely publish a frozen selected-root export receipt.
#[pyfunction]
fn publish_document_molecule_export(
    py: Python<'_>,
    receipt: PyRef<'_, PyDocumentMoleculeExport>,
    destination: PathBuf,
) -> PyResult<PyDocumentMoleculeExportPublication> {
    let outcome = match publish_export_receipt(&receipt.receipt, destination) {
        Ok(outcome) => outcome,
        Err(DocumentMoleculeExportPublicationError::ResourceAllocation { destination }) => {
            return Err(super::document_error_binding::publication_resource_error(
                py,
                destination,
                RESOURCE_REASON,
            )?);
        }
        Err(DocumentMoleculeExportPublicationError::Publication(error)) => {
            return Err(super::document_error_binding::map_artifact_publication_error(py, error)?);
        }
    };
    Ok(PyDocumentMoleculeExportPublication {
        directory_entry_confirmed: outcome.durability()
            == ferrum_document::artifact_publication_v1::ArtifactPublicationDurabilityV1::Confirmed,
    })
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(export_error(py, DIGEST_REASON));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        digest[index] = (hex_value(pair[0]) << 4) | hex_value(pair[1]);
    }
    Ok(digest)
}

fn hex_digest(digest: &[u8; 32]) -> String {
    let mut value = String::with_capacity(64);
    for byte in digest {
        value.push(hex_digit(byte >> 4));
        value.push(hex_digit(byte & 0x0f));
    }
    value
}
const fn hex_value(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}
const fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + value - 10) as char,
    }
}
fn export_error(py: Python<'_>, reason: impl Into<String>) -> PyErr {
    let reason = reason.into();
    let error = DocumentMoleculeExportError::new_err(reason.clone());
    if let Err(attribute_error) = error.value(py).setattr("reason", reason) {
        return attribute_error;
    }
    error
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentMoleculeExportError",
        module.py().get_type::<DocumentMoleculeExportError>(),
    )?;
    module.add_class::<PyDocumentMoleculeExportFormat>()?;
    module.add_class::<PyDocumentMoleculeExport>()?;
    module.add_class::<PyDocumentMoleculeExportPublication>()?;
    module.add_function(wrap_pyfunction!(export_document_molecule, module)?)?;
    module.add_function(wrap_pyfunction!(publish_document_molecule_export, module)?)?;
    Ok(())
}
