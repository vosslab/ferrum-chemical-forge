//! Python transport for the generic prepared session-transition lifecycle.
//!
//! A prepared transition is an opaque, one-use document receipt. Python may
//! copy its presentation before redemption, then redeem it only through the
//! owning document session's generic commit operation.

use super::binding::{
    OperationValidationError, PreparedOperationConsumedError, PreparedOperationForeignSessionError,
    PyDocumentSession, PySessionOperationResultV1, document_result,
};
use super::render_binding::{PyRenderOperationV2, PyRenderPointV1, operation_from};
use ferrum_document::{
    AdmittedSessionTransitionRefusalV1, PreparedSessionTransitionPresentationRefusalV1,
    PreparedSessionTransitionPresentationV1, PreparedSessionTransitionV1,
    SessionOperationTransitionRequestV1,
};
use ferrum_render::{
    BatchSpace, DocumentPrecommitOverlayV1, DocumentPrecommitPaintPrimitiveV1, RenderDisplayLayerV1,
};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

/// Opaque one-use generic document transition prepared by the renderer.
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PreparedSessionTransitionV1"
)]
pub(crate) struct PyPreparedSessionTransitionV1 {
    pub(crate) transition: PreparedSessionTransitionV1,
}

/// Opaque one-use generic input consumed by session-transition preparation.
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "SessionOperationTransitionRequestV1"
)]
pub(crate) struct PySessionOperationTransitionRequestV1 {
    request: Option<SessionOperationTransitionRequestV1>,
}

impl PySessionOperationTransitionRequestV1 {
    pub(crate) const fn from_request(request: SessionOperationTransitionRequestV1) -> Self {
        Self {
            request: Some(request),
        }
    }

    fn take_for_preparation(&mut self) -> PyResult<SessionOperationTransitionRequestV1> {
        self.request.take().ok_or_else(|| {
            PreparedOperationConsumedError::new_err(
                "session operation transition request was already transferred to preparation",
            )
        })
    }
}

impl PyPreparedSessionTransitionV1 {
    pub(crate) const fn from_transition(transition: PreparedSessionTransitionV1) -> Self {
        Self { transition }
    }
}

#[pymethods]
impl PyPreparedSessionTransitionV1 {
    /// Copy immutable presentation facts without redeeming this receipt.
    fn presentation_v1(
        &self,
        py: Python<'_>,
    ) -> PyResult<PyPreparedSessionTransitionPresentationV1> {
        self.transition
            .presentation_v1()
            .map(|presentation| presentation_from(py, presentation))
            .map_err(presentation_refusal_error)?
    }
}

/// Immutable display facts copied from a live generic prepared transition.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PreparedSessionTransitionPresentationV1"
)]
pub(crate) struct PyPreparedSessionTransitionPresentationV1 {
    #[pyo3(get)]
    precommit_overlay: Option<PyDocumentPrecommitOverlayV1>,
}

/// Renderer-owned, identifier-free paint subset for one prepared transition.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentPrecommitOverlayV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentPrecommitOverlayV1 {
    primitives: Vec<PyDocumentPrecommitPaintPrimitiveV1>,
}

#[pymethods]
impl PyDocumentPrecommitOverlayV1 {
    #[getter]
    fn primitives(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, self.primitives.iter().cloned())?.unbind())
    }
}

/// Coordinate context for one precommit paint primitive.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentPrecommitOverlayCoordinateSpaceV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentPrecommitOverlayCoordinateSpaceV1 {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    anchor: Option<PyRenderPointV1>,
}

/// One ordered, immutable renderer-owned precommit paint primitive.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentPrecommitPaintPrimitiveV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentPrecommitPaintPrimitiveV1 {
    #[pyo3(get)]
    coordinate_space: PyDocumentPrecommitOverlayCoordinateSpaceV1,
    #[pyo3(get)]
    display_layer: String,
    #[pyo3(get)]
    operation: PyRenderOperationV2,
}

#[pymethods]
impl PyDocumentSession {
    /// Prepare one opaque generic transition request without changing the session.
    fn prepare_session_operation_transition_v1(
        &mut self,
        py: Python<'_>,
        mut request: PyRefMut<'_, PySessionOperationTransitionRequestV1>,
    ) -> PyResult<PyPreparedSessionTransitionV1> {
        let request = request.take_for_preparation()?;
        document_result(
            py,
            self.session
                .prepare_session_operation_transition_v1(request),
        )
        .map(PyPreparedSessionTransitionV1::from_transition)
    }

