//! Private neutral bond-capacity receipt for bundled Ferrum-Qt.

use ferrum_api::{
    DocumentBondCapacityNotCheckedReasonV1, DocumentBondCapacityOutcomeV1,
    DocumentBondCapacityRequestV1, inspect_document_bond_capacity_v1 as inspect_bond_capacity,
};
use ferrum_domain::{NeutralBondCapacityAtomOutcomeV1, NeutralBondCapacityAtomRecordV1};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString, PyTuple};

use crate::binding::FerrumError;
use crate::projection_binding::PySessionDocumentObservationV1;

create_exception!(ferrum_chem, DocumentBondCapacityError, FerrumError);

const SELECTOR_SHAPE_REASON: &str = "molecule selectors must be an exact nonempty tuple of strings";
const RESOURCE_REASON: &str = "bond capacity could not reserve result storage";

/// One immutable atom arithmetic result carried only by the private Qt seam.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentBondCapacityAtomV1",
    skip_from_py_object
)]
struct PyDocumentBondCapacityAtomV1 {
    #[pyo3(get)]
    source_id: Option<String>,
    #[pyo3(get)]
    element: String,
    #[pyo3(get)]
    explicit_hydrogens: Py<PyDocumentBondCapacityExplicitHydrogensFactV1>,
    #[pyo3(get)]
    formal_charge: Py<PyDocumentBondCapacityFormalChargeFactV1>,
    #[pyo3(get)]
    category: String,
    #[pyo3(get)]
    demand: u16,
    #[pyo3(get)]
    capacity: u16,
}

/// Retained explicit-hydrogen authored presence and calculation value.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentBondCapacityExplicitHydrogensFactV1",
    skip_from_py_object
)]
struct PyDocumentBondCapacityExplicitHydrogensFactV1 {
    #[pyo3(get)]
    was_authored: bool,
    #[pyo3(get)]
    value_or_zero: u16,
}

/// Retained formal-charge authored presence and calculation value.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentBondCapacityFormalChargeFactV1",
    skip_from_py_object
)]
struct PyDocumentBondCapacityFormalChargeFactV1 {
    #[pyo3(get)]
    was_authored: bool,
    #[pyo3(get)]
    value_or_zero: i32,
}

/// One immutable root outcome with direct-root corroboration facts.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentBondCapacityRecordV1",
    skip_from_py_object
)]
struct PyDocumentBondCapacityRecordV1 {
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
    category: String,
    #[pyo3(get)]
    not_checked_reason: Option<String>,
    #[pyo3(get)]
    atoms: Py<PyTuple>,
}

/// Complete immutable result tied to one exact source observation.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentBondCapacityV1",
    skip_from_py_object
)]
struct PyDocumentBondCapacityV1 {
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    source_revision: u64,
    #[pyo3(get)]
    source_digest: String,
    #[pyo3(get)]
    records: Py<PyTuple>,
}

/// Evaluate selected roots using Ferrum's closed neutral bond-capacity profile.
#[pyfunction]
fn inspect_document_bond_capacity_v1(
    py: Python<'_>,
    observation: PyRef<'_, PySessionDocumentObservationV1>,
    expected_revision: u64,
    expected_digest: &Bound<'_, PyString>,
    molecule_ids: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentBondCapacityV1> {
    let digest = expected_digest
        .to_str()
        .map_err(|_| error(py, "expected digest must be valid UTF-8 text"))?;
    let digest = parse_digest(py, digest)?;
    let selectors = parse_selectors(py, molecule_ids)?;
    let request = DocumentBondCapacityRequestV1::new(expected_revision, digest, selectors)
        .map_err(|value| error(py, value.to_string()))?;
    let observation = observation.observation().clone();
    let receipt = py
        .detach(move || inspect_bond_capacity(&observation, &request))
        .map_err(|value| error(py, value.to_string()))?;
    receipt_to_python(py, &receipt)
}

fn receipt_to_python(
    py: Python<'_>,
    receipt: &ferrum_api::DocumentBondCapacityV1,
) -> PyResult<PyDocumentBondCapacityV1> {
    let mut records = Vec::new();
    records
        .try_reserve_exact(receipt.records().len())
        .map_err(|_| error(py, RESOURCE_REASON))?;
    for record in receipt.records() {
        records.push(Py::new(py, record_to_python(py, record)?)?);
    }
    Ok(PyDocumentBondCapacityV1 {
        schema: receipt.schema().to_owned(),
        source_revision: receipt.source_revision(),
        source_digest: digest_text(receipt.source_digest()),
        records: PyTuple::new(py, records)?.unbind(),
    })
}

fn record_to_python(
    py: Python<'_>,
    record: &ferrum_api::DocumentBondCapacityRecordV1,
) -> PyResult<PyDocumentBondCapacityRecordV1> {
    let (category, not_checked_reason, atoms) = outcome_facts(record.outcome());
    let source = record.source();
    Ok(PyDocumentBondCapacityRecordV1 {
        molecule_id: source.molecule_id().as_str().to_owned(),
        projection_key: source.projection_key().to_owned(),
        source_id: source.source_id().to_owned(),
        document_root_order: source.document_root_order(),
        authored_name: source.authored_name().map(str::to_owned),
        category: category.to_owned(),
        not_checked_reason: not_checked_reason.map(str::to_owned),
        atoms: atom_records_to_python(py, atoms)?,
    })
}

fn outcome_facts(
    outcome: &DocumentBondCapacityOutcomeV1,
) -> (&'static str, Option<&'static str>, &[NeutralBondCapacityAtomRecordV1]) {
    match outcome {
        DocumentBondCapacityOutcomeV1::WithinCapacity { atoms } => {
            ("within_capacity", None, atoms.as_slice())
        }
        DocumentBondCapacityOutcomeV1::ExceedsCapacity { atoms } => {
            ("exceeds_capacity", None, atoms.as_slice())
        }
        DocumentBondCapacityOutcomeV1::NotChecked { reason } => {
            ("not_checked", Some(reason_name(*reason)), &[])
        }
    }
}

fn atom_records_to_python(
    py: Python<'_>,
    atoms: &[NeutralBondCapacityAtomRecordV1],
) -> PyResult<Py<PyTuple>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(atoms.len())
        .map_err(|_| error(py, RESOURCE_REASON))?;
    for atom in atoms {
        values.push(Py::new(py, atom_to_python(py, atom)?)?);
    }
    Ok(PyTuple::new(py, values)?.unbind())
}

