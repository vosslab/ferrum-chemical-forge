//! Frozen V4 render-plan and V2 observation DTOs and conversions.

use ferrum_document::{DOCUMENT_RENDER_OBSERVATION_SCHEMA_V2, DocumentRenderObservationV2};
use ferrum_render::{
    AtomDecorationRenderOpV1, AtomLabelRenderV1, BatchSpace, BondRenderOpV1,
    CompactGroupRenderOpV1, DepictionSuppressionV1, DocumentMoleculeRenderPlanV4,
    DocumentPlusRenderV1, InkBoundsV1, MoleculeContentBoundsV1, MoleculeRenderPlanV4,
    RenderBatchContentV4, RenderBatchV4, RenderDisplayLayerV1, RenderIssue, RenderIssueKind,
    measure_molecule_render_plan_bounds_v1,
};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::render_plan_content_binding::{
    PyAtomDecorationRenderOpV1, PyAtomLabelRenderV1, PyAtomRenderBatchV1, PyBondRenderBatchV1,
    PyBondRenderOpV1, PyCompactGroupRenderBatchV1, PyCompactGroupRenderOpV1, PyInkBoundsV1,
    PyRenderBatchContentV4,
};
use super::render_primitive_binding::{
    PyAtomLocalSpaceV1, PyMaskOpV1, PyRenderPointV1, PyRenderTargetV1, PyTextOpV1, ellipse_from,
    frozen_tuple, line_from, paint_from, path_from, text_from,
};

#[pyclass(frozen, name = "RenderBatchV4", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderBatchV4 {
    #[pyo3(get)]
    pub(crate) target: PyRenderTargetV1,
    #[pyo3(get)]
    pub(crate) paint_order: u32,
    coordinate_space: PyRenderCoordinateSpace,
    #[pyo3(get)]
    pub(crate) display_layer: String,
    content: PyRenderBatchContentV4,
}

#[derive(Clone)]
enum PyRenderCoordinateSpace {
    AtomLocal(PyAtomLocalSpaceV1),
    Scene(super::render_primitive_binding::PySceneSpaceV1),
}

#[pymethods]
impl PyRenderBatchV4 {
    #[getter]
    fn coordinate_space(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match &self.coordinate_space {
            PyRenderCoordinateSpace::AtomLocal(value) => Ok(Py::new(py, value.clone())?.into_any()),
            PyRenderCoordinateSpace::Scene(value) => Ok(Py::new(py, value.clone())?.into_any()),
        }
    }

    #[getter]
    fn content(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.content.to_python(py)
    }
}

#[pyclass(frozen, name = "RenderIssueV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderIssueV1 {
    #[pyo3(get)]
    pub(crate) target: PyRenderTargetV1,
    #[pyo3(get)]
    pub(crate) paint_order: u32,
    #[pyo3(get)]
    pub(crate) kind: String,
    #[pyo3(get)]
    pub(crate) detail: String,
}

#[pyclass(frozen, name = "RenderPlanV4", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderPlanV4 {
    #[pyo3(get)]
    pub(crate) schema: String,
    #[pyo3(get)]
    pub(crate) provenance: PyRenderProvenanceV1,
    batches: Vec<PyRenderBatchV4>,
    issues: Vec<PyRenderIssueV1>,
}

#[pyclass(frozen, name = "RenderProvenanceV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderProvenanceV1 {
    #[pyo3(get)]
    pub(crate) revision: u64,
    #[pyo3(get)]
    pub(crate) digest: String,
}

#[pyclass(frozen, name = "MoleculeRenderRootV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyMoleculeRenderRootV1 {
    #[pyo3(get)]
    pub(crate) document_object_id: String,
}

#[pyclass(frozen, name = "DocumentMoleculeRenderPlanV4", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyDocumentMoleculeRenderPlanV4 {
    #[pyo3(get)]
    pub(crate) molecule: PyMoleculeRenderRootV1,
    #[pyo3(get)]
    pub(crate) plan: PyRenderPlanV4,
    #[pyo3(get)]
    pub(crate) bounds: PyMoleculeContentBoundsV1,
    member_issues: Vec<PyMoleculeMemberDepictionIssueV1>,
}

#[pymethods]
impl PyRenderPlanV4 {
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
impl PyDocumentMoleculeRenderPlanV4 {
    #[getter]
    fn member_issues(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.member_issues)
    }
}

#[pyclass(frozen, name = "MoleculeMemberDepictionIssueV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyMoleculeMemberDepictionIssueV1 {
    #[pyo3(get)]
    pub(crate) document_object_id: String,
    #[pyo3(get)]
    pub(crate) category: String,
    #[pyo3(get)]
    pub(crate) detail: String,
}

#[pyclass(frozen, name = "MoleculeContentBoundsV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyMoleculeContentBoundsV1 {
    #[pyo3(get)]
    pub(crate) left: f64,
    #[pyo3(get)]
    pub(crate) top: f64,
    #[pyo3(get)]
    pub(crate) right: f64,
    #[pyo3(get)]
    pub(crate) bottom: f64,
}

