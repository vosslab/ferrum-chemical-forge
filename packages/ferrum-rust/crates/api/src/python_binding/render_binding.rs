//! Frozen Python DTOs for the API-owned final render observation.

use ferrum_document::{
    DOCUMENT_RENDER_OBSERVATION_SCHEMA_V1, DocumentRenderObservationErrorV1,
    DocumentRenderObservationV1, PresentationTargetV1,
};
use ferrum_render::{
    BatchSpace, DepictionSuppressionV1, DocumentMoleculeRenderPlanV3, DocumentPlusRenderV1, LineOp,
    MoleculeRenderPlan, RenderBatch, RenderDisplayLayerV1, RenderIssue, RenderIssueKind, RenderOp,
    RenderPoint, RenderTarget, TextOp, TextScript, VectorStrokeLineCapV1,
    verified_telex_regular_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::types::PyTuple;

use super::binding::{FerrumError, map_document_error};

create_exception!(ferrum_chem, RenderObservationError, FerrumError);
create_exception!(ferrum_chem, RenderDepictionError, RenderObservationError);
create_exception!(ferrum_chem, RenderProvenanceError, RenderObservationError);

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
    kind: String,
    #[pyo3(get)]
    document_object_id: String,
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
    kind: String,
    #[pyo3(get)]
    anchor: PyRenderPointV1,
}

#[pyclass(frozen, name = "SceneSpaceV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PySceneSpaceV1 {
    #[pyo3(get)]
    kind: String,
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
    origin: PyRenderPointV1,
    runs: Vec<PyTextRunV1>,
    #[pyo3(get)]
    face: String,
    #[pyo3(get)]
    size: f64,
    #[pyo3(get)]
    paint: PyRenderPaintV3,
    #[pyo3(get)]
    z: i32,
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
    start: PyRenderPointV1,
    #[pyo3(get)]
    end: PyRenderPointV1,
    #[pyo3(get)]
    width: f64,
    #[pyo3(get)]
    paint: PyRenderPaintV3,
    #[pyo3(get)]
    z: i32,
}

#[pyclass(frozen, name = "MaskOpV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyMaskOpV1 {
    #[pyo3(get)]
    origin: PyRenderPointV1,
    #[pyo3(get)]
    width: f64,
    #[pyo3(get)]
    height: f64,
    #[pyo3(get)]
    paint: PyRenderPaintV3,
    #[pyo3(get)]
    z: i32,
}

#[pyclass(frozen, name = "EllipseOpV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyEllipseOpV1 {
    #[pyo3(get)]
    center: PyRenderPointV1,
    #[pyo3(get)]
    radius_x: f64,
    #[pyo3(get)]
    radius_y: f64,
    #[pyo3(get)]
    rotation_degrees: f64,
    #[pyo3(get)]
    stroke_width: Option<f64>,
    #[pyo3(get)]
    stroke_paint: Option<PyRenderPaintV3>,
    #[pyo3(get)]
    fill_paint: Option<PyRenderPaintV3>,
    #[pyo3(get)]
    z: i32,
}

/// Frozen source-owned V3 scene path.  Qt receives only these commands and paint.
#[pyclass(frozen, name = "PathOpV3", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPathOpV3 {
    commands: Vec<PyScenePathCommandV3>,
    #[pyo3(get)]
    stroke_width: Option<f64>,
    #[pyo3(get)]
    stroke_paint: Option<PyRenderPaintV3>,
    #[pyo3(get)]
    stroke_line_cap: Option<String>,
    #[pyo3(get)]
    fill_paint: Option<PyRenderPaintV3>,
    #[pyo3(get)]
    z: i32,
}

#[pyclass(frozen, name = "ScenePathCommandV3", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyScenePathCommandV3 {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    point: Option<PyRenderPointV1>,
    #[pyo3(get)]
    control_1: Option<PyRenderPointV1>,
    #[pyo3(get)]
    control_2: Option<PyRenderPointV1>,
}

#[pymethods]
impl PyPathOpV3 {
    #[getter]
    fn commands(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.commands)
    }
}

/// Frozen tagged V3 paint facts copied from Rust-owned render operations.
#[pyclass(frozen, name = "RenderPaintV3", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderPaintV3 {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    export_rgb: String,
    #[pyo3(get)]
    role: Option<String>,
    #[pyo3(get)]
    element: Option<String>,
}

#[pyclass(frozen, name = "RenderOperationV3", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderOperationV3 {
    #[pyo3(get)]
    kind: String,
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
            PyRenderOperationPayload::Line(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderOperationPayload::DoubleBondCarrierMark(value) => {
                Ok(Py::new(py, value.clone())?.into_any())
            }
            PyRenderOperationPayload::Mask(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderOperationPayload::Ellipse(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderOperationPayload::Path(value) => Ok(Py::new(py, value.clone())?.into_any()),
        }
    }
}

