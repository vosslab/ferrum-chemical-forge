//! Opaque Python DTOs that retain validated interaction state.

use super::types::*;
use crate::{
    RenderInteractionErrorV1, RenderInteractionObservationV1, RenderInteractionSelectionV1,
    RenderInteractionSessionV1, RenderInteractionTranslationGestureV1,
    StructureInteractionObservationV1, StructureInteractionSelectionV1,
};
use ferrum_document::DocumentObjectIdV1;
use pyo3::{PyClass, prelude::*, types::PyTuple};

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "RenderInteractionBoundsV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(super) struct PyBounds {
    #[pyo3(get)]
    pub(super) left: f64,
    #[pyo3(get)]
    pub(super) top: f64,
    #[pyo3(get)]
    pub(super) right: f64,
    #[pyo3(get)]
    pub(super) bottom: f64,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "RenderInteractionRootV1",
    skip_from_py_object
)]
pub(super) struct PyRoot {
    #[pyo3(get)]
    pub(super) document_object_id: String,
    #[pyo3(get)]
    pub(super) paint_order: u32,
    #[pyo3(get)]
    pub(super) kind: PyRootKind,
    #[pyo3(get)]
    pub(super) bounds: PyBounds,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "RenderInteractionExclusionV1",
    skip_from_py_object
)]
pub(super) struct PyExclusion {
    #[pyo3(get)]
    pub(super) document_object_id: String,
    #[pyo3(get)]
    pub(super) reason: PyExclusionReason,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "ReactionAuthoringChoiceV1",
    skip_from_py_object
)]
pub(super) struct PyReactionChoice {
    #[pyo3(get)]
    pub(super) document_object_id: String,
    #[pyo3(get)]
    pub(super) document_paint_order: u32,
    #[pyo3(get)]
    pub(super) kind: PyReactionChoiceKind,
    #[pyo3(get)]
    pub(super) availability: PyReactionChoiceAvailability,
    #[pyo3(get)]
    pub(super) label: String,
    #[pyo3(get)]
    pub(super) bounds: PyBounds,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "ReactionAuthoringExclusionV1",
    skip_from_py_object
)]
pub(super) struct PyReactionExclusion {
    #[pyo3(get)]
    pub(super) diagnostic_key: String,
    #[pyo3(get)]
    pub(super) reason: PyReactionExclusionReason,
    #[pyo3(get)]
    pub(super) recovery: PyReactionExclusionRecovery,
    #[pyo3(get)]
    pub(super) label: String,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "ReactionAuthoringObservationV1",
    skip_from_py_object
)]
pub(super) struct PyReactionAuthoringObservation {
    pub(super) choices: Vec<Py<PyReactionChoice>>,
    pub(super) exclusions: Vec<Py<PyReactionExclusion>>,
}
#[pymethods]
impl PyReactionAuthoringObservation {
    #[getter]
    fn choices(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        tuple(py, &self.choices)
    }
    #[getter]
    fn exclusions(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        tuple(py, &self.exclusions)
    }
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "StructureInteractionTargetV1",
    skip_from_py_object
)]
pub(super) struct PyStructureTarget {
    #[pyo3(get)]
    pub(super) molecule_object_id: String,
    #[pyo3(get)]
    pub(super) object_id: String,
    #[pyo3(get)]
    pub(super) kind: PyStructureTargetKind,
    #[pyo3(get)]
    pub(super) bounds: PyBounds,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "RenderInteractionObservationV1"
)]
pub(super) struct PyObservation {
    pub(super) value: RenderInteractionObservationV1,
    pub(super) roots: Vec<Py<PyRoot>>,
    pub(super) exclusions: Vec<Py<PyExclusion>>,
    pub(super) reaction_authoring: Py<PyReactionAuthoringObservation>,
    #[pyo3(get)]
    pub(super) revision: u64,
    #[pyo3(get)]
    pub(super) digest: String,
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
    #[getter]
    fn reaction_authoring(&self, py: Python<'_>) -> Py<PyReactionAuthoringObservation> {
        self.reaction_authoring.clone_ref(py)
    }
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "RenderInteractionSelectionV1"
)]
pub(crate) struct PySelection {
    pub(super) value: RenderInteractionSelectionV1,
    pub(super) roots: Vec<Py<PyRoot>>,
}

