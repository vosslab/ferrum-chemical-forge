use std::io::Write;
use std::process::{Command, Output, Stdio};

use ferrum_api::{
    OperationProtocolEnvelopeV1, OperationProtocolOutcomeV1, execute_operation_v1,
    generated_operation_protocol_schema_v1,
};
use ferrum_document::DocumentSession;
use serde_json::{Value, json};

const EMPTY: &str = "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"/>";
const DIFFERENT: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";

fn expected_digest(document: &str) -> String {
    DocumentSession::load(document)
        .expect("fixture admits")
        .snapshot()
        .expect("fixture snapshots")
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn execute(operation: Value) -> OperationProtocolEnvelopeV1 {
    let request = json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "document-inspect-fence-test",
        "operation": operation,
    });
    execute_operation_v1(&request.to_string()).expect("request decodes")
}

fn inspect(document: &str) -> Value {
    let response = execute(json!({"kind": "document.inspect", "document": document}));
    let OperationProtocolEnvelopeV1::Success(response) = response else {
        panic!("inspection succeeds")
    };
    let OperationProtocolOutcomeV1::Inspect {
        report: _,
        document_fence,
    } = response.outcome
    else {
        panic!("inspection outcome expected")
    };
    serde_json::to_value(document_fence).expect("fence serializes")
}

fn catalog_insert(document: &str, fence: &Value) -> OperationProtocolEnvelopeV1 {
    execute(json!({
        "kind": "catalog.insert.v1",
        "document": document,
        "expected_revision": fence["expected_revision"],
        "expected_digest_hex": fence["expected_digest_hex"],
        "catalog_id": "system/rings/benzene",
        "anchor_x": 100.0,
        "anchor_y": 50.0,
    }))
}

fn run_inspect(document: &str, json_output: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ferrum"));
    command
        .args(["inspect", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if json_output {
        command.arg("--json");
    }
    let mut child = command.spawn().expect("inspect CLI starts");
    let mut stdin = child.stdin.take().expect("inspect CLI stdin is available");
    stdin
        .write_all(document.as_bytes())
        .expect("inspect CLI input writes");
    drop(stdin);
    child.wait_with_output().expect("inspect CLI completes")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProtocolProcessOutcome {
    Success,
    Refusal,
}

impl ProtocolProcessOutcome {
    const fn exit_code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::Refusal => 1,
        }
    }
}

fn run_protocol_request(
    arguments: &[&str],
    operation: Value,
    expected_outcome: ProtocolProcessOutcome,
) -> Value {
    let request = json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "document-inspect-fence-cli-test",
        "operation": operation,
    });
    let mut child = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("protocol CLI starts");
    child
        .stdin
        .take()
        .expect("protocol CLI stdin is available")
        .write_all(request.to_string().as_bytes())
        .expect("protocol CLI request writes");
    let output = child.wait_with_output().expect("protocol CLI completes");
    assert_eq!(
        output.status.code(),
        Some(expected_outcome.exit_code()),
        "protocol CLI outcome mismatch; stderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        output.stderr.is_empty(),
        "protocol JSON CLI wrote stderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    let envelope: Value =
        serde_json::from_slice(&output.stdout).expect("protocol CLI emits one JSON envelope");
    match expected_outcome {
        ProtocolProcessOutcome::Success => {
            assert!(envelope.get("outcome").is_some());
            assert!(envelope.get("error").is_none());
        }
        ProtocolProcessOutcome::Refusal => {
            assert!(envelope.get("error").is_some());
            assert!(envelope.get("outcome").is_none());
        }
    }
    envelope
}

fn find_named_schema<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    match value {
        Value::Object(object) => object
            .get(name)
            .filter(|candidate| candidate.get("properties").is_some())
            .or_else(|| {
                object
                    .values()
                    .find_map(|candidate| find_named_schema(candidate, name))
            }),
        Value::Array(values) => values
            .iter()
            .find_map(|candidate| find_named_schema(candidate, name)),
        _ => None,
    }
}

fn find_inspect_outcome_schema(value: &Value) -> Option<&Value> {
    match value {
        Value::Object(object) => {
            if object
                .get("properties")
                .and_then(|properties| properties.get("kind"))
                .and_then(|kind| kind.get("const"))
                == Some(&Value::String("document.inspect".to_owned()))
                && object
                    .get("properties")
                    .is_some_and(|properties| properties.get("report").is_some())
            {
                return Some(value);
            }
            object.values().find_map(find_inspect_outcome_schema)
        }
        Value::Array(values) => values.iter().find_map(find_inspect_outcome_schema),
        _ => None,
    }
}

