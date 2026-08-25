//! Frozen Python transport for renderer-owned immutable presentation plans.

use ferrum_document::DocumentRenderObservationV1;
use ferrum_render::{
    DocumentVectorOpV1, PathCommandV1, PresentationRenderBoundsV1, PresentationRenderPlanV1,
    PresentationRenderRootV1, RenderError, render_presentation_stack_v1,
};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::binding::document_result;
use super::document_session_binding::{PyDocumentSession, hex_digest};
use super::render_binding::{
    PyDocumentPlusRenderV1, PyRenderPointV1, PyRenderTargetV1, RenderDepictionError,
    RenderProvenanceError,
};

/// Renderer-calculated finite painted scene bounds for one presentation root.
#[pyclass(frozen, name = "PresentationRenderBoundsV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationRenderBoundsV1 {
    #[pyo3(get)]
    left: f64,
    #[pyo3(get)]
    top: f64,
    #[pyo3(get)]
    right: f64,
    #[pyo3(get)]
    bottom: f64,
}

impl From<PresentationRenderBoundsV1> for PyPresentationRenderBoundsV1 {
    fn from(value: PresentationRenderBoundsV1) -> Self {
        Self {
            left: value.left(),
            top: value.top(),
            right: value.right(),
            bottom: value.bottom(),
        }
    }
}

/// One command in a renderer-issued presentation path.
#[pyclass(frozen, name = "PresentationPathCommandV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationPathCommandV1 {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    point: Option<PyRenderPointV1>,
    #[pyo3(get)]
    control_1: Option<PyRenderPointV1>,
    #[pyo3(get)]
    control_2: Option<PyRenderPointV1>,
}

impl From<&PathCommandV1> for PyPresentationPathCommandV1 {
    fn from(value: &PathCommandV1) -> Self {
        match value {
            PathCommandV1::MoveTo(point) => Self {
                kind: "move_to".to_owned(),
                point: Some((*point).into()),
                control_1: None,
                control_2: None,
            },
            PathCommandV1::LineTo(point) => Self {
                kind: "line_to".to_owned(),
                point: Some((*point).into()),
                control_1: None,
                control_2: None,
            },
            PathCommandV1::CubicTo {
                control_1,
                control_2,
                end,
            } => Self {
                kind: "cubic_to".to_owned(),
                point: Some((*end).into()),
                control_1: Some((*control_1).into()),
                control_2: Some((*control_2).into()),
            },
            PathCommandV1::Close => Self {
                kind: "close".to_owned(),
                point: None,
                control_1: None,
                control_2: None,
            },
        }
    }
}

/// One explicit renderer-issued stroke.
#[pyclass(frozen, name = "PresentationRenderStrokeV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationRenderStrokeV1 {
    #[pyo3(get)]
    paint: String,
    #[pyo3(get)]
    width: f64,
    #[pyo3(get)]
    line_cap: String,
    #[pyo3(get)]
    line_join: String,
    #[pyo3(get)]
    miter_limit: f64,
}

/// One exact renderer-issued vector operation.
#[pyclass(frozen, name = "PresentationVectorOperationV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationVectorOperationV1 {
    #[pyo3(get)]
    kind: String,
    commands: Vec<PyPresentationPathCommandV1>,
    #[pyo3(get)]
    center: Option<PyRenderPointV1>,
    #[pyo3(get)]
    radius_x: Option<f64>,
    #[pyo3(get)]
    radius_y: Option<f64>,
    #[pyo3(get)]
    stroke: Option<PyPresentationRenderStrokeV1>,
    #[pyo3(get)]
    fill: Option<String>,
}

#[pymethods]
impl PyPresentationVectorOperationV1 {
    #[getter]
    fn commands(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, self.commands.iter().cloned())?.unbind())
    }
}

/// One renderer-owned root in source order.
#[pyclass(frozen, name = "PresentationRenderRootV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationRenderRootV1 {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    target: PyRenderTargetV1,
    #[pyo3(get)]
    bounds: PyPresentationRenderBoundsV1,
    vector_operations: Vec<PyPresentationVectorOperationV1>,
    #[pyo3(get)]
    plus: Option<PyDocumentPlusRenderV1>,
    #[pyo3(get)]
    text: Option<super::presentation_text_render_binding::PyDocumentTextRenderV1>,
}

#[pymethods]
impl PyPresentationRenderRootV1 {
    #[getter]
    fn vector_operations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, self.vector_operations.iter().cloned())?.unbind())
    }
}

/// Immutable pure renderer plan for one fenced document presentation stack.
#[pyclass(frozen, name = "PresentationRenderPlanV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationRenderPlanV1 {
    #[pyo3(get)]
    schema: &'static str,
    #[pyo3(get)]
    revision: u64,
    #[pyo3(get)]
    digest: String,
    roots: Vec<PyPresentationRenderRootV1>,
}

