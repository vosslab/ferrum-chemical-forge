//! PyO3 registration facade for revision-fenced direct normal-bond gestures.

use super::direct_bond_gesture_support::*;

use super::binding::PyDocumentSession;
use pyo3::prelude::*;

#[pymethods]
impl PyDocumentSession {
    #[allow(clippy::too_many_arguments)]
    fn begin_direct_bond_gesture_v2(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        start: PyRef<'_, PyDirectBondEndpointIntentV2>,
        presentation: PyRef<'_, PyDocumentBondPresentationV1>,
        new_atom_element: String,
        snap: PyRef<'_, PyDirectBondSnapPolicyV1>,
    ) -> PyResult<PyDirectBondGestureV2> {
        self.session
            .begin_direct_bond_gesture_v2(
                DocumentFenceV1::new(expected_revision, parse_digest(&expected_digest_hex)?),
                start.intent.clone(),
                (*presentation).into(),
                new_atom_element,
                snap.policy,
            )
            .map(|gesture| PyDirectBondGestureV2 { gesture })
            .map_err(|error| direct_error(py, error))
    }

    fn admit_direct_bond_candidate_v2(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyDirectBondGestureV2>,
        end: PyRef<'_, PyDirectBondEndpointIntentV2>,
    ) -> PyResult<Py<PyAny>> {
        match self
            .session
            .admit_direct_bond_candidate_v2(&gesture.gesture, end.intent.clone())
        {
            Ok(admission) => {
                Py::new(py, admission_v2_binding(admission)).map(|value| value.into_any())
            }
            Err(
                error @ (DirectBondAdmissionRefusalV1::ForeignSession
                | DirectBondAdmissionRefusalV1::StaleRevision
                | DirectBondAdmissionRefusalV1::StaleDigest),
            ) => Err(admission_protocol_error(py, error)),
            Err(error) => Py::new(
                py,
                PyDirectBondAdmissionRefusalV1 {
                    category: Py::new(py, admission_category(&error))?,
                    recovery: Py::new(py, admission_recovery(&error))?,
                },
            )
            .map(|value| value.into_any()),
        }
    }

    fn commit_direct_bond_admission_v2(
        &mut self,
        py: Python<'_>,
        admission: PyRef<'_, PyDirectBondAdmissionV2>,
    ) -> PyResult<PyDirectBondCommitV2> {
        self.session
            .commit_direct_bond_admission_v2(&admission.admission)
            .map(commit_v2_binding)
            .map_err(|error| admission_commit_error(py, error))
    }

