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

#[pymethods]
impl PyDocumentSession {
    /// Return source-ordered immutable reaction membership and renderer facts.
    fn observe_reaction_list_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
    ) -> PyResult<PyReactionListObservationV1> {
        let fence = authoring_fence(&expected_digest_hex, expected_revision)?;
        self.session
            .observe_reaction_list_v1(fence)
            .map_err(|error| authoring_error(py, error))
            .and_then(|value| reaction_list(py, value))
    }
    /// Issue one opaque strict reaction selection from a fresh list observation.
    fn select_reaction_v1(
        &self,
        py: Python<'_>,
        list: PyRef<'_, PyReactionListObservationV1>,
        reaction_id: String,
    ) -> PyResult<PyReactionSelectionV1> {
        self.session
            .select_reaction_v1(&list.value, &reaction_id)
            .map(|value| PyReactionSelectionV1 {
                reaction_id: value.reaction_id().to_owned(),
                value,
            })
            .map_err(|error| authoring_error(py, error))
    }
    /// Refuse foreign or stale opaque reaction selection without changing CDML.
    fn validate_reaction_selection_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PyReactionSelectionV1>,
    ) -> PyResult<()> {
        self.session
            .validate_reaction_selection_v1(&selection.value)
            .map_err(|error| authoring_error(py, error))
    }
    /// Begin a complete membership replacement from one opaque strict selection.
    #[allow(clippy::too_many_arguments)]
    fn begin_reaction_membership_patch_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PyReactionSelectionV1>,
        expected_revision: u64,
        reactants: Vec<String>,
        products: Vec<String>,
        arrow: String,
        conditions: Vec<String>,
        pluses: Vec<String>,
    ) -> PyResult<PyReactionLifecycleGestureV1> {
        let request = ReactionMembershipPatchRequestV1::new(
            expected_revision,
            reactants,
            products,
            arrow,
            conditions,
            pluses,
        )
        .map_err(|error| gesture_error(py, error))?;
        begin_api_reaction_membership_patch_v1(&self.session, &selection.value, request)
            .map(|value| PyReactionLifecycleGestureV1 { value })
            .map_err(|error| gesture_error(py, error))
    }
    /// Begin removal of only one selected strict reaction definition.
    fn begin_reaction_definition_delete_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PyReactionSelectionV1>,
    ) -> PyResult<PyReactionLifecycleGestureV1> {
        begin_api_reaction_definition_delete_v1(&self.session, &selection.value)
            .map(|value| PyReactionLifecycleGestureV1 { value })
            .map_err(|error| gesture_error(py, error))
    }
    /// Renderer-preflight a private lifecycle candidate; no document mutation occurs here.
    fn prepare_reaction_lifecycle_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyReactionLifecycleGestureV1>,
    ) -> PyResult<PyPreparedReactionLifecycleV1> {
        prepare_api_reaction_lifecycle_v1(&mut self.session, &gesture.value)
            .map(|value| PyPreparedReactionLifecycleV1 { value })
            .map_err(|error| gesture_error(py, error))
    }
    /// Commit exactly one prepared lifecycle receipt.
    fn commit_reaction_lifecycle_v1(
        &mut self,
        py: Python<'_>,
        mut prepared: PyRefMut<'_, PyPreparedReactionLifecycleV1>,
    ) -> PyResult<PyReactionLifecycleCommitV1> {
        commit_api_reaction_lifecycle_v1(&mut self.session, &mut prepared.value)
            .map(|commit| PyReactionLifecycleCommitV1 {
                reaction_id: commit.reaction_id().to_owned(),
                result: commit.result().clone().into(),
            })
            .map_err(|error| gesture_error(py, error))
    }
    /// Begin one opaque aggregate translation from an exact strict selection.
    #[pyo3(signature = (selection, press_x, press_y, view_hex_grid=false))]
    fn begin_reaction_translation_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PyReactionSelectionV1>,
        press_x: f64,
        press_y: f64,
        view_hex_grid: bool,
    ) -> PyResult<PyReactionTranslationGestureV1> {
        let snap = if view_hex_grid {
            RenderInteractionSnapV1::with_grid(RenderInteractionGridSnapPolicyV1::ViewHexGrid)
        } else {
            RenderInteractionSnapV1::free()
        };
        begin_api_reaction_translation_v1(&self.session, &selection.value, press_x, press_y, snap)
            .map(|value| PyReactionTranslationGestureV1 { value })
            .map_err(|error| gesture_error(py, error))
    }
    /// Compute a transient Rust-owned translation preview without changing CDML.
    fn preview_reaction_translation_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyReactionTranslationGestureV1>,
        pointer_x: f64,
        pointer_y: f64,
    ) -> PyResult<PyReactionTranslationPreviewV1> {
        preview_api_reaction_translation_v1(&self.session, &gesture.value, pointer_x, pointer_y)
            .map(|value| PyReactionTranslationPreviewV1 { value })
            .map_err(|error| gesture_error(py, error))
    }
    /// Renderer-preflight a selected-reaction translation candidate without mutation.
    fn prepare_reaction_translation_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyReactionTranslationGestureV1>,
        preview: PyRef<'_, PyReactionTranslationPreviewV1>,
    ) -> PyResult<PyPreparedReactionTranslationV1> {
        prepare_api_reaction_translation_v1(&mut self.session, &gesture.value, &preview.value)
            .map(|value| PyPreparedReactionTranslationV1 { value })
            .map_err(|error| gesture_error(py, error))
    }
    /// Commit exactly one renderer-admitted aggregate translation receipt.
    fn commit_reaction_translation_v1(
        &mut self,
        py: Python<'_>,
        mut prepared: PyRefMut<'_, PyPreparedReactionTranslationV1>,
    ) -> PyResult<PyReactionTranslationCommitV1> {
        commit_api_reaction_translation_v1(&mut self.session, &mut prepared.value)
            .map(|commit| PyReactionTranslationCommitV1 {
                reaction_id: commit.reaction_id().to_owned(),
                result: commit.result().clone().into(),
            })
            .map_err(|error| gesture_error(py, error))
    }
    /// Return immutable renderer-observed reaction-member choices for one exact snapshot.
    fn observe_reaction_authoring_choices_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
    ) -> PyResult<PyReactionAuthoringChoicesV1> {
        let fence = authoring_fence(&expected_digest_hex, expected_revision)?;
        self.session
            .observe_reaction_authoring_choices_v1(fence)
            .map_err(|error| authoring_error(py, error))
            .and_then(|value| authoring_choices(py, value))
    }
    /// Refuse a stale or foreign immutable authoring observation without changing CDML.
    fn validate_reaction_authoring_choices_v1(
        &self,
        py: Python<'_>,
        choices: PyRef<'_, PyReactionAuthoringChoicesV1>,
    ) -> PyResult<()> {
        self.session
            .validate_reaction_authoring_choices_v1(&choices.value)
            .map_err(|error| authoring_error(py, error))
    }
    /// Create one reaction only through the renderer-preflighted native bridge.
    // This positional PyO3 API preserves the established public reaction protocol.
    #[allow(clippy::too_many_arguments)]
    fn create_reaction_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        reactants: Vec<String>,
        products: Vec<String>,
        arrow: String,
        conditions: Vec<String>,
        pluses: Vec<String>,
    ) -> PyResult<PyReactionCreateCommitV1> {
        let request = ReactionCreateRequestV1::new(
            expected_revision,
            reactants,
            products,
            arrow,
            conditions,
            pluses,
        )
        .map_err(|error| gesture_error(py, error))?;
        let snapshot = self
            .session
            .snapshot()
            .map_err(|error| ReactionGestureError::new_err(error.to_string()))?;
        let fence = DocumentFenceV1::new(expected_revision, *snapshot.digest());
        let gesture = begin_api_reaction_gesture_v1(&self.session, fence, request)
            .map_err(|error| gesture_error(py, error))?;
        let mut prepared = prepare_api_reaction_gesture_v1(&mut self.session, &gesture)
            .map_err(|error| gesture_error(py, error))?;
        commit_api_reaction_gesture_v1(&mut self.session, &mut prepared)
            .map(|commit| PyReactionCreateCommitV1 {
                reaction_id: commit.reaction_id().to_owned(),
                result: commit.result().clone().into(),
            })
            .map_err(|error| gesture_error(py, error))
    }
}

