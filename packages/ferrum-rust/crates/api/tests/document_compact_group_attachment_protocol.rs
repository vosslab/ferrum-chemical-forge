use ferrum_api::execute_operation_v1;
use ferrum_document::{
    DocumentSession, load_document_utf8_bytes_with_budget, local_cdml_ingress_format_v1,
};
use serde_json::{Value, json};

const COMPACT_CDML: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:object=\"urn:ferrum:document-object:v1\">",
    "<molecule id=\"source-molecule\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000001\">",
    "<atom id=\"anchor\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000002\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
    "</molecule></cdml>"
);

fn session(document: &str) -> DocumentSession {
    load_document_utf8_bytes_with_budget(document.as_bytes(), local_cdml_ingress_format_v1())
        .expect("inline attachment document admits")
}

fn fence(document: &str) -> Value {
    let snapshot = session(document).snapshot().expect("document snapshots");
    json!({
        "expected_revision": snapshot.revision(),
        "expected_digest_hex": snapshot.digest().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
    })
}

fn attach_request(document: &str, molecule_id: &str, anchor_atom_id: &str) -> Value {
    let document_fence = fence(document);
    json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "compact-group-attachment-protocol-test",
        "operation": {
            "kind": "document.compact-group.attach.v1",
            "document": {
                "cdml": document,
                "expected_revision": document_fence["expected_revision"],
                "expected_digest_hex": document_fence["expected_digest_hex"],
            },
            "molecule_id": molecule_id,
            "anchor_atom_id": anchor_atom_id,
            "catalog_key": "methyl",
            "release": { "x": 40.0, "y": 0.0 },
        },
    })
}

fn execute(request: Value) -> Value {
    serde_json::to_value(
        execute_operation_v1(&request.to_string()).expect("attachment request is protocol JSON"),
    )
    .expect("protocol envelope serializes")
}

#[test]
fn generic_attachment_commits_one_fenced_receipt_without_release_geometry() {
    let response = execute(attach_request(
        COMPACT_CDML,
        "ferrum-document-object-v1/00000000000000000000000000000001",
        "ferrum-document-object-v1/00000000000000000000000000000002",
    ));
    let receipt = &response["outcome"]["attachment"];
    assert_eq!(
        response["outcome"]["kind"],
        "document.compact-group.attach.v1"
    );
    assert_eq!(
        receipt["molecule_id"],
        "ferrum-document-object-v1/00000000000000000000000000000001"
    );
    assert_eq!(
        receipt["anchor_atom_id"],
        "ferrum-document-object-v1/00000000000000000000000000000002"
    );
    assert_eq!(receipt["catalog_key"], "methyl");
    assert!(receipt["compact_group_id"].as_str().is_some());
    assert!(receipt.get("release").is_none());
    assert!(receipt.get("candidate").is_none());
    let reopened_snapshot = session(receipt["document"].as_str().expect("committed CDML"))
        .snapshot()
        .expect("returned CDML re-admits");
    assert_eq!(
        receipt["document_fence"]["expected_revision"],
        reopened_snapshot.revision()
    );
    assert_eq!(
        receipt["document_fence"]["expected_digest_hex"],
        reopened_snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
}

#[test]
fn unknown_target_and_unknown_catalog_key_return_one_typed_refusal() {
    let foreign = execute(attach_request(
        COMPACT_CDML,
        "ferrum-document-object-v1/00000000000000000000000000000001",
        "ferrum-document-object-v1/0000000000000000000000000000000f",
    ));
    assert_eq!(
        foreign["error"]["compact_group_attachment_refusal"]["category"],
        "unknown_target"
    );

    let mut invalid = attach_request(
        COMPACT_CDML,
        "ferrum-document-object-v1/00000000000000000000000000000001",
        "ferrum-document-object-v1/00000000000000000000000000000002",
    );
    invalid["operation"]["catalog_key"] = json!("not-a-catalog-key");
    let decoded = execute(invalid);
    assert_eq!(decoded["error"]["category"], "invalid_request");
    assert!(
        decoded["error"]
            .get("compact_group_attachment_refusal")
            .is_none()
    );
}
