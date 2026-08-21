//! Private Rust-owned Direct-Glycosidic Haworth delivery seam.

use std::collections::BTreeMap;

use ferrum_chemistry::{ChemistryError, NativeChemEngine};
use ferrum_document::{DocumentSessionError, PendingDirectHaworthV1, Point3V1};
use ferrum_domain::haworth::{
    build_direct_haworth_from_smiles_v1, DirectHaworthFromSmilesBuildErrorV1,
};
use ferrum_domain::haworth::{
    DirectGlycosidicHaworthAuthoringReceiptV1, DirectGlycosidicHaworthBondStyleV1,
};
use ferrum_geometry::{MoleculePlacementV1, Point2 as MoleculePlacementPointV1};
use ferrum_render::{
    build_haworth_front_preview_ops, BondStyle, LineOp, Paint, PositiveFinite, RenderOp,
    RenderPoint, Rgb24,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::{
    binding::{PyDocumentSession, PySessionOperationResultV1},
    render_binding::{operation_from, PyRenderOperationV2},
};

create_exception!(ferrum_chem, DirectHaworthError, super::binding::FerrumError);
create_exception!(ferrum_chem, DirectHaworthInputError, DirectHaworthError);
create_exception!(ferrum_chem, DirectHaworthProfileError, DirectHaworthError);
create_exception!(ferrum_chem, DirectHaworthResourceError, DirectHaworthError);
create_exception!(ferrum_chem, DirectHaworthReceiptError, DirectHaworthError);

const OPERATION: &str = "prepare_direct_haworth_from_smiles_v1";
const NATIVE_HAWORTH_BOND_LENGTH_PT: f64 = 40.0;
const PREVIEW_LINE_WIDTH_PT: f64 = 1.0;
const PREVIEW_WEDGE_WIDTH_PT: f64 = 5.0;

/// A frozen source-owned V2 preview tier with no mutable geometry authority.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DirectHaworthPreviewBatchV2",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDirectHaworthPreviewBatchV2 {
    #[pyo3(get)]
    display_layer: String,
    operations: Vec<PyRenderOperationV2>,
}

#[pymethods]
impl PyDirectHaworthPreviewBatchV2 {
    #[getter]
    fn operations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        super::render_binding::frozen_tuple(py, &self.operations)
    }
}

/// Opaque parsed direct-Haworth source receipt, before any document mutation.
#[pyclass(
    frozen,
    unsendable,
    module = "ferrum_chem",
    name = "PreparedDirectHaworthFromSmilesV1"
)]
pub(crate) struct PyPreparedDirectHaworthFromSmilesV1 {
    receipt: DirectGlycosidicHaworthAuthoringReceiptV1,
    #[pyo3(get)]
    local_scale: f64,
}

/// Opaque anchor-bound, one-use document candidate.
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PreparedDirectHaworthInsertionV1"
)]
pub(crate) struct PyPreparedDirectHaworthInsertionV1 {
    pending: PendingDirectHaworthV1,
    #[pyo3(get)]
    source_revision: u64,
    #[pyo3(get)]
    source_digest: String,
    #[pyo3(get)]
    molecule_identifier: String,
    #[pyo3(get)]
    atom_identifiers: Vec<String>,
    #[pyo3(get)]
    bond_identifiers: Vec<String>,
    preview_batches: Vec<PyDirectHaworthPreviewBatchV2>,
}

#[pymethods]
impl PyPreparedDirectHaworthInsertionV1 {
    #[getter]
    fn preview_batches(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        super::render_binding::frozen_tuple(py, &self.preview_batches)
    }
}

