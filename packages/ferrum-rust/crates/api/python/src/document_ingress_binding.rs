//! Explicit Python admission of untrusted CDML and CD-SVG bytes or local files.

use std::path::PathBuf;

use ferrum_api::{
    CdmlIngressBudgetV1, CdmlIngressErrorV1, CdsvgIngressBudgetV1, DocumentIngressErrorV1,
    DocumentIngressFormatV1, DocumentIngressOriginV1, SourcePolicyErrorV1,
    load_document_file_with_budget, load_document_utf8_bytes_with_budget,
};
use ferrum_document::{
    CdsvgExtractionError, TypedDocumentError, XmlBudgetError, XmlInputBudgetV1, XmlInputError,
};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes, PyInt, PyString};

use crate::binding::PyDocumentSession;
use crate::document_error_binding::{DocumentInputError, DocumentLoadError};

/// One complete caller-owned XML resource budget.
///
/// The extension selects no defaults. Every field is copied into Rust before a source is
/// admitted, and the resulting value has no mutable Python alias.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "XmlInputBudgetV1",
    skip_from_py_object
)]
#[derive(Clone, Copy)]
pub(crate) struct PyXmlInputBudgetV1 {
    budget: XmlInputBudgetV1,
    #[pyo3(get)]
    max_utf8_bytes: usize,
    #[pyo3(get)]
    max_elements: usize,
    #[pyo3(get)]
    max_depth: usize,
    #[pyo3(get)]
    max_attributes: usize,
    #[pyo3(get)]
    max_text_bytes: usize,
}

#[pymethods]
impl PyXmlInputBudgetV1 {
    #[new]
    fn new(
        max_utf8_bytes: &Bound<'_, PyAny>,
        max_elements: &Bound<'_, PyAny>,
        max_depth: &Bound<'_, PyAny>,
        max_attributes: &Bound<'_, PyAny>,
        max_text_bytes: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let max_utf8_bytes = exact_usize(max_utf8_bytes, "max_utf8_bytes")?;
        let max_elements = exact_usize(max_elements, "max_elements")?;
        let max_depth = exact_usize(max_depth, "max_depth")?;
        let max_attributes = exact_usize(max_attributes, "max_attributes")?;
        let max_text_bytes = exact_usize(max_text_bytes, "max_text_bytes")?;
        Ok(Self {
            budget: XmlInputBudgetV1 {
                max_utf8_bytes,
                max_elements,
                max_depth,
                max_attributes,
                max_text_bytes,
            },
            max_utf8_bytes,
            max_elements,
            max_depth,
            max_attributes,
            max_text_bytes,
        })
    }
}

