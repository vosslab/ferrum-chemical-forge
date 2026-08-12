//! Safe ownership boundary for a dynamically loaded Ferrum chemistry adapter.
//!
//! This crate deliberately knows only the versioned byte-buffer ABI.  It does
//! not interpret a chemistry request or response, and it does not create a
//! link-time dependency on the adapter or RDKit.

use std::marker::PhantomData;
use std::path::Path;
use std::ptr::NonNull;
use std::rc::Rc;

use libloading::Library;

type AbiVersionFn = unsafe extern "C" fn() -> u32;
type KekulizeFn = unsafe extern "C" fn(*const u8, u64, *mut FerrumChemOwnedBuffer) -> u32;
type BufferFreeFn = unsafe extern "C" fn(*mut FerrumChemOwnedBuffer);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct FerrumChemOwnedBuffer {
    data: *mut u8,
    len: u64,
}

/// A loaded adapter whose native result buffers are released by this crate.
///
/// The native adapter is deliberately neither `Send` nor `Sync`: its ABI does
/// not promise thread safety, and callers can add a serializing owner later
/// without weakening this boundary.
pub struct ChemistryAdapter {
    _library: Library,
    abi_version: AbiVersionFn,
    kekulize: KekulizeFn,
    buffer_free: BufferFreeFn,
    _not_thread_safe: PhantomData<Rc<()>>,
}

/// Failures while loading or calling the byte-buffer adapter protocol.
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    /// The dynamic library could not be opened or lacks a required symbol.
    #[error("could not load Ferrum chemistry adapter: {0}")]
    Load(#[from] libloading::Error),

    /// The loaded adapter reports a different ABI version than its caller requires.
    #[error("Ferrum chemistry ABI mismatch: expected {expected}, found {actual}")]
    AbiMismatch { expected: u32, actual: u32 },

    /// The native adapter returned a non-null result length without a result pointer.
    #[error("Ferrum chemistry adapter returned a null buffer with length {length}")]
    NullBuffer { length: u64 },

    /// A native buffer length cannot be represented by this Rust process.
    #[error("Ferrum chemistry adapter returned an unrepresentable buffer length {length}")]
    BufferTooLarge { length: u64 },

    /// The adapter's release operation violated its ownership contract.
    #[error("Ferrum chemistry adapter did not clear its released buffer")]
    BufferFreeDidNotClear,

    /// The adapter returned a non-zero protocol status.
    #[error("Ferrum chemistry adapter execution failed with status {status}")]
    NativeStatus { status: u32 },
}

impl ChemistryAdapter {
    /// Opens the adapter at `library_path` and verifies its caller-selected ABI.
    ///
    /// The path is explicit so a process cannot accidentally load a system copy
    /// or a library discovered through a loader search path.
    pub fn load(library_path: &Path, expected_abi: u32) -> Result<Self, AdapterError> {
        // SAFETY: `library_path` is supplied explicitly by the caller.  Keeping
        // `Library` in this struct guarantees resolved function pointers remain
        // valid until after the last possible native call.
        let library = unsafe { Library::new(library_path) }?;
        let abi_version: AbiVersionFn = load_symbol(&library, b"ferrum_chem_abi_version\0")?;
        let kekulize: KekulizeFn = load_symbol(&library, b"ferrum_chem_kekulize_v1\0")?;
        let buffer_free: BufferFreeFn =
            load_symbol(&library, b"ferrum_chem_owned_buffer_free_v1\0")?;

        // SAFETY: the symbol was resolved from the retained library with the
        // exact C ABI signature declared by the Ferrum adapter contract.
        let actual_abi = unsafe { abi_version() };
        if actual_abi != expected_abi {
            return Err(AdapterError::AbiMismatch {
                expected: expected_abi,
                actual: actual_abi,
            });
        }

        Ok(Self {
            _library: library,
            abi_version,
            kekulize,
            buffer_free,
            _not_thread_safe: PhantomData,
        })
    }

    /// Returns the version reported by the already-validated native adapter.
    #[must_use]
    pub fn abi_version(&self) -> u32 {
        // SAFETY: construction resolved and validated this function pointer;
        // `_library` remains alive for the duration of `self`.
        unsafe { (self.abi_version)() }
    }

    /// Kekulizes an opaque version-one request and returns its owned response bytes.
    ///
    /// A native result buffer is always released through
    /// `ferrum_chem_owned_buffer_free_v1`,
    /// including when copying it into Rust allocation fails validation.
    pub fn kekulize(&self, input: &[u8]) -> Result<Vec<u8>, AdapterError> {
        let input_length = u64::try_from(input.len())
            .map_err(|_| AdapterError::BufferTooLarge { length: u64::MAX })?;
        let mut output = FerrumChemOwnedBuffer {
            data: std::ptr::null_mut(),
            len: 0,
        };

        // SAFETY: `input` remains borrowed for the call; the two output locations
        // is a valid writable stack slot; function pointer and ABI were validated
        // at construction and the library remains retained by `self`.
        let status = unsafe { (self.kekulize)(input.as_ptr(), input_length, &mut output) };

        finish_call(status, output, self.buffer_free)
    }
}

fn load_symbol<T: Copy>(library: &Library, name: &'static [u8]) -> Result<T, AdapterError> {
    // SAFETY: each requested symbol is part of the fixed Ferrum C ABI.  The
    // returned function pointer is copied while `ChemistryAdapter` retains the
    // owning `Library`, so it cannot outlive the dynamic library.
    Ok(unsafe { *library.get::<T>(name)? })
}

