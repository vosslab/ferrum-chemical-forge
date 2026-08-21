//! Explicit dynamic-library loading and safe adapter operation dispatch.
// This module is the sole private owner of native pointers, loaded symbols,
// foreign allocations, and their release callback. Its safe parent maps all
// failures into the established typed chemistry error contract.

use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;

use libloading::Library;

#[path = "adapter_boundary_buffer.rs"]
mod adapter_boundary_buffer;

#[cfg(test)]
#[path = "adapter_boundary_buffer_tests.rs"]
mod adapter_boundary_buffer_tests;
#[cfg(test)]
#[path = "adapter_boundary_contract_tests.rs"]
mod adapter_boundary_contract_tests;

use self::adapter_boundary_buffer::{FerrumChemOwnedBuffer, finish_call};
use crate::{
    FERRUM_CHEM_ALL_KNOWN_CAPABILITIES, FERRUM_CHEM_CAPABILITY_COMPOSITION,
    FERRUM_CHEM_CAPABILITY_GENERATE_2D, FERRUM_CHEM_CAPABILITY_INCHI,
    FERRUM_CHEM_CAPABILITY_KEKULIZE, FERRUM_CHEM_CAPABILITY_MOLFILE,
    FERRUM_CHEM_CAPABILITY_MOLFILE_READ, FERRUM_CHEM_CAPABILITY_MOLFILE_TITLE,
    FERRUM_CHEM_CAPABILITY_SDF_READ, FERRUM_CHEM_CAPABILITY_SDF_WRITE,
    FERRUM_CHEM_CAPABILITY_SMARTS, FERRUM_CHEM_CAPABILITY_SMARTS_MATCH,
    FERRUM_CHEM_CAPABILITY_SMILES_MOLECULE, FERRUM_CHEM_CAPABILITY_SMILES_WRITE,
};

type AbiVersionFn = unsafe extern "C" fn() -> u32;
type CapabilitiesFn = unsafe extern "C" fn() -> u64;
type OperationFn = unsafe extern "C" fn(*const u8, u64, *mut FerrumChemOwnedBuffer) -> u32;
type BufferFreeFn = unsafe extern "C" fn(*mut FerrumChemOwnedBuffer);

