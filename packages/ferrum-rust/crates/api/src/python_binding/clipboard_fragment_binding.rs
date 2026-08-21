//! Private PyO3 boundary for revision-bound native clipboard Copy.
//!
//! This unsupported entry point belongs only to the bundled Ferrum route. It
//! deliberately has no wheel stub, CLI, serde, or wire-format commitment.

use ferrum_document::DocumentObjectIdV1;
use ferrum_document::{
    extract_document_clipboard_fragment_v1, DocumentClipboardFragmentKindV1,
    DocumentClipboardFragmentV1, DocumentClipboardSelectionV1,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString, PyTuple};

use super::binding::FerrumError;
use super::projection_binding::PySessionDocumentObservationV1;

create_exception!(ferrum_chem, DocumentClipboardFragmentError, FerrumError);

const RESOURCE_REASON: &str = "clipboard Copy could not reserve result storage";
const SELECTION_SHAPE_REASON: &str =
    "clipboard Copy selectors must be one nonempty exact tuple of durable object IDs";
const SELECTOR_TEXT_REASON: &str = "clipboard Copy selector must be valid UTF-8 text";

/// One immutable selected-only CDML fragment from an exact document observation.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentClipboardFragmentV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentClipboardFragmentV1 {
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    source_revision: u64,
    #[pyo3(get)]
    source_digest: String,
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    selected_objects: Py<PyTuple>,
    #[pyo3(get)]
    copied_roots: Py<PyTuple>,
    #[pyo3(get)]
    copied_atoms: Py<PyTuple>,
    #[pyo3(get)]
    copied_bonds: Py<PyTuple>,
    #[pyo3(get)]
    fragment_cdml: String,
}

/// Extract one selected-only CDML fragment without mutating the Rust session.
///
/// Experimental internal-to-Ferrum API. The tuple contains opaque durable
/// object IDs resolved from the exact observation's projected selection.
#[pyfunction(name = "extract_document_clipboard_fragment_v1")]
fn extract_document_clipboard_fragment_v1_binding(
    py: Python<'_>,
    observation: PyRef<'_, PySessionDocumentObservationV1>,
    object_ids: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentClipboardFragmentV1> {
    let selection = parse_clipboard_selection(py, observation.observation(), object_ids)?;
    let receipt = extract_document_clipboard_fragment_v1(observation.observation(), selection)
        .map_err(|error| clipboard_error(py, error.to_string()))?;
    receipt_to_python(py, &receipt)
}

pub(crate) fn parse_clipboard_selection(
    py: Python<'_>,
    observation: &ferrum_document::SessionDocumentObservationV1,
    object_ids: &Bound<'_, PyAny>,
) -> PyResult<DocumentClipboardSelectionV1> {
    if !object_ids.is_exact_instance_of::<PyTuple>() {
        return Err(clipboard_error(py, SELECTION_SHAPE_REASON));
    }
    let object_ids = object_ids.cast::<PyTuple>()?;
    let maximum =
        selectable_object_count(observation).ok_or_else(|| clipboard_error(py, RESOURCE_REASON))?;
    if object_ids.is_empty() || object_ids.len() > maximum {
        return Err(clipboard_error(py, SELECTION_SHAPE_REASON));
    }
    let mut selectors = Vec::new();
    selectors
        .try_reserve_exact(object_ids.len())
        .map_err(|_| clipboard_error(py, RESOURCE_REASON))?;
    for item in object_ids.iter() {
        if !item.is_exact_instance_of::<PyString>() {
            return Err(clipboard_error(py, SELECTION_SHAPE_REASON));
        }
        let selector = item
            .cast::<PyString>()?
            .to_str()
            .map_err(|_| clipboard_error(py, SELECTOR_TEXT_REASON))?;
        let selector = copied(py, selector)?;
        let selector = DocumentObjectIdV1::parse(selector)
            .map_err(|error| clipboard_error(py, error.to_string()))?;
        selectors.push(selector);
    }
    DocumentClipboardSelectionV1::new(selectors)
        .map_err(|error| clipboard_error(py, error.to_string()))
}

fn selectable_object_count(
    observation: &ferrum_document::SessionDocumentObservationV1,
) -> Option<usize> {
    let projection = observation.projection();
    let structure = projection
        .molecules()
        .iter()
        .try_fold(0_usize, |count, molecule| {
            count
                .checked_add(molecule.atoms().len())?
                .checked_add(molecule.bonds().len())
        })?;
    structure.checked_add(projection.presentation_stack().roots().len())
}

fn receipt_to_python(
    py: Python<'_>,
    receipt: &DocumentClipboardFragmentV1,
) -> PyResult<PyDocumentClipboardFragmentV1> {
    Ok(PyDocumentClipboardFragmentV1 {
        schema: copied(py, receipt.schema())?,
        source_revision: receipt.source_revision(),
        source_digest: hex_digest(py, receipt.source_digest())?,
        kind: copied(
            py,
            match receipt.kind() {
                DocumentClipboardFragmentKindV1::Structure => "structure",
                DocumentClipboardFragmentKindV1::TopLevel => "top_level",
            },
        )?,
        selected_objects: object_tuple(py, receipt.selected_objects())?,
        copied_roots: object_tuple(py, receipt.copied_roots())?,
        copied_atoms: object_tuple(py, receipt.copied_atoms())?,
        copied_bonds: object_tuple(py, receipt.copied_bonds())?,
        fragment_cdml: copied(py, receipt.fragment_cdml())?,
    })
}

pub(crate) fn object_tuple(py: Python<'_>, values: &[DocumentObjectIdV1]) -> PyResult<Py<PyTuple>> {
    let mut objects = Vec::new();
    objects
        .try_reserve_exact(values.len())
        .map_err(|_| clipboard_error(py, RESOURCE_REASON))?;
    for value in values {
        objects.push(copied(py, value.as_str())?);
    }
    Ok(PyTuple::new(py, objects)?.unbind())
}

fn copied(py: Python<'_>, value: &str) -> PyResult<String> {
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| clipboard_error(py, RESOURCE_REASON))?;
    result.push_str(value);
    Ok(result)
}

pub(crate) fn hex_digest(py: Python<'_>, digest: &[u8; 32]) -> PyResult<String> {
    let mut value = String::new();
    value
        .try_reserve_exact(64)
        .map_err(|_| clipboard_error(py, RESOURCE_REASON))?;
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

fn clipboard_error(py: Python<'_>, reason: impl Into<String>) -> PyErr {
    let reason = reason.into();
    let error = DocumentClipboardFragmentError::new_err(reason.clone());
    if let Err(attribute_error) = error.value(py).setattr("reason", reason) {
        return attribute_error;
    }
    error
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentClipboardFragmentError",
        module.py().get_type::<DocumentClipboardFragmentError>(),
    )?;
    module.add_class::<PyDocumentClipboardFragmentV1>()?;
    module.add_function(wrap_pyfunction!(
        extract_document_clipboard_fragment_v1_binding,
        module
    )?)?;
    Ok(())
}
