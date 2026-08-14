//! Frozen Python DTOs for the API-owned final render observation.

use ferrum_api::{
    BatchSpace, DepictionIssueV1, DepictionSuppressionV1, DocumentMoleculeRenderPlanV1,
    DocumentPlusRenderV1, RecordOrigin, RenderBatch, RenderIssue, RenderIssueKind,
    RenderObservationError as ApiRenderObservationError, RenderObservationV1, RenderOp,
    RenderPoint, RenderTarget, TextScript, verified_telex_regular_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::types::PyTuple;

use crate::binding::{FerrumError, map_document_error};

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

impl From<&ferrum_api::RecordId> for PyRenderRecordIdV1 {
    fn from(value: &ferrum_api::RecordId) -> Self {
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

#[pyclass(frozen, name = "RenderOperationV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderOperationV1 {
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
}

#[pymethods]
impl PyRenderOperationV1 {
    #[getter]
    fn operation(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match &self.operation {
            PyRenderOperationPayload::Text(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderOperationPayload::Line(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderOperationPayload::Mask(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderOperationPayload::Ellipse(value) => Ok(Py::new(py, value.clone())?.into_any()),
        }
    }
}

#[pyclass(frozen, name = "RenderBatchV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderBatchV1 {
    #[pyo3(get)]
    target: PyRenderTargetV1,
    coordinate_space: PyRenderCoordinateSpace,
    operations: Vec<PyRenderOperationV1>,
}

#[derive(Clone)]
enum PyRenderCoordinateSpace {
    AtomLocal(PyAtomLocalSpaceV1),
    Scene(PySceneSpaceV1),
}

#[pymethods]
impl PyRenderBatchV1 {
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

#[pyclass(frozen, name = "RenderPlanV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderPlanV1 {
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    provenance: PyRenderProvenanceV1,
    batches: Vec<PyRenderBatchV1>,
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

#[pyclass(frozen, name = "DocumentMoleculeRenderPlanV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyDocumentMoleculeRenderPlanV1 {
    #[pyo3(get)]
    molecule: PyMoleculeRenderRootV1,
    #[pyo3(get)]
    plan: PyRenderPlanV1,
}

#[pymethods]
impl PyRenderPlanV1 {
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
    target: crate::projection_binding::PyPresentationTargetV1,
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
    document: crate::projection_binding::PySessionDocumentObservationV1,
    #[pyo3(get)]
    profile: String,
    molecule_plans: Vec<PyDocumentMoleculeRenderPlanV1>,
    plus_renders: Vec<PyDocumentPlusRenderV1>,
    text_renders: Vec<crate::presentation_text_render_binding::PyDocumentTextRenderV1>,
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
        schema: ferrum_api::RENDER_OBSERVATION_SCHEMA_V1.to_owned(),
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
    value: &DocumentMoleculeRenderPlanV1,
) -> PyResult<PyDocumentMoleculeRenderPlanV1> {
    Ok(PyDocumentMoleculeRenderPlanV1 {
        molecule: PyMoleculeRenderRootV1 {
            id: value.molecule().id().map(str::to_owned),
            projection_key: value.molecule().projection_key().to_owned(),
            source_id: value.molecule().source_id().map(str::to_owned),
            source_order: value.molecule().source_order(),
        },
        plan: plan_from(py, value.plan())?,
    })
}

fn plan_from(py: Python<'_>, plan: &ferrum_api::MoleculeRenderPlan) -> PyResult<PyRenderPlanV1> {
    Ok(PyRenderPlanV1 {
        schema: "ferrum-render-plan-v1".to_owned(),
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

fn batch_from(py: Python<'_>, batch: &RenderBatch) -> PyResult<PyRenderBatchV1> {
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
    Ok(PyRenderBatchV1 {
        target: batch.target().into(),
        coordinate_space,
        operations: batch
            .operations()
            .iter()
            .map(|operation| operation_from(py, operation))
            .collect::<PyResult<_>>()?,
    })
}

fn operation_from(_py: Python<'_>, value: &RenderOp) -> PyResult<PyRenderOperationV1> {
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
    };
    Ok(PyRenderOperationV1 {
        kind: kind.to_owned(),
        operation,
    })
}

fn text_from(text: &ferrum_api::TextOp) -> PyTextOpV1 {
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

fn frozen_tuple<T>(py: Python<'_>, values: &[T]) -> PyResult<Py<PyTuple>>
where
    T: Clone + for<'a> IntoPyObject<'a>,
{
    Ok(PyTuple::new(py, values.iter().cloned())?.unbind())
}

fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