/// Parse and classify one structural SMILES string without touching a document.
#[pyfunction]
pub(crate) fn prepare_direct_haworth_from_smiles_v1(
    py: Python<'_>,
    smiles: &str,
) -> PyResult<PyPreparedDirectHaworthFromSmilesV1> {
    let library_path = super::chemistry_binding::packaged_library_path(py, OPERATION)
        .map_err(|_| receipt_error(py, "native structural-SMILES preparation is unavailable"))?;
    let worker_path = library_path.clone();
    let smiles = smiles.to_owned();
    let result = py.detach(move || {
        let engine = NativeChemEngine::load(&worker_path).map_err(|_| PreparationFailure::Load)?;
        let anchor =
            MoleculePlacementPointV1::new(0.0, 0.0).map_err(|_| PreparationFailure::Resource)?;
        let placement = MoleculePlacementV1::new(NATIVE_HAWORTH_BOND_LENGTH_PT, anchor)
            .map_err(|_| PreparationFailure::Resource)?;
        build_direct_haworth_from_smiles_v1(&engine, &smiles, placement)
            .map_err(PreparationFailure::Build)
    });
    match result {
        Ok(prepared) => Ok(PyPreparedDirectHaworthFromSmilesV1 {
            local_scale: prepared.receipt().local_scale(),
            receipt: prepared.receipt().clone(),
        }),
        Err(error) => Err(map_preparation_error(py, error)),
    }
}

#[pymethods]
impl PyDocumentSession {
    fn prepare_create_direct_haworth_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        prepared: PyRef<'_, PyPreparedDirectHaworthFromSmilesV1>,
        anchor_x: f64,
        anchor_y: f64,
    ) -> PyResult<PyPreparedDirectHaworthInsertionV1> {
        let anchor = Point3V1::new(anchor_x, anchor_y, 0.0)
            .map_err(|_| receipt_error(py, "choose a finite empty page location"))?;
        let snapshot = self
            .session
            .snapshot()
            .map_err(|error| map_session_error(py, error))?;
        if snapshot.revision() != expected_revision {
            return Err(receipt_error(py, "the document changed before placement"));
        }
        let receipt = prepared.receipt.clone();
        let preview_batches = preview_batches(py, &receipt, anchor)?;
        let pending = self
            .session
            .prepare_create_direct_haworth_v1(expected_revision, &receipt, anchor)
            .map_err(|error| map_session_error(py, error))?;
        Ok(PyPreparedDirectHaworthInsertionV1 {
            source_revision: expected_revision,
            source_digest: hex_digest(snapshot.digest()),
            molecule_identifier: pending.molecule_identifier().as_str().to_owned(),
            atom_identifiers: pending
                .atom_identifiers()
                .iter()
                .map(|identifier| identifier.as_str().to_owned())
                .collect(),
            bond_identifiers: pending
                .bond_identifiers()
                .iter()
                .map(|identifier| identifier.as_str().to_owned())
                .collect(),
            preview_batches,
            pending,
        })
    }

    fn commit_create_direct_haworth_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyPreparedDirectHaworthInsertionV1>,
    ) -> PyResult<PySessionOperationResultV1> {
        self.session
            .commit_create_direct_haworth_v1(expected_revision, &mut prepared.pending)
            .map_err(|error| map_session_error(py, error))
            .map(|result| result.operation().clone().into())
    }
}

enum PreparationFailure {
    Load,
    Build(DirectHaworthFromSmilesBuildErrorV1),
    Resource,
}

fn map_preparation_error(py: Python<'_>, error: PreparationFailure) -> PyErr {
    match error {
        PreparationFailure::Load => {
            receipt_error(py, "native structural-SMILES preparation is unavailable")
        }
        PreparationFailure::Resource => resource_error(
            py,
            "direct Haworth preparation could not reserve private storage",
        ),
        PreparationFailure::Build(error) => match error {
            DirectHaworthFromSmilesBuildErrorV1::InvalidInput(_) => {
                input_error(py, "enter a structural SMILES")
            }
            DirectHaworthFromSmilesBuildErrorV1::SmilesSyntax { .. } => {
                input_error(py, "structural SMILES could not be parsed")
            }
            DirectHaworthFromSmilesBuildErrorV1::Chemistry(ChemistryError::ResourceExhausted {
                ..
            })
            | DirectHaworthFromSmilesBuildErrorV1::Resource { .. } => resource_error(
                py,
                "direct Haworth preparation could not reserve private storage",
            ),
            DirectHaworthFromSmilesBuildErrorV1::UnsupportedAtomFact { .. }
            | DirectHaworthFromSmilesBuildErrorV1::UnsupportedBondFact { .. }
            | DirectHaworthFromSmilesBuildErrorV1::Profile { .. }
            | DirectHaworthFromSmilesBuildErrorV1::Topology(_)
            | DirectHaworthFromSmilesBuildErrorV1::Authoring(_) => profile_error(
                py,
                "use a neutral, single-bond C/O two-ring structure with one exterior oxygen bridge",
            ),
            DirectHaworthFromSmilesBuildErrorV1::Chemistry(_) => {
                input_error(py, "structural SMILES could not be parsed")
            }
            DirectHaworthFromSmilesBuildErrorV1::Core(_)
            | DirectHaworthFromSmilesBuildErrorV1::Identifier(_) => receipt_error(
                py,
                "direct Haworth preparation could not create a checked receipt",
            ),
        },
    }
}

