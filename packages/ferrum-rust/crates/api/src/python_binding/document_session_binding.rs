use crate::RenderInteractionSessionV1;
use ferrum_document::{
    DocumentBondOrderV1, DocumentBondPresentationV1, DocumentRenderObservationV1, DocumentSession,
    DocumentSnapshot, PendingAdmittedMoleculeInsertionV1, PendingCreateAtom, PendingCreateBond,
    PendingCreateBondedAtom, PendingCreateWavy, Point3V1, Publication, SaveOutcome,
    SessionOperation, SessionOperationResultV1, SessionOperationV1,
};
use pyo3::prelude::*;

use super::bracket_binding::{
    PyDocumentBracketBoundsV1, PyDocumentBracketStyleV1, PyPreparedBracketInsertion,
};
use super::document_error_binding::{document_object_id, document_result, projection_error};
use super::document_operation_binding::PyDocumentOperationV1;
use super::interchange_insertion_binding::{
    PyAdmittedInterchangeRecordInsertionV1, PyInterchangeRecordBatchInsertionV1,
};
use super::molecule_coordinate_binding::{
    PyPreparedCleanGeometryV1, PyPreparedMoleculeCoordinatesV1,
};
use super::projection_binding::PySessionDocumentObservationV1;
use super::render_binding::{self, PyRenderObservationV1};
use super::smiles_insertion_binding::PyMoleculeInsertionV1;

/// Immutable Python-owned copy of one authoritative document revision.
///
/// All values are copied from Rust. A snapshot has no mutable alias to its
/// originating [`PyDocumentSession`], so callers may retain it after later session
/// calls, but it never observes those later revisions.
#[pyclass(frozen, name = "DocumentSnapshot", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyDocumentSnapshot {
    #[pyo3(get)]
    cdml: String,
    #[pyo3(get)]
    revision: u64,
    #[pyo3(get)]
    digest: String,
    #[pyo3(get)]
    is_dirty: bool,
}

impl From<DocumentSnapshot> for PyDocumentSnapshot {
    fn from(snapshot: DocumentSnapshot) -> Self {
        Self {
            cdml: snapshot.cdml().to_owned(),
            revision: snapshot.revision(),
            digest: hex_digest(snapshot.digest()),
            is_dirty: snapshot.is_dirty(),
        }
    }
}

/// Immutable result of one accepted document mutation or history transition.
///
/// `observation` owns the one authoritative post-operation snapshot and projection.
#[pyclass(frozen, name = "SessionOperationResultV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PySessionOperationResultV1 {
    #[pyo3(get)]
    observation: PySessionDocumentObservationV1,
}

impl From<SessionOperationResultV1> for PySessionOperationResultV1 {
    fn from(result: SessionOperationResultV1) -> Self {
        Self {
            observation: result.observation().clone().into(),
        }
    }
}

/// Closed outcome of an ordinary save publication.
///
/// Instances are created only by [`PyPublication`]. Use the boolean facts instead
/// of comparing a mutable spelling.
#[pyclass(frozen, name = "SaveOutcome", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PySaveOutcome {
    outcome: SaveOutcome,
}

#[pymethods]
impl PySaveOutcome {
    /// Return whether the saved baseline was advanced.
    #[getter]
    fn is_confirmed(&self) -> bool {
        self.outcome == SaveOutcome::Confirmed
    }

    /// Return whether the destination needs explicit verification or recovery.
    #[getter]
    fn requires_destination_verification(&self) -> bool {
        self.outcome == SaveOutcome::DirectoryEntryUnconfirmed
    }
}

/// Immutable result of one document publication attempt.
///
/// `snapshot` is the session state after the operation. `published_snapshot` is
/// the exact value given to the publisher. A confirmed ordinary save returns a
/// clean `snapshot`; recovery exports and unconfirmed replacements do not alter
/// the session baseline.
#[pyclass(frozen, name = "Publication")]
pub(crate) struct PyPublication {
    #[pyo3(get)]
    snapshot: PyDocumentSnapshot,
    #[pyo3(get)]
    published_snapshot: PyDocumentSnapshot,
    #[pyo3(get)]
    outcome: PySaveOutcome,
}

