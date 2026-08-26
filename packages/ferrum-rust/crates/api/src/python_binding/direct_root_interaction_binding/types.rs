//! Python-facing interaction enums and semantic mappings.

use crate::{
    ReactionAuthoringChoiceAvailabilityV1, ReactionAuthoringChoiceKindV1,
    ReactionAuthoringExclusionReasonV1, ReactionAuthoringExclusionRecoveryV1,
    RenderInteractionAxisV1, RenderInteractionGridSnapPolicyV1, RenderInteractionModifierV1,
    StructureTargetKindV1,
};
use ferrum_document::TopLevelRootKindV1;
use pyo3::prelude::*;

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "RenderInteractionCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum PyCategory {
    StaleRevision,
    StaleDigest,
    ForeignSession,
    SelectionChanged,
    EmptySelection,
    NonFinitePoint,
    InvalidRectangle,
    NoTarget,
    UnrenderableDepiction,
    AmbiguousRootIdentifier,
    DisplayOnly,
    Observation,
    SessionConflict,
    RendererAdmission,
    UnrenderableCandidate,
    CrossMoleculeSelection,
    UnsupportedTarget,
    InvalidCompactGroupDeletionSelection,
    InvalidCompactGroupDeletionTopology,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "RenderInteractionRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum PyRecovery {
    RefreshAndRestart,
    SelectRenderableRoot,
    CorrectInput,
    ChangePresentation,
    ReportConflict,
    RepairDocument,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "RenderInteractionExclusionReasonV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum PyExclusionReason {
    UnrenderableDepiction,
    AmbiguousRootIdentifier,
    DisplayOnly,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "RenderInteractionModifierV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum PyModifier {
    Replace,
    Toggle,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "RenderInteractionAxisV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum PyAxis {
    Free,
    Horizontal,
    Vertical,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "RenderInteractionGridSnapPolicyV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum PyGridSnapPolicy {
    Free,
    ViewHexGrid,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "StructureTargetKindV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum PyStructureTargetKind {
    Atom,
    Bond,
    CompactGroup,
    DisplayOnly,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "TopLevelRootKindV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum PyRootKind {
    Molecule,
    Arrow,
    Plus,
    Text,
    Rectangle,
    Square,
    Oval,
    Circle,
    Polygon,
    Polyline,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ReactionAuthoringChoiceKindV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum PyReactionChoiceKind {
    Molecule,
    Arrow,
    Plus,
    ConditionText,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ReactionAuthoringChoiceAvailabilityV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum PyReactionChoiceAvailability {
    Eligible,
    AlreadyInReaction,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ReactionAuthoringExclusionReasonV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum PyReactionExclusionReason {
    DisplayOnly,
    Unrenderable,
    MissingSemanticIdentity,
    AmbiguousSemanticIdentity,
    KindMismatch,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ReactionAuthoringExclusionRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum PyReactionExclusionRecovery {
    ChooseSupportedMember,
    RepairDocument,
}
impl From<PyModifier> for RenderInteractionModifierV1 {
    fn from(value: PyModifier) -> Self {
        match value {
            PyModifier::Replace => Self::Replace,
            PyModifier::Toggle => Self::Toggle,
        }
    }
}
impl From<PyAxis> for RenderInteractionAxisV1 {
    fn from(value: PyAxis) -> Self {
        match value {
            PyAxis::Free => Self::Free,
            PyAxis::Horizontal => Self::Horizontal,
            PyAxis::Vertical => Self::Vertical,
        }
    }
}
impl From<PyGridSnapPolicy> for RenderInteractionGridSnapPolicyV1 {
    fn from(value: PyGridSnapPolicy) -> Self {
        match value {
            PyGridSnapPolicy::Free => Self::Free,
            PyGridSnapPolicy::ViewHexGrid => Self::ViewHexGrid,
        }
    }
}
pub(super) fn structure_kind(value: StructureTargetKindV1) -> PyStructureTargetKind {
    match value {
        StructureTargetKindV1::Atom => PyStructureTargetKind::Atom,
        StructureTargetKindV1::Bond => PyStructureTargetKind::Bond,
        StructureTargetKindV1::CompactGroup => PyStructureTargetKind::CompactGroup,
        StructureTargetKindV1::DisplayOnly => PyStructureTargetKind::DisplayOnly,
    }
}
pub(super) fn root_kind(value: TopLevelRootKindV1) -> PyRootKind {
    match value {
        TopLevelRootKindV1::Molecule => PyRootKind::Molecule,
        TopLevelRootKindV1::Arrow => PyRootKind::Arrow,
        TopLevelRootKindV1::Plus => PyRootKind::Plus,
        TopLevelRootKindV1::Text => PyRootKind::Text,
        TopLevelRootKindV1::Rectangle => PyRootKind::Rectangle,
        TopLevelRootKindV1::Square => PyRootKind::Square,
        TopLevelRootKindV1::Oval => PyRootKind::Oval,
        TopLevelRootKindV1::Circle => PyRootKind::Circle,
        TopLevelRootKindV1::Polygon => PyRootKind::Polygon,
        TopLevelRootKindV1::Polyline => PyRootKind::Polyline,
    }
}
pub(super) fn reaction_choice_kind(value: ReactionAuthoringChoiceKindV1) -> PyReactionChoiceKind {
    match value {
        ReactionAuthoringChoiceKindV1::Molecule => PyReactionChoiceKind::Molecule,
        ReactionAuthoringChoiceKindV1::Arrow => PyReactionChoiceKind::Arrow,
        ReactionAuthoringChoiceKindV1::Plus => PyReactionChoiceKind::Plus,
        ReactionAuthoringChoiceKindV1::ConditionText => PyReactionChoiceKind::ConditionText,
    }
}
pub(super) fn reaction_choice_availability(
    value: ReactionAuthoringChoiceAvailabilityV1,
) -> PyReactionChoiceAvailability {
    match value {
        ReactionAuthoringChoiceAvailabilityV1::Eligible => PyReactionChoiceAvailability::Eligible,
        ReactionAuthoringChoiceAvailabilityV1::AlreadyInReaction => {
            PyReactionChoiceAvailability::AlreadyInReaction
        }
    }
}
pub(super) fn reaction_exclusion_reason(
    value: ReactionAuthoringExclusionReasonV1,
) -> PyReactionExclusionReason {
    match value {
        ReactionAuthoringExclusionReasonV1::DisplayOnly => PyReactionExclusionReason::DisplayOnly,
        ReactionAuthoringExclusionReasonV1::Unrenderable => PyReactionExclusionReason::Unrenderable,
        ReactionAuthoringExclusionReasonV1::MissingSemanticIdentity => {
            PyReactionExclusionReason::MissingSemanticIdentity
        }
        ReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity => {
            PyReactionExclusionReason::AmbiguousSemanticIdentity
        }
        ReactionAuthoringExclusionReasonV1::KindMismatch => PyReactionExclusionReason::KindMismatch,
    }
}
pub(super) fn reaction_exclusion_recovery(
    value: ReactionAuthoringExclusionRecoveryV1,
) -> PyReactionExclusionRecovery {
    match value {
        ReactionAuthoringExclusionRecoveryV1::ChooseSupportedMember => {
            PyReactionExclusionRecovery::ChooseSupportedMember
        }
        ReactionAuthoringExclusionRecoveryV1::RepairDocument => {
            PyReactionExclusionRecovery::RepairDocument
        }
    }
}
