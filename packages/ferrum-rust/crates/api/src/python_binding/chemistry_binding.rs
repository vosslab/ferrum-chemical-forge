//! Direct, owned SMILES binding over Ferrum's ABI-4 chemistry engine.
//!
//! This module finds only the library shipped beside the extension, copies all
//! input and output at the call boundary, and never retains a native handle.

use std::path::PathBuf;

use ferrum_chemistry::{
    ChemistryError as RustChemistryError, ImportedSdfRecord, InchiMode, MolblockVersion,
    NativeChemEngine, NativeTextOutputLimit, SdfProperty, SdfRecord, SmilesMolecule,
    validate_inchi_input, validate_molblock_input, validate_sdf_input, validate_smiles_input,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::binding::FerrumError;

mod sdf_values;
mod value_conversion;

use sdf_values::{PyImportedSdfRecordV1, PySdfPropertyV1};
use value_conversion::{atom_chirality, bond_direction, bond_order, bond_stereo};

const PYTHON_CHEMISTRY_TEXT_LIMIT: NativeTextOutputLimit = NativeTextOutputLimit::ADAPTER_MAXIMUM;

create_exception!(ferrum_chem, ChemistryError, FerrumError);
create_exception!(ferrum_chem, InvalidSmiles, ChemistryError);
create_exception!(ferrum_chem, InvalidSdf, ChemistryError);
create_exception!(ferrum_chem, InvalidMolblock, ChemistryError);
create_exception!(ferrum_chem, InvalidInchi, ChemistryError);
create_exception!(ferrum_chem, ChemistryUnavailable, ChemistryError);
create_exception!(ferrum_chem, ChemistryParse, ChemistryError);
create_exception!(ferrum_chem, ChemistryCodec, ChemistryError);
create_exception!(ferrum_chem, ChemistryBoundary, ChemistryError);

/// Closed ABI-4 atom chirality vocabulary.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "SmilesAtomChiralityV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PySmilesAtomChiralityV1 {
    Unspecified,
    TetrahedralCw,
    TetrahedralCcw,
    Other,
}

/// Closed ABI-4 bond order vocabulary.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "SmilesBondOrderV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PySmilesBondOrderV1 {
    Aromatic,
    Single,
    Double,
    Triple,
    Quadruple,
}

/// Closed ABI-4 bond stereo vocabulary.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "SmilesBondStereoV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PySmilesBondStereoV1 {
    None,
    Any,
    Z,
    E,
    Cis,
    Trans,
    Other,
}

/// Closed ABI-4 bond drawing-direction vocabulary.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "SmilesBondDirectionV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PySmilesBondDirectionV1 {
    None,
    BeginWedge,
    BeginDash,
    EndUpRight,
    EndDownRight,
    Other,
}

/// Closed molfile syntax vocabulary for explicit native export.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "MolblockVersionV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyMolblockVersionV1 {
    V2000,
    V3000,
}

impl PyMolblockVersionV1 {
    pub(crate) const fn into_rust(self) -> MolblockVersion {
        match self {
            Self::V2000 => MolblockVersion::V2000,
            Self::V3000 => MolblockVersion::V3000,
        }
    }

    pub(crate) const fn from_rust(version: MolblockVersion) -> Self {
        match version {
            MolblockVersion::V2000 => Self::V2000,
            MolblockVersion::V3000 => Self::V3000,
        }
    }
}

/// Closed InChI serialization vocabulary.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "InchiModeV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyInchiModeV1 {
    Standard,
    FixedHydrogen,
}

impl PyInchiModeV1 {
    pub(crate) const fn into_rust(self) -> InchiMode {
        match self {
            Self::Standard => InchiMode::Standard,
            Self::FixedHydrogen => InchiMode::FixedHydrogen,
        }
    }
}

/// Immutable two-dimensional coordinate copied from an ABI-4 molecule.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "SmilesPoint2V1",
    skip_from_py_object
)]
#[derive(Clone)]
struct PySmilesPoint2V1 {
    #[pyo3(get)]
    x: f64,
    #[pyo3(get)]
    y: f64,
}

