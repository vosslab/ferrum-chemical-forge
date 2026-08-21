//! Opaque PyO3 facade for render-evidence-backed direct-root interaction.

use crate::{
    CommittedRenderInteractionTranslationV1, CommittedStructureDeletionV1, RenderInteractionAxisV1,
    RenderInteractionBoundsV1, RenderInteractionErrorV1, RenderInteractionExclusionReasonV1,
    RenderInteractionExclusionV1, RenderInteractionGridSnapPolicyV1, RenderInteractionModifierV1,
    RenderInteractionObservationV1, RenderInteractionQueryV1, RenderInteractionRootV1,
    RenderInteractionSelectionV1, RenderInteractionSessionV1, RenderInteractionSnapV1,
    RenderInteractionTranslationGestureV1, RenderInteractionTranslationPreviewV1,
    StructureInteractionObservationV1, StructureInteractionQueryV1,
    StructureInteractionSelectionV1, StructureInteractionTargetV1, StructureTargetKindV1,
};
use ferrum_document::DocumentFenceV1;
use pyo3::{PyClass, create_exception, prelude::*, types::PyTuple};

use super::binding::{PyDocumentSession, RevisionConflictError};

create_exception!(
    ferrum_chem,
    RenderInteractionError,
    super::binding::DocumentError
);

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
enum PyCategory {
    StaleRevision,
    StaleDigest,
    ForeignSession,
    SelectionChanged,
    EmptySelection,
    NonFinitePoint,
    InvalidRectangle,
    PreviewMismatch,
    NoTarget,
    UnrenderableDepiction,
    AmbiguousRootIdentifier,
    DisplayOnly,
    Observation,
    SessionConflict,
    CrossMoleculeSelection,
    UnsupportedTarget,
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
enum PyRecovery {
    RefreshAndRestart,
    SelectRenderableRoot,
    CorrectInput,
    ChangePresentation,
    ReportConflict,
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
enum PyExclusionReason {
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
enum PyModifier {
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
enum PyAxis {
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
enum PyGridSnapPolicy {
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
enum PyStructureTargetKind {
    Atom,
    Bond,
    DisplayOnly,
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
fn structure_kind(value: StructureTargetKindV1) -> PyStructureTargetKind {
    match value {
        StructureTargetKindV1::Atom => PyStructureTargetKind::Atom,
        StructureTargetKindV1::Bond => PyStructureTargetKind::Bond,
        StructureTargetKindV1::DisplayOnly => PyStructureTargetKind::DisplayOnly,
    }
}

#[pyclass(frozen, module = "ferrum_chem", name = "RenderInteractionQueryV1")]
pub(crate) struct PyQuery {
    query: RenderInteractionQueryV1,
}
#[pymethods]
impl PyQuery {
    #[staticmethod]
    fn point(x: f64, y: f64, modifier: PyRef<'_, PyModifier>) -> Self {
        Self {
            query: RenderInteractionQueryV1::Point {
                x,
                y,
                modifier: (*modifier).into(),
            },
        }
    }
    #[staticmethod]
    fn marquee(
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
        modifier: PyRef<'_, PyModifier>,
    ) -> Self {
        Self {
            query: RenderInteractionQueryV1::Marquee {
                left,
                top,
                right,
                bottom,
                modifier: (*modifier).into(),
            },
        }
    }
    #[staticmethod]
    #[pyo3(signature = (identifier, modifier = None))]
    fn root(identifier: String, modifier: Option<PyRef<'_, PyModifier>>) -> Self {
        Self {
            query: RenderInteractionQueryV1::Root {
                identifier,
                modifier: modifier.map_or(RenderInteractionModifierV1::Replace, |value| {
                    (*value).into()
                }),
            },
        }
    }
    #[staticmethod]
    fn clear() -> Self {
        Self {
            query: RenderInteractionQueryV1::Clear,
        }
    }
}
#[pyclass(frozen, module = "ferrum_chem", name = "StructureInteractionQueryV1")]
pub(crate) struct PyStructureQuery {
    query: StructureInteractionQueryV1,
}
#[pymethods]
impl PyStructureQuery {
    #[staticmethod]
    fn point(x: f64, y: f64, modifier: PyRef<'_, PyModifier>) -> Self {
        Self {
            query: StructureInteractionQueryV1::Point {
                x,
                y,
                modifier: (*modifier).into(),
            },
        }
    }
    #[staticmethod]
    fn marquee(
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
        modifier: PyRef<'_, PyModifier>,
    ) -> Self {
        Self {
            query: StructureInteractionQueryV1::Marquee {
                left,
                top,
                right,
                bottom,
                modifier: (*modifier).into(),
            },
        }
    }
    #[staticmethod]
    fn clear() -> Self {
        Self {
            query: StructureInteractionQueryV1::Clear,
        }
    }
}
#[pyclass(frozen, module = "ferrum_chem", name = "RenderInteractionSnapV1")]
pub(crate) struct PySnap {
    snap: RenderInteractionSnapV1,
}
#[pymethods]
impl PySnap {
    #[new]
    fn new(axis: PyRef<'_, PyAxis>) -> Self {
        Self {
            snap: RenderInteractionSnapV1::new((*axis).into()),
        }
    }
    #[staticmethod]
    fn free() -> Self {
        Self {
            snap: RenderInteractionSnapV1::free(),
        }
    }
    #[staticmethod]
    fn with_grid_policy(axis: PyRef<'_, PyAxis>, policy: PyRef<'_, PyGridSnapPolicy>) -> Self {
        Self {
            snap: RenderInteractionSnapV1::with_grid_policy((*axis).into(), (*policy).into()),
        }
    }
}

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "RenderInteractionBoundsV1",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyBounds {
    #[pyo3(get)]
    left: f64,
    #[pyo3(get)]
    top: f64,
    #[pyo3(get)]
    right: f64,
    #[pyo3(get)]
    bottom: f64,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "RenderInteractionRootV1",
    skip_from_py_object
)]
struct PyRoot {
    #[pyo3(get)]
    identifier: String,
    #[pyo3(get)]
    source_order: u32,
    #[pyo3(get)]
    bounds: PyBounds,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "RenderInteractionExclusionV1",
    skip_from_py_object
)]
struct PyExclusion {
    #[pyo3(get)]
    identifier: String,
    #[pyo3(get)]
    reason: PyExclusionReason,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "StructureInteractionTargetV1",
    skip_from_py_object
)]
struct PyStructureTarget {
    #[pyo3(get)]
    molecule_id: String,
    #[pyo3(get)]
    identifier: String,
    #[pyo3(get)]
    source_order: u32,
    #[pyo3(get)]
    kind: PyStructureTargetKind,
    #[pyo3(get)]
    bounds: PyBounds,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "RenderInteractionObservationV1"
)]
pub(crate) struct PyObservation {
    value: RenderInteractionObservationV1,
    roots: Vec<Py<PyRoot>>,
    exclusions: Vec<Py<PyExclusion>>,
    #[pyo3(get)]
    revision: u64,
    #[pyo3(get)]
    digest: String,
}
#[pymethods]
impl PyObservation {
    #[getter]
    fn roots(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        tuple(py, &self.roots)
    }
    #[getter]
    fn exclusions(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        tuple(py, &self.exclusions)
    }
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "RenderInteractionSelectionV1"
)]
pub(crate) struct PySelection {
    value: RenderInteractionSelectionV1,
    roots: Vec<Py<PyRoot>>,
}

/// A session-validated direct root selected through the opaque interaction API.
/// The borrowed identifier stays within the private Rust binding boundary.
pub(crate) enum SelectedDirectRootV1<'a> {
    Empty,
    Multiple,
    One(&'a str),
}

pub(crate) fn selected_direct_root_v1<'a>(
    session: &RenderInteractionSessionV1,
    selection: &'a PySelection,
) -> Result<SelectedDirectRootV1<'a>, RenderInteractionErrorV1> {
    selected_direct_root_from_value_v1(session, &selection.value)
}