impl From<MoleculeContentBoundsV1> for PyMoleculeContentBoundsV1 {
    fn from(value: MoleculeContentBoundsV1) -> Self {
        Self {
            left: value.left(),
            top: value.top(),
            right: value.right(),
            bottom: value.bottom(),
        }
    }
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
    pub(crate) target: PyRenderTargetV1,
    #[pyo3(get)]
    pub(crate) anchor: PyRenderPointV1,
    #[pyo3(get)]
    pub(crate) operation: PyTextOpV1,
    #[pyo3(get)]
    pub(crate) bounds: PyPresentationTextBoundsV1,
    #[pyo3(get)]
    pub(crate) background: Option<super::render_primitive_binding::PyRenderPaintV3>,
}

#[pyclass(frozen, name = "RenderObservationV2", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyRenderObservationV2 {
    #[pyo3(get)]
    pub(crate) schema: String,
    #[pyo3(get)]
    pub(crate) document: super::projection_binding::PySessionDocumentObservationV1,
    #[pyo3(get)]
    pub(crate) profile: String,
    molecule_plans: Vec<PyDocumentMoleculeRenderPlanV4>,
    plus_renders: Vec<PyDocumentPlusRenderV1>,
    text_renders: Vec<super::presentation_text_render_binding::PyDocumentTextRenderV1>,
    #[pyo3(get)]
    pub(crate) suppression: Option<String>,
}

