//! Explicit Python admission of untrusted CDML and CD-SVG bytes or local files.

use std::{path::PathBuf, sync::Arc};

use ferrum_document::artifact_publication_v1::RetainedSourceFileGuardV1;
use ferrum_document::{
    CdmlIngressBudgetV1, CdmlIngressErrorV1, CdsvgIngressBudgetV1, DocumentIngressErrorV1,
    DocumentIngressFormatV1, DocumentIngressOriginV1, SourcePolicyErrorV1,
    load_document_file_with_budget, load_document_utf8_bytes_with_budget,
    prepare_local_cdml_file_with_origin_v1, prepare_local_decoded_cdsvg_file_with_origin_v1,
};
use ferrum_document::{
    CdsvgExtractionError, DocumentRenderObservationErrorV1, DocumentRenderObservationV1,
    DocumentSession, TypedDocumentError, XmlBudgetError, XmlInputBudgetV1, XmlInputError,
    observe_document_render_v1,
};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes, PyInt, PyString};

use super::binding::PyDocumentSession;
use super::document_error_binding::{
    DocumentInputError, DocumentLoadError, PreparedOperationConsumedError,
};
use crate::{
    CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1, InterchangeDirectionV1, InterchangeFormatRegistryV1,
    LocalDocumentIngressDirectionV1, LocalDocumentIngressRegistryV1,
    interchange_import_v1::InterchangeImportRefusalV1,
};

/// One closed local container kind carried by a prepared desktop admission.
#[derive(Clone, Copy)]
enum LocalDocumentSourceKindV1 {
    Cdml,
    DecodedCdsvg,
    Cml,
    Interchange,
}

impl LocalDocumentSourceKindV1 {
    const fn for_interchange_descriptor(
        descriptor: &crate::interchange_import_v1::InterchangeFormatDescriptorV1,
    ) -> Self {
        match descriptor.decoder() {
            crate::interchange_import_v1::InterchangeDecoderKeyV1::CmlSimpleMolecule => Self::Cml,
            crate::interchange_import_v1::InterchangeDecoderKeyV1::Sdf => Self::Interchange,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Cdml => "cdml",
            Self::DecodedCdsvg => "decoded_cdsvg",
            Self::Cml => "cml",
            Self::Interchange => "interchange",
        }
    }
}

/// Immutable API-owned facts for one ordinary File/Open interchange route.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "LocalInterchangeOpenDescriptorV1"
)]
pub(crate) struct PyLocalInterchangeOpenDescriptorV1 {
    #[pyo3(get)]
    display_name: String,
    #[pyo3(get)]
    suffixes: Vec<String>,
    #[pyo3(get)]
    source_kind: String,
    #[pyo3(get)]
    allows_current_tab_replacement: bool,
    route_handle: Option<Py<PyLocalInterchangeOpenRouteHandleV1>>,
}

/// Opaque registry identity issued only inside an eligible File/Open descriptor.
///
/// This type deliberately has no Python constructor, fields, comparison, or
/// serialization surface.  Python can retain a descriptor-issued handle and
/// return it to the preparation boundary, but cannot mint a route selector.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "LocalInterchangeOpenRouteHandleV1",
    skip_from_py_object
)]
pub(crate) struct PyLocalInterchangeOpenRouteHandleV1 {
    format_id: &'static str,
}

#[pymethods]
impl PyLocalInterchangeOpenDescriptorV1 {
    #[getter]
    fn route_handle(&self, py: Python<'_>) -> Option<Py<PyLocalInterchangeOpenRouteHandleV1>> {
        self.route_handle
            .as_ref()
            .map(|handle| handle.clone_ref(py))
    }
}

/// One worker-safe, one-use local-document admission ready for UI-thread ownership.
///
/// The session is created entirely in Rust on the calling worker thread. Python
/// cannot inspect or mutate it until `take_admission_v1` transfers the session
/// and its authenticated observation exactly once.
#[pyclass(
    module = "ferrum_chem",
    name = "PreparedLocalDocumentOpenV1",
    skip_from_py_object
)]
pub(crate) struct PyPreparedLocalDocumentOpenV1 {
    session: Option<DocumentSession>,
    observation: Option<super::render_binding::PyRenderObservationV1>,
    origin: Option<PyLocalDocumentOriginTokenV1>,
    source_kind: Option<LocalDocumentSourceKindV1>,
    interchange_summary:
        Option<super::document_interchange_receipt_binding::PyLocalInterchangeImportSummaryV1>,
}

