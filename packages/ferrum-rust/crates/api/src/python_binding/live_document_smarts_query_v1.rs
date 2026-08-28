//! Private published-plan SMARTS attachment for one Python document session.
//!
//! A receipt is only a session-bound opaque key. Durable record identities,
//! graph positions, and renderer correspondence remain private to this module.

use std::collections::{HashMap, HashSet};

use ferrum_chemistry::{
    ChemEngine, ChemistryError, SmartsMatchOptions, SmartsMatchUnavailableReason,
};
use ferrum_document::{
    DocumentRenderObservationV2, SessionDocumentObservationV1,
    derive_document_render_observation_from_accepted_operation_v2,
};
use ferrum_render::{BatchSpace, RenderTarget};
use getrandom::fill;
use pyo3::{create_exception, prelude::*};

use crate::{
    RenderInteractionSessionV1, protocol::document_smarts_snapshot::OwnedDocumentSmartsSnapshot,
};

const MAX_QUERY_BYTES: usize = 8_192;
const MAX_PER_MOLECULE: u32 = 128;
const MAX_TOTAL: u32 = 256;

create_exception!(
    ferrum_chem,
    LiveDocumentSmartsError,
    super::binding::FerrumError
);

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "LiveDocumentSmartsCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PyLiveDocumentSmartsCategoryV1 {
    InvalidQuery,
    UnsupportedDocument,
    ResourceLimit,
    Stale,
    Unavailable,
    Refused,
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "LiveDocumentSmartsReasonV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PyLiveDocumentSmartsReasonV1 {
    EmptyQuery,
    QueryTooLong,
    InvalidQuery,
    MatchCapsInconsistent,
    SelectedRootEmpty,
    SelectedRootMultiple,
    SelectedTargetNotMolecule,
    UnsupportedDocument,
    StaleDocument,
    StaleSelection,
    ForeignSelection,
    PlanNotPublished,
    NativeRuntimeUnavailable,
    MatchUnavailable,
    ReceiptUnavailable,
    PaintUnavailable,
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "LiveDocumentSmartsRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PyLiveDocumentSmartsRecoveryV1 {
    EditQuery,
    ReduceScope,
    SelectOneMolecule,
    RefreshAndRerun,
    Retry,
}

#[derive(Clone, Copy)]
pub(crate) enum LiveFailure {
    InvalidQuery(PyLiveDocumentSmartsReasonV1),
    UnsupportedDocument(PyLiveDocumentSmartsReasonV1),
    ResourceLimit(PyLiveDocumentSmartsReasonV1),
    Stale(PyLiveDocumentSmartsReasonV1),
    Unavailable(PyLiveDocumentSmartsReasonV1),
    Refused(PyLiveDocumentSmartsReasonV1),
}

impl LiveFailure {
    const fn facts(
        self,
    ) -> (
        PyLiveDocumentSmartsCategoryV1,
        PyLiveDocumentSmartsReasonV1,
        PyLiveDocumentSmartsRecoveryV1,
    ) {
        match self {
            Self::InvalidQuery(reason) => (
                PyLiveDocumentSmartsCategoryV1::InvalidQuery,
                reason,
                PyLiveDocumentSmartsRecoveryV1::EditQuery,
            ),
            Self::UnsupportedDocument(reason) => (
                PyLiveDocumentSmartsCategoryV1::UnsupportedDocument,
                reason,
                PyLiveDocumentSmartsRecoveryV1::RefreshAndRerun,
            ),
            Self::ResourceLimit(reason) => (
                PyLiveDocumentSmartsCategoryV1::ResourceLimit,
                reason,
                PyLiveDocumentSmartsRecoveryV1::ReduceScope,
            ),
            Self::Stale(reason) => (
                PyLiveDocumentSmartsCategoryV1::Stale,
                reason,
                PyLiveDocumentSmartsRecoveryV1::RefreshAndRerun,
            ),
            Self::Unavailable(reason) => (
                PyLiveDocumentSmartsCategoryV1::Unavailable,
                reason,
                PyLiveDocumentSmartsRecoveryV1::Retry,
            ),
            Self::Refused(reason) => (
                PyLiveDocumentSmartsCategoryV1::Refused,
                reason,
                PyLiveDocumentSmartsRecoveryV1::SelectOneMolecule,
            ),
        }
    }

