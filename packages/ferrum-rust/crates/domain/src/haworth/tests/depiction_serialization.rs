use crate::haworth::{HaworthDepiction, RingForm, layout_single_ring};

use super::fixtures::request;

#[test]
fn json_is_deterministic_and_validated_at_the_boundary() {
    let depiction =
        layout_single_ring(&request(RingForm::Pyranose, 12.0, false, false)).expect("layout");
    let first = serde_json::to_string(&depiction).expect("json");
    let second = serde_json::to_string(&depiction).expect("json");
    assert_eq!(first, second);
    let round_trip: HaworthDepiction = serde_json::from_str(&first).expect("validated round trip");
    assert_eq!(round_trip, depiction);
}

#[test]
fn serde_rejects_duplicate_entries_and_stale_bounds() {
    let depiction =
        layout_single_ring(&request(RingForm::Pyranose, 12.0, false, false)).expect("layout");
    let mut duplicate: serde_json::Value = serde_json::to_value(&depiction).expect("wire");
    let coordinates = duplicate["coordinates"]
        .as_array_mut()
        .expect("coordinates");
    coordinates.push(coordinates[0].clone());
    assert!(serde_json::from_value::<HaworthDepiction>(duplicate).is_err());

    let mut stale: serde_json::Value = serde_json::to_value(&depiction).expect("wire");
    stale["bounds"][0]["x"] = serde_json::json!(999.0);
    assert!(serde_json::from_value::<HaworthDepiction>(stale).is_err());

    let mut wrong_identity: serde_json::Value = serde_json::to_value(&depiction).expect("wire");
    wrong_identity["coordinates"][0][0]["kind"] = serde_json::json!("Bond");
    assert!(serde_json::from_value::<HaworthDepiction>(wrong_identity).is_err());

    let mut repeated_role: serde_json::Value = serde_json::to_value(&depiction).expect("wire");
    let bonds = repeated_role["bonds"].as_array_mut().expect("bonds");
    let front_indices: Vec<_> = bonds
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| entry[1].get("HaworthFront").is_some().then_some(index))
        .collect();
    let role = bonds[front_indices[0]][1]["HaworthFront"]["edge_role"].clone();
    bonds[front_indices[1]][1]["HaworthFront"]["edge_role"] = role;
    assert!(serde_json::from_value::<HaworthDepiction>(repeated_role).is_err());

    let nonfinite = serde_json::to_string(&depiction)
        .expect("wire")
        .replace("12.0", "1e999");
    assert!(serde_json::from_str::<HaworthDepiction>(&nonfinite).is_err());
}
