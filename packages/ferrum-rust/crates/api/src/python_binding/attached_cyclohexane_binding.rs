//! Private opaque PyO3 seam for one atom-anchored cyclohexane attachment.

use ferrum_document::{
    AttachedCyclohexaneReleaseV1, AttachedCyclohexaneSessionErrorV1, DocumentFenceV1,
    DocumentObjectIdV1, DocumentSession, PendingAttachedCyclohexaneV1,
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
    AttachedCyclohexaneAttachmentError,
    super::document_error_binding::DocumentError
);

/// Closed refusal facts for the one private atom-anchored C6 operation.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "AttachedCyclohexaneCategoryV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PyAttachedCyclohexaneCategoryV1 {
    StaleRevision,
    StaleDigest,
    ForeignSession,
    Retired,
    UnknownAnchor,
    IneligibleAnchor,
    InvalidPose,
    RendererAdmission,
    SessionConflict,
}

/// Opaque one-use candidate retained by the native tab only.
#[pyclass(
    unsendable,
    module = "ferrum_chem",
    name = "PendingAttachedCyclohexaneV1"
)]
pub(crate) struct PyPendingAttachedCyclohexaneV1 {
    pending: PendingAttachedCyclohexaneV1,
}

/// Immutable renderer-issued C6 overlay for native-tab paint-only replay.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "AttachedCyclohexanePreviewV1",
    skip_from_py_object
)]
pub(crate) struct PyAttachedCyclohexanePreviewV1 {
    #[pyo3(get)]
    overlay: PyDocumentPrecommitOverlayV1,
}

/// Identity-free authoritative facts after the one committed transition.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "AttachedCyclohexaneCommitFactsV1",
    skip_from_py_object
)]
pub(crate) struct PyAttachedCyclohexaneCommitFactsV1 {
    #[pyo3(get)]
    revision: u64,
    #[pyo3(get)]
    is_dirty: bool,
}

#[pymethods]
impl PyDocumentSession {
    /// Begin the private native-tab C6 attach operation from durable target and pointer facts.
    #[allow(clippy::too_many_arguments)]
    fn _begin_attach_cyclohexane_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        anchor_object_id: String,
        raw_release_x: f64,
        raw_release_y: f64,
    ) -> PyResult<PyPendingAttachedCyclohexaneV1> {
        let fence = DocumentFenceV1::new(expected_revision, parse_digest(&expected_digest_hex)?);
        let anchor = document_object_id(py, anchor_object_id)?;
        let release = AttachedCyclohexaneReleaseV1::new(raw_release_x, raw_release_y)
            .map_err(|_| attached_error(py, AttachedCyclohexaneSessionErrorV1::InvalidPose))?;
        begin(&mut self.session, fence, anchor, release).map_err(|error| attached_error(py, error))
    }

    /// Return identifier-free renderer paint facts while the private candidate remains live.
    fn _preview_attach_cyclohexane_v1(
        &self,
        py: Python<'_>,
        pending: PyRef<'_, PyPendingAttachedCyclohexaneV1>,
    ) -> PyResult<PyAttachedCyclohexanePreviewV1> {
        preview(py, &pending.pending)
    }

    /// Commit the private C6 candidate once and return identity-free outcome facts.
    fn _commit_attach_cyclohexane_v1(
        &mut self,
        py: Python<'_>,
        mut pending: PyRefMut<'_, PyPendingAttachedCyclohexaneV1>,
    ) -> PyResult<PyAttachedCyclohexaneCommitFactsV1> {
        commit(&mut self.session, &mut pending.pending).map_err(|error| attached_error(py, error))
    }

    /// Retire one native-tab preview without exposing candidate state.
    fn _cancel_attach_cyclohexane_v1(
        &mut self,
        py: Python<'_>,
        mut pending: PyRefMut<'_, PyPendingAttachedCyclohexaneV1>,
    ) -> PyResult<()> {
        cancel(&mut self.session, &mut pending.pending).map_err(|error| attached_error(py, error))
    }
}

