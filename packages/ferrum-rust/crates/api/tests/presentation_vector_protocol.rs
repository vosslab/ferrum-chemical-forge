use ferrum_api::{OperationProtocolEnvelopeV1, OperationProtocolOutcomeV1, execute_operation_v1};
use ferrum_document::DocumentSession;

const CDML: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom></molecule></cdml>";

fn digest(document: &str) -> String {
    DocumentSession::load(document)
        .expect("fixture loads")
        .snapshot()
        .expect("fixture snapshots")
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn request(document: &str, expected_revision: u64, end_x: f64, end_y: f64) -> String {
    serde_json::json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "vector-test",
        "operation": {
            "kind": "presentation.vector.create.v1",
            "document": document,
            "expected_revision": expected_revision,
            "expected_digest_hex": digest(document),
            "vector_kind": "rectangle",
            "start_x": 10.0,
            "start_y": 20.0,
            "end_x": end_x,
            "end_y": end_y,
            "appearance_policy": "effective_drawing_standard"
        }
    })
    .to_string()
}

#[test]
fn vector_protocol_preflights_then_returns_a_chainable_stateless_result() {
    let first = execute_operation_v1(&request(CDML, 0, 40.0, 60.0)).expect("request decodes");
    let OperationProtocolEnvelopeV1::Success(response) = first else {
        panic!("vector creation must succeed")
    };
    let OperationProtocolOutcomeV1::PresentationVectorCreate {
        document,
        identifier,
        input_revision,
        committed_revision,
        next_input_expected_revision,
        renderer_observation,
        ..
    } = response.outcome
    else {
        panic!("vector result expected")
    };
    assert_eq!(input_revision, 0);
    assert_eq!(committed_revision, 1);
    assert_eq!(next_input_expected_revision, 0);
    assert!(document.contains(&format!("id=\"{identifier}\"")));
    assert!(renderer_observation.is_object());

    let second = execute_operation_v1(&request(&document, 0, 70.0, 90.0)).expect("request decodes");
    assert!(matches!(second, OperationProtocolEnvelopeV1::Success(_)));
}

#[test]
fn vector_protocol_refuses_invalid_geometry_with_closed_recovery() {
    let response = execute_operation_v1(&request(CDML, 0, 10.0, 20.0)).expect("request decodes");
    let OperationProtocolEnvelopeV1::Error(response) = response else {
        panic!("degenerate rectangle must refuse")
    };
    let refusal = response
        .error
        .presentation_vector_refusal
        .expect("closed vector refusal facts");
    assert_eq!(
        serde_json::to_value(refusal.category).unwrap(),
        "degenerate_geometry"
    );
    assert_eq!(
        serde_json::to_value(refusal.recovery).unwrap(),
        "change_geometry"
    );
}

#[test]
fn vector_protocol_rejects_nonzero_stateless_revision_with_closed_recovery() {
    let response = execute_operation_v1(&request(CDML, 1, 40.0, 60.0)).expect("request decodes");
    let OperationProtocolEnvelopeV1::Error(response) = response else {
        panic!("stateless revision must refuse")
    };
    let refusal = response
        .error
        .presentation_vector_refusal
        .expect("closed vector refusal facts");
    assert_eq!(
        serde_json::to_value(refusal.category).unwrap(),
        "stale_snapshot"
    );
    assert_eq!(
        serde_json::to_value(refusal.recovery).unwrap(),
        "refresh_and_restart"
    );
}

#[test]
fn vector_protocol_refuses_a_renderer_excluded_existing_root_before_commit() {
    let excluded = "<cdml xmlns=\"urn:ferrum:cdml\"><text id=\"bad\"><point x=\"1\" y=\"2\"/><ftext><b>x</b></ftext></text></cdml>";
    let response =
        execute_operation_v1(&request(excluded, 0, 40.0, 60.0)).expect("request decodes");
    let OperationProtocolEnvelopeV1::Error(response) = response else {
        panic!("renderer-excluded root must reject vector commit")
    };
    let refusal = response
        .error
        .presentation_vector_refusal
        .expect("closed vector refusal facts");
    assert_eq!(
        serde_json::to_value(refusal.category).unwrap(),
        "render_preparation"
    );
    assert_eq!(
        serde_json::to_value(refusal.recovery).unwrap(),
        "document_unchanged"
    );
}
