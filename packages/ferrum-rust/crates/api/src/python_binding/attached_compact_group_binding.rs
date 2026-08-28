//! Private opaque PyO3 seam for reviewed molecule-plus-anchor compact-group attachment.

use ferrum_document::{
    AttachCompactGroupV1, AttachedCompactGroupAvailabilityCategoryV1,
    AttachedCompactGroupAvailabilityV1, AttachedCompactGroupCommitResultV1,
    AttachedCompactGroupReleaseV1, AttachedCompactGroupSessionErrorV1,
    AttachedCompactGroupTargetV1, CompactGroupCatalogKeyV1, DocumentFenceV1, DocumentSession,
    PendingAttachedCompactGroupV1, attached_compact_group_choices_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;

use super::{
    binding::PyDocumentSession,
    document_error_binding::{RevisionConflictError, document_object_id},
    prepared_transition_binding::{PyDocumentPrecommitOverlayV1, overlay_from},
};

create_exception!(
    ferrum_chem,
    AttachedCompactGroupAttachmentError,
    super::document_error_binding::DocumentError
);

/// Closed refusal facts for the private reviewed compact-group operation.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "AttachedCompactGroupCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyAttachedCompactGroupCategoryV1 {
    InvalidDigest,
    InvalidCatalogKey,
    StaleRevision,
    StaleDigest,
    ForeignSession,
    Consumed,
    UnknownMolecule,
    UnknownAnchor,
    ForeignTarget,
    InvalidPose,
    CandidateAdmission,
    RendererAdmission,
    SessionConflict,
}

/// Closed categories for read-only attached compact-group action availability.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "AttachedCompactGroupAvailabilityCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyAttachedCompactGroupAvailabilityCategoryV1 {
    Available,
    StaleRevision,
    StaleDigest,
    UnknownMolecule,
    UnknownAnchor,
    ForeignTarget,
    CandidateAdmission,
    SessionConflict,
}

/// One Rust-owned reviewed choice for the private attached-group interaction.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "AttachedCompactGroupChoiceFactsV1",
    skip_from_py_object
)]
pub(crate) struct PyAttachedCompactGroupChoiceFactsV1 {
    #[pyo3(get)]
    catalog_key: String,
    #[pyo3(get)]
    label: String,
}

/// Immutable read-only facts suitable for compact-group action enablement.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "AttachedCompactGroupAvailabilityFactsV1",
    skip_from_py_object
)]
pub(crate) struct PyAttachedCompactGroupAvailabilityFactsV1 {
    #[pyo3(get)]
    available: bool,
    #[pyo3(get)]
    category: PyAttachedCompactGroupAvailabilityCategoryV1,
    #[pyo3(get)]
    revision: u64,
    #[pyo3(get)]
    digest: String,
    #[pyo3(get)]
    anchor_object_id: String,
    #[pyo3(get)]
    catalog_key: String,
}

/// Opaque one-use candidate retained by the native tab only.
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PendingAttachedCompactGroupV1"
)]
pub(crate) struct PyPendingAttachedCompactGroupV1 {
    pending: PendingAttachedCompactGroupV1,
}

/// Immutable renderer-issued compact-group overlay for native-tab paint-only replay.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "AttachedCompactGroupPreviewV1",
    skip_from_py_object
)]
pub(crate) struct PyAttachedCompactGroupPreviewV1 {
    #[pyo3(get)]
    overlay: PyDocumentPrecommitOverlayV1,
}

/// Authoritative durable facts after one committed compact-group attachment.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "AttachedCompactGroupCommitFactsV1",
    skip_from_py_object
)]
pub(crate) struct PyAttachedCompactGroupCommitFactsV1 {
    #[pyo3(get)]
    revision: u64,
    #[pyo3(get)]
    digest: String,
    #[pyo3(get)]
    is_dirty: bool,
    #[pyo3(get)]
    focus_object_id: String,
    #[pyo3(get)]
    compact_group_object_id: String,
}