fn map_session_error(py: Python<'_>, error: DocumentSessionError) -> PyErr {
    match error {
        DocumentSessionError::RevisionConflict { .. } => {
            receipt_error(py, "the document changed before placement")
        }
        DocumentSessionError::PreparedOperationConsumed => {
            receipt_error(py, "this direct Haworth receipt was already used")
        }
        DocumentSessionError::PreparedOperationForeignSession => receipt_error(
            py,
            "this direct Haworth receipt belongs to another document",
        ),
        DocumentSessionError::Projection(_) => receipt_error(
            py,
            "the accepted document observation could not be installed",
        ),
        DocumentSessionError::Operation(_) => {
            receipt_error(py, "the direct Haworth candidate could not be accepted")
        }
        DocumentSessionError::RevisionExhausted => {
            resource_error(py, "the document cannot reserve another revision")
        }
        DocumentSessionError::Load(_)
        | DocumentSessionError::Serialize(_)
        | DocumentSessionError::EmptyMoleculeBatch
        | DocumentSessionError::DirectHaworthReobservation(_)
        | DocumentSessionError::HistoryUnavailable
        | DocumentSessionError::ClipboardPaste(_)
        | DocumentSessionError::ClipboardCut(_)
        | DocumentSessionError::UserTemplate(_)
        | DocumentSessionError::InvalidDestination { .. }
        | DocumentSessionError::PublishNotStarted { .. }
        | DocumentSessionError::PublishNotStartedWithCleanup { .. }
        | DocumentSessionError::ReplacementRejectedWithCleanup { .. }
        | DocumentSessionError::TemporaryName { .. }
        | DocumentSessionError::TemporaryNameExhausted { .. }
        | DocumentSessionError::PublishPossiblyCompleted { .. } => {
            receipt_error(py, "the direct Haworth document operation is unavailable")
        }
    }
}

