//! Private whole-document artifact bridge for the bundled Ferrum window.
//!
//! These entry points intentionally stay out of the wheel stub, CLI, serde, and
//! wire surfaces.  They accept only an immutable Rust observation and closed
//! profiles, then delegate all destination handling to the Rust publisher.

use std::path::PathBuf;

use ferrum_document::{
    DocumentNativeArtifactErrorV1, DocumentNativeArtifactProfileV1,
    PreparedDocumentNativeArtifactV1, artifact_publication_v1::ArtifactPublicationDurabilityV1,
    prepare_document_native_artifact_v1 as prepare_native_artifact,
    publish_prepared_document_native_artifact_v1 as publish_native_artifact,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyString;

use super::binding::FerrumError;
use super::document_error_binding::publication_error;
use super::document_ingress_binding::PyLocalDocumentOriginTokenV1;
use super::projection_binding::PySessionDocumentObservationV1;

create_exception!(ferrum_chem, DocumentNativeArtifactError, FerrumError);

const DIGEST_REASON: &str = "expected digest must be exactly 64 lowercase hexadecimal characters";
const DIGEST_TEXT_REASON: &str = "expected digest must be valid UTF-8 text";
const PROFILE_REASON: &str =
    "native artifact profile must be svg, pdf, or png_one_pixel_per_point_transparent";
const RESOURCE_REASON: &str = "native artifact publication could not reserve result storage";
const CATEGORY_PROVENANCE_MISMATCH: &str = "provenance_mismatch";
const CATEGORY_UNSUPPORTED_COMPLETE_DOCUMENT: &str = "unsupported_complete_document";
const CATEGORY_PREPARATION_FAILED: &str = "preparation_failed";

/// One opaque, one-use prepared artifact receipt for the ordinary Qt export flow.
#[pyclass(
    module = "ferrum_chem",
    name = "PreparedDocumentNativeArtifactV1",
    skip_from_py_object
)]
struct PyPreparedDocumentNativeArtifactV1 {
    receipt: Option<PreparedDocumentNativeArtifactV1>,
}

#[pymethods]
impl PyPreparedDocumentNativeArtifactV1 {
    #[getter]
    fn profile(&self, py: Python<'_>) -> PyResult<&'static str> {
        self.receipt
            .as_ref()
            .map(|receipt| receipt.profile().format_name())
            .ok_or_else(|| {
                native_artifact_error(
                    py,
                    CATEGORY_PREPARATION_FAILED,
                    "native artifact receipt was consumed",
                )
            })
    }

    #[getter]
    fn source_revision(&self, py: Python<'_>) -> PyResult<u64> {
        self.receipt
            .as_ref()
            .map(PreparedDocumentNativeArtifactV1::source_revision)
            .ok_or_else(|| {
                native_artifact_error(
                    py,
                    CATEGORY_PREPARATION_FAILED,
                    "native artifact receipt was consumed",
                )
            })
    }

    #[getter]
    fn source_digest(&self, py: Python<'_>) -> PyResult<String> {
        let receipt = self.receipt.as_ref().ok_or_else(|| {
            native_artifact_error(
                py,
                CATEGORY_PREPARATION_FAILED,
                "native artifact receipt was consumed",
            )
        })?;
        hex_digest(py, receipt.source_digest())
    }
}

/// Closed durable publication fact for one consumed native artifact receipt.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentNativeArtifactPublicationV1",
    skip_from_py_object
)]
struct PyDocumentNativeArtifactPublicationV1 {
    #[pyo3(get)]
    directory_entry_confirmed: bool,
}

/// Prepare exact whole-document native artifact bytes from an immutable observation.
#[pyfunction]
fn prepare_document_native_artifact_v1(
    py: Python<'_>,
    observation: PyRef<'_, PySessionDocumentObservationV1>,
    expected_revision: u64,
    expected_digest: &Bound<'_, PyString>,
    profile: &Bound<'_, PyString>,
) -> PyResult<PyPreparedDocumentNativeArtifactV1> {
    let expected_digest = expected_digest
        .to_str()
        .map_err(|_| native_artifact_error(py, CATEGORY_PROVENANCE_MISMATCH, DIGEST_TEXT_REASON))?;
    let expected_digest = parse_digest(py, expected_digest)?;
    let profile = profile
        .to_str()
        .map_err(|_| native_artifact_error(py, CATEGORY_PREPARATION_FAILED, PROFILE_REASON))?;
    let profile = parse_profile(py, profile)?;
    let observation = observation.observation().clone();
    let receipt = py.detach(move || {
        prepare_native_artifact(&observation, expected_revision, expected_digest, profile)
    });
    let receipt = receipt.map_err(|error| {
        native_artifact_error(
            py,
            category_for_preparation_error(&error),
            error.to_string(),
        )
    })?;
    Ok(PyPreparedDocumentNativeArtifactV1 {
        receipt: Some(receipt),
    })
}

