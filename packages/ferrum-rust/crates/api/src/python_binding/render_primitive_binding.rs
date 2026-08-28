//! Frozen primitive render DTOs and one-way Rust-to-Python conversion.

use ferrum_document::PresentationTargetV1;
use ferrum_render::{
    EllipseOp, LineOp, RenderOp, RenderPoint, RenderTarget, TextOp, TextScript,
    VectorStrokeLineCapV1,
};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

#[pyclass(frozen, name = "RenderPointV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderPointV1 {
    #[pyo3(get)]
    pub(crate) x: f64,
    #[pyo3(get)]
    pub(crate) y: f64,
}

impl From<RenderPoint> for PyRenderPointV1 {
    fn from(value: RenderPoint) -> Self {
        Self {
            x: value.x(),
            y: value.y(),
        }
    }
}

#[pyclass(frozen, name = "RenderTargetV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderTargetV1 {
    #[pyo3(get)]
    pub(crate) kind: String,
    #[pyo3(get)]
    pub(crate) document_object_id: String,
}

impl From<&RenderTarget> for PyRenderTargetV1 {
    fn from(value: &RenderTarget) -> Self {
        Self {
            kind: "document_object".to_owned(),
            document_object_id: value.document_object_id().as_str().to_owned(),
        }
    }
}

impl From<&PresentationTargetV1> for PyRenderTargetV1 {
    fn from(value: &PresentationTargetV1) -> Self {
        Self {
            kind: "document_object".to_owned(),
            document_object_id: value.document_object_id().as_str().to_owned(),
        }
    }
}

#[pyclass(frozen, name = "AtomLocalSpaceV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyAtomLocalSpaceV1 {
    #[pyo3(get)]
    pub(crate) kind: String,
    #[pyo3(get)]
    pub(crate) anchor: PyRenderPointV1,
}

#[pyclass(frozen, name = "SceneSpaceV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PySceneSpaceV1 {
    #[pyo3(get)]
    pub(crate) kind: String,
}

#[pyclass(frozen, name = "GlyphPlacementV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyGlyphPlacementV1 {
    #[pyo3(get)]
    pub(crate) glyph_index: u32,
    #[pyo3(get)]
    pub(crate) origin: PyRenderPointV1,
}

#[pyclass(frozen, name = "TextRunV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyTextRunV1 {
    #[pyo3(get)]
    pub(crate) text: String,
    #[pyo3(get)]
    pub(crate) script: String,
    #[pyo3(get)]
    pub(crate) origin: PyRenderPointV1,
    glyphs: Vec<PyGlyphPlacementV1>,
    #[pyo3(get)]
    pub(crate) scale: f64,
}

#[pymethods]
impl PyTextRunV1 {
    #[getter]
    fn glyphs(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.glyphs)
    }
}

#[pyclass(frozen, name = "TextOpV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyTextOpV1 {
    #[pyo3(get)]
    pub(crate) origin: PyRenderPointV1,
    runs: Vec<PyTextRunV1>,
    #[pyo3(get)]
    pub(crate) face: String,
    #[pyo3(get)]
    pub(crate) size: f64,
    #[pyo3(get)]
    pub(crate) paint: PyRenderPaintV3,
    #[pyo3(get)]
    pub(crate) z: i32,
}

#[pymethods]
impl PyTextOpV1 {
    #[getter]
    fn runs(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.runs)
    }
}

#[pyclass(frozen, name = "LineOpV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyLineOpV1 {
    #[pyo3(get)]
    pub(crate) start: PyRenderPointV1,
    #[pyo3(get)]
    pub(crate) end: PyRenderPointV1,
    #[pyo3(get)]
    pub(crate) width: f64,
    #[pyo3(get)]
    pub(crate) paint: PyRenderPaintV3,
    #[pyo3(get)]
    pub(crate) z: i32,
}

