// Native adapter implementation of the safe chemistry engine.
//
// The byte protocol is deliberately private to this module. All callers see
// only owned MolGraph values and typed ChemistryError variants.

mod adapter_boundary;
mod codec_support;
mod composition_wire;
mod fcm1;
mod graph_wire;
mod inchi_wire;
mod molblock_import;
mod molblock_wire;
mod sdf_import;
mod sdf_wire;
mod smarts_wire;
mod text_response;

#[cfg(test)]
mod smiles_write_tests;

use codec_support::{put_i32, put_u16, put_u32, Reader};

use std::path::Path;

use crate::{
    AtomChirality, AtomicNumber, BondDirection, BondOrder, BondStereo, ChemEngine, ChemistryError,
    Coordinates, ImportedSdfRecord, InchiMode, KekulizeOptions, MolAtom, MolBond, MolGraph,
    MolblockVersion, MoleculeComposition, NativeTextOutputLimit, Point2, SdfProperty, SdfRecord,
    SmartsMatchOptions,
    SmartsMatchResult, SmartsMatchUnavailableReason, SmilesMolecule,
    FERRUM_CHEM_CALL_ALLOCATION_FAILURE, FERRUM_CHEM_COMPOSITION_ENTRY_BYTES,
    FERRUM_CHEM_COMPOSITION_FLAGS_NONE, FERRUM_CHEM_COMPOSITION_MAX_DETAIL_BYTES,
    FERRUM_CHEM_COMPOSITION_MAX_FORMULA_BYTES, FERRUM_CHEM_COMPOSITION_RESPONSE_HEADER_BYTES,
    FERRUM_CHEM_COMPOSITION_WIRE_VERSION, FERRUM_CHEM_COORDINATE_BYTES,
    FERRUM_CHEM_GRAPH_ATOM_BYTES, FERRUM_CHEM_GRAPH_BOND_BYTES, FERRUM_CHEM_GRAPH_FLAGS_NONE,
    FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES, FERRUM_CHEM_GRAPH_WIRE_VERSION,
    FERRUM_CHEM_INCHI_MAX_BYTES, FERRUM_CHEM_KEKULIZE_ATOM_BYTES, FERRUM_CHEM_KEKULIZE_BOND_BYTES,
    FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC, FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE,
    FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE, FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE,
    FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE, FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED,
    FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS, FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE,
    FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE, FERRUM_CHEM_KEKULIZE_MAX_ATOMS,
    FERRUM_CHEM_KEKULIZE_MAX_BACKTRACKS, FERRUM_CHEM_KEKULIZE_MAX_BONDS,
    FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES, FERRUM_CHEM_KEKULIZE_OPTION_CANONICAL,
    FERRUM_CHEM_KEKULIZE_OPTION_CLEAR_AROMATIC_FLAGS, FERRUM_CHEM_KEKULIZE_REQUEST_HEADER_BYTES,
    FERRUM_CHEM_KEKULIZE_RESPONSE_HEADER_BYTES, FERRUM_CHEM_KEKULIZE_WIRE_VERSION,
    FERRUM_CHEM_MAX_RESPONSE_BYTES, FERRUM_CHEM_MOLBLOCK_FLAGS_NONE,
    FERRUM_CHEM_MOLBLOCK_FORMAT_V2000, FERRUM_CHEM_MOLBLOCK_FORMAT_V3000,
    FERRUM_CHEM_MOLBLOCK_REQUEST_HEADER_BYTES, FERRUM_CHEM_MOLBLOCK_WIRE_VERSION,
    FERRUM_CHEM_MOLECULE_ATOM_BYTES, FERRUM_CHEM_MOLECULE_BOND_BYTES,
    FERRUM_CHEM_MOLECULE_RESPONSE_HEADER_BYTES, FERRUM_CHEM_RESULT_DEPICTION_FAILURE,
    FERRUM_CHEM_RESULT_INTERNAL_FAILURE, FERRUM_CHEM_RESULT_INVALID_MOLECULE,
    FERRUM_CHEM_RESULT_MALFORMED_REQUEST, FERRUM_CHEM_RESULT_OK, FERRUM_CHEM_RESULT_RESOURCE_LIMIT,
    FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE, FERRUM_CHEM_SDF_FLAGS_NONE,
    FERRUM_CHEM_SDF_MAX_PROPERTIES, FERRUM_CHEM_SDF_MAX_RECORDS,
    FERRUM_CHEM_SDF_PROPERTY_HEADER_BYTES, FERRUM_CHEM_SDF_RECORD_HEADER_BYTES,
    FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES, FERRUM_CHEM_SDF_RESPONSE_HEADER_BYTES,
    FERRUM_CHEM_SDF_WIRE_VERSION, FERRUM_CHEM_SMILES_MAX_BYTES, FERRUM_CHEM_SMILES_WRITE_MAX_BYTES,
    FERRUM_CHEM_TEXT_FLAGS_NONE, FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES,
    FERRUM_CHEM_TEXT_WIRE_VERSION, FERRUM_CHEM_TITLED_MOLBLOCK_REQUEST_HEADER_BYTES,
    FERRUM_CHEM_TITLED_MOLBLOCK_WIRE_VERSION,
};
use adapter_boundary::{AdapterError, ChemistryAdapter};

