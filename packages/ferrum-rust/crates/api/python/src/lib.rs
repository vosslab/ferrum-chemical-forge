use std::path::PathBuf;

use ferrum_document::{
    DocumentSession, DocumentSessionError, DocumentSnapshot, PendingCreateAtom, PersistentId,
    Publication, SaveOutcome, SessionObservationV1, SessionOperation, SessionOperationError,
    SessionOperationV1,
};
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(ferrum_chem, FerrumError, PyException);
create_exception!(ferrum_chem, DocumentError, FerrumError);
create_exception!(ferrum_chem, DocumentLoadError, DocumentError);
create_exception!(ferrum_chem, DocumentSerializationError, DocumentError);
create_exception!(ferrum_chem, RevisionConflictError, DocumentError);
create_exception!(ferrum_chem, RevisionExhaustedError, DocumentError);
create_exception!(ferrum_chem, HistoryUnavailableError, DocumentError);
create_exception!(ferrum_chem, OperationValidationError, DocumentError);
create_exception!(
    ferrum_chem,
    InvalidAtomElementError,
    OperationValidationError
);
create_exception!(
    ferrum_chem,
    InvalidDocumentObjectIdError,
    OperationValidationError
);
create_exception!(
    ferrum_chem,
    UnknownDocumentObjectError,
    OperationValidationError
);
create_exception!(ferrum_chem, PreparedOperationError, DocumentError);
create_exception!(
    ferrum_chem,
    PreparedOperationConsumedError,
    PreparedOperationError
);
create_exception!(ferrum_chem, PublicationError, FerrumError);
create_exception!(ferrum_chem, InvalidDestinationError, PublicationError);
create_exception!(ferrum_chem, PublicationNotStartedError, PublicationError);
create_exception!(
    ferrum_chem,
    PublicationPossiblyCompletedError,
    PublicationError
);

/// Immutable Python-owned copy of one authoritative document revision.
///
/// All values are copied from Rust. A snapshot has no mutable alias to its
/// originating [`PyDocumentSession`], so callers may retain it after later session
/// calls, but it never observes those later revisions.
#[pyclass(frozen, name = "DocumentSnapshot", skip_from_py_object)]
#[derive(Clone)]
struct PyDocumentSnapshot {
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

/// Immutable revision-checked observation of the current document session state.
///
/// The snapshot is a copied value, not a live view into the thread-affine session.
#[pyclass(frozen, name = "SessionObservationV1")]
struct PySessionObservationV1 {
    #[pyo3(get)]
    snapshot: PyDocumentSnapshot,
}

impl From<SessionObservationV1> for PySessionObservationV1 {
    fn from(observation: SessionObservationV1) -> Self {
        Self {
            snapshot: observation.snapshot().clone().into(),
        }
    }
}

/// Immutable result of one document publication attempt.
///
/// `snapshot` is the session state after the operation. `published_snapshot` is
/// the exact value given to the publisher. A confirmed ordinary save returns a
/// clean `snapshot`; recovery exports and unconfirmed replacements do not alter
/// the session baseline.
#[pyclass(frozen, name = "Publication")]
struct PyPublication {
    #[pyo3(get)]
    snapshot: PyDocumentSnapshot,
    #[pyo3(get)]
    published_snapshot: PyDocumentSnapshot,
    #[pyo3(get)]
    outcome: &'static str,
}

impl From<Publication> for PyPublication {
    fn from(publication: Publication) -> Self {
        let outcome = match publication.outcome() {
            SaveOutcome::Confirmed => "confirmed",
            SaveOutcome::DirectoryEntryUnconfirmed => "directory_entry_unconfirmed",
        };
        Self {
            snapshot: publication.snapshot().clone().into(),
            published_snapshot: publication.published_snapshot().clone().into(),
            outcome,
        }
    }
}

/// Closed V1 operation grammar for authoritative session mutations.
///
/// This value owns a Rust enum rather than a mapping or XML fragment. It can only
/// be created by a named factory and is consumed by no Python-side parser.
#[pyclass(frozen, name = "DocumentOperationV1", skip_from_py_object)]
#[derive(Clone)]
struct PyDocumentOperationV1 {
    operation: SessionOperation,
}

#[pymethods]
impl PyDocumentOperationV1 {
    /// Build the V1 operation that replaces one existing atom's element spelling.
    #[staticmethod]
    fn set_atom_element(atom_id: String, element: String) -> Self {
        Self {
            operation: SessionOperation::V1(SessionOperationV1::SetAtomElement {
                atom_id,
                element,
            }),
        }
    }
}

/// Opaque one-use prepared atom insertion.
///
/// The Rust value binds its candidate to the revision at which it was prepared.
/// It is deliberately thread-affine and exposes only the durable identifier that
/// would be created; the internal provisional token is never serialized to Python.
#[pyclass(unsendable, name = "PreparedAtomInsertion")]
struct PyPreparedAtomInsertion {
    pending: PendingCreateAtom,
    #[pyo3(get)]
    identifier: String,
}

/// Thread-affine owner of one mutable Rust CDML document session.
///
/// A session is deliberately unsendable: callers create, mutate, and destroy it
/// on its Python-owning thread. Every method is synchronous; it retains no Python
/// input or callback after return. Snapshots, observations, and publications are
/// frozen owned copies and may outlive their originating session.
#[pyclass(unsendable, name = "DocumentSession")]
struct PyDocumentSession {
    session: DocumentSession,
}

#[pymethods]
impl PyDocumentSession {
    /// Create a thread-affine session from CDML copied during this call.
    #[staticmethod]
    fn load(py: Python<'_>, cdml: &str) -> PyResult<Self> {
        let session = document_result(py, DocumentSession::load(cdml))?;
        Ok(Self { session })
    }