#[pyclass(frozen, name = "RenderBatchV3", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderBatchV3 {
    #[pyo3(get)]
    target: PyRenderTargetV1,
    coordinate_space: PyRenderCoordinateSpace,
    #[pyo3(get)]
    display_layer: String,
    operations: Vec<PyRenderOperationV3>,
}

#[derive(Clone)]
enum PyRenderCoordinateSpace {
    AtomLocal(PyAtomLocalSpaceV1),
    Scene(PySceneSpaceV1),
}

#[pymethods]
impl PyRenderBatchV3 {
    #[getter]
    fn coordinate_space(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match &self.coordinate_space {
            PyRenderCoordinateSpace::AtomLocal(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderCoordinateSpace::Scene(value) => Ok(Py::new(py, value.clone())?.into_any()),
        }
    }
    #[getter]
    fn operations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.operations)
    }
}

#[pyclass(frozen, name = "RenderIssueV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderIssueV1 {
    #[pyo3(get)]
    target: PyRenderTargetV1,
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    detail: String,
}

#[pyclass(frozen, name = "RenderPlanV3", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderPlanV3 {
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    provenance: PyRenderProvenanceV1,
    batches: Vec<PyRenderBatchV3>,
    issues: Vec<PyRenderIssueV1>,
}

#[pyclass(frozen, name = "RenderProvenanceV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderProvenanceV1 {
    #[pyo3(get)]
    revision: u64,
    #[pyo3(get)]
    digest: String,
}

#[pyclass(frozen, name = "MoleculeRenderRootV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyMoleculeRenderRootV1 {
    #[pyo3(get)]
    document_object_id: String,
}

#[pyclass(frozen, name = "DocumentMoleculeRenderPlanV3", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyDocumentMoleculeRenderPlanV3 {
    #[pyo3(get)]
    molecule: PyMoleculeRenderRootV1,
    #[pyo3(get)]
    plan: PyRenderPlanV3,
    member_issues: Vec<PyMoleculeMemberDepictionIssueV1>,
}

#[pymethods]
impl PyRenderPlanV3 {
    #[getter]
    fn batches(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.batches)
    }
    #[getter]
    fn issues(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.issues)
    }
}

#[pymethods]
impl PyDocumentMoleculeRenderPlanV3 {
    #[getter]
    fn member_issues(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.member_issues)
    }
}

#[pyclass(frozen, name = "MoleculeMemberDepictionIssueV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyMoleculeMemberDepictionIssueV1 {
    #[pyo3(get)]
    document_object_id: String,
    #[pyo3(get)]
    category: String,
    #[pyo3(get)]
    detail: String,
}

#[pyclass(frozen, name = "PresentationTextBoundsV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationTextBoundsV1 {
    #[pyo3(get)]
    pub(crate) left: f64,
    #[pyo3(get)]
    pub(crate) top: f64,
    #[pyo3(get)]
    pub(crate) right: f64,
    #[pyo3(get)]
    pub(crate) bottom: f64,
}

#[pyclass(frozen, name = "DocumentPlusRenderV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyDocumentPlusRenderV1 {
    #[pyo3(get)]
    target: PyRenderTargetV1,
    #[pyo3(get)]
    anchor: PyRenderPointV1,
    #[pyo3(get)]
    operation: PyTextOpV1,
    #[pyo3(get)]
    bounds: PyPresentationTextBoundsV1,
    #[pyo3(get)]
    background: Option<PyRenderPaintV3>,
}

#[pyclass(frozen, name = "RenderObservationV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderObservationV1 {
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    document: super::projection_binding::PySessionDocumentObservationV1,
    #[pyo3(get)]
    profile: String,
    molecule_plans: Vec<PyDocumentMoleculeRenderPlanV3>,
    plus_renders: Vec<PyDocumentPlusRenderV1>,
    text_renders: Vec<super::presentation_text_render_binding::PyDocumentTextRenderV1>,
    #[pyo3(get)]
    suppression: Option<String>,
}

#[pymethods]
impl PyRenderObservationV1 {
    #[getter]
    fn molecule_plans(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.molecule_plans)
    }
    #[getter]
    fn plus_renders(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.plus_renders)
    }
    #[getter]
    fn text_renders(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.text_renders)
    }
}

#[pyclass(frozen, name = "VerifiedTelexRegularV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyVerifiedTelexRegularV1 {
    #[pyo3(get)]
    resource_id: String,
    data: Vec<u8>,
    #[pyo3(get)]
    byte_length: u64,
    #[pyo3(get)]
    sha256: String,
    #[pyo3(get)]
    family: String,
    #[pyo3(get)]
    postscript_name: String,
}