#[pymethods]
impl PyDocumentSession {
    /// Return Rust-owned reviewed choices for attached compact-group authoring.
    fn _attached_compact_group_choices_v1(&self) -> Vec<PyAttachedCompactGroupChoiceFactsV1> {
        attached_compact_group_choices_v1()
            .map(|choice| PyAttachedCompactGroupChoiceFactsV1 {
                catalog_key: choice.catalog_key().as_str().to_owned(),
                label: choice.label().to_owned(),
            })
            .collect()
    }

    /// Observe current read-only enablement facts for one fenced molecule-plus-anchor pair and choice.
    fn _attach_compact_group_availability_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        molecule_object_id: String,
        anchor_object_id: String,
        catalog_key: String,
    ) -> PyResult<PyAttachedCompactGroupAvailabilityFactsV1> {
        let fence =
            DocumentFenceV1::new(expected_revision, parse_digest(py, &expected_digest_hex)?);
        let target = AttachedCompactGroupTargetV1::new(
            document_object_id(py, molecule_object_id)?,
            document_object_id(py, anchor_object_id)?,
        );
        let key = parse_catalog_key(py, &catalog_key)?;
        Ok(availability_facts(
            self.session
                .observe_attach_compact_group_availability_v1(fence, target, key),
        ))
    }

    /// Begin one private native-tab compact-group attachment from durable target and pointer facts.
    #[allow(clippy::too_many_arguments)]
    fn _begin_attach_compact_group_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        molecule_object_id: String,
        anchor_object_id: String,
        catalog_key: String,
        raw_release_x: f64,
        raw_release_y: f64,
    ) -> PyResult<PyPendingAttachedCompactGroupV1> {
        let fence =
            DocumentFenceV1::new(expected_revision, parse_digest(py, &expected_digest_hex)?);
        let target = AttachedCompactGroupTargetV1::new(
            document_object_id(py, molecule_object_id)?,
            document_object_id(py, anchor_object_id)?,
        );
        let key = parse_catalog_key(py, &catalog_key)?;
        let release = AttachedCompactGroupReleaseV1::new(raw_release_x, raw_release_y)
            .map_err(|_| attached_error(py, AttachedCompactGroupSessionErrorV1::InvalidPose))?;
        begin(
            &mut self.session,
            fence,
            target,
            AttachCompactGroupV1::new(key, release),
        )
        .map_err(|error| attached_error(py, error))
    }

    /// Return identifier-free renderer paint facts while the private candidate remains live.
    fn _preview_attach_compact_group_v1(
        &self,
        py: Python<'_>,
        pending: PyRef<'_, PyPendingAttachedCompactGroupV1>,
    ) -> PyResult<PyAttachedCompactGroupPreviewV1> {
        preview(py, &pending.pending)
    }

    /// Commit the private compact-group candidate once and return authoritative durable facts.
    fn _commit_attach_compact_group_v1(
        &mut self,
        py: Python<'_>,
        mut pending: PyRefMut<'_, PyPendingAttachedCompactGroupV1>,
    ) -> PyResult<PyAttachedCompactGroupCommitFactsV1> {
        commit(&mut self.session, &mut pending.pending).map_err(|error| attached_error(py, error))
    }

    /// Cancel one pending native-tab preview without exposing candidate state.
    fn _cancel_attach_compact_group_v1(
        &mut self,
        py: Python<'_>,
        mut pending: PyRefMut<'_, PyPendingAttachedCompactGroupV1>,
    ) -> PyResult<()> {
        cancel(&mut self.session, &mut pending.pending).map_err(|error| attached_error(py, error))
    }
}

fn begin(
    session: &mut DocumentSession,
    fence: DocumentFenceV1,
    target: AttachedCompactGroupTargetV1,
    request: AttachCompactGroupV1,
) -> Result<PyPendingAttachedCompactGroupV1, AttachedCompactGroupSessionErrorV1> {
    session
        .prepare_attach_compact_group_v1(fence, target, request)
        .map(|pending| PyPendingAttachedCompactGroupV1 { pending })
}

