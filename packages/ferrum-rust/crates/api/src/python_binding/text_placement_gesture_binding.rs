//! PyO3 boundary for opaque standalone Text placement.

use super::binding::{PyDocumentSession, PySessionOperationResultV1};
use super::presentation_creation_gesture_binding::digest;
use super::presentation_text_render_binding::PyDocumentTextRenderV1;
use super::text_properties_binding::PyDocumentTextEditRunV1;
use crate::{
    ApiTextPlacementGestureV1, ApiTextPlacementPreviewV1, begin_api_text_placement_gesture_v1,
    commit_api_text_placement_gesture_v1, preview_api_text_placement_gesture_v1,
    text_placement_defaults_v1,
};
use ferrum_document::{
    DocumentFenceV1, PresentationGesturePoint2V1, Rgb24V1, TextPlacementContentV1,
    TextPlacementErrorV1,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

create_exception!(
    ferrum_chem,
    TextPlacementError,
    super::binding::DocumentError
);
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "TextPlacementErrorCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyTextPlacementErrorCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    ReplayedGesture,
    InvalidAnchor,
    BlankContent,
    UnsupportedStyle,
    InvalidFontOverride,
    UnrenderableStandard,
    RenderPreparation,
    SessionConflict,
}
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "TextPlacementRecoveryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyTextPlacementRecoveryV1 {
    RestartTool,
    ChooseAnotherLocation,
    CorrectText,
    RepairDrawingStandard,
    RecoverCanvas,
    RefreshThenRetry,
}
#[pyclass(unsendable, module = "ferrum_chem", name = "TextPlacementGestureV1")]
pub(crate) struct PyTextPlacementGestureV1 {
    gesture: ApiTextPlacementGestureV1,
}
#[pyclass(unsendable, module = "ferrum_chem", name = "TextPlacementPreviewV1")]
pub(crate) struct PyTextPlacementPreviewV1 {
    preview: ApiTextPlacementPreviewV1,
    #[pyo3(get)]
    overlay: PyDocumentTextRenderV1,
}
#[pyclass(frozen, module = "ferrum_chem", name = "TextPlacementDefaultsV1")]
pub(crate) struct PyTextPlacementDefaultsV1 {
    runs: Vec<PyDocumentTextEditRunV1>,
    #[pyo3(get)]
    font_size: f64,
    #[pyo3(get)]
    color: String,
    #[pyo3(get)]
    bold_supported: bool,
    #[pyo3(get)]
    italic_supported: bool,
    #[pyo3(get)]
    font_family_supported: bool,
}
#[pymethods]
impl PyTextPlacementDefaultsV1 {
    #[getter]
    fn runs(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, self.runs.iter().cloned())?.unbind())
    }
}
#[pyclass(frozen, module = "ferrum_chem", name = "TextPlacementCommitV1")]
pub(crate) struct PyTextPlacementCommitV1 {
    #[pyo3(get)]
    identifier: String,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}

