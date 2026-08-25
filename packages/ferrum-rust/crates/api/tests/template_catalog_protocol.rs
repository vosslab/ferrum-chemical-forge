use ferrum_api::{
    DocumentRequestFenceV1, OperationProtocolEnvelopeV1, OperationProtocolOutcomeV1,
    execute_operation_v1,
};
use ferrum_document::DocumentSession;
use serde_json::Value;

const EMPTY: &str = "<cdml xmlns=\"urn:ferrum:cdml\"/>";
const EXCLUDED: &str = "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:object=\"urn:ferrum:document-object:v1\"><text id=\"bad\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000001\"><point x=\"1\" y=\"2\"/><ftext><b>x</b></ftext></text></cdml>";

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

fn request(operation: serde_json::Value) -> String {
    serde_json::json!({"schema": "ferrum-operation-request-v1", "request_id": "catalog-protocol-test", "operation": operation}).to_string()
}

fn list(family: Option<&str>, category: Option<&str>, query: Option<&str>) -> String {
    request(serde_json::json!({
        "kind": "catalog.list.v1",
        "family": family,
        "category": category,
        "query": query,
    }))
}

fn listed_summaries(payload: &str) -> Vec<Value> {
    let response = execute_operation_v1(payload).expect("request decodes");
    let OperationProtocolEnvelopeV1::Success(response) = response else {
        panic!("list succeeds")
    };
    let OperationProtocolOutcomeV1::CatalogList { entries, .. } = response.outcome else {
        panic!("catalog list result expected")
    };
    entries
        .into_iter()
        .map(|entry| serde_json::to_value(entry).expect("summary serializes"))
        .collect()
}

fn catalog_subject() -> Value {
    listed_summaries(&request(serde_json::json!({"kind": "catalog.list.v1"})))
        .into_iter()
        .find(|entry| {
            entry["id"].as_str().is_some_and(|value| !value.is_empty())
                && entry["family"]
                    .as_str()
                    .is_some_and(|value| !value.is_empty())
                && entry["category"]["id"]
                    .as_str()
                    .is_some_and(|value| !value.is_empty())
        })
        .expect("public catalog.list.v1 must expose an entry with id, family, and category.id")
}

fn assert_summary_only(entry: &Value) {
    assert!(entry["id"].is_string());
    assert!(entry["family"].is_string());
    assert!(entry["category"]["id"].is_string());
    assert!(entry["category"]["name"].is_string());
    assert!(entry["name"].is_string());
    assert!(entry["provenance"]["source_kind"].is_string());
    assert!(entry["provenance"]["source_id"].is_string());
    assert!(entry["provenance"]["license_spdx"].is_string());
    assert!(entry.get("document").is_none());
    assert!(entry.get("template_cdml").is_none());
}

fn insert(document: &str, revision: u64, catalog_id: &str, x: f64, y: f64) -> String {
    insert_with_fence(
        document,
        &DocumentRequestFenceV1 {
            expected_revision: revision,
            ..current_fence(document)
        },
        catalog_id,
        x,
        y,
    )
}

fn current_fence(document: &str) -> DocumentRequestFenceV1 {
    DocumentRequestFenceV1 {
        expected_revision: 0,
        expected_digest_hex: digest(document),
    }
}

fn insert_with_fence(
    document: &str,
    fence: &DocumentRequestFenceV1,
    catalog_id: &str,
    x: f64,
    y: f64,
) -> String {
    request(
        serde_json::json!({"kind": "catalog.insert.v1", "document": document, "expected_revision": fence.expected_revision, "expected_digest_hex": fence.expected_digest_hex, "catalog_id": catalog_id, "anchor_x": x, "anchor_y": y}),
    )
}

#[test]
fn catalog_list_is_summary_only_and_does_not_leak_a_recipe() {
    let response = execute_operation_v1(&request(serde_json::json!({"kind": "catalog.list.v1"})))
        .expect("request decodes");
    let OperationProtocolEnvelopeV1::Success(response) = response else {
        panic!("list succeeds")
    };
    let OperationProtocolOutcomeV1::CatalogList {
        catalog_schema,
        entries,
        ..
    } = response.outcome
    else {
        panic!("catalog list result expected")
    };
    assert_eq!(catalog_schema, "ferrum-template-catalog-v1");
    for entry in entries {
        let summary = serde_json::to_value(entry).expect("summary serializes");
        assert_summary_only(&summary);
    }
    assert_summary_only(&catalog_subject());
}

#[test]
fn catalog_list_applies_requested_family_and_category_filters() {
    let subject = catalog_subject();
    let catalog_id = subject["id"].as_str().expect("catalog subject id");
    let family = subject["family"].as_str().expect("catalog subject family");
    let category_id = subject["category"]["id"]
        .as_str()
        .expect("catalog subject category id");
    let filtered = listed_summaries(&list(Some(family), Some(category_id), None));

    assert!(
        filtered
            .iter()
            .any(|entry| entry["id"].as_str() == Some(catalog_id)),
        "family/category filters must retain selected public catalog entry {catalog_id}"
    );
    for entry in filtered {
        assert_summary_only(&entry);
        assert_eq!(entry["family"].as_str(), Some(family));
        assert_eq!(entry["category"]["id"].as_str(), Some(category_id));
    }
}

#[test]
fn catalog_insert_returns_a_chainable_stateless_catalog_transition() {
    let subject = catalog_subject();
    let catalog_id = subject["id"].as_str().expect("catalog subject id");
    let first =
        execute_operation_v1(&insert(EMPTY, 0, catalog_id, 100.0, 50.0)).expect("request decodes");
    let OperationProtocolEnvelopeV1::Success(response) = first else {
        panic!("insert succeeds")
    };
    let OperationProtocolOutcomeV1::CatalogInsert {
        document,
        identifier,
        committed_revision,
        document_fence,
    } = response.outcome
    else {
        panic!("catalog insert result expected")
    };
    assert_eq!(committed_revision, 1);
    assert_eq!(document_fence.expected_revision, 0);
    assert_eq!(document_fence.expected_digest_hex, digest(&document));
    assert!(document.contains(&format!("id=\"{identifier}\"")));
    let second = execute_operation_v1(&insert_with_fence(
        &document,
        &document_fence,
        catalog_id,
        200.0,
        50.0,
    ))
    .expect("chainable request decodes");
    assert!(matches!(second, OperationProtocolEnvelopeV1::Success(_)));
}

#[test]
fn catalog_insert_refuses_stale_unknown_and_render_excluded_without_a_commit() {
    let subject = catalog_subject();
    let catalog_id = subject["id"].as_str().expect("catalog subject id");
    let missing_catalog_id = format!("{catalog_id}__catalog_protocol_missing");
    for (payload, category, recovery) in [
        (
            insert(EMPTY, 1, catalog_id, 1.0, 1.0),
            "stale_snapshot",
            "refresh_and_restart",
        ),
        (
            insert(EMPTY, 0, &missing_catalog_id, 1.0, 1.0),
            "unknown_key",
            "choose_catalog_entry",
        ),
        (
            insert(EXCLUDED, 0, catalog_id, 1.0, 1.0),
            "render_preparation",
            "document_unchanged",
        ),
    ] {
        let response = execute_operation_v1(&payload).expect("request decodes");
        let OperationProtocolEnvelopeV1::Error(response) = response else {
            panic!("request refuses")
        };
        let refusal = response
            .error
            .catalog_placement_refusal
            .expect("catalog facts");
        assert_eq!(serde_json::to_value(refusal.category).unwrap(), category);
        assert_eq!(serde_json::to_value(refusal.recovery).unwrap(), recovery);
    }
}
