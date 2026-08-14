use std::path::PathBuf;

use ferrum_document::{
    DocumentBondOrderV1, DocumentSession, DocumentSnapshot, PendingCreateAtom, PendingCreateBond,
    PendingCreateBondedAtom, PendingCreateMolecule, PendingCreateWavy, Point3V1, Publication,
    SaveOutcome, SessionOperation, SessionOperationResultV1, SessionOperationV1,
};
use pyo3::prelude::*;

use crate::atom_mark_binding::{PyAtomMarkActionV1, PyAtomMarkKindV1};
use crate::atom_properties_binding::PyDocumentAtomPropertyChangeV1;
use crate::bond_properties_binding::{PyDocumentBondPropertyChangeV1, PyDocumentBondStyleV1};
use crate::bracket_binding::{
    PyBracketPairProjectionV1, PyDocumentBracketBoundsV1, PyDocumentBracketStyleV1,
    PyPreparedBracketInsertion,
};
pub(crate) use crate::document_error_binding::{
    DocumentError, DocumentInputError, DocumentLoadError, DocumentSerializationError, FerrumError,
    HistoryUnavailableError, InvalidAtomElementError, InvalidDestinationError,
    InvalidDocumentObjectIdError, OperationValidationError, PreparedOperationConsumedError,
    PreparedOperationError, PreparedOperationForeignSessionError, ProjectionError,
    PublicationError, PublicationNotStartedError, PublicationPossiblyCompletedError,
    RevisionConflictError, RevisionExhaustedError, UnknownDocumentObjectError, document_object_id,
    document_result, map_document_error, operation_validation_error, projection_error,
};
use crate::document_operation_binding::PyDocumentOperationV1;
use crate::molecule_coordinate_binding::{
    PyPreparedCleanGeometryV1, PyPreparedMoleculeCoordinatesV1,
};
use crate::presentation_root_binding::PyPresentationRootProjectionV1;
use crate::projection_binding::{
    PyArrowHeadShapeV1, PyArrowHeadV1, PyArrowPathV1, PyArrowProjectionV1, PyAtomMarkProjectionV1,
    PyAtomProjectionV1, PyBondEndpointV1, PyBondProjectionV1, PyBoxShapeProjectionV1,
    PyDocumentHaworthPositionV1, PyDocumentProjectionV1, PyFontFactsV1, PyMoleculeProjectionV1,
    PyPlusProjectionV1, PyPoint3V1, PyPolygonPathV1, PyPolygonProjectionV1, PyPolylinePathV1,
    PyPolylineProjectionV1, PyPresentationBoundsV1, PyPresentationFillV1, PyPresentationFontV1,
    PyPresentationProjectionIssueV1, PyPresentationStackProjectionV1, PyPresentationStrokeV1,
    PyPresentationTargetV1, PyProjectionIssueV1, PySessionDocumentObservationV1,
};
use crate::render_binding::{self, PyRenderObservationV1};
use crate::sdf_insertion_binding::{PyPreparedSdfRecordInsertion, PySdfMoleculeBatchInsertionV1};
use crate::smiles_insertion_binding::PyMoleculeInsertionV1;

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
struct PySessionOperationResultV1 {
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
struct PySaveOutcome {
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
struct PyPublication {
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
struct PyPreparedAtomInsertion {
    pending: PendingCreateAtom,
    #[pyo3(get)]
    identifier: String,
}

/// Opaque one-use prepared Wavy insertion.
#[pyclass(unsendable, module = "ferrum_chem", name = "PreparedWavyInsertion")]
struct PyPreparedWavyInsertion {
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

/// Opaque one-use prepared molecule-local bond insertion.
#[pyclass(unsendable, module = "ferrum_chem", name = "PreparedBondInsertion")]
struct PyPreparedBondInsertion {
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
struct PyPreparedBondedAtomInsertion {
    pending: PendingCreateBondedAtom,
    #[pyo3(get)]
    atom_identifier: String,
    #[pyo3(get)]
    bond_identifier: String,
}

/// Opaque one-use prepared complete molecule insertion.
#[pyclass(unsendable, module = "ferrum_chem", name = "PreparedMoleculeInsertion")]
struct PyPreparedMoleculeInsertion {
    pending: PendingCreateMolecule,
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
    session: DocumentSession,
}

impl PyDocumentSession {
    pub(crate) fn from_session(session: DocumentSession) -> Self {
        Self { session }
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
        Ok(Self { session })
    }

