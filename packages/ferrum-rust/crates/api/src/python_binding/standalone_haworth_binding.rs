//! Private native standalone D-glucose Haworth insertion seam.

use ferrum_document::{PendingStandaloneHaworthV1, Point3V1};
use ferrum_domain::haworth::{
    StandaloneDGlucoseHaworthRecipeV1, StandaloneHaworthBondTokenV1,
    standalone_d_glucose_haworth_recipe_v1,
};
use pyo3::prelude::*;

use super::{
    binding::{
        PyDocumentSession, PySessionOperationResultV1, document_result, operation_validation_error,
        projection_error,
    },
    projection_binding::PyPoint3V1,
    render_binding::{PyRenderOperationV2, operation_from},
};

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "StandaloneHaworthPreviewBatchV2",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyStandaloneHaworthPreviewBatchV2 {
    #[pyo3(get)]
    pub(crate) display_layer: String,
    operations: Vec<PyRenderOperationV2>,
}
#[pymethods]
impl PyStandaloneHaworthPreviewBatchV2 {
    #[getter]
    fn operations(&self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyTuple>> {
        super::render_binding::frozen_tuple(py, &self.operations)
    }
}

#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PreparedStandaloneHaworthInsertionV1"
)]
pub(crate) struct PyPreparedStandaloneHaworthInsertionV1 {
    pub(crate) pending: PendingStandaloneHaworthV1,
    #[pyo3(get)]
    pub(crate) molecule_identifier: String,
    #[pyo3(get)]
    pub(crate) atom_identifiers: Vec<String>,
    #[pyo3(get)]
    pub(crate) bond_identifiers: Vec<String>,
    #[pyo3(get)]
    pub(crate) vertices: Vec<PyPoint3V1>,
    #[pyo3(get)]
    pub(crate) edges: Vec<(usize, usize)>,
    preview_batches: Vec<PyStandaloneHaworthPreviewBatchV2>,
}
#[pymethods]
impl PyPreparedStandaloneHaworthInsertionV1 {
    #[getter]
    fn preview_batches(&self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyTuple>> {
        super::render_binding::frozen_tuple(py, &self.preview_batches)
    }
}

#[pymethods]
impl PyDocumentSession {
    fn prepare_create_standalone_haworth_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        recipe: &str,
        center_x: f64,
        center_y: f64,
    ) -> PyResult<PyPreparedStandaloneHaworthInsertionV1> {
        let recipe = recipe_from_text(recipe).ok_or_else(|| {
            operation_validation_error(
                py,
                "choose one supported D-glucose Haworth recipe".to_owned(),
            )
        })?;
        let anchor = Point3V1::new(center_x, center_y, 0.0)
            .map_err(|error| projection_error(py, error).expect("projection error construction"))?;
        let pending = document_result(
            py,
            self.session
                .prepare_create_standalone_haworth_v1(expected_revision, recipe, anchor),
        )?;
        let preview_batches = preview_batches(py, pending.recipe(), pending.vertices())?;
        Ok(PyPreparedStandaloneHaworthInsertionV1 {
            molecule_identifier: pending.molecule_identifier().as_str().to_owned(),
            atom_identifiers: pending
                .atom_identifiers()
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect(),
            bond_identifiers: pending
                .bond_identifiers()
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect(),
            vertices: pending
                .vertices()
                .iter()
                .map(|point| PyPoint3V1 {
                    x: point.x(),
                    y: point.y(),
                    z: point.z(),
                })
                .collect(),
            edges: pending
                .edges()
                .iter()
                .map(|edge| (edge[0], edge[1]))
                .collect(),
            preview_batches,
            pending,
        })
    }
    fn commit_create_standalone_haworth_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        mut prepared: PyRefMut<'_, PyPreparedStandaloneHaworthInsertionV1>,
    ) -> PyResult<PySessionOperationResultV1> {
        document_result(
            py,
            self.session
                .commit_create_standalone_haworth_v1(expected_revision, &mut prepared.pending),
        )
        .map(Into::into)
    }
}

fn preview_batches(
    py: Python<'_>,
    recipe: StandaloneDGlucoseHaworthRecipeV1,
    vertices: &[Point3V1],
) -> PyResult<Vec<PyStandaloneHaworthPreviewBatchV2>> {
    let receipt = standalone_d_glucose_haworth_recipe_v1(recipe)
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    let width = ferrum_render::PositiveFinite::new(1.0)
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    let wedge_width = ferrum_render::PositiveFinite::new(5.0)
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    let paint = ferrum_render::Paint::rgb24(
        ferrum_render::Rgb24::new("000000")
            .map_err(|error| operation_validation_error(py, error.to_string()))?,
    );
    receipt
        .bonds()
        .iter()
        .map(|bond| {
            let start = vertices[bond.start()];
            let end = vertices[bond.end()];
            let tip = ferrum_render::RenderPoint::new(start.x(), start.y())
                .map_err(|error| operation_validation_error(py, error.to_string()))?;
            let base = ferrum_render::RenderPoint::new(end.x(), end.y())
                .map_err(|error| operation_validation_error(py, error.to_string()))?;
            let (display_layer, operations) = match bond.token() {
                StandaloneHaworthBondTokenV1::N1 => (
                    "ordinary",
                    vec![ferrum_render::RenderOp::Line(
                        ferrum_render::LineOp::new(tip, base, width, paint.clone(), 10)
                            .map_err(|error| operation_validation_error(py, error.to_string()))?,
                    )],
                ),
                StandaloneHaworthBondTokenV1::Q1 => (
                    "haworth_front_stroke",
                    ferrum_render::build_haworth_front_preview_ops(
                        ferrum_render::BondStyle::HaworthFrontStroke,
                        tip,
                        base,
                        width,
                        wedge_width,
                        paint.clone(),
                    )
                    .map_err(|error| operation_validation_error(py, error.to_string()))?,
                ),
                StandaloneHaworthBondTokenV1::W1 => (
                    "haworth_front_wedge",
                    ferrum_render::build_haworth_front_preview_ops(
                        ferrum_render::BondStyle::HaworthFrontWedge,
                        tip,
                        base,
                        width,
                        wedge_width,
                        paint.clone(),
                    )
                    .map_err(|error| operation_validation_error(py, error.to_string()))?,
                ),
            };
            Ok(PyStandaloneHaworthPreviewBatchV2 {
                display_layer: display_layer.to_owned(),
                operations: operations
                    .iter()
                    .map(|operation| operation_from(py, operation))
                    .collect::<PyResult<_>>()?,
            })
        })
        .collect()
}

fn recipe_from_text(value: &str) -> Option<StandaloneDGlucoseHaworthRecipeV1> {
    Some(match value {
        "alpha-D-glucopyranose" => StandaloneDGlucoseHaworthRecipeV1::AlphaDGlucopyranose,
        "beta-D-glucopyranose" => StandaloneDGlucoseHaworthRecipeV1::BetaDGlucopyranose,
        "alpha-D-glucofuranose" => StandaloneDGlucoseHaworthRecipeV1::AlphaDGlucofuranose,
        "beta-D-glucofuranose" => StandaloneDGlucoseHaworthRecipeV1::BetaDGlucofuranose,
        _ => return None,
    })
}
