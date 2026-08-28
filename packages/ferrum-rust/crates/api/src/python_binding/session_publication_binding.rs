//! Python publication DTOs and methods for one document session.

use std::path::PathBuf;

use ferrum_document::{
    DocumentSnapshot, PreparedDocumentUserTemplatePublicationV1, Publication, SaveOutcome,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::document_error_binding::{FerrumError, document_result};
use super::document_session_binding::PyDocumentSession;

create_exception!(ferrum_chem, UserTemplatePublicationError, FerrumError);

/// Immutable Python-owned copy of one authoritative document revision.
#[pyclass(frozen, name = "DocumentSnapshot", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyDocumentSnapshot {
    #[pyo3(get)]
    cdml: String,
    #[pyo3(get)]
    revision: u64,
    #[pyo3(get)]
    digest: String,
    #[pyo3(get)]
    is_dirty: bool,
}

impl From<DocumentSnapshot> for PyDocumentSnapshot {
    fn from(snapshot: DocumentSnapshot) -> Self {
        Self {
            cdml: snapshot.cdml().to_owned(),
            revision: snapshot.revision(),
            digest: hex_digest(snapshot.digest()),
            is_dirty: snapshot.is_dirty(),
        }
    }
}

impl PyDocumentSnapshot {
    /// Return the authoritative revision carried by this native-issued snapshot.
    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    /// Return the canonical hexadecimal digest carried by this native-issued snapshot.
    pub(crate) fn digest_hex(&self) -> &str {
        &self.digest
    }
}

/// Closed outcome of one save-like publication.
#[pyclass(frozen, name = "SaveOutcome", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PySaveOutcome {
    outcome: SaveOutcome,
}

#[pymethods]
impl PySaveOutcome {
    #[getter]
    fn is_confirmed(&self) -> bool {
        self.outcome == SaveOutcome::Confirmed
    }

    #[getter]
    fn requires_destination_verification(&self) -> bool {
        self.outcome == SaveOutcome::DirectoryEntryUnconfirmed
    }
}

/// Immutable outcome of one document publication attempt.
#[pyclass(frozen, name = "Publication")]
pub(crate) struct PyPublication {
    #[pyo3(get)]
    snapshot: PyDocumentSnapshot,
    #[pyo3(get)]
    published_snapshot: PyDocumentSnapshot,
    #[pyo3(get)]
    outcome: PySaveOutcome,
}

impl From<Publication> for PyPublication {
    fn from(publication: Publication) -> Self {
        Self {
            snapshot: publication.snapshot().clone().into(),
            published_snapshot: publication.published_snapshot().clone().into(),
            outcome: PySaveOutcome {
                outcome: publication.outcome(),
            },
        }
    }
}

/// Opaque, one-use session-owned authorization for template publication.
#[pyclass(
    unsendable,
    frozen,
    module = "ferrum_chem",
    name = "PreparedUserTemplatePublicationV1",
    skip_from_py_object
)]
pub(crate) struct PyPreparedUserTemplatePublicationV1 {
    prepared: PreparedDocumentUserTemplatePublicationV1,
    #[pyo3(get)]
    revision: u64,
    #[pyo3(get)]
    digest: String,
    #[pyo3(get)]
    display_name: Option<String>,
}

#[pymethods]
impl PyDocumentSession {
    /// Publish the current revision and update the saved baseline only if confirmed.
    ///
    /// The path is copied before publication. This operation requires an explicit
    /// revision so an unrelated stale caller cannot silently write session state.
    fn save_atomic(
        &mut self,
        py: Python<'_>,
        path: PathBuf,
        expected_revision: u64,
    ) -> PyResult<PyPublication> {
        document_result(py, self.session.save_atomic(&path, expected_revision)).map(Into::into)
    }

    /// Export the current revision without changing baseline, history, or dirty state.
    fn recovery_export(
        &self,
        py: Python<'_>,
        path: PathBuf,
        expected_revision: u64,
    ) -> PyResult<PyPublication> {
        document_result(py, self.session.recovery_export(&path, expected_revision)).map(Into::into)
    }

    /// Prepare one opaque, exact-fence saved-template publication receipt.
    fn prepare_user_template_publication_v1(
        &self,
        py: Python<'_>,
    ) -> PyResult<PyPreparedUserTemplatePublicationV1> {
        let prepared = match self.session.prepare_document_user_template_publication_v1() {
            Ok(prepared) => prepared,
            Err(error) => return Err(user_template_publication_error(py, error)),
        };
        let digest = hex_digest(prepared.digest());
        let display_name = prepared
            .display_name()
            .map(copy_publication_fact)
            .transpose()?;
        Ok(PyPreparedUserTemplatePublicationV1 {
            revision: prepared.revision(),
            digest,
            display_name,
            prepared,
        })
    }

    /// Publish the live document authorized by one opaque template receipt.
    fn publish_user_template_v1(
        &self,
        py: Python<'_>,
        prepared: PyRef<'_, PyPreparedUserTemplatePublicationV1>,
        path: PathBuf,
    ) -> PyResult<PyPublication> {
        match self
            .session
            .publish_document_user_template_v1(&prepared.prepared, &path)
        {
            Ok(publication) => Ok(publication.into()),
            Err(error) => Err(user_template_publication_error(py, error)),
        }
    }
}

pub(crate) fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn copy_publication_fact(value: &str) -> PyResult<String> {
    let mut copied = String::new();
    copied
        .try_reserve_exact(value.len())
        .map_err(|_| UserTemplatePublicationError::new_err("result allocation failed"))?;
    copied.push_str(value);
    Ok(copied)
}

pub(crate) fn user_template_publication_error(
    py: Python<'_>,
    error: ferrum_document::DocumentUserTemplatePublicationErrorV1,
) -> PyErr {
    use ferrum_document::DocumentUserTemplatePublicationErrorV1 as Error;

    match error {
        Error::Session(error) => match document_result::<()>(py, Err(error)) {
            Ok(()) => unreachable!("an error result cannot succeed"),
            Err(error) => error,
        },
        Error::Ineligible(_) => closed_user_template_publication_error(py, "ineligible"),
        Error::ForeignSession => closed_user_template_publication_error(py, "foreign_session"),
        Error::Consumed => closed_user_template_publication_error(py, "consumed"),
    }
}

fn closed_user_template_publication_error(py: Python<'_>, reason: &'static str) -> PyErr {
    let error = UserTemplatePublicationError::new_err("user-template publication refused");
    error
        .value(py)
        .setattr("reason", reason)
        .expect("closed error reason assignment must succeed");
    error
}