    /// Create a session from already-allocated CDML copied during this call.
    ///
    /// This unbounded compatibility route applies no resource policy. External input must use
    /// an explicit-budget byte or file admission method instead.
    #[staticmethod]
    fn load(py: Python<'_>, cdml: &str) -> PyResult<Self> {
        let session = document_result(py, DocumentSession::load(cdml))?;
        Ok(Self { session })
    }

    /// Admit exact built-in bytes as CDML under one explicit caller-owned budget.
    #[staticmethod]
    fn load_utf8_bytes_with_budget(
        py: Python<'_>,
        source: &Bound<'_, pyo3::types::PyAny>,
        budget: &Bound<'_, pyo3::types::PyAny>,
    ) -> PyResult<Self> {
        crate::document_ingress_binding::load_utf8_bytes_with_budget(py, source, budget)
    }

    /// Admit an exact built-in string local path as CDML under one explicit budget.
    #[staticmethod]
    fn load_file_with_budget(
        py: Python<'_>,
        path: &Bound<'_, pyo3::types::PyAny>,
        budget: &Bound<'_, pyo3::types::PyAny>,
    ) -> PyResult<Self> {
        crate::document_ingress_binding::load_file_with_budget(py, path, budget)
    }

    /// Admit exact built-in bytes as CD-SVG under independent wrapper and payload budgets.
    #[staticmethod]
    fn load_cdsvg_utf8_bytes_with_budget(
        py: Python<'_>,
        source: &Bound<'_, pyo3::types::PyAny>,
        wrapper_budget: &Bound<'_, pyo3::types::PyAny>,
        payload_budget: &Bound<'_, pyo3::types::PyAny>,
    ) -> PyResult<Self> {
        crate::document_ingress_binding::load_cdsvg_utf8_bytes_with_budget(
            py,
            source,
            wrapper_budget,
            payload_budget,
        )
    }

    /// Admit an exact built-in string local path as CD-SVG with independent budgets.
    #[staticmethod]
    fn load_cdsvg_file_with_budget(
        py: Python<'_>,
        path: &Bound<'_, pyo3::types::PyAny>,
        wrapper_budget: &Bound<'_, pyo3::types::PyAny>,
        payload_budget: &Bound<'_, pyo3::types::PyAny>,
    ) -> PyResult<Self> {
        crate::document_ingress_binding::load_cdsvg_file_with_budget(
            py,
            path,
            wrapper_budget,
            payload_budget,
        )
    }