fn begin(
    session: &mut DocumentSession,
    fence: DocumentFenceV1,
    anchor: DocumentObjectIdV1,
    release: AttachedCyclohexaneReleaseV1,
) -> Result<PyPendingAttachedCyclohexaneV1, AttachedCyclohexaneSessionErrorV1> {
    session
        .prepare_attach_cyclohexane_v1(fence, anchor, release)
        .map(|pending| PyPendingAttachedCyclohexaneV1 { pending })
}

fn preview(
    py: Python<'_>,
    pending: &PendingAttachedCyclohexaneV1,
) -> PyResult<PyAttachedCyclohexanePreviewV1> {
    let overlay = pending
        .precommit_overlay_v1()
        .ok_or_else(|| attached_error(py, AttachedCyclohexaneSessionErrorV1::Retired))?;
    let overlay = overlay_from(py, overlay)
        .map_err(|_| attached_error(py, AttachedCyclohexaneSessionErrorV1::RendererAdmission))?;
    Ok(PyAttachedCyclohexanePreviewV1 { overlay })
}

fn commit(
    session: &mut DocumentSession,
    pending: &mut PendingAttachedCyclohexaneV1,
) -> Result<PyAttachedCyclohexaneCommitFactsV1, AttachedCyclohexaneSessionErrorV1> {
    session
        .commit_attach_cyclohexane_v1(pending)
        .map(|result| PyAttachedCyclohexaneCommitFactsV1 {
            revision: result.observation().snapshot().revision(),
            is_dirty: result.observation().snapshot().is_dirty(),
        })
}

fn cancel(
    session: &mut DocumentSession,
    pending: &mut PendingAttachedCyclohexaneV1,
) -> Result<(), AttachedCyclohexaneSessionErrorV1> {
    session.retire_attach_cyclohexane_v1(pending)
}

fn category(error: AttachedCyclohexaneSessionErrorV1) -> PyAttachedCyclohexaneCategoryV1 {
    match error {
        AttachedCyclohexaneSessionErrorV1::StaleRevision => {
            PyAttachedCyclohexaneCategoryV1::StaleRevision
        }
        AttachedCyclohexaneSessionErrorV1::StaleDigest => {
            PyAttachedCyclohexaneCategoryV1::StaleDigest
        }
        AttachedCyclohexaneSessionErrorV1::ForeignSession => {
            PyAttachedCyclohexaneCategoryV1::ForeignSession
        }
        AttachedCyclohexaneSessionErrorV1::Retired => PyAttachedCyclohexaneCategoryV1::Retired,
        AttachedCyclohexaneSessionErrorV1::UnknownAnchor => {
            PyAttachedCyclohexaneCategoryV1::UnknownAnchor
        }
        AttachedCyclohexaneSessionErrorV1::IneligibleAnchor => {
            PyAttachedCyclohexaneCategoryV1::IneligibleAnchor
        }
        AttachedCyclohexaneSessionErrorV1::InvalidPose => {
            PyAttachedCyclohexaneCategoryV1::InvalidPose
        }
        AttachedCyclohexaneSessionErrorV1::RendererAdmission => {
            PyAttachedCyclohexaneCategoryV1::RendererAdmission
        }
        AttachedCyclohexaneSessionErrorV1::SessionConflict => {
            PyAttachedCyclohexaneCategoryV1::SessionConflict
        }
    }
}

fn attached_error(py: Python<'_>, error: AttachedCyclohexaneSessionErrorV1) -> PyErr {
    let exception = match error {
        AttachedCyclohexaneSessionErrorV1::StaleRevision
        | AttachedCyclohexaneSessionErrorV1::StaleDigest => {
            RevisionConflictError::new_err(error.to_string())
        }
        _ => AttachedCyclohexaneAttachmentError::new_err(error.to_string()),
    };
    exception
        .value(py)
        .setattr(
            "category",
            Py::new(py, category(error)).expect("closed category enum allocates"),
        )
        .expect("attached-cyclohexane error category attaches");
    exception
}

