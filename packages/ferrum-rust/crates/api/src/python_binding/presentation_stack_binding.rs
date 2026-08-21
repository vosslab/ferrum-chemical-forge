//! Frozen Python intent values for direct-root presentation stack ordering.

use ferrum_document::{
    PresentationRootSelectorV1, PresentationStackOrderV1, PresentationStackReorderV1,
    SessionOperation, SessionOperationV1,
};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::binding::operation_validation_error;
use super::presentation_deletion_binding::PyDocumentPresentationRootKindV1;

/// Closed ordering transformations accepted by the Rust document session.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DocumentPresentationStackOrderV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyDocumentPresentationStackOrderV1 {
    BringToFront,
    SendToBack,
    ReverseSelectedSlots,
}

impl From<PyDocumentPresentationStackOrderV1> for PresentationStackOrderV1 {
    fn from(value: PyDocumentPresentationStackOrderV1) -> Self {
        match value {
            PyDocumentPresentationStackOrderV1::BringToFront => Self::BringToFront,
            PyDocumentPresentationStackOrderV1::SendToBack => Self::SendToBack,
            PyDocumentPresentationStackOrderV1::ReverseSelectedSlots => Self::ReverseSelectedSlots,
        }
    }
}

/// One immutable exact-kind durable presentation target.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentPresentationRootSelectorV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentPresentationRootSelectorV1 {
    pub(crate) selector: PresentationRootSelectorV1,
    #[pyo3(get)]
    presentation_id: String,
    #[pyo3(get)]
    kind: PyDocumentPresentationRootKindV1,
}

#[pymethods]
impl PyDocumentPresentationRootSelectorV1 {
    /// Validate one authored persistent ID and closed presentation kind.
    #[staticmethod]
    fn create(
        py: Python<'_>,
        presentation_id: String,
        kind: PyRef<'_, PyDocumentPresentationRootKindV1>,
    ) -> PyResult<Self> {
        let selector = PresentationRootSelectorV1::new(presentation_id.clone(), (*kind).into())
            .map_err(|error| operation_validation_error(py, error.to_string()))?;
        Ok(Self {
            selector,
            presentation_id,
            kind: *kind,
        })
    }
}

pub(crate) fn reorder_presentation_roots(
    py: Python<'_>,
    order: PyRef<'_, PyDocumentPresentationStackOrderV1>,
    targets: &Bound<'_, PyTuple>,
) -> PyResult<SessionOperation> {
    let targets = presentation_selectors(py, targets, "presentation stack")?;
    let reorder = PresentationStackReorderV1::new((*order).into(), targets)
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(SessionOperation::V1(
        SessionOperationV1::ReorderPresentationRoots { reorder },
    ))
}

pub(crate) fn presentation_selectors(
    py: Python<'_>,
    targets: &Bound<'_, PyTuple>,
    operation: &str,
) -> PyResult<Vec<PresentationRootSelectorV1>> {
    if !targets.is_exact_instance_of::<PyTuple>() {
        return Err(operation_validation_error(
            py,
            format!("{operation} targets must be an exact built-in tuple"),
        ));
    }
    targets
        .iter()
        .map(|value| {
            value
                .extract::<PyRef<'_, PyDocumentPresentationRootSelectorV1>>()
                .map(|target| target.selector.clone())
                .map_err(Into::into)
        })
        .collect::<PyResult<Vec<_>>>()
}
