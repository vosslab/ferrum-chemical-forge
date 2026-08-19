//! Native adapter implementation of the safe chemistry engine.
//!
//! The byte protocol is deliberately private to this module.  All callers see
//! only owned [`MolGraph`] values and typed [`ChemistryError`] variants.

mod codec_support;
mod composition_wire;
mod fcm1;
mod graph_wire;
mod inchi_wire;
mod molblock_import;
mod molblock_wire;
mod sdf_import;
mod sdf_wire;
mod text_response;

#[cfg(test)]
mod smiles_write_tests;

use codec_support::{Reader, put_i32, put_u16, put_u32};

use std::path::Path;

use ferrum_chemistry_sys::{AdapterError, ChemistryAdapter, FERRUM_CHEM_CALL_ALLOCATION_FAILURE};

use crate::{
    AtomChirality, AtomicNumber, BondDirection, BondOrder, BondStereo, ChemEngine, ChemistryError,
    Coordinates, FERRUM_CHEM_COMPOSITION_ENTRY_BYTES, FERRUM_CHEM_COMPOSITION_FLAGS_NONE,
    FERRUM_CHEM_COMPOSITION_MAX_DETAIL_BYTES, FERRUM_CHEM_COMPOSITION_MAX_FORMULA_BYTES,
    FERRUM_CHEM_COMPOSITION_RESPONSE_HEADER_BYTES, FERRUM_CHEM_COMPOSITION_WIRE_VERSION,
    FERRUM_CHEM_COORDINATE_BYTES, FERRUM_CHEM_GRAPH_ATOM_BYTES, FERRUM_CHEM_GRAPH_BOND_BYTES,
    FERRUM_CHEM_GRAPH_FLAGS_NONE, FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES,
    FERRUM_CHEM_GRAPH_WIRE_VERSION, FERRUM_CHEM_INCHI_MAX_BYTES, FERRUM_CHEM_KEKULIZE_ATOM_BYTES,
    FERRUM_CHEM_KEKULIZE_BOND_BYTES, FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC,
    FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE, FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE,
    FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE, FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE,
    FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED, FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS,
    FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE, FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE,
    FERRUM_CHEM_KEKULIZE_MAX_ATOMS, FERRUM_CHEM_KEKULIZE_MAX_BACKTRACKS,
    FERRUM_CHEM_KEKULIZE_MAX_BONDS, FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES,
    FERRUM_CHEM_KEKULIZE_OPTION_CANONICAL, FERRUM_CHEM_KEKULIZE_OPTION_CLEAR_AROMATIC_FLAGS,
    FERRUM_CHEM_KEKULIZE_REQUEST_HEADER_BYTES, FERRUM_CHEM_KEKULIZE_RESPONSE_HEADER_BYTES,
    FERRUM_CHEM_KEKULIZE_WIRE_VERSION, FERRUM_CHEM_MAX_RESPONSE_BYTES,
    FERRUM_CHEM_MOLBLOCK_FLAGS_NONE, FERRUM_CHEM_MOLBLOCK_FORMAT_V2000,
    FERRUM_CHEM_MOLBLOCK_FORMAT_V3000, FERRUM_CHEM_MOLBLOCK_REQUEST_HEADER_BYTES,
    FERRUM_CHEM_MOLBLOCK_WIRE_VERSION, FERRUM_CHEM_MOLECULE_ATOM_BYTES,
    FERRUM_CHEM_MOLECULE_BOND_BYTES, FERRUM_CHEM_MOLECULE_RESPONSE_HEADER_BYTES,
    FERRUM_CHEM_RESULT_DEPICTION_FAILURE, FERRUM_CHEM_RESULT_INTERNAL_FAILURE,
    FERRUM_CHEM_RESULT_INVALID_MOLECULE, FERRUM_CHEM_RESULT_MALFORMED_REQUEST,
    FERRUM_CHEM_RESULT_OK, FERRUM_CHEM_RESULT_RESOURCE_LIMIT,
    FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE, FERRUM_CHEM_SDF_FLAGS_NONE,
    FERRUM_CHEM_SDF_MAX_PROPERTIES, FERRUM_CHEM_SDF_MAX_RECORDS,
    FERRUM_CHEM_SDF_PROPERTY_HEADER_BYTES, FERRUM_CHEM_SDF_RECORD_HEADER_BYTES,
    FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES, FERRUM_CHEM_SDF_RESPONSE_HEADER_BYTES,
    FERRUM_CHEM_SDF_WIRE_VERSION, FERRUM_CHEM_SMILES_MAX_BYTES, FERRUM_CHEM_SMILES_WRITE_MAX_BYTES,
    FERRUM_CHEM_TEXT_FLAGS_NONE, FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES,
    FERRUM_CHEM_TEXT_WIRE_VERSION, FERRUM_CHEM_TITLED_MOLBLOCK_REQUEST_HEADER_BYTES,
    FERRUM_CHEM_TITLED_MOLBLOCK_WIRE_VERSION, ImportedSdfRecord, InchiMode, KekulizeOptions,
    MolAtom, MolBond, MolGraph, MolblockVersion, MoleculeComposition, Point2, SdfProperty,
    SdfRecord, SmilesMolecule,
};

