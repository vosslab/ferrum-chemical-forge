//! Private native standalone D-glucose Haworth insertion seam.

use ferrum_document::{PendingStandaloneHaworthV1, Point3V1};
use ferrum_domain::haworth::StandaloneDGlucoseHaworthRecipeV1;
use ferrum_render::{
    DocumentRenderContentV1, DocumentRenderIdentityV1, preview_root_render_overlay_v1,
};
use pyo3::prelude::*;

use super::{
    binding::{
        PyDocumentSession, PySessionOperationResultV1, document_result, operation_validation_error,
        projection_error,
    },
    render_binding::{PyRenderPlanV2, plan_from},
};

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
    preview_plan: PyRenderPlanV2,
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
        let preview_plan = preview_plan(py, &pending)?;
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
            preview_plan,
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

fn preview_plan(py: Python<'_>, pending: &PendingStandaloneHaworthV1) -> PyResult<PyRenderPlanV2> {
    let identity = DocumentRenderIdentityV1::durable(pending.molecule_identifier().as_str())
        .map_err(|_| {
            operation_validation_error(
                py,
                "renderer plan did not preserve the pending molecule".to_owned(),
            )
        })?;
    let overlay =
        preview_root_render_overlay_v1(pending.render_plan_v1(), &identity).map_err(|_| {
            operation_validation_error(
                py,
                "renderer plan did not preserve the pending molecule".to_owned(),
            )
        })?;
    let DocumentRenderContentV1::Molecule(plan) = overlay.content() else {
        return Err(operation_validation_error(
            py,
            "renderer plan did not preserve the pending molecule".to_owned(),
        ));
    };
    plan_from(py, plan)
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