fn preview(
    py: Python<'_>,
    pending: &PendingAttachedCompactGroupV1,
) -> PyResult<PyAttachedCompactGroupPreviewV1> {
    let overlay = pending
        .precommit_overlay_v1()
        .ok_or_else(|| attached_error(py, AttachedCompactGroupSessionErrorV1::Consumed))?;
    let overlay = overlay_from(py, overlay)
        .map_err(|_| attached_error(py, AttachedCompactGroupSessionErrorV1::RendererAdmission))?;
    Ok(PyAttachedCompactGroupPreviewV1 { overlay })
}

fn commit(
    session: &mut DocumentSession,
    pending: &mut PendingAttachedCompactGroupV1,
) -> Result<PyAttachedCompactGroupCommitFactsV1, AttachedCompactGroupSessionErrorV1> {
    session
        .commit_attach_compact_group_v1(pending)
        .map(commit_facts)
}

fn cancel(
    session: &mut DocumentSession,
    pending: &mut PendingAttachedCompactGroupV1,
) -> Result<(), AttachedCompactGroupSessionErrorV1> {
    session.cancel_attach_compact_group_v1(pending)
}

fn commit_facts(result: AttachedCompactGroupCommitResultV1) -> PyAttachedCompactGroupCommitFactsV1 {
    let snapshot = result.observation().snapshot();
    PyAttachedCompactGroupCommitFactsV1 {
        revision: snapshot.revision(),
        digest: snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        is_dirty: snapshot.is_dirty(),
        focus_object_id: result.focus_object_id().as_str().to_owned(),
        compact_group_object_id: result.compact_group_object_id().as_str().to_owned(),
    }
}

fn availability_facts(
    result: AttachedCompactGroupAvailabilityV1,
) -> PyAttachedCompactGroupAvailabilityFactsV1 {
    PyAttachedCompactGroupAvailabilityFactsV1 {
        available: result.is_available(),
        category: availability_category(result.category()),
        revision: result.revision(),
        digest: result
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        anchor_object_id: result.anchor_object_id().as_str().to_owned(),
        catalog_key: result.catalog_key().as_str().to_owned(),
    }
}

fn availability_category(
    category: AttachedCompactGroupAvailabilityCategoryV1,
) -> PyAttachedCompactGroupAvailabilityCategoryV1 {
    match category {
        AttachedCompactGroupAvailabilityCategoryV1::Available => {
            PyAttachedCompactGroupAvailabilityCategoryV1::Available
        }
        AttachedCompactGroupAvailabilityCategoryV1::StaleRevision => {
            PyAttachedCompactGroupAvailabilityCategoryV1::StaleRevision
        }
        AttachedCompactGroupAvailabilityCategoryV1::StaleDigest => {
            PyAttachedCompactGroupAvailabilityCategoryV1::StaleDigest
        }
        AttachedCompactGroupAvailabilityCategoryV1::UnknownMolecule => {
            PyAttachedCompactGroupAvailabilityCategoryV1::UnknownMolecule
        }
        AttachedCompactGroupAvailabilityCategoryV1::UnknownAnchor => {
            PyAttachedCompactGroupAvailabilityCategoryV1::UnknownAnchor
        }
        AttachedCompactGroupAvailabilityCategoryV1::ForeignTarget => {
            PyAttachedCompactGroupAvailabilityCategoryV1::ForeignTarget
        }
        AttachedCompactGroupAvailabilityCategoryV1::CandidateAdmission => {
            PyAttachedCompactGroupAvailabilityCategoryV1::CandidateAdmission
        }
        AttachedCompactGroupAvailabilityCategoryV1::SessionConflict => {
            PyAttachedCompactGroupAvailabilityCategoryV1::SessionConflict
        }
    }
}