const REQUEST_MAGIC: [u8; 4] = *b"FCK1";
const NATIVE_ADAPTER_BOUNDARY_REASON: &str =
    "the Ferrum chemistry adapter is unavailable or returned an invalid response";

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
    pub fn molecule_to_smiles(
        &self,
        molecule: &MolGraph,
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        <Self as ChemEngine>::molecule_to_smiles(self, molecule, limit)
    }

    /// Calculate isotope-aware formula, counts, charge, and masses.
    pub fn molecule_composition(
        &self,
        molecule: &MolGraph,
    ) -> Result<MoleculeComposition, ChemistryError> {
        <Self as ChemEngine>::molecule_composition(self, molecule)
    }

    /// Enumerate bounded query-ordered SMARTS matches for one supplied graph.
    pub fn smarts_match(
        &self,
        query: &str,
        target: &MolGraph,
        options: SmartsMatchOptions,
    ) -> Result<SmartsMatchResult, ChemistryError> {
        <Self as ChemEngine>::smarts_match(self, query, target, options)
    }

    /// Export a complete coordinate-bearing graph as explicit molblock syntax.
    pub fn molecule_to_molblock(
        &self,
        molecule: &MolGraph,
        version: MolblockVersion,
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        <Self as ChemEngine>::molecule_to_molblock(self, molecule, version, limit)
    }

    /// Export a coordinate-bearing graph with an exact first-line title.
    pub fn molecule_to_molblock_with_title(
        &self,
        molecule: &MolGraph,
        version: MolblockVersion,
        title: &str,
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        <Self as ChemEngine>::molecule_to_molblock_with_title(self, molecule, version, title, limit)
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
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        <Self as ChemEngine>::molecule_to_inchi(self, molecule, mode, limit)
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
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        <Self as ChemEngine>::records_to_sdf(self, records, version, limit)
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

    fn molecule_to_smiles(
        &self,
        molecule: &MolGraph,
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        let request = graph_wire::encode(molecule)?;
        let response = self
            .adapter
            .molecule_to_smiles(&request, limit.bytes())
            .map_err(adapter_error)?;
        text_response::decode_smiles(&response, limit)
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

    fn smarts_match(
        &self,
        query: &str,
        target: &MolGraph,
        options: SmartsMatchOptions,
    ) -> Result<SmartsMatchResult, ChemistryError> {
        let request = smarts_wire::encode_request(query, target, options)
            .map_err(smarts_wire::map_wire_error)?;
        let response = self
            .adapter
            .smarts_match(&request)
            .map_err(smarts_adapter_error)?;
        smarts_wire::decode_response(&response, target.atoms().len(), options.max_matches())
            .map_err(smarts_wire::map_wire_error)
    }

    fn molecule_to_molblock(
        &self,
        molecule: &MolGraph,
        version: MolblockVersion,
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        let request = molblock_wire::encode(molecule, version)?;
        let response = self
            .adapter
            .molecule_to_molblock(&request, limit.bytes())
            .map_err(adapter_error)?;
        text_response::decode_multiline(&response, "molblock", limit)
    }

    fn molecule_to_molblock_with_title(
        &self,
        molecule: &MolGraph,
        version: MolblockVersion,
        title: &str,
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        let request = molblock_wire::encode_titled(molecule, version, title)?;
        let response = self
            .adapter
            .molecule_to_molblock_with_title(&request, limit.bytes())
            .map_err(adapter_error)?;
        let output = text_response::decode_multiline(&response, "molblock", limit)?;
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
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        let request = inchi_wire::encode(molecule, mode)?;
        let response = self
            .adapter
            .molecule_to_inchi(&request, limit.bytes())
            .map_err(adapter_error)?;
        let output = text_response::decode_bounded(&response, "InChI", limit)?;
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
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        let request = sdf_wire::encode(records, version)?;
        let response = self
            .adapter
            .records_to_sdf(&request, limit.bytes())
            .map_err(adapter_error)?;
        text_response::decode_multiline(&response, "SDF", limit)
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