    /// Redeem one renderer-admitted transition through the generic authority.
    fn commit_session_operation_transition_v1(
        &mut self,
        mut prepared: PyRefMut<'_, PyPreparedSessionTransitionV1>,
    ) -> PyResult<PySessionOperationResultV1> {
        self.session
            .commit_session_operation_transition_v1(&mut prepared.transition)
            .map(Into::into)
            .map_err(commit_refusal_error)
    }
}

pub(crate) fn presentation_from(
    py: Python<'_>,
    presentation: PreparedSessionTransitionPresentationV1,
) -> PyResult<PyPreparedSessionTransitionPresentationV1> {
    let precommit_overlay = presentation
        .precommit_overlay()
        .map(|overlay| overlay_from(py, overlay))
        .transpose()?;
    Ok(PyPreparedSessionTransitionPresentationV1 { precommit_overlay })
}

pub(crate) fn overlay_from(
    py: Python<'_>,
    overlay: &DocumentPrecommitOverlayV1,
) -> PyResult<PyDocumentPrecommitOverlayV1> {
    let primitives = overlay
        .primitives()
        .iter()
        .map(|primitive| paint_primitive_from(py, primitive))
        .collect::<PyResult<_>>()?;
    Ok(PyDocumentPrecommitOverlayV1 { primitives })
}

fn paint_primitive_from(
    py: Python<'_>,
    primitive: &DocumentPrecommitPaintPrimitiveV1,
) -> PyResult<PyDocumentPrecommitPaintPrimitiveV1> {
    let coordinate_space = match primitive.coordinate_space() {
        BatchSpace::AtomLocal { anchor } => PyDocumentPrecommitOverlayCoordinateSpaceV1 {
            kind: "atom_local".to_owned(),
            anchor: Some((*anchor).into()),
        },
        BatchSpace::Scene => PyDocumentPrecommitOverlayCoordinateSpaceV1 {
            kind: "scene".to_owned(),
            anchor: None,
        },
    };
    let display_layer = match primitive.display_layer() {
        RenderDisplayLayerV1::Ordinary => "ordinary".to_owned(),
        RenderDisplayLayerV1::HaworthFrontStroke => "haworth_front_stroke".to_owned(),
        RenderDisplayLayerV1::HaworthFrontWedge => "haworth_front_wedge".to_owned(),
    };
    Ok(PyDocumentPrecommitPaintPrimitiveV1 {
        coordinate_space,
        display_layer,
        operation: operation_from(py, primitive.operation())?,
    })
}

fn presentation_refusal_error(error: PreparedSessionTransitionPresentationRefusalV1) -> PyErr {
    match error {
        PreparedSessionTransitionPresentationRefusalV1::Retired => {
            PreparedOperationConsumedError::new_err(
                "prepared session transition was already retired",
            )
        }
    }
}

fn commit_refusal_error(error: AdmittedSessionTransitionRefusalV1) -> PyErr {
    match error {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            PreparedOperationForeignSessionError::new_err(
                "prepared session transition belongs to another session",
            )
        }
        AdmittedSessionTransitionRefusalV1::Replayed => PreparedOperationConsumedError::new_err(
            "prepared session transition was already redeemed or retired",
        ),
        AdmittedSessionTransitionRefusalV1::StaleSnapshot
        | AdmittedSessionTransitionRefusalV1::RendererAdmission
        | AdmittedSessionTransitionRefusalV1::ProvisionalCapability
        | AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
            OperationValidationError::new_err("prepared session transition could not be redeemed")
        }
        _ => OperationValidationError::new_err("prepared session transition could not be redeemed"),
    }
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PySessionOperationTransitionRequestV1>()?;
    module.add_class::<PyPreparedSessionTransitionV1>()?;
    module.add_class::<PyPreparedSessionTransitionPresentationV1>()?;
    module.add_class::<PyDocumentPrecommitOverlayV1>()?;
    module.add_class::<PyDocumentPrecommitOverlayCoordinateSpaceV1>()?;
    module.add_class::<PyDocumentPrecommitPaintPrimitiveV1>()
}