/// Opaque equality-only descriptor identity for one admitted local document source.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "LocalDocumentOriginTokenV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyLocalDocumentOriginTokenV1 {
    source: Arc<RetainedSourceFileGuardV1>,
}

#[pymethods]
impl PyLocalDocumentOriginTokenV1 {
    fn __richcmp__(&self, other: PyRef<'_, Self>, compare: pyo3::basic::CompareOp) -> bool {
        match compare {
            pyo3::basic::CompareOp::Eq => self.source.identity() == other.source.identity(),
            pyo3::basic::CompareOp::Ne => self.source.identity() != other.source.identity(),
            _ => false,
        }
    }
}

impl PyLocalDocumentOriginTokenV1 {
    pub(crate) fn try_clone_source(&self) -> Result<RetainedSourceFileGuardV1, std::io::Error> {
        self.source.try_clone()
    }
}

#[pymethods]
impl PyPreparedLocalDocumentOpenV1 {
    /// Return safe generic interchange facts without exposing a commit bypass.
    #[getter]
    fn interchange_summary(
        &self,
    ) -> Option<super::document_interchange_receipt_binding::PyLocalInterchangeImportSummaryV1>
    {
        self.interchange_summary.clone()
    }

    /// Consume this admission and establish one thread-affine document session.
    fn take_admission_v1(
        &mut self,
    ) -> PyResult<(
        PyDocumentSession,
        super::render_binding::PyRenderObservationV1,
        PyLocalDocumentOriginTokenV1,
        String,
    )> {
        match (
            self.session.take(),
            self.observation.take(),
            self.origin.take(),
            self.source_kind.take(),
        ) {
            (Some(session), Some(observation), Some(origin), Some(source_kind)) => Ok((
                PyDocumentSession::from_session(session),
                observation,
                origin,
                source_kind.as_str().to_owned(),
            )),
            _ => Err(PreparedOperationConsumedError::new_err(
                "local document admission receipt was already consumed",
            )),
        }
    }
}

enum LocalDocumentOpenPreparationError {
    Ingress(DocumentIngressErrorV1),
    Render(DocumentRenderObservationErrorV1),
}

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

#[pymethods]
impl PyDocumentSession {
    /// Return every accepted Rust-owned local document route for Qt File/Open.
    #[staticmethod]
    fn local_document_open_descriptors_v1(
        py: Python<'_>,
    ) -> PyResult<Vec<PyLocalInterchangeOpenDescriptorV1>> {
        LocalDocumentIngressRegistryV1::descriptors()
            .iter()
            .map(|descriptor| {
                let route_handle = if descriptor.route()
                    == crate::LocalDocumentIngressRouteV1::CmlSimpleMolecule
                {
                    Some(Py::new(
                        py,
                        PyLocalInterchangeOpenRouteHandleV1 {
                            format_id: CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1,
                        },
                    )?)
                } else {
                    None
                };
                Ok(PyLocalInterchangeOpenDescriptorV1 {
                    display_name: descriptor.display_name().to_owned(),
                    suffixes: descriptor
                        .suffixes()
                        .iter()
                        .map(|suffix| (*suffix).to_owned())
                        .collect(),
                    source_kind: descriptor.route().source_kind().to_owned(),
                    allows_current_tab_replacement: descriptor.direction()
                        == LocalDocumentIngressDirectionV1::ReplacePristineOrNewTab,
                    route_handle,
                })
            })
            .collect()
    }

    /// Return immutable Rust-owned interchange routes eligible for File/Open.
    #[staticmethod]
    fn local_interchange_open_descriptors_v1(
        py: Python<'_>,
    ) -> PyResult<Vec<PyLocalInterchangeOpenDescriptorV1>> {
        InterchangeFormatRegistryV1::descriptors()
            .iter()
            .filter(|descriptor| {
                descriptor
                    .directions()
                    .contains(&InterchangeDirectionV1::DocumentImportNew)
            })
            .map(|descriptor| {
                Ok(PyLocalInterchangeOpenDescriptorV1 {
                    display_name: descriptor.display_name().to_owned(),
                    suffixes: descriptor
                        .input_suffixes()
                        .iter()
                        .map(|suffix| (*suffix).to_owned())
                        .collect(),
                    source_kind: LocalDocumentSourceKindV1::for_interchange_descriptor(descriptor)
                        .as_str()
                        .to_owned(),
                    allows_current_tab_replacement: false,
                    route_handle: Some(Py::new(
                        py,
                        PyLocalInterchangeOpenRouteHandleV1 {
                            format_id: descriptor.format_id(),
                        },
                    )?),
                })
            })
            .collect::<PyResult<Vec<_>>>()
    }