const REQUEST_MAGIC: [u8; 4] = *b"FCK1";

/// Maximum UTF-8 SMILES bytes accepted by the loaded Ferrum-Chem adapter.
///
/// Higher-level ingress policies may derive smaller, grammar-specific budgets
/// from this adapter boundary.
pub const NATIVE_SMILES_MAX_INPUT_BYTES: usize = FERRUM_CHEM_SMILES_MAX_BYTES;
/// Maximum printable ASCII bytes returned by canonical native SMILES export.
pub const NATIVE_SMILES_MAX_OUTPUT_BYTES: usize = FERRUM_CHEM_SMILES_WRITE_MAX_BYTES;
const RESPONSE_MAGIC: [u8; 4] = *b"FCR1";
const COORDINATE_RESPONSE_MAGIC: [u8; 4] = *b"FCL1";
const COORDINATE_RESPONSE_HEADER_LENGTH: usize = 20;
const COORDINATE_BYTES: usize = FERRUM_CHEM_COORDINATE_BYTES;
const MOLECULE_RESPONSE_MAGIC: [u8; 4] = *b"FCM1";
const MOLECULE_RESPONSE_HEADER_LENGTH: usize = FERRUM_CHEM_MOLECULE_RESPONSE_HEADER_BYTES;
const REQUEST_HEADER_LENGTH: usize = FERRUM_CHEM_KEKULIZE_REQUEST_HEADER_BYTES;
const RESPONSE_HEADER_LENGTH: usize = FERRUM_CHEM_KEKULIZE_RESPONSE_HEADER_BYTES;
const ATOM_LENGTH: usize = FERRUM_CHEM_KEKULIZE_ATOM_BYTES;
const BOND_LENGTH: usize = FERRUM_CHEM_KEKULIZE_BOND_BYTES;
const OPTION_MASK: u32 =
    FERRUM_CHEM_KEKULIZE_OPTION_CLEAR_AROMATIC_FLAGS | FERRUM_CHEM_KEKULIZE_OPTION_CANONICAL;
const FACT_MASK: u32 = FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE
    | FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE
    | FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS;

/// Fixed wire policy for deterministic 2D depictions.
///
/// The graph envelope is shared with Kekulize, but this policy never exposes
/// or accepts Kekulize's bond-form or search-budget controls.
#[derive(Clone, Copy)]
struct DepictionRequestOptions;

impl DepictionRequestOptions {
    const fn option_bits(self) -> u32 {
        0
    }

    const fn parser_backtrack_sentinel(self) -> u32 {
        1
    }
}

/// A safe, dynamically loaded native chemistry engine.
///
/// It owns the adapter library and is intentionally neither `Send` nor `Sync`;
/// the native ABI has not promised concurrent access. The adapter path is
/// explicit and its ABI comes from the public C header during this crate's
/// build, rather than from a second Rust constant.
pub struct NativeChemEngine {
    adapter: ChemistryAdapter,
}

impl NativeChemEngine {
    /// Load the native adapter at an explicit filesystem path.
    pub fn load(library_path: &Path) -> Result<Self, ChemistryError> {
        ChemistryAdapter::load(library_path, crate::ADAPTER_ABI_VERSION)
            .map(|adapter| Self { adapter })
            .map_err(adapter_error)
    }

    /// Parse SMILES into a complete, atom-order-preserving native molecule.
    pub fn smiles_to_molecule(&self, smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
        <Self as ChemEngine>::smiles_to_molecule(self, smiles)
    }

    /// Export a complete graph as canonical SMARTS through the loaded adapter build.
    pub fn molecule_to_smarts(&self, molecule: &MolGraph) -> Result<String, ChemistryError> {
        <Self as ChemEngine>::molecule_to_smarts(self, molecule)
    }

    /// Export a complete graph as canonical isomeric SMILES.
    pub fn molecule_to_smiles(&self, molecule: &MolGraph) -> Result<String, ChemistryError> {
        <Self as ChemEngine>::molecule_to_smiles(self, molecule)
    }

    /// Calculate isotope-aware formula, counts, charge, and masses.
    pub fn molecule_composition(
        &self,
        molecule: &MolGraph,
    ) -> Result<MoleculeComposition, ChemistryError> {
        <Self as ChemEngine>::molecule_composition(self, molecule)
    }

    /// Export a complete coordinate-bearing graph as explicit molblock syntax.
    pub fn molecule_to_molblock(
        &self,
        molecule: &MolGraph,
        version: MolblockVersion,
    ) -> Result<String, ChemistryError> {
        <Self as ChemEngine>::molecule_to_molblock(self, molecule, version)
    }

    /// Export a coordinate-bearing graph with an exact first-line title.
    pub fn molecule_to_molblock_with_title(
        &self,
        molecule: &MolGraph,
        version: MolblockVersion,
        title: &str,
    ) -> Result<String, ChemistryError> {
        <Self as ChemEngine>::molecule_to_molblock_with_title(self, molecule, version, title)
    }