pub(crate) fn selection_value_v1(selection: &PySelection) -> &RenderInteractionSelectionV1 {
    &selection.value
}

pub(crate) fn selected_direct_root_from_value_v1<'a>(
    session: &RenderInteractionSessionV1,
    selection: &'a RenderInteractionSelectionV1,
) -> Result<SelectedDirectRootV1<'a>, RenderInteractionErrorV1> {
    session.validate_render_interaction_selection_v1(selection)?;
    match selection.roots() {
        [] => Ok(SelectedDirectRootV1::Empty),
        [root] => Ok(SelectedDirectRootV1::One(root.identifier())),
        _ => Ok(SelectedDirectRootV1::Multiple),
    }
}
#[pymethods]
impl PySelection {
    #[getter]
    fn roots(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        tuple(py, &self.roots)
    }
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "StructureInteractionObservationV1"
)]
pub(crate) struct PyStructureObservation {
    value: StructureInteractionObservationV1,
    targets: Vec<Py<PyStructureTarget>>,
    #[pyo3(get)]
    revision: u64,
    #[pyo3(get)]
    digest: String,
}
#[pymethods]
impl PyStructureObservation {
    #[getter]
    fn targets(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        tuple(py, &self.targets)
    }
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "StructureInteractionSelectionV1"
)]
pub(crate) struct PyStructureSelection {
    value: StructureInteractionSelectionV1,
    targets: Vec<Py<PyStructureTarget>>,
}
#[pymethods]
impl PyStructureSelection {
    #[getter]
    fn targets(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        tuple(py, &self.targets)
    }
}
#[pyclass(frozen, module = "ferrum_chem", name = "StructureDeletionCommitV1")]
pub(crate) struct PyStructureCommit {
    #[pyo3(get)]
    result: super::binding::PySessionOperationResultV1,
    #[pyo3(get)]
    removed_atoms: Vec<String>,
    #[pyo3(get)]
    removed_bonds: Vec<String>,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "RenderInteractionTranslationGestureV1"
)]
pub(crate) struct PyGesture {
    value: RenderInteractionTranslationGestureV1,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "RenderInteractionTranslationPreviewV1"
)]
pub(crate) struct PyPreview {
    value: RenderInteractionTranslationPreviewV1,
    #[pyo3(get)]
    dx: f64,
    #[pyo3(get)]
    dy: f64,
    bounds: Vec<Py<PyBounds>>,
}
#[pymethods]
impl PyPreview {
    #[getter]
    fn bounds(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        tuple(py, &self.bounds)
    }
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "RenderInteractionSelectionFactsV1",
    skip_from_py_object
)]
struct PySelectionFacts {
    roots: Vec<Py<PyRoot>>,
}
#[pymethods]
impl PySelectionFacts {
    #[getter]
    fn roots(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        tuple(py, &self.roots)
    }
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "RenderInteractionTranslationCommitV1"
)]
pub(crate) struct PyCommit {
    #[pyo3(get)]
    changed: bool,
    #[pyo3(get)]
    result: super::binding::PySessionOperationResultV1,
    selection: Py<PySelectionFacts>,
}
#[pymethods]
impl PyCommit {
    #[getter]
    fn selection(&self, py: Python<'_>) -> PyResult<Py<PySelectionFacts>> {
        Ok(self.selection.clone_ref(py))
    }
}

