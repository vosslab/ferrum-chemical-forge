//! Frozen Python DTOs for the API-owned final render observation.

use ferrum_core::{RecordId, RecordOrigin};
use ferrum_render::{
    BatchSpace, BondStyle, DepictionIssueV1, DepictionSuppressionV1, DocumentMoleculeRenderPlanV2,
    DocumentPlusRenderV1, MoleculeRenderPlan, Paint, PositiveFinite, RENDER_OBSERVATION_SCHEMA_V1,
    RenderBatch, RenderDisplayLayerV1, RenderIssue, RenderIssueKind,
    RenderObservationError as ApiRenderObservationError, RenderObservationV1, RenderOp,
    RenderPoint, RenderTarget, Rgb24, TextOp, TextScript, VectorStrokeLineCapV1,
    build_directed_bond_preview_ops, verified_telex_regular_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::types::PyTuple;

use super::binding::PyDocumentBondPresentationV1;
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

#[pyclass(frozen, name = "RenderRecordIdV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderRecordIdV1 {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    id: Option<String>,
}

impl From<&RecordId> for PyRenderRecordIdV1 {
    fn from(value: &RecordId) -> Self {
        let id = match value.origin() {
            RecordOrigin::Source(identifier) => Some(identifier.as_str().to_owned()),
            RecordOrigin::Legacy { .. } => None,
        };
        Self {
            kind: format!("{:?}", value.kind()),
            id,
        }
    }
}

#[pyclass(frozen, name = "RenderTargetV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderTargetV1 {
    #[pyo3(get)]
    record_id: PyRenderRecordIdV1,
    #[pyo3(get)]
    source_order: u32,
}

impl From<&RenderTarget> for PyRenderTargetV1 {
    fn from(value: &RenderTarget) -> Self {
        Self {
            record_id: value.record_id().into(),
            source_order: value.source_order(),
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
    paint: String,
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
    paint: String,
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
    paint: String,
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
    stroke_paint: Option<String>,
    #[pyo3(get)]
    fill_paint: Option<String>,
    #[pyo3(get)]
    z: i32,
}

/// Frozen source-owned V2 scene path.  Qt receives only these commands and paint.
#[pyclass(frozen, name = "PathOpV2", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPathOpV2 {
    commands: Vec<PyScenePathCommandV2>,
    #[pyo3(get)]
    stroke_width: Option<f64>,
    #[pyo3(get)]
    stroke_paint: Option<String>,
    #[pyo3(get)]
    stroke_line_cap: Option<String>,
    #[pyo3(get)]
    fill_paint: Option<String>,
    #[pyo3(get)]
    z: i32,
}

#[pyclass(frozen, name = "ScenePathCommandV2", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyScenePathCommandV2 {
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
impl PyPathOpV2 {
    #[getter]
    fn commands(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.commands)
    }
}

#[pyclass(frozen, name = "RenderOperationV2", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderOperationV2 {
    #[pyo3(get)]
    kind: String,
    operation: PyRenderOperationPayload,
}

#[derive(Clone)]
enum PyRenderOperationPayload {
    Text(PyTextOpV1),
    Line(PyLineOpV1),
    Mask(PyMaskOpV1),
    Ellipse(PyEllipseOpV1),
    Path(PyPathOpV2),
}