#[pyclass(frozen, name = "MaskOpV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyMaskOpV1 {
    #[pyo3(get)]
    pub(crate) origin: PyRenderPointV1,
    #[pyo3(get)]
    pub(crate) width: f64,
    #[pyo3(get)]
    pub(crate) height: f64,
    #[pyo3(get)]
    pub(crate) paint: PyRenderPaintV3,
    #[pyo3(get)]
    pub(crate) z: i32,
}

#[pyclass(frozen, name = "EllipseOpV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyEllipseOpV1 {
    #[pyo3(get)]
    pub(crate) center: PyRenderPointV1,
    #[pyo3(get)]
    pub(crate) radius_x: f64,
    #[pyo3(get)]
    pub(crate) radius_y: f64,
    #[pyo3(get)]
    pub(crate) rotation_degrees: f64,
    #[pyo3(get)]
    pub(crate) stroke_width: Option<f64>,
    #[pyo3(get)]
    pub(crate) stroke_paint: Option<PyRenderPaintV3>,
    #[pyo3(get)]
    pub(crate) fill_paint: Option<PyRenderPaintV3>,
    #[pyo3(get)]
    pub(crate) z: i32,
}

#[pyclass(frozen, name = "PathOpV3", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPathOpV3 {
    commands: Vec<PyScenePathCommandV3>,
    #[pyo3(get)]
    pub(crate) stroke_width: Option<f64>,
    #[pyo3(get)]
    pub(crate) stroke_paint: Option<PyRenderPaintV3>,
    #[pyo3(get)]
    pub(crate) stroke_line_cap: Option<String>,
    #[pyo3(get)]
    pub(crate) fill_paint: Option<PyRenderPaintV3>,
    #[pyo3(get)]
    pub(crate) z: i32,
}

#[pyclass(frozen, name = "ScenePathCommandV3", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyScenePathCommandV3 {
    #[pyo3(get)]
    pub(crate) kind: String,
    #[pyo3(get)]
    pub(crate) point: Option<PyRenderPointV1>,
    #[pyo3(get)]
    pub(crate) control_1: Option<PyRenderPointV1>,
    #[pyo3(get)]
    pub(crate) control_2: Option<PyRenderPointV1>,
}

#[pymethods]
impl PyPathOpV3 {
    #[getter]
    fn commands(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.commands)
    }
}

#[pyclass(frozen, name = "RenderPaintV3", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderPaintV3 {
    #[pyo3(get)]
    pub(crate) kind: String,
    #[pyo3(get)]
    pub(crate) export_rgb: String,
    #[pyo3(get)]
    pub(crate) role: Option<String>,
    #[pyo3(get)]
    pub(crate) element: Option<String>,
}

#[pyclass(frozen, name = "RenderOperationV3", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderOperationV3 {
    #[pyo3(get)]
    pub(crate) kind: String,
    operation: PyRenderOperationPayload,
}

#[derive(Clone)]
enum PyRenderOperationPayload {
    Text(PyTextOpV1),
    Line(PyLineOpV1),
    DoubleBondCarrierMark(PyLineOpV1),
    Mask(PyMaskOpV1),
    Ellipse(PyEllipseOpV1),
    Path(PyPathOpV3),
}

