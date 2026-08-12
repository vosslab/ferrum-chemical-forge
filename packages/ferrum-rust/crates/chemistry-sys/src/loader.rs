//! Explicit dynamic-library loading and safe adapter operation dispatch.

use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;

use libloading::Library;

use crate::buffer::{FerrumChemOwnedBuffer, finish_call};
use crate::contract::{
    AbiVersionFn, AdapterError, BufferFreeFn, CapabilitiesFn, FERRUM_CHEM_ALL_KNOWN_CAPABILITIES,
    FERRUM_CHEM_CAPABILITY_GENERATE_2D, FERRUM_CHEM_CAPABILITY_KEKULIZE,
    FERRUM_CHEM_CAPABILITY_SMILES, OperationFn,
};

/// A loaded adapter whose native result buffers are released by this crate.
///
/// The adapter owns foreign allocations until this safe wrapper copies and
/// releases them. It is intentionally neither `Send` nor `Sync`: ABI-3 makes
/// no thread-safety promise, and the retained `Library` keeps every resolved
/// function pointer valid through the last native call.
pub struct ChemistryAdapter {
    _library: Library,
    abi_version: AbiVersionFn,
    capabilities: u64,
    kekulize: OperationFn,
    generate_2d: OperationFn,
    smiles_to_2d: OperationFn,
    buffer_free: BufferFreeFn,
    not_thread_safe: PhantomData<Rc<()>>,
}

impl ChemistryAdapter {
    /// Opens `library_path` and verifies the caller-selected ABI version.
    ///
    /// The explicit path prevents accidental loading through a dynamic-loader
    /// search path. On success this object owns the library for the lifetime of
    /// every resolved function pointer and returned native allocation.
    pub fn load(library_path: &Path, expected_abi: u32) -> Result<Self, AdapterError> {
        // SAFETY: `library_path` is selected explicitly by the caller. The
        // resulting `Library` is stored in this value, which outlives all copied
        // symbols and every adapter call made through them.
        let library = unsafe { Library::new(library_path) }?;
        let abi_version: AbiVersionFn = load_symbol(&library, b"ferrum_chem_abi_version\0")?;
        let kekulize: OperationFn = load_symbol(&library, b"ferrum_chem_kekulize_v1\0")?;
        let capabilities: CapabilitiesFn = load_symbol(&library, b"ferrum_chem_capabilities_v1\0")?;
        let generate_2d: OperationFn = load_symbol(&library, b"ferrum_chem_generate_2d_v1\0")?;
        let smiles_to_2d: OperationFn = load_symbol(&library, b"ferrum_chem_smiles_to_2d_v1\0")?;
        let buffer_free: BufferFreeFn =
            load_symbol(&library, b"ferrum_chem_owned_buffer_free_v1\0")?;

        // SAFETY: the symbol was resolved from `library` with ABI-3's exact C
        // function type. The library remains retained after construction.
        let actual_abi = unsafe { abi_version() };
        if actual_abi != expected_abi {
            return Err(AdapterError::AbiMismatch {
                expected: expected_abi,
                actual: actual_abi,
            });
        }
        // SAFETY: construction resolved the exact ABI-3 function type and
        // `library` remains alive throughout this validation call.
        let capability_bits = unsafe { capabilities() };
        let unknown = capability_bits & !FERRUM_CHEM_ALL_KNOWN_CAPABILITIES;
        if unknown != 0 {
            return Err(AdapterError::UnknownCapabilities { unknown });
        }

        Ok(Self {
            _library: library,
            abi_version,
            capabilities: capability_bits,
            kekulize,
            generate_2d,
            smiles_to_2d,
            buffer_free,
            not_thread_safe: PhantomData,
        })
    }

    /// Returns the version reported by the already-validated native adapter.
    #[must_use]
    pub fn abi_version(&self) -> u32 {
        // SAFETY: construction resolved and validated this function pointer.
        // `self._library` remains alive for the duration of this call.
        unsafe { (self.abi_version)() }
    }

    /// Returns the immutable ABI-3 operation bitset declared by this adapter.
    #[must_use]
    pub fn capabilities(&self) -> u64 {
        self.capabilities
    }

    /// Whether this adapter declares its deterministic depiction operation.
    #[must_use]
    pub fn supports_generate_2d(&self) -> bool {
        self.capabilities() & FERRUM_CHEM_CAPABILITY_GENERATE_2D != 0
    }

    /// Kekulizes an opaque version-one request and returns owned response bytes.
    pub fn kekulize(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        if self.capabilities() & FERRUM_CHEM_CAPABILITY_KEKULIZE == 0 {
            return Err(AdapterError::OperationUnavailable {
                operation: "kekulize",
            });
        }
        self.call(self.kekulize, input)
    }

    /// Generates an opaque version-one two-dimensional coordinate response.
    pub fn generate_2d(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        if !self.supports_generate_2d() {
            return Err(AdapterError::OperationUnavailable {
                operation: "generate_2d_coordinates",
            });
        }
        self.call(self.generate_2d, input)
    }

    /// Parses SMILES and returns canonical SMILES with atom-aligned 2D coordinates.
    pub fn smiles_to_2d(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        if self.capabilities() & FERRUM_CHEM_CAPABILITY_SMILES == 0 {
            return Err(AdapterError::OperationUnavailable {
                operation: "smiles_to_2d",
            });
        }
        self.call(self.smiles_to_2d, input)
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
    // SAFETY: each `name` is a NUL-terminated required ABI-3 symbol. `T` is
    // inferred only from a concrete ABI-3 function field, and copying the
    // pointer is safe while `ChemistryAdapter` retains `library`.
    Ok(unsafe { *library.get::<T>(name)? })
}