    /// Return one immutable owned snapshot without changing session state.
    fn snapshot(&self, py: Python<'_>) -> PyResult<PyDocumentSnapshot> {
        document_result(py, self.session.snapshot()).map(PyDocumentSnapshot::from)
    }

    /// Observe the current session state after checking its expected revision.
    fn observe(&self, py: Python<'_>, expected_revision: u64) -> PyResult<PySessionObservationV1> {
        document_result(py, self.session.observe(expected_revision)).map(Into::into)
    }

    /// Submit one closed V1 operation against an exact expected revision.
    fn submit(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        operation: PyRef<'_, PyDocumentOperationV1>,
    ) -> PyResult<PyDocumentSnapshot> {
        document_result(
            py,
            self.session
                .submit(expected_revision, operation.operation.clone()),
        )
        .map(Into::into)
    }

    /// Move to the preceding retained state, producing a new monotonic revision.
    fn undo(&mut self, py: Python<'_>, expected_revision: u64) -> PyResult<PyDocumentSnapshot> {
        document_result(py, self.session.undo(expected_revision)).map(Into::into)
    }

    /// Move to the succeeding retained state, producing a new monotonic revision.
    fn redo(&mut self, py: Python<'_>, expected_revision: u64) -> PyResult<PyDocumentSnapshot> {
        document_result(py, self.session.redo(expected_revision)).map(Into::into)
    }

    /// Prepare a revision-bound, one-use atom insertion without changing the session.
    fn prepare_create_atom(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        molecule_id: String,
        atom_id: String,
        element: String,
    ) -> PyResult<PyPreparedAtomInsertion> {
        let molecule_id = persistent_id(py, molecule_id)?;
        let atom_id = persistent_id(py, atom_id)?;
        let identifier = atom_id.as_str().to_owned();
        let pending = document_result(
            py,
            self.session
                .prepare_create_atom(expected_revision, &molecule_id, atom_id, &element),
        )?;
        Ok(PyPreparedAtomInsertion {
            pending,
            identifier,
        })
    }

    /// Commit one prepared atom insertion exactly once at its prepared revision.
    fn commit_create_atom(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyPreparedAtomInsertion>,
    ) -> PyResult<PyDocumentSnapshot> {
        document_result(
            py,
            self.session
                .commit_create_atom(expected_revision, &mut prepared.pending),
        )
        .map(Into::into)
    }

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
}

fn persistent_id(py: Python<'_>, value: String) -> PyResult<PersistentId> {
    let object_id = value.clone();
    PersistentId::new(value).map_err(|error| {
        let py_error = InvalidDocumentObjectIdError::new_err(error.to_string());
        let result = py_error.value(py).setattr("object_id", object_id);
        if let Err(attribute_error) = result {
            return attribute_error;
        }
        py_error
    })
}

fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn document_result<T>(py: Python<'_>, result: Result<T, DocumentSessionError>) -> PyResult<T> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(map_document_error(py, error)?),
    }
}

fn map_document_error(py: Python<'_>, error: DocumentSessionError) -> PyResult<PyErr> {
    Ok(match error {
        DocumentSessionError::Load(error) => DocumentLoadError::new_err(error.to_string()),
        DocumentSessionError::Serialize(error) => {
            DocumentSerializationError::new_err(error.to_string())
        }
        DocumentSessionError::RevisionConflict { expected, actual } => {
            revision_conflict_error(py, expected, actual)?
        }
        DocumentSessionError::RevisionExhausted => {
            RevisionExhaustedError::new_err(error.to_string())
        }
        DocumentSessionError::PreparedOperationConsumed => {
            PreparedOperationConsumedError::new_err(error.to_string())
        }
        DocumentSessionError::Operation(error) => operation_error(py, error)?,
        DocumentSessionError::HistoryUnavailable => {
            HistoryUnavailableError::new_err(error.to_string())
        }
        DocumentSessionError::InvalidDestination { path, reason } => {
            publication_error(py, InvalidDestinationError::new_err, path, reason)?
        }
        DocumentSessionError::PublishNotStarted { path, source } => {
            publication_error(py, PublicationNotStartedError::new_err, path, source)?
        }
        DocumentSessionError::PublishNotStartedWithCleanup {
            path,
            source,
            cleanup,
        } => publication_error(
            py,
            PublicationNotStartedError::new_err,
            path,
            format!("{source}; temporary cleanup failed: {cleanup}"),
        )?,
        DocumentSessionError::ReplacementRejectedWithCleanup {
            path,
            reason,
            cleanup,
        } => publication_error(
            py,
            PublicationNotStartedError::new_err,
            path,
            format!("{reason}; temporary cleanup failed: {cleanup}"),
        )?,
        DocumentSessionError::TemporaryName { path, detail } => {
            publication_error(py, PublicationNotStartedError::new_err, path, detail)?
        }
        DocumentSessionError::TemporaryNameExhausted { path } => publication_error(
            py,
            PublicationNotStartedError::new_err,
            path,
            "could not reserve a unique temporary file",
        )?,
        DocumentSessionError::PublishPossiblyCompleted { path, source } => {
            publication_error(py, PublicationPossiblyCompletedError::new_err, path, source)?
        }
    })
}