#[pymethods]
impl PyRenderOperationV2 {
    #[getter]
    fn operation(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match &self.operation {
            PyRenderOperationPayload::Text(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderOperationPayload::Line(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderOperationPayload::Mask(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderOperationPayload::Ellipse(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderOperationPayload::Path(value) => Ok(Py::new(py, value.clone())?.into_any()),
        }
    }
}

#[pyclass(frozen, name = "RenderBatchV2", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderBatchV2 {
    #[pyo3(get)]
    target: PyRenderTargetV1,
    coordinate_space: PyRenderCoordinateSpace,
    #[pyo3(get)]
    display_layer: String,
    operations: Vec<PyRenderOperationV2>,
}

#[derive(Clone)]
enum PyRenderCoordinateSpace {
    AtomLocal(PyAtomLocalSpaceV1),
    Scene(PySceneSpaceV1),
}

#[pymethods]
impl PyRenderBatchV2 {
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

#[pyclass(frozen, name = "RenderPlanV2", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderPlanV2 {
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    provenance: PyRenderProvenanceV1,
    batches: Vec<PyRenderBatchV2>,
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
    id: Option<String>,
    #[pyo3(get)]
    projection_key: String,
    #[pyo3(get)]
    source_id: Option<String>,
    #[pyo3(get)]
    source_order: u32,
}

#[pyclass(frozen, name = "DocumentMoleculeRenderPlanV2", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyDocumentMoleculeRenderPlanV2 {
    #[pyo3(get)]
    molecule: PyMoleculeRenderRootV1,
    #[pyo3(get)]
    plan: PyRenderPlanV2,
}

#[pymethods]
impl PyRenderPlanV2 {
    #[getter]
    fn batches(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.batches)
    }
    #[getter]
    fn issues(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.issues)
    }
}

#[pyclass(frozen, name = "DepictionIssueV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyDepictionIssueV1 {
    #[pyo3(get)]
    code: String,
    #[pyo3(get)]
    target: String,
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
    target: super::projection_binding::PyPresentationTargetV1,
    #[pyo3(get)]
    anchor: PyRenderPointV1,
    #[pyo3(get)]
    operation: PyTextOpV1,
    #[pyo3(get)]
    bounds: PyPresentationTextBoundsV1,
    #[pyo3(get)]
    background: Option<String>,
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
    molecule_plans: Vec<PyDocumentMoleculeRenderPlanV2>,
    plus_renders: Vec<PyDocumentPlusRenderV1>,
    text_renders: Vec<super::presentation_text_render_binding::PyDocumentTextRenderV1>,
    issues: Vec<PyDepictionIssueV1>,
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
    #[getter]
    fn issues(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.issues)
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

/// Return source-owned V2 operations for one disposable directed-bond preview.
#[pyfunction]
pub(crate) fn native_directed_bond_preview_v1(
    py: Python<'_>,
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    presentation: PyRef<'_, PyDocumentBondPresentationV1>,
) -> PyResult<Py<PyTuple>> {
    let style = match *presentation {
        PyDocumentBondPresentationV1::SolidWedge => BondStyle::SolidWedge,
        PyDocumentBondPresentationV1::HashedWedge => BondStyle::HashedWedge,
        _ => {
            return Err(RenderDepictionError::new_err(
                "choose a directed wedge presentation for a directed bond preview",
            ));
        }
    };
    let start = RenderPoint::new(start_x, start_y)
        .map_err(|error| RenderDepictionError::new_err(error.to_string()))?;
    let end = RenderPoint::new(end_x, end_y)
        .map_err(|error| RenderDepictionError::new_err(error.to_string()))?;
    let width = PositiveFinite::new(1.0)
        .map_err(|error| RenderDepictionError::new_err(error.to_string()))?;
    let wedge_width = PositiveFinite::new(5.0)
        .map_err(|error| RenderDepictionError::new_err(error.to_string()))?;
    let paint = Paint::rgb24(
        Rgb24::new("000000").map_err(|error| RenderDepictionError::new_err(error.to_string()))?,
    );
    let operations = build_directed_bond_preview_ops(style, start, end, width, wedge_width, paint)
        .map_err(|error| RenderDepictionError::new_err(error.to_string()))?;
    let values = operations
        .iter()
        .map(|operation| operation_from(py, operation))
        .collect::<PyResult<Vec<_>>>()?;
    frozen_tuple(py, &values)
}

