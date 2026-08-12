use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

mod adapter_abi;

include!(concat!(env!("OUT_DIR"), "/ferrum_chem_adapter_abi.rs"));

#[link(name = "ferrum_chem")]
unsafe extern "C" {
    fn ferrum_chem_abi_version() -> u32;
}

fn adapter_probe() -> PyResult<u32> {
    // ABI version is the sole PyO3 loader-contract probe. Chemistry operations
    // remain on the Rust boundary rather than growing test-only C exports.
    let abi_version = unsafe { ferrum_chem_abi_version() };
    ensure_supported_abi_version(abi_version)?;
    Ok(abi_version)
}

fn ensure_supported_abi_version(abi_version: u32) -> PyResult<()> {
    adapter_abi::ensure_supported_adapter_abi_version(abi_version, SUPPORTED_ADAPTER_ABI_VERSION)
        .map_err(|error| PyRuntimeError::new_err(error.to_string()))
}

#[pyfunction]
fn probe() -> PyResult<u32> {
    adapter_probe()
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(probe, module)?)
}
