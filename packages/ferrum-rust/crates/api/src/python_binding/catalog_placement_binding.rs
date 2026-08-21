//! Opaque PyO3 transport for Ferrum-owned catalog placement.
use super::binding::{PyDocumentSession, PySessionOperationResultV1};
use super::presentation_creation_gesture_binding::digest;
use super::render_binding::{PyRenderPlanV2, plan_from};
use crate::{
    ApiCatalogPlacementGestureV2, ApiCatalogPlacementPreparedV2, ApiCatalogPlacementPreviewV2,
    CatalogPlacementErrorV2, begin_api_catalog_placement_v2,
    cancel_api_catalog_placement_gesture_v2, commit_api_catalog_placement_v2,
    prepare_api_catalog_placement_v2, preview_api_catalog_placement_v2,
    release_api_catalog_placement_preview_v2,
};
use ferrum_document::{DocumentFenceV1, PresentationGesturePoint2V1};
use ferrum_domain::{CatalogFamilyV1, catalog_manifest_v1, search_catalog_v1};
use pyo3::create_exception;
use pyo3::prelude::*;
create_exception!(
    ferrum_chem,
    CatalogPlacementError,
    super::binding::DocumentError
);
#[pyclass(frozen, module = "ferrum_chem", name = "CatalogSummaryV1")]
struct PyCatalogSummaryV1 {
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    catalog_version: String,
    #[pyo3(get)]
    key: String,
    #[pyo3(get)]
    family: String,
    #[pyo3(get)]
    category: String,
    #[pyo3(get)]
    label: String,
    #[pyo3(get)]
    provenance_source: String,
}
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "CatalogRenderOverlayV2",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyCatalogRenderOverlayV2 {
    #[pyo3(get)]
    plan: PyRenderPlanV2,
    #[pyo3(get)]
    source_order: u32,
}
#[pyclass(unsendable, module = "ferrum_chem", name = "CatalogPlacementGestureV2")]
struct PyCatalogPlacementGestureV2 {
    value: ApiCatalogPlacementGestureV2,
}
#[pyclass(unsendable, module = "ferrum_chem", name = "CatalogPlacementPreviewV2")]
struct PyCatalogPlacementPreviewV2 {
    value: ApiCatalogPlacementPreviewV2,
    #[pyo3(get)]
    overlay: PyCatalogRenderOverlayV2,
}
#[pyclass(unsendable, module = "ferrum_chem", name = "CatalogPlacementReceiptV2")]
struct PyCatalogPlacementReceiptV2 {
    value: ApiCatalogPlacementPreparedV2,
}
#[pyclass(frozen, module = "ferrum_chem", name = "CatalogPlacementCommitV2")]
struct PyCatalogPlacementCommitV2 {
    #[pyo3(get)]
    identifier: String,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
}
#[pyfunction]
fn list_catalog_v1(
    family: Option<String>,
    category: Option<String>,
    query: Option<String>,
) -> PyResult<Vec<PyCatalogSummaryV1>> {
    let family = match family.as_deref() {
        None => None,
        Some("system") => Some(CatalogFamilyV1::System),
        Some("biomolecule") => Some(CatalogFamilyV1::Biomolecule),
        Some(_) => return Err(CatalogPlacementError::new_err("unknown catalog family")),
    };
    let manifest = catalog_manifest_v1();
    Ok(
        search_catalog_v1(family, category.as_deref(), query.as_deref())
            .into_iter()
            .map(|entry| PyCatalogSummaryV1 {
                schema: manifest.schema().to_owned(),
                catalog_version: manifest.catalog_version().to_owned(),
                key: entry.key().as_str().to_owned(),
                family: match entry.family() {
                    CatalogFamilyV1::System => "system",
                    CatalogFamilyV1::Biomolecule => "biomolecule",
                }
                .to_owned(),
                category: entry.category().key().to_owned(),
                label: entry.label().to_owned(),
                provenance_source: entry.provenance().source_id().to_owned(),
            })
            .collect(),
    )
}
#[pymethods]
impl PyDocumentSession {
    fn begin_catalog_placement_v2(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        key: String,
    ) -> PyResult<PyCatalogPlacementGestureV2> {
        begin_api_catalog_placement_v2(
            &self.session,
            DocumentFenceV1::new(expected_revision, digest(&expected_digest_hex)?),
            &key,
        )
        .map(|value| PyCatalogPlacementGestureV2 { value })
        .map_err(|error| catalog_error_v2(py, error))
    }
    fn preview_catalog_placement_v2(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyCatalogPlacementGestureV2>,
        x: f64,
        y: f64,
    ) -> PyResult<PyCatalogPlacementPreviewV2> {
        let anchor = PresentationGesturePoint2V1::new(x, y)
            .map_err(|_| catalog_error_v2(py, CatalogPlacementErrorV2::InvalidPoint))?;
        let value = preview_api_catalog_placement_v2(&mut self.session, &gesture.value, anchor)
            .map_err(|error| catalog_error_v2(py, error))?;
        let plan = value
            .molecule_plan()
            .ok_or_else(|| catalog_error_v2(py, CatalogPlacementErrorV2::RenderPreparation))?;
        let overlay = PyCatalogRenderOverlayV2 {
            plan: plan_from(py, plan)?,
            source_order: value.source_order(),
        };
        Ok(PyCatalogPlacementPreviewV2 { value, overlay })
    }
    fn prepare_catalog_placement_v2(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyCatalogPlacementGestureV2>,
        mut preview: PyRefMut<'_, PyCatalogPlacementPreviewV2>,
    ) -> PyResult<PyCatalogPlacementReceiptV2> {
        prepare_api_catalog_placement_v2(&mut self.session, &gesture.value, &mut preview.value)
            .map(|value| PyCatalogPlacementReceiptV2 { value })
            .map_err(|error| catalog_error_v2(py, error))
    }
    fn release_catalog_placement_preview_v2(
        &mut self,
        mut preview: PyRefMut<'_, PyCatalogPlacementPreviewV2>,
    ) {
        release_api_catalog_placement_preview_v2(&mut preview.value);
    }
    fn cancel_catalog_placement_gesture_v2(&self, gesture: PyRef<'_, PyCatalogPlacementGestureV2>) {
        cancel_api_catalog_placement_gesture_v2(gesture.value.clone());
    }
    fn commit_catalog_placement_v2(
        &mut self,
        py: Python<'_>,
        mut receipt: PyRefMut<'_, PyCatalogPlacementReceiptV2>,
    ) -> PyResult<PyCatalogPlacementCommitV2> {
        commit_api_catalog_placement_v2(&mut self.session, &mut receipt.value)
            .map(|value| PyCatalogPlacementCommitV2 {
                identifier: value.identifier().to_owned(),
                result: value.result().clone().into(),
            })
            .map_err(|error| catalog_error_v2(py, error))
    }
}
fn catalog_error_v2(py: Python<'_>, error: CatalogPlacementErrorV2) -> PyErr {
    let exception = CatalogPlacementError::new_err(error.to_string());
    let value = exception.value(py);
    value
        .setattr("category", format!("{:?}", error.category()))
        .expect("category attaches");
    value
        .setattr("recovery", format!("{:?}", error.recovery()))
        .expect("recovery attaches");
    exception
}
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "CatalogPlacementError",
        module.py().get_type::<CatalogPlacementError>(),
    )?;
    module.add_class::<PyCatalogSummaryV1>()?;
    module.add_class::<PyCatalogRenderOverlayV2>()?;
    module.add_class::<PyCatalogPlacementGestureV2>()?;
    module.add_class::<PyCatalogPlacementPreviewV2>()?;
    module.add_class::<PyCatalogPlacementReceiptV2>()?;
    module.add_class::<PyCatalogPlacementCommitV2>()?;
    module.add_function(wrap_pyfunction!(list_catalog_v1, module)?)
}