impl From<Publication> for PyPublication {
    fn from(publication: Publication) -> Self {
        Self {
            snapshot: publication.snapshot().clone().into(),
            published_snapshot: publication.published_snapshot().clone().into(),
            outcome: PySaveOutcome {
                outcome: publication.outcome(),
            },
        }
    }
}

/// Opaque one-use prepared atom insertion.
///
/// The Rust value binds its candidate to the revision at which it was prepared.
/// It is deliberately thread-affine and exposes only the durable identifier that
/// would be created; the internal provisional token is never serialized to Python.
#[pyclass(unsendable, module = "ferrum_chem", name = "PreparedAtomInsertion")]
pub(crate) struct PyPreparedAtomInsertion {
    pending: PendingCreateAtom,
    #[pyo3(get)]
    identifier: String,
}

/// Opaque one-use prepared Wavy insertion.
#[pyclass(unsendable, module = "ferrum_chem", name = "PreparedWavyInsertion")]
pub(crate) struct PyPreparedWavyInsertion {
    pending: PendingCreateWavy,
    #[pyo3(get)]
    identifier: String,
}

/// Closed bond-order vocabulary accepted by the document session.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DocumentBondOrderV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyDocumentBondOrderV1 {
    Single,
    Double,
    Triple,
}

impl From<PyDocumentBondOrderV1> for DocumentBondOrderV1 {
    fn from(value: PyDocumentBondOrderV1) -> Self {
        match value {
            PyDocumentBondOrderV1::Single => Self::Single,
            PyDocumentBondOrderV1::Double => Self::Double,
            PyDocumentBondOrderV1::Triple => Self::Triple,
        }
    }
}

/// Closed native bond presentations accepted by the private creation seam.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DocumentBondPresentationV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyDocumentBondPresentationV1 {
    /// A normal single bond.
    NormalSingle,
    /// A normal double bond.
    NormalDouble,
    /// A normal triple bond.
    NormalTriple,
    /// A directed solid wedge from start atom to end atom.
    SolidWedge,
    /// A directed hashed wedge from start atom to end atom.
    HashedWedge,
}

impl From<PyDocumentBondPresentationV1> for DocumentBondPresentationV1 {
    fn from(value: PyDocumentBondPresentationV1) -> Self {
        match value {
            PyDocumentBondPresentationV1::NormalSingle => Self::Normal(DocumentBondOrderV1::Single),
            PyDocumentBondPresentationV1::NormalDouble => Self::Normal(DocumentBondOrderV1::Double),
            PyDocumentBondPresentationV1::NormalTriple => Self::Normal(DocumentBondOrderV1::Triple),
            PyDocumentBondPresentationV1::SolidWedge => Self::SolidWedge,
            PyDocumentBondPresentationV1::HashedWedge => Self::HashedWedge,
        }
    }
}

/// Opaque one-use prepared molecule-local bond insertion.
#[pyclass(unsendable, module = "ferrum_chem", name = "PreparedBondInsertion")]
pub(crate) struct PyPreparedBondInsertion {
    pending: PendingCreateBond,
    #[pyo3(get)]
    identifier: String,
}

/// Opaque one-use prepared atom-plus-bond insertion.
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PreparedBondedAtomInsertion"
)]
pub(crate) struct PyPreparedBondedAtomInsertion {
    pending: PendingCreateBondedAtom,
    #[pyo3(get)]
    atom_identifier: String,
    #[pyo3(get)]
    bond_identifier: String,
}

/// Opaque one-use renderer-admitted complete molecule insertion.
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "AdmittedMoleculeInsertionV1"
)]
pub(crate) struct PyAdmittedMoleculeInsertionV1 {
    pending: PendingAdmittedMoleculeInsertionV1,
    #[pyo3(get)]
    molecule_identifier: String,
}

/// Thread-affine owner of one mutable Rust CDML document session.
///
/// A session is deliberately unsendable: callers create, mutate, and destroy it
/// on its Python-owning thread. Every method is synchronous; it retains no Python
/// input or callback after return. Snapshots, observations, and publications are
/// frozen owned copies and may outlive their originating session.
#[pyclass(unsendable, name = "DocumentSession")]
pub(crate) struct PyDocumentSession {
    pub(crate) session: RenderInteractionSessionV1,
    live_smarts: super::live_document_smarts_query_v1::LiveDocumentSmartsBridgeV1,
    pub(crate) published_presentation_plan: Option<ferrum_render::PresentationRenderPlanV1>,
}