fn preview_batches(
    py: Python<'_>,
    receipt: &DirectGlycosidicHaworthAuthoringReceiptV1,
    anchor: Point3V1,
) -> PyResult<Vec<PyDirectHaworthPreviewBatchV2>> {
    let mut atom_indexes = BTreeMap::new();
    for (index, atom) in receipt.atoms_in_canonical_order().iter().enumerate() {
        atom_indexes.insert(atom.source_atom_identity().clone(), index);
    }
    let width = PositiveFinite::new(PREVIEW_LINE_WIDTH_PT)
        .map_err(|_| receipt_error(py, "direct Haworth preview is unavailable"))?;
    let wedge_width = PositiveFinite::new(PREVIEW_WEDGE_WIDTH_PT)
        .map_err(|_| receipt_error(py, "direct Haworth preview is unavailable"))?;
    let paint = Paint::rgb24(
        Rgb24::new("000000")
            .map_err(|_| receipt_error(py, "direct Haworth preview is unavailable"))?,
    );
    receipt
        .bonds_in_canonical_order()
        .iter()
        .map(|bond| {
            let [start_id, end_id] = bond.endpoints();
            let start = *atom_indexes
                .get(start_id)
                .ok_or_else(|| receipt_error(py, "direct Haworth preview receipt is invalid"))?;
            let end = *atom_indexes
                .get(end_id)
                .ok_or_else(|| receipt_error(py, "direct Haworth preview receipt is invalid"))?;
            let start = receipt.atoms_in_canonical_order()[start].local();
            let end = receipt.atoms_in_canonical_order()[end].local();
            let start = RenderPoint::new(start.x + anchor.x(), start.y + anchor.y())
                .map_err(|_| receipt_error(py, "direct Haworth preview receipt is invalid"))?;
            let end = RenderPoint::new(end.x + anchor.x(), end.y + anchor.y())
                .map_err(|_| receipt_error(py, "direct Haworth preview receipt is invalid"))?;
            let (display_layer, operations) = match bond.token() {
                DirectGlycosidicHaworthBondStyleV1::N1 => (
                    "ordinary",
                    vec![RenderOp::Line(
                        LineOp::new(start, end, width, paint.clone(), 10).map_err(|_| {
                            receipt_error(py, "direct Haworth preview is unavailable")
                        })?,
                    )],
                ),
                DirectGlycosidicHaworthBondStyleV1::Q1 => (
                    "haworth_front_stroke",
                    build_haworth_front_preview_ops(
                        BondStyle::HaworthFrontStroke,
                        start,
                        end,
                        width,
                        wedge_width,
                        paint.clone(),
                    )
                    .map_err(|_| receipt_error(py, "direct Haworth preview is unavailable"))?,
                ),
                DirectGlycosidicHaworthBondStyleV1::W1 => (
                    "haworth_front_wedge",
                    build_haworth_front_preview_ops(
                        BondStyle::HaworthFrontWedge,
                        start,
                        end,
                        width,
                        wedge_width,
                        paint.clone(),
                    )
                    .map_err(|_| receipt_error(py, "direct Haworth preview is unavailable"))?,
                ),
            };
            Ok(PyDirectHaworthPreviewBatchV2 {
                display_layer: display_layer.to_owned(),
                operations: operations
                    .iter()
                    .map(|operation| operation_from(py, operation))
                    .collect::<PyResult<_>>()?,
            })
        })
        .collect()
}

fn input_error(py: Python<'_>, reason: &str) -> PyErr {
    typed_error(py, DirectHaworthInputError::new_err, reason)
}
fn profile_error(py: Python<'_>, reason: &str) -> PyErr {
    typed_error(py, DirectHaworthProfileError::new_err, reason)
}
fn resource_error(py: Python<'_>, reason: &str) -> PyErr {
    typed_error(py, DirectHaworthResourceError::new_err, reason)
}
fn receipt_error(py: Python<'_>, reason: &str) -> PyErr {
    typed_error(py, DirectHaworthReceiptError::new_err, reason)
}
fn typed_error(py: Python<'_>, constructor: impl FnOnce(String) -> PyErr, reason: &str) -> PyErr {
    let reason = reason.to_owned();
    let error = constructor(reason.clone());
    let _ = error.value(py).setattr("reason", reason);
    error
}

fn hex_digest(digest: &[u8; 32]) -> String {
    let mut result = String::with_capacity(64);
    for value in digest {
        use std::fmt::Write;
        let _ = write!(result, "{value:02x}");
    }
    result
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DirectHaworthError",
        module.py().get_type::<DirectHaworthError>(),
    )?;
    module.add(
        "DirectHaworthInputError",
        module.py().get_type::<DirectHaworthInputError>(),
    )?;
    module.add(
        "DirectHaworthProfileError",
        module.py().get_type::<DirectHaworthProfileError>(),
    )?;
    module.add(
        "DirectHaworthResourceError",
        module.py().get_type::<DirectHaworthResourceError>(),
    )?;
    module.add(
        "DirectHaworthReceiptError",
        module.py().get_type::<DirectHaworthReceiptError>(),
    )?;
    module.add_function(pyo3::wrap_pyfunction!(
        prepare_direct_haworth_from_smiles_v1,
        module
    )?)
}
