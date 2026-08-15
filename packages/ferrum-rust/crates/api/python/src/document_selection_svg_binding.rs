//! Private selected-root SVG boundary for the bundled Ferrum-Qt application.
//!
//! The entry point stays absent from the wheel stub, CLI, serde, and wire APIs.

use ferrum_api::{
    DocumentRenderIdentityV1, DocumentSelectionSvgRootV1, DocumentSelectionSvgV1,
    DocumentSvgSelectionV1, LOCAL_SVG_COMPLETED_BYTES_V1, SvgOutputBudgetV1,
    render_document_selection_to_svg_v1,
};
use ferrum_document::{DocumentObjectIdV1, SessionDocumentObservationV1};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString, PyTuple};

use crate::binding::FerrumError;
use crate::projection_binding::PySessionDocumentObservationV1;

create_exception!(ferrum_chem, DocumentSelectionSvgError, FerrumError);

const RESOURCE_REASON: &str = "selected SVG could not reserve result storage";
const SELECTION_SHAPE_REASON: &str =
    "selected SVG selectors must be one nonempty exact tuple of durable object IDs";
const SELECTOR_TEXT_REASON: &str = "selected SVG selector must be valid UTF-8 text";

/// One retained direct render root in the completed selected SVG.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentSelectionSvgRootV1",
    skip_from_py_object
)]
struct PyDocumentSelectionSvgRootV1 {
    #[pyo3(get)]
    source_order: u32,
    #[pyo3(get)]
    identity_kind: String,
    #[pyo3(get)]
    identity: String,
}

/// Conservative fitted viewport measured from native lowered content.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentSelectionSvgViewportV1",
    skip_from_py_object
)]
struct PyDocumentSelectionSvgViewportV1 {
    #[pyo3(get)]
    x: f64,
    #[pyo3(get)]
    y: f64,
    #[pyo3(get)]
    width: f64,
    #[pyo3(get)]
    height: f64,
}

/// One immutable selected-root SVG and its exact source corroborators.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentSelectionSvgV1",
    skip_from_py_object
)]
struct PyDocumentSelectionSvgV1 {
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    source_revision: u64,
    #[pyo3(get)]
    source_digest: String,
    #[pyo3(get)]
    selected_objects: Py<PyTuple>,
    #[pyo3(get)]
    selected_roots: Py<PyTuple>,
    #[pyo3(get)]
    viewport: Py<PyDocumentSelectionSvgViewportV1>,
    #[pyo3(get)]
    svg: String,
}

