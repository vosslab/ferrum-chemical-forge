//! Frozen Python boundary for Ferrum's stateless operation protocol V1.
//!
//! The binding owns a caller-independent copy of the request text, releases
//! Python only while the pure Rust executor runs, and returns response-owned
//! JSON. It deliberately exposes no session, receipt, path, or DTO surface.

use ferrum_api::{
    OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1, OperationProtocolInputErrorV1,
    execute_operation_with_runtime_v1 as execute_rust_operation_with_runtime_v1,
    operation_protocol_schema_v1 as rust_operation_protocol_schema_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use crate::binding::FerrumError;

create_exception!(ferrum_chem, OperationProtocolErrorV1, FerrumError);

const INVALID_JSON: &str = "invalid_json";
const RESOURCE_LIMIT: &str = "resource_limit";
const EXECUTION_UNAVAILABLE: &str = "execution_unavailable";

/// Execute one stateless V1 JSON request and return its JSON response envelope.
///
/// A syntactically decodable domain or version refusal is protocol data. Only
/// malformed JSON, a transport-budget refusal, and a failure to serialize an
/// otherwise valid envelope raise this binding's categorized exception.
#[pyfunction]
fn execute_operation_v1(py: Python<'_>, request_json: &str) -> PyResult<String> {
    let observed = request_json.len();
    if observed > OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1 {
        return Err(protocol_error(
            py,
            RESOURCE_LIMIT,
            OperationProtocolInputErrorV1::ResourceLimit {
                limit: OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1,
                observed,
            }
            .to_string(),
        ));
    }
    let request_json = request_json.to_owned();
    let runtime = crate::chemistry_binding::packaged_protocol_runtime();
    let envelope =
        py.detach(move || execute_rust_operation_with_runtime_v1(&request_json, &runtime));
    let envelope = envelope.map_err(|error| protocol_input_error(py, error))?;
    serde_json::to_string(&envelope)
        .map_err(|error| protocol_error(py, EXECUTION_UNAVAILABLE, error.to_string()))
}

fn protocol_input_error(py: Python<'_>, error: OperationProtocolInputErrorV1) -> PyErr {
    let category = match &error {
        OperationProtocolInputErrorV1::ResourceLimit { .. } => RESOURCE_LIMIT,
        OperationProtocolInputErrorV1::InvalidJson(_) => INVALID_JSON,
    };
    protocol_error(py, category, error.to_string())
}

/// Return the checked-in V1 protocol schema packaged with Ferrum distributions.
#[pyfunction]
fn operation_protocol_schema_v1() -> &'static str {
    rust_operation_protocol_schema_v1()
}

fn protocol_error(py: Python<'_>, category: &str, message: String) -> PyErr {
    let error = OperationProtocolErrorV1::new_err(message);
    if let Err(attribute_error) = error.value(py).setattr("category", category) {
        return attribute_error;
    }
    error
}

/// Register the frozen protocol API and no protocol-shaped convenience aliases.
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "OperationProtocolErrorV1",
        module.py().get_type::<OperationProtocolErrorV1>(),
    )?;
    module.add_function(wrap_pyfunction!(execute_operation_v1, module)?)?;
    module.add_function(wrap_pyfunction!(operation_protocol_schema_v1, module)?)?;
    Ok(())
}