    /// Import one bounded V2000 or V3000 molblock.
    pub fn molblock_to_molecule(&self, molblock: &str) -> Result<SmilesMolecule, ChemistryError> {
        <Self as ChemEngine>::molblock_to_molecule(self, molblock)
    }

    /// Import one standard or non-standard InChI into an owned 2D molecule.
    pub fn inchi_to_molecule(&self, inchi: &str) -> Result<SmilesMolecule, ChemistryError> {
        <Self as ChemEngine>::inchi_to_molecule(self, inchi)
    }

    /// Export one complete graph through the selected closed InChI mode.
    pub fn molecule_to_inchi(
        &self,
        molecule: &MolGraph,
        mode: InchiMode,
    ) -> Result<String, ChemistryError> {
        <Self as ChemEngine>::molecule_to_inchi(self, molecule, mode)
    }

    /// Derive the official InChIKey for one bounded InChI line.
    pub fn inchi_to_inchi_key(&self, inchi: &str) -> Result<String, ChemistryError> {
        <Self as ChemEngine>::inchi_to_inchi_key(self, inchi)
    }

    /// Export ordered records through the native RDKit SD writer.
    pub fn records_to_sdf(
        &self,
        records: &[SdfRecord],
        version: MolblockVersion,
    ) -> Result<String, ChemistryError> {
        <Self as ChemEngine>::records_to_sdf(self, records, version)
    }

    /// Import bounded UTF-8 SDF text into owned ordered records.
    pub fn sdf_to_records(&self, input: &str) -> Result<Vec<ImportedSdfRecord>, ChemistryError> {
        validate_sdf_input(input)?;
        let response = self
            .adapter
            .sdf_to_records(input.as_bytes())
            .map_err(adapter_error)?;
        sdf_import::decode(&response)
    }
}

/// Validate a SMILES request before it reaches the native adapter boundary.
///
/// This keeps the ABI-4 nonempty, NUL, and byte-length contract available to
/// callers that must reject malformed text before loading an adapter.
pub fn validate_smiles_input(smiles: &str) -> Result<(), ChemistryError> {
    fcm1::validate_input(smiles)
}

/// Maximum UTF-8 SDF input accepted by the ABI-4 import operation.
pub const SDF_MAX_INPUT_BYTES: usize = FERRUM_CHEM_MAX_RESPONSE_BYTES;

/// Maximum UTF-8 molblock input accepted by the ABI-4 import operation.
pub const MOLBLOCK_MAX_INPUT_BYTES: usize = FERRUM_CHEM_MAX_RESPONSE_BYTES;

/// Maximum ASCII InChI input accepted by the ABI-4 operation.
pub const INCHI_MAX_INPUT_BYTES: usize = FERRUM_CHEM_INCHI_MAX_BYTES;

/// Validate SDF text before it reaches the native adapter boundary.
///
/// This keeps the ABI-4 UTF-8, nonempty, NUL, and byte-length contract
/// available to callers that must reject malformed text before loading an
/// adapter.
pub fn validate_sdf_input(input: &str) -> Result<(), ChemistryError> {
    sdf_import::validate_input(input)
}

/// Validate one V2000 or V3000 molblock before loading a native adapter.
pub fn validate_molblock_input(input: &str) -> Result<(), ChemistryError> {
    molblock_import::validate_input(input)
}

/// Validate an exact first-line Molfile title before loading a native adapter.
pub fn validate_molblock_title(title: &str) -> Result<(), ChemistryError> {
    molblock_wire::validate_title(title)
}

/// Validate one InChI line before loading or calling a native adapter.
pub fn validate_inchi_input(input: &str) -> Result<(), ChemistryError> {
    inchi_wire::validate_input(input)
}

impl ChemEngine for NativeChemEngine {
    fn smiles_to_molecule(&self, smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
        validate_smiles_input(smiles)?;
        let response = self
            .adapter
            .smiles_to_molecule(smiles.as_bytes())
            .map_err(adapter_error)?;
        fcm1::decode(&response)
    }

    fn generate_2d_coordinates(&self, molecule: &MolGraph) -> Result<Coordinates, ChemistryError> {
        let request = encode_depiction_request(molecule)?;
        let response = self.adapter.generate_2d(&request).map_err(adapter_error)?;
        decode_coordinate_response(&response, molecule.atoms().len())
    }

    fn molecule_to_smarts(&self, molecule: &MolGraph) -> Result<String, ChemistryError> {
        let request = graph_wire::encode(molecule)?;
        let response = self
            .adapter
            .molecule_to_smarts(&request)
            .map_err(adapter_error)?;
        text_response::decode(&response, "SMARTS")
    }

    fn molecule_to_smiles(&self, molecule: &MolGraph) -> Result<String, ChemistryError> {
        let request = graph_wire::encode(molecule)?;
        let response = self
            .adapter
            .molecule_to_smiles(&request)
            .map_err(adapter_error)?;
        text_response::decode_smiles(&response)
    }

