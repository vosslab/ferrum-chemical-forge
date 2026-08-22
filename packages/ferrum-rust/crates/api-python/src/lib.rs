//! Python ABI entrypoint for Ferrum's staged local runtime.
//!
//! This crate deliberately owns only the CPython extension symbol.  The
//! `ferrum-api` crate owns every protocol operation and validates the
//! package-local `.dylibs/libferrum_chem.dylib` runtime closure.

use pyo3::{prelude::*, types::PyModule};

/// Export the sole Python module ABI entrypoint.
#[pymodule]
fn ferrum_chem(module: &Bound<'_, PyModule>) -> PyResult<()> {
    ferrum_api::initialize_python_extension_v1(module)
}