/// Immutable complete atom vocabulary copied from an ABI-4 molecule.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "SmilesAtomV1",
    skip_from_py_object
)]
struct PySmilesAtomV1 {
    #[pyo3(get)]
    atomic_number: u8,
    #[pyo3(get)]
    formal_charge: Option<i32>,
    #[pyo3(get)]
    isotope: Option<u16>,
    #[pyo3(get)]
    explicit_hydrogens: Option<u16>,
    #[pyo3(get)]
    aromatic: bool,
    #[pyo3(get)]
    chirality: Py<PySmilesAtomChiralityV1>,
    #[pyo3(get)]
    radical_electrons: u8,
    #[pyo3(get)]
    no_implicit: bool,
    #[pyo3(get)]
    atom_map_number: Option<u32>,
}

/// Immutable complete bond vocabulary copied from an ABI-4 molecule.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "SmilesBondV1",
    skip_from_py_object
)]
struct PySmilesBondV1 {
    #[pyo3(get)]
    start: usize,
    #[pyo3(get)]
    end: usize,
    #[pyo3(get)]
    order: Py<PySmilesBondOrderV1>,
    #[pyo3(get)]
    aromatic: bool,
    #[pyo3(get)]
    stereo: Py<PySmilesBondStereoV1>,
    #[pyo3(get)]
    direction: Py<PySmilesBondDirectionV1>,
    #[pyo3(get)]
    stereo_atoms: Option<(usize, usize)>,
}

/// Frozen, native-handle-free molecule result from [`parse_smiles`].
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "SmilesMoleculeV1",
    skip_from_py_object
)]
pub(crate) struct PySmilesMoleculeV1 {
    molecule: SmilesMolecule,
    #[pyo3(get)]
    canonical_smiles: String,
    #[pyo3(get)]
    atoms: Py<PyTuple>,
    #[pyo3(get)]
    bonds: Py<PyTuple>,
    #[pyo3(get)]
    coordinates: Py<PyTuple>,
}

/// Immutable coordinate-bearing molecule record prepared for SDF export.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "SdfRecordV1",
    skip_from_py_object
)]
struct PySdfRecordV1 {
    record: SdfRecord,
    #[pyo3(get)]
    molecule: Py<PySmilesMoleculeV1>,
    #[pyo3(get)]
    title: String,
    #[pyo3(get)]
    properties: Py<PyTuple>,
}

/// Parse SMILES through exactly the ABI-4 library shipped beside this extension.
///
/// The GIL remains held throughout the call. The adapter is not thread-safe,
/// and this direct boundary intentionally has no cache or alternative lookup.
#[pyfunction]
pub(crate) fn parse_smiles(py: Python<'_>, smiles: String) -> PyResult<PySmilesMoleculeV1> {
    if let Err(error) = validate_smiles_input(&smiles) {
        return Err(map_chemistry_error(py, error)?);
    }
    let (engine, library_path) = packaged_native_engine(py, "parse_smiles")?;
    let molecule = match engine.smiles_to_molecule(&smiles) {
        Ok(molecule) => molecule,
        Err(error) => {
            return Err(map_packaged_operation_error(
                py,
                "parse_smiles",
                &library_path,
                error,
            )?);
        }
    };
    molecule_to_python(py, molecule)
}

/// Import one bounded V2000 or V3000 molblock through the packaged adapter.
#[pyfunction]
fn molblock_to_molecule(py: Python<'_>, molblock: String) -> PyResult<PySmilesMoleculeV1> {
    if let Err(error) = validate_molblock_input(&molblock) {
        return Err(map_chemistry_error(py, error)?);
    }
    let (engine, library_path) = packaged_native_engine(py, "molblock_to_molecule")?;
    let molecule = match engine.molblock_to_molecule(&molblock) {
        Ok(molecule) => molecule,
        Err(error) => {
            return Err(map_packaged_operation_error(
                py,
                "molblock_to_molecule",
                &library_path,
                error,
            )?);
        }
    };
    molecule_to_python(py, molecule)
}