fn parse_digest(value: &str) -> PyResult<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(AttachedCyclohexaneAttachmentError::new_err(
            "expected digest must be exactly 64 lowercase hexadecimal characters",
        ));
    }
    let mut digest = [0; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
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

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "AttachedCyclohexaneAttachmentError",
        module.py().get_type::<AttachedCyclohexaneAttachmentError>(),
    )?;
    module.add_class::<PyAttachedCyclohexaneCategoryV1>()?;
    module.add_class::<PyAttachedCyclohexanePreviewV1>()?;
    module.add_class::<PyAttachedCyclohexaneCommitFactsV1>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    fn anchor(session: &DocumentSession) -> DocumentObjectIdV1 {
        session
            .observe(0)
            .expect("observation")
            .projection()
            .molecules()[0]
            .atoms()[0]
            .id()
            .expect("direct atom")
            .clone()
    }

    fn release() -> AttachedCyclohexaneReleaseV1 {
        AttachedCyclohexaneReleaseV1::new(40.0, 0.0).expect("finite release")
    }

    fn snapshot_facts(session: &pyo3::Bound<'_, PyDocumentSession>) -> (String, u64, String, bool) {
        let snapshot = session.call_method0("snapshot").expect("session snapshot");
        (
            snapshot
                .getattr("cdml")
                .expect("snapshot CDML")
                .extract::<String>()
                .expect("text CDML"),
            snapshot
                .getattr("revision")
                .expect("snapshot revision")
                .extract::<u64>()
                .expect("numeric revision"),
            snapshot
                .getattr("digest")
                .expect("snapshot digest")
                .extract::<String>()
                .expect("text digest"),
            snapshot
                .getattr("is_dirty")
                .expect("snapshot dirty state")
                .extract::<bool>()
                .expect("boolean dirty state"),
        )
    }

    #[test]
    fn private_bridge_refuses_foreign_retired_replayed_and_stale_handles_before_mutation() {
        Python::initialize();
        let mut owner = DocumentSession::load(SOURCE).expect("owner loads");
        let mut foreign = DocumentSession::load(SOURCE).expect("foreign loads");
        let owner_before = owner.snapshot().expect("owner snapshot");
        let foreign_before = foreign.snapshot().expect("foreign snapshot");
        let owner_fence = fence(&owner);
        let owner_anchor = anchor(&owner);
        let mut pending = begin(&mut owner, owner_fence, owner_anchor, release()).expect("begin");
        Python::attach(|py| {
            preview(py, &pending.pending).expect("renderer plan preview");
        });
        assert!(matches!(
            commit(&mut foreign, &mut pending.pending),
            Err(AttachedCyclohexaneSessionErrorV1::ForeignSession)
        ));
        assert_eq!(
            foreign.snapshot().expect("foreign unchanged"),
            foreign_before
        );
        cancel(&mut owner, &mut pending.pending).expect("cancel");
        assert_eq!(owner.snapshot().expect("cancel unchanged"), owner_before);
        Python::attach(|py| match preview(py, &pending.pending) {
            Err(error) => assert!(error.is_instance_of::<AttachedCyclohexaneAttachmentError>(py)),
            Ok(_) => panic!("retired candidate must refuse preview"),
        });
        assert!(matches!(
            commit(&mut owner, &mut pending.pending),
            Err(AttachedCyclohexaneSessionErrorV1::Retired)
        ));

        let current_fence = fence(&owner);
        let current_anchor = anchor(&owner);
        let mut first =
            begin(&mut owner, current_fence, current_anchor.clone(), release()).expect("first");
        let mut stale = begin(&mut owner, current_fence, current_anchor, release()).expect("stale");
        let committed = commit(&mut owner, &mut first.pending).expect("commit");
        assert_eq!(committed.revision, owner_before.revision() + 1);
        let after = owner.snapshot().expect("accepted transition");
        assert!(matches!(
            commit(&mut owner, &mut stale.pending),
            Err(AttachedCyclohexaneSessionErrorV1::StaleRevision)
        ));
        assert_eq!(owner.snapshot().expect("stale unchanged"), after);
    }

    #[test]
    fn registered_bridge_refuses_digest_anchor_foreign_stale_and_replayed_receipts() {
        Python::initialize();
        Python::attach(|py| {
            let module = PyModule::new(py, "ferrum_chem").expect("extension module");
            super::super::binding::initialize(&module).expect("extension module registers");

            let owner_document = DocumentSession::load(SOURCE).expect("owner loads");
            let owner_fence = fence(&owner_document);
            let owner_anchor = anchor(&owner_document).as_str().to_owned();
            let owner =
                Py::new(py, PyDocumentSession::from_session(owner_document)).expect("owner binds");
            let owner = owner.bind(py);

            let foreign_document = DocumentSession::load(SOURCE).expect("foreign loads");
            let foreign = Py::new(py, PyDocumentSession::from_session(foreign_document))
                .expect("foreign binds");
            let foreign = foreign.bind(py);

            let revision = owner_fence.revision();
            let digest: String = owner_fence
                .digest()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            let before = owner
                .call_method0("snapshot")
                .expect("owner snapshot")
                .getattr("revision")
                .expect("snapshot revision")
                .extract::<u64>()
                .expect("numeric revision");

            let wrong_digest = if digest == "0".repeat(64) {
                "1".repeat(64)
            } else {
                "0".repeat(64)
            };
            assert!(
                owner
                    .call_method1(
                        "_begin_attach_cyclohexane_v1",
                        (revision, wrong_digest, owner_anchor.clone(), 40.0, 0.0),
                    )
                    .is_err()
            );
            assert!(
                owner
                    .call_method1(
                        "_begin_attach_cyclohexane_v1",
                        (revision, digest.clone(), "missing-anchor", 40.0, 0.0),
                    )
                    .is_err()
            );
            assert!(
                owner
                    .call_method1(
                        "_begin_attach_cyclohexane_v1",
                        (
                            revision + 1,
                            digest.clone(),
                            owner_anchor.clone(),
                            40.0,
                            0.0
                        ),
                    )
                    .is_err()
            );
            assert_eq!(
                owner
                    .call_method0("snapshot")
                    .expect("refusals leave session available")
                    .getattr("revision")
                    .expect("snapshot revision")
                    .extract::<u64>()
                    .expect("numeric revision"),
                before
            );

            let first = owner
                .call_method1(
                    "_begin_attach_cyclohexane_v1",
                    (revision, digest.clone(), owner_anchor.clone(), 40.0, 0.0),
                )
                .expect("owner begins");
            let owner_before_foreign_commit = snapshot_facts(owner);
            let foreign_before_commit = snapshot_facts(foreign);
            assert!(
                foreign
                    .call_method1("_commit_attach_cyclohexane_v1", (&first,))
                    .is_err()
            );
            assert_eq!(snapshot_facts(owner), owner_before_foreign_commit);
            assert_eq!(snapshot_facts(foreign), foreign_before_commit);
            let committed = owner
                .call_method1("_commit_attach_cyclohexane_v1", (&first,))
                .expect("owner commits");
            assert_eq!(
                committed
                    .getattr("revision")
                    .expect("commit revision")
                    .extract::<u64>()
                    .expect("numeric revision"),
                before + 1
            );
            let owner_after_commit = snapshot_facts(owner);
            assert!(
                owner
                    .call_method1("_commit_attach_cyclohexane_v1", (&first,))
                    .is_err()
            );
            assert_eq!(snapshot_facts(owner), owner_after_commit);
        });
    }
}