#[pymethods]
impl PyRenderObservationV2 {
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

pub(crate) fn observation(
    py: Python<'_>,
    value: DocumentRenderObservationV2,
) -> PyResult<PyRenderObservationV2> {
    let resolved = value.resolved();
    let molecule_plans = resolved
        .molecule_plans()
        .iter()
        .map(|entry| document_molecule_plan_from(py, entry))
        .collect::<PyResult<_>>()?;
    Ok(PyRenderObservationV2 {
        schema: DOCUMENT_RENDER_OBSERVATION_SCHEMA_V2.to_owned(),
        document: value.document().clone().into(),
        profile: resolved.profile().schema().to_owned(),
        molecule_plans,
        plus_renders: resolved.plus_renders().iter().map(plus_from).collect(),
        text_renders: resolved.text_renders().iter().map(Into::into).collect(),
        suppression: resolved.suppression().map(suppression_name),
    })
}

fn document_molecule_plan_from(
    py: Python<'_>,
    value: &DocumentMoleculeRenderPlanV4,
) -> PyResult<PyDocumentMoleculeRenderPlanV4> {
    let bounds = measure_molecule_render_plan_bounds_v1(value.plan())
        .map_err(|error| super::render_binding::RenderDepictionError::new_err(error.to_string()))?;
    Ok(PyDocumentMoleculeRenderPlanV4 {
        molecule: PyMoleculeRenderRootV1 {
            document_object_id: value.molecule().document_object_id().as_str().to_owned(),
        },
        plan: plan_from(py, value.plan())?,
        bounds: bounds.into(),
        member_issues: value
            .member_issues()
            .iter()
            .map(member_issue_from)
            .collect(),
    })
}

pub(crate) fn plan_from(py: Python<'_>, plan: &MoleculeRenderPlanV4) -> PyResult<PyRenderPlanV4> {
    Ok(PyRenderPlanV4 {
        schema: "ferrum-render-plan-v4".to_owned(),
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

fn batch_from(py: Python<'_>, batch: &RenderBatchV4) -> PyResult<PyRenderBatchV4> {
    let coordinate_space = match batch.coordinate_space() {
        BatchSpace::AtomLocal { anchor } => {
            PyRenderCoordinateSpace::AtomLocal(PyAtomLocalSpaceV1 {
                kind: "atom_local".to_owned(),
                anchor: anchor.into(),
            })
        }
        BatchSpace::Scene => {
            PyRenderCoordinateSpace::Scene(super::render_primitive_binding::PySceneSpaceV1 {
                kind: "scene".to_owned(),
            })
        }
    };
    Ok(PyRenderBatchV4 {
        target: batch.target().into(),
        paint_order: batch.paint_order(),
        coordinate_space,
        display_layer: match batch.display_layer() {
            RenderDisplayLayerV1::Ordinary => "ordinary".to_owned(),
            RenderDisplayLayerV1::HaworthFrontStroke => "haworth_front_stroke".to_owned(),
            RenderDisplayLayerV1::HaworthFrontWedge => "haworth_front_wedge".to_owned(),
        },
        content: batch_content_from(py, batch)?,
    })
}

fn batch_content_from(_py: Python<'_>, batch: &RenderBatchV4) -> PyResult<PyRenderBatchContentV4> {
    Ok(match batch.content() {
        RenderBatchContentV4::Atom(atom) => {
            PyRenderBatchContentV4::Atom(Box::new(PyAtomRenderBatchV1 {
                kind: "atom".to_owned(),
                atom_local_anchor: atom.atom_local_anchor().into(),
                label: atom_label_from(atom.label()),
                decorations: atom
                    .decorations()
                    .iter()
                    .map(atom_decoration_from)
                    .collect(),
            }))
        }
        RenderBatchContentV4::CompactGroup(group) => {
            PyRenderBatchContentV4::CompactGroup(PyCompactGroupRenderBatchV1 {
                kind: "compact_group".to_owned(),
                atom_local_anchor: group.atom_local_anchor().into(),
                operations: group
                    .operations()
                    .iter()
                    .map(compact_group_operation_from)
                    .collect(),
            })
        }
        RenderBatchContentV4::Bond(bond) => PyRenderBatchContentV4::Bond(PyBondRenderBatchV1 {
            kind: "bond".to_owned(),
            operations: bond.operations().iter().map(bond_operation_from).collect(),
        }),
    })
}

fn atom_decoration_from(value: &AtomDecorationRenderOpV1) -> PyAtomDecorationRenderOpV1 {
    match value {
        AtomDecorationRenderOpV1::Text(operation) => {
            PyAtomDecorationRenderOpV1::text(text_from(operation))
        }
        AtomDecorationRenderOpV1::Line(operation) => {
            PyAtomDecorationRenderOpV1::line(line_from(operation))
        }
        AtomDecorationRenderOpV1::Ellipse(operation) => {
            PyAtomDecorationRenderOpV1::ellipse(ellipse_from(operation))
        }
    }
}

fn compact_group_operation_from(value: &CompactGroupRenderOpV1) -> PyCompactGroupRenderOpV1 {
    match value {
        CompactGroupRenderOpV1::Text(operation) => {
            PyCompactGroupRenderOpV1::text(text_from(operation))
        }
        CompactGroupRenderOpV1::Line(operation) => {
            PyCompactGroupRenderOpV1::line(line_from(operation))
        }
        CompactGroupRenderOpV1::Ellipse(operation) => {
            PyCompactGroupRenderOpV1::ellipse(ellipse_from(operation))
        }
    }
}

fn bond_operation_from(value: &BondRenderOpV1) -> PyBondRenderOpV1 {
    match value {
        BondRenderOpV1::Line(operation) => PyBondRenderOpV1::line(line_from(operation)),
        BondRenderOpV1::Path(operation) => PyBondRenderOpV1::path(path_from(operation)),
        BondRenderOpV1::DoubleBondCarrierMark(operation) => {
            PyBondRenderOpV1::double_bond_carrier_mark(line_from(&operation.accent_line()))
        }
    }
}

fn atom_label_from(value: &AtomLabelRenderV1) -> PyAtomLabelRenderV1 {
    PyAtomLabelRenderV1 {
        mask: value.mask().map(|mask| PyMaskOpV1 {
            origin: mask.origin().into(),
            width: mask.width().get(),
            height: mask.height().get(),
            paint: paint_from(mask.paint()),
            z: mask.z(),
        }),
        text: text_from(value.text()),
        core_element_run_index: value.core_element_run_index(),
        full_ink_bounds: ink_bounds_from(value.full_ink_bounds()),
        core_element_ink_bounds: ink_bounds_from(value.core_element_ink_bounds()),
    }
}

fn ink_bounds_from(value: InkBoundsV1) -> PyInkBoundsV1 {
    PyInkBoundsV1 {
        min_x: value.min_x(),
        min_y: value.min_y(),
        max_x: value.max_x(),
        max_y: value.max_y(),
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
        paint_order: value.paint_order(),
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

fn suppression_name(value: DepictionSuppressionV1) -> String {
    format!("{:?}", value).to_ascii_lowercase()
}
fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyRenderObservationV2>()?;
    module.add_class::<PyMoleculeRenderRootV1>()?;
    module.add_class::<PyDocumentMoleculeRenderPlanV4>()?;
    module.add_class::<PyRenderPlanV4>()?;
    module.add_class::<PyRenderProvenanceV1>()?;
    module.add_class::<PyRenderBatchV4>()?;
    module.add_class::<PyAtomRenderBatchV1>()?;
    module.add_class::<PyAtomDecorationRenderOpV1>()?;
    module.add_class::<PyCompactGroupRenderBatchV1>()?;
    module.add_class::<PyCompactGroupRenderOpV1>()?;
    module.add_class::<PyBondRenderBatchV1>()?;
    module.add_class::<PyBondRenderOpV1>()?;
    module.add_class::<PyAtomLabelRenderV1>()?;
    module.add_class::<PyInkBoundsV1>()?;
    module.add_class::<PyRenderIssueV1>()?;
    module.add_class::<PyMoleculeMemberDepictionIssueV1>()?;
    module.add_class::<PyMoleculeContentBoundsV1>()?;
    module.add_class::<PyDocumentPlusRenderV1>()?;
    module.add_class::<PyPresentationTextBoundsV1>()
}
