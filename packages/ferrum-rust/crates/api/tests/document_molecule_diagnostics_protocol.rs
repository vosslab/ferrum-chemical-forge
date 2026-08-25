use std::io::Write;
use std::process::{Command, Stdio};

use ferrum_api::execute_operation_v1;
use ferrum_document::{
    DocumentObjectIdV1, DocumentSession, load_document_utf8_bytes_with_budget,
    local_cdml_ingress_format_v1,
};
use serde_json::{Value, json};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:object=\"urn:ferrum:document-object:v1\">",
    "<molecule id=\"first\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000010\">",
    "<atom id=\"carbon\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000011\" name=\"C\" charge=\"0\"><point x=\"0\" y=\"0\"/></atom>",
    "</molecule><molecule id=\"second\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000012\">",
    "<atom id=\"nitrogen\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000013\" name=\"N\" charge=\"0\"><point x=\"1\" y=\"0\"/></atom>",
    "</molecule></cdml>"
);

const ATTACHED_METHYL_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:object=\"urn:ferrum:document-object:v1\">",
    "<molecule id=\"attached\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000020\">",
    "<atom id=\"anchor\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000021\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
    "<compact-group id=\"methyl\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000022\" version=\"1\" catalog-key=\"methyl\" ",
    "attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/>",
    "</compact-group><bond id=\"attachment\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000023\" start=\"anchor\" end=\"methyl\" type=\"n1\"/>",
    "</molecule></cdml>"
);

fn protocol_session(document: &str) -> DocumentSession {
    load_document_utf8_bytes_with_budget(document.as_bytes(), local_cdml_ingress_format_v1())
        .expect("source admits")
}

fn digest(document: &str) -> String {
    protocol_session(document)
        .snapshot()
        .expect("source snapshots")
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn molecule_ids(document: &str) -> Vec<String> {
    protocol_session(document)
        .observe(0)
        .expect("source observes")
        .projection()
        .molecules()
        .iter()
        .map(|molecule| {
            molecule
                .id()
                .expect("direct root has a durable identifier")
                .as_str()
                .to_owned()
        })
        .collect()
}

fn first_atom_id(document: &str) -> String {
    protocol_session(document)
        .observe(0)
        .expect("source observes")
        .projection()
        .molecules()[0]
        .atoms()[0]
        .id()
        .expect("direct-root atom has a durable identifier")
        .as_str()
        .to_owned()
}

fn request(document: &str, source_revision: u64, ids: Vec<String>) -> Value {
    json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "molecule-diagnostics-protocol-test",
        "operation": {
            "kind": "document.molecule.diagnostics.v1",
            "snapshot": {
                "cdml": document,
                "revision": source_revision,
                "digest_hex": digest(document),
            },
            "molecule_ids": ids,
        },
    })
}

fn execute(request: Value) -> Value {
    serde_json::to_value(execute_operation_v1(&request.to_string()).expect("request decodes"))
        .expect("envelope serializes")
}

fn compact_materialization_request(document: &str) -> Value {
    let observation = protocol_session(document)
        .observe(0)
        .expect("compact source observes");
    let molecule = &observation.projection().molecules()[0];
    json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "mismatched-materialization",
        "operation": {
            "kind": "document.compact-group.materialize.v1",
            "document": {
                "cdml": document,
                "expected_revision": 0,
                "expected_digest_hex": digest(document),
            },
            "molecule_id": molecule.document_object_id().as_str(),
            "compact_group_id": molecule.compact_groups()[0].document_object_id().as_str(),
        },
    })
}

#[test]
fn diagnostics_preserve_source_fence_and_root_order() {
    let ids = molecule_ids(SOURCE);
    let response = execute(request(SOURCE, 23, vec![ids[1].clone(), ids[0].clone()]));
    let diagnostics = &response["outcome"]["diagnostics"];
    assert_eq!(
        response["outcome"]["kind"],
        "document.molecule.diagnostics.v1"
    );
    assert_eq!(diagnostics["source_revision"], 23);
    assert_eq!(diagnostics["source_digest_hex"], digest(SOURCE));
    assert_eq!(
        diagnostics["records"]
            .as_array()
            .expect("records array")
            .iter()
            .map(|record| record["molecule_id"].as_str().expect("durable root id"))
            .collect::<Vec<_>>(),
        vec![ids[0].as_str(), ids[1].as_str()]
    );
}

