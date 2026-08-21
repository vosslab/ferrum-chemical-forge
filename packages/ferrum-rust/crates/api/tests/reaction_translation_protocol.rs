use std::io::Write;
use std::process::{Command, Stdio};

use ferrum_api::{
    OperationProtocolEnvelopeV1, OperationProtocolOutcomeV1, execute_operation_v1,
    generated_operation_protocol_schema_v1, operation_protocol_schema_v1,
};
use ferrum_document::DocumentSession;

const CDML_NAMESPACE: &str = "http://www.freesoftware.fsf.org/bkchem/cdml";
const SOURCE: &str = concat!(
    "<c:cdml xmlns:c=\"http://www.freesoftware.fsf.org/bkchem/cdml\">",
    "<c:molecule id=\"left\"><c:atom id=\"left-a\" name=\"C\"><c:point x=\"0\" y=\"0\"/>",
    "</c:atom></c:molecule><c:molecule id=\"right\"><c:atom id=\"right-a\" name=\"O\">",
    "<c:point x=\"100\" y=\"0\"/></c:atom></c:molecule><c:arrow id=\"arrow\">",
    "<c:point x=\"25\" y=\"0\"/><c:point x=\"75\" y=\"0\"/></c:arrow>",
    "<c:reaction id=\"strict\"><c:reactant idref=\"left\"/><c:product idref=\"right\"/>",
    "<c:arrow idref=\"arrow\"/></c:reaction></c:cdml>"
);
const EXCLUDED_SOURCE: &str = concat!(
    "<cdml><molecule id=\"left\"><atom id=\"left-a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
    "</atom></molecule><molecule id=\"right\"><atom id=\"right-a\" name=\"O\">",
    "<point x=\"100\" y=\"0\"/></atom></molecule><arrow id=\"arrow\"><point x=\"25\" y=\"0\"/>",
    "<point x=\"75\" y=\"0\"/></arrow><plus id=\"bad\"><point x=\"1\" y=\"2\"/>",
    "<font family=\"Arial\"/></plus>",
    "<reaction id=\"strict\"><reactant idref=\"left\"/>",
    "<product idref=\"right\"/><arrow idref=\"arrow\"/></reaction></cdml>"
);

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

fn request(document: &str, revision: u64, snap: &str) -> String {
    serde_json::json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "reaction-translation-protocol-test",
        "operation": {
            "kind": "reaction.translate.v1",
            "document": document,
            "expected_revision": revision,
            "expected_digest_hex": digest(document),
            "reaction_id": "strict",
            "press_x": 0.0,
            "press_y": 0.0,
            "pointer_x": 20.0,
            "pointer_y": 10.0,
            "snap": snap,
        },
    })
    .to_string()
}

fn translated(response: OperationProtocolEnvelopeV1) -> String {
    let OperationProtocolEnvelopeV1::Success(response) = response else {
        panic!("translation must succeed: {response:?}")
    };
    let OperationProtocolOutcomeV1::ReactionTranslate {
        document,
        reaction_id,
        input_revision,
        committed_revision,
        next_input_expected_revision,
        ..
    } = response.outcome
    else {
        panic!("reaction translation result expected")
    };
    assert_eq!(reaction_id, "strict");
    assert_eq!(
        (
            input_revision,
            committed_revision,
            next_input_expected_revision
        ),
        (0, 1, 0),
    );
    document
}

fn coordinates(document: &str) -> Vec<(f64, f64)> {
    document
        .split("<c:point x=\"")
        .skip(1)
        .map(|part| {
            let (x, remainder) = part.split_once("\" y=\"").expect("point x and y");
            let y = remainder.split_once('\"').expect("point y terminator").0;
            (coordinate_value(x), coordinate_value(y))
        })
        .collect()
}

fn coordinate_value(value: &str) -> f64 {
    const POINTS_PER_CENTIMETER: f64 = 72.0 / 2.54;
    value
        .strip_suffix("cm")
        .map(|centimeters| {
            centimeters
                .parse::<f64>()
                .unwrap_or_else(|_| panic!("invalid centimeter value {value:?}"))
                * POINTS_PER_CENTIMETER
        })
        .unwrap_or_else(|| {
            value
                .parse()
                .unwrap_or_else(|_| panic!("invalid point value {value:?}"))
        })
}