    pub(crate) fn into_pyerr(self) -> PyErr {
        let (category, reason, recovery) = self.facts();
        let error = LiveDocumentSmartsError::new_err("SMARTS query cannot continue");
        Python::attach(|py| {
            let value = error.value(py);
            value
                .setattr("category", Py::new(py, category).expect("enum allocation"))
                .expect("category attaches");
            value
                .setattr("reason", Py::new(py, reason).expect("enum allocation"))
                .expect("reason attaches");
            value
                .setattr("recovery", Py::new(py, recovery).expect("enum allocation"))
                .expect("recovery attaches");
        });
        error
    }
}

#[derive(Eq, Hash, PartialEq, Clone)]
struct LiveSmartsIssuer([u8; 32]);
#[derive(Eq, Hash, PartialEq, Clone)]
struct LiveSmartsReceiptKey([u8; 32]);

struct LiveSmartsPlan {
    generation: u64,
    revision: u64,
    digest: [u8; 32],
    snapshot: OwnedDocumentSmartsSnapshot,
    atom_points_by_graph_position: Vec<Vec<(f64, f64)>>,
}

/// The publication result for the current document fence.
///
/// A render observation may be accepted while excluding a molecule from the
/// V1 depiction profile. That document remains openable, but live SMARTS must
/// not claim a paintable plan that the renderer did not produce.
enum LiveSmartsReadiness {
    Unpublished,
    UnsupportedDocument { revision: u64, digest: [u8; 32] },
    Ready(LiveSmartsPlan),
}

#[derive(Clone)]
struct LiveSmartsRow {
    target_index: usize,
    positions: Vec<usize>,
}
enum LiveSmartsReceiptRow {
    Available(LiveSmartsRow),
    Reserved,
}
struct LiveSmartsReceiptState {
    generation: u64,
    revision: u64,
    digest: [u8; 32],
    rows: Vec<LiveSmartsReceiptRow>,
}

/// Opaque, nonconstructible session receipt. It has no key, issuer, integer,
/// string, serialization, or debug surface.
#[pyclass(
    frozen,
    unsendable,
    module = "ferrum_chem",
    name = "_LiveDocumentSmartsReceipt",
    skip_from_py_object
)]
pub(crate) struct PyLiveDocumentSmartsReceipt {
    issuer: LiveSmartsIssuer,
    key: LiveSmartsReceiptKey,
}

/// Opaque selected-query capability minted by one document session. Generic
/// render selections intentionally remain inspectable for authoring tools;
/// this separate capability prevents that display surface from becoming the
/// SMARTS query authority.
#[pyclass(
    frozen,
    unsendable,
    module = "ferrum_chem",
    name = "_LiveDocumentSmartsSelectedQuery",
    skip_from_py_object
)]
pub(crate) struct PyLiveDocumentSmartsSelectedQuery {
    issuer: LiveSmartsIssuer,
    selection: ferrum_document_render::RenderInteractionSelectionV1,
}

/// Closed readiness facts for an opaque selected-query capability.
///
/// The optional failure facts are absent only when the capability passes the
/// same admission checks that `run_selected` will perform. No selection or
/// document data is exposed through this Python value.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "_LiveDocumentSmartsSelectedReadiness",
    skip_from_py_object
)]
pub(crate) struct PyLiveDocumentSmartsSelectedReadiness {
    #[pyo3(get)]
    available: bool,
    #[pyo3(get)]
    category: Option<PyLiveDocumentSmartsCategoryV1>,
    #[pyo3(get)]
    reason: Option<PyLiveDocumentSmartsReasonV1>,
    #[pyo3(get)]
    recovery: Option<PyLiveDocumentSmartsRecoveryV1>,
}

#[pyclass(
    frozen,
    name = "_LiveDocumentSmartsMoleculeSummary",
    skip_from_py_object
)]
pub(crate) struct PyLiveDocumentSmartsMoleculeSummary {
    #[pyo3(get)]
    match_count: u32,
    #[pyo3(get)]
    completeness: String,
}
#[pyclass(frozen, name = "_LiveDocumentSmartsRunSummary", skip_from_py_object)]
pub(crate) struct PyLiveDocumentSmartsRunSummary {
    #[pyo3(get)]
    receipt: Py<PyLiveDocumentSmartsReceipt>,
    #[pyo3(get)]
    traversal: String,
    #[pyo3(get)]
    molecules: Vec<Py<PyLiveDocumentSmartsMoleculeSummary>>,
}
#[pyclass(frozen, name = "_LiveDocumentSmartsPaint", skip_from_py_object)]
pub(crate) struct PyLiveDocumentSmartsPaint {
    #[pyo3(get)]
    atom_bounds: Vec<(f64, f64, f64, f64)>,
}