#[pymethods]
impl PyVerifiedTelexRegularV1 {
    #[getter]
    fn data(&self, py: Python<'_>) -> Py<PyBytes> {
        PyBytes::new(py, &self.data).unbind()
    }
}

#[pyfunction]
pub(crate) fn verified_telex_regular() -> PyResult<PyVerifiedTelexRegularV1> {
    let resource = verified_telex_regular_v1()
        .map_err(|error| RenderDepictionError::new_err(error.to_string()))?;
    Ok(PyVerifiedTelexRegularV1 {
        resource_id: resource.resource_id().to_owned(),
        data: resource.bytes().to_vec(),
        byte_length: resource.byte_length(),
        sha256: resource.sha256().to_owned(),
        family: resource.family().to_owned(),
        postscript_name: resource.postscript_name().to_owned(),
    })
}

pub(crate) fn observation(
    py: Python<'_>,
    value: DocumentRenderObservationV1,
) -> PyResult<PyRenderObservationV1> {
    let resolved = value.resolved();
    let molecule_plans = resolved
        .molecule_plans()
        .iter()
        .map(|entry| document_molecule_plan_from(py, entry))
        .collect::<PyResult<_>>()?;
    Ok(PyRenderObservationV1 {
        schema: DOCUMENT_RENDER_OBSERVATION_SCHEMA_V1.to_owned(),
        document: value.document().clone().into(),
        profile: resolved.profile().schema().to_owned(),
        molecule_plans,
        plus_renders: resolved.plus_renders().iter().map(plus_from).collect(),
        text_renders: resolved.text_renders().iter().map(Into::into).collect(),
        suppression: resolved.suppression().map(suppression_name),
    })
}

pub(crate) fn error_result(
    py: Python<'_>,
    error: DocumentRenderObservationErrorV1,
) -> PyResult<PyErr> {
    match error {
        DocumentRenderObservationErrorV1::Document(error) => map_document_error(py, error),
        DocumentRenderObservationErrorV1::Render(error) => {
            Ok(RenderDepictionError::new_err(error.to_string()))
        }
        DocumentRenderObservationErrorV1::StereoDepiction(error) => {
            Ok(RenderDepictionError::new_err(error.to_string()))
        }
        DocumentRenderObservationErrorV1::StereoProjection(error) => {
            Ok(RenderDepictionError::new_err(error.to_string()))
        }
        DocumentRenderObservationErrorV1::Projection(error) => {
            Ok(RenderDepictionError::new_err(error.to_string()))
        }
        DocumentRenderObservationErrorV1::ProjectionMismatch => Ok(RenderProvenanceError::new_err(
            "render observation projection identity did not match its authoritative document",
        )),
        DocumentRenderObservationErrorV1::ProvenanceMismatch => Ok(RenderProvenanceError::new_err(
            "render observation provenance did not match its authoritative document",
        )),
    }
}

fn document_molecule_plan_from(
    py: Python<'_>,
    value: &DocumentMoleculeRenderPlanV3,
) -> PyResult<PyDocumentMoleculeRenderPlanV3> {
    Ok(PyDocumentMoleculeRenderPlanV3 {
        molecule: PyMoleculeRenderRootV1 {
            document_object_id: value.molecule().document_object_id().as_str().to_owned(),
        },
        plan: plan_from(py, value.plan())?,
        member_issues: value
            .member_issues()
            .iter()
            .map(member_issue_from)
            .collect(),
    })
}

pub(crate) fn plan_from(py: Python<'_>, plan: &MoleculeRenderPlan) -> PyResult<PyRenderPlanV3> {
    Ok(PyRenderPlanV3 {
        schema: "ferrum-render-plan-v3".to_owned(),
        provenance: PyRenderProvenanceV1 {
            revision: plan.revision().get(),
            digest: hex_digest(&plan.provenance().digest()),
        },
        batches: plan
            .batches()
            .iter()
            .map(|batch| batch_from(py, batch))
            .collect::<PyResult<_>>()?,
        issues: plan.issues().iter().map(issue_render_from).collect(),
    })
}

