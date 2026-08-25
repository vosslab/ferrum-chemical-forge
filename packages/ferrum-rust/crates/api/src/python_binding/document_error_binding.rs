//! Python document exceptions and lossless conversion from typed document errors.

use std::path::PathBuf;

use ferrum_document::{
    DocumentObjectIdV1, DocumentSessionError, ProjectionError as DocumentProjectionError,
    SessionOperationError, TypedDocumentError,
    artifact_publication_v1::{
        ArtifactDestinationRejectionV1, ArtifactPrepublicationPhaseV1, ArtifactPublicationErrorV1,
    },
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

pub(crate) fn document_object_id(
    py: Python<'_>,
    serialized_id: String,
) -> PyResult<DocumentObjectIdV1> {
    let object_id_text = serialized_id.clone();
    DocumentObjectIdV1::parse(serialized_id).map_err(|error| {
        let py_error = InvalidDocumentObjectIdError::new_err(error.to_string());
        let result = py_error.value(py).setattr("object_id", object_id_text);
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
        DocumentSessionError::EmptyMoleculeBatch => {
            OperationValidationError::new_err("document operation rejected")
        }
        DocumentSessionError::Load(error) => DocumentLoadError::new_err(error.to_string()),
        DocumentSessionError::Serialize(error) => {
            DocumentSerializationError::new_err(error.to_string())
        }
        DocumentSessionError::ClipboardPaste(error) => {
            let _ = error;
            closed_operation_validation_error(py, "operation", "clipboard_paste")?
        }
        DocumentSessionError::ClipboardCut(error) => {
            let _ = error;
            closed_operation_validation_error(py, "operation", "clipboard_cut")?
        }
        DocumentSessionError::UserTemplate(error) => {
            let _ = error;
            closed_operation_validation_error(py, "operation", "user_template")?
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
        DocumentSessionError::RendererAdmission => {
            closed_operation_validation_error(py, "operation", "renderer_admission")?
        }
        DocumentSessionError::TransitionAuthorization(_)
        | DocumentSessionError::DirectBondAdmission(_) => {
            closed_operation_validation_error(py, "operation", "transition_authorization")?
        }
        DocumentSessionError::Projection(error) => projection_error(py, error)?,
        DocumentSessionError::Operation(error) => operation_error(py, error)?,
        DocumentSessionError::DirectHaworthReobservation(error) => {
            let _ = error;
            closed_operation_validation_error(py, "operation", "haworth_reobservation")?
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

pub(crate) fn map_artifact_publication_error(
    py: Python<'_>,
    error: ArtifactPublicationErrorV1,
) -> PyResult<PyErr> {
    Ok(match error {
        ArtifactPublicationErrorV1::RejectedDestination {
            destination,
            reason,
        } => publication_error(
            py,
            InvalidDestinationError::new_err,
            destination,
            artifact_destination_rejection(reason),
        )?,
        ArtifactPublicationErrorV1::NotPublished {
            destination,
            phase,
            source,
        } => publication_error(
            py,
            PublicationNotStartedError::new_err,
            destination,
            format!(
                "publication stopped while {}: {source}",
                artifact_phase(phase)
            ),
        )?,
        ArtifactPublicationErrorV1::NotPublishedTemporaryMayRemain {
            destination,
            phase,
            source,
            cleanup,
        } => publication_error(
            py,
            PublicationNotStartedError::new_err,
            destination,
            format!(
                "publication stopped while {}: {source}; temporary cleanup failed: {cleanup}",
                artifact_phase(phase)
            ),
        )?,
        ArtifactPublicationErrorV1::RejectedDestinationTemporaryMayRemain {
            destination,
            reason,
            cleanup,
        } => publication_error(
            py,
            PublicationNotStartedError::new_err,
            destination,
            format!(
                "{}; temporary cleanup failed: {cleanup}",
                artifact_destination_rejection(reason)
            ),
        )?,
        ArtifactPublicationErrorV1::TemporaryName {
            destination,
            source,
        } => publication_error(py, PublicationNotStartedError::new_err, destination, source)?,
        ArtifactPublicationErrorV1::TemporaryNameExhausted { destination } => publication_error(
            py,
            PublicationNotStartedError::new_err,
            destination,
            "could not reserve a unique temporary file",
        )?,
        ArtifactPublicationErrorV1::PossiblyPublished { receipt, source } => publication_error(
            py,
            PublicationPossiblyCompletedError::new_err,
            receipt.into_destination(),
            source,
        )?,
    })
}

const fn artifact_destination_rejection(reason: ArtifactDestinationRejectionV1) -> &'static str {
    match reason {
        ArtifactDestinationRejectionV1::MissingFileName => "destination must include a file name",
        ArtifactDestinationRejectionV1::ParentTraversesSymlink => {
            "destination parent path must not traverse a symbolic link"
        }
        ArtifactDestinationRejectionV1::ParentIsNotDirectory => {
            "every destination parent component must be a directory"
        }
        ArtifactDestinationRejectionV1::FinalIsSymlink => {
            "destination file must not be a symbolic link"
        }
        ArtifactDestinationRejectionV1::FinalIsNotRegular => {
            "existing destination must be a regular file"
        }
        ArtifactDestinationRejectionV1::SourceAliasesDestination => {
            "destination must not alias the retained source file"
        }
    }
}

const fn artifact_phase(phase: ArtifactPrepublicationPhaseV1) -> &'static str {
    match phase {
        ArtifactPrepublicationPhaseV1::OpenParent => "opening the destination parent",
        ArtifactPrepublicationPhaseV1::ValidateBeforeTemporary => {
            "validating the destination before temporary creation"
        }
        ArtifactPrepublicationPhaseV1::ReserveTemporary => "reserving a private temporary file",
        ArtifactPrepublicationPhaseV1::ValidateTemporary => "validating the private temporary file",
        ArtifactPrepublicationPhaseV1::WriteOrSyncTemporary => {
            "writing or synchronizing the private temporary file"
        }
        ArtifactPrepublicationPhaseV1::ValidateBeforeRename => {
            "revalidating the destination before replacement"
        }
        ArtifactPrepublicationPhaseV1::Rename => "replacing the destination",
    }
}

pub(crate) fn publication_resource_error(
    py: Python<'_>,
    destination: PathBuf,
    detail: &'static str,
) -> PyResult<PyErr> {
    publication_error(py, PublicationNotStartedError::new_err, destination, detail)
}

pub(crate) fn projection_error(py: Python<'_>, error: DocumentProjectionError) -> PyResult<PyErr> {
    let py_error = ProjectionError::new_err(error.to_string());
    let value = py_error.value(py);
    value.setattr("reason", error.to_string())?;
    Ok(py_error)
}

pub(crate) fn revision_conflict_error(
    py: Python<'_>,
    expected: u64,
    actual: u64,
) -> PyResult<PyErr> {
    let error = RevisionConflictError::new_err(format!(
        "document revision conflict: expected {expected}, current revision is {actual}"
    ));
    let value = error.value(py);
    value.setattr("expected", expected)?;
    value.setattr("actual", actual)?;
    Ok(error)
}

pub(crate) fn digest_conflict_error(
    py: Python<'_>,
    expected_revision: u64,
    actual_revision: u64,
) -> PyResult<PyErr> {
    let error = RevisionConflictError::new_err(
        "document digest conflict: expected digest does not match the live document",
    );
    let value = error.value(py);
    value.setattr("expected", expected_revision)?;
    value.setattr("actual", actual_revision)?;
    value.setattr("reason", "expected digest does not match the live document")?;
    Ok(error)
}

fn operation_error(py: Python<'_>, error: SessionOperationError) -> PyResult<PyErr> {
    match error {
        SessionOperationError::ExplicitFragment(_)
        | SessionOperationError::InvalidCatalogPlacement(_)
        | SessionOperationError::DirectBond(_)
        | SessionOperationError::Reaction(_)
        | SessionOperationError::CompactGroupMaterialization(_)
        | SessionOperationError::CompactGroupMaterializationRequiresTransitionCore
        | SessionOperationError::HydrogenMaterialization(_)
        | SessionOperationError::HydrogenMaterializationRequiresTransitionCore
        | SessionOperationError::MoleculeInsertionRequiresTransitionCore
        | SessionOperationError::InterchangeRecordBatchInsertionRequiresTransitionCore
        | SessionOperationError::PresentationCreateRequiresTransitionCore => {
            closed_operation_validation_error(py, "operation", "transition")
        }
        SessionOperationError::EmptyLinearFormSelection
        | SessionOperationError::LinearFormPlan(_) => {
            closed_operation_validation_error(py, "operation", "linear_form")
        }
        SessionOperationError::HistoryResourceExhausted
        | SessionOperationError::FragmentIdentifierExhausted
        | SessionOperationError::GeneratedIdentifierAllocationFailed => {
            closed_operation_validation_error(py, "resource_exhausted", "document")
        }
        SessionOperationError::InvalidAtomElement => Ok(InvalidAtomElementError::new_err(
            "document operation rejected",
        )),
        SessionOperationError::InvalidAtomNumberPair
        | SessionOperationError::InvalidAtomMarkSelector => {
            closed_operation_validation_error(py, "invalid_input", "atom_mark")
        }
        SessionOperationError::InvalidDirectHaworthInsertion(_)
        | SessionOperationError::InvalidRegularRingInsertion(_)
        | SessionOperationError::InvalidStandaloneHaworthInsertion(_)
        | SessionOperationError::InvalidStraightenDepiction(_) => {
            closed_operation_validation_error(py, "invalid_input", "presentation")
        }
        SessionOperationError::UnknownAtom(_) => unknown_document_object_error(py, "atom", None),
        SessionOperationError::UnknownBond(_) => unknown_document_object_error(py, "bond", None),
        SessionOperationError::UnknownPlus(_) => unknown_document_object_error(py, "plus", None),
        SessionOperationError::UnknownText(_) => unknown_document_object_error(py, "text", None),
        SessionOperationError::UnknownPresentationRoot(_) => {
            unknown_document_object_error(py, "presentation_root", None)
        }
        SessionOperationError::UnknownArrow(_) => unknown_document_object_error(py, "arrow", None),
        SessionOperationError::UnknownGeometricPresentation(_) => {
            unknown_document_object_error(py, "geometric_presentation", None)
        }
        SessionOperationError::UnknownWavy(identifier) => {
            unknown_document_object_error(py, "wavy", Some(identifier))
        }
        SessionOperationError::UnknownBracketPair(identifier) => {
            unknown_document_object_ids_error(py, "bracket_pair", identifier)
        }
        SessionOperationError::UnknownDocumentObject(_) => {
            unknown_document_object_error(py, "document_object", None)
        }
        SessionOperationError::InvalidCreateAtomTarget(_)
        | SessionOperationError::UnknownMolecule
        | SessionOperationError::PaperDimensionsRequireCustom
        | SessionOperationError::InvalidCreateBondTarget(_)
        | SessionOperationError::InvalidLiveChemicalTarget(_)
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
        | SessionOperationError::GroupIdentifierExhausted
        | SessionOperationError::PresentationIdentifierExhausted
        | SessionOperationError::FragmentImportIdentifierExhausted => {
            closed_operation_validation_error(py, "invalid_input", "document")
        }
        SessionOperationError::Candidate(error) => typed_document_error(py, error),
        SessionOperationError::Serialize(_) => {
            closed_operation_validation_error(py, "operation", "candidate")
        }
    }
}

pub(crate) fn operation_validation_error(py: Python<'_>, reason: String) -> PyErr {
    let error = OperationValidationError::new_err(reason.clone());
    let _ = error.value(py).setattr("reason", reason);
    error
}

fn closed_operation_validation_error(
    py: Python<'_>,
    category: &'static str,
    location: &'static str,
) -> PyResult<PyErr> {
    let py_error = OperationValidationError::new_err("document operation rejected");
    let value = py_error.value(py);
    value.setattr("category", category)?;
    value.setattr("location", location)?;
    Ok(py_error)
}

fn unknown_document_object_error(
    py: Python<'_>,
    location: &'static str,
    object_id: Option<DocumentObjectIdV1>,
) -> PyResult<PyErr> {
    let py_error = UnknownDocumentObjectError::new_err("document operation rejected");
    let value = py_error.value(py);
    value.setattr("category", "unknown_document_object")?;
    value.setattr("location", location)?;
    if let Some(object_id) = object_id {
        value.setattr("object_id", object_id.as_str().to_owned())?;
    }
    Ok(py_error)
}

fn unknown_document_object_ids_error(
    py: Python<'_>,
    location: &'static str,
    object_ids: [DocumentObjectIdV1; 2],
) -> PyResult<PyErr> {
    let py_error = UnknownDocumentObjectError::new_err("document operation rejected");
    let value = py_error.value(py);
    value.setattr("category", "unknown_document_object")?;
    value.setattr("location", location)?;
    value.setattr(
        "object_ids",
        object_ids
            .iter()
            .map(|object_id| object_id.as_str().to_owned())
            .collect::<Vec<_>>(),
    )?;
    Ok(py_error)
}

fn typed_document_error(py: Python<'_>, error: TypedDocumentError) -> PyResult<PyErr> {
    match error {
        TypedDocumentError::PresentationRootIsBracketMember(object_id) => {
            unknown_document_object_error(py, "presentation_bracket_member", Some(object_id))
        }
        TypedDocumentError::PartialBracketDeletion(object_ids) => {
            unknown_document_object_ids_error(py, "bracket_deletion", object_ids)
        }
        TypedDocumentError::PartialBracketStackSelection(object_ids) => {
            unknown_document_object_ids_error(py, "bracket_stack_selection", object_ids)
        }
        TypedDocumentError::PartialBracketTransform(object_ids) => {
            unknown_document_object_ids_error(py, "bracket_transform", object_ids)
        }
        TypedDocumentError::UnknownTopLevelTransformRoot(object_id) => {
            unknown_document_object_error(py, "top_level_transform", Some(object_id))
        }
        TypedDocumentError::InvalidTopLevelTransformGeometry(object_id) => {
            unknown_document_object_error(py, "top_level_transform_geometry", Some(object_id))
        }
        TypedDocumentError::NonFiniteTopLevelTransform(object_id) => {
            unknown_document_object_error(py, "top_level_transform_geometry", Some(object_id))
        }
        TypedDocumentError::InvalidBracketPair(object_ids) => {
            unknown_document_object_ids_error(py, "bracket_pair", object_ids)
        }
        _ => closed_operation_validation_error(py, "operation", "candidate"),
    }
}

pub(crate) fn publication_error(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_transition_core_refusal_maps_to_closed_operation_validation_error() {
        Python::initialize();
        Python::attach(|py| {
            let error = operation_error(
                py,
                SessionOperationError::PresentationCreateRequiresTransitionCore,
            )
            .expect("presentation transition-core refusal should map to Python");

            assert!(error.is_instance_of::<OperationValidationError>(py));
            assert_eq!(
                error
                    .value(py)
                    .getattr("category")
                    .expect("operation validation error should expose its category")
                    .extract::<String>()
                    .expect("operation validation category should be text"),
                "operation"
            );
            assert_eq!(
                error
                    .value(py)
                    .getattr("location")
                    .expect("operation validation error should expose its location")
                    .extract::<String>()
                    .expect("operation validation location should be text"),
                "transition"
            );
        });
    }
}