/// Import one bounded standard or non-standard InChI through the packaged adapter.
#[pyfunction]
fn parse_inchi(py: Python<'_>, inchi: String) -> PyResult<PySmilesMoleculeV1> {
    if let Err(error) = validate_inchi_input(&inchi) {
        return Err(map_chemistry_error(py, error)?);
    }
    let (engine, library_path) = packaged_native_engine(py, "parse_inchi")?;
    let molecule = match engine.inchi_to_molecule(&inchi) {
        Ok(molecule) => molecule,
        Err(error) => {
            return Err(map_packaged_operation_error(
                py,
                "parse_inchi",
                &library_path,
                error,
            )?);
        }
    };
    molecule_to_python(py, molecule)
}

/// Export one frozen native molecule as SMARTS through its packaged ABI-4 adapter.
#[pyfunction]
pub(crate) fn molecule_to_smarts(
    py: Python<'_>,
    molecule: PyRef<'_, PySmilesMoleculeV1>,
) -> PyResult<String> {
    let (engine, library_path) = packaged_native_engine(py, "molecule_to_smarts")?;
    match engine.molecule_to_smarts(molecule.molecule.molecule()) {
        Ok(smarts) => Ok(smarts),
        Err(error) => Err(map_packaged_operation_error(
            py,
            "molecule_to_smarts",
            &library_path,
            error,
        )?),
    }
}

/// Export one frozen native molecule with its exact coordinates as a molblock.
#[pyfunction]
fn molecule_to_molblock(
    py: Python<'_>,
    molecule: PyRef<'_, PySmilesMoleculeV1>,
    version: PyRef<'_, PyMolblockVersionV1>,
) -> PyResult<String> {
    let (engine, library_path) = packaged_native_engine(py, "molecule_to_molblock")?;
    let version = version.into_rust();
    match engine.molecule_to_molblock(
        molecule.molecule.molecule(),
        version,
        PYTHON_CHEMISTRY_TEXT_LIMIT,
    ) {
        Ok(molblock) => Ok(molblock),
        Err(error) => Err(map_packaged_operation_error(
            py,
            "molecule_to_molblock",
            &library_path,
            error,
        )?),
    }
}

/// Export one frozen molecule through an explicit closed InChI mode.
#[pyfunction]
fn molecule_to_inchi(
    py: Python<'_>,
    molecule: PyRef<'_, PySmilesMoleculeV1>,
    mode: PyRef<'_, PyInchiModeV1>,
) -> PyResult<String> {
    let (engine, library_path) = packaged_native_engine(py, "molecule_to_inchi")?;
    let mode = (*mode).into_rust();
    match engine.molecule_to_inchi(
        molecule.molecule.molecule(),
        mode,
        PYTHON_CHEMISTRY_TEXT_LIMIT,
    ) {
        Ok(inchi) => Ok(inchi),
        Err(error) => Err(map_packaged_operation_error(
            py,
            "molecule_to_inchi",
            &library_path,
            error,
        )?),
    }
}

/// Derive the official InChIKey for one bounded InChI line.
#[pyfunction]
fn inchi_to_inchi_key(py: Python<'_>, inchi: String) -> PyResult<String> {
    if let Err(error) = validate_inchi_input(&inchi) {
        return Err(map_chemistry_error(py, error)?);
    }
    let (engine, library_path) = packaged_native_engine(py, "inchi_to_inchi_key")?;
    match engine.inchi_to_inchi_key(&inchi) {
        Ok(key) => Ok(key),
        Err(error) => Err(map_packaged_operation_error(
            py,
            "inchi_to_inchi_key",
            &library_path,
            error,
        )?),
    }
}