    fn molecule_composition(
        &self,
        molecule: &MolGraph,
    ) -> Result<MoleculeComposition, ChemistryError> {
        let request = graph_wire::encode(molecule)?;
        let response = self
            .adapter
            .molecule_composition(&request)
            .map_err(adapter_error)?;
        composition_wire::decode(&response, molecule.atoms().len())
    }

    fn molecule_to_molblock(
        &self,
        molecule: &MolGraph,
        version: MolblockVersion,
    ) -> Result<String, ChemistryError> {
        let request = molblock_wire::encode(molecule, version)?;
        let response = self
            .adapter
            .molecule_to_molblock(&request)
            .map_err(adapter_error)?;
        text_response::decode_multiline(&response, "molblock")
    }

    fn molecule_to_molblock_with_title(
        &self,
        molecule: &MolGraph,
        version: MolblockVersion,
        title: &str,
    ) -> Result<String, ChemistryError> {
        let request = molblock_wire::encode_titled(molecule, version, title)?;
        let response = self
            .adapter
            .molecule_to_molblock_with_title(&request)
            .map_err(adapter_error)?;
        let output = text_response::decode_multiline(&response, "molblock")?;
        molblock_wire::validate_output_title(&output, title)?;
        Ok(output)
    }

    fn molblock_to_molecule(&self, molblock: &str) -> Result<SmilesMolecule, ChemistryError> {
        validate_molblock_input(molblock)?;
        let response = self
            .adapter
            .molblock_to_molecule(molblock.as_bytes())
            .map_err(adapter_error)?;
        fcm1::decode(&response)
    }

    fn inchi_to_molecule(&self, inchi: &str) -> Result<SmilesMolecule, ChemistryError> {
        validate_inchi_input(inchi)?;
        let response = self
            .adapter
            .inchi_to_molecule(inchi.as_bytes())
            .map_err(adapter_error)?;
        fcm1::decode(&response)
    }

    fn molecule_to_inchi(
        &self,
        molecule: &MolGraph,
        mode: InchiMode,
    ) -> Result<String, ChemistryError> {
        let request = inchi_wire::encode(molecule, mode)?;
        let response = self
            .adapter
            .molecule_to_inchi(&request)
            .map_err(adapter_error)?;
        let output = text_response::decode(&response, "InChI")?;
        let valid_prefix = match mode {
            InchiMode::Standard => output.starts_with("InChI=1S/"),
            InchiMode::FixedHydrogen => {
                output.starts_with("InChI=1/") && !output.starts_with("InChI=1S/")
            }
        };
        if !valid_prefix {
            return Err(ChemistryError::MalformedNativeResponse {
                reason: "InChI response prefix does not match the requested mode".to_owned(),
            });
        }
        Ok(output)
    }

    fn inchi_to_inchi_key(&self, inchi: &str) -> Result<String, ChemistryError> {
        validate_inchi_input(inchi)?;
        let response = self
            .adapter
            .inchi_to_inchi_key(inchi.as_bytes())
            .map_err(adapter_error)?;
        let key = text_response::decode(&response, "InChIKey")?;
        inchi_wire::validate_key(inchi, &key)?;
        Ok(key)
    }

    fn records_to_sdf(
        &self,
        records: &[SdfRecord],
        version: MolblockVersion,
    ) -> Result<String, ChemistryError> {
        let request = sdf_wire::encode(records, version)?;
        let response = self
            .adapter
            .records_to_sdf(&request)
            .map_err(adapter_error)?;
        text_response::decode_multiline(&response, "SDF")
    }

    fn sdf_to_records(&self, input: &str) -> Result<Vec<ImportedSdfRecord>, ChemistryError> {
        NativeChemEngine::sdf_to_records(self, input)
    }

    fn kekulize(
        &self,
        molecule: &MolGraph,
        options: KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        molecule.validate_kekulize_input().map_err(|error| {
            ChemistryError::UnsupportedNativeRequest {
                reason: error.to_string(),
            }
        })?;
        let request = encode_kekulize_request(molecule, options)?;
        let response = self.adapter.kekulize(&request).map_err(adapter_error)?;
        let decoded = decode_response(&response).map_err(decode_error)?;
        finish_response(molecule, options, decoded)
    }
}

