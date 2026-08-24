//! PyO3 facade for renderer-preflighted directed direct-bond V3 pointer probes.
//!
//! Clients submit Normal, Solid wedge, or Hashed wedge pointer probes and
//! receive Rust-issued operations at admission. A successful owner commit
//! consumes the opaque receipt once. A foreign-session refusal restores the
//! receipt for its originating session to retry and reports the V3
//! commit-result taxonomy.
//! `DirectBondSnapPolicyV1` is V3-shared configuration, while the V1 commit
//! category and recovery names remain domain-versioned result values.

use super::binding::PyDocumentBondPresentationV1;
use super::binding::PyDocumentSession;
use super::direct_bond_gesture_support::*;
use ferrum_document::DocumentFenceV1;
use pyo3::prelude::*;

#[pymethods]
impl PyDocumentSession {
    #[allow(clippy::too_many_arguments)]
    fn begin_direct_bond_gesture_v3(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        start: PyRef<'_, PyDirectBondPointerProbeV3>,
        presentation: PyRef<'_, PyDocumentBondPresentationV1>,
        new_atom_element: String,
        snap: PyRef<'_, PyDirectBondSnapPolicyV1>,
    ) -> PyResult<PyDirectBondGestureV3> {
        ferrum_document_render::begin_direct_bond_gesture_v3(
            &self.session,
            DocumentFenceV1::new(expected_revision, parse_digest(&expected_digest_hex)?),
            start.probe.clone(),
            (*presentation).into(),
            new_atom_element,
            snap.policy,
        )
        .map(|gesture| PyDirectBondGestureV3 { gesture })
        .map_err(|error| admission_error(py, error))
    }

    fn admit_direct_bond_candidate_v3(
        &mut self,
        py: Python<'_>,
        gesture: PyRef<'_, PyDirectBondGestureV3>,
        end: PyRef<'_, PyDirectBondPointerProbeV3>,
    ) -> PyResult<PyDirectBondAdmissionV3> {
        let admission = ferrum_document_render::admit_direct_bond_candidate_v3(
            &mut self.session,
            &gesture.gesture,
            end.probe.clone(),
        )
        .map_err(|error| admission_error(py, error))?;
        admission_v3_binding(py, admission)
    }

    fn commit_direct_bond_admission_v3(
        &mut self,
        py: Python<'_>,
        mut admission: PyRefMut<'_, PyDirectBondAdmissionV3>,
    ) -> PyResult<PyDirectBondCommitV3> {
        ferrum_document_render::commit_direct_bond_admission_v3(
            &mut self.session,
            &mut admission.admission,
        )
        .map(commit_v3_binding)
        .map_err(|error| commit_error(py, error))
    }
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "DirectBondGestureError",
        module.py().get_type::<DirectBondGestureError>(),
    )?;
    module.add(
        "DirectBondAdmissionRefusalV3",
        module.py().get_type::<DirectBondAdmissionRefusalV3>(),
    )?;
    module.add(
        "DirectBondCommitError",
        module.py().get_type::<DirectBondCommitError>(),
    )?;
    module.add(
        "DirectBondPointerProbeErrorV3",
        module.py().get_type::<DirectBondPointerProbeErrorV3>(),
    )?;
    module.add_class::<PyDirectBondPointerProbeCategoryV3>()?;
    module.add_class::<PyDirectBondPointerProbeRecoveryV3>()?;
    module.add_class::<PyDirectBondAdmissionCategoryV3>()?;
    module.add_class::<PyDirectBondAdmissionRecoveryV3>()?;
    module.add_class::<PyDirectBondPointerHitStateV3>()?;
    // V1 commit names remain the closed result taxonomy emitted by V3.
    module.add_class::<PyDirectBondCommitCategoryV1>()?;
    module.add_class::<PyDirectBondCommitRecoveryV1>()?;
    // V1 snap configuration is shared by the current V3 lifecycle.
    module.add_class::<PyDirectBondSnapPolicyV1>()?;
    module.add_class::<PyDirectBondViewportToSceneV3>()?;
    module.add_class::<PyDirectBondPointerProbeV3>()?;
    module.add_class::<PyDirectBondGestureV3>()?;
    module.add_class::<PyDirectBondOverlayV3>()?;
    module.add_class::<PyDirectBondAdmissionV3>()?;
    module.add_class::<PyDirectBondCommitV3>()?;
    Ok(())
}