fn atom_to_python(
    py: Python<'_>,
    atom: &NeutralBondCapacityAtomRecordV1,
) -> PyResult<PyDocumentBondCapacityAtomV1> {
    let (category, demand, capacity) = match atom.outcome {
        NeutralBondCapacityAtomOutcomeV1::WithinCapacity { demand, capacity } => {
            ("within_capacity", demand, capacity)
        }
        NeutralBondCapacityAtomOutcomeV1::ExceedsCapacity { demand, capacity } => {
            ("exceeds_capacity", demand, capacity)
        }
    };
    Ok(PyDocumentBondCapacityAtomV1 {
        source_id: atom.source_id.clone(),
        element: atom.element.clone(),
        explicit_hydrogens: Py::new(
            py,
            PyDocumentBondCapacityExplicitHydrogensFactV1 {
                was_authored: atom.explicit_hydrogens.was_authored,
                value_or_zero: atom.explicit_hydrogens.value_or_zero,
            },
        )?,
        formal_charge: Py::new(
            py,
            PyDocumentBondCapacityFormalChargeFactV1 {
                was_authored: atom.formal_charge.was_authored,
                value_or_zero: atom.formal_charge.value_or_zero,
            },
        )?,
        category: category.to_owned(),
        demand,
        capacity,
    })
}

fn reason_name(reason: DocumentBondCapacityNotCheckedReasonV1) -> &'static str {
    match reason {
        DocumentBondCapacityNotCheckedReasonV1::NonAtomVertex => "non_atom_vertex",
        DocumentBondCapacityNotCheckedReasonV1::NonNeutralCharge => "non_neutral_charge",
        DocumentBondCapacityNotCheckedReasonV1::AuthoredAtomCapacityFact => {
            "authored_atom_capacity_fact"
        }
        DocumentBondCapacityNotCheckedReasonV1::UnsupportedElement => "unsupported_element",
        DocumentBondCapacityNotCheckedReasonV1::UnsupportedBondEndpoint => {
            "unsupported_bond_endpoint"
        }
        DocumentBondCapacityNotCheckedReasonV1::UnsupportedBondOrder => "unsupported_bond_order",
        DocumentBondCapacityNotCheckedReasonV1::AromaticFact => "aromatic_fact",
    }
}

fn parse_selectors(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
) -> PyResult<Vec<ferrum_document::DocumentObjectIdV1>> {
    if !value.is_exact_instance_of::<PyTuple>() {
        return Err(error(py, SELECTOR_SHAPE_REASON));
    }
    let tuple = value.cast::<PyTuple>()?;
    if tuple.is_empty() {
        return Err(error(py, SELECTOR_SHAPE_REASON));
    }
    let mut selectors = Vec::new();
    selectors
        .try_reserve_exact(tuple.len())
        .map_err(|_| error(py, RESOURCE_REASON))?;
    for value in tuple.iter() {
        if !value.is_exact_instance_of::<PyString>() {
            return Err(error(py, SELECTOR_SHAPE_REASON));
        }
        let text = value
            .cast::<PyString>()?
            .to_str()
            .map_err(|_| error(py, "molecule selectors must be valid UTF-8 text"))?;
        let selector = crate::document_error_binding::document_object_id(py, text.to_owned())
            .map_err(|value| error(py, value.to_string()))?;
        selectors.push(selector);
    }
    Ok(selectors)
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(error(
            py,
            "expected digest must be exactly 64 lowercase hexadecimal characters",
        ));
    }
    let mut digest = [0; 32];
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

fn digest_text(digest: &[u8; 32]) -> String {
    let mut result = String::with_capacity(64);
    for byte in digest {
        result.push(hex_digit(byte >> 4));
        result.push(hex_digit(byte & 15));
    }
    result
}

const fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + value - 10) as char,
    }
}

fn error(py: Python<'_>, reason: impl Into<String>) -> PyErr {
    let reason = reason.into();
    let result = DocumentBondCapacityError::new_err(reason.clone());
    if let Err(value) = result.value(py).setattr("reason", reason) {
        return value;
    }
    result
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentBondCapacityError",
        module.py().get_type::<DocumentBondCapacityError>(),
    )?;
    module.add_class::<PyDocumentBondCapacityAtomV1>()?;
    module.add_class::<PyDocumentBondCapacityExplicitHydrogensFactV1>()?;
    module.add_class::<PyDocumentBondCapacityFormalChargeFactV1>()?;
    module.add_class::<PyDocumentBondCapacityRecordV1>()?;
    module.add_class::<PyDocumentBondCapacityV1>()?;
    module.add_function(wrap_pyfunction!(inspect_document_bond_capacity_v1, module)?)?;
    Ok(())
}