fn decode_coordinate_response(
    response: &[u8],
    expected_atom_count: usize,
) -> Result<Coordinates, ChemistryError> {
    if response.len() < COORDINATE_RESPONSE_HEADER_LENGTH {
        return Err(ChemistryError::TruncatedNativeResponse);
    }
    let mut reader = Reader::new(response);
    if reader.take(4).map_err(decode_error)? != COORDINATE_RESPONSE_MAGIC {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "coordinate response magic is not FCL1".to_owned(),
        });
    }
    if reader.u32().map_err(decode_error)? != 1 {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "unsupported coordinate response wire version".to_owned(),
        });
    }
    let status = reader.u32().map_err(decode_error)?;
    if !matches!(
        status,
        FERRUM_CHEM_RESULT_OK
            | FERRUM_CHEM_RESULT_MALFORMED_REQUEST
            | FERRUM_CHEM_RESULT_INVALID_MOLECULE
            | FERRUM_CHEM_RESULT_INTERNAL_FAILURE
    ) {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "unknown or inapplicable coordinate response result status".to_owned(),
        });
    }
    let detail_length =
        usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    let atom_count = usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    let detail =
        std::str::from_utf8(reader.take(detail_length).map_err(decode_error)?).map_err(|_| {
            ChemistryError::MalformedNativeResponse {
                reason: "coordinate response detail is not UTF-8".to_owned(),
            }
        })?;
    if status != FERRUM_CHEM_RESULT_OK {
        if atom_count != 0 || !reader.is_empty() {
            return Err(ChemistryError::MalformedNativeResponse {
                reason: "failed coordinate response contains coordinate records".to_owned(),
            });
        }
        return Err(ChemistryError::CoordinateGenerationFailed {
            reason: detail.to_owned(),
        });
    }
    if !detail.is_empty() || atom_count != expected_atom_count {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "coordinate response does not match the input atom order".to_owned(),
        });
    }
    let expected_bytes = atom_count.checked_mul(COORDINATE_BYTES).ok_or_else(|| {
        ChemistryError::MalformedNativeResponse {
            reason: "coordinate response length overflows this platform".to_owned(),
        }
    })?;
    if response.len().saturating_sub(reader.cursor) != expected_bytes {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "coordinate response has truncated or trailing records".to_owned(),
        });
    }
    let mut points = Vec::with_capacity(atom_count);
    for _ in 0..atom_count {
        let x = f64::from_le_bytes(
            reader
                .take(8)
                .map_err(decode_error)?
                .try_into()
                .expect("fixed"),
        );
        let y = f64::from_le_bytes(
            reader
                .take(8)
                .map_err(decode_error)?
                .try_into()
                .expect("fixed"),
        );
        points.push(
            Point2::new(x, y).map_err(|_| ChemistryError::MalformedNativeResponse {
                reason: "coordinate response contains a non-finite point".to_owned(),
            })?,
        );
    }
    Ok(Coordinates::new(points))
}
fn adapter_error(error: AdapterError) -> ChemistryError {
    if matches!(
        error,
        AdapterError::NativeStatus { status }
            if u64::from(status) == FERRUM_CHEM_CALL_ALLOCATION_FAILURE
    ) {
        return ChemistryError::ResourceExhausted {
            operation: "native adapter response",
        };
    }
    if let AdapterError::OperationUnavailable { operation } = error {
        return ChemistryError::OperationUnavailable { operation };
    }
    ChemistryError::NativeBoundary {
        reason: error.to_string(),
    }
}
fn encode_kekulize_request(
    molecule: &MolGraph,
    options: KekulizeOptions,
) -> Result<Vec<u8>, ChemistryError> {
    encode_graph_request(molecule, options_bits(options), options.max_backtracks())
}

fn encode_depiction_request(molecule: &MolGraph) -> Result<Vec<u8>, ChemistryError> {
    let policy = DepictionRequestOptions;
    encode_graph_request(
        molecule,
        policy.option_bits(),
        policy.parser_backtrack_sentinel(),
    )
}

fn encode_graph_request(
    molecule: &MolGraph,
    option_bits: u32,
    max_backtracks: u32,
) -> Result<Vec<u8>, ChemistryError> {
    let atom_count = checked_count(
        molecule.atoms().len(),
        FERRUM_CHEM_KEKULIZE_MAX_ATOMS,
        "atom count",
    )?;
    let bond_count = checked_count(
        molecule.bonds().len(),
        FERRUM_CHEM_KEKULIZE_MAX_BONDS,
        "bond count",
    )?;
    if max_backtracks == 0 || max_backtracks > FERRUM_CHEM_KEKULIZE_MAX_BACKTRACKS {
        return Err(ChemistryError::UnsupportedNativeRequest {
            reason: format!(
                "max_backtracks {} exceeds {FERRUM_CHEM_KEKULIZE_MAX_BACKTRACKS}",
                max_backtracks
            ),
        });
    }

    let capacity = REQUEST_HEADER_LENGTH
        .checked_add(usize::try_from(atom_count).expect("u32 fits usize") * ATOM_LENGTH)
        .and_then(|length| {
            length.checked_add(usize::try_from(bond_count).expect("u32 fits usize") * BOND_LENGTH)
        })
        .ok_or_else(|| ChemistryError::UnsupportedNativeRequest {
            reason: "request length overflows this platform".to_owned(),
        })?;
    let mut output = Vec::with_capacity(capacity);
    output.extend_from_slice(&REQUEST_MAGIC);
    put_u32(&mut output, FERRUM_CHEM_KEKULIZE_WIRE_VERSION);
    put_u32(&mut output, option_bits);
    put_u32(&mut output, max_backtracks);
    put_u32(&mut output, atom_count);
    put_u32(&mut output, bond_count);
    debug_assert_eq!(output.len(), REQUEST_HEADER_LENGTH);

    for atom in molecule.atoms() {
        encode_atom(&mut output, atom);
    }
    for bond in molecule.bonds() {
        encode_bond(&mut output, bond)?;
    }
    Ok(output)
}