fn revision_conflict_error(py: Python<'_>, expected: u64, actual: u64) -> PyResult<PyErr> {
    let error = RevisionConflictError::new_err(format!(
        "document revision conflict: expected {expected}, current revision is {actual}"
    ));
    let value = error.value(py);
    value.setattr("expected", expected)?;
    value.setattr("actual", actual)?;
    Ok(error)
}

fn operation_error(py: Python<'_>, error: SessionOperationError) -> PyResult<PyErr> {
    match error {
        SessionOperationError::InvalidAtomElement => {
            Ok(InvalidAtomElementError::new_err(error.to_string()))
        }
        SessionOperationError::UnknownAtom(identifier) => {
            let message = format!("typed atom does not exist: {identifier}");
            let py_error = UnknownDocumentObjectError::new_err(message);
            py_error.value(py).setattr("object_id", identifier)?;
            Ok(py_error)
        }
        SessionOperationError::Candidate(_) | SessionOperationError::Serialize(_) => {
            Ok(OperationValidationError::new_err(error.to_string()))
        }
    }
}

fn publication_error(
    py: Python<'_>,
    constructor: impl FnOnce(String) -> PyErr,
    path: PathBuf,
    detail: impl std::fmt::Display,
) -> PyResult<PyErr> {
    let message = format!("{}: {detail}", path.display());
    let error = constructor(message);
    let value = error.value(py);
    value.setattr("path", path.display().to_string())?;
    value.setattr("reason", detail.to_string())?;
    Ok(error)
}

/// Initialize Ferrum-Chem's public Python extension module.
#[pymodule]
fn ferrum_chem(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("FerrumError", module.py().get_type::<FerrumError>())?;
    module.add("DocumentError", module.py().get_type::<DocumentError>())?;
    module.add(
        "DocumentLoadError",
        module.py().get_type::<DocumentLoadError>(),
    )?;
    module.add(
        "DocumentSerializationError",
        module.py().get_type::<DocumentSerializationError>(),
    )?;
    module.add(
        "RevisionConflictError",
        module.py().get_type::<RevisionConflictError>(),
    )?;
    module.add(
        "RevisionExhaustedError",
        module.py().get_type::<RevisionExhaustedError>(),
    )?;
    module.add(
        "HistoryUnavailableError",
        module.py().get_type::<HistoryUnavailableError>(),
    )?;
    module.add(
        "OperationValidationError",
        module.py().get_type::<OperationValidationError>(),
    )?;
    module.add(
        "InvalidAtomElementError",
        module.py().get_type::<InvalidAtomElementError>(),
    )?;
    module.add(
        "InvalidDocumentObjectIdError",
        module.py().get_type::<InvalidDocumentObjectIdError>(),
    )?;
    module.add(
        "UnknownDocumentObjectError",
        module.py().get_type::<UnknownDocumentObjectError>(),
    )?;
    module.add(
        "PreparedOperationError",
        module.py().get_type::<PreparedOperationError>(),
    )?;
    module.add(
        "PreparedOperationConsumedError",
        module.py().get_type::<PreparedOperationConsumedError>(),
    )?;
    module.add(
        "PublicationError",
        module.py().get_type::<PublicationError>(),
    )?;
    module.add(
        "InvalidDestinationError",
        module.py().get_type::<InvalidDestinationError>(),
    )?;
    module.add(
        "PublicationNotStartedError",
        module.py().get_type::<PublicationNotStartedError>(),
    )?;
    module.add(
        "PublicationPossiblyCompletedError",
        module.py().get_type::<PublicationPossiblyCompletedError>(),
    )?;
    module.add_class::<PyDocumentSession>()?;
    module.add_class::<PyDocumentSnapshot>()?;
    module.add_class::<PySessionObservationV1>()?;
    module.add_class::<PyDocumentOperationV1>()?;
    module.add_class::<PyPreparedAtomInsertion>()?;
    module.add_class::<PyPublication>()
}
