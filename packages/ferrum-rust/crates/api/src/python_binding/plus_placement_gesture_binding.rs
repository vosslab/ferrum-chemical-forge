//! Dedicated PyO3 facade for renderer-backed standard Plus placement.

use super::binding::{PyDocumentSession, PySessionOperationResultV1};
use super::presentation_creation_gesture_binding::{digest, presentation_error};
use ferrum_document::{
    DocumentFenceV1, PendingPresentationGestureV1, PresentationCreationGestureV1,
    PresentationGestureKindV1, PresentationGesturePoint2V1, PresentationGestureSnapPolicyV1,
    PresentationGestureStyleV1,
};
use pyo3::prelude::*;

#[pyclass(unsendable, module = "ferrum_chem", name = "PlusPlacementGestureV1")]
pub(crate) struct PyPlusPlacementGestureV1 {
    gesture: PresentationCreationGestureV1,
    start: PresentationGesturePoint2V1,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PlusPlacementOverlayV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyPlusPlacementOverlayV1 {
    #[pyo3(get)]
    origin_x: f64,
    #[pyo3(get)]
    origin_y: f64,
    #[pyo3(get)]
    left: f64,
    #[pyo3(get)]
    top: f64,
    #[pyo3(get)]
    right: f64,
    #[pyo3(get)]
    bottom: f64,
    #[pyo3(get)]
    text: String,
    #[pyo3(get)]
    font_size: f64,
    #[pyo3(get)]
    color: String,
    #[pyo3(get)]
    background: Option<String>,
}
#[pyclass(unsendable, module = "ferrum_chem", name = "PlusPlacementPreviewV1")]
pub(crate) struct PyPlusPlacementPreviewV1 {
    preview: PendingPresentationGestureV1,
    #[pyo3(get)]
    overlay: PyPlusPlacementOverlayV1,
}
#[pyclass(frozen, module = "ferrum_chem", name = "PlusPlacementCommitV1")]
pub(crate) struct PyPlusPlacementCommitV1 {
    #[pyo3(get)]
    identifier: String,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}

#[pymethods]
impl PyDocumentSession {
    fn begin_plus_placement_gesture_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        x: f64,
        y: f64,
    ) -> PyResult<PyPlusPlacementGestureV1> {
        let fence = DocumentFenceV1::new(expected_revision, digest(&expected_digest_hex)?);
        let start = PresentationGesturePoint2V1::new(x, y)
            .map_err(|error| presentation_error(py, error))?;
        self.session
            .begin_presentation_creation_gesture_v1(
                fence,
                PresentationGestureKindV1::Plus,
                start,
                PresentationGestureStyleV1::Plus,
                PresentationGestureSnapPolicyV1::free(),
            )
            .map(|gesture| PyPlusPlacementGestureV1 { gesture, start })
            .map_err(|error| presentation_error(py, error))
    }
    fn preview_plus_placement_gesture_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyPlusPlacementGestureV1>,
    ) -> PyResult<PyPlusPlacementPreviewV1> {
        self.session
            .prepare_presentation_creation_gesture_v1(&gesture.gesture, gesture.start)
            .map(preview)
            .map_err(|error| presentation_error(py, error))
    }
    fn commit_plus_placement_gesture_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyPlusPlacementGestureV1>,
        mut preview: PyRefMut<'_, PyPlusPlacementPreviewV1>,
    ) -> PyResult<PyPlusPlacementCommitV1> {
        if !preview.preview.matches(&gesture.gesture) {
            return Err(presentation_error(
                py,
                ferrum_document::PresentationGestureErrorV1::PreviewMismatch,
            ));
        }
        self.session
            .commit_presentation_creation_gesture_v1(&mut preview.preview)
            .map(|commit| PyPlusPlacementCommitV1 {
                identifier: commit.root().presentation_id().as_str().to_owned(),
                result: commit.result().clone().into(),
            })
            .map_err(|error| presentation_error(py, error))
    }
}
fn preview(value: PendingPresentationGestureV1) -> PyPlusPlacementPreviewV1 {
    let root = value
        .plan()
        .roots()
        .iter()
        .find(|root| root.target().source_id() == Some(value.identifier()))
        .expect("prepared Plus has reserved render root");
    let overlay = match root {
        ferrum_render::PresentationRenderRootV1::Plus { render, .. } => render,
        _ => unreachable!("prepared Plus root is text"),
    };
    PyPlusPlacementPreviewV1 {
        overlay: PyPlusPlacementOverlayV1 {
            origin_x: overlay.operation().origin().x(),
            origin_y: overlay.operation().origin().y(),
            left: root.bounds().left(),
            top: root.bounds().top(),
            right: root.bounds().right(),
            bottom: root.bounds().bottom(),
            text: overlay
                .operation()
                .runs()
                .iter()
                .map(|run| run.text())
                .collect(),
            font_size: overlay.operation().size().get(),
            color: overlay.operation().paint().color().as_str().to_owned(),
            background: overlay
                .background()
                .map(|paint| paint.color().as_str().to_owned()),
        },
        preview: value,
    }
}
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyPlusPlacementGestureV1>()?;
    module.add_class::<PyPlusPlacementOverlayV1>()?;
    module.add_class::<PyPlusPlacementPreviewV1>()?;
    module.add_class::<PyPlusPlacementCommitV1>()
}
