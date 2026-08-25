use ferrum_api::execute_operation_v1;
use ferrum_document::{DocumentObjectIdV1, DocumentSession};
use serde_json::{Value, json};

const COMPACT_CDML: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"source-molecule\">",
    "<atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
    "<compact-group id=\"source-group\" version=\"1\" catalog-key=\"methyl\" ",
    "attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group>",
    "<bond id=\"outside\" start=\"anchor\" end=\"source-group\" type=\"n1\"/>",
    "</molecule></cdml>"
);

fn digest(document: &str) -> String {
    DocumentSession::load(document)
        .expect("inline compact document loads")
        .snapshot()
        .expect("inline compact document snapshots")
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn compact_request(document: &str, revision: u64, digest_hex: &str, group_id: &str) -> Value {
    json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "compact-group-protocol-test",
        "operation": {
            "kind": "document.compact-group.materialize.v1",
            "document": {
                "cdml": document,
                "expected_revision": revision,
                "expected_digest_hex": digest_hex,
            },
            "molecule_id": "source-molecule",
            "compact_group_id": group_id,
        },
    })
}

fn execute(request: Value) -> Value {
    serde_json::to_value(
        execute_operation_v1(&request.to_string()).expect("compact request is protocol JSON"),
    )
    .expect("protocol envelope serializes")
}

fn materialized() -> Value {
    execute(compact_request(
        COMPACT_CDML,
        0,
        &digest(COMPACT_CDML),
        "source-group",
    ))
}

#[test]
fn accepted_compact_receipt_focus_resolves_and_its_fence_accepts_a_follow_up() {
    let response = materialized();
    let receipt = &response["outcome"]["materialization"];
    assert_eq!(
        response["outcome"]["kind"],
        "document.compact-group.materialize.v1"
    );
    assert_eq!(receipt["molecule_id"], "source-molecule");
    assert_eq!(receipt["compact_group_id"], "source-group");
    let document = receipt["document"]
        .as_str()
        .expect("accepted CDML document");
    let focus = receipt["replacement_focus_atom_id"]
        .as_str()
        .expect("replacement focus identifier");
    let session = DocumentSession::load(document).expect("accepted CDML reloads");
    let observation = session.observe(0).expect("accepted CDML observes");
    assert!(
        observation.projection().molecules()[0]
            .atoms()
            .iter()
            .any(|atom| {
                atom.id()
                    .is_some_and(|identifier| identifier.as_str() == focus)
            })
    );

    let fence = &receipt["document_fence"];
    let committed_molecule_id =
        DocumentObjectIdV1::from_class_source("cdml/molecule", "source-molecule")
            .expect("source molecule has a durable document identity");
    let follow_up = execute(json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "compact-follow-up",
        "operation": {
            "kind": "document.molecule.hydrogen.materialize.v1",
            "document": {
                "cdml": document,
                "expected_revision": fence["expected_revision"],
                "expected_digest_hex": fence["expected_digest_hex"],
            },
            "molecule_id": committed_molecule_id.as_str(),
            "anchor_atom_id": receipt["replacement_focus_atom_id"],
        },
    }));
    assert_eq!(
        follow_up["outcome"]["kind"],
        "document.molecule.hydrogen.materialize.v1"
    );
}

#[test]
fn stale_and_foreign_compact_refusals_leave_the_source_snapshot_current() {
    let source_digest = digest(COMPACT_CDML);
    let stale = execute(compact_request(
        COMPACT_CDML,
        1,
        &source_digest,
        "source-group",
    ));
    let foreign = execute(compact_request(
        COMPACT_CDML,
        0,
        &source_digest,
        "foreign-group",
    ));
    for (response, category, recovery) in [
        (stale, "stale_document_fence", "refresh_and_retry"),
        (foreign, "unknown_or_foreign_target", "correct_target"),
    ] {
        let refusal = &response["error"]["compact_group_materialization_refusal"];
        assert_eq!(refusal["category"], category);
        assert_eq!(refusal["recovery"], recovery);
        let serialized = response.to_string();
        assert!(!serialized.contains("methyl") && !serialized.contains("foreign-group"));
    }
    assert_eq!(digest(COMPACT_CDML), source_digest);
}

#[test]
fn compact_materialization_uses_the_shared_response_budget_refusal() {
    let oversized_group_id = "g".repeat(ferrum_api::DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1);
    let document = COMPACT_CDML.replace("source-group", &oversized_group_id);
    let response = execute(compact_request(
        &document,
        0,
        &digest(&document),
        &oversized_group_id,
    ));
    assert_eq!(
        response["error"]["operation"],
        "document.compact-group.materialize.v1"
    );
    assert_eq!(
        response["error"]["resource_limit"]["reason"],
        "response_size_exceeded"
    );
}
