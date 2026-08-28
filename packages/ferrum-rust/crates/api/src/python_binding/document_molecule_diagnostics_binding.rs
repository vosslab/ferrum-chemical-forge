//! Private PyO3 transport for fenced direct-root structure diagnostics.
//!
//! The frozen protocol route owns every chemical decision. This adapter copies
//! snapshot facts and durable root identifiers, then lowers completed facts
//! into immutable Python values for Ferrum's Qt UI.

use ferrum_document::DocumentObjectIdV1;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString, PyTuple};

use crate::{
    DocumentMoleculeDiagnosticRecordSummaryV1, DocumentMoleculeDiagnosticsRequestV1,
    DocumentMoleculeDiagnosticsSnapshotV1, DocumentMoleculeDiagnosticsSummaryV1,
    DocumentMoleculeReportFindingLocationSummaryV1, DocumentMoleculeReportFindingRecoverySummaryV1,
    DocumentMoleculeReportFindingSeveritySummaryV1, DocumentMoleculeReportFindingSubjectSummaryV1,
    DocumentMoleculeReportFindingSummaryV1, OperationProtocolEnvelopeV1,
    OperationProtocolOperationV1, OperationProtocolOutcomeV1, OperationProtocolRequestV1,
    ProtocolRequestSchemaV1, execute_admitted_operation_v1,
};

use super::document_error_binding::{document_object_id, operation_validation_error};

const REQUEST_ID: &str = "python-document-molecule-diagnostics-v1";
const DIGEST_REASON: &str = "expected digest must be exactly 64 lowercase hexadecimal characters";
const DIGEST_TEXT_REASON: &str = "expected digest must be valid UTF-8 text";
const CDML_TEXT_REASON: &str = "snapshot CDML must be valid UTF-8 text";
const SELECTOR_SHAPE_REASON: &str =
    "molecule diagnostics selectors must be an exact built-in tuple";
const SELECTOR_TEXT_REASON: &str = "molecule diagnostics selectors must be valid UTF-8 text";
const RESOURCE_REASON: &str = "molecule diagnostics could not reserve owned Python result storage";

/// Immutable authenticated location facts for one diagnostic finding.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "_DocumentMoleculeDiagnosticLocationV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentMoleculeDiagnosticLocationV1 {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    identifier: Option<String>,
    #[pyo3(get)]
    subject: Option<String>,
}

/// Immutable closed-vocabulary diagnostic finding for one direct molecule root.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "_DocumentMoleculeDiagnosticFindingV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentMoleculeDiagnosticFindingV1 {
    #[pyo3(get)]
    severity: String,
    #[pyo3(get)]
    code: String,
    #[pyo3(get)]
    recovery: String,
    #[pyo3(get)]
    location: Py<PyDocumentMoleculeDiagnosticLocationV1>,
    /// Rust-supplied optional display detail. Qt never derives chemical detail.
    #[pyo3(get)]
    detail: Option<String>,
}

/// Immutable selected direct-root diagnostics record.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "_DocumentMoleculeDiagnosticRecordV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentMoleculeDiagnosticRecordV1 {
    #[pyo3(get)]
    molecule_id: String,
    #[pyo3(get)]
    document_paint_order: u32,
    #[pyo3(get)]
    findings: Py<PyTuple>,
}

/// Immutable diagnostics receipt tied to the exact initiating live fence.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "_DocumentMoleculeDiagnosticsV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentMoleculeDiagnosticsV1 {
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    source_revision: u64,
    #[pyo3(get)]
    source_digest: String,
    #[pyo3(get)]
    records: Py<PyTuple>,
}

