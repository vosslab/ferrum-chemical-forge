//! PyO3 facade for renderer-preflighted directed direct-bond V3 pointer probes.
//!
//! Clients submit Normal, Solid wedge, or Hashed wedge pointer probes and
//! receive a generic renderer-admitted transition at admission. The owning
//! document session is the sole authority that can redeem that opaque receipt.
//! `DirectBondSnapPolicyV1` is V3-shared configuration.

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
        .map(PyDirectBondGestureV3::from_renderer_gesture)
        .map_err(|error| admission_error(py, error))
    }
}

#[pymethods]
impl PyDirectBondGestureV3 {
    fn resolve_end_v3(
        &mut self,
        py: Python<'_>,
        session: PyRefMut<'_, PyDocumentSession>,
        end: PyRef<'_, PyDirectBondPointerProbeV3>,
    ) -> PyResult<super::prepared_transition_binding::PySessionOperationTransitionRequestV1> {
        let gesture = self.take_for_resolution()?;
        let request = ferrum_document_render::resolve_direct_bond_end_v3(
            &session.session,
            gesture,
            end.probe.clone(),
        )
        .map_err(|error| admission_error(py, error))?;
        Ok(
            super::prepared_transition_binding::PySessionOperationTransitionRequestV1::from_request(
                request,
            ),
        )
    }
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    super::prepared_transition_binding::initialize(module)?;
    module.add(
        "DirectBondGestureError",
        module.py().get_type::<DirectBondGestureError>(),
    )?;
    module.add(
        "DirectBondAdmissionRefusalV3",
        module.py().get_type::<DirectBondAdmissionRefusalV3>(),
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
    // V1 snap configuration is shared by the current V3 lifecycle.
    module.add_class::<PyDirectBondSnapPolicyV1>()?;
    module.add_class::<PyDirectBondViewportToSceneV3>()?;
    module.add_class::<PyDirectBondPointerProbeV3>()?;
    module.add_class::<PyDirectBondGestureV3>()?;
    Ok(())
}