pub(crate) struct LiveDocumentSmartsBridge {
    issuer: Option<LiveSmartsIssuer>,
    next_generation: u64,
    readiness: LiveSmartsReadiness,
    receipts: HashMap<LiveSmartsReceiptKey, LiveSmartsReceiptState>,
}

impl LiveDocumentSmartsBridge {
    pub(crate) fn new() -> Self {
        // Secure entropy mints every opaque live capability. If it is absent,
        // this bridge remains display-only and all capability issuance refuses.
        Self {
            issuer: random_bytes().ok().map(LiveSmartsIssuer),
            next_generation: 0,
            readiness: LiveSmartsReadiness::Unpublished,
            receipts: HashMap::new(),
        }
    }
    #[cfg(test)]
    pub(crate) fn new_without_entropy() -> Self {
        Self {
            issuer: None,
            next_generation: 0,
            readiness: LiveSmartsReadiness::Unpublished,
            receipts: HashMap::new(),
        }
    }
    pub(crate) fn clear_published_plan(&mut self) {
        self.clear_receipts();
        self.readiness = LiveSmartsReadiness::Unpublished;
    }

    /// Invalidate every derived display capability while retaining the immutable
    /// renderer plan that authoritatively produced them. Query-level UI
    /// cleanup uses this narrower boundary; document lifecycle transitions
    /// retain `clear_published_plan` because they also invalidate the plan itself.
    pub(crate) fn clear_receipts(&mut self) {
        self.receipts.clear();
    }

    /// Create the only plan from one accepted observation. The generation is
    /// advanced and old receipts cleared before the replacement is attempted.
    #[cfg(test)]
    pub(crate) fn publish(
        &mut self,
        session: &RenderInteractionSessionV1,
        expected_revision: u64,
    ) -> PyResult<DocumentRenderObservationV2> {
        let observation = session
            .observe(expected_revision)
            .map_err(|_| unavailable_failure().into_pyerr())?;
        self.publish_from_observation(observation)
    }

    /// Initialize this private bridge from an observation already admitted by
    /// the public session boundary, retaining that exact fence for the render
    /// observation and SMARTS snapshot.
    pub(crate) fn publish_from_observation(
        &mut self,
        observation: SessionDocumentObservationV1,
    ) -> PyResult<DocumentRenderObservationV2> {
        self.clear_published_plan();
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .ok_or_else(|| unavailable_failure().into_pyerr())?;
        let rendered = derive_document_render_observation_from_accepted_operation_v2(&observation)
            .map_err(|_| unavailable_failure().into_pyerr())?;
        let revision = observation.snapshot().revision();
        let digest = *observation.snapshot().digest();
        let snapshot = match OwnedDocumentSmartsSnapshot::from_accepted_observation(observation) {
            Ok(snapshot) => snapshot,
            Err(_) => {
                self.readiness = LiveSmartsReadiness::UnsupportedDocument { revision, digest };
                return Ok(rendered);
            }
        };
        let mut expected = HashSet::new();
        for target in snapshot.targets() {
            for object_id in target.graph_position_to_document_object_ids() {
                if !expected.insert(object_id.clone()) {
                    return Err(unavailable_failure().into_pyerr());
                }
            }
        }
        let mut anchors = HashMap::new();
        for plan in rendered.resolved().molecule_plans() {
            for batch in plan.batches() {
                let BatchSpace::AtomLocal { anchor } = batch.coordinate_space() else {
                    continue;
                };
                let object_id = batch.target().document_object_id();
                if !expected.contains(object_id)
                    || anchors
                        .insert(object_id.clone(), (anchor.x(), anchor.y()))
                        .is_some()
                {
                    return Err(unavailable_failure().into_pyerr());
                }
            }
        }
        if anchors.len() != expected.len() {
            self.readiness = LiveSmartsReadiness::UnsupportedDocument {
                revision: snapshot.revision(),
                digest: *snapshot.digest(),
            };
            return Ok(rendered);
        }
        let mut atom_points_by_graph_position = Vec::new();
        for target in snapshot.targets() {
            let mut points = Vec::new();
            for object_id in target.graph_position_to_document_object_ids() {
                let point = anchors
                    .get(object_id)
                    .copied()
                    .ok_or_else(|| unavailable_failure().into_pyerr())?;
                if !point.0.is_finite() || !point.1.is_finite() {
                    return Err(unavailable_failure().into_pyerr());
                }
                points.push(point);
            }
            if points.len() != target.graph().atoms().len() {
                return Err(unavailable_failure().into_pyerr());
            }
            atom_points_by_graph_position.push(points);
        }
        self.readiness = LiveSmartsReadiness::Ready(LiveSmartsPlan {
            generation: self.next_generation,
            revision: snapshot.revision(),
            digest: *snapshot.digest(),
            snapshot,
            atom_points_by_graph_position,
        });
        Ok(rendered)
    }

