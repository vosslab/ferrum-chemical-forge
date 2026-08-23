use ferrum_api::{OperationProtocolEnvelopeV1, OperationProtocolOutcomeV1, execute_operation_v1};
use ferrum_document::DocumentSession;

const EMPTY: &str = "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"/>";

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

fn request(document: &str, authoring: serde_json::Value) -> String {
    request_with_fence(document, 0, &digest(document), authoring)
}

fn request_with_fence(
    document: &str,
    expected_revision: u64,
    expected_digest_hex: &str,
    authoring: serde_json::Value,
) -> String {
    serde_json::json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "presentation-author-test",
        "operation": {
            "kind": "presentation.author.v1",
            "document": document,
            "expected_revision": expected_revision,
            "expected_digest_hex": expected_digest_hex,
            "authoring": authoring,
        }
    })
    .to_string()
}

fn point(x: f64, y: f64) -> serde_json::Value {
    serde_json::json!({ "x": x, "y": y })
}

#[test]
fn closed_authoring_variants_commit_one_chainable_document_transition() {
    let variants = [
        (
            "vector",
            serde_json::json!({
                "kind": "vector", "vector_kind": "rectangle", "start": point(0.0, 0.0),
                "end": point(40.0, 30.0), "appearance_policy": "effective_drawing_standard"
            }),
        ),
        (
            "curved_terminal_arrow",
            serde_json::json!({
                "kind": "curved_terminal_arrow", "terminal_kind": "electron",
                "start": point(0.0, 0.0), "control": point(20.0, 20.0), "end": point(40.0, 0.0)
            }),
        ),
        (
            "curved_terminal_arrow",
            serde_json::json!({
                "kind": "curved_terminal_arrow", "terminal_kind": "retro",
                "start": point(0.0, 0.0), "control": point(20.0, 20.0), "end": point(40.0, 0.0)
            }),
        ),
        (
            "curved_terminal_arrow",
            serde_json::json!({
                "kind": "curved_terminal_arrow", "terminal_kind": "normal",
                "start": point(0.0, 0.0), "control": point(20.0, 20.0), "end": point(40.0, 0.0)
            }),
        ),
        (
            "curved_equilibrium_arrow",
            serde_json::json!({
                "kind": "curved_equilibrium_arrow", "start": point(0.0, 0.0),
                "control": point(40.0, 12.0), "end": point(80.0, 0.0)
            }),
        ),
        (
            "path",
            serde_json::json!({
                "kind": "path", "path_kind": "polyline",
                "points": [point(0.0, 0.0), point(20.0, 20.0), point(40.0, 0.0)]
            }),
        ),
        (
            "direct_bond",
            serde_json::json!({
                "kind": "direct_bond",
                "start": { "kind": "new_atom", "point": point(0.0, 0.0) },
                "end": { "kind": "new_atom", "point": point(40.0, 0.0) },
                "presentation": { "kind": "normal", "order": "single" },
                "new_atom_element": "C",
                "snap": { "hex_grid": false, "angle_increment_degrees": null, "fixed_length_pt": null }
            }),
        ),
    ];
    for (expected_kind, authoring) in variants {
        let response = execute_operation_v1(&request(EMPTY, authoring)).expect("request decodes");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("{expected_kind} must succeed: {response:?}")
        };
        let envelope = serde_json::to_value(&response).expect("public response serializes");
        let OperationProtocolOutcomeV1::PresentationAuthor {
            authoring_kind,
            document,
            identifier,
            committed_revision,
            document_fence,
            direct_bond,
            ..
        } = response.outcome
        else {
            panic!("{expected_kind} must return a presentation author outcome")
        };
        assert_eq!(serde_json::to_value(authoring_kind).unwrap(), expected_kind);
        assert_eq!(committed_revision, 1);
        assert!(document.contains(&identifier));
        assert_eq!(document_fence.expected_revision, 0);
        assert_eq!(document_fence.expected_digest_hex, digest(&document));
        assert!(envelope["outcome"].get("renderer_observation").is_none());
        assert!(envelope["outcome"].get("input_revision").is_none());
        assert!(
            envelope["outcome"]
                .get("next_input_expected_revision")
                .is_none()
        );
        assert!(envelope["outcome"].get("digest_hex").is_none());
        assert_eq!(direct_bond.is_some(), expected_kind == "direct_bond");
    }
}

#[test]
fn authoring_results_are_request_owned_chain_inputs() {
    let first = serde_json::json!({
        "kind": "vector", "vector_kind": "line", "start": point(0.0, 0.0),
        "end": point(40.0, 0.0), "appearance_policy": "effective_drawing_standard"
    });
    let OperationProtocolEnvelopeV1::Success(response) =
        execute_operation_v1(&request(EMPTY, first)).expect("first request decodes")
    else {
        panic!("first authoring request must succeed")
    };
    let OperationProtocolOutcomeV1::PresentationAuthor {
        document,
        document_fence,
        ..
    } = response.outcome
    else {
        panic!("first request returns a document transition")
    };

    let second = serde_json::json!({
        "kind": "curved_terminal_arrow", "terminal_kind": "normal",
        "start": point(0.0, 20.0), "control": point(20.0, 40.0), "end": point(40.0, 20.0)
    });
    let OperationProtocolEnvelopeV1::Success(response) = execute_operation_v1(&request_with_fence(
        &document,
        document_fence.expected_revision,
        &document_fence.expected_digest_hex,
        second,
    ))
    .expect("chained request decodes") else {
        panic!("chained authoring request must succeed")
    };
    let OperationProtocolOutcomeV1::PresentationAuthor {
        document: chained_document,
        identifier,
        committed_revision,
        ..
    } = response.outcome
    else {
        panic!("chained request returns a document transition")
    };
    assert_eq!(committed_revision, 1);
    assert_ne!(chained_document, document);
    assert!(chained_document.contains(&identifier));
}

#[test]
fn authoring_refusals_are_typed_and_do_not_admit_cross_variant_fields() {
    let invalid_path = serde_json::json!({
        "kind": "path", "path_kind": "polygon", "points": [point(0.0, 0.0), point(20.0, 0.0)]
    });
    let response = execute_operation_v1(&request(EMPTY, invalid_path)).expect("request decodes");
    let OperationProtocolEnvelopeV1::Error(response) = response else {
        panic!("underspecified polygon must refuse")
    };
    let refusal = response
        .error
        .presentation_author_refusal
        .expect("typed refusal");
    assert_eq!(
        serde_json::to_value(refusal.authoring_kind).unwrap(),
        "path"
    );
    assert_eq!(
        serde_json::to_value(refusal.category).unwrap(),
        "path_cardinality"
    );
    assert_eq!(
        serde_json::to_value(refusal.recovery).unwrap(),
        "change_geometry"
    );

    let malformed = request(
        EMPTY,
        serde_json::json!({
            "kind": "vector", "vector_kind": "line", "start": point(0.0, 0.0),
            "end": point(40.0, 0.0), "appearance_policy": "effective_drawing_standard",
            "terminal_kind": "electron"
        }),
    );
    let response = execute_operation_v1(&malformed).expect("envelope parses");
    let OperationProtocolEnvelopeV1::Error(response) = response else {
        panic!("cross-variant field must reject")
    };
    assert_eq!(
        serde_json::to_value(response.error.category).unwrap(),
        "invalid_request"
    );
}
