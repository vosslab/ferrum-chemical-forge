//! Frozen Python facts for durable document-owned reaction authoring.

use ferrum_document::{
    DocumentCreateReactionCommandV1, DocumentDeleteReactionCommandV1, DocumentFenceV1,
    DocumentReactionListDispositionV1, DocumentReactionListObservationV1,
    DocumentReactionMemberSelectionV1, DocumentReactionMemberTargetsV1,
    DocumentReplaceReactionMembersCommandV1, ReactionAuthoringCommandRefusalV1,
    ReactionMemberSelectionRefusalV1,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyModule, PyTuple};

use super::binding::PyDocumentSession;
use super::prepared_transition_binding::PySessionOperationTransitionRequestV1;

#[path = "reaction_binding_methods.rs"]
mod reaction_binding_methods;
#[path = "reaction_binding_support.rs"]
mod reaction_binding_support;

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ReactionCommandRefusalCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyReactionCommandRefusalCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    ReplayedCommand,
    InvalidMembers,
    InvalidSelection,
    RendererAdmission,
    SessionConflict,
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "ReactionCommandRefusalRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyReactionCommandRefusalRecoveryV1 {
    RefreshAndRestart,
    CorrectDocumentObjectIds,
    RepairDocument,
}

create_exception!(
    ferrum_chem,
    ReactionCommandError,
    super::binding::DocumentError
);

#[pyclass(frozen, module = "ferrum_chem", name = "ReactionMemberObservationV1")]
struct PyReactionMemberObservationV1 {
    #[pyo3(get)]
    document_object_id: String,
    #[pyo3(get)]
    document_paint_order: u32,
    #[pyo3(get)]
    role: String,
    #[pyo3(get)]
    role_ordinal: u32,
}

#[pyclass(frozen, module = "ferrum_chem", name = "ReactionObservationV1")]
struct PyReactionObservationV1 {
    #[pyo3(get)]
    document_object_id: String,
    #[pyo3(get)]
    strict: bool,
    diagnostics: Vec<String>,
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
    value: DocumentReactionListObservationV1,
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
    value: DocumentReactionMemberSelectionV1,
    #[pyo3(get)]
    reaction_document_object_id: String,
}

#[pyclass(unsendable, module = "ferrum_chem", name = "CreateReactionCommandV1")]
struct PyCreateReactionCommandV1 {
    value: Option<DocumentCreateReactionCommandV1>,
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "ReplaceReactionMembersCommandV1"
)]
struct PyReplaceReactionMembersCommandV1 {
    value: Option<DocumentReplaceReactionMembersCommandV1>,
}

#[pyclass(unsendable, module = "ferrum_chem", name = "DeleteReactionCommandV1")]
struct PyDeleteReactionCommandV1 {
    value: Option<DocumentDeleteReactionCommandV1>,
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "ReactionCommandError",
        module.py().get_type::<ReactionCommandError>(),
    )?;
    module.add_class::<PyReactionCommandRefusalCategoryV1>()?;
    module.add_class::<PyReactionCommandRefusalRecoveryV1>()?;
    module.add_class::<PyReactionMemberObservationV1>()?;
    module.add_class::<PyReactionObservationV1>()?;
    module.add_class::<PyReactionListObservationV1>()?;
    module.add_class::<PyReactionSelectionV1>()?;
    module.add_class::<PyCreateReactionCommandV1>()?;
    module.add_class::<PyReplaceReactionMembersCommandV1>()?;
    module.add_class::<PyDeleteReactionCommandV1>()
}