fn checked_count(count: usize, maximum: u32, name: &str) -> Result<u32, ChemistryError> {
    let count = u32::try_from(count).map_err(|_| ChemistryError::UnsupportedNativeRequest {
        reason: format!("{name} does not fit the adapter protocol"),
    })?;
    if count > maximum {
        return Err(ChemistryError::UnsupportedNativeRequest {
            reason: format!("{name} {count} exceeds {maximum}"),
        });
    }
    Ok(count)
}

fn options_bits(options: KekulizeOptions) -> u32 {
    (u32::from(options.clear_aromatic_flags()) * FERRUM_CHEM_KEKULIZE_OPTION_CLEAR_AROMATIC_FLAGS)
        | (u32::from(options.canonical()) * FERRUM_CHEM_KEKULIZE_OPTION_CANONICAL)
}

fn encode_atom(output: &mut Vec<u8>, atom: &MolAtom) {
    output.push(atom.atomic_number().get());
    output.push(u8::from(atom.is_aromatic()));
    let mut facts = 0_u32;
    if atom.formal_charge().is_some() {
        facts |= FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE;
    }
    if atom.isotope().is_some() {
        facts |= FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE;
    }
    if atom.explicit_hydrogens().is_some() {
        facts |= FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS;
    }
    put_u16(
        output,
        u16::try_from(facts).expect("generated fact constants fit u16"),
    );
    put_i32(output, atom.formal_charge().unwrap_or(0));
    put_u16(output, atom.isotope().unwrap_or(0));
    put_u16(output, atom.explicit_hydrogens().unwrap_or(0));
}

fn encode_bond(output: &mut Vec<u8>, bond: &MolBond) -> Result<(), ChemistryError> {
    let start =
        u32::try_from(bond.start()).map_err(|_| ChemistryError::UnsupportedNativeRequest {
            reason: "bond start index does not fit the adapter protocol".to_owned(),
        })?;
    let end = u32::try_from(bond.end()).map_err(|_| ChemistryError::UnsupportedNativeRequest {
        reason: "bond end index does not fit the adapter protocol".to_owned(),
    })?;
    put_u32(output, start);
    put_u32(output, end);
    output.push(match bond.order() {
        BondOrder::Single => wire_bond_type(FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE),
        BondOrder::Double => wire_bond_type(FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE),
        BondOrder::Triple => wire_bond_type(FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE),
        BondOrder::Aromatic => wire_bond_type(FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC),
        BondOrder::Quadruple => {
            return Err(ChemistryError::UnsupportedNativeRequest {
                reason: "quadruple bonds are not representable by adapter wire version 1"
                    .to_owned(),
            });
        }
    });
    output.push(u8::from(bond.is_aromatic()));
    put_u16(output, 0);
    Ok(())
}

fn wire_bond_type(value: u32) -> u8 {
    u8::try_from(value).expect("generated bond-type constant fits u8")
}

struct DecodedResponse {
    status: u32,
    detail: String,
    echoed_options: u32,
    echoed_max_backtracks: u32,
    atoms: Vec<MolAtom>,
    bonds: Vec<MolBond>,
}

