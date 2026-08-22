use ferrum_api::{OperationProtocolEnvelopeV1, OperationProtocolOutcomeV1, execute_operation_v1};
use ferrum_document::DocumentSession;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

const EMPTY: &str = "<cdml xmlns=\"urn:ferrum:cdml\"/>";
const EXCLUDED: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><text id=\"bad\"><point x=\"1\" y=\"2\"/><ftext><b>x</b></ftext></text></cdml>";

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

fn listed_ids(payload: &str) -> Vec<String> {
    let response = execute_operation_v1(payload).expect("request decodes");
    let OperationProtocolEnvelopeV1::Success(response) = response else {
        panic!("list succeeds")
    };
    let OperationProtocolOutcomeV1::CatalogList { entries, .. } = response.outcome else {
        panic!("catalog list result expected")
    };
    entries.into_iter().map(|entry| entry.id).collect()
}

fn listed_summary_facts(payload: &str) -> BTreeMap<String, Value> {
    let response = execute_operation_v1(payload).expect("request decodes");
    let OperationProtocolEnvelopeV1::Success(response) = response else {
        panic!("list succeeds")
    };
    let OperationProtocolOutcomeV1::CatalogList { entries, .. } = response.outcome else {
        panic!("catalog list result expected")
    };
    entries
        .into_iter()
        .map(|entry| {
            let value = serde_json::to_value(entry).expect("summary serializes");
            let id = value["id"].as_str().expect("summary has ID").to_owned();
            (id, value)
        })
        .collect()
}

fn expected_haworth_summary_facts() -> BTreeMap<String, Value> {
    [
        (
            "biomolecules/carbohydrates/d-glucose/alpha-d-glucopyranose",
            "alpha-D-glucopyranose",
        ),
        (
            "biomolecules/carbohydrates/d-glucose/beta-d-glucopyranose",
            "beta-D-glucopyranose",
        ),
        (
            "biomolecules/carbohydrates/d-glucose/alpha-d-glucofuranose",
            "alpha-D-glucofuranose",
        ),
        (
            "biomolecules/carbohydrates/d-glucose/beta-d-glucofuranose",
            "beta-D-glucofuranose",
        ),
    ]
    .into_iter()
    .map(|(id, name)| {
        (
            id.to_owned(),
            json!({
                "id": id,
                "family": "biomolecule",
                "category": {
                    "id": "carbohydrates_d_glucose",
                    "name": "Carbohydrates / D-glucose",
                },
                "name": name,
                "provenance": {
                    "source_kind": "curated_ferrum",
                    "source_id": "ferrum-authored-d-glucose-haworth-depictions-v1",
                    "license_spdx": "LGPL-3.0-only",
                },
            }),
        )
    })
    .collect()
}

fn public_haworth_summary_facts(entries: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    entries
        .into_iter()
        .map(|(id, entry)| {
            let mut summary = Map::new();
            summary.insert("id".to_owned(), entry["id"].clone());
            summary.insert("family".to_owned(), entry["family"].clone());
            summary.insert(
                "category".to_owned(),
                json!({
                    "id": entry["category"]["id"],
                    "name": entry["category"]["name"],
                }),
            );
            summary.insert("name".to_owned(), entry["name"].clone());
            summary.insert(
                "provenance".to_owned(),
                json!({
                    "source_kind": entry["provenance"]["source_kind"],
                    "source_id": entry["provenance"]["source_id"],
                    "license_spdx": entry["provenance"]["license_spdx"],
                }),
            );
            (id, Value::Object(summary))
        })
        .collect()
}