    /// Return one immutable owned snapshot without changing session state.
    fn snapshot(&self, py: Python<'_>) -> PyResult<PyDocumentSnapshot> {
        document_result(py, self.session.snapshot()).map(PyDocumentSnapshot::from)
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
        &self,
        py: Python<'_>,
        expected_revision: u64,
    ) -> PyResult<PyRenderObservationV1> {
        render_binding::result(
            py,
            ferrum_api::observe_render_v1(&self.session, expected_revision),
        )
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
    fn prepare_create_bond_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        start_atom_object_id: String,
        end_atom_object_id: String,
        order: PyRef<'_, PyDocumentBondOrderV1>,
    ) -> PyResult<PyPreparedBondInsertion> {
        let start_atom_object_id = document_object_id(py, start_atom_object_id)?;
        let end_atom_object_id = document_object_id(py, end_atom_object_id)?;
        let pending = document_result(
            py,
            self.session.prepare_create_bond_v1(
                expected_revision,
                &start_atom_object_id,
                &end_atom_object_id,
                (*order).into(),
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
    fn prepare_create_bonded_atom_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        start_atom_object_id: String,
        element: String,
        x: f64,
        y: f64,
        z: f64,
        order: PyRef<'_, PyDocumentBondOrderV1>,
    ) -> PyResult<PyPreparedBondedAtomInsertion> {
        let start_atom_object_id = document_object_id(py, start_atom_object_id)?;
        let position = match Point3V1::new(x, y, z) {
            Ok(position) => position,
            Err(error) => return Err(projection_error(py, error)?),
        };
        let pending = document_result(
            py,
            self.session.prepare_create_bonded_atom_v1(
                expected_revision,
                &start_atom_object_id,
                &element,
                position,
                (*order).into(),
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
    fn prepare_insert_molecule_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        molecule: PyRef<'_, PyMoleculeInsertionV1>,
    ) -> PyResult<PyPreparedMoleculeInsertion> {
        let pending = document_result(
            py,
            self.session
                .prepare_create_molecule_v1(expected_revision, molecule.insertion()),
        )?;
        let molecule_identifier = pending.molecule_identifier().as_str().to_owned();
        Ok(PyPreparedMoleculeInsertion {
            pending,
            molecule_identifier,
        })
    }

    /// Commit one complete prepared molecule exactly once at its prepared revision.
    fn commit_create_molecule(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyPreparedMoleculeInsertion>,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(
            py,
            self.session
                .commit_create_molecule(expected_revision, &mut prepared.pending),
        )
        .map(Into::into)
    }

    /// Prepare every worker-built SDF record as one exact-revision transaction.
    fn prepare_insert_sdf_records_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        batch: PyRef<'_, PySdfMoleculeBatchInsertionV1>,
    ) -> PyResult<PyPreparedSdfRecordInsertion> {
        let pending = document_result(
            py,
            self.session
                .prepare_create_sdf_records_v1(expected_revision, batch.batch()),
        )?;
        Ok(PyPreparedSdfRecordInsertion::new(pending))
    }

    /// Commit one complete prepared SDF batch exactly once.
    fn commit_create_sdf_records(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyPreparedSdfRecordInsertion>,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(
            py,
            self.session
                .commit_create_sdf_records(expected_revision, &mut prepared.pending),
        )
        .map(Into::into)
    }

    /// Publish the current revision and update the saved baseline only if confirmed.
    ///
    /// The path is copied before publication. This operation requires an explicit
    /// revision so an unrelated stale caller cannot silently write session state.
    fn save_atomic(
        &mut self,
        py: Python<'_>,
        path: PathBuf,
        expected_revision: u64,
    ) -> PyResult<PyPublication> {
        document_result(py, self.session.save_atomic(&path, expected_revision)).map(Into::into)
    }

    /// Export the current revision without changing baseline, history, or dirty state.
    fn recovery_export(
        &self,
        py: Python<'_>,
        path: PathBuf,
        expected_revision: u64,
    ) -> PyResult<PyPublication> {
        document_result(py, self.session.recovery_export(&path, expected_revision)).map(Into::into)
    }
}

fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Register Ferrum-Chem's public Python API in its extension module.
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("FerrumError", module.py().get_type::<FerrumError>())?;
    crate::chemistry_binding::initialize(module)?;
    crate::document_molecule_inchi_binding::initialize(module)?;
    crate::geometry_binding::initialize(module)?;
    crate::molecule_coordinate_binding::initialize(module)?;
    crate::paper_properties_binding::initialize(module)?;
    crate::paper_size_binding::initialize(module)?;
    crate::periodic_display_binding::initialize(module)?;
    crate::smiles_insertion_binding::initialize(module)?;
    crate::inchi_insertion_binding::initialize(module)?;
    crate::molblock_insertion_binding::initialize(module)?;
    crate::sdf_insertion_binding::initialize(module)?;
    module.add("DocumentError", module.py().get_type::<DocumentError>())?;
    module.add(
        "DocumentInputError",
        module.py().get_type::<DocumentInputError>(),
    )?;
    module.add(
        "DocumentLoadError",
        module.py().get_type::<DocumentLoadError>(),
    )?;
    module.add(
        "PreparedOperationForeignSessionError",
        module
            .py()
            .get_type::<PreparedOperationForeignSessionError>(),
    )?;
    module.add(
        "DocumentSerializationError",
        module.py().get_type::<DocumentSerializationError>(),
    )?;
    module.add(
        "RevisionConflictError",
        module.py().get_type::<RevisionConflictError>(),
    )?;
    module.add(
        "RevisionExhaustedError",
        module.py().get_type::<RevisionExhaustedError>(),
    )?;
    module.add(
        "HistoryUnavailableError",
        module.py().get_type::<HistoryUnavailableError>(),
    )?;
    module.add("ProjectionError", module.py().get_type::<ProjectionError>())?;
    module.add(
        "OperationValidationError",
        module.py().get_type::<OperationValidationError>(),
    )?;
    module.add(
        "InvalidAtomElementError",
        module.py().get_type::<InvalidAtomElementError>(),
    )?;
    module.add(
        "InvalidDocumentObjectIdError",
        module.py().get_type::<InvalidDocumentObjectIdError>(),
    )?;
    module.add(
        "UnknownDocumentObjectError",
        module.py().get_type::<UnknownDocumentObjectError>(),
    )?;
    module.add(
        "PreparedOperationError",
        module.py().get_type::<PreparedOperationError>(),
    )?;
    module.add(
        "PreparedOperationConsumedError",
        module.py().get_type::<PreparedOperationConsumedError>(),
    )?;
    module.add(
        "PublicationError",
        module.py().get_type::<PublicationError>(),
    )?;
    module.add(
        "InvalidDestinationError",
        module.py().get_type::<InvalidDestinationError>(),
    )?;
    module.add(
        "PublicationNotStartedError",
        module.py().get_type::<PublicationNotStartedError>(),
    )?;
    module.add(
        "PublicationPossiblyCompletedError",
        module.py().get_type::<PublicationPossiblyCompletedError>(),
    )?;
    module.add_class::<PyDocumentSession>()?;
    module.add_class::<crate::document_ingress_binding::PyXmlInputBudgetV1>()?;
    module.add_class::<PyDocumentSnapshot>()?;
    module.add_class::<PySessionDocumentObservationV1>()?;
    module.add_class::<PyDocumentProjectionV1>()?;
    module.add_class::<PyPresentationStackProjectionV1>()?;
    module.add_class::<PyBracketPairProjectionV1>()?;
    module.add_class::<PyPresentationRootProjectionV1>()?;
    crate::presentation_text_binding::register(module)?;
    module.add_class::<PyArrowProjectionV1>()?;
    module.add_class::<PyArrowPathV1>()?;
    module.add_class::<PyArrowHeadShapeV1>()?;
    module.add_class::<PyArrowHeadV1>()?;
    module.add_class::<PyPlusProjectionV1>()?;
    module.add_class::<PyPresentationFontV1>()?;
    module.add_class::<PyPolylineProjectionV1>()?;
    module.add_class::<PyBoxShapeProjectionV1>()?;
    module.add_class::<PyPolygonProjectionV1>()?;
    module.add_class::<PyPresentationTargetV1>()?;
    module.add_class::<PyPolylinePathV1>()?;
    module.add_class::<PyPolygonPathV1>()?;
    module.add_class::<PyPresentationStrokeV1>()?;
    module.add_class::<PyPresentationBoundsV1>()?;
    module.add_class::<PyPresentationFillV1>()?;
    module.add_class::<PyPresentationProjectionIssueV1>()?;
    module.add_class::<PyMoleculeProjectionV1>()?;
    module.add_class::<PyAtomMarkProjectionV1>()?;
    module.add_class::<PyAtomProjectionV1>()?;
    module.add_class::<PyBondProjectionV1>()?;
    module.add_class::<PyDocumentHaworthPositionV1>()?;
    module.add_class::<PyBondEndpointV1>()?;
    module.add_class::<PyPoint3V1>()?;
    module.add_class::<PyFontFactsV1>()?;
    module.add_class::<PyProjectionIssueV1>()?;
    module.add_class::<PySessionOperationResultV1>()?;
    module.add_class::<PyDocumentOperationV1>()?;
    module.add_class::<PyAtomMarkActionV1>()?;
    module.add_class::<PyAtomMarkKindV1>()?;
    module.add_class::<PyDocumentAtomPropertyChangeV1>()?;
    module.add_class::<crate::atom_rotation_binding::PyDocumentAtomRotationTargetV1>()?;
    module.add_class::<crate::geometry_repair_binding::PyDocumentGeometryRepairKindV1>()?;
    module.add_class::<PyDocumentBondPropertyChangeV1>()?;
    module.add_class::<crate::plus_properties_binding::PyDocumentPlusPropertyChangeV1>()?;
    module.add_class::<crate::text_properties_binding::PyDocumentTextEditStyleV1>()?;
    module.add_class::<crate::text_properties_binding::PyDocumentTextEditRunV1>()?;
    module.add_class::<crate::text_properties_binding::PyDocumentTextPropertyChangeV1>()?;
    module.add_class::<crate::presentation_deletion_binding::PyDocumentPresentationRootKindV1>()?;
    module.add_class::<crate::presentation_stack_binding::PyDocumentPresentationStackOrderV1>()?;
    module
        .add_class::<crate::presentation_stack_binding::PyDocumentPresentationRootSelectorV1>()?;
    module.add_class::<crate::top_level_transform_binding::PyDocumentTopLevelRootKindV1>()?;
    module.add_class::<crate::top_level_transform_binding::PyDocumentTopLevelRootSelectorV1>()?;
    module.add_class::<crate::top_level_transform_binding::PyDocumentTopLevelAlignmentV1>()?;
    module.add_class::<crate::top_level_transform_binding::PyDocumentTopLevelMirrorV1>()?;
    module.add_class::<crate::arrow_properties_binding::PyDocumentArrowPropertyChangeV1>()?;
    module
        .add_class::<crate::geometric_properties_binding::PyDocumentGeometricPropertyChangeV1>()?;
    module.add_class::<crate::wavy_properties_binding::PyDocumentWavyPropertyChangeV1>()?;
    module.add_class::<crate::bracket_binding::PyDocumentBracketPropertyChangeV1>()?;
    module.add_class::<PyPreparedAtomInsertion>()?;
    module.add_class::<PyPreparedWavyInsertion>()?;
    module.add_class::<PyDocumentBracketStyleV1>()?;
    module.add_class::<PyDocumentBracketBoundsV1>()?;
    module.add_class::<PyPreparedBracketInsertion>()?;
    module.add_class::<PyDocumentBondOrderV1>()?;
    module.add_class::<PyDocumentBondStyleV1>()?;
    module.add_class::<PyPreparedBondInsertion>()?;
    module.add_class::<PyPreparedBondedAtomInsertion>()?;
    module.add_class::<PyPreparedMoleculeInsertion>()?;
    module.add_class::<PyPublication>()?;
    module.add_class::<PySaveOutcome>()?;
    module.add(
        "RenderObservationError",
        module
            .py()
            .get_type::<render_binding::RenderObservationError>(),
    )?;
    module.add(
        "RenderDepictionError",
        module
            .py()
            .get_type::<render_binding::RenderDepictionError>(),
    )?;
    module.add(
        "RenderProvenanceError",
        module
            .py()
            .get_type::<render_binding::RenderProvenanceError>(),
    )?;
    module.add_function(wrap_pyfunction!(
        render_binding::verified_telex_regular,
        module
    )?)?;
    module.add_class::<PyRenderObservationV1>()?;
    module.add_class::<render_binding::PyMoleculeRenderRootV1>()?;
    module.add_class::<render_binding::PyDocumentMoleculeRenderPlanV1>()?;
    module.add_class::<render_binding::PyRenderPlanV1>()?;
    module.add_class::<render_binding::PyRenderProvenanceV1>()?;
    module.add_class::<render_binding::PyRenderBatchV1>()?;
    module.add_class::<render_binding::PyRenderTargetV1>()?;
    module.add_class::<render_binding::PyRenderRecordIdV1>()?;
    module.add_class::<render_binding::PyAtomLocalSpaceV1>()?;
    module.add_class::<render_binding::PySceneSpaceV1>()?;
    module.add_class::<render_binding::PyRenderOperationV1>()?;
    module.add_class::<render_binding::PyTextOpV1>()?;
    module.add_class::<render_binding::PyTextRunV1>()?;
    module.add_class::<render_binding::PyGlyphPlacementV1>()?;
    module.add_class::<render_binding::PyLineOpV1>()?;
    module.add_class::<render_binding::PyMaskOpV1>()?;
    module.add_class::<render_binding::PyEllipseOpV1>()?;
    module.add_class::<render_binding::PyRenderIssueV1>()?;
    module.add_class::<render_binding::PyDepictionIssueV1>()?;
    module.add_class::<render_binding::PyDocumentPlusRenderV1>()?;
    crate::presentation_text_render_binding::register(module)?;
    module.add_class::<render_binding::PyPresentationTextBoundsV1>()?;
    module.add_class::<render_binding::PyRenderPointV1>()?;
    module.add_class::<render_binding::PyVerifiedTelexRegularV1>()
}