/// Publish one prepared receipt to a concrete destination without session mutation.
#[pyfunction]
fn publish_prepared_document_native_artifact_v1(
    py: Python<'_>,
    mut receipt: PyRefMut<'_, PyPreparedDocumentNativeArtifactV1>,
    destination: PathBuf,
    origin: Option<PyRef<'_, PyLocalDocumentOriginTokenV1>>,
) -> PyResult<PyDocumentNativeArtifactPublicationV1> {
    let retained_source = match origin {
        Some(origin) => Some(match origin.try_clone_source() {
            Ok(source) => source,
            Err(error) => {
                return Err(publication_error(
                    py,
                    super::binding::PublicationNotStartedError::new_err,
                    destination.clone(),
                    format!(
                        "could not retain the opened document source for alias protection: {error}"
                    ),
                )?);
            }
        }),
        None => None,
    };
    let receipt = receipt.receipt.take().ok_or_else(|| {
        native_artifact_error(
            py,
            CATEGORY_PREPARATION_FAILED,
            "native artifact receipt was consumed",
        )
    })?;
    let outcome = match publish_native_artifact(receipt, destination, retained_source) {
        Ok(outcome) => outcome,
        Err(error) => {
            return Err(super::document_error_binding::map_artifact_publication_error(py, error)?);
        }
    };
    Ok(PyDocumentNativeArtifactPublicationV1 {
        directory_entry_confirmed: outcome.durability()
            == ArtifactPublicationDurabilityV1::Confirmed,
    })
}

fn parse_profile(py: Python<'_>, value: &str) -> PyResult<DocumentNativeArtifactProfileV1> {
    match value {
        "svg" => Ok(DocumentNativeArtifactProfileV1::Svg),
        "pdf" => Ok(DocumentNativeArtifactProfileV1::Pdf),
        "png_one_pixel_per_point_transparent" => {
            Ok(DocumentNativeArtifactProfileV1::PngOnePixelPerPointTransparent)
        }
        _ => Err(native_artifact_error(
            py,
            CATEGORY_PREPARATION_FAILED,
            PROFILE_REASON,
        )),
    }
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(native_artifact_error(
            py,
            CATEGORY_PROVENANCE_MISMATCH,
            DIGEST_REASON,
        ));
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

fn hex_digest(py: Python<'_>, digest: &[u8; 32]) -> PyResult<String> {
    let mut value = String::new();
    value
        .try_reserve_exact(64)
        .map_err(|_| native_artifact_error(py, CATEGORY_PREPARATION_FAILED, RESOURCE_REASON))?;
    for byte in digest {
        value.push(hex_digit(byte >> 4));
        value.push(hex_digit(byte & 0x0f));
    }
    Ok(value)
}

const fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + value - 10) as char,
    }
}

fn category_for_preparation_error(error: &DocumentNativeArtifactErrorV1) -> &'static str {
    match error {
        DocumentNativeArtifactErrorV1::ProvenanceMismatch => CATEGORY_PROVENANCE_MISMATCH,
        DocumentNativeArtifactErrorV1::ExcludedRoots => CATEGORY_UNSUPPORTED_COMPLETE_DOCUMENT,
        DocumentNativeArtifactErrorV1::Observation(_)
        | DocumentNativeArtifactErrorV1::Composition(_)
        | DocumentNativeArtifactErrorV1::PageDimension { .. }
        | DocumentNativeArtifactErrorV1::Svg(_)
        | DocumentNativeArtifactErrorV1::Pdf(_)
        | DocumentNativeArtifactErrorV1::Png(_) => CATEGORY_PREPARATION_FAILED,
    }
}

fn native_artifact_error(
    py: Python<'_>,
    category: &'static str,
    reason: impl Into<String>,
) -> PyErr {
    let reason = reason.into();
    let error = DocumentNativeArtifactError::new_err(reason.clone());
    if let Err(attribute_error) = error.value(py).setattr("reason", reason) {
        return attribute_error;
    }
    if let Err(attribute_error) = error.value(py).setattr("category", category) {
        return attribute_error;
    }
    error
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DocumentNativeArtifactError",
        module.py().get_type::<DocumentNativeArtifactError>(),
    )?;
    module.add_class::<PyPreparedDocumentNativeArtifactV1>()?;
    module.add_class::<PyDocumentNativeArtifactPublicationV1>()
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(
        prepare_document_native_artifact_v1,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        publish_prepared_document_native_artifact_v1,
        module
    )?)?;
    Ok(())
}