/// Prepare one exact frozen SDF record without silently omitting properties.
#[pyfunction]
fn prepare_sdf_record(
    py: Python<'_>,
    molecule: PyRef<'_, PySmilesMoleculeV1>,
    title: String,
    properties: &Bound<'_, PyTuple>,
) -> PyResult<PySdfRecordV1> {
    let mut rust_properties = Vec::with_capacity(properties.len());
    let mut python_properties = Vec::with_capacity(properties.len());
    for item in properties.iter() {
        if !item.is_exact_instance_of::<PyTuple>() {
            return Err(ChemistryBoundary::new_err(
                "each SDF property must be an exact (name, value) tuple",
            ));
        }
        let (name, value) = item.extract::<(String, String)>()?;
        let property = SdfProperty::new(name.clone(), value.clone())
            .map_err(|error| ChemistryBoundary::new_err(error.to_string()))?;
        rust_properties.push(property);
        python_properties.push(Py::new(py, PySdfPropertyV1 { name, value })?);
    }
    let rust_molecule = molecule.molecule.clone();
    let record = SdfRecord::new(
        rust_molecule.molecule().clone(),
        title.clone(),
        rust_properties,
    )
    .map_err(|error| ChemistryBoundary::new_err(error.to_string()))?;
    let python_molecule = Py::new(py, molecule_to_python(py, rust_molecule)?)?;
    Ok(PySdfRecordV1 {
        record,
        molecule: python_molecule,
        title,
        properties: PyTuple::new(py, python_properties)?.unbind(),
    })
}

/// Export exact frozen records through the packaged native RDKit SD writer.
#[pyfunction]
fn records_to_sdf(
    py: Python<'_>,
    records: &Bound<'_, PyTuple>,
    version: PyRef<'_, PyMolblockVersionV1>,
) -> PyResult<String> {
    let mut rust_records = Vec::with_capacity(records.len());
    for item in records.iter() {
        if !item.is_exact_instance_of::<PySdfRecordV1>() {
            return Err(ChemistryBoundary::new_err(
                "SDF records must be exact ferrum_chem.SdfRecordV1 values",
            ));
        }
        rust_records.push(item.extract::<PyRef<'_, PySdfRecordV1>>()?.record.clone());
    }
    let version = version.into_rust();
    let (engine, library_path) = packaged_native_engine(py, "records_to_sdf")?;
    match engine.records_to_sdf(&rust_records, version, PYTHON_CHEMISTRY_TEXT_LIMIT) {
        Ok(sdf) => Ok(sdf),
        Err(error) => Err(map_packaged_operation_error(
            py,
            "records_to_sdf",
            &library_path,
            error,
        )?),
    }
}

/// Import bounded UTF-8 SDF through the packaged native RDKit supplier.
///
/// All molecule, title, and property values are copied before the foreign
/// response buffer is released. Repeated property names remain separate
/// ordered entries.
#[pyfunction]
fn sdf_to_records(py: Python<'_>, input: String) -> PyResult<Py<PyTuple>> {
    if let Err(error) = validate_sdf_input(&input) {
        return Err(map_chemistry_error(py, error)?);
    }
    let (engine, library_path) = packaged_native_engine(py, "sdf_to_records")?;
    let records = match engine.sdf_to_records(&input) {
        Ok(records) => records,
        Err(error) => {
            return Err(map_packaged_operation_error(
                py,
                "sdf_to_records",
                &library_path,
                error,
            )?);
        }
    };
    let records = records
        .into_iter()
        .map(|record| imported_sdf_record_to_python(py, record))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyTuple::new(py, records)?.unbind())
}

pub(crate) fn packaged_native_engine(
    py: Python<'_>,
    operation: &'static str,
) -> PyResult<(NativeChemEngine, PathBuf)> {
    let library_path = packaged_library_path(py, operation)?;
    match super::super::staged_extension_native_engine_v1() {
        Ok(engine) => Ok((engine, library_path)),
        Err(error) => Err(map_load_error(py, operation, &library_path, error)?),
    }
}