    /// Prepare one registry-issued local interchange source for a new document.
    #[staticmethod]
    fn prepare_local_interchange_file_v1(
        py: Python<'_>,
        path: &Bound<'_, PyAny>,
        route_handle: &Bound<'_, PyAny>,
    ) -> PyResult<PyPreparedLocalDocumentOpenV1> {
        prepare_local_interchange_file_v1(py, path, route_handle)
    }

    /// Read one descriptor-authorized local interchange source as bounded UTF-8.
    ///
    /// This source-only capability is for current-document import adapters.  It
    /// neither decodes the source nor creates or commits a document.
    #[staticmethod]
    fn read_local_interchange_utf8_v1(
        py: Python<'_>,
        path: &Bound<'_, PyAny>,
        route_handle: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        read_local_interchange_utf8_v1(py, path, route_handle)
    }

    /// Admit exact built-in bytes as CDML under one explicit caller-owned budget.
    #[staticmethod]
    fn load_utf8_bytes_with_budget(
        py: Python<'_>,
        source: &Bound<'_, PyAny>,
        budget: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        load_utf8_bytes_with_budget(py, source, budget)
    }

    /// Admit an exact built-in string local path as CDML under one explicit budget.
    #[staticmethod]
    fn load_file_with_budget(
        py: Python<'_>,
        path: &Bound<'_, PyAny>,
        budget: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        load_file_with_budget(py, path, budget)
    }

    /// Prepare one ordinary local CDML file through Rust's immutable V1 profile.
    #[staticmethod]
    fn prepare_local_cdml_file_v1(
        py: Python<'_>,
        path: &Bound<'_, PyAny>,
    ) -> PyResult<PyPreparedLocalDocumentOpenV1> {
        prepare_local_cdml_file_v1(py, path)
    }

    /// Prepare one decoded local CD-SVG file through Rust's immutable V1 profile.
    #[staticmethod]
    fn prepare_local_decoded_cdsvg_file_v1(
        py: Python<'_>,
        path: &Bound<'_, PyAny>,
    ) -> PyResult<PyPreparedLocalDocumentOpenV1> {
        prepare_local_decoded_cdsvg_file_v1(py, path)
    }

    /// Admit exact built-in bytes as CD-SVG under independent wrapper and payload budgets.
    #[staticmethod]
    fn load_cdsvg_utf8_bytes_with_budget(
        py: Python<'_>,
        source: &Bound<'_, PyAny>,
        wrapper: &Bound<'_, PyAny>,
        payload: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        load_cdsvg_utf8_bytes_with_budget(py, source, wrapper, payload)
    }

