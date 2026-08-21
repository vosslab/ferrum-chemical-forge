//! Private frozen PyO3 boundary for exact durable molecule inspection.
//!
//! This unsupported extension entry point is for the bundled Ferrum native
//! route only.  It deliberately has no wheel-stub, CLI, serde, or wire surface.

use ferrum_document::{
    inspect_document_molecule_v1 as inspect_rust_document_molecule_v1,
    DocumentMoleculeInspectionRequestV1, DocumentMoleculeInspectionV1,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyString, PyTuple};

use super::binding::FerrumError;
use super::projection_binding::PySessionDocumentObservationV1;

create_exception!(ferrum_chem, DocumentMoleculeInspectionError, FerrumError);

const DIGEST_REASON: &str = "expected digest must be exactly 64 lowercase hexadecimal characters";
const DIGEST_TEXT_REASON: &str = "expected digest must be valid UTF-8 text";
const RESOURCE_REASON: &str = "molecule inspection could not reserve result storage";
const SELECTOR_TEXT_REASON: &str = "molecule selector must be valid UTF-8 text";

/// One immutable lexical element count from an inspection receipt.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculeElementCountV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentMoleculeElementCountV1 {
    #[pyo3(get)]
    symbol: String,
    #[pyo3(get)]
    atom_count: usize,
}

/// One immutable normalized atom-coordinate bounds receipt.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculeBoundsV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentMoleculeBoundsV1 {
    #[pyo3(get)]
    min_x: f64,
    #[pyo3(get)]
    min_y: f64,
    #[pyo3(get)]
    max_x: f64,
    #[pyo3(get)]
    max_y: f64,
}

/// One immutable source-fact receipt for a durable direct-root molecule.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculeInspectionV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentMoleculeInspectionV1 {
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    source_revision: u64,
    #[pyo3(get)]
    source_digest: String,
    #[pyo3(get)]
    molecule_id: String,
    #[pyo3(get)]
    projection_key: String,
    #[pyo3(get)]
    source_id: String,
    #[pyo3(get)]
    document_root_order: u32,
    #[pyo3(get)]
    authored_name: Option<String>,
    #[pyo3(get)]
    atom_count: usize,
    #[pyo3(get)]
    bond_count: usize,
    #[pyo3(get)]
    element_inventory: Py<PyTuple>,
    #[pyo3(get)]
    total_formal_charge: Option<i64>,
    #[pyo3(get)]
    bounds: Option<PyDocumentMoleculeBoundsV1>,
}

/// Inspect one exact durable direct-root molecule without mutating a session.
///
/// Experimental internal-to-Ferrum API.  The receipt reports retained source
/// facts only: its element inventory is not a molecular formula, and bounds are
/// normalized atom-coordinate bounds in points.
#[pyfunction]
fn inspect_document_molecule_v1(
    py: Python<'_>,
    observation: PyRef<'_, PySessionDocumentObservationV1>,
    expected_revision: u64,
    expected_digest: &Bound<'_, PyString>,
    molecule_id: &Bound<'_, PyString>,
) -> PyResult<PyDocumentMoleculeInspectionV1> {
    let expected_digest = expected_digest
        .to_str()
        .map_err(|_| inspection_error(py, DIGEST_TEXT_REASON))?;
    let expected_digest = parse_digest(py, expected_digest)?;
    let molecule_id = molecule_id
        .to_str()
        .map_err(|_| inspection_error(py, SELECTOR_TEXT_REASON))?;
    let molecule_id = copied(py, molecule_id)?;
    let molecule_id = super::document_error_binding::document_object_id(py, molecule_id)
        .map_err(|error| inspection_error(py, error.to_string()))?;
    let request =
        DocumentMoleculeInspectionRequestV1::new(expected_revision, expected_digest, molecule_id);
    let receipt = inspect_rust_document_molecule_v1(observation.observation(), &request)
        .map_err(|error| inspection_error(py, error.to_string()))?;
    receipt_to_python(py, &receipt)
}

pub(crate) fn receipt_to_python(
    py: Python<'_>,
    receipt: &DocumentMoleculeInspectionV1,
) -> PyResult<PyDocumentMoleculeInspectionV1> {
    let mut elements = Vec::new();
    if elements
        .try_reserve_exact(receipt.element_inventory().len())
        .is_err()
    {
        return Err(resource_error(py)?);
    }
    for entry in receipt.element_inventory() {
        elements.push(PyDocumentMoleculeElementCountV1 {
            symbol: copied(py, entry.symbol())?,
            atom_count: entry.atom_count(),
        });
    }
    let bounds = receipt.bounds().map(|bounds| PyDocumentMoleculeBoundsV1 {
        min_x: bounds.min_x(),
        min_y: bounds.min_y(),
        max_x: bounds.max_x(),
        max_y: bounds.max_y(),
    });
    Ok(PyDocumentMoleculeInspectionV1 {
        schema: copied(py, receipt.schema())?,
        source_revision: receipt.source_revision(),
        source_digest: hex_digest(py, receipt.source_digest())?,
        molecule_id: copied(py, receipt.molecule_id().as_str())?,
        projection_key: copied(py, receipt.projection_key())?,
        source_id: copied(py, receipt.source_id())?,
        document_root_order: receipt.document_root_order(),
        authored_name: receipt
            .authored_name()
            .map(|name| copied(py, name))
            .transpose()?,
        atom_count: receipt.atom_count(),
        bond_count: receipt.bond_count(),
        element_inventory: PyTuple::new(py, elements)?.unbind(),
        total_formal_charge: receipt.total_formal_charge(),
        bounds,
    })
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(inspection_error(py, DIGEST_REASON));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = hex_value(pair[0]);
        let low = hex_value(pair[1]);
        digest[index] = (high << 4) | low;
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
    if value.try_reserve_exact(64).is_err() {
        return Err(resource_error(py)?);
    }
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
    if result.try_reserve_exact(value.len()).is_err() {
        return Err(resource_error(py)?);
    }
    result.push_str(value);
    Ok(result)
}

fn resource_error(py: Python<'_>) -> PyResult<PyErr> {
    let error = DocumentMoleculeInspectionError::new_err(RESOURCE_REASON);
    error.value(py).setattr("reason", RESOURCE_REASON)?;
    Ok(error)
}

fn inspection_error(py: Python<'_>, reason: impl Into<String>) -> PyErr {
    let reason = reason.into();
    let error = DocumentMoleculeInspectionError::new_err(reason.clone());
    if let Err(attribute_error) = error.value(py).setattr("reason", reason) {
        return attribute_error;
    }
    error
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentMoleculeInspectionError",
        module.py().get_type::<DocumentMoleculeInspectionError>(),
    )?;
    module.add_class::<PyDocumentMoleculeElementCountV1>()?;
    module.add_class::<PyDocumentMoleculeBoundsV1>()?;
    module.add_class::<PyDocumentMoleculeInspectionV1>()?;
    module.add_function(wrap_pyfunction!(inspect_document_molecule_v1, module)?)?;
    Ok(())
}