    pub(crate) fn run(
        &mut self,
        py: Python<'_>,
        session: &RenderInteractionSessionV1,
        engine: &dyn ChemEngine,
        query: String,
        per: u32,
        total: u32,
    ) -> PyResult<Py<PyLiveDocumentSmartsRunSummary>> {
        // Fail before query execution or receipt-state mutation when secure
        // entropy could not establish this bridge's capability namespace.
        if self.issuer.is_none() {
            return Err(unavailable_failure().into_pyerr());
        }
        self.validate_raw_request(session, &query, per, total)?;
        self.receipts.clear();
        let plan = self.validate_plan_current(session)?;
        let query = valid_query(query)?;
        let mut remaining = total;
        let mut incomplete = false;
        let mut rows = Vec::new();
        let mut molecules = Vec::new();
        for (target_index, target) in plan.snapshot.targets().iter().enumerate() {
            if remaining == 0 {
                incomplete = true;
                break;
            }
            let result = engine
                .smarts_match(
                    &query,
                    target.graph(),
                    SmartsMatchOptions::new(per.min(remaining))
                        .map_err(|_| unavailable_failure().into_pyerr())?,
                )
                .map_err(|error| map_chemistry_error(error).into_pyerr())?;
            let count = u32::try_from(result.rows().len())
                .map_err(|_| unavailable_failure().into_pyerr())?;
            remaining = remaining
                .checked_sub(count)
                .ok_or_else(|| unavailable_failure().into_pyerr())?;
            if count == 0 {
                continue;
            }
            for row in result.rows() {
                let mut seen = HashSet::new();
                let mut positions = Vec::new();
                for &position in row {
                    if !seen.insert(position)
                        || position >= plan.atom_points_by_graph_position[target_index].len()
                    {
                        return Err(unavailable_failure().into_pyerr());
                    }
                    positions.push(position);
                }
                if positions.is_empty() {
                    return Err(unavailable_failure().into_pyerr());
                }
                rows.push(LiveSmartsReceiptRow::Available(LiveSmartsRow {
                    target_index,
                    positions,
                }));
            }
            molecules.push(Py::new(
                py,
                PyLiveDocumentSmartsMoleculeSummary {
                    match_count: count,
                    completeness: if result.truncated() {
                        "truncated"
                    } else {
                        "complete"
                    }
                    .to_owned(),
                },
            )?);
        }
        let key =
            LiveSmartsReceiptKey(random_bytes().map_err(|_| unavailable_failure().into_pyerr())?);
        self.receipts.insert(
            key.clone(),
            LiveSmartsReceiptState {
                generation: plan.generation,
                revision: plan.revision,
                digest: plan.digest,
                rows,
            },
        );
        let issuer = self
            .issuer
            .clone()
            .ok_or_else(|| unavailable_failure().into_pyerr())?;
        let receipt = Py::new(py, PyLiveDocumentSmartsReceipt { issuer, key })?;
        Py::new(
            py,
            PyLiveDocumentSmartsRunSummary {
                receipt,
                traversal: if incomplete {
                    "total_match_budget_reached"
                } else {
                    "complete"
                }
                .to_owned(),
                molecules,
            },
        )
    }