fn category(error: AttachedCompactGroupSessionErrorV1) -> PyAttachedCompactGroupCategoryV1 {
    match error {
        AttachedCompactGroupSessionErrorV1::StaleRevision => {
            PyAttachedCompactGroupCategoryV1::StaleRevision
        }
        AttachedCompactGroupSessionErrorV1::StaleDigest => {
            PyAttachedCompactGroupCategoryV1::StaleDigest
        }
        AttachedCompactGroupSessionErrorV1::ForeignSession => {
            PyAttachedCompactGroupCategoryV1::ForeignSession
        }
        AttachedCompactGroupSessionErrorV1::Consumed => PyAttachedCompactGroupCategoryV1::Consumed,
        AttachedCompactGroupSessionErrorV1::UnknownMolecule => {
            PyAttachedCompactGroupCategoryV1::UnknownMolecule
        }
        AttachedCompactGroupSessionErrorV1::UnknownAnchor => {
            PyAttachedCompactGroupCategoryV1::UnknownAnchor
        }
        AttachedCompactGroupSessionErrorV1::ForeignTarget => {
            PyAttachedCompactGroupCategoryV1::ForeignTarget
        }
        AttachedCompactGroupSessionErrorV1::InvalidPose => {
            PyAttachedCompactGroupCategoryV1::InvalidPose
        }
        AttachedCompactGroupSessionErrorV1::CandidateAdmission => {
            PyAttachedCompactGroupCategoryV1::CandidateAdmission
        }
        AttachedCompactGroupSessionErrorV1::RendererAdmission => {
            PyAttachedCompactGroupCategoryV1::RendererAdmission
        }
        AttachedCompactGroupSessionErrorV1::SessionConflict => {
            PyAttachedCompactGroupCategoryV1::SessionConflict
        }
    }
}

fn attached_error(py: Python<'_>, error: AttachedCompactGroupSessionErrorV1) -> PyErr {
    let exception = match error {
        AttachedCompactGroupSessionErrorV1::StaleRevision
        | AttachedCompactGroupSessionErrorV1::StaleDigest => {
            RevisionConflictError::new_err(error.to_string())
        }
        _ => AttachedCompactGroupAttachmentError::new_err(error.to_string()),
    };
    exception
        .value(py)
        .setattr(
            "category",
            Py::new(py, category(error)).expect("closed category enum allocates"),
        )
        .expect("attached-compact-group error category attaches");
    exception
}

fn parse_catalog_key(py: Python<'_>, value: &str) -> PyResult<CompactGroupCatalogKeyV1> {
    CompactGroupCatalogKeyV1::parse(value).ok_or_else(|| {
        attached_category_error(
            py,
            PyAttachedCompactGroupCategoryV1::InvalidCatalogKey,
            "catalog key is not recognized",
        )
    })
}

fn parse_digest(py: Python<'_>, value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(attached_category_error(
            py,
            PyAttachedCompactGroupCategoryV1::InvalidDigest,
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

fn attached_category_error(
    py: Python<'_>,
    category: PyAttachedCompactGroupCategoryV1,
    message: &str,
) -> PyErr {
    let exception = AttachedCompactGroupAttachmentError::new_err(message.to_owned());
    exception
        .value(py)
        .setattr(
            "category",
            Py::new(py, category).expect("closed category enum allocates"),
        )
        .expect("attached-compact-group error category attaches");
    exception
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "AttachedCompactGroupAttachmentError",
        module
            .py()
            .get_type::<AttachedCompactGroupAttachmentError>(),
    )?;
    module.add_class::<PyAttachedCompactGroupCategoryV1>()?;
    module.add_class::<PyAttachedCompactGroupAvailabilityCategoryV1>()?;
    module.add_class::<PyAttachedCompactGroupChoiceFactsV1>()?;
    module.add_class::<PyAttachedCompactGroupAvailabilityFactsV1>()?;
    module.add_class::<PyAttachedCompactGroupPreviewV1>()?;
    module.add_class::<PyAttachedCompactGroupCommitFactsV1>()?;
    Ok(())
}