/// Evaluate selected durable direct roots from immutable snapshot facts.
///
/// The function retains no session, callback, or borrowed Python state after
/// return. A Qt worker may therefore call it from a detached thread with one
/// copied CDML/revision/digest fence and an exact tuple of copied root IDs.
#[pyfunction]
fn _document_molecule_diagnostics_from_snapshot_v1(
    py: Python<'_>,
    cdml: &Bound<'_, PyString>,
    source_revision: u64,
    source_digest: &Bound<'_, PyString>,
    molecule_ids: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentMoleculeDiagnosticsV1> {
    let cdml = cdml
        .to_str()
        .map_err(|_| operation_validation_error(py, CDML_TEXT_REASON.to_owned()))?;
    let source_digest = source_digest
        .to_str()
        .map_err(|_| operation_validation_error(py, DIGEST_TEXT_REASON.to_owned()))?;
    let source_digest = parse_digest(py, source_digest)?;
    let molecule_ids = parse_molecule_ids(py, molecule_ids)?;
    let request = OperationProtocolRequestV1 {
        schema: ProtocolRequestSchemaV1::V1,
        request_id: REQUEST_ID.to_owned(),
        operation: OperationProtocolOperationV1::DocumentMoleculeDiagnostics(
            DocumentMoleculeDiagnosticsRequestV1 {
                snapshot: DocumentMoleculeDiagnosticsSnapshotV1 {
                    cdml: copied(py, cdml)?,
                    revision: source_revision,
                    digest_hex: hex_digest(&source_digest),
                },
                molecule_ids,
            },
        ),
    };
    let diagnostics = completed_diagnostics(py, execute_admitted_operation_v1(request))?;
    diagnostics_to_python(py, diagnostics)
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(operation_validation_error(py, DIGEST_REASON.to_owned()));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
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

fn parse_molecule_ids(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Vec<String>> {
    if !value.is_exact_instance_of::<PyTuple>() {
        return Err(operation_validation_error(
            py,
            SELECTOR_SHAPE_REASON.to_owned(),
        ));
    }
    let values = value.cast::<PyTuple>()?;
    let mut parsed = Vec::new();
    parsed
        .try_reserve_exact(values.len())
        .map_err(|_| operation_validation_error(py, RESOURCE_REASON.to_owned()))?;
    for value in values.iter() {
        if !value.is_exact_instance_of::<PyString>() {
            return Err(operation_validation_error(
                py,
                SELECTOR_SHAPE_REASON.to_owned(),
            ));
        }
        let value = value
            .cast::<PyString>()?
            .to_str()
            .map_err(|_| operation_validation_error(py, SELECTOR_TEXT_REASON.to_owned()))?;
        let value = copied(py, value)?;
        let object_id: DocumentObjectIdV1 = document_object_id(py, value)?;
        parsed.push(object_id.as_str().to_owned());
    }
    Ok(parsed)
}

fn completed_diagnostics(
    py: Python<'_>,
    envelope: OperationProtocolEnvelopeV1,
) -> PyResult<DocumentMoleculeDiagnosticsSummaryV1> {
    match envelope {
        OperationProtocolEnvelopeV1::Success(response) => match response.outcome {
            OperationProtocolOutcomeV1::DocumentMoleculeDiagnostics { diagnostics } => {
                Ok(diagnostics)
            }
            _ => Err(operation_validation_error(
                py,
                "molecule diagnostics executor returned an unexpected completed operation"
                    .to_owned(),
            )),
        },
        OperationProtocolEnvelopeV1::Error(response) => {
            Err(operation_validation_error(py, response.error.message))
        }
    }
}

fn diagnostics_to_python(
    py: Python<'_>,
    diagnostics: DocumentMoleculeDiagnosticsSummaryV1,
) -> PyResult<PyDocumentMoleculeDiagnosticsV1> {
    let mut records = Vec::new();
    records
        .try_reserve_exact(diagnostics.records.len())
        .map_err(|_| operation_validation_error(py, RESOURCE_REASON.to_owned()))?;
    for record in diagnostics.records {
        records.push(Py::new(py, record_to_python(py, record)?)?);
    }
    Ok(PyDocumentMoleculeDiagnosticsV1 {
        schema: diagnostics.schema,
        source_revision: diagnostics.source_revision,
        source_digest: diagnostics.source_digest_hex,
        records: PyTuple::new(py, records)?.unbind(),
    })
}

fn record_to_python(
    py: Python<'_>,
    record: DocumentMoleculeDiagnosticRecordSummaryV1,
) -> PyResult<PyDocumentMoleculeDiagnosticRecordV1> {
    let mut findings = Vec::new();
    findings
        .try_reserve_exact(record.findings.len())
        .map_err(|_| operation_validation_error(py, RESOURCE_REASON.to_owned()))?;
    for finding in record.findings {
        findings.push(Py::new(py, finding_to_python(py, finding)?)?);
    }
    Ok(PyDocumentMoleculeDiagnosticRecordV1 {
        molecule_id: record.molecule_id,
        document_paint_order: record.document_paint_order,
        findings: PyTuple::new(py, findings)?.unbind(),
    })
}

fn finding_to_python(
    py: Python<'_>,
    finding: DocumentMoleculeReportFindingSummaryV1,
) -> PyResult<PyDocumentMoleculeDiagnosticFindingV1> {
    let location = Py::new(py, location_to_python(finding.location)?)?;
    Ok(PyDocumentMoleculeDiagnosticFindingV1 {
        severity: severity(finding.severity).to_owned(),
        code: finding.code.as_str().to_owned(),
        recovery: recovery(finding.recovery).to_owned(),
        location,
        detail: finding.detail,
    })
}

fn location_to_python(
    location: DocumentMoleculeReportFindingLocationSummaryV1,
) -> PyResult<PyDocumentMoleculeDiagnosticLocationV1> {
    let result = match location {
        DocumentMoleculeReportFindingLocationSummaryV1::Root => {
            PyDocumentMoleculeDiagnosticLocationV1 {
                kind: "root".to_owned(),
                identifier: None,
                subject: None,
            }
        }
        DocumentMoleculeReportFindingLocationSummaryV1::Atom { identifier } => {
            PyDocumentMoleculeDiagnosticLocationV1 {
                kind: "atom".to_owned(),
                identifier: Some(identifier),
                subject: None,
            }
        }
        DocumentMoleculeReportFindingLocationSummaryV1::Vertex { identifier } => {
            PyDocumentMoleculeDiagnosticLocationV1 {
                kind: "vertex".to_owned(),
                identifier: Some(identifier),
                subject: None,
            }
        }
        DocumentMoleculeReportFindingLocationSummaryV1::Bond { identifier } => {
            PyDocumentMoleculeDiagnosticLocationV1 {
                kind: "bond".to_owned(),
                identifier: Some(identifier),
                subject: None,
            }
        }
        DocumentMoleculeReportFindingLocationSummaryV1::Unaddressable { subject } => {
            PyDocumentMoleculeDiagnosticLocationV1 {
                kind: "unaddressable".to_owned(),
                identifier: None,
                subject: Some(location_subject(subject).to_owned()),
            }
        }
    };
    Ok(result)
}

const fn severity(value: DocumentMoleculeReportFindingSeveritySummaryV1) -> &'static str {
    match value {
        DocumentMoleculeReportFindingSeveritySummaryV1::Info => "info",
        DocumentMoleculeReportFindingSeveritySummaryV1::Warning => "warning",
        DocumentMoleculeReportFindingSeveritySummaryV1::Error => "error",
    }
}

const fn recovery(value: DocumentMoleculeReportFindingRecoverySummaryV1) -> &'static str {
    match value {
        DocumentMoleculeReportFindingRecoverySummaryV1::None => "none",
        DocumentMoleculeReportFindingRecoverySummaryV1::InspectStructure => "inspect_structure",
        DocumentMoleculeReportFindingRecoverySummaryV1::CorrectChemicalFacts => {
            "correct_chemical_facts"
        }
        DocumentMoleculeReportFindingRecoverySummaryV1::ChooseSupportedRepresentation => {
            "choose_supported_representation"
        }
        DocumentMoleculeReportFindingRecoverySummaryV1::MaterializeCompactGroup => {
            "materialize_compact_group"
        }
        DocumentMoleculeReportFindingRecoverySummaryV1::ReduceSelection => "reduce_selection",
        DocumentMoleculeReportFindingRecoverySummaryV1::RetryWithChemistryRuntime => {
            "retry_with_chemistry_runtime"
        }
    }
}

const fn location_subject(value: DocumentMoleculeReportFindingSubjectSummaryV1) -> &'static str {
    match value {
        DocumentMoleculeReportFindingSubjectSummaryV1::Atom => "atom",
        DocumentMoleculeReportFindingSubjectSummaryV1::Vertex => "vertex",
        DocumentMoleculeReportFindingSubjectSummaryV1::Bond => "bond",
    }
}

fn copied(py: Python<'_>, value: &str) -> PyResult<String> {
    let mut copied = String::new();
    copied
        .try_reserve_exact(value.len())
        .map_err(|_| operation_validation_error(py, RESOURCE_REASON.to_owned()))?;
    copied.push_str(value);
    Ok(copied)
}

fn hex_digest(value: &[u8; 32]) -> String {
    let mut result = String::with_capacity(64);
    for byte in value {
        result.push(hex_digit(byte >> 4));
        result.push(hex_digit(byte & 0x0f));
    }
    result
}

const fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + value - 10) as char,
    }
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyDocumentMoleculeDiagnosticLocationV1>()?;
    module.add_class::<PyDocumentMoleculeDiagnosticFindingV1>()?;
    module.add_class::<PyDocumentMoleculeDiagnosticRecordV1>()?;
    module.add_class::<PyDocumentMoleculeDiagnosticsV1>()?;
    module.add_function(wrap_pyfunction!(
        _document_molecule_diagnostics_from_snapshot_v1,
        module
    )?)?;
    Ok(())
}
