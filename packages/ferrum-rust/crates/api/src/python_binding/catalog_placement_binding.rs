//! PyO3 transport for one generic catalog semantic operation.

use super::binding::{PyDocumentSession, PySessionOperationResultV1};
use super::presentation_creation_gesture_binding::digest;
use ferrum_catalog_placement::{CatalogPlacementErrorV1, resolve_catalog_molecule_placement_v1};
use ferrum_document::{
    PresentationGesturePoint2V1, SessionOperation, SessionOperationOutcomeV1,
    SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
};
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

#[pyclass(frozen, module = "ferrum_chem", name = "CatalogPlacementResultV1")]
struct PyCatalogPlacementResultV1 {
    #[pyo3(get)]
    root_identifier: String,
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
    fn place_catalog_molecule_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        key: String,
        x: f64,
        y: f64,
    ) -> PyResult<PyCatalogPlacementResultV1> {
        let expected_digest = digest(&expected_digest_hex)?;
        let snapshot = self
            .session
            .snapshot()
            .map_err(|error| CatalogPlacementError::new_err(error.to_string()))?;
        if snapshot.revision() != expected_revision || snapshot.digest() != &expected_digest {
            return Err(catalog_error(py, "stale_snapshot", "refresh_and_restart"));
        }
        let anchor = PresentationGesturePoint2V1::new(x, y)
            .map_err(|_| catalog_error(py, "invalid_point", "document_unchanged"))?;
        let request = resolve_catalog_molecule_placement_v1(&key, anchor)
            .map_err(|error| catalog_resolution_error(py, error))?;
        let mut prepared = self
            .session
            .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
                expected_revision,
                SessionOperation::V1(SessionOperationV1::PlaceCatalogMoleculeV1(request)),
                TransitionAuthorizationV1::None,
            ))
            .map_err(|_| catalog_error(py, "session_conflict", "refresh_and_restart"))?;
        let result = self
            .session
            .commit_session_operation_transition_v1(&mut prepared)
            .map_err(|_| catalog_error(py, "session_conflict", "refresh_and_restart"))?;
        let SessionOperationOutcomeV1::CatalogMoleculePlacementV1(outcome) = result.outcome()
        else {
            return Err(catalog_error(py, "session_conflict", "refresh_and_restart"));
        };
        Ok(PyCatalogPlacementResultV1 {
            root_identifier: outcome.root_identifier().as_str().to_owned(),
            result: result.into(),
        })
    }
}

fn catalog_resolution_error(py: Python<'_>, error: CatalogPlacementErrorV1) -> PyErr {
    catalog_error(
        py,
        &format!("{:?}", error.category()).to_lowercase(),
        &format!("{:?}", error.recovery()).to_lowercase(),
    )
}

fn catalog_error(py: Python<'_>, category: &str, recovery: &str) -> PyErr {
    let exception = CatalogPlacementError::new_err(category.to_owned());
    let value = exception.value(py);
    value
        .setattr("category", category)
        .expect("category attaches");
    value
        .setattr("recovery", recovery)
        .expect("recovery attaches");
    exception
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "CatalogPlacementError",
        module.py().get_type::<CatalogPlacementError>(),
    )?;
    module.add_class::<PyCatalogSummaryV1>()?;
    module.add_class::<PyCatalogPlacementResultV1>()?;
    module.add_function(wrap_pyfunction!(list_catalog_v1, module)?)
}
