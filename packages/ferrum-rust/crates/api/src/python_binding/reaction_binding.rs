//! Frozen Python facts for renderer-preflighted Rust reaction authoring.

use crate::{
    ApiPreparedReactionLifecycleV1, ApiPreparedReactionTranslationV1,
    ApiReactionLifecycleGestureV1, ApiReactionTranslationGestureV1,
    ApiReactionTranslationPreviewV1, ReactionAuthoringChoiceAvailabilityV1,
    ReactionAuthoringChoiceKindV1, ReactionAuthoringChoicesV1, ReactionAuthoringExclusionReasonV1,
    ReactionAuthoringExclusionRecoveryV1, ReactionCreateRequestV1, ReactionDefinitionDispositionV1,
    ReactionGestureCategoryV1, ReactionGestureErrorV1, ReactionListObservationV1,
    ReactionMembershipPatchRequestV1, ReactionSelectionV1, RenderInteractionBoundsV1,
    RenderInteractionErrorV1, RenderInteractionGridSnapPolicyV1, RenderInteractionSnapV1,
    begin_api_reaction_definition_delete_v1, begin_api_reaction_gesture_v1,
    begin_api_reaction_membership_patch_v1, begin_api_reaction_translation_v1,
    commit_api_reaction_gesture_v1, commit_api_reaction_lifecycle_v1,
    commit_api_reaction_translation_v1, prepare_api_reaction_gesture_v1,
    prepare_api_reaction_lifecycle_v1, prepare_api_reaction_translation_v1,
    preview_api_reaction_translation_v1,
};
use ferrum_document::{DocumentFenceV1, ReactionDefinitionDiagnosticV1};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyModule, PyTuple};

use super::binding::{PyDocumentSession, PySessionOperationResultV1};

