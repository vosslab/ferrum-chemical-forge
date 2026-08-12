use std::ffi::CStr;
use std::os::raw::c_char;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

const SUPPORTED_ADAPTER_ABI_VERSION: u32 = 1;

#[link(name = "ferrum_chem")]
unsafe extern "C" {
    fn ferrum_chem_abi_version() -> u32;
    fn ferrum_chem_build_marker() -> *const c_char;
}

fn adapter_probe() -> PyResult<(u32, String)> {
    // The C header promises scalar output and static, NUL-terminated marker storage.
    let abi_version = unsafe { ferrum_chem_abi_version() };
    ensure_supported_abi_version(abi_version)?;
    let marker = unsafe { ferrum_chem_build_marker() };
    if marker.is_null() {
        return Err(PyRuntimeError::new_err(
            "Ferrum-Chem returned a null build marker",
        ));
    }
    let marker = unsafe { CStr::from_ptr(marker) }
        .to_str()
        .map_err(|error| {
            PyRuntimeError::new_err(format!("Ferrum-Chem marker is not UTF-8: {error}"))
        })?
        .to_owned();
    Ok((abi_version, marker))
}

fn ensure_supported_abi_version(abi_version: u32) -> PyResult<()> {
    if abi_version != SUPPORTED_ADAPTER_ABI_VERSION {
        return Err(PyRuntimeError::new_err(format!(
            "Ferrum-Chem adapter ABI {abi_version} is unsupported; this Ferrum API requires ABI {SUPPORTED_ADAPTER_ABI_VERSION}"
        )));
    }
    Ok(())
}

#[pyfunction]
fn probe() -> PyResult<(u32, String)> {
    adapter_probe()
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(probe, module)?)
}

#[cfg(test)]
mod tests {
    use super::{SUPPORTED_ADAPTER_ABI_VERSION, adapter_probe, ensure_supported_abi_version};

    #[test]
    fn probe_exposes_the_adapter_contract() {
        let (abi_version, marker) = adapter_probe().expect("adapter probe succeeds");
        assert_eq!(abi_version, 1);
        assert!(!marker.is_empty());
    }

    #[test]
    fn unsupported_adapter_abi_is_rejected_before_marker_use() {
        let error = ensure_supported_abi_version(SUPPORTED_ADAPTER_ABI_VERSION + 1)
            .expect_err("future adapter ABI must be rejected");
        assert!(error.to_string().contains("requires ABI 1"));
    }
}
