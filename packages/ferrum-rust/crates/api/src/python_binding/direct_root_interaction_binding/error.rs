//! Python exception classification for direct-root interaction failures.

use super::{
    super::binding::RevisionConflictError,
    types::{PyCategory, PyRecovery},
};
use crate::RenderInteractionErrorV1;
use pyo3::{create_exception, prelude::*};

create_exception!(
    ferrum_chem,
    RenderInteractionError,
    super::super::binding::DocumentError
);

pub(super) fn category(error: &RenderInteractionErrorV1) -> PyCategory {
    match error {
        RenderInteractionErrorV1::StaleRevision => PyCategory::StaleRevision,
        RenderInteractionErrorV1::StaleDigest => PyCategory::StaleDigest,
        RenderInteractionErrorV1::ForeignSession => PyCategory::ForeignSession,
        RenderInteractionErrorV1::SelectionChanged => PyCategory::SelectionChanged,
        RenderInteractionErrorV1::EmptySelection => PyCategory::EmptySelection,
        RenderInteractionErrorV1::NonFinitePoint => PyCategory::NonFinitePoint,
        RenderInteractionErrorV1::InvalidRectangle => PyCategory::InvalidRectangle,
        RenderInteractionErrorV1::NoTarget => PyCategory::NoTarget,
        RenderInteractionErrorV1::UnrenderableDepiction => PyCategory::UnrenderableDepiction,
        RenderInteractionErrorV1::AmbiguousRootIdentifier => PyCategory::AmbiguousRootIdentifier,
        RenderInteractionErrorV1::DisplayOnly => PyCategory::DisplayOnly,
        RenderInteractionErrorV1::Observation => PyCategory::Observation,
        RenderInteractionErrorV1::SessionConflict => PyCategory::SessionConflict,
        RenderInteractionErrorV1::RendererAdmission => PyCategory::RendererAdmission,
        RenderInteractionErrorV1::UnrenderableCandidate => PyCategory::UnrenderableCandidate,
        RenderInteractionErrorV1::CrossMoleculeSelection => PyCategory::CrossMoleculeSelection,
        RenderInteractionErrorV1::UnsupportedTarget => PyCategory::UnsupportedTarget,
        RenderInteractionErrorV1::InvalidCompactGroupDeletionSelection => {
            PyCategory::InvalidCompactGroupDeletionSelection
        }
        RenderInteractionErrorV1::InvalidCompactGroupDeletionTopology => {
            PyCategory::InvalidCompactGroupDeletionTopology
        }
        RenderInteractionErrorV1::UnsupportedDocument => PyCategory::Observation,
    }
}
pub(super) fn recovery(error: &RenderInteractionErrorV1) -> PyRecovery {
    match error {
        RenderInteractionErrorV1::StaleRevision
        | RenderInteractionErrorV1::StaleDigest
        | RenderInteractionErrorV1::ForeignSession
        | RenderInteractionErrorV1::SelectionChanged => PyRecovery::RefreshAndRestart,
        RenderInteractionErrorV1::EmptySelection
        | RenderInteractionErrorV1::NoTarget
        | RenderInteractionErrorV1::CrossMoleculeSelection
        | RenderInteractionErrorV1::InvalidCompactGroupDeletionSelection => {
            PyRecovery::SelectRenderableRoot
        }
        RenderInteractionErrorV1::InvalidCompactGroupDeletionTopology => PyRecovery::RepairDocument,
        RenderInteractionErrorV1::NonFinitePoint | RenderInteractionErrorV1::InvalidRectangle => {
            PyRecovery::CorrectInput
        }
        RenderInteractionErrorV1::UnrenderableDepiction
        | RenderInteractionErrorV1::AmbiguousRootIdentifier
        | RenderInteractionErrorV1::DisplayOnly
        | RenderInteractionErrorV1::UnrenderableCandidate
        | RenderInteractionErrorV1::UnsupportedTarget => PyRecovery::ChangePresentation,
        RenderInteractionErrorV1::Observation | RenderInteractionErrorV1::SessionConflict => {
            PyRecovery::ReportConflict
        }
        RenderInteractionErrorV1::RendererAdmission => PyRecovery::ChangePresentation,
        RenderInteractionErrorV1::UnsupportedDocument => PyRecovery::ChangePresentation,
    }
}
pub(super) fn interaction_error(py: Python<'_>, error: RenderInteractionErrorV1) -> PyErr {
    let exception = match error {
        RenderInteractionErrorV1::StaleRevision | RenderInteractionErrorV1::StaleDigest => {
            RevisionConflictError::new_err(error.to_string())
        }
        _ => RenderInteractionError::new_err(error.to_string()),
    };
    let instance = exception.value(py);
    instance
        .setattr(
            "category",
            Py::new(py, category(&error)).expect("enum allocates"),
        )
        .expect("category attaches");
    instance
        .setattr(
            "recovery",
            Py::new(py, recovery(&error)).expect("enum allocates"),
        )
        .expect("recovery attaches");
    exception
}
