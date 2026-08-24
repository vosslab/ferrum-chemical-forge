//! Private Rust-owned Direct-Glycosidic Haworth delivery seam.

use ferrum_chemistry::{ChemistryError, NativeChemEngine};
use ferrum_document::{DocumentSessionError, PendingDirectHaworthV1, Point3V1};
use ferrum_domain::haworth::DirectGlycosidicHaworthAuthoringReceiptV1;
use ferrum_domain::haworth::{
    DirectHaworthFromSmilesBuildErrorV1, build_direct_haworth_from_smiles_v1,
};
use ferrum_geometry::{MoleculePlacementV1, Point2 as MoleculePlacementPointV1};
use ferrum_render::{
    DocumentRenderContentV1, DocumentRenderIdentityV1, preview_root_render_overlay_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::{
    binding::{PyDocumentSession, PySessionOperationResultV1},
    render_binding::{PyRenderPlanV2, plan_from},
};

create_exception!(ferrum_chem, DirectHaworthError, super::binding::FerrumError);
create_exception!(ferrum_chem, DirectHaworthInputError, DirectHaworthError);
create_exception!(ferrum_chem, DirectHaworthProfileError, DirectHaworthError);
create_exception!(ferrum_chem, DirectHaworthResourceError, DirectHaworthError);
create_exception!(ferrum_chem, DirectHaworthReceiptError, DirectHaworthError);

const OPERATION: &str = "prepare_direct_haworth_from_smiles_v1";
const NATIVE_HAWORTH_BOND_LENGTH_PT: f64 = 40.0;
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
    #[pyo3(get)]
    preview_plan: PyRenderPlanV2,
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
        let pending = self
            .session
            .prepare_create_direct_haworth_v1(expected_revision, &receipt, anchor)
            .map_err(|error| map_session_error(py, error))?;
        let preview_plan = preview_plan(py, &pending)?;
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
            preview_plan,
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
        DocumentSessionError::RendererAdmission => receipt_error(
            py,
            "the direct Haworth candidate cannot be rendered completely",
        ),
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

fn preview_plan(py: Python<'_>, pending: &PendingDirectHaworthV1) -> PyResult<PyRenderPlanV2> {
    let identity = DocumentRenderIdentityV1::durable(pending.molecule_identifier().as_str())
        .map_err(|_| receipt_error(py, "renderer plan did not preserve the pending molecule"))?;
    let overlay = preview_root_render_overlay_v1(pending.render_plan_v1(), &identity)
        .map_err(|_| receipt_error(py, "renderer plan did not preserve the pending molecule"))?;
    let DocumentRenderContentV1::Molecule(plan) = overlay.content() else {
        return Err(receipt_error(
            py,
            "renderer plan did not preserve the pending molecule",
        ));
    };
    plan_from(py, plan)
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