#[test]
fn inspect_fence_chains_to_catalog_insert_and_refuses_a_different_document() {
    let fence = inspect(EMPTY);
    assert_eq!(fence["expected_revision"], 0);
    let digest = fence["expected_digest_hex"]
        .as_str()
        .expect("digest is text");
    assert_eq!(digest, expected_digest(EMPTY));
    assert_eq!(digest.len(), 64);
    assert!(
        digest
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    );

    let inserted = catalog_insert(EMPTY, &fence);
    let OperationProtocolEnvelopeV1::Success(response) = inserted else {
        panic!("inspect fence permits catalog insertion")
    };
    let OperationProtocolOutcomeV1::CatalogInsert {
        document,
        identifier,
        committed_revision,
        document_fence,
    } = response.outcome
    else {
        panic!("catalog insertion outcome expected")
    };
    assert!(document.contains("<molecule"));
    assert!(document.contains(&format!("id=\"{identifier}\"")));
    assert_eq!(committed_revision, 1);
    assert_eq!(document_fence.expected_revision, 0);
    assert_eq!(
        document_fence.expected_digest_hex,
        expected_digest(&document)
    );
    let next_fence = serde_json::to_value(document_fence).expect("returned fence serializes");
    assert!(matches!(
        catalog_insert(&document, &next_fence),
        OperationProtocolEnvelopeV1::Success(_)
    ));

    let stale = catalog_insert(DIFFERENT, &fence);
    let OperationProtocolEnvelopeV1::Error(response) = stale else {
        panic!("different document must refuse the prior inspection fence")
    };
    assert_eq!(
        serde_json::to_value(
            response
                .error
                .catalog_placement_refusal
                .expect("refusal facts")
        )
        .expect("refusal serializes")["category"],
        "stale_snapshot"
    );
    assert!(matches!(
        catalog_insert(EMPTY, &fence),
        OperationProtocolEnvelopeV1::Success(_)
    ));
}

#[test]
fn named_catalog_insert_cli_chains_the_returned_document_fence_without_reinspection() {
    let inspected = run_protocol_request(
        &["protocol", "run", "-"],
        json!({"kind": "document.inspect", "document": EMPTY}),
        ProtocolProcessOutcome::Success,
    );
    let initial_fence = inspected["outcome"]["document_fence"].clone();
    let first = run_protocol_request(
        &["document", "command", "catalog.insert.v1", "-"],
        json!({
            "kind": "catalog.insert.v1",
            "document": EMPTY,
            "expected_revision": initial_fence["expected_revision"],
            "expected_digest_hex": initial_fence["expected_digest_hex"],
            "catalog_id": "system/rings/benzene",
            "anchor_x": 100.0,
            "anchor_y": 50.0,
        }),
        ProtocolProcessOutcome::Success,
    );
    let document = first["outcome"]["document"]
        .as_str()
        .expect("catalog insertion returns canonical CDML");
    let identifier = first["outcome"]["identifier"]
        .as_str()
        .expect("catalog insertion returns a durable root ID");
    assert!(document.contains(&format!("id=\"{identifier}\"")));
    assert_eq!(first["outcome"]["committed_revision"], 1);
    let next_fence = first["outcome"]["document_fence"].clone();
    assert_eq!(next_fence["expected_revision"], 0);
    assert_eq!(next_fence["expected_digest_hex"], expected_digest(document));

    let chained = run_protocol_request(
        &["document", "command", "catalog.insert.v1", "-"],
        json!({
            "kind": "catalog.insert.v1",
            "document": document,
            "expected_revision": next_fence["expected_revision"],
            "expected_digest_hex": next_fence["expected_digest_hex"],
            "catalog_id": "system/rings/cyclohexane",
            "anchor_x": 200.0,
            "anchor_y": 50.0,
        }),
        ProtocolProcessOutcome::Success,
    );
    assert_eq!(chained["outcome"]["kind"], "catalog.insert.v1");
    assert_eq!(chained["outcome"]["committed_revision"], 1);

    let stale = run_protocol_request(
        &["document", "command", "catalog.insert.v1", "-"],
        json!({
            "kind": "catalog.insert.v1",
            "document": EMPTY,
            "expected_revision": 1,
            "expected_digest_hex": expected_digest(EMPTY),
            "catalog_id": "system/rings/benzene",
            "anchor_x": 100.0,
            "anchor_y": 50.0,
        }),
        ProtocolProcessOutcome::Refusal,
    );
    assert_eq!(
        stale["error"]["catalog_placement_refusal"]["category"],
        "stale_snapshot"
    );
    assert!(stale.get("outcome").is_none());
    assert_eq!(expected_digest(EMPTY), initial_fence["expected_digest_hex"]);
}

#[test]
fn inspect_cli_json_exposes_the_fence_while_human_output_stays_report_only() {
    let json_output = run_inspect(EMPTY, true);
    assert!(
        json_output.status.success(),
        "inspect JSON stderr: {}",
        String::from_utf8_lossy(&json_output.stderr)
    );
    let envelope: Value = serde_json::from_slice(&json_output.stdout).expect("JSON envelope");
    assert_eq!(envelope["outcome"]["kind"], "document.inspect");
    assert_eq!(
        envelope["outcome"]["document_fence"]["expected_revision"],
        0
    );
    assert_eq!(
        envelope["outcome"]["document_fence"]["expected_digest_hex"],
        expected_digest(EMPTY)
    );

    let human_output = run_inspect(EMPTY, false);
    assert!(
        human_output.status.success(),
        "inspect stderr: {}",
        String::from_utf8_lossy(&human_output.stderr)
    );
    let report: Value = serde_json::from_slice(&human_output.stdout).expect("human report JSON");
    assert_eq!(report, envelope["outcome"]["report"]);
}

#[test]
fn generated_schema_requires_the_complete_inspect_fence_contract() {
    let schema = generated_operation_protocol_schema_v1();
    let fence = find_named_schema(&schema, "DocumentRequestFenceV1").expect("fence schema");
    let fence_required = fence["required"].as_array().expect("fence required fields");
    assert!(fence_required.contains(&json!("expected_revision")));
    assert!(fence_required.contains(&json!("expected_digest_hex")));

    let inspect = find_inspect_outcome_schema(&schema).expect("inspect success schema");
    let required = inspect["required"]
        .as_array()
        .expect("inspect required fields");
    assert!(required.contains(&json!("report")));
    assert!(required.contains(&json!("document_fence")));
}