#[path = "reaction_binding_methods.rs"]
mod reaction_binding_methods;
#[path = "reaction_binding_support.rs"]
mod reaction_binding_support;

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ReactionRefusalCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyReactionRefusalCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    ReplayedGesture,
    InvalidRequest,
    MissingTarget,
    WrongTargetKind,
    DuplicateTarget,
    CrossReactionReuse,
    UnrenderableDocument,
    RenderPreparation,
    SessionConflict,
    MissingReaction,
    LegacyDefinitionNotEditable,
    MembershipChanged,
    RendererExclusion,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ReactionRefusalRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyReactionRefusalRecoveryV1 {
    RefreshAndRestart,
    CorrectSelectors,
    ChooseRenderableMembers,
    RepairLegacyDefinition,
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
enum PyReactionAuthoringChoiceKindV1 {
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
enum PyReactionAuthoringChoiceAvailabilityV1 {
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
enum PyReactionAuthoringExclusionReasonV1 {
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
enum PyReactionAuthoringExclusionRecoveryV1 {
    ChooseSupportedMember,
    RepairDocument,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ReactionAuthoringChoicesRefusalCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyReactionAuthoringChoicesRefusalCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    Observation,
}
create_exception!(
    ferrum_chem,
    ReactionGestureError,
    super::binding::DocumentError
);
create_exception!(
    ferrum_chem,
    ReactionAuthoringChoicesError,
    super::binding::DocumentError
);
#[pyclass(frozen, module = "ferrum_chem", name = "ReactionCreateCommitV1")]
struct PyReactionCreateCommitV1 {
    #[pyo3(get)]
    reaction_id: String,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "ReactionAuthoringChoiceBoundsV1",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyReactionAuthoringChoiceBoundsV1 {
    #[pyo3(get)]
    left: f64,
    #[pyo3(get)]
    top: f64,
    #[pyo3(get)]
    right: f64,
    #[pyo3(get)]
    bottom: f64,
}
#[pyclass(frozen, module = "ferrum_chem", name = "ReactionAuthoringChoiceV1")]
struct PyReactionAuthoringChoiceV1 {
    #[pyo3(get)]
    identifier: String,
    #[pyo3(get)]
    source_order: u32,
    #[pyo3(get)]
    kind: PyReactionAuthoringChoiceKindV1,
    #[pyo3(get)]
    availability: PyReactionAuthoringChoiceAvailabilityV1,
    #[pyo3(get)]
    label: String,
    #[pyo3(get)]
    bounds: PyReactionAuthoringChoiceBoundsV1,
}
#[pyclass(frozen, module = "ferrum_chem", name = "ReactionAuthoringExclusionV1")]
struct PyReactionAuthoringExclusionV1 {
    #[pyo3(get)]
    diagnostic_key: String,
    #[pyo3(get)]
    reason: PyReactionAuthoringExclusionReasonV1,
    #[pyo3(get)]
    recovery: PyReactionAuthoringExclusionRecoveryV1,
    #[pyo3(get)]
    label: String,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ReactionDefinitionDispositionV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyReactionDefinitionDispositionV1 {
    Strict,
    DisplayOnly,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ReactionDiagnosticReasonV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyReactionDiagnosticReasonV1 {
    MissingReactionId,
    EmptyReactionId,
    UnknownRoleChild,
    MissingIdref,
    EmptyIdref,
    MissingReactants,
    MissingProducts,
    MissingArrow,
    MultipleArrows,
    MissingTarget,
    UnrenderableMember,
    WrongTargetKind,
    DuplicateTarget,
    CrossReactionReuse,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ReactionDiagnosticRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyReactionDiagnosticRecoveryV1 {
    RepairDocument,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ReactionDiagnosticSelectorSourceV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyReactionDiagnosticSelectorSourceV1 {
    DirectCdmlSemanticIndex,
    RendererAdmission,
}
#[pyclass(frozen, module = "ferrum_chem", name = "ReactionDiagnosticV1")]
struct PyReactionDiagnosticV1 {
    #[pyo3(get)]
    reason: PyReactionDiagnosticReasonV1,
    #[pyo3(get)]
    recovery: PyReactionDiagnosticRecoveryV1,
    #[pyo3(get)]
    selector_source: PyReactionDiagnosticSelectorSourceV1,
}
#[pyclass(frozen, module = "ferrum_chem", name = "ReactionMemberObservationV1")]
struct PyReactionMemberObservationV1 {
    #[pyo3(get)]
    identifier: String,
    #[pyo3(get)]
    role: String,
    #[pyo3(get)]
    role_ordinal: u32,
    #[pyo3(get)]
    source_order: u32,
    #[pyo3(get)]
    bounds: Option<PyReactionAuthoringChoiceBoundsV1>,
}
#[pyclass(frozen, module = "ferrum_chem", name = "ReactionObservationV1")]
struct PyReactionObservationV1 {
    #[pyo3(get)]
    reaction_id: String,
    #[pyo3(get)]
    source_order: u32,
    #[pyo3(get)]
    disposition: PyReactionDefinitionDispositionV1,
    #[pyo3(get)]
    membership_digest: String,
    #[pyo3(get)]
    union_bounds: Option<PyReactionAuthoringChoiceBoundsV1>,
    diagnostics: Vec<Py<PyReactionDiagnosticV1>>,
    members: Vec<Py<PyReactionMemberObservationV1>>,
}
#[pymethods]
impl PyReactionObservationV1 {
    #[getter]
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        PyTuple::new(py, &self.diagnostics).map(Into::into)
    }
    #[getter]
    fn members(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        PyTuple::new(py, &self.members).map(Into::into)
    }
}
#[pyclass(unsendable, module = "ferrum_chem", name = "ReactionListObservationV1")]
struct PyReactionListObservationV1 {
    value: ReactionListObservationV1,
    reactions: Vec<Py<PyReactionObservationV1>>,
    #[pyo3(get)]
    revision: u64,
    #[pyo3(get)]
    digest: String,
}
#[pymethods]
impl PyReactionListObservationV1 {
    #[getter]
    fn reactions(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        PyTuple::new(py, &self.reactions).map(Into::into)
    }
}
#[pyclass(unsendable, module = "ferrum_chem", name = "ReactionSelectionV1")]
struct PyReactionSelectionV1 {
    value: ReactionSelectionV1,
    #[pyo3(get)]
    reaction_id: String,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "ReactionLifecycleGestureV1"
)]
struct PyReactionLifecycleGestureV1 {
    value: ApiReactionLifecycleGestureV1,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PreparedReactionLifecycleV1"
)]
struct PyPreparedReactionLifecycleV1 {
    value: ApiPreparedReactionLifecycleV1,
}
#[pyclass(frozen, module = "ferrum_chem", name = "ReactionLifecycleCommitV1")]
struct PyReactionLifecycleCommitV1 {
    #[pyo3(get)]
    reaction_id: String,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "ReactionTranslationGestureV1"
)]
struct PyReactionTranslationGestureV1 {
    value: ApiReactionTranslationGestureV1,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "ReactionTranslationPreviewV1"
)]
struct PyReactionTranslationPreviewV1 {
    value: ApiReactionTranslationPreviewV1,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PreparedReactionTranslationV1"
)]
struct PyPreparedReactionTranslationV1 {
    value: ApiPreparedReactionTranslationV1,
}
#[pyclass(frozen, module = "ferrum_chem", name = "ReactionTranslationCommitV1")]
struct PyReactionTranslationCommitV1 {
    #[pyo3(get)]
    reaction_id: String,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "ReactionAuthoringChoicesV1"
)]
struct PyReactionAuthoringChoicesV1 {
    value: ReactionAuthoringChoicesV1,
    choices: Vec<Py<PyReactionAuthoringChoiceV1>>,
    exclusions: Vec<Py<PyReactionAuthoringExclusionV1>>,
    #[pyo3(get)]
    revision: u64,
    #[pyo3(get)]
    digest: String,
}
#[pymethods]
impl PyReactionAuthoringChoicesV1 {
    #[getter]
    fn choices(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        PyTuple::new(py, &self.choices).map(Into::into)
    }
    #[getter]
    fn exclusions(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        PyTuple::new(py, &self.exclusions).map(Into::into)
    }
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "ReactionGestureError",
        module.py().get_type::<ReactionGestureError>(),
    )?;
    module.add(
        "ReactionAuthoringChoicesError",
        module.py().get_type::<ReactionAuthoringChoicesError>(),
    )?;
    module.add_class::<PyReactionRefusalCategoryV1>()?;
    module.add_class::<PyReactionRefusalRecoveryV1>()?;
    module.add_class::<PyReactionCreateCommitV1>()?;
    module.add_class::<PyReactionAuthoringChoiceKindV1>()?;
    module.add_class::<PyReactionAuthoringChoiceAvailabilityV1>()?;
    module.add_class::<PyReactionAuthoringExclusionReasonV1>()?;
    module.add_class::<PyReactionAuthoringExclusionRecoveryV1>()?;
    module.add_class::<PyReactionAuthoringChoicesRefusalCategoryV1>()?;
    module.add_class::<PyReactionAuthoringChoiceBoundsV1>()?;
    module.add_class::<PyReactionAuthoringChoiceV1>()?;
    module.add_class::<PyReactionAuthoringExclusionV1>()?;
    module.add_class::<PyReactionDefinitionDispositionV1>()?;
    module.add_class::<PyReactionDiagnosticReasonV1>()?;
    module.add_class::<PyReactionDiagnosticRecoveryV1>()?;
    module.add_class::<PyReactionDiagnosticSelectorSourceV1>()?;
    module.add_class::<PyReactionDiagnosticV1>()?;
    module.add_class::<PyReactionMemberObservationV1>()?;
    module.add_class::<PyReactionObservationV1>()?;
    module.add_class::<PyReactionListObservationV1>()?;
    module.add_class::<PyReactionSelectionV1>()?;
    module.add_class::<PyReactionLifecycleGestureV1>()?;
    module.add_class::<PyPreparedReactionLifecycleV1>()?;
    module.add_class::<PyReactionLifecycleCommitV1>()?;
    module.add_class::<PyReactionTranslationGestureV1>()?;
    module.add_class::<PyReactionTranslationPreviewV1>()?;
    module.add_class::<PyPreparedReactionTranslationV1>()?;
    module.add_class::<PyReactionTranslationCommitV1>()?;
    module.add_class::<PyReactionAuthoringChoicesV1>()
}
