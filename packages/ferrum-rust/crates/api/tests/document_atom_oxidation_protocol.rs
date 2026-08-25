use std::io::Write;
use std::process::{Command, Stdio};

use ferrum_api::{OperationProtocolEnvelopeV1, execute_operation_v1};
use ferrum_document::{
    DocumentSession, load_document_utf8_bytes_with_budget, local_cdml_ingress_format_v1,
};
use serde_json::{Value, json};

const DOCUMENT_OBJECT_NAMESPACE_V1: &str = "urn:ferrum:document-object:v1";
const MOLECULE_OBJECT_ID: &str = "ferrum-document-object-v1/00000000000000000000000000000000";
const OXYGEN_OBJECT_ID: &str = "ferrum-document-object-v1/00000000000000000000000000000001";

const WATER: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:ferrum-object=\"urn:ferrum:document-object:v1\" version=\"1.0\"><molecule id=\"water\" ferrum-object:id=\"ferrum-document-object-v1/00000000000000000000000000000000\">",
    "<atom id=\"oxygen\" ferrum-object:id=\"ferrum-document-object-v1/00000000000000000000000000000001\" name=\"O\" charge=\"0\" explicit_hydrogens=\"0\"><point x=\"0\" y=\"0\"/></atom>",
    "<atom id=\"hydrogen-a\" ferrum-object:id=\"ferrum-document-object-v1/00000000000000000000000000000002\" name=\"H\" charge=\"0\" explicit_hydrogens=\"0\"><point x=\"1\" y=\"0\"/></atom>",
    "<atom id=\"hydrogen-b\" ferrum-object:id=\"ferrum-document-object-v1/00000000000000000000000000000003\" name=\"H\" charge=\"0\" explicit_hydrogens=\"0\"><point x=\"-1\" y=\"0\"/></atom>",
    "<bond id=\"bond-a\" ferrum-object:id=\"ferrum-document-object-v1/00000000000000000000000000000004\" start=\"oxygen\" end=\"hydrogen-a\" type=\"n1\"/>",
    "<bond id=\"bond-b\" ferrum-object:id=\"ferrum-document-object-v1/00000000000000000000000000000005\" start=\"oxygen\" end=\"hydrogen-b\" type=\"n1\"/>",
    "</molecule></cdml>"
);

const CHLORIDE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:ferrum-object=\"urn:ferrum:document-object:v1\" version=\"1.0\"><molecule id=\"chloride\" ferrum-object:id=\"ferrum-document-object-v1/00000000000000000000000000000000\">",
    "<atom id=\"chlorine\" ferrum-object:id=\"ferrum-document-object-v1/00000000000000000000000000000001\" name=\"Cl\" charge=\"-1\" explicit_hydrogens=\"0\"><point x=\"0\" y=\"0\"/></atom>",
    "</molecule></cdml>"
);

fn admitted_session(document: &str) -> DocumentSession {
    load_document_utf8_bytes_with_budget(document.as_bytes(), local_cdml_ingress_format_v1())
        .expect("inline CDML admits")
}

fn digest(document: &str) -> String {
    admitted_session(document)
        .snapshot()
        .expect("fixture snapshots")
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn selection(document: &str) -> (String, String) {
    let session = admitted_session(document);
    let observation = session.observe(0).expect("fixture observes");
    let molecule = &observation.projection().molecules()[0];
    (
        molecule
            .id()
            .expect("molecule has durable ID")
            .as_str()
            .to_owned(),
        molecule.atoms()[0]
            .id()
            .expect("oxygen has durable ID")
            .as_str()
            .to_owned(),
    )
}

fn request(document: &str, revision: u64, expected_digest: String) -> Value {
    let (molecule_id, atom_id) = selection(document);
    json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "oxidation-protocol-test",
        "operation": {
            "kind": "document.atom.oxidation.observe.v1",
            "document": {
                "cdml": document,
                "expected_revision": revision,
                "expected_digest_hex": expected_digest,
            },
            "molecule_id": molecule_id,
            "atom_id": atom_id,
        },
    })
}

fn resource_limited_document() -> String {
    let atoms = (0..257)
        .map(|index| {
            format!(
                "<atom id=\"hydrogen-{index}\" ferrum-object:id=\"ferrum-document-object-v1/{:032x}\" name=\"H\" charge=\"0\" explicit_hydrogens=\"0\"><point x=\"0\" y=\"0\"/></atom>",
                index + 1,
            )
        })
        .collect::<String>();
    format!(
        "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:ferrum-object=\"{DOCUMENT_OBJECT_NAMESPACE_V1}\"><molecule id=\"large\" ferrum-object:id=\"{MOLECULE_OBJECT_ID}\">{atoms}</molecule></cdml>"
    )
}

