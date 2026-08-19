//! Private PyO3 seam for native explicit fragment creation and inspection.

use ferrum_document::{
    DocumentExplicitFragmentRequestV1, create_document_explicit_fragment_v1,
    inspect_document_explicit_fragments_v1 as inspect_rust_document_explicit_fragments_v1,
};
use ferrum_document::{DocumentObjectIdV1, DocumentSession, PersistentId};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString, PyTuple};

use crate::binding::{FerrumError, PySessionOperationResultV1};
use crate::projection_binding::PySessionDocumentObservationV1;

create_exception!(ferrum_chem, DocumentExplicitFragmentError, FerrumError);
const RESOURCE_REASON: &str = "explicit fragment could not reserve private input or result storage";

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentExplicitFragmentRecordV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentExplicitFragmentRecordV1 {
    #[pyo3(get)]
    fragment_id: String,
    #[pyo3(get)]
    fragment_type: String,
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    molecule_id: String,
    #[pyo3(get)]
    bond_ids: Py<PyTuple>,
    #[pyo3(get)]
    atom_ids: Py<PyTuple>,
}

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentExplicitFragmentCreateResultV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentExplicitFragmentCreateResultV1 {
    #[pyo3(get)]
    operation: PySessionOperationResultV1,
    #[pyo3(get)]
    fragment: Py<PyDocumentExplicitFragmentRecordV1>,
}

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentExplicitFragmentObservationV1",
    skip_from_py_object
)]
struct PyDocumentExplicitFragmentObservationV1 {
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    source_revision: u64,
    #[pyo3(get)]
    source_digest: String,
    #[pyo3(get)]
    records: Py<PyTuple>,
    #[pyo3(get)]
    has_retained_fragment_metadata: bool,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_explicit_fragment_v1(
    py: Python<'_>,
    session: &mut DocumentSession,
    expected_revision: u64,
    expected_digest: &Bound<'_, PyString>,
    molecule_id: &Bound<'_, PyString>,
    name: &Bound<'_, PyString>,
    selected_atom_ids: &Bound<'_, PyAny>,
    selected_bond_ids: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentExplicitFragmentCreateResultV1> {
    let digest = digest(py, expected_digest)?;
    let molecule_id = object_id(py, molecule_id)?;
    let name = text(py, name, "fragment name")?;
    let atoms = ids(py, selected_atom_ids, "selected atom IDs")?;
    let bonds = ids(py, selected_bond_ids, "selected bond IDs")?;
    let request = DocumentExplicitFragmentRequestV1::new(
        expected_revision,
        digest,
        molecule_id,
        name,
        atoms,
        bonds,
    );
    let result = create_document_explicit_fragment_v1(session, request)
        .map_err(|error| feature_error(py, error.to_string()))?;
    Ok(PyDocumentExplicitFragmentCreateResultV1 {
        operation: result.operation().clone().into(),
        fragment: Py::new(py, record(py, result.record())?)?,
    })
}

#[pyfunction]
fn inspect_document_explicit_fragments_v1(
    py: Python<'_>,
    observation: PyRef<'_, PySessionDocumentObservationV1>,
    expected_revision: u64,
    expected_digest: &Bound<'_, PyString>,
) -> PyResult<PyDocumentExplicitFragmentObservationV1> {
    let digest = digest(py, expected_digest)?;
    let receipt = inspect_rust_document_explicit_fragments_v1(
        observation.observation(),
        expected_revision,
        digest,
    )
    .map_err(|error| feature_error(py, error.to_string()))?;
    let mut records = Vec::new();
    records
        .try_reserve_exact(receipt.records().len())
        .map_err(|_| feature_error(py, RESOURCE_REASON))?;
    for value in receipt.records() {
        records.push(Py::new(py, record(py, value)?)?);
    }
    Ok(PyDocumentExplicitFragmentObservationV1 {
        schema: receipt.schema().to_owned(),
        source_revision: receipt.source_revision(),
        source_digest: hex(receipt.source_digest()),
        records: PyTuple::new(py, records)?.unbind(),
        has_retained_fragment_metadata: receipt.has_retained_fragment_metadata(),
    })
}

fn record(
    py: Python<'_>,
    value: &ferrum_document::DocumentExplicitFragmentRecordV1,
) -> PyResult<PyDocumentExplicitFragmentRecordV1> {
    Ok(PyDocumentExplicitFragmentRecordV1 {
        fragment_id: value.fragment_id().as_str().to_owned(),
        fragment_type: "explicit".to_owned(),
        name: value.name().to_owned(),
        molecule_id: value.molecule_id().as_str().to_owned(),
        bond_ids: tuple_ids(py, value.bond_ids())?,
        atom_ids: tuple_ids(py, value.atom_ids())?,
    })
}
fn tuple_ids(py: Python<'_>, values: &[PersistentId]) -> PyResult<Py<PyTuple>> {
    let mut copied = Vec::new();
    copied
        .try_reserve_exact(values.len())
        .map_err(|_| feature_error(py, RESOURCE_REASON))?;
    copied.extend(values.iter().map(|value| value.as_str()));
    Ok(PyTuple::new(py, copied)?.unbind())
}
fn ids(py: Python<'_>, values: &Bound<'_, PyAny>, label: &str) -> PyResult<Vec<PersistentId>> {
    if !values.is_exact_instance_of::<PyTuple>() {
        return Err(feature_error(
            py,
            format!("{label} must be an exact built-in tuple of strings"),
        ));
    }
    let values = values.cast::<PyTuple>()?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(values.len())
        .map_err(|_| feature_error(py, RESOURCE_REASON))?;
    for value in values.iter() {
        let value = value.cast::<PyString>().map_err(|_| {
            feature_error(
                py,
                format!("{label} must be an exact built-in tuple of strings"),
            )
        })?;
        let value = text(py, value, label)?;
        result
            .push(PersistentId::new(value).map_err(|error| feature_error(py, error.to_string()))?);
    }
    Ok(result)
}
fn object_id(py: Python<'_>, value: &Bound<'_, PyString>) -> PyResult<DocumentObjectIdV1> {
    let value = text(py, value, "molecule selector")?;
    DocumentObjectIdV1::parse(value).map_err(|error| feature_error(py, error.to_string()))
}
fn text(py: Python<'_>, value: &Bound<'_, PyString>, label: &str) -> PyResult<String> {
    let value = value
        .to_str()
        .map_err(|_| feature_error(py, format!("{label} must be valid UTF-8 text")))?;
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| feature_error(py, RESOURCE_REASON))?;
    result.push_str(value);
    Ok(result)
}
fn digest(py: Python<'_>, value: &Bound<'_, PyString>) -> PyResult<[u8; 32]> {
    let value = value
        .to_str()
        .map_err(|_| feature_error(py, "expected digest must be valid UTF-8 text"))?;
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(feature_error(
            py,
            "expected digest must be exactly 64 lowercase hexadecimal characters",
        ));
    }
    let mut result = [0; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        result[index] = (hex_value(pair[0]) << 4) | hex_value(pair[1]);
    }
    Ok(result)
}
const fn hex_value(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}
fn hex(value: &[u8; 32]) -> String {
    let mut result = String::with_capacity(64);
    for byte in value {
        use std::fmt::Write as _;
        let _ = write!(result, "{byte:02x}");
    }
    result
}
fn feature_error(py: Python<'_>, reason: impl AsRef<str>) -> PyErr {
    let reason = reason.as_ref().to_owned();
    let error = DocumentExplicitFragmentError::new_err(reason.clone());
    let _ = error.value(py).setattr("reason", reason);
    error
}
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentExplicitFragmentError",
        module.py().get_type::<DocumentExplicitFragmentError>(),
    )?;
    module.add_function(wrap_pyfunction!(
        inspect_document_explicit_fragments_v1,
        module
    )?)
}
