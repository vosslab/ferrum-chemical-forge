//! Private native molecule-information operation for bundled Ferrum-Qt.
//!
//! This discoverable entry point deliberately remains absent from the wheel
//! stub, CLI, serde, and wire contracts.

use ferrum_api::{
    DocumentMoleculeInformationErrorV1, DocumentMoleculeInformationRequestV1,
    DocumentMoleculeInformationV1, execute_prepared_document_molecule_information_v1,
    prepare_document_molecule_information_v1,
};
use ferrum_chemistry::{
    ChemistryError as RustChemistryError, MoleculeComposition, NativeChemEngine,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString, PyTuple};

use crate::binding::FerrumError;
use crate::document_molecule_inspection_binding::{
    PyDocumentMoleculeInspectionV1, receipt_to_python,
};
use crate::projection_binding::PySessionDocumentObservationV1;

create_exception!(ferrum_chem, DocumentMoleculeInformationError, FerrumError);

const OPERATION: &str = "inspect_document_molecule_information_v1";
const DIGEST_REASON: &str = "expected digest must be exactly 64 lowercase hexadecimal characters";
const DIGEST_TEXT_REASON: &str = "expected digest must be valid UTF-8 text";
const SELECTOR_TEXT_REASON: &str = "molecule selectors must be valid UTF-8 text";
const SELECTOR_SHAPE_REASON: &str = "molecule selectors must be an exact nonempty tuple of strings";
const RESOURCE_REASON: &str = "molecule information could not reserve result storage";

/// One immutable isotope-aware count.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "MoleculeCompositionElementCountV1",
    skip_from_py_object
)]
struct PyMoleculeCompositionElementCountV1 {
    #[pyo3(get)]
    atomic_number: u8,
    #[pyo3(get)]
    isotope: Option<u16>,
    #[pyo3(get)]
    symbol: String,
    #[pyo3(get)]
    atom_count: u64,
}

/// One immutable average-mass contribution and percentage.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "MoleculeCompositionMassPercentageV1",
    skip_from_py_object
)]
struct PyMoleculeCompositionMassPercentageV1 {
    #[pyo3(get)]
    atomic_number: u8,
    #[pyo3(get)]
    isotope: Option<u16>,
    #[pyo3(get)]
    symbol: String,
    #[pyo3(get)]
    average_mass_contribution: f64,
    #[pyo3(get)]
    percentage: f64,
}

/// One immutable perceived composition receipt.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "MoleculeCompositionV1",
    skip_from_py_object
)]
struct PyMoleculeCompositionV1 {
    #[pyo3(get)]
    formula: String,
    #[pyo3(get)]
    net_formal_charge: i64,
    #[pyo3(get)]
    average_molecular_weight: f64,
    #[pyo3(get)]
    monoisotopic_mass: f64,
    #[pyo3(get)]
    element_counts: Py<PyTuple>,
    #[pyo3(get)]
    mass_percentages: Py<PyTuple>,
}

/// One immutable source-fact and composition pair.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculeInformationRecordV1",
    skip_from_py_object
)]
struct PyDocumentMoleculeInformationRecordV1 {
    #[pyo3(get)]
    source_facts: Py<PyDocumentMoleculeInspectionV1>,
    #[pyo3(get)]
    composition: Py<PyMoleculeCompositionV1>,
}

/// Complete exact-revision information for one or more durable roots.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentMoleculeInformationV1",
    skip_from_py_object
)]
struct PyDocumentMoleculeInformationV1 {
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    source_revision: u64,
    #[pyo3(get)]
    source_digest: String,
    #[pyo3(get)]
    records: Py<PyTuple>,
    #[pyo3(get)]
    combined_selection: Option<Py<PyMoleculeCompositionV1>>,
}

enum NativeInformationFailure {
    Load(RustChemistryError),
    Execute(DocumentMoleculeInformationErrorV1),
}

/// Calculate source facts and composition for exact selected durable roots.
///
/// Experimental internal-to-Ferrum-Qt operation. Rust authenticates and owns
/// every graph before the packaged adapter path is resolved or loaded.
#[pyfunction]
fn inspect_document_molecule_information_v1(
    py: Python<'_>,
    observation: PyRef<'_, PySessionDocumentObservationV1>,
    expected_revision: u64,
    expected_digest: &Bound<'_, PyString>,
    molecule_ids: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentMoleculeInformationV1> {
    let expected_digest = expected_digest
        .to_str()
        .map_err(|_| information_error(py, DIGEST_TEXT_REASON))?;
    let expected_digest = parse_digest(py, expected_digest)?;
    if !molecule_ids.is_exact_instance_of::<PyTuple>() {
        return Err(information_error(py, SELECTOR_SHAPE_REASON));
    }
    let molecule_ids = molecule_ids.cast::<PyTuple>()?;
    if molecule_ids.is_empty() {
        return Err(information_error(py, SELECTOR_SHAPE_REASON));
    }
    let mut selectors = Vec::new();
    selectors
        .try_reserve_exact(molecule_ids.len())
        .map_err(|_| information_error(py, RESOURCE_REASON))?;
    for item in molecule_ids.iter() {
        if !item.is_exact_instance_of::<PyString>() {
            return Err(information_error(py, SELECTOR_SHAPE_REASON));
        }
        let selector = item
            .cast::<PyString>()?
            .to_str()
            .map_err(|_| information_error(py, SELECTOR_TEXT_REASON))?;
        let selector = copied(py, selector)?;
        let selector = crate::document_error_binding::document_object_id(py, selector)
            .map_err(|error| information_error(py, error.to_string()))?;
        selectors.push(selector);
    }
    let request =
        DocumentMoleculeInformationRequestV1::new(expected_revision, expected_digest, selectors)
            .map_err(|error| information_error(py, error.to_string()))?;
    let prepared = prepare_document_molecule_information_v1(observation.observation(), &request)
        .map_err(|error| information_error(py, error.to_string()))?;
    let library_path = crate::chemistry_binding::packaged_library_path(py, OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine =
            NativeChemEngine::load(&worker_path).map_err(NativeInformationFailure::Load)?;
        execute_prepared_document_molecule_information_v1(&engine, prepared)
            .map_err(NativeInformationFailure::Execute)
    });
    let information = match result {
        Ok(information) => information,
        Err(NativeInformationFailure::Load(error)) => {
            return Err(crate::chemistry_binding::map_load_error(
                py,
                OPERATION,
                &library_path,
                error,
            )?);
        }
        Err(NativeInformationFailure::Execute(DocumentMoleculeInformationErrorV1::Chemistry(
            error,
        ))) => {
            return Err(crate::chemistry_binding::map_packaged_operation_error(
                py,
                OPERATION,
                &library_path,
                error,
            )?);
        }
        Err(NativeInformationFailure::Execute(error)) => {
            return Err(information_error(py, error.to_string()));
        }
    };
    information_to_python(py, &information)
}