#[derive(Debug)]
struct OutputBuffer {
    native: FerrumChemOwnedBuffer,
    pointer: Option<NonNull<u8>>,
    length: usize,
    buffer_free: BufferFreeFn,
    released: bool,
}

impl OutputBuffer {
    fn new(
        mut native: FerrumChemOwnedBuffer,
        buffer_free: BufferFreeFn,
    ) -> Result<Self, AdapterError> {
        let (pointer, length) = match validate_native_buffer(native) {
            Ok(validated) => validated,
            Err(error) => {
                release_native(&mut native, buffer_free)?;
                return Err(error);
            }
        };
        Ok(Self {
            native,
            pointer,
            length,
            buffer_free,
            released: false,
        })
    }

    fn into_vec(mut self) -> Result<Vec<u8>, AdapterError> {
        let Some(pointer) = self.pointer else {
            self.release()?;
            return Ok(Vec::new());
        };
        // SAFETY: `OutputBuffer` owns this native allocation until `Drop`; its
        // length was validated for `usize`, and the adapter contract promises a
        // readable buffer of exactly `length` bytes.
        let output = unsafe { std::slice::from_raw_parts(pointer.as_ptr(), self.length) }.to_vec();
        self.release()?;
        Ok(output)
    }

    fn release(&mut self) -> Result<(), AdapterError> {
        if self.released {
            return Ok(());
        }
        self.released = true;
        release_native(&mut self.native, self.buffer_free)
    }
}

impl Drop for OutputBuffer {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

fn finish_call(
    status: u32,
    native: FerrumChemOwnedBuffer,
    buffer_free: BufferFreeFn,
) -> Result<Vec<u8>, AdapterError> {
    let mut output = OutputBuffer::new(native, buffer_free)?;
    if status != 0 {
        output.release()?;
        return Err(AdapterError::NativeStatus { status });
    }
    output.into_vec()
}

fn validate_native_buffer(
    native: FerrumChemOwnedBuffer,
) -> Result<(Option<NonNull<u8>>, usize), AdapterError> {
    let pointer = NonNull::new(native.data);
    let length = checked_length(native.len)?;
    if pointer.is_none() && length != 0 {
        return Err(AdapterError::NullBuffer { length: native.len });
    }
    Ok((pointer, length))
}

fn release_native(
    native: &mut FerrumChemOwnedBuffer,
    buffer_free: BufferFreeFn,
) -> Result<(), AdapterError> {
    // SAFETY: `native` is an adapter-owned output structure supplied by the
    // matching `kekulize_v1` call. The adapter contract makes its release
    // operation valid for this structure and requires it to clear both fields.
    unsafe { buffer_free(native) };
    if native.data.is_null() && native.len == 0 {
        Ok(())
    } else {
        *native = FerrumChemOwnedBuffer {
            data: std::ptr::null_mut(),
            len: 0,
        };
        Err(AdapterError::BufferFreeDidNotClear)
    }
}

fn checked_length(length: u64) -> Result<usize, AdapterError> {
    usize::try_from(length).map_err(|_| AdapterError::BufferTooLarge { length })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static FREE_CALLS: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn accepts_empty_null_output() {
        assert!(matches!(checked_length(0), Ok(0)));
    }

    #[test]
    fn rejects_null_nonempty_output_before_reading_memory() {
        let error = OutputBuffer::new(
            FerrumChemOwnedBuffer {
                data: std::ptr::null_mut(),
                len: 1,
            },
            unused_buffer_free,
        )
        .expect_err("a non-empty buffer must have a pointer");
        assert!(matches!(error, AdapterError::NullBuffer { length: 1 }));
    }

    #[test]
    fn nonzero_status_releases_the_owned_response_once() {
        FREE_CALLS.store(0, Ordering::Relaxed);
        let mut response = [9_u8, 8];
        let error = finish_call(
            17,
            FerrumChemOwnedBuffer {
                data: response.as_mut_ptr(),
                len: u64::try_from(response.len()).expect("test response length fits u64"),
            },
            clearing_buffer_free,
        )
        .expect_err("a nonzero call status must be returned");

        assert!(matches!(error, AdapterError::NativeStatus { status: 17 }));
        assert_eq!(FREE_CALLS.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn reports_a_release_operation_that_fails_to_clear_ownership() {
        let mut response = [7_u8];
        let error = finish_call(
            1,
            FerrumChemOwnedBuffer {
                data: response.as_mut_ptr(),
                len: 1,
            },
            nonclearing_buffer_free,
        )
        .expect_err("the release contract must clear the buffer structure");
        assert!(matches!(error, AdapterError::BufferFreeDidNotClear));
    }

    #[test]
    fn rejects_lengths_outside_usize() {
        if usize::BITS < u64::BITS {
            let error = checked_length(u64::MAX).expect_err("length must fit usize");
            assert!(matches!(
                error,
                AdapterError::BufferTooLarge { length: u64::MAX }
            ));
        }
    }

    unsafe extern "C" fn unused_buffer_free(output: *mut FerrumChemOwnedBuffer) {
        // SAFETY: the pointer is a valid mutable buffer structure passed by the unit test.
        unsafe {
            (*output).data = std::ptr::null_mut();
            (*output).len = 0;
        }
    }

    unsafe extern "C" fn clearing_buffer_free(output: *mut FerrumChemOwnedBuffer) {
        FREE_CALLS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: each test passes a valid mutable stack-owned buffer structure.
        unsafe {
            (*output).data = std::ptr::null_mut();
            (*output).len = 0;
        }
    }

    unsafe extern "C" fn nonclearing_buffer_free(_: *mut FerrumChemOwnedBuffer) {}
}
