//! Frozen Python boundary for Ferrum's stateless operation protocol V1.
//!
//! The binding owns a caller-independent copy of the request text, releases
//! Python only while the pure Rust executor runs, and returns response-owned
//! JSON. It deliberately exposes no session, receipt, path, or DTO surface.

use crate::{
    OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1, OperationProtocolEnvelopeV1,
    OperationProtocolInputErrorV1,
    operation_protocol_schema_v1 as rust_operation_protocol_schema_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::binding::FerrumError;

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
    let envelope = super::super::execute_operation_from_staged_extension_v1(&request_json);
    let envelope = envelope.map_err(|error| protocol_input_error(py, error))?;
    serialize_protocol_envelope_v1(py, envelope)
}

fn serialize_protocol_envelope_v1(
    py: Python<'_>,
    envelope: OperationProtocolEnvelopeV1,
) -> PyResult<String> {
    let bytes = crate::protocol::canonical_protocol_envelope_json_v1(&envelope)
        .map_err(|error| protocol_error(py, EXECUTION_UNAVAILABLE, error.to_string()))?;
    String::from_utf8(bytes)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
        protocol::{
            execute_operation_with_runtime_v1,
            runtime::{ChemistryRuntimeErrorV1, ChemistryRuntimeV1},
        },
    };
    use ferrum_chemistry::{ChemEngine, ChemistryError};

    const HOSTILE_RUNTIME_DETAIL: &str = "/private/ferrum/.dylibs/libferrum_chem.dylib: private_native_adapter dlopen native loader text";
    const CDML: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom></molecule></cdml>";

    struct HostileRuntime;

    impl ChemistryRuntimeV1 for HostileRuntime {
        fn with_engine<T>(
            &self,
            _operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
        ) -> Result<T, ChemistryRuntimeErrorV1> {
            Err(ChemistryRuntimeErrorV1::Chemistry(
                ChemistryError::NativeBoundary {
                    reason: HOSTILE_RUNTIME_DETAIL.to_owned(),
                },
            ))
        }
    }

    #[test]
    fn python_protocol_serialization_redacts_hostile_runtime_failures() {
        Python::initialize();
        Python::attach(|py| {
            for (request, expected_category) in [
                (
                    serde_json::json!({
                    "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
                    "request_id": "python-hostile-convert",
                    "operation": {
                        "kind": "chemistry.convert",
                        "input": {"format": "smiles", "text": "CCO"},
                        "output_format": "inchi_standard",
                    },
                    }),
                    "chemistry_unavailable",
                ),
                (
                    serde_json::json!({
                    "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
                    "request_id": "python-hostile-coordinates",
                    "operation": {"kind": "document.generate_coordinates", "document": CDML},
                    }),
                    "chemistry_unavailable",
                ),
            ] {
                let envelope =
                    execute_operation_with_runtime_v1(&request.to_string(), &HostileRuntime)
                        .expect("request decodes");
                let json = serialize_protocol_envelope_v1(py, envelope)
                    .expect("Python bridge serializes protocol envelope");
                let value: serde_json::Value = serde_json::from_str(&json).expect("response JSON");
                assert_eq!(value["request_id"], request["request_id"]);
                assert_eq!(value["error"]["category"], expected_category);
                assert_eq!(
                    value["error"]["message"],
                    "Ferrum chemistry runtime is unavailable"
                );
                for private_detail in [
                    HOSTILE_RUNTIME_DETAIL,
                    ".dylibs",
                    "libferrum_chem",
                    "private_native_adapter",
                    "dlopen",
                ] {
                    assert!(!json.contains(private_detail));
                }
            }
        });
    }
}