    pub(crate) fn run_selected(
        &mut self,
        py: Python<'_>,
        session: &RenderInteractionSessionV1,
        engine: &dyn ChemEngine,
        selection: &PyLiveDocumentSmartsSelectedQuery,
        per: u32,
        total: u32,
    ) -> PyResult<Py<PyLiveDocumentSmartsRunSummary>> {
        let target_index = self.prepare_selected_request(session, selection, per, total)?;
        let target = self
            .validate_plan_current(session)?
            .snapshot
            .targets()
            .get(target_index)
            .ok_or_else(|| unavailable_failure().into_pyerr())?;
        let query = engine
            .molecule_to_smarts(target.graph())
            .map_err(|_| unavailable_failure().into_pyerr())?;
        self.run(py, session, engine, query, per, total)
    }

    pub(crate) fn capture_selected_query(
        &self,
        py: Python<'_>,
        session: &RenderInteractionSessionV1,
        selection: &super::direct_root_interaction_binding::PySelection,
    ) -> PyResult<Py<PyLiveDocumentSmartsSelectedQuery>> {
        self.selected_target_index(
            session,
            super::direct_root_interaction_binding::selection_value_v1(selection),
        )
        .map_err(LiveFailure::into_pyerr)?;
        let issuer = self
            .issuer
            .clone()
            .ok_or_else(|| unavailable_failure().into_pyerr())?;
        Py::new(
            py,
            PyLiveDocumentSmartsSelectedQuery {
                issuer,
                selection: super::direct_root_interaction_binding::selection_value_v1(selection)
                    .clone(),
            },
        )
    }

    /// Validate every refusal that does not require chemistry before the
    /// installed native runtime is acquired.
    pub(crate) fn validate_raw_request(
        &self,
        session: &RenderInteractionSessionV1,
        query: &str,
        per: u32,
        total: u32,
    ) -> PyResult<()> {
        self.validate_caps(per, total)?;
        self.validate_plan_current(session)?;
        valid_query(query.to_owned()).map(|_| ())
    }

    pub(crate) fn prepare_selected_request(
        &self,
        session: &RenderInteractionSessionV1,
        selection: &PyLiveDocumentSmartsSelectedQuery,
        per: u32,
        total: u32,
    ) -> PyResult<usize> {
        self.selected_admission(session, selection, per, total)
            .map_err(LiveFailure::into_pyerr)
    }

    /// Report the exact selected-token admission state without consuming any
    /// query or receipt capability.
    pub(crate) fn selected_readiness(
        &self,
        session: &RenderInteractionSessionV1,
        selection: &PyLiveDocumentSmartsSelectedQuery,
    ) -> PyLiveDocumentSmartsSelectedReadiness {
        match self.selected_admission(session, selection, 1, 1) {
            Ok(_) => PyLiveDocumentSmartsSelectedReadiness {
                available: true,
                category: None,
                reason: None,
                recovery: None,
            },
            Err(failure) => {
                let (category, reason, recovery) = failure.facts();
                PyLiveDocumentSmartsSelectedReadiness {
                    available: false,
                    category: Some(category),
                    reason: Some(reason),
                    recovery: Some(recovery),
                }
            }
        }
    }

    /// Canonical selected-token admission shared by readiness and execution.
    fn selected_admission(
        &self,
        session: &RenderInteractionSessionV1,
        selection: &PyLiveDocumentSmartsSelectedQuery,
        per: u32,
        total: u32,
    ) -> Result<usize, LiveFailure> {
        self.validate_caps_failure(per, total)?;
        if self.issuer.as_ref() != Some(&selection.issuer) {
            return Err(LiveFailure::Refused(
                PyLiveDocumentSmartsReasonV1::ForeignSelection,
            ));
        }
        self.selected_target_index(session, &selection.selection)
    }

