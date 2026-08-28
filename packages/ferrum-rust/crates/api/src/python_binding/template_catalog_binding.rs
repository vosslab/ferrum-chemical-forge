//! Frozen PyO3 projection for the Rust-owned template catalog.

use std::path::PathBuf;

use ferrum_template_catalog::{
    TemplateCatalogApplyErrorV1, TemplateCatalogEntryV1, TemplateCatalogErrorV1,
    TemplateCatalogLimitsV1, TemplateCatalogPlacementResultV1, TemplateCatalogRefusalV1,
    TemplateCatalogSnapshotV1, TemplateCatalogSourceV1, apply_template_catalog_entry_v1,
    snapshot_template_catalog_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString};

use super::binding::{
    DocumentError, PyDocumentSession, PyDocumentSnapshot, PySessionOperationResultV1,
};

create_exception!(ferrum_chem, TemplateCatalogError, DocumentError);

/// Immutable presentation copy of one native catalog entry.
#[derive(Clone)]
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "TemplateCatalogEntryV1",
    skip_from_py_object
)]
struct PyTemplateCatalogEntryV1 {
    #[pyo3(get)]
    key: String,
    #[pyo3(get)]
    content_identity_algorithm: String,
    #[pyo3(get)]
    content_identity: String,
    #[pyo3(get)]
    label: String,
    #[pyo3(get)]
    source_kind: String,
    #[pyo3(get)]
    family: Option<String>,
    #[pyo3(get)]
    family_label: Option<String>,
    #[pyo3(get)]
    family_order: usize,
    #[pyo3(get)]
    category: Option<String>,
    #[pyo3(get)]
    category_label: Option<String>,
    #[pyo3(get)]
    category_order: usize,
    #[pyo3(get)]
    entry_order: usize,
    #[pyo3(get)]
    provenance_source_id: String,
    #[pyo3(get)]
    provenance_source_kind: String,
    #[pyo3(get)]
    provenance_license_spdx: Option<String>,
    #[pyo3(get)]
    provenance_reviewed_on: Option<String>,
    #[pyo3(get)]
    provenance_chemistry_scope: Option<String>,
    #[pyo3(get)]
    compatibility_profile: String,
    #[pyo3(get)]
    compatibility_format: String,
    #[pyo3(get)]
    aliases: Vec<String>,
    #[pyo3(get)]
    search_terms: Vec<String>,
}

/// Immutable presentation copy of one retained native refusal.
#[derive(Clone)]
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "TemplateCatalogRefusalV1",
    skip_from_py_object
)]
struct PyTemplateCatalogRefusalV1 {
    #[pyo3(get)]
    category: String,
    #[pyo3(get)]
    recovery: String,
    #[pyo3(get)]
    basename: Option<String>,
    #[pyo3(get)]
    occurrences: u64,
}

/// Opaque snapshot capability.  The native snapshot is deliberately private.
#[pyclass(
    frozen,
    unsendable,
    module = "ferrum_chem",
    name = "TemplateCatalogSnapshotV1",
    skip_from_py_object
)]
struct PyTemplateCatalogSnapshotV1 {
    snapshot: TemplateCatalogSnapshotV1,
    #[pyo3(get)]
    schema: String,
    #[pyo3(get)]
    catalog_version: String,
    #[pyo3(get)]
    snapshot_identity_algorithm: String,
    #[pyo3(get)]
    snapshot_identity: String,
    #[pyo3(get)]
    limits_max_entries: usize,
    #[pyo3(get)]
    limits_max_candidates: usize,
    #[pyo3(get)]
    limits_max_refusals: usize,
    #[pyo3(get)]
    limits_max_file_bytes: usize,
    #[pyo3(get)]
    limits_max_total_bytes: usize,
    #[pyo3(get)]
    entries: Vec<PyTemplateCatalogEntryV1>,
    #[pyo3(get)]
    refusals: Vec<PyTemplateCatalogRefusalV1>,
}