/// Failures while the private native-adapter boundary loads or releases data.
#[derive(Debug, thiserror::Error)]
pub(super) enum AdapterError {
    #[error("could not load Ferrum chemistry adapter: {0}")]
    Load(#[from] libloading::Error),
    #[error("Ferrum chemistry ABI mismatch: expected {expected}, found {actual}")]
    AbiMismatch { expected: u32, actual: u32 },
    #[error("Ferrum chemistry adapter advertised unknown capability bits 0x{unknown:016x}")]
    UnknownCapabilities { unknown: u64 },
    #[error("Ferrum chemistry adapter returned a null buffer with length {length}")]
    NullBuffer { length: u64 },
    #[error("Ferrum chemistry adapter returned an unrepresentable buffer length {length}")]
    BufferTooLarge { length: u64 },
    #[error("Ferrum chemistry adapter returned {length} bytes, above the {maximum}-byte limit")]
    ResponseTooLarge { length: u64, maximum: u64 },
    #[error("Ferrum chemistry adapter did not clear its released buffer")]
    BufferFreeDidNotClear,
    #[error("Ferrum chemistry adapter execution failed with status {status}")]
    NativeStatus { status: u32 },
    #[error("Ferrum chemistry adapter does not provide operation: {operation}")]
    OperationUnavailable { operation: &'static str },
}

/// A loaded adapter whose native result buffers are released by this crate.
///
/// The adapter owns foreign allocations until this safe wrapper copies and
/// releases them. It is intentionally neither `Send` nor `Sync`: ABI-4 makes
/// no thread-safety promise, and the retained `Library` keeps every resolved
/// function pointer valid through the last native call.
pub(super) struct ChemistryAdapter {
    _library: Library,
    #[cfg_attr(not(test), allow(dead_code))]
    abi_version: AbiVersionFn,
    #[cfg_attr(not(test), allow(dead_code))]
    capabilities: u64,
    kekulize: Option<OperationFn>,
    generate_2d: Option<OperationFn>,
    smiles_to_molecule: Option<OperationFn>,
    molecule_to_smarts: Option<OperationFn>,
    molecule_to_smiles: Option<OperationFn>,
    molecule_to_molblock: Option<OperationFn>,
    molecule_to_molblock_with_title: Option<OperationFn>,
    records_to_sdf: Option<OperationFn>,
    sdf_to_records: Option<OperationFn>,
    molblock_to_molecule: Option<OperationFn>,
    inchi_to_molecule: Option<OperationFn>,
    molecule_to_inchi: Option<OperationFn>,
    inchi_to_inchi_key: Option<OperationFn>,
    molecule_composition: Option<OperationFn>,
    smarts_match: Option<OperationFn>,
    buffer_free: BufferFreeFn,
    not_thread_safe: PhantomData<Rc<()>>,
}

impl ChemistryAdapter {
    /// Opens `library_path` and verifies the caller-selected ABI version.
    ///
    /// The explicit path prevents accidental loading through a dynamic-loader
    /// search path. On success this object owns the library for the lifetime of
    /// every resolved function pointer and returned native allocation.
    pub(super) fn load(library_path: &Path, expected_abi: u32) -> Result<Self, AdapterError> {
        // SAFETY: `library_path` is selected explicitly by the caller. The
        // resulting `Library` is stored in this value, which outlives all copied
        // symbols and every adapter call made through them.
        let library = unsafe { Library::new(library_path) }?;
        let abi_version: AbiVersionFn = load_symbol(&library, b"ferrum_chem_abi_version\0")?;
        let capabilities: CapabilitiesFn = load_symbol(&library, b"ferrum_chem_capabilities_v1\0")?;
        let buffer_free: BufferFreeFn =
            load_symbol(&library, b"ferrum_chem_owned_buffer_free_v1\0")?;

        // SAFETY: the symbol was resolved from `library` with ABI-4's exact C
        // function type. The library remains retained after construction.
        let actual_abi = unsafe { abi_version() };
        if actual_abi != expected_abi {
            return Err(AdapterError::AbiMismatch {
                expected: expected_abi,
                actual: actual_abi,
            });
        }
        // SAFETY: construction resolved the exact ABI-4 function type and
        // `library` remains alive throughout this validation call.
        let capability_bits = unsafe { capabilities() };
        let unknown = capability_bits & !FERRUM_CHEM_ALL_KNOWN_CAPABILITIES;
        if unknown != 0 {
            return Err(AdapterError::UnknownCapabilities { unknown });
        }
        let kekulize = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_KEKULIZE,
            b"ferrum_chem_kekulize_v1\0",
        )?;
        let generate_2d = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_GENERATE_2D,
            b"ferrum_chem_generate_2d_v1\0",
        )?;
        let smiles_to_molecule = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_SMILES_MOLECULE,
            b"ferrum_chem_smiles_to_molecule_v1\0",
        )?;
        let molecule_to_smarts = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_SMARTS,
            b"ferrum_chem_molecule_to_smarts_v1\0",
        )?;
        let molecule_to_smiles = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_SMILES_WRITE,
            b"ferrum_chem_molecule_to_smiles_v1\0",
        )?;
        let molecule_to_molblock = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_MOLFILE,
            b"ferrum_chem_molecule_to_molblock_v1\0",
        )?;
        let molecule_to_molblock_with_title = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_MOLFILE_TITLE,
            b"ferrum_chem_molecule_to_molblock_with_title_v1\0",
        )?;
        let records_to_sdf = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_SDF_WRITE,
            b"ferrum_chem_records_to_sdf_v1\0",
        )?;
        let sdf_to_records = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_SDF_READ,
            b"ferrum_chem_sdf_to_records_v1\0",
        )?;
        let molblock_to_molecule = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_MOLFILE_READ,
            b"ferrum_chem_molblock_to_molecule_v1\0",
        )?;
        let inchi_to_molecule = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_INCHI,
            b"ferrum_chem_inchi_to_molecule_v1\0",
        )?;
        let molecule_to_inchi = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_INCHI,
            b"ferrum_chem_molecule_to_inchi_v1\0",
        )?;
        let inchi_to_inchi_key = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_INCHI,
            b"ferrum_chem_inchi_to_inchi_key_v1\0",
        )?;
        let molecule_composition = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_COMPOSITION,
            b"ferrum_chem_molecule_composition_v1\0",
        )?;
        // ABI-5 reserves this transport for the API-owned FCQ1/FQM1 core.
        // Resolve its exact symbol when advertised, but do not retain or expose
        // a raw-byte call before that private consumer exists.
        let smarts_match = load_operation(
            &library,
            capability_bits,
            FERRUM_CHEM_CAPABILITY_SMARTS_MATCH,
            b"ferrum_chem_smarts_match_v1\0",
        )?;

        Ok(Self {
            _library: library,
            abi_version,
            capabilities: capability_bits,
            kekulize,
            generate_2d,
            smiles_to_molecule,
            molecule_to_smarts,
            molecule_to_smiles,
            molecule_to_molblock,
            molecule_to_molblock_with_title,
            records_to_sdf,
            sdf_to_records,
            molblock_to_molecule,
            inchi_to_molecule,
            molecule_to_inchi,
            inchi_to_inchi_key,
            molecule_composition,
            smarts_match,
            buffer_free,
            not_thread_safe: PhantomData,
        })
    }

    /// Returns the version reported by the already-validated native adapter.
    #[must_use]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn abi_version(&self) -> u32 {
        // SAFETY: construction resolved and validated this function pointer.
        // `self._library` remains alive for the duration of this call.
        unsafe { (self.abi_version)() }
    }

    /// Returns the immutable ABI-5 operation bitset declared by this adapter.
    #[must_use]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn capabilities(&self) -> u64 {
        self.capabilities
    }

    /// Whether this adapter declares its deterministic depiction operation.
    #[must_use]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn supports_generate_2d(&self) -> bool {
        self.capabilities() & FERRUM_CHEM_CAPABILITY_GENERATE_2D != 0
    }

    /// Kekulizes an opaque version-one request and returns owned response bytes.
    pub(super) fn kekulize(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.kekulize, "kekulize", input)
    }

    /// Generates an opaque version-one two-dimensional coordinate response.
    pub(super) fn generate_2d(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.generate_2d, "generate_2d_coordinates", input)
    }

    /// Parses SMILES and returns a complete owned molecule envelope.
    pub(super) fn smiles_to_molecule(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.smiles_to_molecule, "smiles_to_molecule", input)
    }

    /// Exports a complete graph request as canonical SMARTS text.
    pub(super) fn molecule_to_smarts(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.molecule_to_smarts, "molecule_to_smarts", input)
    }

    /// Exports a complete graph request as canonical isomeric SMILES text.
    pub(super) fn molecule_to_smiles(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.molecule_to_smiles, "molecule_to_smiles", input)
    }

    /// Exports a coordinate-bearing graph request as V2000 or V3000 molblock text.
    pub(super) fn molecule_to_molblock(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.molecule_to_molblock, "molecule_to_molblock", input)
    }

    /// Exports a titled coordinate-bearing graph as V2000 or V3000 molblock text.
    pub(super) fn molecule_to_molblock_with_title(
        &self,
        input: &[u8],
    ) -> Result<Vec<u8>, AdapterError> {
        self.call_required(
            self.molecule_to_molblock_with_title,
            "molecule_to_molblock_with_title",
            input,
        )
    }

    /// Exports ordered coordinate-bearing records through RDKit's SD writer.
    pub(super) fn records_to_sdf(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.records_to_sdf, "records_to_sdf", input)
    }

    /// Imports bounded SDF text into ordered owned-record response bytes.
    pub(super) fn sdf_to_records(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.sdf_to_records, "sdf_to_records", input)
    }

    /// Imports one bounded V2000 or V3000 molblock into owned molecule bytes.
    pub(super) fn molblock_to_molecule(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.molblock_to_molecule, "molblock_to_molecule", input)
    }

    /// Imports one bounded InChI line into owned molecule response bytes.
    pub(super) fn inchi_to_molecule(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.inchi_to_molecule, "inchi_to_molecule", input)
    }

    /// Exports one closed-mode complete-graph request as InChI text.
    pub(super) fn molecule_to_inchi(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.molecule_to_inchi, "molecule_to_inchi", input)
    }

    /// Derives an official InChIKey from one bounded InChI line.
    pub(super) fn inchi_to_inchi_key(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.inchi_to_inchi_key, "inchi_to_inchi_key", input)
    }

    /// Calculates isotope-aware composition for one complete graph request.
    pub(super) fn molecule_composition(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.molecule_composition, "molecule_composition", input)
    }

    /// Runs the private ABI-5 SMARTS transport and returns copied response bytes.
    pub(super) fn smarts_match(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        self.call_required(self.smarts_match, "smarts_match", input)
    }

    fn call_required(
        &self,
        operation: Option<OperationFn>,
        operation_name: &'static str,
        input: &[u8],
    ) -> Result<Vec<u8>, AdapterError> {
        let operation = operation.ok_or(AdapterError::OperationUnavailable {
            operation: operation_name,
        })?;
        self.call(operation, input)
    }

    fn call(&self, operation: OperationFn, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        let input_length = u64::try_from(input.len())
            .map_err(|_| AdapterError::BufferTooLarge { length: u64::MAX })?;
        let mut output = FerrumChemOwnedBuffer {
            data: std::ptr::null_mut(),
            len: 0,
        };

        // SAFETY: `input` remains borrowed for the duration of the call; its
        // pointer/length pair accurately describes that slice. `output` is a
        // writable `repr(C)` stack slot, and the validated operation comes from
        // `self._library`, retained for `self`'s lifetime. The adapter does not
        // retain input, and `self` is !Send/!Sync, so no cross-thread call occurs.
        let status = unsafe { operation(input.as_ptr(), input_length, &mut output) };
        finish_call(status, output, self.buffer_free)
    }
}

fn load_symbol<T: Copy>(library: &Library, name: &'static [u8]) -> Result<T, AdapterError> {
    // SAFETY: each `name` is a NUL-terminated required ABI-5 symbol. `T` is
    // inferred only from a concrete ABI-5 function field, and copying the
    // pointer is safe while `ChemistryAdapter` retains `library`.
    Ok(unsafe { *library.get::<T>(name)? })
}

fn load_operation(
    library: &Library,
    capabilities: u64,
    capability: u64,
    name: &'static [u8],
) -> Result<Option<OperationFn>, AdapterError> {
    if capabilities & capability == 0 {
        return Ok(None);
    }
    load_symbol(library, name).map(Some)
}
