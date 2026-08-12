//! ABI-3 declarations generated from the public native adapter header.

use crate::buffer::FerrumChemOwnedBuffer;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/adapter_abi_constants.rs"));
}

/// The ABI version required when loading a Ferrum chemistry adapter.
pub const FERRUM_CHEM_ADAPTER_ABI_VERSION: u64 = generated::FERRUM_CHEM_ADAPTER_ABI_VERSION;
/// Capability bit for the mandatory Kekulize operation.
pub const FERRUM_CHEM_CAPABILITY_KEKULIZE: u64 = generated::FERRUM_CHEM_CAPABILITY_KEKULIZE;
/// Capability bit for the optional SMILES-to-coordinate operation.
pub const FERRUM_CHEM_CAPABILITY_SMILES: u64 = generated::FERRUM_CHEM_CAPABILITY_SMILES;
/// Capability bit for the optional deterministic 2D depiction operation.
pub const FERRUM_CHEM_CAPABILITY_GENERATE_2D: u64 = generated::FERRUM_CHEM_CAPABILITY_GENERATE_2D;
/// Bitmask containing every ABI-3 capability currently named by the public C header.
///
/// The loader rejects advertised bits outside this mask. Each operation is
/// capability-gated, so an adapter may truthfully advertise its supported subset.
pub const FERRUM_CHEM_ALL_KNOWN_CAPABILITIES: u64 = generated::FERRUM_CHEM_ALL_KNOWN_CAPABILITIES;
/// Largest adapter-owned response that this crate will read into Rust memory.
pub const FERRUM_CHEM_MAX_RESPONSE_BYTES: u64 = generated::FERRUM_CHEM_MAX_RESPONSE_BYTES;

pub(crate) type AbiVersionFn = unsafe extern "C" fn() -> u32;
pub(crate) type CapabilitiesFn = unsafe extern "C" fn() -> u64;
pub(crate) type OperationFn =
    unsafe extern "C" fn(*const u8, u64, *mut FerrumChemOwnedBuffer) -> u32;
pub(crate) type BufferFreeFn = unsafe extern "C" fn(*mut FerrumChemOwnedBuffer);

/// Failures while loading or calling the byte-buffer adapter protocol.
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    /// The dynamic library could not be opened or lacks a required symbol.
    #[error("could not load Ferrum chemistry adapter: {0}")]
    Load(#[from] libloading::Error),

    /// The loaded adapter reports a different ABI version than its caller requires.
    #[error("Ferrum chemistry ABI mismatch: expected {expected}, found {actual}")]
    AbiMismatch { expected: u32, actual: u32 },

    /// The adapter advertised bits that the public ABI-3 header does not define.
    #[error("Ferrum chemistry adapter advertised unknown capability bits 0x{unknown:016x}")]
    UnknownCapabilities { unknown: u64 },

    /// The native adapter returned a non-null result length without a result pointer.
    #[error("Ferrum chemistry adapter returned a null buffer with length {length}")]
    NullBuffer { length: u64 },

    /// A native buffer length cannot be represented by this Rust process.
    #[error("Ferrum chemistry adapter returned an unrepresentable buffer length {length}")]
    BufferTooLarge { length: u64 },

    /// A native buffer exceeds the public adapter contract before Rust reads it.
    #[error("Ferrum chemistry adapter returned {length} bytes, above the {maximum}-byte limit")]
    ResponseTooLarge { length: u64, maximum: u64 },

    /// The adapter's release operation violated its ownership contract.
    #[error("Ferrum chemistry adapter did not clear its released buffer")]
    BufferFreeDidNotClear,

    /// The adapter returned a non-zero protocol status.
    #[error("Ferrum chemistry adapter execution failed with status {status}")]
    NativeStatus { status: u32 },

    /// The loaded adapter does not advertise a requested ABI-3 operation.
    #[error("Ferrum chemistry adapter does not provide operation: {operation}")]
    OperationUnavailable { operation: &'static str },
}
