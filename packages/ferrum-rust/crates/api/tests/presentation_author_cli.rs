use std::io::Write;
use std::process::{Command, Stdio};

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

fn point(x: f64, y: f64) -> serde_json::Value {
    serde_json::json!({ "x": x, "y": y })
}

fn run(document: &str, authoring: serde_json::Value) -> serde_json::Value {
    run_with_fence(document, 0, &digest(document), authoring)
}

fn run_with_fence(
    document: &str,
    expected_revision: u64,
    expected_digest_hex: &str,
    authoring: serde_json::Value,
) -> serde_json::Value {
    let request = serde_json::json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "presentation-author-cli",
        "operation": {
            "kind": "presentation.author.v1",
            "document": document,
            "expected_revision": expected_revision,
            "expected_digest_hex": expected_digest_hex,
            "authoring": authoring,
        }
    });
    let mut child = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["protocol", "run", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("ferrum protocol command starts");
    child
        .stdin
        .take()
        .expect("stdin is available")
        .write_all(request.to_string().as_bytes())
        .expect("request is written");
    let output = child
        .wait_with_output()
        .expect("ferrum protocol command completes");
    assert!(
        output.status.success(),
        "protocol stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("protocol emits one JSON envelope")
}

#[test]
fn protocol_cli_executes_every_closed_presentation_authoring_variant() {
    let variants = [
        serde_json::json!({
            "kind": "vector", "vector_kind": "line", "start": point(0.0, 0.0),
            "end": point(40.0, 0.0), "appearance_policy": "effective_drawing_standard"
        }),
        serde_json::json!({
            "kind": "curved_terminal_arrow", "terminal_kind": "electron",
            "start": point(0.0, 0.0), "control": point(20.0, 20.0), "end": point(40.0, 0.0)
        }),
        serde_json::json!({
            "kind": "curved_terminal_arrow", "terminal_kind": "retro",
            "start": point(0.0, 0.0), "control": point(20.0, 20.0), "end": point(40.0, 0.0)
        }),
        serde_json::json!({
            "kind": "curved_terminal_arrow", "terminal_kind": "normal",
            "start": point(0.0, 0.0), "control": point(20.0, 20.0), "end": point(40.0, 0.0)
        }),
        serde_json::json!({
            "kind": "curved_equilibrium_arrow", "start": point(0.0, 0.0),
            "control": point(40.0, 12.0), "end": point(80.0, 0.0)
        }),
        serde_json::json!({
            "kind": "path", "path_kind": "polygon",
            "points": [point(0.0, 0.0), point(40.0, 0.0), point(20.0, 30.0)]
        }),
        serde_json::json!({
            "kind": "direct_bond",
            "start": { "kind": "new_atom", "point": point(0.0, 0.0) },
            "end": { "kind": "new_atom", "point": point(40.0, 0.0) },
            "presentation": { "kind": "solid_wedge" },
            "new_atom_element": "C",
            "snap": { "hex_grid": false, "angle_increment_degrees": null, "fixed_length_pt": null }
        }),
    ];
    for authoring in variants {
        let response = run(EMPTY, authoring);
        assert_eq!(response["schema"], "ferrum-operation-response-v1");
        assert_eq!(response["outcome"]["kind"], "presentation.author.v1");
        assert_eq!(response["outcome"]["committed_revision"], 1);
        assert_eq!(
            response["outcome"]["document_fence"]["expected_revision"],
            0
        );
        assert!(response["outcome"].get("renderer_observation").is_none());
    }
}

#[test]
fn protocol_cli_accepts_an_authoring_result_as_the_next_request_document() {
    let first = run(
        EMPTY,
        serde_json::json!({
            "kind": "vector", "vector_kind": "line", "start": point(0.0, 0.0),
            "end": point(40.0, 0.0), "appearance_policy": "effective_drawing_standard"
        }),
    );
    let document = first["outcome"]["document"]
        .as_str()
        .expect("first response contains its accepted document");
    let document_fence = first["outcome"]["document_fence"].clone();
    let expected_revision = document_fence["expected_revision"]
        .as_u64()
        .expect("first response contains a typed request revision");
    let expected_digest_hex = document_fence["expected_digest_hex"]
        .as_str()
        .expect("first response contains a typed request digest");
    let second = run_with_fence(
        document,
        expected_revision,
        expected_digest_hex,
        serde_json::json!({
            "kind": "curved_terminal_arrow", "terminal_kind": "normal",
            "start": point(0.0, 20.0), "control": point(20.0, 40.0), "end": point(40.0, 20.0)
        }),
    );
    assert_eq!(second["schema"], "ferrum-operation-response-v1");
    assert_eq!(second["outcome"]["kind"], "presentation.author.v1");
    let identifier = second["outcome"]["identifier"]
        .as_str()
        .expect("second response contains the committed root identifier");
    let chained_document = second["outcome"]["document"]
        .as_str()
        .expect("second response contains the accepted document");
    assert!(chained_document.contains(identifier));
}

#[test]
fn protocol_cli_returns_a_typed_direct_bond_refusal_without_a_document_outcome() {
    let response = run(
        EMPTY,
        serde_json::json!({
            "kind": "direct_bond",
            "start": { "kind": "new_atom", "point": point(0.0, 0.0) },
            "end": { "kind": "new_atom", "point": point(0.0, 0.0) },
            "presentation": { "kind": "normal", "order": "single" },
            "new_atom_element": "C",
            "snap": { "hex_grid": false, "angle_increment_degrees": null, "fixed_length_pt": null }
        }),
    );
    assert_eq!(response["schema"], "ferrum-operation-error-v1");
    assert!(response.get("outcome").is_none());
    assert_eq!(
        response["error"]["presentation_author_refusal"]["authoring_kind"],
        "direct_bond"
    );
    assert_eq!(
        response["error"]["presentation_author_refusal"]["category"],
        "invalid_endpoint"
    );
}
