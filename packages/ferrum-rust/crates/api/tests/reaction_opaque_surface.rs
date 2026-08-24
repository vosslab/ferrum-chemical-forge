use ferrum_api::{
    ReactionCreateRequestV1, begin_api_reaction_gesture_v1, resolve_api_reaction_gesture_v1,
};
use ferrum_document::{DocumentFenceV1, DocumentSession};
use ferrum_document_render::{begin_reaction_gesture_v1, resolve_reaction_gesture_v1};

const OPAQUE_MARKER: &str = "FERRUM_REACTION_CANDIDATE_SECRET_9c324a";
const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"left\" ferrum_marker=\"FERRUM_REACTION_CANDIDATE_SECRET_9c324a\"><atom id=\"left-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"product\"><atom id=\"product-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><arrow id=\"arrow\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow></cdml>";

fn request(revision: u64) -> ReactionCreateRequestV1 {
    ReactionCreateRequestV1::new(
        revision,
        vec!["left".into()],
        vec!["product".into()],
        "arrow".into(),
        vec![],
        vec![],
    )
    .expect("fixture request is valid")
}

fn fence(session: &DocumentSession) -> DocumentFenceV1 {
    let snapshot = session.snapshot().expect("fixture snapshots");
    DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
}

fn assert_redacted(debug: &str) {
    assert!(
        !debug.contains(OPAQUE_MARKER),
        "opaque candidate CDML marker escaped through Debug: {debug}"
    );
    assert!(
        !debug.contains("<cdml") && !debug.contains("<reaction"),
        "Debug must not contain serialized candidate XML: {debug}"
    );
}

#[test]
fn reaction_capabilities_remain_redacted_across_public_document_render_and_api_surfaces() {
    let mut bridge_session = DocumentSession::load(SOURCE).expect("fixture loads");
    let bridge_fence = fence(&bridge_session);
    let bridge_gesture = begin_reaction_gesture_v1(&bridge_session, bridge_fence, request(0))
        .expect("bridge gesture begins");
    let bridge_request = resolve_reaction_gesture_v1(&bridge_session, bridge_gesture)
        .expect("bridge gesture resolves");
    let bridge_prepared = bridge_session
        .prepare_session_operation_transition_v1(bridge_request)
        .expect("bridge capability prepares");
    let bridge_debug = format!("{bridge_prepared:?}");
    assert_redacted(&bridge_debug);

    let mut api_session = DocumentSession::load(SOURCE).expect("fixture loads");
    let api_fence = fence(&api_session);
    let api_gesture = begin_api_reaction_gesture_v1(&api_session, api_fence, request(0))
        .expect("API gesture begins");
    let api_request =
        resolve_api_reaction_gesture_v1(&api_session, api_gesture).expect("API gesture resolves");
    let api_prepared = api_session
        .prepare_session_operation_transition_v1(api_request)
        .expect("API capability prepares");
    assert_redacted(&format!("{api_prepared:?}"));
}