#[test]
fn diagnostics_refuse_invalid_fences_and_non_root_targets_without_mutating_source() {
    let ids = molecule_ids(SOURCE);
    let original_digest = digest(SOURCE);
    let mut invalid_fence = request(SOURCE, 0, vec![ids[0].clone()]);
    invalid_fence["operation"]["snapshot"]["digest_hex"] = json!("0".repeat(64));
    let non_root = request(SOURCE, 0, vec![first_atom_id(SOURCE)]);
    let missing = request(
        SOURCE,
        0,
        vec![
            DocumentObjectIdV1::from_entropy_bytes([0xf0; 16])
                .as_str()
                .to_owned(),
        ],
    );
    for refusal in [execute(invalid_fence), execute(non_root), execute(missing)] {
        assert_eq!(refusal["schema"], "ferrum-operation-error-v1");
        assert_eq!(
            refusal["error"]["operation"],
            "document.molecule.diagnostics.v1"
        );
        assert!(refusal.get("outcome").is_none());
    }
    assert_eq!(digest(SOURCE), original_digest);
}

#[test]
fn diagnostics_report_attached_compact_group_membership_and_rust_recovery() {
    let molecule_id = molecule_ids(ATTACHED_METHYL_SOURCE)[0].clone();
    let response = execute(request(
        ATTACHED_METHYL_SOURCE,
        0,
        vec![molecule_id.clone()],
    ));
    let record = response["outcome"]["diagnostics"]["records"]
        .as_array()
        .expect("diagnostics records are public records")
        .iter()
        .find(|record| record["molecule_id"] == molecule_id)
        .expect("selected molecule has a diagnostic record");
    let findings = record["findings"]
        .as_array()
        .expect("diagnostics findings are public records");
    let group_finding = findings
        .iter()
        .find(|finding| finding["code"] == "unexpanded_group_present")
        .expect("attached compact group is reported semantically");
    assert_eq!(group_finding["recovery"], "choose_supported_representation");
}

#[test]
fn diagnostics_selector_bound_returns_a_typed_refusal() {
    let root = molecule_ids(SOURCE)[0].clone();
    let response = execute(request(SOURCE, 0, vec![root; 129]));
    assert_eq!(response["schema"], "ferrum-operation-error-v1");
    assert_eq!(response["error"]["category"], "resource_limit");
}

#[test]
fn diagnostics_refuse_aggregate_selector_bytes_before_parsing() {
    let selectors = (0..64)
        .map(|index| {
            DocumentObjectIdV1::from_entropy_bytes([index as u8; 16])
                .as_str()
                .to_owned()
        })
        .collect::<Vec<_>>();
    assert!(selectors.iter().map(String::len).sum::<usize>() > 2 * 1024);

    let response = execute(request(SOURCE, 0, selectors));
    assert_eq!(response["schema"], "ferrum-operation-error-v1");
    assert_eq!(response["error"]["category"], "resource_limit");
}

#[test]
fn named_diagnostics_cli_forwards_the_generic_protocol_envelope() {
    let payload = request(SOURCE, 0, vec![molecule_ids(SOURCE)[0].clone()]).to_string();
    let mut child = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "document",
            "command",
            "document.molecule.diagnostics.v1",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("named diagnostics CLI starts");
    child
        .stdin
        .take()
        .expect("CLI stdin")
        .write_all(payload.as_bytes())
        .expect("CLI input writes");
    let output = child.wait_with_output().expect("named CLI completes");
    assert!(
        output.status.success(),
        "CLI stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: Value = serde_json::from_slice(&output.stdout).expect("CLI emits JSON envelope");
    assert_eq!(response["request_id"], "molecule-diagnostics-protocol-test");
    assert_eq!(
        response["outcome"]["kind"],
        "document.molecule.diagnostics.v1"
    );
}

#[test]
fn named_diagnostics_cli_refuses_a_mutating_operation_before_execution() {
    let payload = compact_materialization_request(ATTACHED_METHYL_SOURCE).to_string();
    let generic_response = execute(serde_json::from_str(&payload).expect("request JSON"));
    assert_eq!(
        generic_response["outcome"]["kind"], "document.compact-group.materialize.v1",
        "generic materialization request must be executable: {generic_response}"
    );

    let mut child = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "document",
            "command",
            "document.molecule.diagnostics.v1",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("named diagnostics CLI starts");
    child
        .stdin
        .take()
        .expect("CLI stdin")
        .write_all(payload.as_bytes())
        .expect("CLI input writes");
    let output = child.wait_with_output().expect("named CLI completes");
    assert!(
        output.status.success(),
        "CLI stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: Value = serde_json::from_slice(&output.stdout).expect("CLI emits JSON envelope");
    assert_eq!(response["schema"], "ferrum-operation-error-v1");
    assert_eq!(response["error"]["category"], "invalid_request");
    assert_eq!(
        response["error"]["operation"],
        "document.compact-group.materialize.v1"
    );
    assert!(response.get("outcome").is_none());
}
