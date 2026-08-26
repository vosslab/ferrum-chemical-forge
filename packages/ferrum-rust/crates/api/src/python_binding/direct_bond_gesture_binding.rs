//! PyO3 facade for renderer-preflighted directed direct-bond pointer probes.
//!
//! Clients submit Normal, Solid wedge, or Hashed wedge pointer probes and
//! receive a generic renderer-admitted transition at admission. The owning
//! document session is the sole authority that can redeem that opaque receipt.
//! `DirectBondSnapPolicyV1` is shared configuration.

use super::binding::PyDocumentBondPresentationV1;
use super::binding::PyDocumentSession;
use super::direct_bond_gesture_support::*;
use ferrum_document::DocumentFenceV1;
use pyo3::prelude::*;

#[pymethods]
impl PyDocumentSession {
    #[allow(clippy::too_many_arguments)]
    fn begin_direct_bond_gesture(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        start: PyRef<'_, PyDirectBondPointerProbe>,
        presentation: PyRef<'_, PyDocumentBondPresentationV1>,
        new_atom_element: String,
        snap: PyRef<'_, PyDirectBondSnapPolicyV1>,
    ) -> PyResult<PyDirectBondGesture> {
        ferrum_document_render::begin_direct_bond_gesture(
            &self.session,
            DocumentFenceV1::new(expected_revision, parse_digest(&expected_digest_hex)?),
            start.probe.clone(),
            (*presentation).into(),
            new_atom_element,
            snap.policy,
        )
        .map(PyDirectBondGesture::from_renderer_gesture)
        .map_err(|error| admission_error(py, error))
    }
}

#[pymethods]
impl PyDirectBondGesture {
    fn resolve_end(
        &mut self,
        py: Python<'_>,
        session: PyRefMut<'_, PyDocumentSession>,
        end: PyRef<'_, PyDirectBondPointerProbe>,
    ) -> PyResult<super::prepared_transition_binding::PySessionOperationTransitionRequestV1> {
        let gesture = self.take_for_resolution()?;
        let request = ferrum_document_render::resolve_direct_bond_end(
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
    module.add(
        "DirectBondGestureError",
        module.py().get_type::<DirectBondGestureError>(),
    )?;
    module.add(
        "DirectBondAdmissionRefusal",
        module.py().get_type::<DirectBondAdmissionRefusal>(),
    )?;
    module.add(
        "DirectBondPointerProbeError",
        module.py().get_type::<DirectBondPointerProbeError>(),
    )?;
    module.add_class::<PyDirectBondPointerProbeCategory>()?;
    module.add_class::<PyDirectBondPointerProbeRecovery>()?;
    module.add_class::<PyDirectBondAdmissionCategory>()?;
    module.add_class::<PyDirectBondAdmissionRecovery>()?;
    module.add_class::<PyDirectBondPointerHitState>()?;
    // V1 snap configuration is shared by the current gesture lifecycle.
    module.add_class::<PyDirectBondSnapPolicyV1>()?;
    module.add_class::<PyDirectBondViewportToScene>()?;
    module.add_class::<PyDirectBondPointerProbe>()?;
    module.add_class::<PyDirectBondGesture>()?;
    Ok(())
}
