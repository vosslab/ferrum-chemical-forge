//! Private published-plan SMARTS attachment for one Python document session.
//!
//! A receipt is only a session-bound opaque key. Durable record identities,
//! graph positions, and renderer correspondence remain private to this module.

use std::collections::{HashMap, HashSet};

use ferrum_chemistry::{
    ChemEngine, ChemistryError, SmartsMatchOptions, SmartsMatchUnavailableReason,
};
use ferrum_render::{
    BatchSpace, RenderObservationV1, document_observation_from_accepted_operation_v1,
};
use getrandom::fill;
use pyo3::{create_exception, prelude::*};

use crate::{
    RenderInteractionSessionV1,
    protocol::document_smarts_snapshot_v1::OwnedDocumentSmartsSnapshotV1,
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
    SelectedSourceNotMolecule,
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
pub(crate) enum LiveFailureV1 {
    InvalidQuery(PyLiveDocumentSmartsReasonV1),
    UnsupportedDocument(PyLiveDocumentSmartsReasonV1),
    ResourceLimit(PyLiveDocumentSmartsReasonV1),
    Stale(PyLiveDocumentSmartsReasonV1),
    Unavailable(PyLiveDocumentSmartsReasonV1),
    Refused(PyLiveDocumentSmartsReasonV1),
}

impl LiveFailureV1 {
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
struct LiveSmartsIssuerV1([u8; 32]);
#[derive(Eq, Hash, PartialEq, Clone)]
struct LiveSmartsReceiptKeyV1([u8; 32]);

struct LiveSmartsPlanV1 {
    generation: u64,
    revision: u64,
    digest: [u8; 32],
    snapshot: OwnedDocumentSmartsSnapshotV1,
    atom_points_by_graph_position: Vec<Vec<(f64, f64)>>,
}

#[derive(Clone)]
struct LiveSmartsRowV1 {
    target_index: usize,
    positions: Vec<usize>,
}
enum LiveSmartsReceiptRowV1 {
    Available(LiveSmartsRowV1),
    Reserved,
}
struct LiveSmartsReceiptStateV1 {
    generation: u64,
    revision: u64,
    digest: [u8; 32],
    rows: Vec<LiveSmartsReceiptRowV1>,
}

/// Opaque, nonconstructible session receipt. It has no key, issuer, integer,
/// string, serialization, or debug surface.
#[pyclass(
    frozen,
    unsendable,
    module = "ferrum_chem",
    name = "_LiveDocumentSmartsReceiptV1",
    skip_from_py_object
)]
pub(crate) struct PyLiveDocumentSmartsReceiptV1 {
    issuer: LiveSmartsIssuerV1,
    key: LiveSmartsReceiptKeyV1,
}

/// Opaque selected-query capability minted by one document session. Generic
/// render selections intentionally remain inspectable for authoring tools;
/// this separate capability prevents that display surface from becoming the
/// SMARTS query authority.
#[pyclass(
    frozen,
    unsendable,
    module = "ferrum_chem",
    name = "_LiveDocumentSmartsSelectedQueryV1",
    skip_from_py_object
)]
pub(crate) struct PyLiveDocumentSmartsSelectedQueryV1 {
    issuer: LiveSmartsIssuerV1,
    selection: ferrum_document_render::RenderInteractionSelectionV1,
}

#[pyclass(
    frozen,
    name = "_LiveDocumentSmartsMoleculeSummaryV1",
    skip_from_py_object
)]
pub(crate) struct PyLiveDocumentSmartsMoleculeSummaryV1 {
    #[pyo3(get)]
    source_order: u32,
    #[pyo3(get)]
    match_count: u32,
    #[pyo3(get)]
    completeness: String,
}
#[pyclass(frozen, name = "_LiveDocumentSmartsRunSummaryV1", skip_from_py_object)]
pub(crate) struct PyLiveDocumentSmartsRunSummaryV1 {
    #[pyo3(get)]
    receipt: Py<PyLiveDocumentSmartsReceiptV1>,
    #[pyo3(get)]
    traversal: String,
    #[pyo3(get)]
    molecules: Vec<Py<PyLiveDocumentSmartsMoleculeSummaryV1>>,
}
#[pyclass(frozen, name = "_LiveDocumentSmartsPaintV1", skip_from_py_object)]
pub(crate) struct PyLiveDocumentSmartsPaintV1 {
    #[pyo3(get)]
    atom_bounds: Vec<(f64, f64, f64, f64)>,
}