#[pymethods]
impl PyDocumentSession {
    fn observe_render_interaction_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
    ) -> PyResult<PyObservation> {
        self.session
            .observe_render_interaction_v1(fence(&expected_digest_hex, expected_revision)?)
            .map_err(|error| interaction_error(py, error))
            .and_then(|value| observation(py, value))
    }
    fn select_render_interaction_roots_v1(
        &self,
        py: Python<'_>,
        observation: PyRef<'_, PyObservation>,
        previous: Option<PyRef<'_, PySelection>>,
        query: PyRef<'_, PyQuery>,
    ) -> PyResult<PySelection> {
        self.session
            .select_render_interaction_roots_v1(
                &observation.value,
                previous.as_ref().map(|value| &value.value),
                query.query.clone(),
            )
            .map_err(|error| interaction_error(py, error))
            .and_then(|value| selection(py, value))
    }
    fn begin_render_interaction_translation_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, PySelection>,
        press_x: f64,
        press_y: f64,
        snap: PyRef<'_, PySnap>,
    ) -> PyResult<PyGesture> {
        self.session
            .begin_render_interaction_translation_v1(&selection.value, press_x, press_y, snap.snap)
            .map(|value| PyGesture { value })
            .map_err(|error| interaction_error(py, error))
    }
    fn preview_render_interaction_translation_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyGesture>,
        pointer_x: f64,
        pointer_y: f64,
    ) -> PyResult<PyPreview> {
        self.session
            .preview_render_interaction_translation_v1(&gesture.value, pointer_x, pointer_y)
            .map_err(|error| interaction_error(py, error))
            .and_then(|value| preview(py, value))
    }
    fn commit_render_interaction_translation_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyGesture>,
        preview: PyRef<'_, PyPreview>,
    ) -> PyResult<PyCommit> {
        self.session
            .commit_render_interaction_translation_v1(&gesture.value, &preview.value)
            .map_err(|error| interaction_error(py, error))
            .and_then(|value| commit(py, value))
    }
    fn observe_structure_interaction_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
    ) -> PyResult<PyStructureObservation> {
        self.session
            .observe_structure_interaction_v1(fence(&expected_digest_hex, expected_revision)?)
            .map_err(|error| interaction_error(py, error))
            .and_then(|value| structure_observation(py, value))
    }
    fn select_structure_interaction_v1(
        &self,
        py: Python<'_>,
        observation: PyRef<'_, PyStructureObservation>,
        previous: Option<PyRef<'_, PyStructureSelection>>,
        query: PyRef<'_, PyStructureQuery>,
    ) -> PyResult<PyStructureSelection> {
        self.session
            .select_structure_interaction_v1(
                &observation.value,
                previous.as_ref().map(|value| &value.value),
                query.query.clone(),
            )
            .map_err(|error| interaction_error(py, error))
            .and_then(|value| structure_selection(py, value))
    }
    fn commit_structure_deletion_v1(
        &mut self,
        py: Python<'_>,
        selection: PyRef<'_, PyStructureSelection>,
    ) -> PyResult<PyStructureCommit> {
        self.session
            .commit_structure_deletion_v1(&selection.value)
            .map_err(|error| interaction_error(py, error))
            .map(structure_commit)
    }
}