#[pymethods]
impl PyRenderOperationV3 {
    #[getter]
    fn operation(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match &self.operation {
            PyRenderOperationPayload::Text(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderOperationPayload::Line(value)
            | PyRenderOperationPayload::DoubleBondCarrierMark(value) => {
                Ok(Py::new(py, value.clone())?.into_any())
            }
            PyRenderOperationPayload::Mask(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderOperationPayload::Ellipse(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderOperationPayload::Path(value) => Ok(Py::new(py, value.clone())?.into_any()),
        }
    }
}

pub(crate) fn paint_from(value: &ferrum_render::RenderPaintV3) -> PyRenderPaintV3 {
    use ferrum_render::{DocumentContentPaintRoleV1, RenderPaintV3};
    let export_rgb = value.export_rgb().as_str().to_owned();
    match value {
        RenderPaintV3::AuthoredRgb24 { .. } => PyRenderPaintV3 {
            kind: "authored_rgb24".to_owned(),
            export_rgb,
            role: None,
            element: None,
        },
        RenderPaintV3::ThemeRole { role } => PyRenderPaintV3 {
            kind: "theme_role".to_owned(),
            export_rgb,
            role: Some(
                match role {
                    DocumentContentPaintRoleV1::DocumentForeground => "document_foreground",
                    DocumentContentPaintRoleV1::AtomNumber => "atom_number",
                }
                .to_owned(),
            ),
            element: None,
        },
        RenderPaintV3::ElementRole { element } => PyRenderPaintV3 {
            kind: "element_role".to_owned(),
            export_rgb,
            role: None,
            element: Some(element.as_str().to_owned()),
        },
    }
}

pub(crate) fn operation_from(_py: Python<'_>, value: &RenderOp) -> PyResult<PyRenderOperationV3> {
    let (kind, operation) = match value {
        RenderOp::Text(text) => ("text", PyRenderOperationPayload::Text(text_from(text))),
        RenderOp::Line(line) => ("line", PyRenderOperationPayload::Line(line_from(line))),
        RenderOp::DoubleBondCarrierMark(mark) => (
            "double_bond_carrier_mark",
            PyRenderOperationPayload::DoubleBondCarrierMark(line_from(&mark.accent_line())),
        ),
        RenderOp::Mask(mask) => (
            "mask",
            PyRenderOperationPayload::Mask(PyMaskOpV1 {
                origin: mask.origin().into(),
                width: mask.width().get(),
                height: mask.height().get(),
                paint: paint_from(mask.paint()),
                z: mask.z(),
            }),
        ),
        RenderOp::Ellipse(ellipse) => (
            "ellipse",
            PyRenderOperationPayload::Ellipse(ellipse_from(ellipse)),
        ),
        RenderOp::Path(path) => ("path", PyRenderOperationPayload::Path(path_from(path))),
    };
    Ok(PyRenderOperationV3 {
        kind: kind.to_owned(),
        operation,
    })
}

pub(crate) fn text_operation(value: PyTextOpV1) -> PyRenderOperationV3 {
    PyRenderOperationV3 {
        kind: "text".to_owned(),
        operation: PyRenderOperationPayload::Text(value),
    }
}

pub(crate) fn line_operation(value: PyLineOpV1) -> PyRenderOperationV3 {
    PyRenderOperationV3 {
        kind: "line".to_owned(),
        operation: PyRenderOperationPayload::Line(value),
    }
}

pub(crate) fn ellipse_operation(value: PyEllipseOpV1) -> PyRenderOperationV3 {
    PyRenderOperationV3 {
        kind: "ellipse".to_owned(),
        operation: PyRenderOperationPayload::Ellipse(value),
    }
}

pub(crate) fn path_operation(value: PyPathOpV3) -> PyRenderOperationV3 {
    PyRenderOperationV3 {
        kind: "path".to_owned(),
        operation: PyRenderOperationPayload::Path(value),
    }
}

pub(crate) fn double_bond_carrier_mark_operation(value: PyLineOpV1) -> PyRenderOperationV3 {
    PyRenderOperationV3 {
        kind: "double_bond_carrier_mark".to_owned(),
        operation: PyRenderOperationPayload::DoubleBondCarrierMark(value),
    }
}

pub(crate) fn mask_operation(value: PyMaskOpV1) -> PyRenderOperationV3 {
    PyRenderOperationV3 {
        kind: "mask".to_owned(),
        operation: PyRenderOperationPayload::Mask(value),
    }
}

pub(crate) fn line_from(line: &LineOp) -> PyLineOpV1 {
    PyLineOpV1 {
        start: line.start().into(),
        end: line.end().into(),
        width: line.width().get(),
        paint: paint_from(line.paint()),
        z: line.z(),
    }
}

pub(crate) fn ellipse_from(ellipse: &EllipseOp) -> PyEllipseOpV1 {
    PyEllipseOpV1 {
        center: ellipse.center().into(),
        radius_x: ellipse.radius_x().get(),
        radius_y: ellipse.radius_y().get(),
        rotation_degrees: ellipse.rotation_degrees(),
        stroke_width: ellipse.stroke_width().map(|width| width.get()),
        stroke_paint: ellipse.stroke_paint().map(paint_from),
        fill_paint: ellipse.fill_paint().map(paint_from),
        z: ellipse.z(),
    }
}

pub(crate) fn text_from(text: &TextOp) -> PyTextOpV1 {
    PyTextOpV1 {
        origin: text.origin().into(),
        runs: text
            .runs()
            .iter()
            .map(|run| PyTextRunV1 {
                text: run.text().to_owned(),
                script: script_name(run.script()).to_owned(),
                origin: run.origin().into(),
                glyphs: run
                    .glyphs()
                    .iter()
                    .map(|glyph| PyGlyphPlacementV1 {
                        glyph_index: glyph.glyph_index(),
                        origin: glyph.origin().into(),
                    })
                    .collect(),
                scale: run.scale().get(),
            })
            .collect(),
        face: text.face().as_str().to_owned(),
        size: text.size().get(),
        paint: paint_from(text.paint()),
        z: text.z(),
    }
}

pub(crate) fn path_from(path: &ferrum_render::PathOpV3) -> PyPathOpV3 {
    use ferrum_render::ScenePathCommandV3;
    let commands = path
        .commands()
        .iter()
        .map(|command| match command {
            ScenePathCommandV3::MoveTo(point) => PyScenePathCommandV3 {
                kind: "move_to".to_owned(),
                point: Some((*point).into()),
                control_1: None,
                control_2: None,
            },
            ScenePathCommandV3::LineTo(point) => PyScenePathCommandV3 {
                kind: "line_to".to_owned(),
                point: Some((*point).into()),
                control_1: None,
                control_2: None,
            },
            ScenePathCommandV3::CubicTo {
                control_1,
                control_2,
                end,
            } => PyScenePathCommandV3 {
                kind: "cubic_to".to_owned(),
                point: Some((*end).into()),
                control_1: Some((*control_1).into()),
                control_2: Some((*control_2).into()),
            },
            ScenePathCommandV3::Close => PyScenePathCommandV3 {
                kind: "close".to_owned(),
                point: None,
                control_1: None,
                control_2: None,
            },
        })
        .collect();
    PyPathOpV3 {
        commands,
        stroke_width: path.stroke().map(|stroke| stroke.width().get()),
        stroke_paint: path.stroke().map(|stroke| paint_from(stroke.paint())),
        stroke_line_cap: path.stroke().map(|stroke| match stroke.line_cap() {
            VectorStrokeLineCapV1::Butt => "butt".to_owned(),
            VectorStrokeLineCapV1::Round => "round".to_owned(),
        }),
        fill_paint: path.fill().map(paint_from),
        z: path.z(),
    }
}

fn script_name(value: TextScript) -> &'static str {
    match value {
        TextScript::Baseline => "baseline",
        TextScript::Subscript => "subscript",
        TextScript::Superscript => "superscript",
    }
}

pub(crate) fn frozen_tuple<T>(py: Python<'_>, values: &[T]) -> PyResult<Py<PyTuple>>
where
    T: Clone + for<'a> IntoPyObject<'a>,
{
    Ok(PyTuple::new(py, values.iter().cloned())?.unbind())
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyRenderTargetV1>()?;
    module.add_class::<PyAtomLocalSpaceV1>()?;
    module.add_class::<PySceneSpaceV1>()?;
    module.add_class::<PyRenderOperationV3>()?;
    module.add_class::<PyRenderPaintV3>()?;
    module.add_class::<PyTextOpV1>()?;
    module.add_class::<PyTextRunV1>()?;
    module.add_class::<PyGlyphPlacementV1>()?;
    module.add_class::<PyLineOpV1>()?;
    module.add_class::<PyMaskOpV1>()?;
    module.add_class::<PyEllipseOpV1>()?;
    module.add_class::<PyPathOpV3>()?;
    module.add_class::<PyScenePathCommandV3>()?;
    module.add_class::<PyRenderPointV1>()
}