pub(crate) struct LiveDocumentSmartsBridgeV1 {
    issuer: LiveSmartsIssuerV1,
    next_generation: u64,
    plan: Option<LiveSmartsPlanV1>,
    receipts: HashMap<LiveSmartsReceiptKeyV1, LiveSmartsReceiptStateV1>,
}

impl LiveDocumentSmartsBridgeV1 {
    pub(crate) fn new() -> Self {
        Self {
            issuer: LiveSmartsIssuerV1(random_bytes().unwrap_or([0; 32])),
            next_generation: 0,
            plan: None,
            receipts: HashMap::new(),
        }
    }
    pub(crate) fn retire(&mut self) {
        self.retire_receipts();
        self.plan = None;
    }

    /// Revoke every derived display capability while retaining the immutable
    /// renderer plan that authoritatively produced them. Query-level UI
    /// cleanup uses this narrower boundary; document lifecycle transitions
    /// retain `retire` because they also invalidate the plan itself.
    pub(crate) fn retire_receipts(&mut self) {
        self.receipts.clear();
    }

    /// Create the only plan from one accepted observation. The generation is
    /// advanced and old receipts retired before the replacement is attempted.
    pub(crate) fn publish(
        &mut self,
        session: &RenderInteractionSessionV1,
        expected_revision: u64,
    ) -> PyResult<RenderObservationV1> {
        self.retire();
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .ok_or_else(|| unavailable_failure().into_pyerr())?;
        let observation = session
            .observe(expected_revision)
            .map_err(|_| unavailable_failure().into_pyerr())?;
        let snapshot = OwnedDocumentSmartsSnapshotV1::from_accepted_observation_v1(&observation)
            .map_err(|_| unavailable_failure().into_pyerr())?;
        let rendered = document_observation_from_accepted_operation_v1(&observation)
            .map_err(|_| unavailable_failure().into_pyerr())?;
        let mut expected = HashSet::new();
        for target in snapshot.targets() {
            for record in target.graph_position_to_record_id() {
                if !expected.insert(record.clone()) {
                    return Err(unavailable_failure().into_pyerr());
                }
            }
        }
        let mut anchors = HashMap::new();
        for plan in rendered.molecule_plans() {
            for batch in plan.batches() {
                let BatchSpace::AtomLocal { anchor } = batch.coordinate_space() else {
                    continue;
                };
                let record = batch.target().record_id();
                if !expected.contains(record)
                    || anchors
                        .insert(record.clone(), (anchor.x(), anchor.y()))
                        .is_some()
                {
                    return Err(unavailable_failure().into_pyerr());
                }
            }
        }
        if anchors.len() != expected.len() {
            return Err(unavailable_failure().into_pyerr());
        }
        let mut atom_points_by_graph_position = Vec::new();
        for target in snapshot.targets() {
            let mut points = Vec::new();
            for record in target.graph_position_to_record_id() {
                let point = anchors
                    .get(record)
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
        self.plan = Some(LiveSmartsPlanV1 {
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
    ) -> PyResult<Py<PyLiveDocumentSmartsRunSummaryV1>> {
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
                rows.push(LiveSmartsReceiptRowV1::Available(LiveSmartsRowV1 {
                    target_index,
                    positions,
                }));
            }
            molecules.push(Py::new(
                py,
                PyLiveDocumentSmartsMoleculeSummaryV1 {
                    source_order: target.source_order(),
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
            LiveSmartsReceiptKeyV1(random_bytes().map_err(|_| unavailable_failure().into_pyerr())?);
        self.receipts.insert(
            key.clone(),
            LiveSmartsReceiptStateV1 {
                generation: plan.generation,
                revision: plan.revision,
                digest: plan.digest,
                rows,
            },
        );
        let receipt = Py::new(
            py,
            PyLiveDocumentSmartsReceiptV1 {
                issuer: self.issuer.clone(),
                key,
            },
        )?;
        Py::new(
            py,
            PyLiveDocumentSmartsRunSummaryV1 {
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
        selection: &PyLiveDocumentSmartsSelectedQueryV1,
        per: u32,
        total: u32,
    ) -> PyResult<Py<PyLiveDocumentSmartsRunSummaryV1>> {
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
    ) -> PyResult<Py<PyLiveDocumentSmartsSelectedQueryV1>> {
        super::direct_root_interaction_binding::selected_direct_root_v1(session, selection)
            .map_err(map_selection_error)?;
        Py::new(
            py,
            PyLiveDocumentSmartsSelectedQueryV1 {
                issuer: self.issuer.clone(),
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
        selection: &PyLiveDocumentSmartsSelectedQueryV1,
        per: u32,
        total: u32,
    ) -> PyResult<usize> {
        self.validate_caps(per, total)?;
        if selection.issuer != self.issuer {
            return Err(
                LiveFailureV1::Refused(PyLiveDocumentSmartsReasonV1::ForeignSelection).into_pyerr(),
            );
        }
        let selected = super::direct_root_interaction_binding::selected_direct_root_from_value_v1(
            session,
            &selection.selection,
        )
        .map_err(map_selection_error)?;
        let identifier = match selected {
            super::direct_root_interaction_binding::SelectedDirectRootV1::Empty => {
                return Err(LiveFailureV1::Refused(
                    PyLiveDocumentSmartsReasonV1::SelectedRootEmpty,
                )
                .into_pyerr());
            }
            super::direct_root_interaction_binding::SelectedDirectRootV1::Multiple => {
                return Err(LiveFailureV1::Refused(
                    PyLiveDocumentSmartsReasonV1::SelectedRootMultiple,
                )
                .into_pyerr());
            }
            super::direct_root_interaction_binding::SelectedDirectRootV1::One(value) => value,
        };
        let plan = self.validate_plan_current(session)?;
        let target = plan
            .snapshot
            .selected_target_by_renderer_source_id(identifier)
            .ok_or_else(|| {
                LiveFailureV1::UnsupportedDocument(
                    PyLiveDocumentSmartsReasonV1::SelectedSourceNotMolecule,
                )
                .into_pyerr()
            })?;
        plan.snapshot
            .targets()
            .iter()
            .position(|candidate| std::ptr::eq(candidate, target))
            .ok_or_else(|| unavailable_failure().into_pyerr())
    }

    fn validate_caps(&self, per: u32, total: u32) -> PyResult<()> {
        if per == 0 || per > MAX_PER_MOLECULE || total == 0 || total > MAX_TOTAL || per > total {
            return Err(LiveFailureV1::ResourceLimit(
                PyLiveDocumentSmartsReasonV1::MatchCapsInconsistent,
            )
            .into_pyerr());
        }
        Ok(())
    }

    fn validate_plan_current(
        &self,
        session: &RenderInteractionSessionV1,
    ) -> PyResult<&LiveSmartsPlanV1> {
        let plan = self.plan.as_ref().ok_or_else(|| {
            LiveFailureV1::Unavailable(PyLiveDocumentSmartsReasonV1::PlanNotPublished).into_pyerr()
        })?;
        let current = session.snapshot().map_err(|_| {
            LiveFailureV1::Stale(PyLiveDocumentSmartsReasonV1::StaleDocument).into_pyerr()
        })?;
        if current.revision() != plan.revision || current.digest() != &plan.digest {
            return Err(
                LiveFailureV1::Stale(PyLiveDocumentSmartsReasonV1::StaleDocument).into_pyerr(),
            );
        }
        Ok(plan)
    }

    pub(crate) fn show(
        &mut self,
        py: Python<'_>,
        session: &RenderInteractionSessionV1,
        receipt: PyRef<'_, PyLiveDocumentSmartsReceiptV1>,
        row_index: usize,
    ) -> PyResult<Py<PyLiveDocumentSmartsPaintV1>> {
        if receipt.issuer != self.issuer {
            return Err(receipt_failure().into_pyerr());
        }

        // A failed redemption must leave its row usable. In particular, do
        // not reserve a row until the issuer, plan/session fence, row data,
        // and Python-owned paint have all been validated and constructed.
        // This bridge is exclusively borrowed for the entire call, so the
        // final reservation is atomic with respect to those checks.
        let plan = self
            .plan
            .as_ref()
            .ok_or_else(|| stale_failure().into_pyerr())?;
        let current = session
            .snapshot()
            .map_err(|_| stale_failure().into_pyerr())?;
        let row = {
            let state = self
                .receipts
                .get(&receipt.key)
                .ok_or_else(|| receipt_failure().into_pyerr())?;
            if state.generation != plan.generation
                || state.revision != plan.revision
                || state.digest != plan.digest
                || state.revision != current.revision()
                || state.digest != *current.digest()
            {
                return Err(stale_failure().into_pyerr());
            }
            let LiveSmartsReceiptRowV1::Available(row) = state
                .rows
                .get(row_index)
                .ok_or_else(|| receipt_failure().into_pyerr())?
            else {
                return Err(receipt_failure().into_pyerr());
            };
            row.clone()
        };
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
        let paint = Py::new(py, PyLiveDocumentSmartsPaintV1 { atom_bounds })
            .map_err(|_| paint_failure().into_pyerr())?;
        let state = self
            .receipts
            .get_mut(&receipt.key)
            .ok_or_else(|| receipt_failure().into_pyerr())?;
        let row = state
            .rows
            .get_mut(row_index)
            .ok_or_else(|| receipt_failure().into_pyerr())?;
        if !matches!(row, LiveSmartsReceiptRowV1::Available(_)) {
            return Err(receipt_failure().into_pyerr());
        }
        *row = LiveSmartsReceiptRowV1::Reserved;
        Ok(paint)
    }
}

fn valid_query(value: String) -> PyResult<String> {
    if value.is_empty() {
        Err(LiveFailureV1::InvalidQuery(PyLiveDocumentSmartsReasonV1::EmptyQuery).into_pyerr())
    } else if value.len() > MAX_QUERY_BYTES || value.contains('\0') {
        Err(LiveFailureV1::InvalidQuery(PyLiveDocumentSmartsReasonV1::QueryTooLong).into_pyerr())
    } else {
        Ok(value)
    }
}
fn random_bytes() -> Result<[u8; 32], getrandom::Error> {
    let mut value = [0; 32];
    fill(&mut value)?;
    Ok(value)
}
fn unavailable_failure() -> LiveFailureV1 {
    LiveFailureV1::Unavailable(PyLiveDocumentSmartsReasonV1::MatchUnavailable)
}

fn receipt_failure() -> LiveFailureV1 {
    LiveFailureV1::Unavailable(PyLiveDocumentSmartsReasonV1::ReceiptUnavailable)
}

fn stale_failure() -> LiveFailureV1 {
    LiveFailureV1::Stale(PyLiveDocumentSmartsReasonV1::StaleDocument)
}

fn paint_failure() -> LiveFailureV1 {
    LiveFailureV1::Unavailable(PyLiveDocumentSmartsReasonV1::PaintUnavailable)
}

fn map_chemistry_error(error: ChemistryError) -> LiveFailureV1 {
    match error {
        ChemistryError::SmartsMatchUnavailable {
            reason: SmartsMatchUnavailableReason::NativeRejected,
        } => LiveFailureV1::InvalidQuery(PyLiveDocumentSmartsReasonV1::InvalidQuery),
        ChemistryError::SmartsMatchUnavailable { .. } => unavailable_failure(),
        _ => unavailable_failure(),
    }
}

fn map_selection_error(error: ferrum_document_render::RenderInteractionErrorV1) -> PyErr {
    use ferrum_document_render::RenderInteractionErrorV1;

    let failure = match error {
        RenderInteractionErrorV1::ForeignSession => {
            LiveFailureV1::Refused(PyLiveDocumentSmartsReasonV1::ForeignSelection)
        }
        RenderInteractionErrorV1::StaleRevision
        | RenderInteractionErrorV1::StaleDigest
        | RenderInteractionErrorV1::SelectionChanged => {
            LiveFailureV1::Stale(PyLiveDocumentSmartsReasonV1::StaleSelection)
        }
        _ => LiveFailureV1::UnsupportedDocument(PyLiveDocumentSmartsReasonV1::UnsupportedDocument),
    };
    failure.into_pyerr()
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "LiveDocumentSmartsError",
        module.py().get_type::<LiveDocumentSmartsError>(),
    )?;
    module.add_class::<PyLiveDocumentSmartsCategoryV1>()?;
    module.add_class::<PyLiveDocumentSmartsReasonV1>()?;
    module.add_class::<PyLiveDocumentSmartsRecoveryV1>()?;
    module.add_class::<PyLiveDocumentSmartsReceiptV1>()?;
    module.add_class::<PyLiveDocumentSmartsSelectedQueryV1>()?;
    module.add_class::<PyLiveDocumentSmartsMoleculeSummaryV1>()?;
    module.add_class::<PyLiveDocumentSmartsRunSummaryV1>()?;
    module.add_class::<PyLiveDocumentSmartsPaintV1>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        LiveDocumentSmartsBridgeV1, LiveFailureV1, PyLiveDocumentSmartsCategoryV1,
        PyLiveDocumentSmartsReasonV1, PyLiveDocumentSmartsSelectedQueryV1,
    };
    use crate::{
        RenderInteractionModifierV1, RenderInteractionQueryV1, RenderInteractionSessionV1,
    };
    use ferrum_chemistry::{
        ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, SmartsMatchOptions,
        SmartsMatchResult, SmilesMolecule,
    };
    use ferrum_document::{DocumentFenceV1, DocumentSession};
    use pyo3::types::PyAnyMethods;

    const SOURCE: &str = concat!(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/>",
        "</atom></molecule></cdml>"
    );

    struct ReceiptLifecycleEngine;

    impl ChemEngine for ReceiptLifecycleEngine {
        fn smiles_to_molecule(&self, _: &str) -> Result<SmilesMolecule, ChemistryError> {
            unavailable("smiles_to_molecule")
        }

        fn generate_2d_coordinates(&self, _: &MolGraph) -> Result<Coordinates, ChemistryError> {
            unavailable("generate_2d_coordinates")
        }

        fn smarts_match(
            &self,
            _: &str,
            target: &MolGraph,
            options: SmartsMatchOptions,
        ) -> Result<SmartsMatchResult, ChemistryError> {
            SmartsMatchResult::try_from_rows(target, options, vec![vec![0]], false).map_err(|_| {
                ChemistryError::SmartsMatchUnavailable {
                    reason: ferrum_chemistry::SmartsMatchUnavailableReason::MalformedNativeResponse,
                }
            })
        }

        fn molecule_to_smarts(&self, _: &MolGraph) -> Result<String, ChemistryError> {
            Ok("C".to_owned())
        }

        fn kekulize(&self, _: &MolGraph, _: KekulizeOptions) -> Result<MolGraph, ChemistryError> {
            unavailable("kekulize")
        }
    }

    fn unavailable<T>(operation: &'static str) -> Result<T, ChemistryError> {
        Err(ChemistryError::OperationUnavailable { operation })
    }

    fn session() -> RenderInteractionSessionV1 {
        RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("fixture CDML loads"))
    }

    fn fence(session: &RenderInteractionSessionV1) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("fixture session snapshots");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    fn reason(py: pyo3::Python<'_>, error: pyo3::PyErr) -> PyLiveDocumentSmartsReasonV1 {
        *error
            .value(py)
            .getattr("reason")
            .expect("closed failure reason")
            .extract::<pyo3::PyRef<'_, PyLiveDocumentSmartsReasonV1>>()
            .expect("closed failure reason enum")
    }

    #[test]
    fn live_failure_facts_are_closed_and_redacted() {
        let (category, reason, _) =
            LiveFailureV1::InvalidQuery(PyLiveDocumentSmartsReasonV1::InvalidQuery).facts();
        assert_eq!(category, PyLiveDocumentSmartsCategoryV1::InvalidQuery);
        assert_eq!(reason, PyLiveDocumentSmartsReasonV1::InvalidQuery);
        let rendered = format!("{:?} {:?}", category, reason);
        for forbidden in ["FCQ1", "FQM1", "CDML", "molecule_id", "libferrum_chem"] {
            assert!(!rendered.contains(forbidden));
        }
    }

    #[test]
    fn live_failure_exposes_only_closed_python_recovery_facts() {
        pyo3::Python::initialize();
        pyo3::Python::attach(|py| {
            let error =
                LiveFailureV1::Stale(PyLiveDocumentSmartsReasonV1::StaleSelection).into_pyerr();
            let value = error.value(py);
            let category = value
                .getattr("category")
                .expect("closed category is attached")
                .extract::<pyo3::PyRef<'_, PyLiveDocumentSmartsCategoryV1>>()
                .expect("category remains in the closed vocabulary");
            let reason = value
                .getattr("reason")
                .expect("closed reason is attached")
                .extract::<pyo3::PyRef<'_, PyLiveDocumentSmartsReasonV1>>()
                .expect("reason remains in the closed vocabulary");
            let recovery = value
                .getattr("recovery")
                .expect("closed recovery is attached")
                .extract::<pyo3::PyRef<'_, super::PyLiveDocumentSmartsRecoveryV1>>()
                .expect("recovery remains in the closed vocabulary");
            assert_eq!(*category, PyLiveDocumentSmartsCategoryV1::Stale);
            assert_eq!(*reason, PyLiveDocumentSmartsReasonV1::StaleSelection);
            assert_eq!(
                *recovery,
                super::PyLiveDocumentSmartsRecoveryV1::RefreshAndRerun
            );
            let rendered = format!("{error}");
            for forbidden in ["FCQ1", "FQM1", "CDML", "molecule_id", "libferrum_chem"] {
                assert!(!rendered.contains(forbidden));
            }
        });
    }

    #[test]
    fn selected_query_token_has_no_python_data_surface() {
        pyo3::Python::initialize();
        pyo3::Python::attach(|py| {
            let module = pyo3::types::PyModule::new(py, "ferrum_chem").expect("module");
            super::initialize(&module).expect("private classes register");
            let class = module
                .getattr("_LiveDocumentSmartsSelectedQueryV1")
                .expect("selected capability class");
            assert!(class.call0().is_err());
            for forbidden in [
                "issuer",
                "selection",
                "roots",
                "identifier",
                "graph",
                "query",
            ] {
                assert!(class.getattr(forbidden).is_err());
            }
            let _ = std::any::TypeId::of::<PyLiveDocumentSmartsSelectedQueryV1>();
        });
    }

    #[test]
    fn receipt_only_retirement_retains_the_plan_for_raw_and_selected_reruns() {
        pyo3::Python::initialize();
        pyo3::Python::attach(|py| {
            let session = session();
            let expected = fence(&session);
            let mut bridge = LiveDocumentSmartsBridgeV1::new();
            bridge
                .publish(&session, expected.revision())
                .expect("published renderer plan");
            let engine = ReceiptLifecycleEngine;

            let raw = bridge
                .run(py, &session, &engine, "C".to_owned(), 1, 1)
                .expect("raw SMARTS run");
            let old_receipt = raw.bind(py).borrow().receipt.clone_ref(py);

            let observation = session
                .observe_render_interaction_v1(expected)
                .expect("renderer observation");
            let selection = session
                .select_render_interaction_roots_v1(
                    &observation,
                    None,
                    RenderInteractionQueryV1::Root {
                        identifier: "m".to_owned(),
                        modifier: RenderInteractionModifierV1::Replace,
                    },
                )
                .expect("one selected molecule");
            let selected = PyLiveDocumentSmartsSelectedQueryV1 {
                issuer: bridge.issuer.clone(),
                selection,
            };

            bridge.retire_receipts();
            let retired = bridge
                .show(py, &session, old_receipt.bind(py).borrow(), 0)
                .expect_err("receipt-only retirement revokes the old opaque receipt");
            assert_eq!(
                reason(py, retired),
                PyLiveDocumentSmartsReasonV1::ReceiptUnavailable
            );

            bridge
                .run(py, &session, &engine, "C".to_owned(), 1, 1)
                .expect("retained plan admits a raw rerun");
            bridge
                .run_selected(py, &session, &engine, &selected, 1, 1)
                .expect("retained plan admits a fresh opaque selected rerun");

            bridge.retire();
            let retired_plan = bridge
                .validate_raw_request(&session, "C", 1, 1)
                .expect_err("full retirement revokes the published plan");
            assert_eq!(
                reason(py, retired_plan),
                PyLiveDocumentSmartsReasonV1::PlanNotPublished
            );
        });
    }
}
