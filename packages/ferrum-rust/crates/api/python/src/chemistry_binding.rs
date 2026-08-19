//! Direct, owned SMILES binding over Ferrum's ABI-4 chemistry engine.
//!
//! This module finds only the library shipped beside the extension, copies all
//! input and output at the call boundary, and never retains a native handle.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ferrum_api::TrustedLibraryChemistryRuntimeV1;
use ferrum_chemistry::{
    AtomChirality, BondDirection, BondOrder, BondStereo, ChemistryError as RustChemistryError,
    ImportedSdfRecord, InchiMode, MolblockVersion, NativeChemEngine, SdfProperty, SdfRecord,
    SmilesMolecule, validate_inchi_input, validate_molblock_input, validate_sdf_input,
    validate_smiles_input,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::binding::FerrumError;

create_exception!(ferrum_chem, ChemistryError, FerrumError);
create_exception!(ferrum_chem, InvalidSmiles, ChemistryError);
create_exception!(ferrum_chem, InvalidSdf, ChemistryError);
create_exception!(ferrum_chem, InvalidMolblock, ChemistryError);
create_exception!(ferrum_chem, InvalidInchi, ChemistryError);
create_exception!(ferrum_chem, ChemistryUnavailable, ChemistryError);
create_exception!(ferrum_chem, ChemistryParse, ChemistryError);
create_exception!(ferrum_chem, ChemistryCodec, ChemistryError);
create_exception!(ferrum_chem, ChemistryBoundary, ChemistryError);

static EXTENSION_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();

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

    pub(crate) const fn from_rust(value: InchiMode) -> Self {
        match value {
            InchiMode::Standard => Self::Standard,
            InchiMode::FixedHydrogen => Self::FixedHydrogen,
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

/// Immutable ordered SDF property prepared for native export.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "SdfPropertyV1",
    skip_from_py_object
)]
#[derive(Clone)]
struct PySdfPropertyV1 {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    value: String,
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

/// Immutable ordered record copied from native SDF input.
///
/// This is distinct from [`PySdfRecordV1`] because authored SDF input may
/// contain repeated property names, which import preserves in encounter order.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "ImportedSdfRecordV1",
    skip_from_py_object
)]
struct PyImportedSdfRecordV1 {
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
    match engine.molecule_to_molblock(molecule.molecule.molecule(), version) {
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
    match engine.molecule_to_inchi(molecule.molecule.molecule(), mode) {
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
    match engine.records_to_sdf(&rust_records, version) {
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
    match NativeChemEngine::load(&library_path) {
        Ok(engine) => Ok((engine, library_path)),
        Err(error) => Err(map_load_error(py, operation, &library_path, error)?),
    }
}

pub(crate) fn packaged_library_path(py: Python<'_>, operation: &'static str) -> PyResult<PathBuf> {
    match EXTENSION_DIRECTORY.get() {
        Some(directory) => Ok(directory.join(".dylibs").join("libferrum_chem.dylib")),
        None => {
            let error = unavailable_error(
                py,
                operation,
                PathBuf::new(),
                "Ferrum-Chem extension origin was not initialized".to_owned(),
            )?;
            Err(error)
        }
    }
}

/// Construct the protocol's trusted wheel runtime without exposing its path.
///
/// The protocol executor owns the loaded native engine for one operation; this
/// binding retains only the wheel-local location established at module import.
pub(crate) fn packaged_protocol_runtime() -> TrustedLibraryChemistryRuntimeV1 {
    let library_path = EXTENSION_DIRECTORY
        .get()
        .map(|directory| directory.join(".dylibs").join("libferrum_chem.dylib"))
        .unwrap_or_default();
    TrustedLibraryChemistryRuntimeV1::from_trusted_library(library_path)
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

fn atom_chirality(value: AtomChirality) -> PySmilesAtomChiralityV1 {
    match value {
        AtomChirality::Unspecified => PySmilesAtomChiralityV1::Unspecified,
        AtomChirality::TetrahedralCw => PySmilesAtomChiralityV1::TetrahedralCw,
        AtomChirality::TetrahedralCcw => PySmilesAtomChiralityV1::TetrahedralCcw,
        AtomChirality::Other => PySmilesAtomChiralityV1::Other,
    }
}

fn bond_order(value: BondOrder) -> PySmilesBondOrderV1 {
    match value {
        BondOrder::Aromatic => PySmilesBondOrderV1::Aromatic,
        BondOrder::Single => PySmilesBondOrderV1::Single,
        BondOrder::Double => PySmilesBondOrderV1::Double,
        BondOrder::Triple => PySmilesBondOrderV1::Triple,
        BondOrder::Quadruple => PySmilesBondOrderV1::Quadruple,
    }
}

fn bond_stereo(value: BondStereo) -> PySmilesBondStereoV1 {
    match value {
        BondStereo::None => PySmilesBondStereoV1::None,
        BondStereo::Any => PySmilesBondStereoV1::Any,
        BondStereo::Z => PySmilesBondStereoV1::Z,
        BondStereo::E => PySmilesBondStereoV1::E,
        BondStereo::Cis => PySmilesBondStereoV1::Cis,
        BondStereo::Trans => PySmilesBondStereoV1::Trans,
        BondStereo::Other => PySmilesBondStereoV1::Other,
    }
}

fn bond_direction(value: BondDirection) -> PySmilesBondDirectionV1 {
    match value {
        BondDirection::None => PySmilesBondDirectionV1::None,
        BondDirection::BeginWedge => PySmilesBondDirectionV1::BeginWedge,
        BondDirection::BeginDash => PySmilesBondDirectionV1::BeginDash,
        BondDirection::EndUpRight => PySmilesBondDirectionV1::EndUpRight,
        BondDirection::EndDownRight => PySmilesBondDirectionV1::EndDownRight,
        BondDirection::Other => PySmilesBondDirectionV1::Other,
    }
}

pub(crate) fn map_load_error(
    py: Python<'_>,
    operation: &'static str,
    library_path: &std::path::Path,
    error: RustChemistryError,
) -> PyResult<PyErr> {
    let mapped = map_chemistry_error(py, error)?;
    if mapped.is_instance_of::<ChemistryUnavailable>(py) {
        return Ok(mapped);
    }
    unavailable_error(
        py,
        operation,
        library_path.to_path_buf(),
        mapped.to_string(),
    )
}

pub(crate) fn map_packaged_operation_error(
    py: Python<'_>,
    operation: &'static str,
    library_path: &std::path::Path,
    error: RustChemistryError,
) -> PyResult<PyErr> {
    match error {
        RustChemistryError::OperationUnavailable { operation } => unavailable_error(
            py,
            operation,
            library_path.to_path_buf(),
            format!("chemistry operation is unavailable: {operation}"),
        ),
        error => {
            let mapped = map_chemistry_error(py, error)?;
            mapped.value(py).setattr("operation", operation)?;
            mapped
                .value(py)
                .setattr("library_path", library_path.display().to_string())?;
            Ok(mapped)
        }
    }
}

pub(crate) fn map_chemistry_error(py: Python<'_>, error: RustChemistryError) -> PyResult<PyErr> {
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
        | RustChemistryError::CoordinateGenerationFailed { reason }
        | RustChemistryError::NativeBoundary { reason }
        | RustChemistryError::MalformedNativeResponse { reason }
        | RustChemistryError::UnsupportedNativeRequest { reason } => {
            structured_error(py, ChemistryBoundary::new_err, reason, None, None)
        }
        RustChemistryError::TruncatedNativeResponse => structured_error(
            py,
            ChemistryBoundary::new_err,
            "Ferrum chemistry adapter returned a truncated response".to_owned(),
            None,
            None,
        ),
        RustChemistryError::TrailingNativeResponse => structured_error(
            py,
            ChemistryBoundary::new_err,
            "Ferrum chemistry adapter returned trailing response bytes".to_owned(),
            None,
            None,
        ),
    }
}

fn unavailable_error(
    py: Python<'_>,
    operation: &str,
    path: PathBuf,
    reason: String,
) -> PyResult<PyErr> {
    let error = ChemistryUnavailable::new_err(reason);
    let value = error.value(py);
    value.setattr("reason", error.to_string())?;
    value.setattr("operation", operation)?;
    value.setattr("library_path", path.display().to_string())?;
    Ok(error)
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
    initialize_packaged_library_path(module)?;
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

fn initialize_packaged_library_path(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let extension_file = module.getattr("__file__")?.extract::<String>()?;
    let extension_path = PathBuf::from(extension_file);
    if !is_direct_extension_path(&extension_path) {
        return Err(ChemistryUnavailable::new_err(
            "Ferrum-Chem module origin is not a regular extension file",
        ));
    }
    let parent = extension_path.parent().ok_or_else(|| {
        ChemistryUnavailable::new_err("Ferrum-Chem extension has no package directory")
    })?;
    // Maturin may install this extension either at wheel root or as the
    // private implementation of the public ``ferrum_chem`` package.  The
    // sealed closure is always adjacent to that public package root.
    let closure_directory = if parent.file_name().is_some_and(|name| name == "ferrum_chem") {
        parent.parent().ok_or_else(|| {
            ChemistryUnavailable::new_err("Ferrum-Chem package extension has no wheel root")
        })?
    } else {
        parent
    };
    if EXTENSION_DIRECTORY
        .set(closure_directory.to_path_buf())
        .is_err()
    {
        return Err(ChemistryUnavailable::new_err(
            "Ferrum-Chem extension origin was initialized more than once",
        ));
    }
    Ok(())
}

fn is_direct_extension_path(path: &Path) -> bool {
    path.is_file()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("ferrum_chem") && name.ends_with(".so"))
}