/// One accepted native catalog placement.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "TemplateCatalogPlacementResultV1",
    skip_from_py_object
)]
struct PyTemplateCatalogPlacementResultV1 {
    #[pyo3(get)]
    source_kind: String,
    #[pyo3(get)]
    result: PySessionOperationResultV1,
    #[pyo3(get)]
    inserted_molecule_object_id: Option<String>,
    #[pyo3(get)]
    inserted_molecule_source_id: Option<String>,
}

#[pyfunction(name = "snapshot_template_catalog_v1")]
fn snapshot_template_catalog_v1_binding(
    py: Python<'_>,
    directory: Option<&Bound<'_, PyAny>>,
) -> PyResult<PyTemplateCatalogSnapshotV1> {
    let directory = exact_directory(py, directory)?;
    let snapshot = py
        .detach(move || {
            snapshot_template_catalog_v1(
                directory.as_deref(),
                TemplateCatalogLimitsV1::product_default(),
            )
        })
        .map_err(|error| snapshot_error(py, error))?;
    snapshot_projection(snapshot)
}

#[pymethods]
impl PyDocumentSession {
    fn place_template_catalog_entry_v1(
        &mut self,
        py: Python<'_>,
        snapshot: PyRef<'_, PyTemplateCatalogSnapshotV1>,
        key: &Bound<'_, PyAny>,
        expected_document_snapshot: PyRef<'_, PyDocumentSnapshot>,
        x: f64,
        y: f64,
    ) -> PyResult<PyTemplateCatalogPlacementResultV1> {
        let key = exact_string(py, key)?;
        let key = snapshot
            .snapshot
            .find_key_v1(&key)
            .ok_or_else(|| catalog_error(py, "selection_not_found", "choose_entry"))?;
        let digest = parse_digest(expected_document_snapshot.digest_hex())?;
        let placed = apply_template_catalog_entry_v1(
            &mut self.session,
            &snapshot.snapshot,
            key,
            expected_document_snapshot.revision(),
            &digest,
            x,
            y,
        )
        .map_err(|error| apply_error(py, error))?;
        match placed {
            TemplateCatalogPlacementResultV1::Shipped(result) => {
                Ok(PyTemplateCatalogPlacementResultV1 {
                    source_kind: "shipped".to_owned(),
                    result: result.into(),
                    inserted_molecule_object_id: None,
                    inserted_molecule_source_id: None,
                })
            }
            TemplateCatalogPlacementResultV1::User(result) => {
                let object_id = result.inserted_molecule().object_id().as_str().to_owned();
                let source_id = result.inserted_molecule().source_id().as_str().to_owned();
                Ok(PyTemplateCatalogPlacementResultV1 {
                    source_kind: "user_directory".to_owned(),
                    result: result.into_operation_result().into(),
                    inserted_molecule_object_id: Some(object_id),
                    inserted_molecule_source_id: Some(source_id),
                })
            }
        }
    }
}

fn snapshot_projection(
    snapshot: TemplateCatalogSnapshotV1,
) -> PyResult<PyTemplateCatalogSnapshotV1> {
    Ok(PyTemplateCatalogSnapshotV1 {
        schema: snapshot.schema().to_owned(),
        catalog_version: snapshot.catalog_version().to_owned(),
        snapshot_identity_algorithm: snapshot.identity().algorithm().to_owned(),
        snapshot_identity: snapshot.identity().hex().to_owned(),
        limits_max_entries: snapshot.limits().max_entries(),
        limits_max_candidates: snapshot.limits().max_candidates(),
        limits_max_refusals: snapshot.limits().max_refusals(),
        limits_max_file_bytes: snapshot.limits().max_file_bytes(),
        limits_max_total_bytes: snapshot.limits().max_total_bytes(),
        entries: snapshot.entries().iter().map(entry_projection).collect(),
        refusals: snapshot.refusals().iter().map(refusal_projection).collect(),
        snapshot,
    })
}

