//! Frozen Python DTOs for direct-root Text source projections.

use ferrum_document::{
    PresentationFactProvenanceV1, PresentationTextFontV1, PresentationTextRunV1,
    PresentationTextStyleV1, TextProjectionV1,
};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::projection_binding::{PyPoint3V1, PyPresentationFillV1, PyPresentationTargetV1};

#[pyclass(frozen, name = "PresentationTextRunV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationTextRunV1 {
    #[pyo3(get)]
    text: String,
    styles: Vec<String>,
}

#[pymethods]
impl PyPresentationTextRunV1 {
    #[getter]
    fn styles(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, self.styles.iter())?.unbind())
    }
}

impl From<&PresentationTextRunV1> for PyPresentationTextRunV1 {
    fn from(value: &PresentationTextRunV1) -> Self {
        Self {
            text: value.text().to_owned(),
            styles: value
                .styles()
                .iter()
                .map(|style| text_style(*style).to_owned())
                .collect(),
        }
    }
}

#[pyclass(frozen, name = "PresentationTextFontV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationTextFontV1 {
    #[pyo3(get)]
    family: Option<String>,
    #[pyo3(get)]
    family_provenance: String,
    #[pyo3(get)]
    size: f64,
    #[pyo3(get)]
    size_provenance: String,
    #[pyo3(get)]
    color: String,
    #[pyo3(get)]
    color_provenance: String,
}

impl From<&PresentationTextFontV1> for PyPresentationTextFontV1 {
    fn from(value: &PresentationTextFontV1) -> Self {
        Self {
            family: value.family().map(str::to_owned),
            family_provenance: provenance(value.family_provenance()).to_owned(),
            size: value.size().value(),
            size_provenance: provenance(value.size_provenance()).to_owned(),
            color: value.color().as_str().to_owned(),
            color_provenance: provenance(value.color_provenance()).to_owned(),
        }
    }
}

#[pyclass(frozen, name = "TextProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyTextProjectionV1 {
    #[pyo3(get)]
    target: PyPresentationTargetV1,
    #[pyo3(get)]
    anchor: PyPoint3V1,
    runs: Vec<PyPresentationTextRunV1>,
    #[pyo3(get)]
    font: PyPresentationTextFontV1,
    #[pyo3(get)]
    background: PyPresentationFillV1,
}

#[pymethods]
impl PyTextProjectionV1 {
    #[getter]
    fn runs(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, self.runs.iter().cloned())?.unbind())
    }
}

impl From<&TextProjectionV1> for PyTextProjectionV1 {
    fn from(value: &TextProjectionV1) -> Self {
        Self {
            target: value.target().into(),
            anchor: PyPoint3V1 {
                x: value.anchor().x(),
                y: value.anchor().y(),
                z: value.anchor().z(),
            },
            runs: value.runs().iter().map(Into::into).collect(),
            font: value.font().into(),
            background: value.background().into(),
        }
    }
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyPresentationTextRunV1>()?;
    module.add_class::<PyPresentationTextFontV1>()?;
    module.add_class::<PyTextProjectionV1>()
}

fn provenance(value: PresentationFactProvenanceV1) -> &'static str {
    match value {
        PresentationFactProvenanceV1::Root => "root",
        PresentationFactProvenanceV1::Standard => "standard",
        PresentationFactProvenanceV1::Builtin => "builtin",
    }
}

fn text_style(value: PresentationTextStyleV1) -> &'static str {
    match value {
        PresentationTextStyleV1::Bold => "bold",
        PresentationTextStyleV1::Italic => "italic",
        PresentationTextStyleV1::Subscript => "subscript",
        PresentationTextStyleV1::Superscript => "superscript",
    }
}
