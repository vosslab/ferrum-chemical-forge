use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

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
    if abi_version != SUPPORTED_ADAPTER_ABI_VERSION {
        return Err(PyRuntimeError::new_err(format!(
            "Ferrum-Chem adapter ABI {abi_version} is unsupported; this Ferrum API requires ABI {SUPPORTED_ADAPTER_ABI_VERSION}"
        )));
    }
    Ok(())
}

#[pyfunction]
fn probe() -> PyResult<u32> {
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
    fn probe_exposes_the_adapter_abi_contract() {
        let abi_version = adapter_probe().expect("adapter probe succeeds");
        assert_eq!(abi_version, SUPPORTED_ADAPTER_ABI_VERSION);
    }

    #[test]
    fn unsupported_adapter_abi_is_rejected() {
        let deliberately_different_version = SUPPORTED_ADAPTER_ABI_VERSION
            .checked_add(1)
            .expect("supported adapter ABI permits a distinct test version");
        let error = ensure_supported_abi_version(deliberately_different_version)
            .expect_err("future adapter ABI must be rejected");
        let required_abi = format!("requires ABI {SUPPORTED_ADAPTER_ABI_VERSION}");
        assert!(error.to_string().contains(&required_abi));
    }
}