fn entry_projection(entry: &TemplateCatalogEntryV1) -> PyTemplateCatalogEntryV1 {
    PyTemplateCatalogEntryV1 {
        key: entry.key().as_str().to_owned(),
        content_identity_algorithm: entry.content_identity().algorithm().to_owned(),
        content_identity: entry.content_identity().hex().to_owned(),
        label: entry.label().to_owned(),
        source_kind: source_kind(entry.source()).to_owned(),
        family: entry.family().map(str::to_owned),
        family_label: entry.family_label().map(str::to_owned),
        family_order: entry.family_order(),
        category: entry.category().map(str::to_owned),
        category_label: entry.category_label().map(str::to_owned),
        category_order: entry.category_order(),
        entry_order: entry.entry_order(),
        provenance_source_id: entry.provenance().source_id().to_owned(),
        provenance_source_kind: entry.provenance().source_kind().to_owned(),
        provenance_license_spdx: entry.provenance().license_spdx().map(str::to_owned),
        provenance_reviewed_on: entry.provenance().reviewed_on().map(str::to_owned),
        provenance_chemistry_scope: entry.provenance().chemistry_scope().map(str::to_owned),
        compatibility_profile: entry.compatibility().profile().to_owned(),
        compatibility_format: match entry.compatibility().format() {
            ferrum_template_catalog::TemplateFormatV1::FerrumAuthoredRecipe => {
                "ferrum_authored_recipe"
            }
            ferrum_template_catalog::TemplateFormatV1::Cdml => "cdml",
        }
        .to_owned(),
        aliases: entry.aliases().to_vec(),
        search_terms: entry.search_terms().to_vec(),
    }
}

fn refusal_projection(refusal: &TemplateCatalogRefusalV1) -> PyTemplateCatalogRefusalV1 {
    PyTemplateCatalogRefusalV1 {
        category: category_name(refusal.category()).to_owned(),
        recovery: recovery_name(refusal.recovery()).to_owned(),
        basename: refusal.basename().map(str::to_owned),
        occurrences: refusal.occurrences(),
    }
}

fn exact_directory(
    py: Python<'_>,
    directory: Option<&Bound<'_, PyAny>>,
) -> PyResult<Option<PathBuf>> {
    directory
        .map(|value| exact_string(py, value).map(PathBuf::from))
        .transpose()
}

fn exact_string(_py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<String> {
    if !value.is_exact_instance_of::<PyString>() {
        return Err(pyo3::exceptions::PyTypeError::new_err(
            "template catalog input must be an exact built-in string",
        ));
    }
    value
        .cast::<PyString>()?
        .to_str()
        .map(str::to_owned)
        .map_err(|_| {
            pyo3::exceptions::PyValueError::new_err("template catalog input must be valid UTF-8")
        })
}

fn parse_digest(value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "expected digest must be exactly 64 lowercase hexadecimal characters",
        ));
    }
    let mut digest = [0; 32];
    for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        digest[index] = (hex_value(pair[0]) << 4) | hex_value(pair[1]);
    }
    Ok(digest)
}

const fn hex_value(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}

fn source_kind(source: TemplateCatalogSourceV1) -> &'static str {
    match source {
        TemplateCatalogSourceV1::Shipped => "shipped",
        TemplateCatalogSourceV1::UserDirectory => "user_directory",
    }
}

fn category_name(
    category: ferrum_template_catalog::TemplateCatalogRefusalCategoryV1,
) -> &'static str {
    use ferrum_template_catalog::TemplateCatalogRefusalCategoryV1 as Category;
    match category {
        Category::DirectorySymlink => "directory_symlink",
        Category::DirectoryNotDirectory => "directory_not_directory",
        Category::FilenameNonUtf8 => "filename_non_utf8",
        Category::CandidateSymlink => "candidate_symlink",
        Category::CandidateNotRegular => "candidate_not_regular",
        Category::CandidateOpenFailed => "candidate_open_failed",
        Category::CandidateReadFailed => "candidate_read_failed",
        Category::FileTooLarge => "file_too_large",
        Category::CatalogLimitExceeded => "catalog_limit_exceeded",
        Category::Utf8Invalid => "utf8_invalid",
        Category::DocumentAdmission => "document_admission",
        Category::DuplicateContent => "duplicate_content",
    }
}