fn decode_response(response: &[u8]) -> Result<DecodedResponse, DecodeFailure> {
    if response.len() < RESPONSE_HEADER_LENGTH {
        return Err(DecodeFailure::Truncated);
    }
    let mut reader = Reader::new(response);
    if reader.take(4)? != RESPONSE_MAGIC {
        return Err(DecodeFailure::Malformed("response magic is not FCR1"));
    }
    if reader.u32()? != FERRUM_CHEM_KEKULIZE_WIRE_VERSION {
        return Err(DecodeFailure::Malformed(
            "unsupported response wire version",
        ));
    }
    let status = reader.u32()?;
    if !matches!(
        status,
        FERRUM_CHEM_RESULT_OK
            | FERRUM_CHEM_RESULT_MALFORMED_REQUEST
            | FERRUM_CHEM_RESULT_INVALID_MOLECULE
            | FERRUM_CHEM_RESULT_DEPICTION_FAILURE
            | FERRUM_CHEM_RESULT_INTERNAL_FAILURE
    ) {
        return Err(DecodeFailure::Malformed("unknown response result status"));
    }
    let detail_length = usize::try_from(reader.u32()?).expect("u32 fits usize");
    if detail_length > FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES {
        return Err(DecodeFailure::Malformed(
            "response detail exceeds protocol maximum",
        ));
    }
    let echoed_options = reader.u32()?;
    if echoed_options & !OPTION_MASK != 0 {
        return Err(DecodeFailure::Malformed(
            "response echoed reserved option bits",
        ));
    }
    let echoed_max_backtracks = reader.u32()?;
    if status == FERRUM_CHEM_RESULT_OK {
        if echoed_max_backtracks == 0 || echoed_max_backtracks > FERRUM_CHEM_KEKULIZE_MAX_BACKTRACKS
        {
            return Err(DecodeFailure::Malformed(
                "response echoed invalid max_backtracks",
            ));
        }
    } else if echoed_options != 0 || echoed_max_backtracks != 0 {
        return Err(DecodeFailure::Malformed(
            "error response includes request option echoes",
        ));
    }
    let atom_count = reader.u32()?;
    let bond_count = reader.u32()?;
    if atom_count > FERRUM_CHEM_KEKULIZE_MAX_ATOMS || bond_count > FERRUM_CHEM_KEKULIZE_MAX_BONDS {
        return Err(DecodeFailure::Malformed(
            "response count exceeds protocol maximum",
        ));
    }
    let detail = std::str::from_utf8(reader.take(detail_length)?)
        .map_err(|_| DecodeFailure::Malformed("response detail is not UTF-8"))?
        .to_owned();
    if status != FERRUM_CHEM_RESULT_OK && (atom_count != 0 || bond_count != 0) {
        return Err(DecodeFailure::Malformed("error response includes topology"));
    }

    let required_records = u64::from(atom_count)
        .checked_mul(u64::try_from(ATOM_LENGTH).expect("record length fits u64"))
        .and_then(|length| {
            length.checked_add(
                u64::from(bond_count) * u64::try_from(BOND_LENGTH).expect("record length fits u64"),
            )
        })
        .ok_or(DecodeFailure::Malformed("response record length overflows"))?;
    let remaining = response
        .len()
        .checked_sub(reader.cursor)
        .ok_or(DecodeFailure::Truncated)?;
    let remaining = u64::try_from(remaining).expect("usize fits u64");
    if remaining < required_records {
        return Err(DecodeFailure::Truncated);
    }
    if remaining > required_records {
        return Err(DecodeFailure::Trailing);
    }

    let mut atoms = Vec::with_capacity(usize::try_from(atom_count).expect("u32 fits usize"));
    for _ in 0..atom_count {
        atoms.push(decode_atom(&mut reader)?);
    }
    let mut bonds = Vec::with_capacity(usize::try_from(bond_count).expect("u32 fits usize"));
    for _ in 0..bond_count {
        bonds.push(decode_bond(&mut reader)?);
    }
    if !reader.is_empty() {
        return Err(DecodeFailure::Trailing);
    }
    Ok(DecodedResponse {
        status,
        detail,
        echoed_options,
        echoed_max_backtracks,
        atoms,
        bonds,
    })
}

fn decode_atom(reader: &mut Reader<'_>) -> Result<MolAtom, DecodeFailure> {
    let atomic_number = AtomicNumber::try_from(reader.u8()?)
        .map_err(|_| DecodeFailure::Malformed("response has an unsupported atomic number"))?;
    let aromatic = bool_from_byte(reader.u8()?, "atom aromatic flag")?;
    let facts = u32::from(reader.u16()?);
    if facts & !FACT_MASK != 0 {
        return Err(DecodeFailure::Malformed(
            "response atom has reserved presence bits",
        ));
    }
    let formal_charge_value = reader.i32()?;
    let isotope_value = reader.u16()?;
    let hydrogens_value = reader.u16()?;
    let formal_charge = optional_fact(
        facts,
        FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE,
        formal_charge_value,
        "formal charge",
    )?;
    let isotope = optional_fact(
        facts,
        FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE,
        isotope_value,
        "isotope",
    )?;
    let explicit_hydrogens = optional_fact(
        facts,
        FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS,
        hydrogens_value,
        "explicit hydrogens",
    )?;
    MolAtom::new(
        atomic_number,
        formal_charge,
        isotope,
        explicit_hydrogens,
        aromatic,
    )
    .map_err(|_| DecodeFailure::Malformed("response atom facts violate MolGraph invariants"))
}

fn optional_fact<T: Eq + Default>(
    flags: u32,
    flag: u32,
    value: T,
    name: &'static str,
) -> Result<Option<T>, DecodeFailure> {
    if flags & flag != 0 {
        Ok(Some(value))
    } else if value == T::default() {
        Ok(None)
    } else {
        Err(DecodeFailure::Malformed(match name {
            "formal charge" => "absent formal charge has a nonzero value",
            "isotope" => "absent isotope has a nonzero value",
            _ => "absent explicit hydrogens has a nonzero value",
        }))
    }
}

fn decode_bond(reader: &mut Reader<'_>) -> Result<MolBond, DecodeFailure> {
    let start = usize::try_from(reader.u32()?).expect("u32 fits usize");
    let end = usize::try_from(reader.u32()?).expect("u32 fits usize");
    let order = match u32::from(reader.u8()?) {
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE => BondOrder::Single,
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE => BondOrder::Double,
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE => BondOrder::Triple,
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC => BondOrder::Aromatic,
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED | FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE => {
            return Err(DecodeFailure::Malformed(
                "response has an unsupported bond type",
            ));
        }
        _ => {
            return Err(DecodeFailure::Malformed(
                "response has an unsupported bond type",
            ));
        }
    };
    let aromatic = bool_from_byte(reader.u8()?, "bond aromatic flag")?;
    if reader.u16()? != 0 {
        return Err(DecodeFailure::Malformed(
            "response bond reserved field is nonzero",
        ));
    }
    if order == BondOrder::Aromatic && !aromatic {
        return Err(DecodeFailure::Malformed(
            "aromatic bond type lacks aromatic flag",
        ));
    }
    if aromatic && matches!(order, BondOrder::Triple) {
        return Err(DecodeFailure::Malformed("aromatic triple bond is invalid"));
    }
    Ok(MolBond::new(start, end, order, aromatic))
}