#[test]
fn protocol_returns_a_fenced_accepted_or_unavailable_observation() {
    let accepted = execute_operation_v1(&request(WATER, 0, digest(WATER)).to_string())
        .expect("accepted request decodes");
    let unavailable = execute_operation_v1(&request(CHLORIDE, 0, digest(CHLORIDE)).to_string())
        .expect("unavailable request decodes");

    let accepted_json = serde_json::to_value(accepted).expect("accepted envelope serializes");
    let unavailable_json =
        serde_json::to_value(unavailable).expect("unavailable envelope serializes");
    assert_eq!(
        accepted_json["outcome"]["observation"]["oxidation_number"], -2,
        "{accepted_json}"
    );
    assert_eq!(
        unavailable_json["outcome"]["observation"]["unavailable_reason"],
        "element_outside_profile"
    );
    let accepted_observation = &accepted_json["outcome"]["observation"];
    let unavailable_observation = &unavailable_json["outcome"]["observation"];
    assert_eq!(accepted_observation["source_revision"], 0);
    assert_eq!(accepted_observation["source_digest_hex"], digest(WATER));
    assert_eq!(accepted_observation["molecule_id"], MOLECULE_OBJECT_ID);
    assert_eq!(accepted_observation["atom_id"], OXYGEN_OBJECT_ID);
    assert_eq!(accepted_observation["document_paint_order"], 0);
    assert!(accepted_observation.get("unavailable_reason").is_none());
    assert!(unavailable_observation.get("oxidation_number").is_none());
    assert!(accepted_observation.get("cdml").is_none());
    assert!(accepted_observation.get("graph").is_none());
    assert!(accepted_observation.get("receipt").is_none());
}

#[test]
fn protocol_accepts_document_fence_and_refuses_invalid_selection_and_resource_limits() {
    let accepted = execute_operation_v1(&request(WATER, 1, digest(WATER)).to_string())
        .expect("source-provenance request decodes");
    let OperationProtocolEnvelopeV1::Success(accepted) = accepted else {
        panic!("source-provenance request must complete")
    };
    let (molecule_id, atom_id) = selection(WATER);
    let mut unknown_molecule = request(WATER, 0, digest(WATER));
    unknown_molecule["operation"]["molecule_id"] = json!(atom_id);
    let unknown_molecule = execute_operation_v1(&unknown_molecule.to_string())
        .expect("unknown molecule request decodes");
    let OperationProtocolEnvelopeV1::Error(unknown_molecule) = unknown_molecule else {
        panic!("unknown molecule must refuse")
    };
    let mut unknown_atom = request(WATER, 0, digest(WATER));
    unknown_atom["operation"]["atom_id"] = json!(molecule_id);
    let unknown_atom =
        execute_operation_v1(&unknown_atom.to_string()).expect("unknown atom request decodes");
    let OperationProtocolEnvelopeV1::Error(unknown_atom) = unknown_atom else {
        panic!("unknown atom must refuse")
    };
    let large = resource_limited_document();
    let resource_limit = execute_operation_v1(&request(&large, 0, digest(&large)).to_string())
        .expect("resource-limited request decodes");
    let OperationProtocolEnvelopeV1::Error(resource_limit) = resource_limit else {
        panic!("large root must refuse")
    };

    let accepted_json = serde_json::to_value(accepted).expect("accepted response serializes");
    assert_eq!(
        accepted_json["outcome"]["observation"]["source_revision"],
        1
    );
    assert_eq!(
        accepted_json["outcome"]["observation"]["source_digest_hex"],
        digest(WATER)
    );
    assert_eq!(
        serde_json::to_value(unknown_molecule.error).expect("error serializes")["category"],
        "molecule_not_direct_root"
    );
    assert_eq!(
        serde_json::to_value(unknown_atom.error).expect("error serializes")["category"],
        "atom_not_found"
    );
    assert_eq!(
        serde_json::to_value(resource_limit.error).expect("error serializes")["resource_limit"],
        json!({
            "reason": "oxidation_root_atoms_exceeded",
            "recovery": "use_smaller_root",
        })
    );
}

#[test]
fn protocol_rejects_unknown_fields_and_named_cli_preserves_stdin_envelopes() {
    let mut unknown_field = request(WATER, 0, digest(WATER));
    unknown_field["operation"]["unexpected"] = json!(true);
    let unknown_field = execute_operation_v1(&unknown_field.to_string())
        .expect("unknown-field request remains an error envelope");
    let OperationProtocolEnvelopeV1::Error(unknown_field) = unknown_field else {
        panic!("unknown field must refuse")
    };
    let payload = request(WATER, 0, digest(WATER)).to_string();
    let mut child = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["document-atom-oxidation-observe", "--request", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("named CLI starts");
    child
        .stdin
        .take()
        .expect("CLI stdin")
        .write_all(payload.as_bytes())
        .expect("CLI input writes");
    let output = child.wait_with_output().expect("named CLI completes");
    let cli_json: Value = serde_json::from_slice(&output.stdout).expect("CLI response is JSON");

    assert_eq!(
        serde_json::to_value(unknown_field.error).expect("error serializes")["category"],
        "invalid_request"
    );
    assert!(output.status.success());
    assert!(output.stdout.ends_with(b"\n"));
    assert_eq!(
        cli_json["request_id"], "oxidation-protocol-test",
        "{cli_json}"
    );
    assert_eq!(
        cli_json["outcome"]["kind"], "document.atom.oxidation.observe.v1",
        "{cli_json}"
    );
}
