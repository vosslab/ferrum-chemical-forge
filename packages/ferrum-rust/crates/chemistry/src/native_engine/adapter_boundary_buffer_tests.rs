//! Deterministic ownership-boundary tests for adapter output buffers.

use std::sync::atomic::{AtomicUsize, Ordering};

use super::AdapterError;
use super::adapter_boundary_buffer::{
    FerrumChemOwnedBuffer, OutputBuffer, checked_length, finish_call,
};
use crate::FERRUM_CHEM_MAX_RESPONSE_BYTES;

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

#[test]
fn rejects_an_oversized_foreign_length_before_forming_a_slice() {
    let error = checked_length((FERRUM_CHEM_MAX_RESPONSE_BYTES + 1) as u64)
        .expect_err("the public output limit is checked before any foreign read");
    assert!(matches!(error, AdapterError::ResponseTooLarge { .. }));
}

unsafe extern "C" fn unused_buffer_free(output: *mut FerrumChemOwnedBuffer) {
    // SAFETY: this test passes a valid mutable stack-owned buffer structure.
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
