//! Python resolver seam for standalone D-glucose Haworth authoring.

use ferrum_document::Point3V1;
use ferrum_domain::haworth::StandaloneDGlucoseHaworthRecipeV1;
use pyo3::prelude::*;

use super::{
    binding::{PyDocumentSession, document_result, operation_validation_error, projection_error},
    prepared_transition_binding::PySessionOperationTransitionRequestV1,
};

#[pymethods]
impl PyDocumentSession {
    /// Resolve a supported D-glucose recipe and anchor into generic authority.
    fn resolve_standalone_haworth_transition_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        recipe: &str,
        center_x: f64,
        center_y: f64,
    ) -> PyResult<PySessionOperationTransitionRequestV1> {
        let recipe = recipe_from_text(recipe).ok_or_else(|| {
            operation_validation_error(
                py,
                "choose one supported D-glucose Haworth recipe".to_owned(),
            )
        })?;
        let anchor = Point3V1::new(center_x, center_y, 0.0)
            .map_err(|error| projection_error(py, error).expect("projection error construction"))?;
        document_result(
            py,
            self.session.resolve_standalone_haworth_transition_v1(
                expected_revision,
                recipe,
                anchor,
            ),
        )
        .map(PySessionOperationTransitionRequestV1::from_request)
    }
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