/// Render exact durable selected objects as complete native SVG roots.
///
/// Experimental internal-to-Ferrum-Qt operation. Atom and bond selection maps
/// to its complete molecule root; presentation selection maps to exact roots.
#[pyfunction]
fn render_document_selection_svg_v1(
    py: Python<'_>,
    observation: PyRef<'_, PySessionDocumentObservationV1>,
    object_ids: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentSelectionSvgV1> {
    let selection = parse_selection(py, observation.observation(), object_ids)?;
    let observation = observation.observation().clone();
    let budget = SvgOutputBudgetV1::new(LOCAL_SVG_COMPLETED_BYTES_V1)
        .map_err(|error| selection_error(py, error.to_string()))?;
    let result =
        py.detach(move || render_document_selection_to_svg_v1(&observation, selection, budget));
    let receipt = result.map_err(|error| selection_error(py, error.to_string()))?;
    receipt_to_python(py, &receipt)
}

fn parse_selection(
    py: Python<'_>,
    observation: &SessionDocumentObservationV1,
    object_ids: &Bound<'_, PyAny>,
) -> PyResult<DocumentSvgSelectionV1> {
    if !object_ids.is_exact_instance_of::<PyTuple>() {
        return Err(selection_error(py, SELECTION_SHAPE_REASON));
    }
    let object_ids = object_ids.cast::<PyTuple>()?;
    let maximum =
        selectable_object_count(observation).ok_or_else(|| selection_error(py, RESOURCE_REASON))?;
    if object_ids.is_empty() || object_ids.len() > maximum {
        return Err(selection_error(py, SELECTION_SHAPE_REASON));
    }
    let mut selectors = Vec::new();
    selectors
        .try_reserve_exact(object_ids.len())
        .map_err(|_| selection_error(py, RESOURCE_REASON))?;
    for item in object_ids.iter() {
        if !item.is_exact_instance_of::<PyString>() {
            return Err(selection_error(py, SELECTION_SHAPE_REASON));
        }
        let selector = item
            .cast::<PyString>()?
            .to_str()
            .map_err(|_| selection_error(py, SELECTOR_TEXT_REASON))?;
        let selector = copied(py, selector)?;
        selectors.push(
            DocumentObjectIdV1::parse(selector)
                .map_err(|error| selection_error(py, error.to_string()))?,
        );
    }
    DocumentSvgSelectionV1::new(selectors).map_err(|error| selection_error(py, error.to_string()))
}

fn selectable_object_count(observation: &SessionDocumentObservationV1) -> Option<usize> {
    let projection = observation.projection();
    let structure = projection
        .molecules()
        .iter()
        .try_fold(0_usize, |count, molecule| {
            count
                .checked_add(usize::from(molecule.id().is_some()))?
                .checked_add(molecule.atoms().len())?
                .checked_add(molecule.bonds().len())
        })?;
    structure.checked_add(projection.presentation_stack().roots().len())
}

fn receipt_to_python(
    py: Python<'_>,
    receipt: &DocumentSelectionSvgV1,
) -> PyResult<PyDocumentSelectionSvgV1> {
    let mut roots = Vec::new();
    roots
        .try_reserve_exact(receipt.selected_roots().len())
        .map_err(|_| selection_error(py, RESOURCE_REASON))?;
    for root in receipt.selected_roots() {
        roots.push(Py::new(py, root_to_python(py, root)?)?);
    }
    let viewport = receipt.viewport();
    Ok(PyDocumentSelectionSvgV1 {
        schema: copied(py, receipt.schema())?,
        source_revision: receipt.source_revision(),
        source_digest: hex_digest(py, receipt.source_digest())?,
        selected_objects: object_tuple(py, receipt.selected_objects())?,
        selected_roots: PyTuple::new(py, roots)?.unbind(),
        viewport: Py::new(
            py,
            PyDocumentSelectionSvgViewportV1 {
                x: viewport.x(),
                y: viewport.y(),
                width: viewport.width(),
                height: viewport.height(),
            },
        )?,
        svg: copied(py, receipt.svg().as_str())?,
    })
}

fn root_to_python(
    py: Python<'_>,
    root: &DocumentSelectionSvgRootV1,
) -> PyResult<PyDocumentSelectionSvgRootV1> {
    let (identity_kind, identity) = match root.identity() {
        DocumentRenderIdentityV1::Durable(value) => ("durable", value),
        DocumentRenderIdentityV1::ProjectionLocal(value) => ("projection_local", value),
    };
    Ok(PyDocumentSelectionSvgRootV1 {
        source_order: root.source_order(),
        identity_kind: copied(py, identity_kind)?,
        identity: copied(py, identity)?,
    })
}

fn object_tuple(py: Python<'_>, values: &[DocumentObjectIdV1]) -> PyResult<Py<PyTuple>> {
    let mut objects = Vec::new();
    objects
        .try_reserve_exact(values.len())
        .map_err(|_| selection_error(py, RESOURCE_REASON))?;
    for value in values {
        objects.push(copied(py, value.as_str())?);
    }
    Ok(PyTuple::new(py, objects)?.unbind())
}

fn copied(py: Python<'_>, value: &str) -> PyResult<String> {
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| selection_error(py, RESOURCE_REASON))?;
    result.push_str(value);
    Ok(result)
}

fn hex_digest(py: Python<'_>, digest: &[u8; 32]) -> PyResult<String> {
    let mut value = String::new();
    value
        .try_reserve_exact(64)
        .map_err(|_| selection_error(py, RESOURCE_REASON))?;
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

fn selection_error(py: Python<'_>, reason: impl Into<String>) -> PyErr {
    let reason = reason.into();
    let error = DocumentSelectionSvgError::new_err(reason.clone());
    if let Err(attribute_error) = error.value(py).setattr("reason", reason) {
        return attribute_error;
    }
    error
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentSelectionSvgError",
        module.py().get_type::<DocumentSelectionSvgError>(),
    )?;
    module.add_class::<PyDocumentSelectionSvgRootV1>()?;
    module.add_class::<PyDocumentSelectionSvgViewportV1>()?;
    module.add_class::<PyDocumentSelectionSvgV1>()?;
    module.add_function(wrap_pyfunction!(render_document_selection_svg_v1, module)?)?;
    Ok(())
}