fn authoring_fence(digest: &str, revision: u64) -> PyResult<DocumentFenceV1> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(ReactionAuthoringChoicesError::new_err(
            "expected digest must be exactly 64 lowercase hexadecimal characters",
        ));
    }
    let mut bytes = [0; 32];
    for (index, pair) in digest.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (authoring_hex(pair[0]) << 4) | authoring_hex(pair[1]);
    }
    Ok(DocumentFenceV1::new(revision, bytes))
}
const fn authoring_hex(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}
fn choice_kind(value: ReactionAuthoringChoiceKindV1) -> PyReactionAuthoringChoiceKindV1 {
    match value {
        ReactionAuthoringChoiceKindV1::Molecule => PyReactionAuthoringChoiceKindV1::Molecule,
        ReactionAuthoringChoiceKindV1::Arrow => PyReactionAuthoringChoiceKindV1::Arrow,
        ReactionAuthoringChoiceKindV1::Plus => PyReactionAuthoringChoiceKindV1::Plus,
        ReactionAuthoringChoiceKindV1::ConditionText => {
            PyReactionAuthoringChoiceKindV1::ConditionText
        }
    }
}
fn choice_availability(
    value: ReactionAuthoringChoiceAvailabilityV1,
) -> PyReactionAuthoringChoiceAvailabilityV1 {
    match value {
        ReactionAuthoringChoiceAvailabilityV1::Eligible => {
            PyReactionAuthoringChoiceAvailabilityV1::Eligible
        }
        ReactionAuthoringChoiceAvailabilityV1::AlreadyInReaction => {
            PyReactionAuthoringChoiceAvailabilityV1::AlreadyInReaction
        }
    }
}
fn exclusion_reason(
    value: ReactionAuthoringExclusionReasonV1,
) -> PyReactionAuthoringExclusionReasonV1 {
    match value {
        ReactionAuthoringExclusionReasonV1::DisplayOnly => {
            PyReactionAuthoringExclusionReasonV1::DisplayOnly
        }
        ReactionAuthoringExclusionReasonV1::Unrenderable => {
            PyReactionAuthoringExclusionReasonV1::Unrenderable
        }
        ReactionAuthoringExclusionReasonV1::MissingSemanticIdentity => {
            PyReactionAuthoringExclusionReasonV1::MissingSemanticIdentity
        }
        ReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity => {
            PyReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity
        }
        ReactionAuthoringExclusionReasonV1::KindMismatch => {
            PyReactionAuthoringExclusionReasonV1::KindMismatch
        }
    }
}
fn exclusion_recovery(
    value: ReactionAuthoringExclusionRecoveryV1,
) -> PyReactionAuthoringExclusionRecoveryV1 {
    match value {
        ReactionAuthoringExclusionRecoveryV1::ChooseSupportedMember => {
            PyReactionAuthoringExclusionRecoveryV1::ChooseSupportedMember
        }
        ReactionAuthoringExclusionRecoveryV1::RepairDocument => {
            PyReactionAuthoringExclusionRecoveryV1::RepairDocument
        }
    }
}
fn choice_bounds(value: RenderInteractionBoundsV1) -> PyReactionAuthoringChoiceBoundsV1 {
    PyReactionAuthoringChoiceBoundsV1 {
        left: value.left(),
        top: value.top(),
        right: value.right(),
        bottom: value.bottom(),
    }
}
fn reaction_diagnostic(value: ReactionDefinitionDiagnosticV1) -> PyReactionDiagnosticV1 {
    let reason = match value {
        ReactionDefinitionDiagnosticV1::MissingReactionId => {
            PyReactionDiagnosticReasonV1::MissingReactionId
        }
        ReactionDefinitionDiagnosticV1::EmptyReactionId => {
            PyReactionDiagnosticReasonV1::EmptyReactionId
        }
        ReactionDefinitionDiagnosticV1::UnknownRoleChild => {
            PyReactionDiagnosticReasonV1::UnknownRoleChild
        }
        ReactionDefinitionDiagnosticV1::MissingIdref => PyReactionDiagnosticReasonV1::MissingIdref,
        ReactionDefinitionDiagnosticV1::EmptyIdref => PyReactionDiagnosticReasonV1::EmptyIdref,
        ReactionDefinitionDiagnosticV1::MissingReactants => {
            PyReactionDiagnosticReasonV1::MissingReactants
        }
        ReactionDefinitionDiagnosticV1::MissingProducts => {
            PyReactionDiagnosticReasonV1::MissingProducts
        }
        ReactionDefinitionDiagnosticV1::MissingArrow => PyReactionDiagnosticReasonV1::MissingArrow,
        ReactionDefinitionDiagnosticV1::MultipleArrows => {
            PyReactionDiagnosticReasonV1::MultipleArrows
        }
        ReactionDefinitionDiagnosticV1::MissingTarget => {
            PyReactionDiagnosticReasonV1::MissingTarget
        }
        ReactionDefinitionDiagnosticV1::UnrenderableMember => {
            PyReactionDiagnosticReasonV1::UnrenderableMember
        }
        ReactionDefinitionDiagnosticV1::WrongTargetKind => {
            PyReactionDiagnosticReasonV1::WrongTargetKind
        }
        ReactionDefinitionDiagnosticV1::DuplicateTarget => {
            PyReactionDiagnosticReasonV1::DuplicateTarget
        }
        ReactionDefinitionDiagnosticV1::CrossReactionReuse => {
            PyReactionDiagnosticReasonV1::CrossReactionReuse
        }
    };
    let selector_source = match value {
        ReactionDefinitionDiagnosticV1::UnrenderableMember => {
            PyReactionDiagnosticSelectorSourceV1::RendererAdmission
        }
        _ => PyReactionDiagnosticSelectorSourceV1::DirectCdmlSemanticIndex,
    };
    PyReactionDiagnosticV1 {
        reason,
        recovery: PyReactionDiagnosticRecoveryV1::RepairDocument,
        selector_source,
    }
}
fn authoring_choices(
    py: Python<'_>,
    value: ReactionAuthoringChoicesV1,
) -> PyResult<PyReactionAuthoringChoicesV1> {
    let fence = value.fence();
    let choices = value
        .choices()
        .iter()
        .map(|choice| {
            Py::new(
                py,
                PyReactionAuthoringChoiceV1 {
                    identifier: choice.identifier().to_owned(),
                    source_order: choice.source_order(),
                    kind: choice_kind(choice.kind()),
                    availability: choice_availability(choice.availability()),
                    label: choice.label().to_owned(),
                    bounds: choice_bounds(choice.bounds()),
                },
            )
        })
        .collect::<PyResult<_>>()?;
    let exclusions = value
        .exclusions()
        .iter()
        .map(|exclusion| {
            Py::new(
                py,
                PyReactionAuthoringExclusionV1 {
                    diagnostic_key: exclusion.diagnostic_key().to_owned(),
                    reason: exclusion_reason(exclusion.reason()),
                    recovery: exclusion_recovery(exclusion.recovery()),
                    label: exclusion.label().to_owned(),
                },
            )
        })
        .collect::<PyResult<_>>()?;
    Ok(PyReactionAuthoringChoicesV1 {
        value,
        choices,
        exclusions,
        revision: fence.revision(),
        digest: fence
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    })
}
fn reaction_list(
    py: Python<'_>,
    value: ReactionListObservationV1,
) -> PyResult<PyReactionListObservationV1> {
    let fence = value.fence();
    let reactions = value
        .reactions()
        .iter()
        .map(|reaction| {
            let diagnostics = reaction
                .diagnostics()
                .iter()
                .map(|diagnostic| Py::new(py, reaction_diagnostic(*diagnostic)))
                .collect::<PyResult<_>>()?;
            let members = reaction
                .members()
                .iter()
                .map(|member| {
                    Py::new(
                        py,
                        PyReactionMemberObservationV1 {
                            identifier: member.identifier().to_owned(),
                            role: member.role().local_name().to_owned(),
                            role_ordinal: member.role_ordinal(),
                            source_order: member.source_order(),
                            bounds: member.bounds().map(choice_bounds),
                        },
                    )
                })
                .collect::<PyResult<_>>()?;
            Py::new(
                py,
                PyReactionObservationV1 {
                    reaction_id: reaction.reaction_id().to_owned(),
                    source_order: reaction.source_order(),
                    disposition: match reaction.disposition() {
                        ReactionDefinitionDispositionV1::Strict => {
                            PyReactionDefinitionDispositionV1::Strict
                        }
                        ReactionDefinitionDispositionV1::DisplayOnly => {
                            PyReactionDefinitionDispositionV1::DisplayOnly
                        }
                    },
                    membership_digest: reaction.membership_digest().to_owned(),
                    union_bounds: reaction.union_bounds().map(choice_bounds),
                    diagnostics,
                    members,
                },
            )
        })
        .collect::<PyResult<_>>()?;
    Ok(PyReactionListObservationV1 {
        value,
        reactions,
        revision: fence.revision(),
        digest: fence
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    })
}
fn authoring_error(py: Python<'_>, error: RenderInteractionErrorV1) -> PyErr {
    let category = match error {
        RenderInteractionErrorV1::StaleRevision | RenderInteractionErrorV1::StaleDigest => {
            PyReactionAuthoringChoicesRefusalCategoryV1::StaleSnapshot
        }
        RenderInteractionErrorV1::ForeignSession => {
            PyReactionAuthoringChoicesRefusalCategoryV1::ForeignSession
        }
        _ => PyReactionAuthoringChoicesRefusalCategoryV1::Observation,
    };
    let exception = ReactionAuthoringChoicesError::new_err(error.to_string());
    exception
        .value(py)
        .setattr(
            "category",
            Py::new(py, category).expect("category allocates"),
        )
        .expect("category attaches");
    exception
}

