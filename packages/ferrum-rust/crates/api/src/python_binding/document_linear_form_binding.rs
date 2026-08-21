//! Private PyO3 boundary for authenticated direct-root linear-form conversion.
//!
//! This unsupported extension entry point is for the bundled Ferrum native
//! route only. It deliberately has no wheel stub, CLI, serde, or wire surface.

use ferrum_document::{
    convert_document_linear_form_v1 as convert_rust_document_linear_form_v1,
    DocumentLinearFormErrorV1, DocumentLinearFormRequestV1,
};
use ferrum_document::{
    DocumentObjectIdV1, DocumentSession, DocumentSessionError, PersistentId, SessionOperationError,
    SessionOperationResultV1,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString, PyTuple};

use super::binding::FerrumError;

create_exception!(ferrum_chem, DocumentLinearFormError, FerrumError);

const DIGEST_REASON: &str = "expected digest must be exactly 64 lowercase hexadecimal characters";
const DIGEST_TEXT_REASON: &str = "expected digest must be valid UTF-8 text";
const RESOURCE_REASON: &str = "linear-form conversion could not reserve input storage";
const ROOT_TEXT_REASON: &str = "molecule selector must be valid UTF-8 text";
const SELECTION_SHAPE_REASON: &str = "selected atom IDs must be an exact built-in tuple";
const SELECTION_TEXT_REASON: &str = "selected atom ID must be valid UTF-8 text";

pub(crate) fn convert_linear_form_v1(
    py: Python<'_>,
    session: &mut DocumentSession,
    expected_revision: u64,
    expected_digest: &Bound<'_, PyString>,
    molecule_id: &Bound<'_, PyString>,
    selected_atom_ids: &Bound<'_, PyAny>,
) -> PyResult<SessionOperationResultV1> {
    let expected_digest = match expected_digest.to_str() {
        Ok(value) => value,
        Err(_) => return Err(linear_form_error(py, DIGEST_TEXT_REASON)?),
    };
    let expected_digest = parse_digest(py, expected_digest)?;
    let molecule_id = match molecule_id.to_str() {
        Ok(value) => value,
        Err(_) => return Err(linear_form_error(py, ROOT_TEXT_REASON)?),
    };
    let molecule_id = copied(py, molecule_id)?;
    let molecule_id = match DocumentObjectIdV1::parse(molecule_id) {
        Ok(value) => value,
        Err(error) => return Err(linear_form_error(py, error.to_string())?),
    };
    let selected_atom_ids = atom_ids(py, selected_atom_ids)?;
    let request = DocumentLinearFormRequestV1::new(
        expected_revision,
        expected_digest,
        molecule_id,
        selected_atom_ids,
    );
    match convert_rust_document_linear_form_v1(session, request) {
        Ok(result) => Ok(result.into_operation_result()),
        Err(DocumentLinearFormErrorV1::Observation(error)) => {
            Err(linear_form_error(py, error.to_string())?)
        }
        Err(DocumentLinearFormErrorV1::Session(DocumentSessionError::Operation(error)))
            if is_linear_form_operation_error(&error) =>
        {
            Err(selection_error(py, error)?)
        }
        Err(DocumentLinearFormErrorV1::Session(error)) => Err(
            super::document_error_binding::map_document_error(py, error)?,
        ),
    }
}

fn atom_ids(py: Python<'_>, values: &Bound<'_, PyAny>) -> PyResult<Vec<PersistentId>> {
    if !values.is_exact_instance_of::<PyTuple>() {
        return Err(linear_form_error(py, SELECTION_SHAPE_REASON)?);
    }
    let values = values.cast::<PyTuple>()?;
    let mut atom_ids = Vec::new();
    if atom_ids.try_reserve_exact(values.len()).is_err() {
        return Err(linear_form_error(py, RESOURCE_REASON)?);
    }
    for value in values.iter() {
        if !value.is_exact_instance_of::<PyString>() {
            return Err(linear_form_error(py, SELECTION_SHAPE_REASON)?);
        }
        let string = value.cast::<PyString>()?;
        let value = match string.to_str() {
            Ok(value) => value,
            Err(_) => return Err(linear_form_error(py, SELECTION_TEXT_REASON)?),
        };
        let value = copied(py, value)?;
        let atom_id = match PersistentId::new(value) {
            Ok(value) => value,
            Err(error) => return Err(linear_form_error(py, error.to_string())?),
        };
        atom_ids.push(atom_id);
    }
    Ok(atom_ids)
}

fn selection_error(py: Python<'_>, error: SessionOperationError) -> PyResult<PyErr> {
    linear_form_error(py, error.to_string())
}

fn is_linear_form_operation_error(error: &SessionOperationError) -> bool {
    matches!(
        error,
        SessionOperationError::EmptyLinearFormSelection
            | SessionOperationError::LinearFormPlan(_)
            | SessionOperationError::HistoryResourceExhausted
            | SessionOperationError::FragmentIdentifierExhausted
            | SessionOperationError::GeneratedIdentifierAllocationFailed
            | SessionOperationError::Candidate(_)
    )
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(linear_form_error(py, DIGEST_REASON)?);
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
        return Err(linear_form_error(py, RESOURCE_REASON)?);
    }
    result.push_str(value);
    Ok(result)
}

fn linear_form_error(py: Python<'_>, reason: impl AsRef<str>) -> PyResult<PyErr> {
    let reason = reason.as_ref();
    let message = owned_reason(reason)?;
    let attribute = owned_reason(reason)?;
    let error = DocumentLinearFormError::new_err(message);
    error.value(py).setattr("reason", attribute)?;
    Ok(error)
}

fn owned_reason(reason: &str) -> PyResult<String> {
    let mut owned = String::new();
    owned
        .try_reserve_exact(reason.len())
        .map_err(|_| DocumentLinearFormError::new_err(RESOURCE_REASON))?;
    owned.push_str(reason);
    Ok(owned)
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentLinearFormError",
        module.py().get_type::<DocumentLinearFormError>(),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_form_resource_failures_keep_the_private_error_route() {
        for error in [
            SessionOperationError::HistoryResourceExhausted,
            SessionOperationError::FragmentIdentifierExhausted,
            SessionOperationError::GeneratedIdentifierAllocationFailed,
        ] {
            assert!(is_linear_form_operation_error(&error), "{error}");
        }
    }
}