fn root(py: Python<'_>, value: &RenderInteractionRootV1) -> PyResult<Py<PyRoot>> {
    Py::new(
        py,
        PyRoot {
            identifier: value.identifier().to_owned(),
            source_order: value.source_order(),
            bounds: bounds(value.bounds()),
        },
    )
}
fn roots(py: Python<'_>, values: &[RenderInteractionRootV1]) -> PyResult<Vec<Py<PyRoot>>> {
    values.iter().map(|value| root(py, value)).collect()
}
fn bounds(value: RenderInteractionBoundsV1) -> PyBounds {
    PyBounds {
        left: value.left(),
        top: value.top(),
        right: value.right(),
        bottom: value.bottom(),
    }
}
fn exclusion_reason(value: RenderInteractionExclusionReasonV1) -> PyExclusionReason {
    match value {
        RenderInteractionExclusionReasonV1::UnrenderableDepiction => {
            PyExclusionReason::UnrenderableDepiction
        }
        RenderInteractionExclusionReasonV1::AmbiguousRootIdentifier => {
            PyExclusionReason::AmbiguousRootIdentifier
        }
        RenderInteractionExclusionReasonV1::DisplayOnly => PyExclusionReason::DisplayOnly,
    }
}
fn exclusions(
    py: Python<'_>,
    values: &[RenderInteractionExclusionV1],
) -> PyResult<Vec<Py<PyExclusion>>> {
    values
        .iter()
        .map(|value| {
            Py::new(
                py,
                PyExclusion {
                    identifier: value.identifier().to_owned(),
                    reason: exclusion_reason(value.reason()),
                },
            )
        })
        .collect()
}
fn observation(py: Python<'_>, value: RenderInteractionObservationV1) -> PyResult<PyObservation> {
    let fence = value.fence();
    let roots = roots(py, value.roots())?;
    let exclusions = exclusions(py, value.exclusions())?;
    Ok(PyObservation {
        value,
        roots,
        exclusions,
        revision: fence.revision(),
        digest: hex_digest(&fence.digest()),
    })
}
fn selection(py: Python<'_>, value: RenderInteractionSelectionV1) -> PyResult<PySelection> {
    let roots = roots(py, value.roots())?;
    Ok(PySelection { value, roots })
}
fn preview(py: Python<'_>, value: RenderInteractionTranslationPreviewV1) -> PyResult<PyPreview> {
    let bounds = value
        .bounds()
        .iter()
        .copied()
        .map(|value| Py::new(py, bounds(value)))
        .collect::<PyResult<_>>()?;
    Ok(PyPreview {
        dx: value.dx(),
        dy: value.dy(),
        value,
        bounds,
    })
}
fn commit(py: Python<'_>, value: CommittedRenderInteractionTranslationV1) -> PyResult<PyCommit> {
    let selection = Py::new(
        py,
        PySelectionFacts {
            roots: roots(py, value.selection().roots())?,
        },
    )?;
    Ok(PyCommit {
        changed: value.changed(),
        result: value.result().clone().into(),
        selection,
    })
}
fn structure_target(
    py: Python<'_>,
    value: &StructureInteractionTargetV1,
) -> PyResult<Py<PyStructureTarget>> {
    Py::new(
        py,
        PyStructureTarget {
            molecule_id: value.molecule_id().to_owned(),
            identifier: value.identifier().to_owned(),
            source_order: value.source_order(),
            kind: structure_kind(value.kind()),
            bounds: bounds(value.bounds()),
        },
    )
}
fn structure_targets(
    py: Python<'_>,
    values: &[StructureInteractionTargetV1],
) -> PyResult<Vec<Py<PyStructureTarget>>> {
    values
        .iter()
        .map(|value| structure_target(py, value))
        .collect()
}
fn structure_observation(
    py: Python<'_>,
    value: StructureInteractionObservationV1,
) -> PyResult<PyStructureObservation> {
    let fence = value.fence();
    let targets = structure_targets(py, value.targets())?;
    Ok(PyStructureObservation {
        value,
        targets,
        revision: fence.revision(),
        digest: hex_digest(&fence.digest()),
    })
}
fn structure_selection(
    py: Python<'_>,
    value: StructureInteractionSelectionV1,
) -> PyResult<PyStructureSelection> {
    let targets = structure_targets(py, value.targets())?;
    Ok(PyStructureSelection { value, targets })
}
fn structure_commit(value: CommittedStructureDeletionV1) -> PyStructureCommit {
    PyStructureCommit {
        result: value.result().clone().into(),
        removed_atoms: value.removed_atoms().to_vec(),
        removed_bonds: value.removed_bonds().to_vec(),
    }
}
fn tuple<T: PyClass>(py: Python<'_>, values: &[Py<T>]) -> PyResult<Py<PyTuple>> {
    PyTuple::new(py, values).map(Into::into)
}
fn fence(digest: &str, revision: u64) -> PyResult<DocumentFenceV1> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(RenderInteractionError::new_err(
            "expected digest must be exactly 64 lowercase hexadecimal characters",
        ));
    }
    let mut bytes = [0; 32];
    for (index, pair) in digest.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (hex(pair[0]) << 4) | hex(pair[1]);
    }
    Ok(DocumentFenceV1::new(revision, bytes))
}
const fn hex(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}
fn hex_digest(value: &[u8; 32]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn category(error: &RenderInteractionErrorV1) -> PyCategory {
    match error {
        RenderInteractionErrorV1::StaleRevision => PyCategory::StaleRevision,
        RenderInteractionErrorV1::StaleDigest => PyCategory::StaleDigest,
        RenderInteractionErrorV1::ForeignSession => PyCategory::ForeignSession,
        RenderInteractionErrorV1::SelectionChanged => PyCategory::SelectionChanged,
        RenderInteractionErrorV1::EmptySelection => PyCategory::EmptySelection,
        RenderInteractionErrorV1::NonFinitePoint => PyCategory::NonFinitePoint,
        RenderInteractionErrorV1::InvalidRectangle => PyCategory::InvalidRectangle,
        RenderInteractionErrorV1::PreviewMismatch => PyCategory::PreviewMismatch,
        RenderInteractionErrorV1::NoTarget => PyCategory::NoTarget,
        RenderInteractionErrorV1::UnrenderableDepiction => PyCategory::UnrenderableDepiction,
        RenderInteractionErrorV1::AmbiguousRootIdentifier => PyCategory::AmbiguousRootIdentifier,
        RenderInteractionErrorV1::DisplayOnly => PyCategory::DisplayOnly,
        RenderInteractionErrorV1::Observation => PyCategory::Observation,
        RenderInteractionErrorV1::SessionConflict => PyCategory::SessionConflict,
        RenderInteractionErrorV1::CrossMoleculeSelection => PyCategory::CrossMoleculeSelection,
        RenderInteractionErrorV1::UnsupportedTarget => PyCategory::UnsupportedTarget,
        RenderInteractionErrorV1::UnsupportedDocument => PyCategory::Observation,
    }
}
fn recovery(error: &RenderInteractionErrorV1) -> PyRecovery {
    match error {
        RenderInteractionErrorV1::StaleRevision
        | RenderInteractionErrorV1::StaleDigest
        | RenderInteractionErrorV1::ForeignSession
        | RenderInteractionErrorV1::SelectionChanged => PyRecovery::RefreshAndRestart,
        RenderInteractionErrorV1::EmptySelection
        | RenderInteractionErrorV1::NoTarget
        | RenderInteractionErrorV1::CrossMoleculeSelection => PyRecovery::SelectRenderableRoot,
        RenderInteractionErrorV1::NonFinitePoint | RenderInteractionErrorV1::InvalidRectangle => {
            PyRecovery::CorrectInput
        }
        RenderInteractionErrorV1::UnrenderableDepiction
        | RenderInteractionErrorV1::AmbiguousRootIdentifier
        | RenderInteractionErrorV1::DisplayOnly
        | RenderInteractionErrorV1::UnsupportedTarget => PyRecovery::ChangePresentation,
        RenderInteractionErrorV1::PreviewMismatch
        | RenderInteractionErrorV1::Observation
        | RenderInteractionErrorV1::SessionConflict => PyRecovery::ReportConflict,
        RenderInteractionErrorV1::UnsupportedDocument => PyRecovery::ChangePresentation,
    }
}
fn interaction_error(py: Python<'_>, error: RenderInteractionErrorV1) -> PyErr {
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
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "RenderInteractionError",
        module.py().get_type::<RenderInteractionError>(),
    )?;
    module.add_class::<PyCategory>()?;
    module.add_class::<PyRecovery>()?;
    module.add_class::<PyExclusionReason>()?;
    module.add_class::<PyModifier>()?;
    module.add_class::<PyAxis>()?;
    module.add_class::<PyGridSnapPolicy>()?;
    module.add_class::<PyStructureTargetKind>()?;
    module.add_class::<PyQuery>()?;
    module.add_class::<PyStructureQuery>()?;
    module.add_class::<PySnap>()?;
    module.add_class::<PyBounds>()?;
    module.add_class::<PyRoot>()?;
    module.add_class::<PyExclusion>()?;
    module.add_class::<PyStructureTarget>()?;
    module.add_class::<PyObservation>()?;
    module.add_class::<PySelection>()?;
    module.add_class::<PyStructureObservation>()?;
    module.add_class::<PyStructureSelection>()?;
    module.add_class::<PyStructureCommit>()?;
    module.add_class::<PyGesture>()?;
    module.add_class::<PyPreview>()?;
    module.add_class::<PySelectionFacts>()?;
    module.add_class::<PyCommit>()?;
    Ok(())
}
