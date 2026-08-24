use std::io::Write;
use std::process::{Command, Stdio};

use ferrum_api::{DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1, execute_operation_v1};
use ferrum_document::DocumentSession;
use serde_json::{Value, json};

const OXYGEN_SKELETON: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\"><molecule id=\"oxygen-root\">",
    "<atom id=\"oxygen-anchor\" name=\"O\"><point x=\"0\" y=\"0\"/></atom>",
    "</molecule></cdml>"
);

fn digest(document: &str) -> String {
    DocumentSession::load(document)
        .expect("ordinary skeleton loads")
        .snapshot()
        .expect("ordinary skeleton snapshots")
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn selection(document: &str) -> (String, String) {
    let session = DocumentSession::load(document).expect("ordinary skeleton loads");
    let observation = session.observe(0).expect("ordinary skeleton projects");
    let molecule = &observation.projection().molecules()[0];
    (
        molecule
            .id()
            .expect("root has a public durable selector")
            .as_str()
            .to_owned(),
        molecule.atoms()[0]
            .id()
            .expect("anchor has a public durable selector")
            .as_str()
            .to_owned(),
    )
}

fn materialization_request(document: &str) -> Value {
    let (molecule_id, anchor_atom_id) = selection(document);
    json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "hydrogen-materialization-protocol-test",
        "operation": {
            "kind": "document.molecule.hydrogen.materialize.v1",
            "document": {
                "cdml": document,
                "expected_revision": 0,
                "expected_digest_hex": digest(document),
            },
            "molecule_id": molecule_id,
            "anchor_atom_id": anchor_atom_id,
        },
    })
}

fn response_limited_oxygen_skeleton() -> String {
    let source_id = "oxygen".repeat(DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1);
    format!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\"><molecule id=\"oxygen-root\"><atom id=\"{source_id}\" name=\"O\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>"
    )
}

fn cli_operation(arguments: &[&str], request: Value) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("named CLI starts");
    child
        .stdin
        .take()
        .expect("CLI stdin")
        .write_all(request.to_string().as_bytes())
        .expect("CLI request writes");
    let output = child.wait_with_output().expect("named CLI completes");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("CLI emits one protocol envelope")
}

#[test]
fn local_materialization_command_chains_to_public_oxidation_observation() {
    let materialized = cli_operation(
        &["document-molecule-hydrogen-materialize", "--request", "-"],
        materialization_request(OXYGEN_SKELETON),
    );
    let receipt = &materialized["outcome"]["materialization"];
    assert_eq!(receipt["status"], "applied");
    assert_eq!(receipt["added_hydrogen_count"], 2);
    assert!(receipt.get("generated_ids").is_none());

    let oxidation_request = json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "hydrogen-materialization-oxidation-chain",
        "operation": {
            "kind": "document.atom.oxidation.observe.v1",
            "document": {
                "cdml": receipt["document"],
                "expected_revision": receipt["document_fence"]["expected_revision"],
                "expected_digest_hex": receipt["document_fence"]["expected_digest_hex"],
            },
            "molecule_id": receipt["molecule_id"],
            "atom_id": receipt["anchor_atom_id"],
        },
    });
    let oxidation = execute_operation_v1(&oxidation_request.to_string())
        .expect("public oxidation request decodes");
    let oxidation = serde_json::to_value(oxidation).expect("oxidation envelope serializes");
    assert_eq!(
        oxidation["outcome"]["observation"]["oxidation_number"], -2,
        "{oxidation}"
    );
}

#[test]
fn materialization_response_limit_keeps_the_public_operation_and_recovery() {
    let document = response_limited_oxygen_skeleton();
    let response = execute_operation_v1(&materialization_request(&document).to_string())
        .expect("materialization request decodes");
    let envelope = serde_json::to_value(response).expect("error envelope serializes");

    assert_eq!(
        envelope["error"]["operation"],
        "document.molecule.hydrogen.materialize.v1"
    );
    assert_eq!(
        envelope["error"]["resource_limit"],
        json!({
            "reason": "response_size_exceeded",
            "recovery": "reduce_requested_result",
        })
    );
}
