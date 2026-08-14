//! Python document exceptions and lossless conversion from typed document errors.

use std::path::PathBuf;

use ferrum_document::{
    DocumentObjectIdV1, DocumentSessionError, ProjectionError as DocumentProjectionError,
    SessionOperationError, TypedDocumentError,
};
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(ferrum_chem, FerrumError, PyException);
create_exception!(ferrum_chem, DocumentError, FerrumError);
create_exception!(ferrum_chem, DocumentInputError, DocumentError);
create_exception!(ferrum_chem, DocumentLoadError, DocumentError);
create_exception!(ferrum_chem, DocumentSerializationError, DocumentError);
create_exception!(ferrum_chem, RevisionConflictError, DocumentError);
create_exception!(ferrum_chem, RevisionExhaustedError, DocumentError);
create_exception!(ferrum_chem, HistoryUnavailableError, DocumentError);
create_exception!(ferrum_chem, ProjectionError, DocumentError);
create_exception!(ferrum_chem, OperationValidationError, DocumentError);
create_exception!(
    ferrum_chem,
    InvalidAtomElementError,
    OperationValidationError
);
create_exception!(
    ferrum_chem,
    PreparedOperationForeignSessionError,
    PreparedOperationError
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

pub(crate) fn document_object_id(py: Python<'_>, value: String) -> PyResult<DocumentObjectIdV1> {
    let object_id = value.clone();
    DocumentObjectIdV1::parse(value).map_err(|error| {
        let py_error = InvalidDocumentObjectIdError::new_err(error.to_string());
        let result = py_error.value(py).setattr("object_id", object_id);
        if let Err(attribute_error) = result {
            return attribute_error;
        }
        py_error
    })
}

pub(crate) fn document_result<T>(
    py: Python<'_>,
    result: Result<T, DocumentSessionError>,
) -> PyResult<T> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(map_document_error(py, error)?),
    }
}

