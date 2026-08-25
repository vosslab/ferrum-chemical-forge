//! Closed Python selector for one direct-root presentation deletion.

use ferrum_document::{
    PresentationRecordKindV1, PresentationRootDeletionSetV1, PresentationRootDeletionV1,
    SessionOperation, SessionOperationV1,
};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::binding::operation_validation_error;
use super::document_error_binding::document_object_id as parse_document_object_id;

/// Closed direct-root record kinds accepted by presentation deletion.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DocumentPresentationRootKindV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyDocumentPresentationRootKindV1 {
    Arrow,
    Plus,
    Text,
    Polyline,
    Rectangle,
    Square,
    Oval,
    Circle,
    Polygon,
}

impl From<PyDocumentPresentationRootKindV1> for PresentationRecordKindV1 {
    fn from(value: PyDocumentPresentationRootKindV1) -> Self {
        match value {
            PyDocumentPresentationRootKindV1::Arrow => Self::Arrow,
            PyDocumentPresentationRootKindV1::Plus => Self::Plus,
            PyDocumentPresentationRootKindV1::Text => Self::Text,
            PyDocumentPresentationRootKindV1::Polyline => Self::Polyline,
            PyDocumentPresentationRootKindV1::Rectangle => Self::Rectangle,
            PyDocumentPresentationRootKindV1::Square => Self::Square,
            PyDocumentPresentationRootKindV1::Oval => Self::Oval,
            PyDocumentPresentationRootKindV1::Circle => Self::Circle,
            PyDocumentPresentationRootKindV1::Polygon => Self::Polygon,
        }
    }
}

pub(crate) fn delete_presentation_root(
    py: Python<'_>,
    document_object_id: String,
    kind: PyRef<'_, PyDocumentPresentationRootKindV1>,
) -> PyResult<SessionOperation> {
    let deletion = PresentationRootDeletionV1::new(
        parse_document_object_id(py, document_object_id)?,
        (*kind).into(),
    );
    Ok(SessionOperation::V1(
        SessionOperationV1::DeletePresentationRoot { deletion },
    ))
}

pub(crate) fn delete_presentation_roots(
    py: Python<'_>,
    targets: &Bound<'_, PyTuple>,
) -> PyResult<SessionOperation> {
    let targets = super::presentation_stack_binding::presentation_selectors(
        py,
        targets,
        "presentation deletion",
    )?;
    let deletions = targets
        .into_iter()
        .map(|target| {
            PresentationRootDeletionV1::new(target.document_object_id().clone(), target.kind())
        })
        .collect();
    let deletions = PresentationRootDeletionSetV1::new(deletions)
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(SessionOperation::V1(
        SessionOperationV1::DeletePresentationRoots { deletions },
    ))
}