/// A session-validated direct root selected through the opaque interaction API.
/// The borrowed durable identity stays within the private Rust binding boundary.
pub(crate) enum SelectedDirectRootV1<'a> {
    Empty,
    Multiple,
    One(&'a DocumentObjectIdV1),
}

pub(crate) fn selection_value_v1(selection: &PySelection) -> &RenderInteractionSelectionV1 {
    &selection.value
}

#[cfg(test)]
pub(crate) fn test_selection_from_value_v1(value: RenderInteractionSelectionV1) -> PySelection {
    PySelection {
        value,
        roots: Vec::new(),
    }
}

pub(crate) fn selected_direct_root_from_value_v1<'a>(
    session: &RenderInteractionSessionV1,
    selection: &'a RenderInteractionSelectionV1,
) -> Result<SelectedDirectRootV1<'a>, RenderInteractionErrorV1> {
    session.validate_render_interaction_selection_v1(selection)?;
    match selection.roots() {
        [] => Ok(SelectedDirectRootV1::Empty),
        [root] => Ok(SelectedDirectRootV1::One(root.document_object_id())),
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
pub(super) struct PyStructureObservation {
    pub(super) value: StructureInteractionObservationV1,
    pub(super) targets: Vec<Py<PyStructureTarget>>,
    #[pyo3(get)]
    pub(super) revision: u64,
    #[pyo3(get)]
    pub(super) digest: String,
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
pub(super) struct PyStructureSelection {
    pub(super) value: StructureInteractionSelectionV1,
    pub(super) targets: Vec<Py<PyStructureTarget>>,
    #[pyo3(get)]
    pub(super) revision: u64,
    #[pyo3(get)]
    pub(super) digest: String,
}
#[pymethods]
impl PyStructureSelection {
    #[getter]
    fn targets(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        tuple(py, &self.targets)
    }
}
#[pyclass(frozen, module = "ferrum_chem", name = "StructureDeletionCommitV1")]
pub(super) struct PyStructureCommit {
    #[pyo3(get)]
    pub(super) result: super::super::binding::PySessionOperationResultV1,
    #[pyo3(get)]
    pub(super) removed_atom_count: usize,
    #[pyo3(get)]
    pub(super) removed_bond_count: usize,
    #[pyo3(get)]
    pub(super) removed_compact_group_count: usize,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "RenderInteractionTranslationGestureV1"
)]
pub(super) struct PyGesture {
    pub(super) value: Option<RenderInteractionTranslationGestureV1>,
}
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "RenderInteractionTranslationPreviewV1"
)]
pub(super) struct PyPreview {
    #[pyo3(get)]
    pub(super) dx: f64,
    #[pyo3(get)]
    pub(super) dy: f64,
    pub(super) bounds: Vec<Py<PyBounds>>,
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
pub(super) struct PySelectionFacts {
    pub(super) roots: Vec<Py<PyRoot>>,
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
pub(super) struct PyCommit {
    #[pyo3(get)]
    pub(super) changed: bool,
    #[pyo3(get)]
    pub(super) result: super::super::binding::PySessionOperationResultV1,
    pub(super) selection: Py<PySelectionFacts>,
}
#[pymethods]
impl PyCommit {
    #[getter]
    fn selection(&self, py: Python<'_>) -> PyResult<Py<PySelectionFacts>> {
        Ok(self.selection.clone_ref(py))
    }
}

pub(super) fn tuple<T: PyClass>(py: Python<'_>, values: &[Py<T>]) -> PyResult<Py<PyTuple>> {
    PyTuple::new(py, values).map(Into::into)
}
