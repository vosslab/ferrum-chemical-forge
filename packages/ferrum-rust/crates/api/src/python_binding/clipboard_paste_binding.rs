//! Private worker-safe preparation and authenticated session boundary for native Paste.
//!
//! This unsupported entry point belongs only to bundled Ferrum. It has no
//! wheel stub, CLI, serde, or wire-format commitment.

use ferrum_document::{
    apply_clipboard_paste_v1, prepare_clipboard_paste_v1, DocumentClipboardPasteApplyErrorV1,
    DocumentClipboardPastePlanV1, DOCUMENT_CLIPBOARD_PASTE_PROFILE_V1,
};
use ferrum_document::{DocumentSession, TopLevelRootKindV1};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString, PyTuple};

use super::binding::{FerrumError, PySessionOperationResultV1};

create_exception!(ferrum_chem, DocumentClipboardPasteError, FerrumError);

const DIGEST_REASON: &str = "expected digest must be exactly 64 lowercase hexadecimal characters";
const DIGEST_TEXT_REASON: &str = "expected digest must be valid UTF-8 text";
const RESOURCE_REASON: &str = "clipboard Paste could not reserve Python result storage";
const SOURCE_REASON: &str = "clipboard Paste source must be an exact built-in string";
const SOURCE_TEXT_REASON: &str = "clipboard Paste source must be valid UTF-8 text";

/// One immutable handle-free plan safe to deliver from a Qt worker.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentClipboardPastePlanV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentClipboardPastePlanV1 {
    plan: DocumentClipboardPastePlanV1,
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    profile: String,
    #[pyo3(get)]
    root_count: usize,
    #[pyo3(get)]
    declared_id_count: usize,
}

/// One committed observation plus inserted-root selection facts.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentClipboardPasteResultV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentClipboardPasteResultV1 {
    #[pyo3(get)]
    operation: PySessionOperationResultV1,
    #[pyo3(get)]
    pasted_roots: Py<PyTuple>,
}

/// Admit one exact clipboard string under the named product profile.
#[pyfunction(name = "prepare_clipboard_paste_v1")]
fn prepare_clipboard_paste_v1_binding(
    py: Python<'_>,
    source: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentClipboardPastePlanV1> {
    if !source.is_exact_instance_of::<PyString>() {
        return Err(paste_error(py, SOURCE_REASON)?);
    }
    let source = source.cast::<PyString>()?;
    let source = match source.to_str() {
        Ok(source) => source,
        Err(_) => return Err(paste_error(py, SOURCE_TEXT_REASON)?),
    };
    let source = copied(py, source)?;
    let plan = match py.detach(move || prepare_clipboard_paste_v1(&source)) {
        Ok(plan) => plan,
        Err(error) => return Err(paste_error(py, error.to_string())?),
    };
    let schema = copied(py, plan.schema())?;
    let profile = copied(py, DOCUMENT_CLIPBOARD_PASTE_PROFILE_V1)?;
    let root_count = plan.roots().len();
    let declared_id_count = plan.declared_id_count();
    Ok(PyDocumentClipboardPastePlanV1 {
        plan,
        schema,
        profile,
        root_count,
        declared_id_count,
    })
}

pub(crate) fn apply_clipboard_paste_v1_binding(
    py: Python<'_>,
    session: &mut DocumentSession,
    expected_revision: u64,
    expected_digest: &Bound<'_, PyString>,
    prepared: PyRef<'_, PyDocumentClipboardPastePlanV1>,
) -> PyResult<PyDocumentClipboardPasteResultV1> {
    let expected_digest = match expected_digest.to_str() {
        Ok(digest) => digest,
        Err(_) => return Err(paste_error(py, DIGEST_TEXT_REASON)?),
    };
    let expected_digest = parse_digest(py, expected_digest)?;
    let result = match apply_clipboard_paste_v1(
        session,
        expected_revision,
        &expected_digest,
        &prepared.plan,
    ) {
        Ok(result) => result,
        Err(DocumentClipboardPasteApplyErrorV1::Session(error)) => {
            return Err(paste_error(py, error.to_string())?);
        }
    };
    let roots = result
        .pasted_roots()
        .iter()
        .map(|root| {
            let kind = copied(py, root_kind(root.kind()))?;
            let source_id = copied(py, root.source_id().as_str())?;
            PyTuple::new(py, [kind, source_id]).map(Bound::unbind)
        })
        .collect::<PyResult<Vec<_>>>()?;
    let pasted_roots = PyTuple::new(py, roots)?.unbind();
    Ok(PyDocumentClipboardPasteResultV1 {
        operation: result.into_operation_result().into(),
        pasted_roots,
    })
}

const fn root_kind(kind: TopLevelRootKindV1) -> &'static str {
    match kind {
        TopLevelRootKindV1::Molecule => "molecule",
        TopLevelRootKindV1::Arrow => "arrow",
        TopLevelRootKindV1::Plus => "plus",
        TopLevelRootKindV1::Text => "text",
        TopLevelRootKindV1::Rectangle => "rectangle",
        TopLevelRootKindV1::Square => "square",
        TopLevelRootKindV1::Oval => "oval",
        TopLevelRootKindV1::Circle => "circle",
        TopLevelRootKindV1::Polygon => "polygon",
        TopLevelRootKindV1::Polyline => "polyline",
    }
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(paste_error(py, DIGEST_REASON)?);
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
    if result.try_reserve_exact(value.len()).is_err() {
        return Err(paste_error(py, RESOURCE_REASON)?);
    }
    result.push_str(value);
    Ok(result)
}

fn paste_error(py: Python<'_>, reason: impl AsRef<str>) -> PyResult<PyErr> {
    let reason = reason.as_ref();
    let message = owned_reason(reason)?;
    let attribute = owned_reason(reason)?;
    let error = DocumentClipboardPasteError::new_err(message);
    error.value(py).setattr("reason", attribute)?;
    Ok(error)
}

fn owned_reason(reason: &str) -> PyResult<String> {
    let mut owned = String::new();
    owned
        .try_reserve_exact(reason.len())
        .map_err(|_| DocumentClipboardPasteError::new_err(RESOURCE_REASON))?;
    owned.push_str(reason);
    Ok(owned)
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentClipboardPasteError",
        module.py().get_type::<DocumentClipboardPasteError>(),
    )?;
    module.add_class::<PyDocumentClipboardPastePlanV1>()?;
    module.add_class::<PyDocumentClipboardPasteResultV1>()?;
    module.add_function(wrap_pyfunction!(
        prepare_clipboard_paste_v1_binding,
        module
    )?)?;
    Ok(())
}