impl PyDocumentSession {
    pub(crate) fn from_session(session: DocumentSession) -> Self {
        Self {
            session: RenderInteractionSessionV1::new(session),
            live_smarts: super::live_document_smarts_query_v1::LiveDocumentSmartsBridgeV1::new(),
            published_presentation_plan: None,
        }
    }

    /// Publish one renderer observation, presentation plan, and live SMARTS
    /// bridge from the same accepted document observation.
    pub(crate) fn publish_live_render_plan_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
    ) -> PyResult<DocumentRenderObservationV1> {
        let accepted = document_result(py, self.session.observe(expected_revision))?;
        let observation = self
            .live_smarts
            .publish_from_observation(&self.session, accepted)?;
        let plan =
            match super::presentation_render_plan_binding::plan_from_observation(&observation) {
                Ok(plan) => plan,
                Err(error) => {
                    self.live_smarts.retire();
                    self.published_presentation_plan = None;
                    return Err(error);
                }
            };
        self.published_presentation_plan = Some(plan);
        Ok(observation)
    }
}

#[pymethods]
impl PyDocumentSession {
    /// Create a canonical, clean, revision-zero empty CDML document.
    ///
    /// The Rust backend owns the root namespace and version. This creates no
    /// selectable roots and does not claim the nonempty authored-26.07 profile.
    #[staticmethod]
    fn create_empty_document_v1(py: Python<'_>) -> PyResult<Self> {
        let session = document_result(py, DocumentSession::create_empty_document_v1())?;
        Ok(Self {
            session: RenderInteractionSessionV1::new(session),
            live_smarts: super::live_document_smarts_query_v1::LiveDocumentSmartsBridgeV1::new(),
            published_presentation_plan: None,
        })
    }

    /// Create a session from already-allocated CDML copied during this call.
    ///
    /// This unbounded compatibility route applies no resource policy. External input must use
    /// an explicit-budget byte or file admission method instead.
    #[staticmethod]
    fn load(py: Python<'_>, cdml: &str) -> PyResult<Self> {
        let session = document_result(py, DocumentSession::load(cdml))?;
        Ok(Self {
            session: RenderInteractionSessionV1::new(session),
            live_smarts: super::live_document_smarts_query_v1::LiveDocumentSmartsBridgeV1::new(),
            published_presentation_plan: None,
        })
    }

    /// Return one immutable owned snapshot without changing session state.
    fn snapshot(&self, py: Python<'_>) -> PyResult<PyDocumentSnapshot> {
        document_result(py, self.session.snapshot()).map(PyDocumentSnapshot::from)
    }

    /// Return whether the authoritative session has an earlier history state.
    #[getter]
    fn can_undo(&self) -> bool {
        self.session.can_undo()
    }

    /// Return whether the authoritative session has a later history state.
    #[getter]
    fn can_redo(&self) -> bool {
        self.session.can_redo()
    }

    /// Private live-tab SMARTS query. Results contain copied summaries and an
    /// opaque session-local receipt only; no graph or document identity escapes.
    fn _run_live_document_smarts_query_v1(
        &mut self,
        py: Python<'_>,
        query: &Bound<'_, pyo3::types::PyAny>,
        max_matches_per_molecule: u32,
        max_total_matches: u32,
    ) -> PyResult<Py<PyAny>> {
        if let Ok(value) = query.extract::<String>() {
            self.live_smarts.validate_raw_request(
                &self.session,
                &value,
                max_matches_per_molecule,
                max_total_matches,
            )?;
            let engine = super::super::staged_extension_native_engine_v1().map_err(|_| {
                super::live_document_smarts_query_v1::LiveFailureV1::Unavailable(
                    super::live_document_smarts_query_v1::PyLiveDocumentSmartsReasonV1::NativeRuntimeUnavailable,
                )
                .into_pyerr()
            })?;
            return self
                .live_smarts
                .run(
                    py,
                    &self.session,
                    &engine,
                    value,
                    max_matches_per_molecule,
                    max_total_matches,
                )
                .map(|value| value.into_any());
        }
        let selection = query
            .extract::<PyRef<'_, super::live_document_smarts_query_v1::PyLiveDocumentSmartsSelectedQueryV1>>()
            .map_err(|_| {
                super::live_document_smarts_query_v1::LiveFailureV1::Refused(
                    super::live_document_smarts_query_v1::PyLiveDocumentSmartsReasonV1::SelectedRootEmpty,
                )
                .into_pyerr()
            })?;
        self.live_smarts.prepare_selected_request(
            &self.session,
            &selection,
            max_matches_per_molecule,
            max_total_matches,
        )?;
        let engine = super::super::staged_extension_native_engine_v1().map_err(|_| {
            super::live_document_smarts_query_v1::LiveFailureV1::Unavailable(
                super::live_document_smarts_query_v1::PyLiveDocumentSmartsReasonV1::NativeRuntimeUnavailable,
            )
            .into_pyerr()
        })?;
        self.live_smarts
            .run_selected(
                py,
                &self.session,
                &engine,
                &selection,
                max_matches_per_molecule,
                max_total_matches,
            )
            .map(|value| value.into_any())
    }