    fn selected_target_index(
        &self,
        session: &RenderInteractionSessionV1,
        selection: &ferrum_document_render::RenderInteractionSelectionV1,
    ) -> Result<usize, LiveFailure> {
        let selected = super::direct_root_interaction_binding::selected_direct_root_from_value_v1(
            session, selection,
        )
        .map_err(map_selection_failure)?;
        let identifier = match selected {
            super::direct_root_interaction_binding::SelectedDirectRootV1::Empty => {
                return Err(LiveFailure::Refused(
                    PyLiveDocumentSmartsReasonV1::SelectedRootEmpty,
                ));
            }
            super::direct_root_interaction_binding::SelectedDirectRootV1::Multiple => {
                return Err(LiveFailure::Refused(
                    PyLiveDocumentSmartsReasonV1::SelectedRootMultiple,
                ));
            }
            super::direct_root_interaction_binding::SelectedDirectRootV1::One(value) => value,
        };
        let plan = self.validate_plan_current_failure(session)?;
        let render_target = RenderTarget::document_object(identifier.clone());
        let target = plan
            .snapshot
            .selected_target_by_render_target(&render_target)
            .ok_or(LiveFailure::UnsupportedDocument(
                PyLiveDocumentSmartsReasonV1::SelectedTargetNotMolecule,
            ))?;
        plan.snapshot
            .targets()
            .iter()
            .position(|candidate| std::ptr::eq(candidate, target))
            .ok_or_else(unavailable_failure)
    }

    fn validate_caps(&self, per: u32, total: u32) -> PyResult<()> {
        self.validate_caps_failure(per, total)
            .map_err(LiveFailure::into_pyerr)
    }

    fn validate_caps_failure(&self, per: u32, total: u32) -> Result<(), LiveFailure> {
        if per == 0 || per > MAX_PER_MOLECULE || total == 0 || total > MAX_TOTAL || per > total {
            return Err(LiveFailure::ResourceLimit(
                PyLiveDocumentSmartsReasonV1::MatchCapsInconsistent,
            ));
        }
        Ok(())
    }

    fn validate_plan_current(
        &self,
        session: &RenderInteractionSessionV1,
    ) -> PyResult<&LiveSmartsPlan> {
        self.validate_plan_current_failure(session)
            .map_err(LiveFailure::into_pyerr)
    }

    fn validate_plan_current_failure(
        &self,
        session: &RenderInteractionSessionV1,
    ) -> Result<&LiveSmartsPlan, LiveFailure> {
        let current = session
            .snapshot()
            .map_err(|_| LiveFailure::Stale(PyLiveDocumentSmartsReasonV1::StaleDocument))?;
        match &self.readiness {
            LiveSmartsReadiness::Unpublished => Err(LiveFailure::Unavailable(
                PyLiveDocumentSmartsReasonV1::PlanNotPublished,
            )),
            LiveSmartsReadiness::UnsupportedDocument { revision, digest } => {
                if current.revision() != *revision || current.digest() != digest {
                    return Err(LiveFailure::Stale(
                        PyLiveDocumentSmartsReasonV1::StaleDocument,
                    ));
                }
                Err(LiveFailure::UnsupportedDocument(
                    PyLiveDocumentSmartsReasonV1::UnsupportedDocument,
                ))
            }
            LiveSmartsReadiness::Ready(plan) => {
                if current.revision() != plan.revision || current.digest() != &plan.digest {
                    return Err(LiveFailure::Stale(
                        PyLiveDocumentSmartsReasonV1::StaleDocument,
                    ));
                }
                Ok(plan)
            }
        }
    }

    pub(crate) fn show(
        &mut self,
        py: Python<'_>,
        session: &RenderInteractionSessionV1,
        receipt: PyRef<'_, PyLiveDocumentSmartsReceipt>,
        row_index: usize,
    ) -> PyResult<Py<PyLiveDocumentSmartsPaint>> {
        if self.issuer.as_ref() != Some(&receipt.issuer) {
            return Err(receipt_failure().into_pyerr());
        }
        if !self.receipts.contains_key(&receipt.key) {
            return Err(receipt_failure().into_pyerr());
        }

        // Reserve the issued row before checking the document fence or
        // constructing paint. This exclusively borrowed bridge makes that
        // transition atomic, and every later refusal permanently consumes the
        // opaque capability that reached this point.
        let (generation, revision, digest, row) = {
            let state = self
                .receipts
                .get_mut(&receipt.key)
                .ok_or_else(|| receipt_failure().into_pyerr())?;
            let receipt_row = state
                .rows
                .get_mut(row_index)
                .ok_or_else(|| receipt_failure().into_pyerr())?;
            let LiveSmartsReceiptRow::Available(available_row) = receipt_row else {
                return Err(receipt_failure().into_pyerr());
            };
            let row = available_row.clone();
            *receipt_row = LiveSmartsReceiptRow::Reserved;
            (state.generation, state.revision, state.digest, row)
        };
        let LiveSmartsReadiness::Ready(plan) = &self.readiness else {
            return Err(stale_failure().into_pyerr());
        };
        let current = session
            .snapshot()
            .map_err(|_| stale_failure().into_pyerr())?;
        if generation != plan.generation
            || revision != plan.revision
            || digest != plan.digest
            || revision != current.revision()
            || digest != *current.digest()
        {
            return Err(stale_failure().into_pyerr());
        }
        let points = plan
            .atom_points_by_graph_position
            .get(row.target_index)
            .ok_or_else(|| paint_failure().into_pyerr())?;
        let mut atom_bounds = Vec::new();
        for position in row.positions {
            let (x, y) = points
                .get(position)
                .copied()
                .ok_or_else(|| paint_failure().into_pyerr())?;
            let bounds = (x - 8.0, y - 8.0, x + 8.0, y + 8.0);
            if !bounds.0.is_finite()
                || !bounds.1.is_finite()
                || !bounds.2.is_finite()
                || !bounds.3.is_finite()
            {
                return Err(paint_failure().into_pyerr());
            }
            atom_bounds.push(bounds);
        }
        let paint = Py::new(py, PyLiveDocumentSmartsPaint { atom_bounds })
            .map_err(|_| paint_failure().into_pyerr())?;
        Ok(paint)
    }
}

