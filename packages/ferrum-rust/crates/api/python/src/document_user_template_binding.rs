//! Private worker-safe inspection and authenticated user-template insertion.
//!
//! This unsupported entry point belongs only to bundled Ferrum. It has no
//! wheel stub, CLI, serde, or wire-format commitment.

use ferrum_document::DocumentSession;
use ferrum_document::{
    DOCUMENT_USER_TEMPLATE_PROFILE_V1, DocumentUserTemplateApplyErrorV1,
    DocumentUserTemplatePlanV1, apply_user_template_v1, prepare_user_template_v1,
};
use ferrum_geometry::Point2;
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString};

use crate::binding::{FerrumError, PySessionOperationResultV1};

create_exception!(ferrum_chem, DocumentUserTemplateError, FerrumError);

const DIGEST_REASON: &str = "expected digest must be exactly 64 lowercase hexadecimal characters";
const DIGEST_TEXT_REASON: &str = "expected digest must be valid UTF-8 text";
const RESOURCE_REASON: &str = "user template operation could not reserve Python result storage";
const SOURCE_REASON: &str = "user template source must be an exact built-in string";
const SOURCE_TEXT_REASON: &str = "user template source must be valid UTF-8 text";

/// One immutable handle-free plan safe to deliver from a Qt worker.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentUserTemplatePlanV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentUserTemplatePlanV1 {
    plan: DocumentUserTemplatePlanV1,
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    profile: String,
    #[pyo3(get)]
    display_name: Option<String>,
    #[pyo3(get)]
    atom_centroid_x: f64,
    #[pyo3(get)]
    atom_centroid_y: f64,
}

/// One committed observation plus the inserted durable molecule identity.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentUserTemplateResultV1",
    skip_from_py_object
)]
pub(crate) struct PyDocumentUserTemplateResultV1 {
    #[pyo3(get)]
    operation: PySessionOperationResultV1,
    #[pyo3(get)]
    inserted_molecule_object_id: String,
    #[pyo3(get)]
    inserted_molecule_source_id: String,
}

/// Admit one exact saved-template string under the named product profile.
#[pyfunction(name = "prepare_user_template_v1")]
fn prepare_user_template_v1_binding(
    py: Python<'_>,
    source: &Bound<'_, PyAny>,
) -> PyResult<PyDocumentUserTemplatePlanV1> {
    if !source.is_exact_instance_of::<PyString>() {
        return Err(template_error(py, SOURCE_REASON)?);
    }
    let source = source.cast::<PyString>()?;
    let source = match source.to_str() {
        Ok(source) => source,
        Err(_) => return Err(template_error(py, SOURCE_TEXT_REASON)?),
    };
    let source = copied(py, source)?;
    let plan = match py.detach(move || prepare_user_template_v1(&source)) {
        Ok(plan) => plan,
        Err(error) => return Err(template_error(py, error.to_string())?),
    };
    let schema = copied(py, plan.schema())?;
    let profile = copied(py, DOCUMENT_USER_TEMPLATE_PROFILE_V1)?;
    let display_name = plan
        .display_name()
        .map(|name| copied(py, name))
        .transpose()?;
    let centroid = plan.atom_centroid();
    Ok(PyDocumentUserTemplatePlanV1 {
        plan,
        schema,
        profile,
        display_name,
        atom_centroid_x: centroid.x(),
        atom_centroid_y: centroid.y(),
    })
}

pub(crate) fn apply_user_template_v1_binding(
    py: Python<'_>,
    session: &mut DocumentSession,
    expected_revision: u64,
    expected_digest: &Bound<'_, PyString>,
    prepared: PyRef<'_, PyDocumentUserTemplatePlanV1>,
    anchor_x: f64,
    anchor_y: f64,
) -> PyResult<PyDocumentUserTemplateResultV1> {
    let expected_digest = match expected_digest.to_str() {
        Ok(digest) => digest,
        Err(_) => return Err(template_error(py, DIGEST_TEXT_REASON)?),
    };
    let expected_digest = parse_digest(py, expected_digest)?;
    let anchor = match Point2::new(anchor_x, anchor_y) {
        Ok(anchor) => anchor,
        Err(error) => return Err(template_error(py, error.to_string())?),
    };
    let result = match apply_user_template_v1(
        session,
        expected_revision,
        &expected_digest,
        &prepared.plan,
        anchor,
    ) {
        Ok(result) => result,
        Err(DocumentUserTemplateApplyErrorV1::Session(error)) => {
            return Err(template_error(py, error.to_string())?);
        }
    };
    let inserted_molecule_object_id = copied(py, result.inserted_molecule().object_id().as_str())?;
    let inserted_molecule_source_id = copied(py, result.inserted_molecule().source_id().as_str())?;
    Ok(PyDocumentUserTemplateResultV1 {
        operation: result.into_operation_result().into(),
        inserted_molecule_object_id,
        inserted_molecule_source_id,
    })
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(template_error(py, DIGEST_REASON)?);
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
        return Err(template_error(py, RESOURCE_REASON)?);
    }
    result.push_str(value);
    Ok(result)
}

fn template_error(py: Python<'_>, reason: impl AsRef<str>) -> PyResult<PyErr> {
    let reason = reason.as_ref();
    let message = owned_reason(reason)?;
    let attribute = owned_reason(reason)?;
    let error = DocumentUserTemplateError::new_err(message);
    error.value(py).setattr("reason", attribute)?;
    Ok(error)
}

fn owned_reason(reason: &str) -> PyResult<String> {
    let mut owned = String::new();
    owned
        .try_reserve_exact(reason.len())
        .map_err(|_| DocumentUserTemplateError::new_err(RESOURCE_REASON))?;
    owned.push_str(reason);
    Ok(owned)
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentUserTemplateError",
        module.py().get_type::<DocumentUserTemplateError>(),
    )?;
    module.add_class::<PyDocumentUserTemplatePlanV1>()?;
    module.add_class::<PyDocumentUserTemplateResultV1>()?;
    module.add_function(wrap_pyfunction!(prepare_user_template_v1_binding, module)?)?;
    Ok(())
}