fn recovery_name(recovery: ferrum_template_catalog::TemplateCatalogRecoveryV1) -> &'static str {
    use ferrum_template_catalog::TemplateCatalogRecoveryV1 as Recovery;
    match recovery {
        Recovery::Refresh => "refresh",
        Recovery::FixDirectory => "fix_directory",
        Recovery::FixFile => "fix_file",
    }
}

fn snapshot_error(py: Python<'_>, error: TemplateCatalogErrorV1) -> PyErr {
    match error {
        TemplateCatalogErrorV1::DirectoryOpen(_) => {
            catalog_error(py, "directory_open_failed", "fix_directory")
        }
        TemplateCatalogErrorV1::Allocation => {
            catalog_error(py, "catalog_limit_exceeded", "fix_file")
        }
    }
}

fn apply_error(py: Python<'_>, error: TemplateCatalogApplyErrorV1) -> PyErr {
    let (category, recovery) = match error {
        TemplateCatalogApplyErrorV1::SelectionNotFound => ("selection_not_found", "choose_entry"),
        TemplateCatalogApplyErrorV1::InvalidPoint => ("invalid_point", "document_unchanged"),
        TemplateCatalogApplyErrorV1::Shipped(error) => match error {
            ferrum_catalog_placement::CatalogPlacementErrorV1::UnknownKey => {
                ("selection_not_found", "choose_entry")
            }
            ferrum_catalog_placement::CatalogPlacementErrorV1::StaleSnapshot
            | ferrum_catalog_placement::CatalogPlacementErrorV1::MismatchedPreview => {
                ("selection_snapshot_stale", "refresh")
            }
            ferrum_catalog_placement::CatalogPlacementErrorV1::ForeignSession
            | ferrum_catalog_placement::CatalogPlacementErrorV1::Consumed
            | ferrum_catalog_placement::CatalogPlacementErrorV1::SessionConflict => {
                ("session_conflict", "document_unchanged")
            }
            ferrum_catalog_placement::CatalogPlacementErrorV1::InvalidPoint => {
                ("invalid_point", "document_unchanged")
            }
            ferrum_catalog_placement::CatalogPlacementErrorV1::RenderPreparation => {
                ("renderer_refused", "document_unchanged")
            }
        },
        TemplateCatalogApplyErrorV1::User(
            ferrum_document::DocumentUserTemplateApplyErrorV1::Session(error),
        ) => document_session_error_category(error),
        TemplateCatalogApplyErrorV1::Session(error) => document_session_error_category(error),
    };
    catalog_error(py, category, recovery)
}

fn document_session_error_category(
    error: ferrum_document::DocumentSessionError,
) -> (&'static str, &'static str) {
    match error {
        ferrum_document::DocumentSessionError::RevisionConflict { .. }
        | ferrum_document::DocumentSessionError::UserTemplate(
            ferrum_document::DocumentUserTemplateErrorV1::DigestMismatch,
        ) => ("document_stale", "document_unchanged"),
        ferrum_document::DocumentSessionError::RendererAdmission => {
            ("renderer_refused", "document_unchanged")
        }
        ferrum_document::DocumentSessionError::PreparedOperationConsumed
        | ferrum_document::DocumentSessionError::PreparedOperationForeignSession
        | ferrum_document::DocumentSessionError::RevisionExhausted => {
            ("session_conflict", "document_unchanged")
        }
        _ => ("session_conflict", "document_unchanged"),
    }
}

fn catalog_error(py: Python<'_>, category: &str, recovery: &str) -> PyErr {
    let error = TemplateCatalogError::new_err(category.to_owned());
    error
        .value(py)
        .setattr("category", category)
        .expect("category attaches");
    error
        .value(py)
        .setattr("recovery", recovery)
        .expect("recovery attaches");
    error
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "TemplateCatalogError",
        module.py().get_type::<TemplateCatalogError>(),
    )?;
    module.add_class::<PyTemplateCatalogEntryV1>()?;
    module.add_class::<PyTemplateCatalogRefusalV1>()?;
    module.add_class::<PyTemplateCatalogSnapshotV1>()?;
    module.add_class::<PyTemplateCatalogPlacementResultV1>()?;
    module.add_function(wrap_pyfunction!(
        snapshot_template_catalog_v1_binding,
        module
    )?)
}