    /// Admit an exact built-in string local path as CD-SVG with independent budgets.
    #[staticmethod]
    fn load_cdsvg_file_with_budget(
        py: Python<'_>,
        path: &Bound<'_, PyAny>,
        wrapper: &Bound<'_, PyAny>,
        payload: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        load_cdsvg_file_with_budget(py, path, wrapper, payload)
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
    let path = exact_path(py, path)?;
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

pub(crate) fn prepare_local_cdml_file_v1(
    py: Python<'_>,
    path: &Bound<'_, PyAny>,
) -> PyResult<PyPreparedLocalDocumentOpenV1> {
    prepare_local_document_file_v1(py, path, LocalDocumentSourceKindV1::Cdml)
}

pub(crate) fn prepare_local_decoded_cdsvg_file_v1(
    py: Python<'_>,
    path: &Bound<'_, PyAny>,
) -> PyResult<PyPreparedLocalDocumentOpenV1> {
    prepare_local_document_file_v1(py, path, LocalDocumentSourceKindV1::DecodedCdsvg)
}

pub(crate) fn prepare_local_interchange_file_v1(
    py: Python<'_>,
    path: &Bound<'_, PyAny>,
    route_handle: &Bound<'_, PyAny>,
) -> PyResult<PyPreparedLocalDocumentOpenV1> {
    let path = exact_path(py, path)?;
    let descriptor = local_interchange_descriptor(route_handle)?;
    let source_kind = LocalDocumentSourceKindV1::for_interchange_descriptor(descriptor);
    let result = py.detach(move || {
        let (source, prepared) =
            crate::document_interchange_import_v1::prepare_local_interchange_new_document_v1(
                descriptor,
                &path,
                &super::super::StagedExtensionInterchangeRuntimeResolverV1,
            )
            .map_err(LocalInterchangePreparationError::Refused)?;
        let (session, summary) = prepared
            .commit_and_take_session()
            .map_err(LocalInterchangePreparationError::Refused)?;
        let summary = super::document_interchange_receipt_binding::PyLocalInterchangeImportSummaryV1::from_summary(&summary);
        let post_import_snapshot = session
            .snapshot()
            .map_err(|_| LocalInterchangePreparationError::Refused(
                InterchangeImportRefusalV1::for_reason(
                    crate::interchange_import_v1::InterchangeImportRefusalReasonV1::InternalFailure,
                ),
            ))?;
        let observation = observe_document_render_v1(&session, post_import_snapshot.revision())
            .map_err(LocalInterchangePreparationError::Render)?;
        Ok::<_, LocalInterchangePreparationError>((
            session,
            observation,
            source
                .retained_source()
                .ok_or_else(|| {
                    LocalInterchangePreparationError::Refused(
                        InterchangeImportRefusalV1::for_reason(
                            crate::interchange_import_v1::InterchangeImportRefusalReasonV1::InternalFailure,
                        ),
                    )
                })?
                .try_clone()
                .map_err(|_| {
                    LocalInterchangePreparationError::Refused(
                        InterchangeImportRefusalV1::for_reason(
                            crate::interchange_import_v1::InterchangeImportRefusalReasonV1::InternalFailure,
                        ),
                    )
                })?,
            source_kind,
            summary,
        ))
    });
    match result {
        Ok((session, observation, origin, source_kind, interchange_summary)) => {
            Ok(PyPreparedLocalDocumentOpenV1 {
                session: Some(session),
                observation: Some(super::render_binding::observation(py, observation)?),
                origin: Some(PyLocalDocumentOriginTokenV1 {
                    source: Arc::new(origin),
                }),
                source_kind: Some(source_kind),
                interchange_summary: Some(interchange_summary),
            })
        }
        Err(LocalInterchangePreparationError::Refused(refusal)) => Err(
            super::document_interchange_receipt_binding::local_interchange_refusal(py, refusal)?,
        ),
        Err(LocalInterchangePreparationError::Render(error)) => {
            Err(super::render_binding::error_result(py, error)?)
        }
    }
}

pub(crate) fn read_local_interchange_utf8_v1(
    py: Python<'_>,
    path: &Bound<'_, PyAny>,
    route_handle: &Bound<'_, PyAny>,
) -> PyResult<String> {
    let path = exact_path(py, path)?;
    let descriptor = local_interchange_descriptor(route_handle)?;
    let result = py.detach(move || {
        crate::document_interchange_import_v1::read_local_interchange_utf8_source_v1(
            descriptor, &path,
        )
    });
    match result {
        Ok(source) => Ok(source),
        Err(refusal) => Err(
            super::document_interchange_receipt_binding::local_interchange_refusal(py, refusal)?,
        ),
    }
}

fn local_interchange_descriptor(
    route_handle: &Bound<'_, PyAny>,
) -> PyResult<&'static crate::interchange_import_v1::InterchangeFormatDescriptorV1> {
    let route_handle = exact_route_handle(route_handle)?;
    InterchangeFormatRegistryV1::descriptors()
        .iter()
        .find(|descriptor| descriptor.format_id() == route_handle.format_id)
        .filter(|descriptor| {
            descriptor
                .directions()
                .contains(&InterchangeDirectionV1::DocumentImportNew)
        })
        .ok_or_else(|| PyTypeError::new_err("local interchange route handle is not API-issued"))
}

enum LocalInterchangePreparationError {
    Refused(InterchangeImportRefusalV1),
    Render(DocumentRenderObservationErrorV1),
}

fn prepare_local_document_file_v1(
    py: Python<'_>,
    path: &Bound<'_, PyAny>,
    source_kind: LocalDocumentSourceKindV1,
) -> PyResult<PyPreparedLocalDocumentOpenV1> {
    let path = exact_path(py, path)?;
    let result = py.detach(move || {
        let preparation = match source_kind {
            LocalDocumentSourceKindV1::Cdml => prepare_local_cdml_file_with_origin_v1(&path),
            LocalDocumentSourceKindV1::DecodedCdsvg => {
                prepare_local_decoded_cdsvg_file_with_origin_v1(&path)
            }
            LocalDocumentSourceKindV1::Cml | LocalDocumentSourceKindV1::Interchange => {
                unreachable!("interchange admission uses its dedicated Rust-owned bridge")
            }
        };
        let (session, origin) = preparation.map_err(LocalDocumentOpenPreparationError::Ingress)?;
        let observation = observe_document_render_v1(&session, 0)
            .map_err(LocalDocumentOpenPreparationError::Render)?;
        Ok::<
            (
                DocumentSession,
                DocumentRenderObservationV1,
                RetainedSourceFileGuardV1,
            ),
            LocalDocumentOpenPreparationError,
        >((session, observation, origin))
    });
    match result {
        Ok((session, observation, origin)) => Ok(PyPreparedLocalDocumentOpenV1 {
            session: Some(session),
            observation: Some(super::render_binding::observation(py, observation)?),
            origin: Some(PyLocalDocumentOriginTokenV1 {
                source: Arc::new(origin),
            }),
            source_kind: Some(source_kind),
            interchange_summary: None,
        }),
        Err(LocalDocumentOpenPreparationError::Ingress(error)) => {
            Err(map_local_document_open_error(py, error)?)
        }
        Err(LocalDocumentOpenPreparationError::Render(error)) => {
            Err(super::render_binding::error_result(py, error)?)
        }
    }
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
    let path = exact_path(py, path)?;
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

fn exact_path(py: Python<'_>, path: &Bound<'_, PyAny>) -> PyResult<PathBuf> {
    if !path.is_exact_instance_of::<PyString>() {
        return Err(PyTypeError::new_err(
            "document file path must be an exact built-in str",
        ));
    }
    let path = match path.cast::<PyString>()?.to_str() {
        Ok(path) => path,
        Err(_) => {
            return Err(input_error(
                py,
                DocumentIngressOriginV1::File(PathBuf::new()),
                "path",
                None,
                None,
                None,
            )?);
        }
    };
    let mut owned = String::new();
    if owned.try_reserve_exact(path.len()).is_err() {
        return Err(input_error(
            py,
            DocumentIngressOriginV1::File(PathBuf::new()),
            "resource",
            None,
            None,
            None,
        )?);
    }
    owned.push_str(path);
    Ok(PathBuf::from(owned))
}

fn exact_route_handle<'py>(
    route_handle: &Bound<'py, PyAny>,
) -> PyResult<PyRef<'py, PyLocalInterchangeOpenRouteHandleV1>> {
    if !route_handle.is_exact_instance_of::<PyLocalInterchangeOpenRouteHandleV1>() {
        return Err(PyTypeError::new_err(
            "local interchange route handle must be an API-issued handle",
        ));
    }
    Ok(route_handle.extract::<PyRef<'_, PyLocalInterchangeOpenRouteHandleV1>>()?)
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

/// Translate desktop preparation failures into the closed UI recovery categories.
///
/// The existing typed input exception remains the private transport so ordinary
/// direct-CDML callers retain their established error shape. These two added
/// attributes contain no source bytes or paths.
fn map_local_document_open_error(py: Python<'_>, error: DocumentIngressErrorV1) -> PyResult<PyErr> {
    let category = match &error {
        DocumentIngressErrorV1::Read { .. } | DocumentIngressErrorV1::SourcePolicy { .. } => {
            "source_rejected"
        }
        DocumentIngressErrorV1::ByteLimitExceeded { .. } => "resource_limit",
        DocumentIngressErrorV1::Utf8 { .. } => "wrapper_rejected",
        DocumentIngressErrorV1::Cdml { .. } => "embedded_cdml_rejected",
        DocumentIngressErrorV1::Cdsvg { source, .. } => match source {
            CdsvgExtractionError::MissingCdmlPayload => "embedded_cdml_not_found",
            CdsvgExtractionError::MultipleCdmlPayload { .. } => "multiple_embedded_cdml",
            CdsvgExtractionError::WrapperInput(XmlInputError::Budget(_))
            | CdsvgExtractionError::PayloadInput(XmlInputError::Budget(_))
            | CdsvgExtractionError::Typed(TypedDocumentError::XmlInput(XmlInputError::Budget(_))) => {
                "resource_limit"
            }
            CdsvgExtractionError::PayloadInput(_) | CdsvgExtractionError::Typed(_) => {
                "embedded_cdml_rejected"
            }
            _ => "wrapper_rejected",
        },
    };
    let py_error = map_ingress_error(py, error)?;
    let value = py_error.value(py);
    value.setattr("category", category)?;
    value.setattr("detail", "local document admission rejected")?;
    Ok(py_error)
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

#[cfg(test)]
#[path = "document_ingress_binding_tests.rs"]
mod tests;