fn batch_from(py: Python<'_>, batch: &RenderBatch) -> PyResult<PyRenderBatchV3> {
    let coordinate_space = match batch.coordinate_space() {
        BatchSpace::AtomLocal { anchor } => {
            PyRenderCoordinateSpace::AtomLocal(PyAtomLocalSpaceV1 {
                kind: "atom_local".to_owned(),
                anchor: (*anchor).into(),
            })
        }
        BatchSpace::Scene => PyRenderCoordinateSpace::Scene(PySceneSpaceV1 {
            kind: "scene".to_owned(),
        }),
    };
    Ok(PyRenderBatchV3 {
        target: batch.target().into(),
        coordinate_space,
        display_layer: match batch.display_layer() {
            RenderDisplayLayerV1::Ordinary => "ordinary".to_owned(),
            RenderDisplayLayerV1::HaworthFrontStroke => "haworth_front_stroke".to_owned(),
            RenderDisplayLayerV1::HaworthFrontWedge => "haworth_front_wedge".to_owned(),
        },
        operations: batch
            .operations()
            .iter()
            .map(|operation| operation_from(py, operation))
            .collect::<PyResult<_>>()?,
    })
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
            PyRenderOperationPayload::Ellipse(PyEllipseOpV1 {
                center: ellipse.center().into(),
                radius_x: ellipse.radius_x().get(),
                radius_y: ellipse.radius_y().get(),
                rotation_degrees: ellipse.rotation_degrees(),
                stroke_width: ellipse.stroke_width().map(|width| width.get()),
                stroke_paint: ellipse.stroke_paint().map(paint_from),
                fill_paint: ellipse.fill_paint().map(paint_from),
                z: ellipse.z(),
            }),
        ),
        RenderOp::Path(path) => ("path", PyRenderOperationPayload::Path(path_from(path))),
    };
    Ok(PyRenderOperationV3 {
        kind: kind.to_owned(),
        operation,
    })
}

fn line_from(line: &LineOp) -> PyLineOpV1 {
    PyLineOpV1 {
        start: line.start().into(),
        end: line.end().into(),
        width: line.width().get(),
        paint: paint_from(line.paint()),
        z: line.z(),
    }
}

fn text_from(text: &TextOp) -> PyTextOpV1 {
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

fn path_from(path: &ferrum_render::PathOpV3) -> PyPathOpV3 {
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

pub(crate) fn plus_from(value: &DocumentPlusRenderV1) -> PyDocumentPlusRenderV1 {
    let bounds = value.bounds();
    PyDocumentPlusRenderV1 {
        target: value.target().into(),
        anchor: value.anchor().into(),
        operation: text_from(value.operation()),
        bounds: PyPresentationTextBoundsV1 {
            left: bounds.left(),
            top: bounds.top(),
            right: bounds.right(),
            bottom: bounds.bottom(),
        },
        background: value.background().map(paint_from),
    }
}

fn issue_render_from(value: &RenderIssue) -> PyRenderIssueV1 {
    let (kind, detail) = match value.kind() {
        RenderIssueKind::UnsupportedFeature { feature } => ("unsupported_feature", feature),
        RenderIssueKind::UnrenderableTarget { reason } => ("unrenderable_target", reason),
    };
    PyRenderIssueV1 {
        target: value.target().into(),
        kind: kind.to_owned(),
        detail: detail.to_owned(),
    }
}

fn member_issue_from(
    value: &ferrum_render::MoleculeMemberDepictionIssueV1,
) -> PyMoleculeMemberDepictionIssueV1 {
    PyMoleculeMemberDepictionIssueV1 {
        document_object_id: value.target().as_str().to_owned(),
        category: format!("{:?}", value.code()).to_ascii_lowercase(),
        detail: value.detail().to_owned(),
    }
}

fn script_name(value: TextScript) -> &'static str {
    match value {
        TextScript::Baseline => "baseline",
        TextScript::Subscript => "subscript",
        TextScript::Superscript => "superscript",
    }
}

fn suppression_name(value: DepictionSuppressionV1) -> String {
    format!("{:?}", value).to_ascii_lowercase()
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "RenderObservationError",
        module.py().get_type::<RenderObservationError>(),
    )?;
    module.add(
        "RenderDepictionError",
        module.py().get_type::<RenderDepictionError>(),
    )?;
    module.add(
        "RenderProvenanceError",
        module.py().get_type::<RenderProvenanceError>(),
    )?;
    module.add_function(wrap_pyfunction!(verified_telex_regular, module)?)?;
    module.add_class::<PyRenderObservationV1>()?;
    module.add_class::<PyMoleculeRenderRootV1>()?;
    module.add_class::<PyDocumentMoleculeRenderPlanV3>()?;
    module.add_class::<PyRenderPlanV3>()?;
    module.add_class::<PyRenderProvenanceV1>()?;
    module.add_class::<PyRenderBatchV3>()?;
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
    module.add_class::<PyRenderIssueV1>()?;
    module.add_class::<PyMoleculeMemberDepictionIssueV1>()?;
    module.add_class::<PyDocumentPlusRenderV1>()?;
    super::presentation_text_render_binding::register(module)?;
    module.add_class::<PyPresentationTextBoundsV1>()?;
    module.add_class::<PyRenderPointV1>()?;
    module.add_class::<PyVerifiedTelexRegularV1>()
}

pub(crate) fn frozen_tuple<T>(py: Python<'_>, values: &[T]) -> PyResult<Py<PyTuple>>
where
    T: Clone + for<'a> IntoPyObject<'a>,
{
    Ok(PyTuple::new(py, values.iter().cloned())?.unbind())
}

fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
