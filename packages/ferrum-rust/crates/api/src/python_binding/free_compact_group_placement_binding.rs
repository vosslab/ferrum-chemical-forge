//! Private opaque PyO3 seam for one free compact-group placement.

use ferrum_document::{
    CompactGroupCatalogKeyV1, DocumentFenceV1, DocumentSession,
    FreeCompactGroupPlacementCommitResultV1, FreeCompactGroupPlacementSessionErrorV1,
    PendingPlaceFreeCompactGroupV1, PlaceFreeCompactGroupV1, Point3V1,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::{
    binding::PyDocumentSession, document_error_binding::RevisionConflictError,
    projection_binding::PySessionDocumentObservationV1,
};

create_exception!(
    ferrum_chem,
    FreeCompactGroupPlacementError,
    super::document_error_binding::DocumentError
);

/// Closed refusal facts for the private free compact-group placement operation.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "FreeCompactGroupPlacementCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyFreeCompactGroupPlacementCategoryV1 {
    InvalidDigest,
    InvalidCatalogKey,
    NonFinitePoint,
    StaleRevision,
    StaleDigest,
    ForeignSession,
    Consumed,
    UnsupportedCatalogKey,
    CandidateAdmission,
    RendererAdmission,
    SessionConflict,
}

/// Opaque one-use candidate retained by the native tab only.
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PendingPlaceFreeCompactGroupV1"
)]
pub(crate) struct PyPendingPlaceFreeCompactGroupV1 {
    pending: PendingPlaceFreeCompactGroupV1,
}

/// Authoritative durable facts after one committed free compact-group placement.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "FreeCompactGroupPlacementCommitFactsV1",
    skip_from_py_object
)]
pub(crate) struct PyFreeCompactGroupPlacementCommitFactsV1 {
    #[pyo3(get)]
    observation: PySessionDocumentObservationV1,
    #[pyo3(get)]
    revision: u64,
    #[pyo3(get)]
    digest: String,
    #[pyo3(get)]
    is_dirty: bool,
    #[pyo3(get)]
    molecule_object_id: String,
    #[pyo3(get)]
    compact_group_object_id: String,
}

#[pymethods]
impl PyDocumentSession {
    /// Begin one native-tab free compact-group placement from snapped scene facts.
    fn _begin_place_free_compact_group_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        catalog_key: String,
        snapped_scene_x: f64,
        snapped_scene_y: f64,
    ) -> PyResult<PyPendingPlaceFreeCompactGroupV1> {
        let fence =
            DocumentFenceV1::new(expected_revision, parse_digest(py, &expected_digest_hex)?);
        let key = CompactGroupCatalogKeyV1::parse(&catalog_key).ok_or_else(|| {
            placement_category_error(
                py,
                PyFreeCompactGroupPlacementCategoryV1::InvalidCatalogKey,
                "catalog key is not recognized",
            )
        })?;
        let anchor = Point3V1::new(snapped_scene_x, snapped_scene_y, 0.0).map_err(|_| {
            placement_category_error(
                py,
                PyFreeCompactGroupPlacementCategoryV1::NonFinitePoint,
                "snapped scene point must be finite",
            )
        })?;
        let request = PlaceFreeCompactGroupV1::new(key, anchor);
        begin(&mut self.session, fence, request).map_err(|error| placement_error(py, error))
    }

    /// Commit the private free compact-group candidate once.
    fn _commit_place_free_compact_group_v1(
        &mut self,
        py: Python<'_>,
        mut pending: PyRefMut<'_, PyPendingPlaceFreeCompactGroupV1>,
    ) -> PyResult<PyFreeCompactGroupPlacementCommitFactsV1> {
        commit(&mut self.session, &mut pending.pending).map_err(|error| placement_error(py, error))
    }

    /// Cancel one pending free compact-group placement without a document edit.
    fn _cancel_place_free_compact_group_v1(
        &mut self,
        py: Python<'_>,
        mut pending: PyRefMut<'_, PyPendingPlaceFreeCompactGroupV1>,
    ) -> PyResult<()> {
        cancel(&mut self.session, &mut pending.pending).map_err(|error| placement_error(py, error))
    }
}

fn begin(
    session: &mut DocumentSession,
    fence: DocumentFenceV1,
    request: PlaceFreeCompactGroupV1,
) -> Result<PyPendingPlaceFreeCompactGroupV1, FreeCompactGroupPlacementSessionErrorV1> {
    session
        .prepare_place_free_compact_group_v1(fence, request)
        .map(|pending| PyPendingPlaceFreeCompactGroupV1 { pending })
}