pub(crate) fn load_utf8_bytes_with_budget(
    py: Python<'_>,
    source: &Bound<'_, PyAny>,
    budget: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentSession> {
    let source = exact_bytes(source)?;
    let budget = exact_budget(budget)?;
    // This intentionally borrows PyBytes through the synchronous Rust admission call. Rust
    // never retains this slice; a successful session owns its document independently.
    load_bytes(
        py,
        source.as_bytes(),
        DocumentIngressFormatV1::Cdml(CdmlIngressBudgetV1 { xml: budget }),
    )
}

pub(crate) fn load_file_with_budget(
    py: Python<'_>,
    path: &Bound<'_, PyAny>,
    budget: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentSession> {
    let path = exact_path(path)?;
    let budget = exact_budget(budget)?;
    // File I/O and Rust parsing are synchronous. This path owns no Python input after the
    // exact string check, so it releases the GIL while the local-file policy is enforced.
    let result = py.detach(move || {
        load_document_file_with_budget(
            &path,
            DocumentIngressFormatV1::Cdml(CdmlIngressBudgetV1 { xml: budget }),
        )
    });
    ingress_result(py, result)
}

pub(crate) fn load_cdsvg_utf8_bytes_with_budget(
    py: Python<'_>,
    source: &Bound<'_, PyAny>,
    wrapper_budget: &Bound<'_, PyAny>,
    payload_budget: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentSession> {
    let source = exact_bytes(source)?;
    let wrapper = exact_budget(wrapper_budget)?;
    let payload = exact_budget(payload_budget)?;
    // See `load_utf8_bytes_with_budget`: PyBytes is borrowed only until this call returns.
    load_bytes(
        py,
        source.as_bytes(),
        DocumentIngressFormatV1::Cdsvg(CdsvgIngressBudgetV1 { wrapper, payload }),
    )
}

pub(crate) fn load_cdsvg_file_with_budget(
    py: Python<'_>,
    path: &Bound<'_, PyAny>,
    wrapper_budget: &Bound<'_, PyAny>,
    payload_budget: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentSession> {
    let path = exact_path(path)?;
    let wrapper = exact_budget(wrapper_budget)?;
    let payload = exact_budget(payload_budget)?;
    // The owned path and copied budgets let this synchronous local-file route release the GIL.
    let result = py.detach(move || {
        load_document_file_with_budget(
            &path,
            DocumentIngressFormatV1::Cdsvg(CdsvgIngressBudgetV1 { wrapper, payload }),
        )
    });
    ingress_result(py, result)
}

fn load_bytes(
    py: Python<'_>,
    source: &[u8],
    format: DocumentIngressFormatV1,
) -> PyResult<PyDocumentSession> {
    ingress_result(py, load_document_utf8_bytes_with_budget(source, format))
}

fn exact_bytes<'py>(source: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyBytes>> {
    if !source.is_exact_instance_of::<PyBytes>() {
        return Err(PyTypeError::new_err(
            "document source must be exact built-in bytes",
        ));
    }
    source.clone().cast_into::<PyBytes>().map_err(Into::into)
}

fn exact_path(path: &Bound<'_, PyAny>) -> PyResult<PathBuf> {
    if !path.is_exact_instance_of::<PyString>() {
        return Err(PyTypeError::new_err(
            "document file path must be an exact built-in str",
        ));
    }
    Ok(PathBuf::from(path.cast::<PyString>()?.to_str()?))
}

fn exact_budget(value: &Bound<'_, PyAny>) -> PyResult<XmlInputBudgetV1> {
    if !value.is_exact_instance_of::<PyXmlInputBudgetV1>() {
        return Err(PyTypeError::new_err(
            "document budget must be an exact XmlInputBudgetV1",
        ));
    }
    Ok(value.extract::<PyRef<'_, PyXmlInputBudgetV1>>()?.budget)
}

fn exact_usize(value: &Bound<'_, PyAny>, name: &str) -> PyResult<usize> {
    if !value.is_exact_instance_of::<PyInt>() {
        return Err(PyTypeError::new_err(format!(
            "{name} must be an exact built-in nonnegative integer"
        )));
    }
    value.extract::<usize>().map_err(|_| {
        PyTypeError::new_err(format!(
            "{name} must be a nonnegative integer representable as usize"
        ))
    })
}

fn ingress_result(
    py: Python<'_>,
    result: Result<ferrum_document::DocumentSession, DocumentIngressErrorV1>,
) -> PyResult<PyDocumentSession> {
    match result {
        Ok(session) => Ok(PyDocumentSession::from_session(session)),
        Err(error) => Err(map_ingress_error(py, error)?),
    }
}

fn map_ingress_error(py: Python<'_>, error: DocumentIngressErrorV1) -> PyResult<PyErr> {
    match error {
        DocumentIngressErrorV1::Read { origin, .. } => {
            input_error(py, origin, "read", None, None, None)
        }
        DocumentIngressErrorV1::SourcePolicy { origin, reason } => match reason {
            SourcePolicyErrorV1::ByteLimitSentinelUnrepresentable { limit } => {
                input_error(py, origin, "source_policy", Some(limit), None, None)
            }
            SourcePolicyErrorV1::Symlink | SourcePolicyErrorV1::NonRegularFile => {
                input_error(py, origin, "source_policy", None, None, None)
            }
        },
        DocumentIngressErrorV1::ByteLimitExceeded {
            origin,
            limit,
            observed_at_least,
        } => input_error(
            py,
            origin,
            "bytes",
            Some(limit),
            None,
            Some(observed_at_least),
        ),
        DocumentIngressErrorV1::Utf8 { origin, .. } => {
            input_error(py, origin, "utf8", None, None, None)
        }
        DocumentIngressErrorV1::Cdml { origin, source } => match source {
            CdmlIngressErrorV1::XmlInput(source) => xml_input_error(py, origin, "cdml", source),
            CdmlIngressErrorV1::Typed(source) => Ok(DocumentLoadError::new_err(source.to_string())),
            CdmlIngressErrorV1::Session(source) => {
                Ok(DocumentLoadError::new_err(source.to_string()))
            }
        },
        DocumentIngressErrorV1::Cdsvg { origin, source } => map_cdsvg_error(py, origin, source),
    }
}

fn map_cdsvg_error(
    py: Python<'_>,
    origin: DocumentIngressOriginV1,
    error: CdsvgExtractionError,
) -> PyResult<PyErr> {
    match error {
        CdsvgExtractionError::WrapperInput(source) => {
            xml_input_error(py, origin, "cdsvg_wrapper", source)
        }
        CdsvgExtractionError::PayloadInput(source) => {
            xml_input_error(py, origin, "cdsvg_payload", source)
        }
        CdsvgExtractionError::Typed(TypedDocumentError::XmlInput(source)) => {
            xml_input_error(py, origin, "cdsvg_payload", source)
        }
        CdsvgExtractionError::Typed(source) => Ok(DocumentLoadError::new_err(source.to_string())),
        _source => input_error(py, origin, "cdsvg_wrapper", None, None, None),
    }
}

fn xml_input_error(
    py: Python<'_>,
    origin: DocumentIngressOriginV1,
    stage: &'static str,
    error: XmlInputError,
) -> PyResult<PyErr> {
    match error {
        XmlInputError::Budget(error) => xml_budget_error(py, origin, stage, error),
        XmlInputError::DtdForbidden | XmlInputError::Preflight(_) | XmlInputError::Xml(_) => {
            input_error(py, origin, stage, None, None, None)
        }
    }
}

fn xml_budget_error(
    py: Python<'_>,
    origin: DocumentIngressOriginV1,
    stage: &'static str,
    error: XmlBudgetError,
) -> PyResult<PyErr> {
    let (limit, actual) = match error {
        XmlBudgetError::Utf8Bytes { limit, actual }
        | XmlBudgetError::Elements { limit, actual }
        | XmlBudgetError::Depth { limit, actual }
        | XmlBudgetError::Attributes { limit, actual }
        | XmlBudgetError::TextBytes { limit, actual } => (limit, actual),
    };
    input_error(py, origin, stage, Some(limit), Some(actual), None)
}

fn input_error(
    py: Python<'_>,
    origin: DocumentIngressOriginV1,
    stage: &'static str,
    limit: Option<usize>,
    actual: Option<usize>,
    observed_at_least: Option<usize>,
) -> PyResult<PyErr> {
    let error = DocumentInputError::new_err(format!("document input rejected at {stage}"));
    let value = error.value(py);
    value.setattr("origin", origin_name(&origin))?;
    value.setattr("stage", stage)?;
    value.setattr("limit", limit)?;
    value.setattr("actual", actual)?;
    value.setattr("observed_at_least", observed_at_least)?;
    Ok(error)
}

fn origin_name(origin: &DocumentIngressOriginV1) -> &'static str {
    match origin {
        DocumentIngressOriginV1::Bytes => "bytes",
        DocumentIngressOriginV1::StandardInput => "standard_input",
        DocumentIngressOriginV1::File(_) => "file",
    }
}
