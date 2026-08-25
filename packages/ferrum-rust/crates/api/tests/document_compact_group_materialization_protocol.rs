use ferrum_api::execute_operation_v1;
use ferrum_document::{
    DocumentObjectIdV1, DocumentSession, load_document_utf8_bytes_with_budget,
    local_cdml_ingress_format_v1,
};
use serde_json::{Value, json};

const COMPACT_CDML: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:object=\"urn:ferrum:document-object:v1\">",
    "<molecule id=\"source-molecule\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000001\">",
    "<atom id=\"anchor\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000002\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
    "<compact-group id=\"source-group\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000003\" version=\"1\" catalog-key=\"methyl\" ",
    "attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group>",
    "<bond id=\"outside\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000004\" start=\"anchor\" end=\"source-group\" type=\"n1\"/>",
    "</molecule></cdml>"
);

fn protocol_session(document: &str) -> DocumentSession {
    load_document_utf8_bytes_with_budget(document.as_bytes(), local_cdml_ingress_format_v1())
        .expect("inline compact document admits")
}

fn digest(document: &str) -> String {
    protocol_session(document)
        .snapshot()
        .expect("inline compact document snapshots")
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn compact_target_ids(document: &str) -> (String, String) {
    let observation = protocol_session(document)
        .observe(0)
        .expect("compact document observes");
    let molecule = &observation.projection().molecules()[0];
    (
        molecule.document_object_id().as_str().to_owned(),
        molecule.compact_groups()[0]
            .document_object_id()
            .as_str()
            .to_owned(),
    )
}

fn compact_request(
    document: &str,
    revision: u64,
    digest_hex: &str,
    molecule_id: &str,
    compact_group_id: &str,
) -> Value {
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
            "molecule_id": molecule_id,
            "compact_group_id": compact_group_id,
        },
    })
}

fn execute(request: Value) -> Value {
    serde_json::to_value(
        execute_operation_v1(&request.to_string()).expect("compact request is protocol JSON"),
    )
    .expect("protocol envelope serializes")
}

fn inspected_fence(document: &str) -> Value {
    execute(json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "compact-group-inspect",
        "operation": {
            "kind": "document.inspect",
            "document": document,
        },
    }))["outcome"]["document_fence"]
        .clone()
}

fn materialized() -> Value {
    let (molecule_id, compact_group_id) = compact_target_ids(COMPACT_CDML);
    let fence = inspected_fence(COMPACT_CDML);
    execute(compact_request(
        COMPACT_CDML,
        fence["expected_revision"]
            .as_u64()
            .expect("inspection returns a revision"),
        fence["expected_digest_hex"]
            .as_str()
            .expect("inspection returns a digest"),
        &molecule_id,
        &compact_group_id,
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
    let (molecule_id, compact_group_id) = compact_target_ids(COMPACT_CDML);
    assert_eq!(receipt["molecule_id"], molecule_id);
    assert_eq!(receipt["compact_group_id"], compact_group_id);
    let document = receipt["document"]
        .as_str()
        .expect("accepted CDML document");
    let focus = receipt["replacement_focus_atom_id"]
        .as_str()
        .expect("replacement focus identifier");
    let session = protocol_session(document);
    let observation = session.observe(0).expect("accepted CDML observes");
    assert!(
        observation.projection().molecules()[0]
            .atoms()
            .iter()
            .any(|atom| { atom.document_object_id().as_str() == focus })
    );

    let fence = &receipt["document_fence"];
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
            "molecule_id": receipt["molecule_id"],
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
    let (molecule_id, compact_group_id) = compact_target_ids(COMPACT_CDML);
    let fence = inspected_fence(COMPACT_CDML);
    let revision = fence["expected_revision"]
        .as_u64()
        .expect("inspection returns a revision");
    let digest_hex = fence["expected_digest_hex"]
        .as_str()
        .expect("inspection returns a digest");
    let stale = execute(compact_request(
        COMPACT_CDML,
        revision + 1,
        digest_hex,
        &molecule_id,
        &compact_group_id,
    ));
    let foreign = execute(compact_request(
        COMPACT_CDML,
        revision,
        digest_hex,
        &molecule_id,
        DocumentObjectIdV1::from_entropy_bytes([0xf1; 16]).as_str(),
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
    let oversized_text = "g".repeat(ferrum_api::OPERATION_PROTOCOL_RESPONSE_UTF8_BYTES_V1);
    let document = COMPACT_CDML.replace(
        "</cdml>",
        &format!(
            "<text id=\"payload\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000005\"><point x=\"1\" y=\"2\"/><font family=\"Telex\"/><ftext>{oversized_text}</ftext></text></cdml>"
        ),
    );
    let (molecule_id, compact_group_id) = compact_target_ids(&document);
    let fence = inspected_fence(&document);
    let response = execute(compact_request(
        &document,
        fence["expected_revision"]
            .as_u64()
            .expect("inspection returns a revision"),
        fence["expected_digest_hex"]
            .as_str()
            .expect("inspection returns a digest"),
        &molecule_id,
        &compact_group_id,
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