#[pymethods]
impl PyDocumentSession {
    fn begin_text_placement_gesture_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        x: f64,
        y: f64,
    ) -> PyResult<PyTextPlacementGestureV1> {
        let anchor = PresentationGesturePoint2V1::new(x, y)
            .map_err(|_| text_error(py, TextPlacementErrorV1::InvalidAnchor))?;
        begin_api_text_placement_gesture_v1(
            &self.session,
            DocumentFenceV1::new(expected_revision, digest(&expected_digest_hex)?),
            anchor,
        )
        .map(|gesture| PyTextPlacementGestureV1 { gesture })
        .map_err(|error| text_error(py, error))
    }
    fn preview_text_placement_gesture_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyTextPlacementGestureV1>,
        runs: &Bound<'_, PyAny>,
        font_size: Option<u16>,
        color: Option<String>,
    ) -> PyResult<PyTextPlacementPreviewV1> {
        let runs = runs
            .cast::<PyTuple>()?
            .iter()
            .map(|item| {
                item.extract::<PyRef<'_, PyDocumentTextEditRunV1>>()
                    .map(|value| value.run.clone())
                    .map_err(Into::into)
            })
            .collect::<PyResult<Vec<_>>>()?;
        let color = color
            .map(|value| {
                Rgb24V1::new(value)
                    .ok_or_else(|| text_error(py, TextPlacementErrorV1::InvalidFontOverride))
            })
            .transpose()?;
        let content = TextPlacementContentV1::new(runs, font_size, color)
            .map_err(|error| text_error(py, error))?;
        preview_api_text_placement_gesture_v1(&self.session, &gesture.gesture, content)
            .map(|preview| PyTextPlacementPreviewV1 {
                overlay: preview.overlay().into(),
                preview,
            })
            .map_err(|error| text_error(py, error))
    }
    fn text_placement_defaults_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyTextPlacementGestureV1>,
    ) -> PyResult<PyTextPlacementDefaultsV1> {
        text_placement_defaults_v1(&self.session, &gesture.gesture)
            .map(|value| PyTextPlacementDefaultsV1 {
                runs: value
                    .runs()
                    .iter()
                    .cloned()
                    .map(|run| PyDocumentTextEditRunV1 { run })
                    .collect(),
                font_size: value.font_size(),
                color: value.color().to_owned(),
                bold_supported: false,
                italic_supported: false,
                font_family_supported: false,
            })
            .map_err(|error| text_error(py, error))
    }
    fn commit_text_placement_gesture_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyTextPlacementGestureV1>,
        preview: PyRef<'_, PyTextPlacementPreviewV1>,
    ) -> PyResult<PyTextPlacementCommitV1> {
        commit_api_text_placement_gesture_v1(&mut self.session, &gesture.gesture, &preview.preview)
            .map(|value| PyTextPlacementCommitV1 {
                identifier: value.identifier().to_owned(),
                result: value.result().clone().into(),
            })
            .map_err(|error| text_error(py, error))
    }
}
fn text_error(py: Python<'_>, error: TextPlacementErrorV1) -> PyErr {
    let category = match error.category() {
        ferrum_document::TextPlacementErrorCategoryV1::StaleSnapshot => {
            PyTextPlacementErrorCategoryV1::StaleSnapshot
        }
        ferrum_document::TextPlacementErrorCategoryV1::ForeignSession => {
            PyTextPlacementErrorCategoryV1::ForeignSession
        }
        ferrum_document::TextPlacementErrorCategoryV1::MismatchedPreview => {
            PyTextPlacementErrorCategoryV1::MismatchedPreview
        }
        ferrum_document::TextPlacementErrorCategoryV1::ReplayedGesture => {
            PyTextPlacementErrorCategoryV1::ReplayedGesture
        }
        ferrum_document::TextPlacementErrorCategoryV1::InvalidAnchor => {
            PyTextPlacementErrorCategoryV1::InvalidAnchor
        }
        ferrum_document::TextPlacementErrorCategoryV1::BlankContent => {
            PyTextPlacementErrorCategoryV1::BlankContent
        }
        ferrum_document::TextPlacementErrorCategoryV1::UnsupportedStyle => {
            PyTextPlacementErrorCategoryV1::UnsupportedStyle
        }
        ferrum_document::TextPlacementErrorCategoryV1::InvalidFontOverride => {
            PyTextPlacementErrorCategoryV1::InvalidFontOverride
        }
        ferrum_document::TextPlacementErrorCategoryV1::UnrenderableStandard => {
            PyTextPlacementErrorCategoryV1::UnrenderableStandard
        }
        ferrum_document::TextPlacementErrorCategoryV1::RenderPreparation => {
            PyTextPlacementErrorCategoryV1::RenderPreparation
        }
        ferrum_document::TextPlacementErrorCategoryV1::SessionConflict => {
            PyTextPlacementErrorCategoryV1::SessionConflict
        }
    };
    let recovery = match error.recovery() {
        ferrum_document::TextPlacementRecoveryV1::RestartTool => {
            PyTextPlacementRecoveryV1::RestartTool
        }
        ferrum_document::TextPlacementRecoveryV1::ChooseAnotherLocation => {
            PyTextPlacementRecoveryV1::ChooseAnotherLocation
        }
        ferrum_document::TextPlacementRecoveryV1::CorrectText => {
            PyTextPlacementRecoveryV1::CorrectText
        }
        ferrum_document::TextPlacementRecoveryV1::RepairDrawingStandard => {
            PyTextPlacementRecoveryV1::RepairDrawingStandard
        }
        ferrum_document::TextPlacementRecoveryV1::RecoverCanvas => {
            PyTextPlacementRecoveryV1::RecoverCanvas
        }
        ferrum_document::TextPlacementRecoveryV1::RefreshThenRetry => {
            PyTextPlacementRecoveryV1::RefreshThenRetry
        }
    };
    let exception = TextPlacementError::new_err(error.to_string());
    let value = exception.value(py);
    value
        .setattr("category", Py::new(py, category).expect("enum allocates"))
        .expect("category attaches");
    value
        .setattr("recovery", Py::new(py, recovery).expect("enum allocates"))
        .expect("recovery attaches");
    exception
}
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "TextPlacementError",
        module.py().get_type::<TextPlacementError>(),
    )?;
    module.add_class::<PyTextPlacementErrorCategoryV1>()?;
    module.add_class::<PyTextPlacementRecoveryV1>()?;
    module.add_class::<PyTextPlacementGestureV1>()?;
    module.add_class::<PyTextPlacementPreviewV1>()?;
    module.add_class::<PyTextPlacementDefaultsV1>()?;
    module.add_class::<PyTextPlacementCommitV1>()
}
