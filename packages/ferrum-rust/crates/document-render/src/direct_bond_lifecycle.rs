//! Interaction facade for document-owned direct-bond admission.

use ferrum_document::{
    AuthoringCapabilityV1, DirectBondAdmissionRefusalV1, DirectBondEndpointIntent,
    DirectBondGestureErrorV1, DirectBondSnapPolicyV1, DocumentBondPresentationV1, DocumentFenceV1,
    DocumentSession, SessionOperation, SessionOperationTransitionRequestV1, SessionOperationV1,
    TransitionAuthorizationV1,
};

#[derive(Debug)]
pub(crate) struct DirectBondLifecycleGesture {
    pub(crate) capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    start: DirectBondEndpointIntent,
    presentation: DocumentBondPresentationV1,
    new_atom_element: String,
    snap: DirectBondSnapPolicyV1,
}

pub(crate) fn begin_direct_bond_lifecycle(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: DirectBondEndpointIntent,
    presentation: DocumentBondPresentationV1,
    new_atom_element: String,
    snap: DirectBondSnapPolicyV1,
) -> Result<DirectBondLifecycleGesture, DirectBondGestureErrorV1> {
    Ok(DirectBondLifecycleGesture {
        capability: session.issue_authoring_capability_v1(),
        fence,
        start,
        presentation,
        new_atom_element,
        snap,
    })
}

pub(crate) fn resolve_direct_bond_lifecycle_end(
    gesture: DirectBondLifecycleGesture,
    end: DirectBondEndpointIntent,
) -> Result<SessionOperationTransitionRequestV1, DirectBondAdmissionRefusalV1> {
    let DirectBondLifecycleGesture {
        capability,
        fence,
        start,
        presentation,
        new_atom_element,
        snap,
    } = gesture;
    let operation = SessionOperation::V1(SessionOperationV1::CreateDirectBondV1(
        ferrum_document::CreateDirectBondV1::new(
            fence,
            start,
            end,
            presentation,
            new_atom_element,
            snap,
        )?,
    ));
    Ok(SessionOperationTransitionRequestV1::new(
        fence.revision(),
        operation,
        TransitionAuthorizationV1::authoring_capability(capability),
    ))
}