fn gesture_error(py: Python<'_>, error: ReactionGestureErrorV1) -> PyErr {
    reaction_error(py, error.category(), error.to_string())
}
fn reaction_error(py: Python<'_>, category: ReactionGestureCategoryV1, message: String) -> PyErr {
    let category = match category {
        ReactionGestureCategoryV1::StaleSnapshot => PyReactionRefusalCategoryV1::StaleSnapshot,
        ReactionGestureCategoryV1::ForeignSession => PyReactionRefusalCategoryV1::ForeignSession,
        ReactionGestureCategoryV1::ReplayedGesture => PyReactionRefusalCategoryV1::ReplayedGesture,
        ReactionGestureCategoryV1::InvalidRequest => PyReactionRefusalCategoryV1::InvalidRequest,
        ReactionGestureCategoryV1::MissingTarget => PyReactionRefusalCategoryV1::MissingTarget,
        ReactionGestureCategoryV1::WrongTargetKind => PyReactionRefusalCategoryV1::WrongTargetKind,
        ReactionGestureCategoryV1::DuplicateTarget => PyReactionRefusalCategoryV1::DuplicateTarget,
        ReactionGestureCategoryV1::CrossReactionReuse => {
            PyReactionRefusalCategoryV1::CrossReactionReuse
        }
        ReactionGestureCategoryV1::UnrenderableDocument => {
            PyReactionRefusalCategoryV1::UnrenderableDocument
        }
        ReactionGestureCategoryV1::RenderPreparation => {
            PyReactionRefusalCategoryV1::RenderPreparation
        }
        ReactionGestureCategoryV1::SessionConflict => PyReactionRefusalCategoryV1::SessionConflict,
        ReactionGestureCategoryV1::MissingReaction => PyReactionRefusalCategoryV1::MissingReaction,
        ReactionGestureCategoryV1::LegacyDefinitionNotEditable => {
            PyReactionRefusalCategoryV1::LegacyDefinitionNotEditable
        }
        ReactionGestureCategoryV1::MembershipChanged => {
            PyReactionRefusalCategoryV1::MembershipChanged
        }
        ReactionGestureCategoryV1::RendererExclusion => {
            PyReactionRefusalCategoryV1::RendererExclusion
        }
        _ => {
            unreachable!("a new reaction refusal category requires an explicit frozen PyO3 member")
        }
    };
    let recovery = match category {
        PyReactionRefusalCategoryV1::StaleSnapshot
        | PyReactionRefusalCategoryV1::ForeignSession
        | PyReactionRefusalCategoryV1::ReplayedGesture
        | PyReactionRefusalCategoryV1::SessionConflict => {
            PyReactionRefusalRecoveryV1::RefreshAndRestart
        }
        PyReactionRefusalCategoryV1::InvalidRequest
        | PyReactionRefusalCategoryV1::MissingTarget
        | PyReactionRefusalCategoryV1::WrongTargetKind
        | PyReactionRefusalCategoryV1::DuplicateTarget
        | PyReactionRefusalCategoryV1::CrossReactionReuse => {
            PyReactionRefusalRecoveryV1::CorrectSelectors
        }
        PyReactionRefusalCategoryV1::UnrenderableDocument
        | PyReactionRefusalCategoryV1::RenderPreparation => {
            PyReactionRefusalRecoveryV1::ChooseRenderableMembers
        }
        PyReactionRefusalCategoryV1::MissingReaction
        | PyReactionRefusalCategoryV1::MembershipChanged => {
            PyReactionRefusalRecoveryV1::RefreshAndRestart
        }
        PyReactionRefusalCategoryV1::LegacyDefinitionNotEditable => {
            PyReactionRefusalRecoveryV1::RepairLegacyDefinition
        }
        PyReactionRefusalCategoryV1::RendererExclusion => {
            PyReactionRefusalRecoveryV1::ChooseRenderableMembers
        }
    };
    let exception = ReactionGestureError::new_err(message);
    let value = exception.value(py);
    value
        .setattr(
            "category",
            Py::new(py, category).expect("category allocates"),
        )
        .expect("category attaches");
    value
        .setattr(
            "recovery",
            Py::new(py, recovery).expect("recovery allocates"),
        )
        .expect("recovery attaches");
    exception
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
