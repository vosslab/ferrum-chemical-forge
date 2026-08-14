//! Private worker-safe preparation and authenticated session boundary for native Cut.
//!
//! This unsupported entry point belongs only to bundled Ferrum-Qt. It has no
//! wheel stub, CLI, serde, or wire-format commitment.

use ferrum_api::{
    DocumentClipboardCutApplyErrorV1, DocumentClipboardCutPlanV1, apply_clipboard_cut_v1,
    prepare_clipboard_cut_v1,
};
use ferrum_document::DocumentSession;
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString, PyTuple};

use crate::binding::{FerrumError, PySessionOperationResultV1};
use crate::projection_binding::PySessionDocumentObservationV1;

create_exception!(ferrum_chem, DocumentClipboardCutError, FerrumError);

const DIGEST_REASON: &str = "expected digest must be exactly 64 lowercase hexadecimal characters";
const DIGEST_TEXT_REASON: &str = "expected digest must be valid UTF-8 text";
const RESOURCE_REASON: &str = "clipboard Cut could not reserve Python result storage";

/// One immutable fragment and source-authenticated deletion plan.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentClipboardCutPlanV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentClipboardCutPlanV1 {
    plan: DocumentClipboardCutPlanV1,
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    source_revision: u64,
    #[pyo3(get)]
    source_digest: String,
    #[pyo3(get)]
    selected_objects: Py<PyTuple>,
    #[pyo3(get)]
    fragment_cdml: String,
}

/// Prepare one insertion-valid fragment and exact Cut deletion plan.
#[pyfunction(name = "prepare_document_clipboard_cut_v1")]
fn prepare_document_clipboard_cut_v1_binding(
    py: Python<'_>,
    observation: PyRef<'_, PySessionDocumentObservationV1>,
    object_ids: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentClipboardCutPlanV1> {
    let selection = crate::clipboard_fragment_binding::parse_clipboard_selection(
        py,
        observation.observation(),
        object_ids,
    )
    .map_err(|error| cut_error(py, error.to_string()))?;
    let plan = prepare_clipboard_cut_v1(observation.observation(), selection)
        .map_err(|error| cut_error(py, error.to_string()))?;
    let schema = copied(py, plan.schema())?;
    let source_revision = plan.source_revision();
    let source_digest = crate::clipboard_fragment_binding::hex_digest(py, plan.source_digest())
        .map_err(|error| cut_error(py, error.to_string()))?;
    let selected_objects =
        crate::clipboard_fragment_binding::object_tuple(py, plan.selected_objects())
            .map_err(|error| cut_error(py, error.to_string()))?;
    let fragment_cdml = copied(py, plan.fragment().fragment_cdml())?;
    Ok(PyDocumentClipboardCutPlanV1 {
        plan,
        schema,
        source_revision,
        source_digest,
        selected_objects,
        fragment_cdml,
    })
}

pub(crate) fn apply_clipboard_cut_v1_binding(
    py: Python<'_>,
    session: &mut DocumentSession,
    expected_revision: u64,
    expected_digest: &Bound<'_, PyString>,
    prepared: PyRef<'_, PyDocumentClipboardCutPlanV1>,
) -> PyResult<PySessionOperationResultV1> {
    let expected_digest = match expected_digest.to_str() {
        Ok(digest) => digest,
        Err(_) => return Err(cut_error(py, DIGEST_TEXT_REASON)),
    };
    let expected_digest = parse_digest(py, expected_digest)?;
    match apply_clipboard_cut_v1(session, expected_revision, &expected_digest, &prepared.plan) {
        Ok(result) => Ok(result.into()),
        Err(DocumentClipboardCutApplyErrorV1::Session(error)) => {
            Err(cut_error(py, error.to_string()))
        }
    }
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(cut_error(py, DIGEST_REASON));
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

fn copied(py: Python<'_>, value: &str) -> PyResult<String> {
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| cut_error(py, RESOURCE_REASON))?;
    result.push_str(value);
    Ok(result)
}

fn cut_error(py: Python<'_>, reason: impl Into<String>) -> PyErr {
    let reason = reason.into();
    let error = DocumentClipboardCutError::new_err(reason.clone());
    if let Err(attribute_error) = error.value(py).setattr("reason", reason) {
        return attribute_error;
    }
    error
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentClipboardCutError",
        module.py().get_type::<DocumentClipboardCutError>(),
    )?;
    module.add_class::<PyDocumentClipboardCutPlanV1>()?;
    module.add_function(wrap_pyfunction!(
        prepare_document_clipboard_cut_v1_binding,
        module
    )?)?;
    Ok(())
}