pub(crate) fn map_document_error(py: Python<'_>, error: DocumentSessionError) -> PyResult<PyErr> {
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
        DocumentSessionError::PreparedOperationForeignSession => {
            PreparedOperationForeignSessionError::new_err(error.to_string())
        }
        DocumentSessionError::Projection(error) => projection_error(py, error)?,
        DocumentSessionError::Operation(error) => operation_error(py, error)?,
        DocumentSessionError::DirectHaworthReobservation(error) => {
            let message = error.to_string();
            let py_error = OperationValidationError::new_err(message.clone());
            py_error.value(py).setattr("reason", message)?;
            py_error
        }
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

pub(crate) fn projection_error(py: Python<'_>, error: DocumentProjectionError) -> PyResult<PyErr> {
    let py_error = ProjectionError::new_err(error.to_string());
    let value = py_error.value(py);
    value.setattr("reason", error.to_string())?;
    Ok(py_error)
}

pub(crate) fn operation_validation_error(py: Python<'_>, reason: String) -> PyErr {
    let error = OperationValidationError::new_err(reason.clone());
    let _ = error.value(py).setattr("reason", reason);
    error
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
        SessionOperationError::InvalidAtomNumberPair
        | SessionOperationError::InvalidAtomMarkSelector => {
            Ok(OperationValidationError::new_err(error.to_string()))
        }
        SessionOperationError::InvalidDirectHaworthInsertion(_)
        | SessionOperationError::InvalidStraightenDepiction(_) => {
            let message = error.to_string();
            let py_error = OperationValidationError::new_err(message.clone());
            py_error.value(py).setattr("reason", message)?;
            Ok(py_error)
        }
        SessionOperationError::UnknownAtom(identifier) => {
            let message = format!("typed atom does not exist: {identifier}");
            let py_error = UnknownDocumentObjectError::new_err(message);
            py_error.value(py).setattr("object_id", identifier)?;
            Ok(py_error)
        }
        SessionOperationError::UnknownBond(identifier) => {
            let message = format!("typed bond does not exist: {identifier}");
            let py_error = UnknownDocumentObjectError::new_err(message);
            py_error.value(py).setattr("object_id", identifier)?;
            Ok(py_error)
        }
        SessionOperationError::UnknownPlus(identifier) => {
            let message = format!("typed Plus does not exist: {identifier}");
            let py_error = UnknownDocumentObjectError::new_err(message);
            py_error.value(py).setattr("object_id", identifier)?;
            Ok(py_error)
        }
        SessionOperationError::UnknownText(identifier) => {
            let message = format!("typed Text does not exist: {identifier}");
            let py_error = UnknownDocumentObjectError::new_err(message);
            py_error.value(py).setattr("object_id", identifier)?;
            Ok(py_error)
        }
        SessionOperationError::UnknownPresentationRoot(identifier) => {
            let message = format!("typed presentation root does not exist: {identifier}");
            let py_error = UnknownDocumentObjectError::new_err(message);
            py_error.value(py).setattr("object_id", identifier)?;
            Ok(py_error)
        }
        SessionOperationError::UnknownArrow(identifier) => {
            let message = format!("typed Arrow does not exist: {identifier}");
            let py_error = UnknownDocumentObjectError::new_err(message);
            py_error.value(py).setattr("object_id", identifier)?;
            Ok(py_error)
        }
        SessionOperationError::UnknownGeometricPresentation(identifier) => {
            let message = format!("typed geometric presentation does not exist: {identifier}");
            let py_error = UnknownDocumentObjectError::new_err(message);
            py_error.value(py).setattr("object_id", identifier)?;
            Ok(py_error)
        }
        SessionOperationError::UnknownWavy(identifier) => {
            let message = format!("typed Wavy presentation does not exist: {identifier}");
            let py_error = UnknownDocumentObjectError::new_err(message);
            py_error.value(py).setattr("object_id", identifier)?;
            Ok(py_error)
        }
        SessionOperationError::UnknownBracketPair(identifier) => {
            let message = format!("typed bracket pair does not exist: {identifier}");
            let py_error = UnknownDocumentObjectError::new_err(message);
            py_error.value(py).setattr("object_id", identifier)?;
            Ok(py_error)
        }
        SessionOperationError::UnknownDocumentObject(object_id) => {
            let message = format!("document object does not exist: {object_id}");
            let py_error = UnknownDocumentObjectError::new_err(message);
            py_error.value(py).setattr("object_id", object_id)?;
            Ok(py_error)
        }
        SessionOperationError::InvalidCreateAtomTarget(_)
        | SessionOperationError::PaperDimensionsRequireCustom
        | SessionOperationError::InvalidCreateBondTarget(_)
        | SessionOperationError::InvalidMoleculeCoordinateTarget(_)
        | SessionOperationError::MoleculeCoordinateRevisionMismatch { .. }
        | SessionOperationError::MoleculeCoordinateDigestMismatch
        | SessionOperationError::CreateBondSelfLoop(_)
        | SessionOperationError::CreateBondAcrossMolecules
        | SessionOperationError::CreateBondDuplicate { .. }
        | SessionOperationError::InvalidWavyInsertion(_)
        | SessionOperationError::InvalidBracketInsertion(_)
        | SessionOperationError::AtomIdentifierExhausted
        | SessionOperationError::MoleculeIdentifierExhausted
        | SessionOperationError::BondIdentifierExhausted
        | SessionOperationError::PresentationIdentifierExhausted => {
            Ok(OperationValidationError::new_err(error.to_string()))
        }
        SessionOperationError::Candidate(TypedDocumentError::UnknownTopLevelTransformRoot(
            ref identifier,
        )) => {
            let object_id = identifier.as_str().to_owned();
            let py_error = UnknownDocumentObjectError::new_err(error.to_string());
            py_error.value(py).setattr("object_id", object_id)?;
            Ok(py_error)
        }
        SessionOperationError::Candidate(TypedDocumentError::UnknownAtomRotationTarget {
            molecule_id: _,
            ref atom_id,
        }) => {
            let py_error = UnknownDocumentObjectError::new_err(error.to_string());
            py_error.value(py).setattr("object_id", atom_id.as_str())?;
            Ok(py_error)
        }
        SessionOperationError::Candidate(TypedDocumentError::UnknownGeometryRepairMolecule(
            ref identifier,
        )) => {
            let object_id = identifier.as_str().to_owned();
            let py_error = UnknownDocumentObjectError::new_err(error.to_string());
            py_error.value(py).setattr("object_id", object_id)?;
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