pub(crate) fn packaged_library_path(py: Python<'_>, operation: &'static str) -> PyResult<PathBuf> {
    match super::super::staged_extension_library_path_v1() {
        Some(library_path) => Ok(library_path),
        None => {
            let error = unavailable_error(
                py,
                operation,
                "Ferrum-Chem extension origin was not initialized".to_owned(),
            )?;
            Err(error)
        }
    }
}

fn molecule_to_python(py: Python<'_>, molecule: SmilesMolecule) -> PyResult<PySmilesMoleculeV1> {
    let graph = molecule.molecule();
    let atoms = graph
        .atoms()
        .iter()
        .map(|atom| {
            Ok(PySmilesAtomV1 {
                atomic_number: atom.atomic_number().get(),
                formal_charge: atom.formal_charge(),
                isotope: atom.isotope(),
                explicit_hydrogens: atom.explicit_hydrogens(),
                aromatic: atom.is_aromatic(),
                chirality: Py::new(py, atom_chirality(atom.chirality()))?,
                radical_electrons: atom.radical_electrons(),
                no_implicit: atom.no_implicit(),
                atom_map_number: atom.atom_map_number(),
            })
        })
        .collect::<PyResult<Vec<_>>>()?;
    let bonds = graph
        .bonds()
        .iter()
        .map(|bond| {
            Ok(PySmilesBondV1 {
                start: bond.start(),
                end: bond.end(),
                order: Py::new(py, bond_order(bond.order()))?,
                aromatic: bond.is_aromatic(),
                stereo: Py::new(py, bond_stereo(bond.stereo()))?,
                direction: Py::new(py, bond_direction(bond.direction()))?,
                stereo_atoms: bond.stereo_atoms(),
            })
        })
        .collect::<PyResult<Vec<_>>>()?;
    let coordinates = graph
        .coordinates()
        .ok_or_else(|| ChemistryBoundary::new_err("ABI-4 molecule omitted atom coordinates"))?
        .points()
        .iter()
        .map(|point| PySmilesPoint2V1 {
            x: point.x(),
            y: point.y(),
        })
        .collect::<Vec<_>>();
    let canonical_smiles = molecule.canonical_smiles().to_owned();
    Ok(PySmilesMoleculeV1 {
        canonical_smiles,
        atoms: PyTuple::new(py, atoms)?.unbind(),
        bonds: PyTuple::new(py, bonds)?.unbind(),
        coordinates: PyTuple::new(py, coordinates)?.unbind(),
        molecule,
    })
}

fn imported_sdf_record_to_python(
    py: Python<'_>,
    record: ImportedSdfRecord,
) -> PyResult<PyImportedSdfRecordV1> {
    let molecule = Py::new(py, molecule_to_python(py, record.molecule().clone())?)?;
    let properties = record
        .properties()
        .iter()
        .map(|property| {
            Py::new(
                py,
                PySdfPropertyV1 {
                    name: property.name().to_owned(),
                    value: property.value().to_owned(),
                },
            )
        })
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyImportedSdfRecordV1 {
        molecule,
        title: record.title().to_owned(),
        properties: PyTuple::new(py, properties)?.unbind(),
    })
}

pub(crate) fn map_load_error(
    py: Python<'_>,
    operation: &'static str,
    _library_path: &std::path::Path,
    _error: RustChemistryError,
) -> PyResult<PyErr> {
    // The installed-wheel adapter location and native loader diagnostics are
    // private capability details. Keep the original error inside the Rust
    // call chain, but never format it into the Python exception.
    unavailable_error(
        py,
        operation,
        "Ferrum chemistry runtime is unavailable".to_owned(),
    )
}