#[pymethods]
impl PyPresentationRenderPlanV1 {
    #[getter]
    fn roots(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, self.roots.iter().cloned())?.unbind())
    }
}

impl From<&PresentationRenderPlanV1> for PyPresentationRenderPlanV1 {
    fn from(value: &PresentationRenderPlanV1) -> Self {
        Self {
            schema: value.schema(),
            revision: value.revision(),
            digest: hex_digest(value.digest()),
            roots: value.roots().iter().map(root_from).collect(),
        }
    }
}

#[pymethods]
impl PyDocumentSession {
    /// Render one immutable presentation plan only when its revision and digest still match.
    fn observe_presentation_render_plan_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest: String,
    ) -> PyResult<PyPresentationRenderPlanV1> {
        let observation = document_result(py, self.session.observe(expected_revision))?;
        if hex_digest(observation.snapshot().digest()) != expected_digest {
            return Err(RenderProvenanceError::new_err(
                "presentation render plan digest did not match the authoritative document",
            ));
        }
        let published_matches = self
            .published_presentation_plan
            .as_ref()
            .is_some_and(|plan| {
                plan.revision() == observation.snapshot().revision()
                    && plan.digest() == observation.snapshot().digest()
            });
        if !published_matches {
            self.publish_live_render_plan_v1(py, expected_revision)?;
        }
        let plan = self
            .published_presentation_plan
            .as_ref()
            .expect("same-fence publication retains its renderer plan");
        Ok(plan.into())
    }
}

/// Derive one presentation plan from the exact document observation that produced
/// the live render publication.
pub(crate) fn plan_from_observation(
    observation: &DocumentRenderObservationV1,
) -> PyResult<PresentationRenderPlanV1> {
    let document = observation.document();
    let plan = render_presentation_stack_v1(document.projection().presentation_stack())
        .map_err(render_error)?;
    if plan.revision() != document.snapshot().revision()
        || plan.digest() != document.snapshot().digest()
    {
        return Err(RenderProvenanceError::new_err(
            "presentation render plan provenance did not match the authoritative document",
        ));
    }
    Ok(plan)
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyPresentationRenderBoundsV1>()?;
    module.add_class::<PyPresentationPathCommandV1>()?;
    module.add_class::<PyPresentationRenderStrokeV1>()?;
    module.add_class::<PyPresentationVectorOperationV1>()?;
    module.add_class::<PyPresentationRenderRootV1>()?;
    module.add_class::<PyPresentationRenderPlanV1>()
}

fn root_from(value: &PresentationRenderRootV1) -> PyPresentationRenderRootV1 {
    let bounds = value.bounds().into();
    match value {
        PresentationRenderRootV1::Vector { target, vector, .. } => PyPresentationRenderRootV1 {
            kind: "vector".to_owned(),
            target: target.into(),
            bounds,
            vector_operations: vector.operations().iter().map(vector_operation).collect(),
            plus: None,
            text: None,
        },
        PresentationRenderRootV1::Plus { render, .. } => PyPresentationRenderRootV1 {
            kind: "plus".to_owned(),
            target: render.target().into(),
            bounds,
            vector_operations: Vec::new(),
            plus: Some(super::render_binding::plus_from(render)),
            text: None,
        },
        PresentationRenderRootV1::Text { render, .. } => PyPresentationRenderRootV1 {
            kind: "text".to_owned(),
            target: render.target().into(),
            bounds,
            vector_operations: Vec::new(),
            plus: None,
            text: Some(render.into()),
        },
    }
}

fn vector_operation(value: &DocumentVectorOpV1) -> PyPresentationVectorOperationV1 {
    let stroke = value.stroke().map(|stroke| PyPresentationRenderStrokeV1 {
        paint: stroke.paint().color().as_str().to_owned(),
        width: stroke.width().get(),
        line_cap: "butt".to_owned(),
        line_join: "miter".to_owned(),
        miter_limit: stroke.miter_limit(),
    });
    let fill = value.fill().map(|paint| paint.color().as_str().to_owned());
    match value {
        DocumentVectorOpV1::Path { commands, .. } => PyPresentationVectorOperationV1 {
            kind: "path".to_owned(),
            commands: commands.iter().map(Into::into).collect(),
            center: None,
            radius_x: None,
            radius_y: None,
            stroke,
            fill,
        },
        DocumentVectorOpV1::Ellipse {
            center,
            radius_x,
            radius_y,
            ..
        } => PyPresentationVectorOperationV1 {
            kind: "ellipse".to_owned(),
            commands: Vec::new(),
            center: Some((*center).into()),
            radius_x: Some(radius_x.get()),
            radius_y: Some(radius_y.get()),
            stroke,
            fill,
        },
    }
}

fn render_error(error: RenderError) -> PyErr {
    RenderDepictionError::new_err(error.to_string())
}