fn commit(
    session: &mut DocumentSession,
    pending: &mut PendingPlaceFreeCompactGroupV1,
) -> Result<PyFreeCompactGroupPlacementCommitFactsV1, FreeCompactGroupPlacementSessionErrorV1> {
    let result = session.commit_place_free_compact_group_v1(pending)?;
    Ok(commit_facts(result))
}

fn cancel(
    session: &mut DocumentSession,
    pending: &mut PendingPlaceFreeCompactGroupV1,
) -> Result<(), FreeCompactGroupPlacementSessionErrorV1> {
    session.cancel_place_free_compact_group_v1(pending)
}

fn commit_facts(
    result: FreeCompactGroupPlacementCommitResultV1,
) -> PyFreeCompactGroupPlacementCommitFactsV1 {
    let snapshot = result.observation().snapshot();
    PyFreeCompactGroupPlacementCommitFactsV1 {
        observation: result.observation().clone().into(),
        revision: snapshot.revision(),
        digest: snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        is_dirty: snapshot.is_dirty(),
        molecule_object_id: result.molecule_object_id().as_str().to_owned(),
        compact_group_object_id: result.compact_group_object_id().as_str().to_owned(),
    }
}

fn category(
    error: FreeCompactGroupPlacementSessionErrorV1,
) -> PyFreeCompactGroupPlacementCategoryV1 {
    match error {
        FreeCompactGroupPlacementSessionErrorV1::StaleRevision => {
            PyFreeCompactGroupPlacementCategoryV1::StaleRevision
        }
        FreeCompactGroupPlacementSessionErrorV1::StaleDigest => {
            PyFreeCompactGroupPlacementCategoryV1::StaleDigest
        }
        FreeCompactGroupPlacementSessionErrorV1::ForeignSession => {
            PyFreeCompactGroupPlacementCategoryV1::ForeignSession
        }
        FreeCompactGroupPlacementSessionErrorV1::Consumed => {
            PyFreeCompactGroupPlacementCategoryV1::Consumed
        }
        FreeCompactGroupPlacementSessionErrorV1::UnsupportedCatalogKey => {
            PyFreeCompactGroupPlacementCategoryV1::UnsupportedCatalogKey
        }
        FreeCompactGroupPlacementSessionErrorV1::CandidateAdmission => {
            PyFreeCompactGroupPlacementCategoryV1::CandidateAdmission
        }
        FreeCompactGroupPlacementSessionErrorV1::RendererAdmission => {
            PyFreeCompactGroupPlacementCategoryV1::RendererAdmission
        }
        FreeCompactGroupPlacementSessionErrorV1::SessionConflict => {
            PyFreeCompactGroupPlacementCategoryV1::SessionConflict
        }
    }
}

fn placement_error(py: Python<'_>, error: FreeCompactGroupPlacementSessionErrorV1) -> PyErr {
    let exception = match error {
        FreeCompactGroupPlacementSessionErrorV1::StaleRevision
        | FreeCompactGroupPlacementSessionErrorV1::StaleDigest => {
            RevisionConflictError::new_err(error.to_string())
        }
        _ => FreeCompactGroupPlacementError::new_err(error.to_string()),
    };
    exception
        .value(py)
        .setattr(
            "category",
            Py::new(py, category(error)).expect("closed category enum allocates"),
        )
        .expect("free compact-group error category attaches");
    exception
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(placement_category_error(
            py,
            PyFreeCompactGroupPlacementCategoryV1::InvalidDigest,
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

fn placement_category_error(
    py: Python<'_>,
    category: PyFreeCompactGroupPlacementCategoryV1,
    message: &str,
) -> PyErr {
    let exception = FreeCompactGroupPlacementError::new_err(message.to_owned());
    exception
        .value(py)
        .setattr(
            "category",
            Py::new(py, category).expect("closed category enum allocates"),
        )
        .expect("free compact-group error category attaches");
    exception
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "FreeCompactGroupPlacementError",
        module.py().get_type::<FreeCompactGroupPlacementError>(),
    )?;
    module.add_class::<PyFreeCompactGroupPlacementCategoryV1>()?;
    module.add_class::<PyFreeCompactGroupPlacementCommitFactsV1>()?;
    Ok(())
}
