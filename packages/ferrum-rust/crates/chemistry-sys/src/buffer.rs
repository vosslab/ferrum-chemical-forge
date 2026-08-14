//! Adapter-owned buffer validation, copying, and deterministic release.

use std::ptr::NonNull;

use crate::contract::{AdapterError, BufferFreeFn, FERRUM_CHEM_MAX_RESPONSE_BYTES};

/// Layout-compatible representation of the ABI-4 `ferrum_chem_owned_buffer`.
///
/// This type stays crate-private so safe callers cannot manufacture a foreign
/// allocation or bypass its matching adapter release function.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct FerrumChemOwnedBuffer {
    pub(crate) data: *mut u8,
    pub(crate) len: u64,
}

/// Owns one adapter allocation until it is copied and released.
#[derive(Debug)]
pub(crate) struct OutputBuffer {
    native: FerrumChemOwnedBuffer,
    pointer: Option<NonNull<u8>>,
    length: usize,
    buffer_free: BufferFreeFn,
    released: bool,
}

impl OutputBuffer {
    /// Validates an adapter-owned result before any foreign memory is read.
    pub(crate) fn new(
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

    /// Copies validated foreign bytes into an owned Rust allocation and releases them.
    pub(crate) fn into_vec(mut self) -> Result<Vec<u8>, AdapterError> {
        let Some(pointer) = self.pointer else {
            self.release()?;
            return Ok(Vec::new());
        };
        // SAFETY: the adapter produced an allocation readable for exactly
        // `length` bytes. `new` checked the byte limit, `usize` conversion, and
        // non-null relationship before this slice is formed. This object holds
        // the matching release callback and releases only after `to_vec` copies
        // the bytes. The adapter is retained by `ChemistryAdapter`, so its
        // function pointer and allocation contract remain live. The adapter is
        // deliberately !Send/!Sync, so this call cannot move across threads.
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

/// Turns a completed ABI call into owned Rust bytes.
pub(crate) fn finish_call(
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
    // SAFETY: `native` has the exact `repr(C)` layout required by ABI-4 and was
    // initialized by the matching adapter operation. Its allocation and its
    // release callback come from the same still-loaded library. The callback
    // receives an exclusive, valid pointer to this stack owner, is expected to
    // free the allocation exactly once, and must clear both fields. Calls stay
    // on the creating thread because `ChemistryAdapter` is !Send/!Sync.
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

pub(crate) fn checked_length(length: u64) -> Result<usize, AdapterError> {
    if length > FERRUM_CHEM_MAX_RESPONSE_BYTES {
        return Err(AdapterError::ResponseTooLarge {
            length,
            maximum: FERRUM_CHEM_MAX_RESPONSE_BYTES,
        });
    }
    usize::try_from(length).map_err(|_| AdapterError::BufferTooLarge { length })
}
