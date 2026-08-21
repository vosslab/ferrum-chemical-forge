//! Private PyO3 boundary for exact direct-root molecule-name mutation.
//!
//! This unsupported extension entry point is for the bundled Ferrum native
//! route only. It deliberately has no wheel stub, CLI, serde, or wire surface.

use ferrum_document::{
    set_document_molecule_name_v1 as set_rust_document_molecule_name_v1,
    DocumentMoleculeNameErrorV1, DocumentMoleculeNameRequestV1,
};
use ferrum_document::{DocumentObjectIdV1, DocumentSession, SessionOperationResultV1};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyString;

use super::binding::FerrumError;

create_exception!(ferrum_chem, DocumentMoleculeNameError, FerrumError);

const DIGEST_REASON: &str = "expected digest must be exactly 64 lowercase hexadecimal characters";
const DIGEST_TEXT_REASON: &str = "expected digest must be valid UTF-8 text";
const NAME_TEXT_REASON: &str = "molecule name must be valid UTF-8 text";
const RESOURCE_REASON: &str = "molecule name operation could not reserve input storage";
const SELECTOR_TEXT_REASON: &str = "molecule selector must be valid UTF-8 text";

pub(crate) fn set_document_molecule_name_v1(
    py: Python<'_>,
    session: &mut DocumentSession,
    expected_revision: u64,
    expected_digest: &Bound<'_, PyString>,
    molecule_id: &Bound<'_, PyString>,
    name: &Bound<'_, PyString>,
) -> PyResult<SessionOperationResultV1> {
    let expected_digest = expected_digest
        .to_str()
        .map_err(|_| name_error(py, DIGEST_TEXT_REASON))?;
    let expected_digest = parse_digest(py, expected_digest)?;
    let molecule_id = molecule_id
        .to_str()
        .map_err(|_| name_error(py, SELECTOR_TEXT_REASON))?;
    let molecule_id = copied(py, molecule_id)?;
    let molecule_id = DocumentObjectIdV1::parse(molecule_id)
        .map_err(|error| name_error(py, error.to_string()))?;
    let name = name
        .to_str()
        .map_err(|_| name_error(py, NAME_TEXT_REASON))?;
    let name = copied(py, name)?;
    let request =
        DocumentMoleculeNameRequestV1::new(expected_revision, expected_digest, molecule_id, name);
    match set_rust_document_molecule_name_v1(session, request) {
        Ok(result) => Ok(result),
        Err(DocumentMoleculeNameErrorV1::Observation(error)) => {
            Err(name_error(py, error.to_string()))
        }
        Err(DocumentMoleculeNameErrorV1::Session(error)) => Err(
            super::document_error_binding::map_document_error(py, error)?,
        ),
    }
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(name_error(py, DIGEST_REASON));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        digest[index] = (hex_value(pair[0]) << 4) | hex_value(pair[1]);
    }
    Ok(digest)
}

const fn hex_value(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}

fn copied(py: Python<'_>, value: &str) -> PyResult<String> {
    let mut result = String::new();
    if result.try_reserve_exact(value.len()).is_err() {
        return Err(name_error(py, RESOURCE_REASON));
    }
    result.push_str(value);
    Ok(result)
}

fn name_error(py: Python<'_>, reason: impl Into<String>) -> PyErr {
    let reason = reason.into();
    let error = DocumentMoleculeNameError::new_err(reason.clone());
    if let Err(attribute_error) = error.value(py).setattr("reason", reason) {
        return attribute_error;
    }
    error
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentMoleculeNameError",
        module.py().get_type::<DocumentMoleculeNameError>(),
    )?;
    Ok(())
}