pub(crate) fn map_packaged_operation_error(
    py: Python<'_>,
    operation: &'static str,
    _library_path: &std::path::Path,
    error: RustChemistryError,
) -> PyResult<PyErr> {
    if is_packaged_runtime_failure(&error) {
        // The adapter, its ABI wire, and its diagnostics are private wheel
        // implementation details. A direct operation must publish the same
        // closed runtime refusal as an adapter-load failure.
        return unavailable_error(
            py,
            operation,
            "Ferrum chemistry runtime is unavailable".to_owned(),
        );
    }
    match error {
        RustChemistryError::OperationUnavailable { operation } => unavailable_error(
            py,
            operation,
            format!("chemistry operation is unavailable: {operation}"),
        ),
        error => {
            let mapped = map_chemistry_error(py, error)?;
            mapped.value(py).setattr("operation", operation)?;
            apply_packaged_error_metadata(
                py,
                &mapped,
                operation,
                "chemistry_operation_refused",
                "inspect_input_or_choose_a_supported_operation",
            )?;
            Ok(mapped)
        }
    }
}

pub(crate) fn map_chemistry_error(py: Python<'_>, error: RustChemistryError) -> PyResult<PyErr> {
    if is_packaged_runtime_failure(&error) {
        // Callers without a concrete packaged operation still cannot expose
        // adapter-originated detail through a generic chemistry error.
        return unavailable_error(
            py,
            "chemistry",
            "Ferrum chemistry runtime is unavailable".to_owned(),
        );
    }
    match error {
        RustChemistryError::InvalidSmilesInput { reason } => {
            structured_error(py, InvalidSmiles::new_err, reason, None, None)
        }
        RustChemistryError::InvalidSdfInput { reason } => {
            structured_error(py, InvalidSdf::new_err, reason, None, None)
        }
        RustChemistryError::InvalidMolblockInput { reason } => {
            structured_error(py, InvalidMolblock::new_err, reason, None, None)
        }
        RustChemistryError::InvalidInchiInput { reason } => {
            structured_error(py, InvalidInchi::new_err, reason, None, None)
        }
        RustChemistryError::OperationUnavailable { operation } => structured_error(
            py,
            ChemistryUnavailable::new_err,
            format!("chemistry operation is unavailable: {operation}"),
            Some(operation),
            None,
        ),
        RustChemistryError::ResourceExhausted { operation } => structured_error(
            py,
            ChemistryBoundary::new_err,
            format!("chemistry operation exhausted memory while producing {operation}"),
            Some(operation),
            None,
        ),
        RustChemistryError::NativeRejected { status, reason } => {
            structured_error(py, ChemistryParse::new_err, reason, None, Some(status))
        }
        RustChemistryError::CodecFailed { codec, reason } => {
            let error = ChemistryCodec::new_err(reason.clone());
            let value = error.value(py);
            value.setattr("reason", reason)?;
            value.setattr("codec", codec)?;
            Ok(error)
        }
        RustChemistryError::KekulizationFailed { reason }
        | RustChemistryError::CoordinateGenerationFailed { reason } => {
            structured_error(py, ChemistryBoundary::new_err, reason, None, None)
        }
        error => unreachable!("native boundary errors are closed before Python mapping: {error:?}"),
    }
}

fn is_packaged_runtime_failure(error: &RustChemistryError) -> bool {
    matches!(
        error,
        RustChemistryError::NativeBoundary { .. }
            | RustChemistryError::MalformedNativeResponse { .. }
            | RustChemistryError::TruncatedNativeResponse
            | RustChemistryError::TrailingNativeResponse
            | RustChemistryError::NativeRejected { .. }
            | RustChemistryError::UnsupportedNativeRequest { .. }
    )
}

fn unavailable_error(py: Python<'_>, operation: &str, reason: String) -> PyResult<PyErr> {
    let error = ChemistryUnavailable::new_err(reason.clone());
    let value = error.value(py);
    value.setattr("reason", reason)?;
    value.setattr("operation", operation)?;
    value.setattr("category", "chemistry_unavailable")?;
    value.setattr("recovery", "verify_ferrum_installation")?;
    Ok(error)
}

fn apply_packaged_error_metadata(
    py: Python<'_>,
    error: &PyErr,
    operation: &str,
    category: &str,
    recovery: &str,
) -> PyResult<()> {
    let value = error.value(py);
    value.setattr("operation", operation)?;
    value.setattr("category", category)?;
    value.setattr("recovery", recovery)
}