fn assert_coordinates_near(actual: &[(f64, f64)], expected: &[(f64, f64)]) {
    assert_eq!(actual.len(), expected.len());
    for ((actual_x, actual_y), (expected_x, expected_y)) in actual.iter().zip(expected) {
        assert!((actual_x - expected_x).abs() < 0.02);
        assert!((actual_y - expected_y).abs() < 0.02);
    }
}

#[test]
fn checked_in_schema_is_current_and_declares_the_complete_translation_contract() {
    let checked_in: serde_json::Value =
        serde_json::from_str(operation_protocol_schema_v1()).expect("checked-in schema is JSON");
    assert_eq!(checked_in, generated_operation_protocol_schema_v1());
    let rendered = checked_in.to_string();
    for fact in [
        "reaction.translate.v1",
        "view_hex_grid",
        "renderer_exclusion",
        "document.molecule.smarts.query.v1",
        "DocumentSmartsQueryRequestV1",
    ] {
        assert!(rendered.contains(fact), "schema omits {fact}");
    }
}

#[test]
fn protocol_moves_the_complete_prefixed_aggregate_for_free_and_hex_grid_snaps() {
    let free = translated(execute_operation_v1(&request(SOURCE, 0, "free")).expect("JSON request"));
    assert_coordinates_near(
        &coordinates(&free),
        &[(20.0, 10.0), (120.0, 10.0), (45.0, 10.0), (95.0, 10.0)],
    );
    assert!(free.contains("xmlns:c=\"http://www.freesoftware.fsf.org/bkchem/cdml\""));
    for reference in ["idref=\"left\"", "idref=\"right\"", "idref=\"arrow\""] {
        assert!(free.contains(reference), "translation changed {reference}");
    }

    let grid = translated(
        execute_operation_v1(&request(SOURCE, 0, "view_hex_grid")).expect("JSON request"),
    );
    let translated_coordinates = coordinates(&grid);
    let dx = translated_coordinates[0].0;
    let dy = translated_coordinates[0].1;
    assert_ne!((dx, dy), (0.0, 0.0));
    for ((source_x, source_y), (result_x, result_y)) in
        coordinates(SOURCE).into_iter().zip(translated_coordinates)
    {
        assert!(((result_x - source_x) - dx).abs() < 0.02);
        assert!(((result_y - source_y) - dy).abs() < 0.02);
    }
}

#[test]
fn protocol_and_named_cli_route_preserve_closed_refusals_and_stateless_input_rules() {
    let stale = execute_operation_v1(&request(SOURCE, 1, "free")).expect("JSON request");
    let OperationProtocolEnvelopeV1::Error(stale) = stale else {
        panic!("nonzero stateless revision must refuse")
    };
    assert_eq!(
        serde_json::to_value(stale.error.reaction_refusal.expect("reaction refusal"))
            .expect("refusal serializes")["category"],
        "stale_snapshot",
    );
    let excluded =
        execute_operation_v1(&request(EXCLUDED_SOURCE, 0, "free")).expect("JSON request");
    let OperationProtocolEnvelopeV1::Error(excluded) = excluded else {
        panic!("renderer-excluded candidate must refuse")
    };
    assert_eq!(
        serde_json::to_value(excluded.error.reaction_refusal.expect("reaction refusal"))
            .expect("refusal serializes")["category"],
        "renderer_exclusion",
    );

    let payload = request(SOURCE, 0, "free");
    let mut child = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["document", "command", "reaction.translate.v1", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("named CLI starts");
    child
        .stdin
        .take()
        .expect("CLI stdin")
        .write_all(payload.as_bytes())
        .expect("CLI request writes");
    let output = child.wait_with_output().expect("named CLI completes");
    assert!(
        output.status.success(),
        "CLI stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).expect("CLI JSON");
    assert_eq!(response["outcome"]["kind"], "reaction.translate.v1");
    assert_eq!(response["outcome"]["committed_revision"], 1);
}

#[test]
fn protocol_fixture_retains_the_declared_cdml_namespace() {
    assert!(SOURCE.contains(CDML_NAMESPACE));
}
