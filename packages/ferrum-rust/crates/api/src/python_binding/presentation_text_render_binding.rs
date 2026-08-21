//! Frozen Python DTOs for exact direct-root Text render layouts.

use ferrum_render::{DocumentTextRenderV1, PresentationGlyphRun, PresentationTextOp};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::projection_binding::PyPresentationTargetV1;
use super::render_binding::{PyGlyphPlacementV1, PyPresentationTextBoundsV1, PyRenderPointV1};

#[pyclass(frozen, name = "PresentationTextSourceRunV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationTextSourceRunV1 {
    #[pyo3(get)]
    text: String,
    #[pyo3(get)]
    script: String,
}

#[pyclass(frozen, name = "PresentationGlyphRunV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationGlyphRunV1 {
    #[pyo3(get)]
    text: String,
    #[pyo3(get)]
    script: String,
    #[pyo3(get)]
    origin: PyRenderPointV1,
    glyphs: Vec<PyGlyphPlacementV1>,
    #[pyo3(get)]
    scale: f64,
}

#[pymethods]
impl PyPresentationGlyphRunV1 {
    #[getter]
    fn glyphs(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, self.glyphs.iter().cloned())?.unbind())
    }
}

#[pyclass(frozen, name = "PresentationTextOpV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationTextOpV1 {
    runs: Vec<PyPresentationGlyphRunV1>,
    #[pyo3(get)]
    face: String,
    #[pyo3(get)]
    size: f64,
    #[pyo3(get)]
    paint: String,
    #[pyo3(get)]
    z: i32,
}

#[pymethods]
impl PyPresentationTextOpV1 {
    #[getter]
    fn runs(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, self.runs.iter().cloned())?.unbind())
    }
}

#[pyclass(frozen, name = "DocumentTextRenderV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyDocumentTextRenderV1 {
    #[pyo3(get)]
    target: PyPresentationTargetV1,
    #[pyo3(get)]
    anchor: PyRenderPointV1,
    source_runs: Vec<PyPresentationTextSourceRunV1>,
    #[pyo3(get)]
    operation: PyPresentationTextOpV1,
    #[pyo3(get)]
    bounds: PyPresentationTextBoundsV1,
    #[pyo3(get)]
    background: Option<String>,
}

#[pymethods]
impl PyDocumentTextRenderV1 {
    #[getter]
    fn source_runs(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, self.source_runs.iter().cloned())?.unbind())
    }
}

impl From<&DocumentTextRenderV1> for PyDocumentTextRenderV1 {
    fn from(value: &DocumentTextRenderV1) -> Self {
        let bounds = value.bounds();
        Self {
            target: value.target().into(),
            anchor: value.anchor().into(),
            source_runs: value
                .source_runs()
                .iter()
                .map(|run| PyPresentationTextSourceRunV1 {
                    text: run.text().to_owned(),
                    script: script(run.script()).to_owned(),
                })
                .collect(),
            operation: operation(value.operation()),
            bounds: PyPresentationTextBoundsV1 {
                left: bounds.left(),
                top: bounds.top(),
                right: bounds.right(),
                bottom: bounds.bottom(),
            },
            background: value
                .background()
                .map(|paint| paint.color().as_str().to_owned()),
        }
    }
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyPresentationTextSourceRunV1>()?;
    module.add_class::<PyPresentationGlyphRunV1>()?;
    module.add_class::<PyPresentationTextOpV1>()?;
    module.add_class::<PyDocumentTextRenderV1>()
}

fn operation(value: &PresentationTextOp) -> PyPresentationTextOpV1 {
    PyPresentationTextOpV1 {
        runs: value.runs().iter().map(glyph_run).collect(),
        face: value.face().as_str().to_owned(),
        size: value.size().get(),
        paint: value.paint().color().as_str().to_owned(),
        z: value.z(),
    }
}

fn glyph_run(value: &PresentationGlyphRun) -> PyPresentationGlyphRunV1 {
    PyPresentationGlyphRunV1 {
        text: value.text().to_owned(),
        script: script(value.script()).to_owned(),
        origin: value.origin().into(),
        glyphs: value
            .glyphs()
            .iter()
            .map(|glyph| PyGlyphPlacementV1 {
                glyph_index: glyph.glyph_index(),
                origin: glyph.origin().into(),
            })
            .collect(),
        scale: value.scale().get(),
    }
}

fn script(value: ferrum_render::TextScript) -> &'static str {
    match value {
        ferrum_render::TextScript::Baseline => "baseline",
        ferrum_render::TextScript::Subscript => "subscript",
        ferrum_render::TextScript::Superscript => "superscript",
    }
}