fn valid_query(value: String) -> PyResult<String> {
    if value.is_empty() {
        Err(LiveFailure::InvalidQuery(PyLiveDocumentSmartsReasonV1::EmptyQuery).into_pyerr())
    } else if value.len() > MAX_QUERY_BYTES || value.contains('\0') {
        Err(LiveFailure::InvalidQuery(PyLiveDocumentSmartsReasonV1::QueryTooLong).into_pyerr())
    } else {
        Ok(value)
    }
}
fn random_bytes() -> Result<[u8; 32], getrandom::Error> {
    let mut value = [0; 32];
    fill(&mut value)?;
    Ok(value)
}
fn unavailable_failure() -> LiveFailure {
    LiveFailure::Unavailable(PyLiveDocumentSmartsReasonV1::MatchUnavailable)
}

fn receipt_failure() -> LiveFailure {
    LiveFailure::Unavailable(PyLiveDocumentSmartsReasonV1::ReceiptUnavailable)
}

fn stale_failure() -> LiveFailure {
    LiveFailure::Stale(PyLiveDocumentSmartsReasonV1::StaleDocument)
}

fn paint_failure() -> LiveFailure {
    LiveFailure::Unavailable(PyLiveDocumentSmartsReasonV1::PaintUnavailable)
}

fn map_chemistry_error(error: ChemistryError) -> LiveFailure {
    match error {
        ChemistryError::SmartsMatchUnavailable {
            reason: SmartsMatchUnavailableReason::NativeRejected,
        } => LiveFailure::InvalidQuery(PyLiveDocumentSmartsReasonV1::InvalidQuery),
        ChemistryError::SmartsMatchUnavailable { .. } => unavailable_failure(),
        _ => unavailable_failure(),
    }
}

fn map_selection_failure(error: ferrum_document_render::RenderInteractionErrorV1) -> LiveFailure {
    use ferrum_document_render::RenderInteractionErrorV1;

    match error {
        RenderInteractionErrorV1::ForeignSession => {
            LiveFailure::Refused(PyLiveDocumentSmartsReasonV1::ForeignSelection)
        }
        RenderInteractionErrorV1::StaleRevision
        | RenderInteractionErrorV1::StaleDigest
        | RenderInteractionErrorV1::SelectionChanged => {
            LiveFailure::Stale(PyLiveDocumentSmartsReasonV1::StaleSelection)
        }
        _ => LiveFailure::UnsupportedDocument(PyLiveDocumentSmartsReasonV1::UnsupportedDocument),
    }
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "LiveDocumentSmartsError",
        module.py().get_type::<LiveDocumentSmartsError>(),
    )?;
    module.add_class::<PyLiveDocumentSmartsCategoryV1>()?;
    module.add_class::<PyLiveDocumentSmartsReasonV1>()?;
    module.add_class::<PyLiveDocumentSmartsRecoveryV1>()?;
    Ok(())
}

#[cfg(test)]
#[path = "live_document_smarts_query_v1/tests.rs"]
mod tests;