    /// Mint the only selected-molecule input accepted by the live SMARTS
    /// operation. The generic selection is consumed inside this private Rust
    /// boundary and cannot be passed through the query call itself.
    fn _capture_live_document_smarts_selected_query_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<'_, super::direct_root_interaction_binding::PySelection>,
    ) -> PyResult<Py<super::live_document_smarts_query_v1::PyLiveDocumentSmartsSelectedQueryV1>>
    {
        self.live_smarts
            .capture_selected_query(py, &self.session, &selection)
    }

    /// Check one opaque selected-query token without consuming it or exposing
    /// any selection, document, or renderer facts.
    fn _live_document_smarts_selected_readiness_v1(
        &self,
        py: Python<'_>,
        selection: PyRef<
            '_,
            super::live_document_smarts_query_v1::PyLiveDocumentSmartsSelectedQueryV1,
        >,
    ) -> PyResult<Py<super::live_document_smarts_query_v1::PyLiveDocumentSmartsSelectedReadinessV1>>
    {
        Py::new(
            py,
            self.live_smarts
                .selected_readiness(&self.session, &selection),
        )
    }

    /// Redeem exactly one local receipt row into identity-free paint bounds.
    fn _show_live_document_smarts_match_v1(
        &mut self,
        py: Python<'_>,
        receipt: PyRef<'_, super::live_document_smarts_query_v1::PyLiveDocumentSmartsReceiptV1>,
        row_index: usize,
    ) -> PyResult<Py<super::live_document_smarts_query_v1::PyLiveDocumentSmartsPaintV1>> {
        self.live_smarts.show(py, &self.session, receipt, row_index)
    }

    /// Invalidate all live SMARTS receipts before a view lifecycle transition.
    fn _retire_live_document_smarts_query_v1(&mut self) {
        self.live_smarts.retire();
        self.published_presentation_plan = None;
    }

    /// Invalidate only derived live SMARTS receipts after query-level UI
    /// cleanup. The private render plan remains owned by the authoritative
    /// projection transaction and is not republished here.
    fn _retire_live_document_smarts_receipts_v1(&mut self) {
        self.live_smarts.retire_receipts();
    }

    /// Private Qt publication seam: the returned render observation and the
    /// receipt plan are derived from the exact same accepted observation.
    fn _publish_live_render_plan_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
    ) -> PyResult<PyRenderObservationV1> {
        let observation = self.publish_live_render_plan_v1(py, expected_revision)?;
        render_binding::observation(py, observation)
    }

    /// Observe the current session state after checking its expected revision.
    fn observe(
        &self,
        py: Python<'_>,
        expected_revision: u64,
    ) -> PyResult<PySessionDocumentObservationV1> {
        document_result(py, self.session.observe(expected_revision)).map(Into::into)
    }

    /// Return one complete frozen API-owned depiction observation for this revision.
    fn observe_render(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
    ) -> PyResult<PyRenderObservationV1> {
        let observation = self.publish_live_render_plan_v1(py, expected_revision)?;
        render_binding::observation(py, observation)
    }

    /// Submit one closed V1 operation against an exact expected revision.
    fn submit(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        operation: PyRef<'_, PyDocumentOperationV1>,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(
            py,
            self.session
                .submit(expected_revision, operation.operation.clone()),
        )
        .map(Into::into)
    }

    /// Replace or clear one authenticated direct-root molecule name.
    fn set_document_molecule_name_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest: &Bound<'_, pyo3::types::PyString>,
        molecule_id: &Bound<'_, pyo3::types::PyString>,
        name: &Bound<'_, pyo3::types::PyString>,
    ) -> PyResult<PySessionOperationResultV1> {
        super::document_molecule_name_binding::set_document_molecule_name_v1(
            py,
            &mut self.session,
            expected_revision,
            expected_digest,
            molecule_id,
            name,
        )
        .map(Into::into)
    }

    /// Convert one authenticated ordered direct-atom path to its linear form.
    fn convert_linear_form_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest: &Bound<'_, pyo3::types::PyString>,
        molecule_id: &Bound<'_, pyo3::types::PyString>,
        selected_atom_ids: &Bound<'_, pyo3::types::PyAny>,
    ) -> PyResult<PySessionOperationResultV1> {
        super::document_linear_form_binding::convert_linear_form_v1(
            py,
            &mut self.session,
            expected_revision,
            expected_digest,
            molecule_id,
            selected_atom_ids,
        )
        .map(Into::into)
    }

    /// Create one authenticated explicit molecule-local fragment annotation.
    #[allow(clippy::too_many_arguments)]
    fn create_explicit_fragment_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest: &Bound<'_, pyo3::types::PyString>,
        molecule_id: &Bound<'_, pyo3::types::PyString>,
        name: &Bound<'_, pyo3::types::PyString>,
        selected_atom_ids: &Bound<'_, pyo3::types::PyAny>,
        selected_bond_ids: &Bound<'_, pyo3::types::PyAny>,
    ) -> PyResult<super::document_explicit_fragment_binding::PyDocumentExplicitFragmentCreateResultV1>
    {
        super::document_explicit_fragment_binding::create_explicit_fragment_v1(
            py,
            &mut self.session,
            expected_revision,
            expected_digest,
            molecule_id,
            name,
            selected_atom_ids,
            selected_bond_ids,
        )
    }

    /// Apply one worker-prepared clipboard fragment to this exact installed state.
    fn apply_clipboard_paste_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest: &Bound<'_, pyo3::types::PyString>,
        prepared: PyRef<'_, super::clipboard_paste_binding::PyDocumentClipboardPastePlanV1>,
    ) -> PyResult<super::clipboard_paste_binding::PyDocumentClipboardPasteResultV1> {
        super::clipboard_paste_binding::apply_clipboard_paste_v1_binding(
            py,
            &mut self.session,
            expected_revision,
            expected_digest,
            prepared,
        )
    }

    /// Place one worker-prepared user template at this exact installed state.
    fn apply_user_template_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest: &Bound<'_, pyo3::types::PyString>,
        prepared: PyRef<'_, super::document_user_template_binding::PyDocumentUserTemplatePlanV1>,
        anchor_x: f64,
        anchor_y: f64,
    ) -> PyResult<super::document_user_template_binding::PyDocumentUserTemplateResultV1> {
        super::document_user_template_binding::apply_user_template_v1_binding(
            py,
            &mut self.session,
            expected_revision,
            expected_digest,
            prepared,
            anchor_x,
            anchor_y,
        )
    }

    /// Apply one worker-prepared Cut plan to this exact installed state.
    fn apply_clipboard_cut_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest: &Bound<'_, pyo3::types::PyString>,
        prepared: PyRef<'_, super::clipboard_cut_binding::PyDocumentClipboardCutPlanV1>,
    ) -> PyResult<PySessionOperationResultV1> {
        super::clipboard_cut_binding::apply_clipboard_cut_v1_binding(
            py,
            &mut self.session,
            expected_revision,
            expected_digest,
            prepared,
        )
    }

    /// Accept one worker-prepared complete coordinate update at its source revision.
    fn apply_molecule_coordinates_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        prepared: PyRef<'_, PyPreparedMoleculeCoordinatesV1>,
    ) -> PyResult<PySessionOperationResultV1> {
        let operation = SessionOperation::V1(SessionOperationV1::SetMoleculeAtomPositions {
            update: prepared.update().clone(),
        });
        document_result(py, self.session.submit(expected_revision, operation)).map(Into::into)
    }

    /// Accept one worker-prepared multi-molecule clean-geometry update atomically.
    fn apply_clean_geometry_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        prepared: PyRef<'_, PyPreparedCleanGeometryV1>,
    ) -> PyResult<PySessionOperationResultV1> {
        let operation = SessionOperation::V1(SessionOperationV1::SetCleanGeometry {
            update: prepared.update().clone(),
        });
        document_result(py, self.session.submit(expected_revision, operation)).map(Into::into)
    }

    /// Move to the preceding retained state, producing a new monotonic revision.
    fn undo(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(py, self.session.undo(expected_revision)).map(Into::into)
    }

    /// Move to the succeeding retained state, producing a new monotonic revision.
    fn redo(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(py, self.session.redo(expected_revision)).map(Into::into)
    }

    /// Prepare a revision-bound, one-use atom insertion without changing the session.
    #[allow(clippy::too_many_arguments)]
    fn prepare_create_atom_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        molecule_object_id: String,
        element: String,
        x: f64,
        y: f64,
        z: f64,
    ) -> PyResult<PyPreparedAtomInsertion> {
        let molecule_object_id = document_object_id(py, molecule_object_id)?;
        let position = match Point3V1::new(x, y, z) {
            Ok(position) => position,
            Err(error) => return Err(projection_error(py, error)?),
        };
        let pending = document_result(
            py,
            self.session.prepare_create_atom_v1(
                expected_revision,
                &molecule_object_id,
                &element,
                position,
            ),
        )?;
        let identifier = pending.identifier().as_str().to_owned();
        Ok(PyPreparedAtomInsertion {
            pending,
            identifier,
        })
    }

    /// Commit one prepared atom insertion exactly once at its prepared revision.
    fn commit_create_atom(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyPreparedAtomInsertion>,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(
            py,
            self.session
                .commit_create_atom(expected_revision, &mut prepared.pending),
        )
        .map(Into::into)
    }

    /// Prepare a bounded revision-bound Wavy insertion without changing the session.
    fn prepare_create_wavy_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
    ) -> PyResult<PyPreparedWavyInsertion> {
        let start = match Point3V1::new(start_x, start_y, 0.0) {
            Ok(point) => point,
            Err(error) => return Err(projection_error(py, error)?),
        };
        let end = match Point3V1::new(end_x, end_y, 0.0) {
            Ok(point) => point,
            Err(error) => return Err(projection_error(py, error)?),
        };
        let pending = document_result(
            py,
            self.session
                .prepare_create_wavy_v1(expected_revision, start, end),
        )?;
        let identifier = pending.identifier().as_str().to_owned();
        Ok(PyPreparedWavyInsertion {
            pending,
            identifier,
        })
    }

    /// Commit one prepared Wavy insertion exactly once at its prepared revision.
    fn commit_create_wavy(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyPreparedWavyInsertion>,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(
            py,
            self.session
                .commit_create_wavy(expected_revision, &mut prepared.pending),
        )
        .map(Into::into)
    }

    /// Prepare one revision-bound bracket pair without changing the session.
    fn prepare_create_bracket_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        style: PyRef<'_, PyDocumentBracketStyleV1>,
        bounds: PyRef<'_, PyDocumentBracketBoundsV1>,
    ) -> PyResult<PyPreparedBracketInsertion> {
        let pending = document_result(
            py,
            self.session.prepare_create_bracket_v1(
                expected_revision,
                (*style).into(),
                bounds.left,
                bounds.top,
                bounds.right,
                bounds.bottom,
            ),
        )?;
        Ok(PyPreparedBracketInsertion {
            pair_identifier: pending.pair_identifier().as_str().to_owned(),
            left_identifier: pending.left_identifier().as_str().to_owned(),
            right_identifier: pending.right_identifier().as_str().to_owned(),
            pending,
        })
    }

    /// Commit one prepared bracket pair exactly once at its prepared revision.
    fn commit_create_bracket(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyPreparedBracketInsertion>,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(
            py,
            self.session
                .commit_create_bracket(expected_revision, &mut prepared.pending),
        )
        .map(Into::into)
    }

    /// Prepare a revision-bound, one-use molecule-local bond insertion.
    fn prepare_create_bond_v2(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        start_atom_object_id: String,
        end_atom_object_id: String,
        presentation: PyRef<'_, PyDocumentBondPresentationV1>,
    ) -> PyResult<PyPreparedBondInsertion> {
        let start_atom_object_id = document_object_id(py, start_atom_object_id)?;
        let end_atom_object_id = document_object_id(py, end_atom_object_id)?;
        let pending = document_result(
            py,
            self.session.prepare_create_bond_v2(
                expected_revision,
                &start_atom_object_id,
                &end_atom_object_id,
                (*presentation).into(),
            ),
        )?;
        let identifier = pending.identifier().as_str().to_owned();
        Ok(PyPreparedBondInsertion {
            pending,
            identifier,
        })
    }

    /// Commit one prepared bond insertion exactly once at its prepared revision.
    fn commit_create_bond(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyPreparedBondInsertion>,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(
            py,
            self.session
                .commit_create_bond(expected_revision, &mut prepared.pending),
        )
        .map(Into::into)
    }

    /// Prepare one atom plus its bond to an existing atom as one Rust edit.
    #[allow(clippy::too_many_arguments)]
    fn prepare_create_bonded_atom_v2(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        start_atom_object_id: String,
        element: String,
        x: f64,
        y: f64,
        z: f64,
        presentation: PyRef<'_, PyDocumentBondPresentationV1>,
    ) -> PyResult<PyPreparedBondedAtomInsertion> {
        let start_atom_object_id = document_object_id(py, start_atom_object_id)?;
        let position = match Point3V1::new(x, y, z) {
            Ok(position) => position,
            Err(error) => return Err(projection_error(py, error)?),
        };
        let pending = document_result(
            py,
            self.session.prepare_create_bonded_atom_v2(
                expected_revision,
                &start_atom_object_id,
                &element,
                position,
                (*presentation).into(),
            ),
        )?;
        let atom_identifier = pending.atom_identifier().as_str().to_owned();
        let bond_identifier = pending.bond_identifier().as_str().to_owned();
        Ok(PyPreparedBondedAtomInsertion {
            pending,
            atom_identifier,
            bond_identifier,
        })
    }

    /// Commit one prepared atom-plus-bond insertion exactly once.
    fn commit_create_bonded_atom(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyPreparedBondedAtomInsertion>,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(
            py,
            self.session
                .commit_create_bonded_atom(expected_revision, &mut prepared.pending),
        )
        .map(Into::into)
    }

    /// Prepare one worker-built molecule against an exact current revision.
    fn prepare_admitted_molecule_insertion_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        molecule: PyRef<'_, PyMoleculeInsertionV1>,
    ) -> PyResult<PyAdmittedMoleculeInsertionV1> {
        let pending = document_result(
            py,
            self.session
                .prepare_admitted_molecule_insertion_v1(expected_revision, molecule.insertion()),
        )?;
        let molecule_identifier = pending.molecule_identifier().as_str().to_owned();
        Ok(PyAdmittedMoleculeInsertionV1 {
            pending,
            molecule_identifier,
        })
    }

    /// Commit one complete prepared molecule exactly once at its prepared revision.
    fn commit_admitted_molecule_insertion_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyAdmittedMoleculeInsertionV1>,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(
            py,
            self.session
                .commit_admitted_molecule_insertion_v1(expected_revision, &mut prepared.pending),
        )
        .map(Into::into)
    }

    /// Prepare every worker-built interchange record as one exact-revision transaction.
    fn prepare_admitted_interchange_records_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        batch: PyRef<'_, PyInterchangeRecordBatchInsertionV1>,
    ) -> PyResult<PyAdmittedInterchangeRecordInsertionV1> {
        let pending = document_result(
            py,
            self.session
                .prepare_admitted_interchange_records_v1(expected_revision, batch.batch()),
        )?;
        Ok(PyAdmittedInterchangeRecordInsertionV1::new(pending))
    }

    /// Commit one complete prepared interchange batch exactly once.
    fn commit_admitted_interchange_records_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyAdmittedInterchangeRecordInsertionV1>,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(
            py,
            self.session
                .commit_admitted_interchange_records_v1(expected_revision, &mut prepared.pending),
        )
        .map(Into::into)
    }
}

pub(crate) fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