    #[allow(clippy::too_many_arguments)]
    fn begin_direct_bond_gesture_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        start_atom_object_id: String,
        presentation: PyRef<'_, PyDocumentBondPresentationV1>,
        new_atom_element: String,
        snap: PyRef<'_, PyDirectBondSnapPolicyV1>,
    ) -> PyResult<PyDirectBondGestureV1> {
        let fence = DocumentFenceV1::new(expected_revision, parse_digest(&expected_digest_hex)?);
        let start_atom = document_object_id(py, start_atom_object_id)?;
        self.session
            .begin_direct_bond_gesture_v1(
                fence,
                start_atom,
                (*presentation).into(),
                new_atom_element,
                snap.policy,
            )
            .map(|gesture| PyDirectBondGestureV1 { gesture })
            .map_err(|error| direct_error(py, error))
    }

    fn preview_direct_bond_gesture_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyDirectBondGestureV1>,
        end: PyRef<'_, PyDirectBondEndIntentV1>,
    ) -> PyResult<Py<PyAny>> {
        match self
            .session
            .preview_direct_bond_gesture_v1(&gesture.gesture, end.intent.clone())
        {
            Ok(preview) => Py::new(py, preview_binding(preview)).map(|value| value.into_any()),
            Err(
                error @ (DirectBondGestureErrorV1::SelfLoop
                | DirectBondGestureErrorV1::CrossMolecule
                | DirectBondGestureErrorV1::DuplicateBond),
            ) => Py::new(
                py,
                PyDirectBondPreviewRefusalV1 {
                    category: Py::new(py, category(&error))?,
                    recovery: Py::new(py, recovery(&error))?,
                },
            )
            .map(|value| value.into_any()),
            Err(error) => Err(direct_error(py, error)),
        }
    }

    fn admit_direct_bond_candidate_v1(
        &self,
        py: Python<'_>,
        gesture: PyRef<'_, PyDirectBondGestureV1>,
        end: PyRef<'_, PyDirectBondEndIntentV1>,
    ) -> PyResult<Py<PyAny>> {
        match self
            .session
            .admit_direct_bond_candidate_v1(&gesture.gesture, end.intent.clone())
        {
            Ok(admission) => {
                Py::new(py, admission_binding(admission)).map(|value| value.into_any())
            }
            Err(
                error @ (DirectBondAdmissionRefusalV1::ForeignSession
                | DirectBondAdmissionRefusalV1::StaleRevision
                | DirectBondAdmissionRefusalV1::StaleDigest),
            ) => Err(admission_protocol_error(py, error)),
            Err(error) => Py::new(
                py,
                PyDirectBondAdmissionRefusalV1 {
                    category: Py::new(py, admission_category(&error))?,
                    recovery: Py::new(py, admission_recovery(&error))?,
                },
            )
            .map(|value| value.into_any()),
        }
    }

    fn commit_direct_bond_admission_v1(
        &mut self,
        py: Python<'_>,
        admission: PyRef<'_, PyDirectBondAdmissionV1>,
    ) -> PyResult<PyDirectBondCommitV1> {
        self.session
            .commit_direct_bond_admission_v1(&admission.admission)
            .map(commit_binding)
            .map_err(|error| admission_commit_error(py, error))
    }

    fn commit_direct_bond_gesture_v1(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyDirectBondGestureV1>,
        preview: PyRef<'_, PyDirectBondPreviewV1>,
    ) -> PyResult<PyDirectBondCommitV1> {
        self.session
            .commit_direct_bond_gesture_v1(&gesture.gesture, &preview.preview)
            .map(commit_binding)
            .map_err(|error| direct_error(py, error))
    }
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DirectBondGestureError",
        module.py().get_type::<DirectBondGestureError>(),
    )?;
    module.add_class::<PyDirectBondGestureCategoryV1>()?;
    module.add_class::<PyDirectBondGestureRecoveryV1>()?;
    module.add_class::<PyDirectBondAdmissionCategoryV1>()?;
    module.add_class::<PyDirectBondCommitCategoryV1>()?;
    module.add_class::<PyDirectBondSnapPolicyV1>()?;
    module.add_class::<PyDirectBondEndIntentV1>()?;
    module.add_class::<PyDirectBondGestureV1>()?;
    module.add_class::<PyDirectBondOverlayV1>()?;
    module.add_class::<PyDirectBondPreviewV1>()?;
    module.add_class::<PyDirectBondAdmissionV1>()?;
    module.add_class::<PyDirectBondPreviewRefusalV1>()?;
    module.add_class::<PyDirectBondAdmissionRefusalV1>()?;
    module.add_class::<PyDirectBondCommitV1>()?;
    module.add_class::<PyDirectBondEndpointIntentV2>()?;
    module.add_class::<PyDirectBondGestureV2>()?;
    module.add_class::<PyDirectBondOverlayV2>()?;
    module.add_class::<PyDirectBondAdmissionV2>()?;
    module.add_class::<PyDirectBondCommitV2>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document::{DirectBondSnapPolicyV1, DocumentSession};

    const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"C\"><point x=\"40\" y=\"0\"/></atom></molecule></cdml>";

    fn digest(session: &DocumentSession) -> String {
        session
            .snapshot()
            .expect("snapshot")
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn atom_ids(session: &DocumentSession) -> (String, String) {
        let observation = session.observe(0).expect("observation");
        let object_id = |source_id: &str| -> String {
            observation
                .projection()
                .molecules()
                .iter()
                .flat_map(|molecule| molecule.atoms())
                .find(|atom| atom.source_id() == Some(source_id))
                .expect("projected atom has expected source ID")
                .id()
                .expect("projected atom has canonical ID")
                .as_str()
                .to_owned()
        };
        (object_id("a"), object_id("b"))
    }

    fn bind_session(py: Python<'_>, session: DocumentSession) -> Py<PyDocumentSession> {
        Py::new(py, PyDocumentSession::from_session(session)).expect("session binds")
    }

    fn begin(
        py: Python<'_>,
        session: &Bound<'_, PyDocumentSession>,
        revision: u64,
        digest: &str,
        start: &str,
    ) -> Py<PyAny> {
        begin_with_presentation(
            py,
            session,
            revision,
            digest,
            start,
            PyDocumentBondPresentationV1::NormalSingle,
        )
    }

    fn begin_with_presentation(
        py: Python<'_>,
        session: &Bound<'_, PyDocumentSession>,
        revision: u64,
        digest: &str,
        start: &str,
        presentation: PyDocumentBondPresentationV1,
    ) -> Py<PyAny> {
        let presentation = Py::new(py, presentation).expect("presentation binds");
        let snap = Py::new(
            py,
            PyDirectBondSnapPolicyV1 {
                policy: DirectBondSnapPolicyV1::free(),
            },
        )
        .expect("snap policy binds");
        session
            .call_method1(
                "begin_direct_bond_gesture_v1",
                (revision, digest, start, presentation, "C", snap),
            )
            .expect("gesture begins")
            .unbind()
    }

    fn existing_end(_py: Python<'_>, module: &Bound<'_, PyModule>, identifier: &str) -> Py<PyAny> {
        module
            .getattr("DirectBondEndIntentV1")
            .expect("end type registers")
            .call_method1("existing_atom", (identifier,))
            .expect("end binds")
            .unbind()
    }

    fn new_end(module: &Bound<'_, PyModule>) -> Py<PyAny> {
        module
            .getattr("DirectBondEndIntentV1")
            .expect("end type registers")
            .call_method1("new_atom_at", (40.0, 0.0))
            .expect("new endpoint binds")
            .unbind()
    }

    fn v2_existing_end(module: &Bound<'_, PyModule>, identifier: &str) -> Py<PyAny> {
        module
            .getattr("DirectBondEndpointIntentV2")
            .expect("v2 endpoint type registers")
            .call_method1("existing_atom", (identifier,))
            .expect("v2 existing endpoint binds")
            .unbind()
    }

    fn v2_new_end(module: &Bound<'_, PyModule>, x: f64, y: f64) -> Py<PyAny> {
        module
            .getattr("DirectBondEndpointIntentV2")
            .expect("v2 endpoint type registers")
            .call_method1("new_atom_at", (x, y))
            .expect("v2 new endpoint binds")
            .unbind()
    }

    fn begin_v2(
        py: Python<'_>,
        session: &Bound<'_, PyDocumentSession>,
        revision: u64,
        digest: &str,
        start: Py<PyAny>,
        presentation: PyDocumentBondPresentationV1,
    ) -> Py<PyAny> {
        let presentation = Py::new(py, presentation).expect("presentation binds");
        let snap = Py::new(
            py,
            PyDirectBondSnapPolicyV1 {
                policy: DirectBondSnapPolicyV1::free(),
            },
        )
        .expect("snap policy binds");
        session
            .call_method1(
                "begin_direct_bond_gesture_v2",
                (revision, digest, start, presentation, "C", snap),
            )
            .expect("v2 gesture begins")
            .unbind()
    }

    #[test]
    fn registered_direct_bond_admission_preserves_opaque_receipts_and_typed_refusals() {
        Python::initialize();
        Python::attach(|py| {
            let module = PyModule::new(py, "ferrum_chem").expect("extension module");
            super::super::binding::initialize(&module).expect("extension module registers");

            let owner_document = DocumentSession::load(SOURCE).expect("owner loads");
            let (start, end) = atom_ids(&owner_document);
            let revision = owner_document.snapshot().expect("snapshot").revision();
            let expected_digest = digest(&owner_document);
            let owner = bind_session(py, owner_document);
            let owner = owner.bind(py);
            let foreign = bind_session(py, DocumentSession::load(SOURCE).expect("foreign loads"));
            let foreign = foreign.bind(py);
            let gesture = begin(py, owner, revision, &expected_digest, &start);
            let end_intent = existing_end(py, &module, &end);

            let foreign_error = foreign
                .call_method1("admit_direct_bond_candidate_v1", (&gesture, &end_intent))
                .expect_err("foreign gesture refuses");
            let foreign_value = foreign_error.value(py);
            assert_eq!(
                *foreign_value
                    .getattr("category")
                    .expect("category attaches")
                    .extract::<PyRef<'_, PyDirectBondAdmissionCategoryV1>>()
                    .expect("category stays closed"),
                PyDirectBondAdmissionCategoryV1::ForeignSession
            );
            assert_eq!(
                *foreign_value
                    .getattr("recovery")
                    .expect("recovery attaches")
                    .extract::<PyRef<'_, PyDirectBondGestureRecoveryV1>>()
                    .expect("recovery stays closed"),
                PyDirectBondGestureRecoveryV1::RefreshAndRestart
            );

            let admission = owner
                .call_method1("admit_direct_bond_candidate_v1", (&gesture, &end_intent))
                .expect("candidate admits");
            assert!(admission.get_type().call0().is_err());
            for forbidden in ["candidate", "fence", "capability", "session_origin"] {
                assert!(admission.getattr(forbidden).is_err());
            }
            assert!(
                py.import("pickle")
                    .expect("standard pickle module")
                    .call_method1("dumps", (&admission,))
                    .is_err()
            );

            owner
                .call_method1("commit_direct_bond_admission_v1", (&admission,))
                .expect("receipt commits");
            let stale_error = owner
                .call_method1("admit_direct_bond_candidate_v1", (&gesture, &end_intent))
                .expect_err("stale gesture refuses");
            let stale_value = stale_error.value(py);
            assert_eq!(
                *stale_value
                    .getattr("category")
                    .expect("category attaches")
                    .extract::<PyRef<'_, PyDirectBondAdmissionCategoryV1>>()
                    .expect("category stays closed"),
                PyDirectBondAdmissionCategoryV1::StaleRevision
            );
            assert_eq!(
                *stale_value
                    .getattr("recovery")
                    .expect("recovery attaches")
                    .extract::<PyRef<'_, PyDirectBondGestureRecoveryV1>>()
                    .expect("recovery stays closed"),
                PyDirectBondGestureRecoveryV1::RefreshAndRestart
            );

            let legacy_document = DocumentSession::load(SOURCE).expect("legacy loads");
            let (legacy_start, legacy_end) = atom_ids(&legacy_document);
            let legacy_digest = digest(&legacy_document);
            let legacy = bind_session(py, legacy_document);
            let legacy = legacy.bind(py);
            let first = begin(py, legacy, 0, &legacy_digest, &legacy_start);
            let second = begin(py, legacy, 0, &legacy_digest, &legacy_start);
            let legacy_end = existing_end(py, &module, &legacy_end);
            let preview = legacy
                .call_method1("preview_direct_bond_gesture_v1", (&first, &legacy_end))
                .expect("preview succeeds");
            let mismatch = legacy
                .call_method1("commit_direct_bond_gesture_v1", (&second, &preview))
                .expect_err("mixed handles refuse");
            let mismatch_value = mismatch.value(py);
            assert_eq!(
                *mismatch_value
                    .getattr("category")
                    .expect("category attaches")
                    .extract::<PyRef<'_, PyDirectBondGestureCategoryV1>>()
                    .expect("category stays closed"),
                PyDirectBondGestureCategoryV1::PreviewMismatch
            );
            assert_eq!(
                *mismatch_value
                    .getattr("recovery")
                    .expect("recovery attaches")
                    .extract::<PyRef<'_, PyDirectBondGestureRecoveryV1>>()
                    .expect("recovery stays closed"),
                PyDirectBondGestureRecoveryV1::ReportConflict
            );
        });
    }

    #[test]
    fn registered_direct_bond_normal_order_mapping_covers_both_endpoint_modes() {
        Python::initialize();
        Python::attach(|py| {
            let module = PyModule::new(py, "ferrum_chem").expect("extension module");
            super::super::binding::initialize(&module).expect("extension module registers");

            for (presentation, expected) in [
                (PyDocumentBondPresentationV1::NormalSingle, "normal_single"),
                (PyDocumentBondPresentationV1::NormalDouble, "normal_double"),
                (PyDocumentBondPresentationV1::NormalTriple, "normal_triple"),
            ] {
                for endpoint_is_new in [false, true] {
                    let session = DocumentSession::load(SOURCE).expect("session loads");
                    let (start, end) = atom_ids(&session);
                    let revision = session.snapshot().expect("snapshot").revision();
                    let expected_digest = digest(&session);
                    let session = bind_session(py, session);
                    let session = session.bind(py);
                    let gesture = begin_with_presentation(
                        py,
                        session,
                        revision,
                        &expected_digest,
                        &start,
                        presentation,
                    );
                    let end = if endpoint_is_new {
                        new_end(&module)
                    } else {
                        existing_end(py, &module, &end)
                    };
                    let admission = session
                        .call_method1("admit_direct_bond_candidate_v1", (&gesture, &end))
                        .expect("normal candidate admits through Python");
                    let overlay = admission.getattr("overlay").expect("overlay exposes");
                    assert_eq!(
                        overlay
                            .getattr("presentation")
                            .expect("presentation exposes")
                            .extract::<String>()
                            .expect("presentation is text"),
                        expected
                    );
                    assert_eq!(
                        overlay
                            .getattr("endpoint_is_new")
                            .expect("endpoint mode exposes")
                            .extract::<bool>()
                            .expect("endpoint mode is boolean"),
                        endpoint_is_new
                    );
                    let commit = session
                        .call_method1("commit_direct_bond_admission_v1", (&admission,))
                        .expect("admission commits through Python");
                    assert_eq!(
                        commit
                            .getattr("created_new_atom")
                            .expect("commit endpoint mode exposes")
                            .extract::<bool>()
                            .expect("commit endpoint mode is boolean"),
                        endpoint_is_new
                    );
                }
            }
        });
    }

    #[test]
    fn registered_direct_bond_v2_receipts_preserve_creation_and_release_endpoint_facts() {
        Python::initialize();
        Python::attach(|py| {
            let module = PyModule::new(py, "ferrum_chem").expect("extension module");
            super::super::binding::initialize(&module).expect("extension module registers");

            for (presentation, expected_presentation) in [
                (PyDocumentBondPresentationV1::NormalSingle, "normal_single"),
                (PyDocumentBondPresentationV1::NormalDouble, "normal_double"),
                (PyDocumentBondPresentationV1::NormalTriple, "normal_triple"),
            ] {
                for (form, start_is_new, end_is_new, expected_created, expected_molecule) in [
                    ("existing_existing", false, false, false, false),
                    ("existing_new", false, true, true, false),
                    ("new_existing", true, false, true, false),
                    ("new_new", true, true, true, true),
                ] {
                    let document = DocumentSession::load(if form == "new_new" {
                        "<cdml xmlns=\"urn:ferrum:cdml\"/>"
                    } else {
                        SOURCE
                    })
                    .expect("session loads");
                    let endpoint_ids = if form == "new_new" {
                        None
                    } else {
                        Some(atom_ids(&document))
                    };
                    let revision = document.snapshot().expect("snapshot").revision();
                    let expected_digest = digest(&document);
                    let session = bind_session(py, document);
                    let session = session.bind(py);
                    let start = if start_is_new {
                        v2_new_end(&module, 80.0, 0.0)
                    } else {
                        v2_existing_end(
                            &module,
                            &endpoint_ids.as_ref().expect("existing start has IDs").0,
                        )
                    };
                    let end = if end_is_new {
                        v2_new_end(&module, 40.0, 0.0)
                    } else {
                        v2_existing_end(
                            &module,
                            &endpoint_ids.as_ref().expect("existing end has IDs").1,
                        )
                    };
                    let gesture =
                        begin_v2(py, session, revision, &expected_digest, start, presentation);
                    let admission = session
                        .call_method1("admit_direct_bond_candidate_v2", (&gesture, &end))
                        .expect("v2 candidate admits");
                    assert_eq!(
                        admission
                            .getattr("overlay")
                            .expect("overlay exposes")
                            .getattr("presentation")
                            .expect("presentation exposes")
                            .extract::<String>()
                            .expect("presentation is text"),
                        expected_presentation
                    );
                    for forbidden in ["candidate", "fence", "capability", "session_origin"] {
                        assert!(admission.getattr(forbidden).is_err());
                    }
                    let receipt = session
                        .call_method1("commit_direct_bond_admission_v2", (&admission,))
                        .expect("v2 candidate commits");
                    assert_eq!(
                        receipt
                            .getattr("created_new_atom")
                            .expect("creation fact exposes")
                            .extract::<bool>()
                            .expect("creation fact is boolean"),
                        expected_created
                    );
                    assert_eq!(
                        receipt
                            .getattr("created_new_molecule")
                            .expect("molecule fact exposes")
                            .extract::<bool>()
                            .expect("molecule fact is boolean"),
                        expected_molecule
                    );
                    if matches!(form, "existing_existing" | "new_existing") {
                        assert_eq!(
                            receipt
                                .getattr("end_atom_identifier")
                                .expect("release endpoint exposes")
                                .extract::<String>()
                                .expect("release endpoint is text"),
                            "b"
                        );
                    }
                }
            }
        });
    }

    #[test]
    fn registered_direct_bond_v2_admission_is_opaque_and_refusal_is_mutation_free() {
        Python::initialize();
        Python::attach(|py| {
            let module = PyModule::new(py, "ferrum_chem").expect("extension module");
            super::super::binding::initialize(&module).expect("extension module registers");

            let document = DocumentSession::load(SOURCE).expect("session loads");
            let (start, end) = atom_ids(&document);
            let revision = document.snapshot().expect("snapshot").revision();
            let expected_digest = digest(&document);
            let session = bind_session(py, document);
            let session = session.bind(py);
            let start = v2_existing_end(&module, &start);
            let valid_end = v2_existing_end(&module, &end);
            let gesture = begin_v2(
                py,
                session,
                revision,
                &expected_digest,
                start.clone_ref(py),
                PyDocumentBondPresentationV1::NormalSingle,
            );
            let admission = session
                .call_method1("admit_direct_bond_candidate_v2", (&gesture, &valid_end))
                .expect("candidate admits");
            assert!(admission.get_type().call0().is_err());
            for forbidden in ["candidate", "fence", "capability", "session_origin"] {
                assert!(admission.getattr(forbidden).is_err());
            }

            let before = session
                .call_method0("snapshot")
                .expect("snapshot before refusal");
            let refusal = session
                .call_method1("admit_direct_bond_candidate_v2", (&gesture, &start))
                .expect("collapsed endpoint returns typed refusal");
            assert_eq!(
                *refusal
                    .getattr("category")
                    .expect("category attaches")
                    .extract::<PyRef<'_, PyDirectBondAdmissionCategoryV1>>()
                    .expect("category stays closed"),
                PyDirectBondAdmissionCategoryV1::CollapsedEndpoint
            );
            assert_eq!(
                refusal
                    .getattr("recovery")
                    .expect("recovery attaches")
                    .extract::<PyRef<'_, PyDirectBondGestureRecoveryV1>>()
                    .expect("recovery stays closed")
                    .clone(),
                PyDirectBondGestureRecoveryV1::AdjustEndpoint
            );
            let after = session
                .call_method0("snapshot")
                .expect("snapshot after refusal");
            assert_eq!(
                before
                    .getattr("revision")
                    .expect("revision exposes")
                    .extract::<u64>()
                    .expect("revision is numeric"),
                after
                    .getattr("revision")
                    .expect("revision exposes")
                    .extract::<u64>()
                    .expect("revision is numeric")
            );
            assert_eq!(
                before
                    .getattr("cdml")
                    .expect("CDML exposes")
                    .extract::<String>()
                    .expect("CDML is text"),
                after
                    .getattr("cdml")
                    .expect("CDML exposes")
                    .extract::<String>()
                    .expect("CDML is text")
            );
        });
    }
}