fn structured_error(
    py: Python<'_>,
    constructor: impl FnOnce(String) -> PyErr,
    reason: String,
    operation: Option<&str>,
    status: Option<u32>,
) -> PyResult<PyErr> {
    let error = constructor(reason.clone());
    let value = error.value(py);
    value.setattr("reason", reason)?;
    if let Some(operation) = operation {
        value.setattr("operation", operation)?;
    }
    if let Some(status) = status {
        value.setattr("status", status)?;
    }
    Ok(error)
}

/// Register the direct ABI-4 SMILES boundary.
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("ChemistryError", module.py().get_type::<ChemistryError>())?;
    module.add("InvalidSmiles", module.py().get_type::<InvalidSmiles>())?;
    module.add("InvalidSdf", module.py().get_type::<InvalidSdf>())?;
    module.add("InvalidMolblock", module.py().get_type::<InvalidMolblock>())?;
    module.add("InvalidInchi", module.py().get_type::<InvalidInchi>())?;
    module.add(
        "ChemistryUnavailable",
        module.py().get_type::<ChemistryUnavailable>(),
    )?;
    module.add("ChemistryParse", module.py().get_type::<ChemistryParse>())?;
    module.add("ChemistryCodec", module.py().get_type::<ChemistryCodec>())?;
    module.add(
        "ChemistryBoundary",
        module.py().get_type::<ChemistryBoundary>(),
    )?;
    module.add_class::<PySmilesPoint2V1>()?;
    module.add_class::<PySmilesAtomChiralityV1>()?;
    module.add_class::<PySmilesBondOrderV1>()?;
    module.add_class::<PySmilesBondStereoV1>()?;
    module.add_class::<PySmilesBondDirectionV1>()?;
    module.add_class::<PyMolblockVersionV1>()?;
    module.add_class::<PyInchiModeV1>()?;
    module.add_class::<PySmilesAtomV1>()?;
    module.add_class::<PySmilesBondV1>()?;
    module.add_class::<PySmilesMoleculeV1>()?;
    module.add_class::<PySdfPropertyV1>()?;
    module.add_class::<PySdfRecordV1>()?;
    module.add_class::<PyImportedSdfRecordV1>()?;
    module.add_function(wrap_pyfunction!(parse_smiles, module)?)?;
    module.add_function(wrap_pyfunction!(molblock_to_molecule, module)?)?;
    module.add_function(wrap_pyfunction!(parse_inchi, module)?)?;
    module.add_function(wrap_pyfunction!(molecule_to_smarts, module)?)?;
    module.add_function(wrap_pyfunction!(molecule_to_molblock, module)?)?;
    module.add_function(wrap_pyfunction!(molecule_to_inchi, module)?)?;
    module.add_function(wrap_pyfunction!(inchi_to_inchi_key, module)?)?;
    module.add_function(wrap_pyfunction!(prepare_sdf_record, module)?)?;
    module.add_function(wrap_pyfunction!(records_to_sdf, module)?)?;
    module.add_function(wrap_pyfunction!(sdf_to_records, module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn loader_failures_are_redacted_at_the_python_boundary() {
        Python::initialize();
        Python::attach(|py| {
            let hostile_detail =
                "/private/ferrum/.dylibs/libferrum_chem.dylib: native loader detail";
            let error = map_load_error(
                py,
                "parse_smiles",
                Path::new(hostile_detail),
                RustChemistryError::NativeBoundary {
                    reason: hostile_detail.to_owned(),
                },
            )
            .expect("loader error maps to a public exception");
            let value = error.value(py);
            let reason: String = value
                .getattr("reason")
                .expect("reason is public")
                .extract()
                .expect("reason is text");
            let category: String = value
                .getattr("category")
                .expect("category is public")
                .extract()
                .expect("category is text");
            let recovery: String = value
                .getattr("recovery")
                .expect("recovery is public")
                .extract()
                .expect("recovery is text");

            assert_eq!(reason, "Ferrum chemistry runtime is unavailable");
            assert_eq!(category, "chemistry_unavailable");
            assert_eq!(recovery, "verify_ferrum_installation");
            for public_text in [error.to_string(), reason] {
                assert!(!public_text.contains(hostile_detail));
                assert!(!public_text.contains(".dylibs"));
                assert!(!public_text.contains("libferrum_chem"));
            }
        });
    }

    #[test]
    fn packaged_native_boundary_failures_are_redacted_at_the_python_boundary() {
        Python::initialize();
        Python::attach(|py| {
            let hostile_detail =
                "/private/ferrum/.dylibs/libferrum_chem.dylib: adapter dlopen failed";
            let error = map_packaged_operation_error(
                py,
                "molecule_to_inchi",
                Path::new(hostile_detail),
                RustChemistryError::NativeBoundary {
                    reason: hostile_detail.to_owned(),
                },
            )
            .expect("packaged operation error maps to a public exception");
            let value = error.value(py);
            let reason: String = value
                .getattr("reason")
                .expect("reason is public")
                .extract()
                .expect("reason is text");
            let category: String = value
                .getattr("category")
                .expect("category is public")
                .extract()
                .expect("category is text");
            let recovery: String = value
                .getattr("recovery")
                .expect("recovery is public")
                .extract()
                .expect("recovery is text");
            let operation: String = value
                .getattr("operation")
                .expect("operation is public")
                .extract()
                .expect("operation is text");

            assert!(value.is_instance_of::<ChemistryUnavailable>());
            assert_eq!(reason, "Ferrum chemistry runtime is unavailable");
            assert_eq!(category, "chemistry_unavailable");
            assert_eq!(recovery, "verify_ferrum_installation");
            assert_eq!(operation, "molecule_to_inchi");
            for public_text in [error.to_string(), reason, category, recovery, operation] {
                for sensitive in [
                    hostile_detail,
                    ".dylibs",
                    "libferrum_chem",
                    "adapter",
                    "dlopen",
                ] {
                    assert!(!public_text.contains(sensitive));
                }
            }
        });
    }

    #[test]
    fn direct_chemistry_mapping_closes_every_native_adapter_failure_kind() {
        Python::initialize();
        Python::attach(|py| {
            let hostile_detail =
                "/private/ferrum/.dylibs/libferrum_chem.dylib: adapter dlopen failed";
            let failures = [
                RustChemistryError::NativeBoundary {
                    reason: hostile_detail.to_owned(),
                },
                RustChemistryError::MalformedNativeResponse {
                    reason: hostile_detail.to_owned(),
                },
                RustChemistryError::TruncatedNativeResponse,
                RustChemistryError::TrailingNativeResponse,
                RustChemistryError::NativeRejected {
                    status: 17,
                    reason: hostile_detail.to_owned(),
                },
                RustChemistryError::UnsupportedNativeRequest {
                    reason: hostile_detail.to_owned(),
                },
            ];

            for failure in failures {
                let error = map_chemistry_error(py, failure)
                    .expect("native adapter error maps to a closed public exception");
                let value = error.value(py);
                let reason: String = value
                    .getattr("reason")
                    .expect("reason is public")
                    .extract()
                    .expect("reason is text");
                let category: String = value
                    .getattr("category")
                    .expect("category is public")
                    .extract()
                    .expect("category is text");
                let recovery: String = value
                    .getattr("recovery")
                    .expect("recovery is public")
                    .extract()
                    .expect("recovery is text");

                assert!(value.is_instance_of::<ChemistryUnavailable>());
                assert_eq!(reason, "Ferrum chemistry runtime is unavailable");
                assert_eq!(category, "chemistry_unavailable");
                assert_eq!(recovery, "verify_ferrum_installation");
                for public_text in [error.to_string(), reason, category, recovery] {
                    for sensitive in [
                        hostile_detail,
                        ".dylibs",
                        "libferrum_chem",
                        "adapter",
                        "dlopen",
                    ] {
                        assert!(!public_text.contains(sensitive));
                    }
                }
            }
        });
    }
}