pub(crate) fn observation(
    py: Python<'_>,
    value: RenderObservationV1,
) -> PyResult<PyRenderObservationV1> {
    let molecule_plans = value
        .molecule_plans()
        .iter()
        .map(|entry| document_molecule_plan_from(py, entry))
        .collect::<PyResult<_>>()?;
    Ok(PyRenderObservationV1 {
        schema: RENDER_OBSERVATION_SCHEMA_V1.to_owned(),
        document: value.document().clone().into(),
        profile: value.profile().schema().to_owned(),
        molecule_plans,
        plus_renders: value.plus_renders().iter().map(plus_from).collect(),
        text_renders: value.text_renders().iter().map(Into::into).collect(),
        issues: value.issues().iter().map(issue_from).collect(),
        suppression: value.suppression().map(suppression_name),
    })
}

pub(crate) fn result(
    py: Python<'_>,
    value: Result<RenderObservationV1, ApiRenderObservationError>,
) -> PyResult<PyRenderObservationV1> {
    match value {
        Ok(render_observation) => observation(py, render_observation),
        Err(error) => Err(error_result(py, error)?),
    }
}

pub(crate) fn error_result(py: Python<'_>, error: ApiRenderObservationError) -> PyResult<PyErr> {
    match error {
        ApiRenderObservationError::Document(error) => map_document_error(py, error),
        ApiRenderObservationError::Depiction(error) => {
            Ok(RenderDepictionError::new_err(error.to_string()))
        }
        ApiRenderObservationError::ProvenanceMismatch => Ok(RenderProvenanceError::new_err(
            "render observation provenance did not match its authoritative document",
        )),
        ApiRenderObservationError::MoleculeRootMismatch => Ok(RenderProvenanceError::new_err(
            "render molecule roots did not match the authoritative document projection",
        )),
        ApiRenderObservationError::PlusRootMismatch => Ok(RenderProvenanceError::new_err(
            "render plus roots did not match the authoritative document projection",
        )),
        ApiRenderObservationError::TextRootMismatch => Ok(RenderProvenanceError::new_err(
            "render Text roots did not match the authoritative document projection",
        )),
    }
}

fn document_molecule_plan_from(
    py: Python<'_>,
    value: &DocumentMoleculeRenderPlanV2,
) -> PyResult<PyDocumentMoleculeRenderPlanV2> {
    Ok(PyDocumentMoleculeRenderPlanV2 {
        molecule: PyMoleculeRenderRootV1 {
            id: value.molecule().id().map(str::to_owned),
            projection_key: value.molecule().projection_key().to_owned(),
            source_id: value.molecule().source_id().map(str::to_owned),
            source_order: value.molecule().source_order(),
        },
        plan: plan_from(py, value.plan())?,
    })
}