fn information_to_python(
    py: Python<'_>,
    information: &DocumentMoleculeInformationV1,
) -> PyResult<PyDocumentMoleculeInformationV1> {
    let mut records = Vec::new();
    records
        .try_reserve_exact(information.records().len())
        .map_err(|_| information_error(py, RESOURCE_REASON))?;
    for record in information.records() {
        records.push(Py::new(
            py,
            PyDocumentMoleculeInformationRecordV1 {
                source_facts: Py::new(py, receipt_to_python(py, record.source_facts())?)?,
                composition: Py::new(py, composition_to_python(py, record.composition())?)?,
            },
        )?);
    }
    let combined_selection = information
        .combined_selection()
        .map(|composition| Py::new(py, composition_to_python(py, composition)?))
        .transpose()?;
    Ok(PyDocumentMoleculeInformationV1 {
        schema: copied(py, information.schema())?,
        source_revision: information.source_revision(),
        source_digest: hex_digest(py, information.source_digest())?,
        records: PyTuple::new(py, records)?.unbind(),
        combined_selection,
    })
}

fn composition_to_python(
    py: Python<'_>,
    composition: &MoleculeComposition,
) -> PyResult<PyMoleculeCompositionV1> {
    let mut counts = Vec::new();
    counts
        .try_reserve_exact(composition.element_counts().len())
        .map_err(|_| information_error(py, RESOURCE_REASON))?;
    for entry in composition.element_counts() {
        counts.push(PyMoleculeCompositionElementCountV1 {
            atomic_number: entry.key().atomic_number().get(),
            isotope: entry.key().isotope(),
            symbol: copied(py, entry.key().symbol())?,
            atom_count: entry.count(),
        });
    }
    let mut percentages = Vec::new();
    percentages
        .try_reserve_exact(composition.mass_percentages().len())
        .map_err(|_| information_error(py, RESOURCE_REASON))?;
    for entry in composition.mass_percentages() {
        percentages.push(PyMoleculeCompositionMassPercentageV1 {
            atomic_number: entry.key().atomic_number().get(),
            isotope: entry.key().isotope(),
            symbol: copied(py, entry.key().symbol())?,
            average_mass_contribution: entry.average_mass_contribution(),
            percentage: entry.percentage(),
        });
    }
    Ok(PyMoleculeCompositionV1 {
        formula: copied(py, composition.formula())?,
        net_formal_charge: composition.net_formal_charge(),
        average_molecular_weight: composition.average_molecular_weight(),
        monoisotopic_mass: composition.monoisotopic_mass(),
        element_counts: PyTuple::new(py, counts)?.unbind(),
        mass_percentages: PyTuple::new(py, percentages)?.unbind(),
    })
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(information_error(py, DIGEST_REASON));
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

fn hex_digest(py: Python<'_>, digest: &[u8; 32]) -> PyResult<String> {
    let mut value = String::new();
    value
        .try_reserve_exact(64)
        .map_err(|_| information_error(py, RESOURCE_REASON))?;
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
    result
        .try_reserve_exact(value.len())
        .map_err(|_| information_error(py, RESOURCE_REASON))?;
    result.push_str(value);
    Ok(result)
}

fn information_error(py: Python<'_>, reason: impl Into<String>) -> PyErr {
    let reason = reason.into();
    let error = DocumentMoleculeInformationError::new_err(reason.clone());
    if let Err(attribute_error) = error.value(py).setattr("reason", reason) {
        return attribute_error;
    }
    error
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentMoleculeInformationError",
        module.py().get_type::<DocumentMoleculeInformationError>(),
    )?;
    module.add_class::<PyMoleculeCompositionElementCountV1>()?;
    module.add_class::<PyMoleculeCompositionMassPercentageV1>()?;
    module.add_class::<PyMoleculeCompositionV1>()?;
    module.add_class::<PyDocumentMoleculeInformationRecordV1>()?;
    module.add_class::<PyDocumentMoleculeInformationV1>()?;
    module.add_function(wrap_pyfunction!(
        inspect_document_molecule_information_v1,
        module
    )?)?;
    Ok(())
}