fn bool_from_byte(value: u8, name: &'static str) -> Result<bool, DecodeFailure> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(DecodeFailure::Malformed(name)),
    }
}

fn finish_response(
    input: &MolGraph,
    options: KekulizeOptions,
    response: DecodedResponse,
) -> Result<MolGraph, ChemistryError> {
    match response.status {
        FERRUM_CHEM_RESULT_OK => {
            if response.echoed_options != options_bits(options)
                || response.echoed_max_backtracks != options.max_backtracks()
            {
                return Err(ChemistryError::MalformedNativeResponse {
                    reason: "response did not echo the submitted options".to_owned(),
                });
            }
        }
        FERRUM_CHEM_RESULT_DEPICTION_FAILURE => {
            return Err(ChemistryError::KekulizationFailed {
                reason: response.detail,
            });
        }
        FERRUM_CHEM_RESULT_MALFORMED_REQUEST
        | FERRUM_CHEM_RESULT_INVALID_MOLECULE
        | FERRUM_CHEM_RESULT_INTERNAL_FAILURE => {
            return Err(ChemistryError::NativeRejected {
                status: response.status,
                reason: response.detail,
            });
        }
        _ => unreachable!("decoded response status is known"),
    }
    if !response.detail.is_empty() {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "successful response contains diagnostic detail".to_owned(),
        });
    }
    validate_output_semantics(input, &response.atoms, &response.bonds, options)?;
    MolGraph::new(response.atoms, response.bonds, input.coordinates().cloned()).map_err(|error| {
        ChemistryError::MalformedNativeResponse {
            reason: format!("response graph violates Ferrum invariants: {error}"),
        }
    })
}

fn validate_output_semantics(
    input: &MolGraph,
    output_atoms: &[MolAtom],
    output_bonds: &[MolBond],
    options: KekulizeOptions,
) -> Result<(), ChemistryError> {
    if input.atoms().len() != output_atoms.len() || input.bonds().len() != output_bonds.len() {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "response changed graph topology counts".to_owned(),
        });
    }
    for (index, (original, returned)) in input.atoms().iter().zip(output_atoms).enumerate() {
        if original.atomic_number() != returned.atomic_number()
            || original.formal_charge() != returned.formal_charge()
            || original.isotope() != returned.isotope()
            || original.explicit_hydrogens() != returned.explicit_hydrogens()
        {
            return Err(ChemistryError::MalformedNativeResponse {
                reason: format!("response changed immutable atom facts at index {index}"),
            });
        }
        if options.clear_aromatic_flags() {
            if original.is_aromatic() && returned.is_aromatic() {
                return Err(ChemistryError::MalformedNativeResponse {
                    reason: format!("response retained aromatic atom flag at index {index}"),
                });
            }
            if !original.is_aromatic() && returned.is_aromatic() {
                return Err(ChemistryError::MalformedNativeResponse {
                    reason: format!("response changed non-aromatic atom flag at index {index}"),
                });
            }
        } else if original.is_aromatic() != returned.is_aromatic() {
            return Err(ChemistryError::MalformedNativeResponse {
                reason: format!("response changed atom aromatic flag at index {index}"),
            });
        }
    }
    for (index, (original, returned)) in input.bonds().iter().zip(output_bonds).enumerate() {
        if original.start() != returned.start() || original.end() != returned.end() {
            return Err(ChemistryError::MalformedNativeResponse {
                reason: format!("response changed bond endpoints at index {index}"),
            });
        }
        if original.order() == BondOrder::Aromatic {
            if !matches!(returned.order(), BondOrder::Single | BondOrder::Double) {
                return Err(ChemistryError::MalformedNativeResponse {
                    reason: format!("response did not kekulize aromatic bond at index {index}"),
                });
            }
            let expected_aromatic = !options.clear_aromatic_flags();
            if returned.is_aromatic() != expected_aromatic {
                return Err(ChemistryError::MalformedNativeResponse {
                    reason: format!(
                        "response has wrong aromatic flag for kekulized bond at index {index}"
                    ),
                });
            }
        } else if original.order() != returned.order()
            || original.is_aromatic() != returned.is_aromatic()
        {
            return Err(ChemistryError::MalformedNativeResponse {
                reason: format!("response changed non-aromatic bond at index {index}"),
            });
        }
    }
    Ok(())
}

fn decode_error(error: DecodeFailure) -> ChemistryError {
    match error {
        DecodeFailure::Malformed(reason) => ChemistryError::MalformedNativeResponse {
            reason: reason.to_owned(),
        },
        DecodeFailure::Truncated => ChemistryError::TruncatedNativeResponse,
        DecodeFailure::Trailing => ChemistryError::TrailingNativeResponse,
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DecodeFailure {
    Malformed(&'static str),
    Truncated,
    Trailing,
}
#[cfg(test)]
mod native_engine_tests;