pub(crate) fn plan_from(py: Python<'_>, plan: &MoleculeRenderPlan) -> PyResult<PyRenderPlanV2> {
    Ok(PyRenderPlanV2 {
        schema: "ferrum-render-plan-v2".to_owned(),
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

fn batch_from(py: Python<'_>, batch: &RenderBatch) -> PyResult<PyRenderBatchV2> {
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
    Ok(PyRenderBatchV2 {
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

pub(crate) fn operation_from(_py: Python<'_>, value: &RenderOp) -> PyResult<PyRenderOperationV2> {
    let (kind, operation) = match value {
        RenderOp::Text(text) => ("text", PyRenderOperationPayload::Text(text_from(text))),
        RenderOp::Line(line) => (
            "line",
            PyRenderOperationPayload::Line(PyLineOpV1 {
                start: line.start().into(),
                end: line.end().into(),
                width: line.width().get(),
                paint: line.paint().color().as_str().to_owned(),
                z: line.z(),
            }),
        ),
        RenderOp::Mask(mask) => (
            "mask",
            PyRenderOperationPayload::Mask(PyMaskOpV1 {
                origin: mask.origin().into(),
                width: mask.width().get(),
                height: mask.height().get(),
                paint: mask.paint().color().as_str().to_owned(),
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
                stroke_paint: ellipse
                    .stroke_paint()
                    .map(|paint| paint.color().as_str().to_owned()),
                fill_paint: ellipse
                    .fill_paint()
                    .map(|paint| paint.color().as_str().to_owned()),
                z: ellipse.z(),
            }),
        ),
        RenderOp::Path(path) => ("path", PyRenderOperationPayload::Path(path_from(path))),
    };
    Ok(PyRenderOperationV2 {
        kind: kind.to_owned(),
        operation,
    })
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
        paint: text.paint().color().as_str().to_owned(),
        z: text.z(),
    }
}

fn path_from(path: &ferrum_render::PathOpV2) -> PyPathOpV2 {
    use ferrum_render::ScenePathCommandV2;
    let commands = path
        .commands()
        .iter()
        .map(|command| match command {
            ScenePathCommandV2::MoveTo(point) => PyScenePathCommandV2 {
                kind: "move_to".to_owned(),
                point: Some((*point).into()),
                control_1: None,
                control_2: None,
            },
            ScenePathCommandV2::LineTo(point) => PyScenePathCommandV2 {
                kind: "line_to".to_owned(),
                point: Some((*point).into()),
                control_1: None,
                control_2: None,
            },
            ScenePathCommandV2::CubicTo {
                control_1,
                control_2,
                end,
            } => PyScenePathCommandV2 {
                kind: "cubic_to".to_owned(),
                point: Some((*end).into()),
                control_1: Some((*control_1).into()),
                control_2: Some((*control_2).into()),
            },
            ScenePathCommandV2::Close => PyScenePathCommandV2 {
                kind: "close".to_owned(),
                point: None,
                control_1: None,
                control_2: None,
            },
        })
        .collect();
    PyPathOpV2 {
        commands,
        stroke_width: path.stroke().map(|stroke| stroke.width().get()),
        stroke_paint: path
            .stroke()
            .map(|stroke| stroke.paint().color().as_str().to_owned()),
        stroke_line_cap: path.stroke().map(|stroke| match stroke.line_cap() {
            VectorStrokeLineCapV1::Butt => "butt".to_owned(),
            VectorStrokeLineCapV1::Round => "round".to_owned(),
        }),
        fill_paint: path.fill().map(|paint| paint.color().as_str().to_owned()),
        z: path.z(),
    }
}

fn plus_from(value: &DocumentPlusRenderV1) -> PyDocumentPlusRenderV1 {
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
        background: value
            .background()
            .map(|paint| paint.color().as_str().to_owned()),
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

fn issue_from(value: &DepictionIssueV1) -> PyDepictionIssueV1 {
    PyDepictionIssueV1 {
        code: format!("{:?}", value.code()).to_ascii_lowercase(),
        target: value.target().to_owned(),
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
    module.add_function(wrap_pyfunction!(native_directed_bond_preview_v1, module)?)?;
    module.add_class::<PyRenderObservationV1>()?;
    module.add_class::<PyMoleculeRenderRootV1>()?;
    module.add_class::<PyDocumentMoleculeRenderPlanV2>()?;
    module.add_class::<PyRenderPlanV2>()?;
    module.add_class::<PyRenderProvenanceV1>()?;
    module.add_class::<PyRenderBatchV2>()?;
    module.add_class::<PyRenderTargetV1>()?;
    module.add_class::<PyRenderRecordIdV1>()?;
    module.add_class::<PyAtomLocalSpaceV1>()?;
    module.add_class::<PySceneSpaceV1>()?;
    module.add_class::<PyRenderOperationV2>()?;
    module.add_class::<PyTextOpV1>()?;
    module.add_class::<PyTextRunV1>()?;
    module.add_class::<PyGlyphPlacementV1>()?;
    module.add_class::<PyLineOpV1>()?;
    module.add_class::<PyMaskOpV1>()?;
    module.add_class::<PyEllipseOpV1>()?;
    module.add_class::<PyPathOpV2>()?;
    module.add_class::<PyScenePathCommandV2>()?;
    module.add_class::<PyRenderIssueV1>()?;
    module.add_class::<PyDepictionIssueV1>()?;
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
