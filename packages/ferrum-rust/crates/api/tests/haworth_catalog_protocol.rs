//! Protocol coverage for the sealed D-glucose Haworth catalog branch.

use ferrum_api::{OperationProtocolEnvelopeV1, OperationProtocolOutcomeV1, execute_operation_v1};
use ferrum_document::DocumentSession;

fn digest(document: &str) -> String {
    DocumentSession::load(document)
        .expect("fixture")
        .snapshot()
        .expect("snapshot")
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn request(operation: serde_json::Value) -> String {
    serde_json::json!({"schema":"ferrum-operation-request-v1","request_id":"haworth-catalog","operation":operation}).to_string()
}

#[test]
fn protocol_lists_and_inserts_a_closed_haworth_entry_without_recipe_payload() {
    let listed = execute_operation_v1(&request(serde_json::json!({
        "kind":"catalog.list.v1", "family":"biomolecule", "category":"carbohydrates_d_glucose", "query":"pyranose"
    }))).expect("list");
    let OperationProtocolEnvelopeV1::Success(listed) = listed else {
        panic!("listed")
    };
    let OperationProtocolOutcomeV1::CatalogList { entries, .. } = listed.outcome else {
        panic!("catalog list")
    };
    assert_eq!(entries.len(), 2);
    let key = "biomolecules/carbohydrates/d-glucose/beta-d-glucopyranose";
    assert!(entries.iter().any(|entry| entry.id == key));
    let wire = serde_json::to_value(&entries).expect("summary JSON");
    assert!(wire[0].get("recipe").is_none());
    assert!(wire[0].get("document").is_none());

    let document = "<cdml xmlns=\"urn:ferrum:cdml\"/>";
    let inserted = execute_operation_v1(&request(serde_json::json!({
        "kind":"catalog.insert.v1", "document":document, "expected_revision":0,
        "expected_digest_hex":digest(document), "catalog_id":key, "anchor_x":42.0, "anchor_y":-9.0
    })))
    .expect("insert");
    let OperationProtocolEnvelopeV1::Success(inserted) = inserted else {
        panic!("inserted")
    };
    let OperationProtocolOutcomeV1::CatalogInsert {
        document,
        committed_revision,
        ..
    } = inserted.outcome
    else {
        panic!("catalog insert")
    };
    assert_eq!(committed_revision, 1);
    assert_eq!(document.matches("type=\"q1\"").count(), 1);
    assert_eq!(document.matches("type=\"w1\"").count(), 2);
    assert!(document.contains("name=\"beta-D-glucopyranose\""));
}