fn insert(document: &str, revision: u64, catalog_id: &str, x: f64, y: f64) -> String {
    request(
        serde_json::json!({"kind": "catalog.insert.v1", "document": document, "expected_revision": revision, "expected_digest_hex": digest(document), "catalog_id": catalog_id, "anchor_x": x, "anchor_y": y}),
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
    let wire = serde_json::to_value(entries).expect("summary serializes");
    assert_eq!(wire[0]["id"], "system/rings/benzene");
    assert!(wire[0].get("document").is_none());
    assert!(wire[0].get("template_cdml").is_none());
}

#[test]
fn catalog_list_exposes_the_sealed_haworth_biomolecule_summary_slice() {
    let expected = expected_haworth_summary_facts();
    let biomolecules = listed_summary_facts(&list(Some("biomolecule"), None, None));
    assert_eq!(public_haworth_summary_facts(biomolecules), expected);
    assert!(listed_ids(&list(Some("biomolecule"), Some("rings"), None)).is_empty());
    assert!(listed_ids(&list(Some("system"), Some("carbohydrates_d_glucose"), None)).is_empty());
    assert!(listed_ids(&list(None, Some("missing"), None)).is_empty());
}

#[test]
fn catalog_list_applies_closed_summary_filters_as_an_intersection() {
    assert_eq!(
        listed_ids(&list(Some("system"), None, None)),
        [
            "system/rings/benzene",
            "system/rings/cyclopropane",
            "system/rings/cyclobutane",
            "system/rings/cyclopentane",
            "system/rings/cyclohexane",
            "system/heterocycles/thiophene",
            "system/heterocycles/furan",
            "system/heterocycles/pyrrole",
            "system/heterocycles/purine"
        ]
    );
    assert_eq!(
        listed_ids(&list(None, Some("rings"), None)),
        [
            "system/rings/benzene",
            "system/rings/cyclopropane",
            "system/rings/cyclobutane",
            "system/rings/cyclopentane",
            "system/rings/cyclohexane"
        ]
    );
    assert!(listed_ids(&list(None, Some("Rings"), None)).is_empty());
    assert_eq!(
        listed_ids(&list(None, None, Some("  SYSTEM/RINGS  "))),
        [
            "system/rings/benzene",
            "system/rings/cyclopropane",
            "system/rings/cyclobutane",
            "system/rings/cyclopentane",
            "system/rings/cyclohexane"
        ]
    );
    assert!(listed_ids(&list(Some("system"), Some("rings"), Some("missing"))).is_empty());
    assert_eq!(
        public_haworth_summary_facts(listed_summary_facts(&list(
            Some("biomolecule"),
            Some("carbohydrates_d_glucose"),
            Some("beta"),
        ))),
        expected_haworth_summary_facts()
            .into_iter()
            .filter(|(id, _)| id.contains("beta-"))
            .collect()
    );
    assert_eq!(
        listed_ids(&list(None, Some("heterocycles"), Some("sulfur"))),
        ["system/heterocycles/thiophene"]
    );
}

#[test]
fn catalog_insert_returns_a_chainable_stateless_benzene_transition() {
    let first = execute_operation_v1(&insert(EMPTY, 0, "system/rings/benzene", 100.0, 50.0))
        .expect("request decodes");
    let OperationProtocolEnvelopeV1::Success(response) = first else {
        panic!("insert succeeds")
    };
    let OperationProtocolOutcomeV1::CatalogInsert {
        document,
        identifier,
        input_revision,
        committed_revision,
        next_input_expected_revision,
        ..
    } = response.outcome
    else {
        panic!("catalog insert result expected")
    };
    assert_eq!(
        (
            input_revision,
            committed_revision,
            next_input_expected_revision
        ),
        (0, 1, 0)
    );
    assert!(document.contains(&format!("id=\"{identifier}\"")));
    assert_eq!(document.matches("name=\"C\"").count(), 6);
    assert_eq!(document.matches("type=\"n2\"").count(), 3);
    let second = execute_operation_v1(&insert(&document, 0, "system/rings/benzene", 200.0, 50.0))
        .expect("chainable request decodes");
    assert!(matches!(second, OperationProtocolEnvelopeV1::Success(_)));
}

#[test]
fn catalog_insert_refuses_stale_unknown_and_render_excluded_without_a_commit() {
    for (payload, category, recovery) in [
        (
            insert(EMPTY, 1, "system/rings/benzene", 1.0, 1.0),
            "stale_snapshot",
            "refresh_and_restart",
        ),
        (
            insert(EMPTY, 0, "missing", 1.0, 1.0),
            "unknown_key",
            "choose_catalog_entry",
        ),
        (
            insert(EXCLUDED, 0, "system/rings/benzene", 1.0, 1.0),
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
