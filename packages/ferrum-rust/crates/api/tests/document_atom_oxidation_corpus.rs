use ferrum_api::execute_operation_v1;
use ferrum_document::DocumentSession;
use serde_json::{Value, json};

struct Atom {
    id: String,
    element: &'static str,
    charge: i8,
}

fn cdml(molecule_id: &str, atoms: &[Atom], bonds: &[(usize, usize, &str)]) -> String {
    let atoms = atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| {
            format!(
                "<atom id=\"{}\" name=\"{}\" charge=\"{}\" explicit_hydrogens=\"0\"><point x=\"{index}\" y=\"0\"/></atom>",
                atom.id, atom.element, atom.charge
            )
        })
        .collect::<String>();
    let bonds = bonds
        .iter()
        .enumerate()
        .map(|(index, &(start, end, order))| {
            format!(
                "<bond id=\"bond-{index}\" start=\"{}\" end=\"{}\" type=\"{order}\"/>",
                atoms_for_bond_id(start, molecule_id),
                atoms_for_bond_id(end, molecule_id),
            )
        })
        .collect::<String>();
    format!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\"><molecule id=\"{molecule_id}\">{atoms}{bonds}</molecule></cdml>"
    )
}

fn atoms_for_bond_id(index: usize, molecule_id: &str) -> String {
    format!("{molecule_id}-atom-{index}")
}

fn numbered_atoms(elements: &[(&'static str, i8)], molecule_id: &str) -> Vec<Atom> {
    elements
        .iter()
        .enumerate()
        .map(|(index, &(element, charge))| Atom {
            id: atoms_for_bond_id(index, molecule_id),
            element,
            charge,
        })
        .collect()
}

fn digest(document: &str) -> String {
    DocumentSession::load(document)
        .expect("corpus CDML loads")
        .snapshot()
        .expect("corpus CDML snapshots")
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn selection(document: &str, atom_index: usize) -> (String, String) {
    let session = DocumentSession::load(document).expect("corpus CDML loads");
    let observation = session.observe(0).expect("corpus CDML observes");
    let molecule = &observation.projection().molecules()[0];
    (
        molecule
            .id()
            .expect("durable molecule ID")
            .as_str()
            .to_owned(),
        molecule.atoms()[atom_index]
            .id()
            .expect("durable atom ID")
            .as_str()
            .to_owned(),
    )
}

fn request(document: &str, atom_index: usize, revision: u64, digest: String) -> Value {
    let (molecule_id, atom_id) = selection(document, atom_index);
    json!({
        "schema": "ferrum-operation-request-v1",
        "request_id": "oxidation-corpus",
        "operation": {
            "kind": "document.atom.oxidation.observe.v1",
            "document": {
                "cdml": document,
                "expected_revision": revision,
                "expected_digest_hex": digest,
            },
            "molecule_id": molecule_id,
            "atom_id": atom_id,
        },
    })
}

fn observation(request: Value) -> Value {
    let response = execute_operation_v1(&request.to_string()).expect("request decodes");
    let response = serde_json::to_value(response).expect("response serializes");
    assert_eq!(
        response["outcome"]["kind"], "document.atom.oxidation.observe.v1",
        "{response}"
    );
    response["outcome"]["observation"].clone()
}

fn refusal_category(request: Value) -> Value {
    let response = execute_operation_v1(&request.to_string()).expect("request decodes");
    let response = serde_json::to_value(response).expect("response serializes");
    response["error"]["category"].clone()
}

fn resource_limited_document() -> String {
    let atoms = (0..257)
        .map(|index| Atom {
            id: format!("large-atom-{index}"),
            element: "H",
            charge: 0,
        })
        .collect::<Vec<_>>();
    cdml("large", &atoms, &[])
}

#[test]
fn public_operation_returns_representative_hcno_oxidation_numbers() {
    let cases = [
        ("h2", vec![("H", 0), ("H", 0)], vec![(0, 1, "n1")], 0, 0),
        (
            "water",
            vec![("O", 0), ("H", 0), ("H", 0)],
            vec![(0, 1, "n1"), (0, 2, "n1")],
            0,
            -2,
        ),
        (
            "ammonia",
            vec![("N", 0), ("H", 0), ("H", 0), ("H", 0)],
            vec![(0, 1, "n1"), (0, 2, "n1"), (0, 3, "n1")],
            0,
            -3,
        ),
        (
            "methane",
            vec![("C", 0), ("H", 0), ("H", 0), ("H", 0), ("H", 0)],
            vec![(0, 1, "n1"), (0, 2, "n1"), (0, 3, "n1"), (0, 4, "n1")],
            0,
            -4,
        ),
        (
            "carbon-dioxide",
            vec![("O", 0), ("C", 0), ("O", 0)],
            vec![(0, 1, "n2"), (1, 2, "n2")],
            1,
            4,
        ),
        (
            "hydroxide",
            vec![("O", -1), ("H", 0)],
            vec![(0, 1, "n1")],
            0,
            -2,
        ),
    ];

    for (name, elements, bonds, selected_atom, expected_number) in cases {
        let atoms = numbered_atoms(&elements, name);
        let document = cdml(name, &atoms, &bonds);
        let expected_digest = digest(&document);
        let (molecule_id, atom_id) = selection(&document, selected_atom);
        let receipt = observation(request(
            &document,
            selected_atom,
            0,
            expected_digest.clone(),
        ));

        assert_eq!(receipt["status"], "accepted", "{name}: {receipt}");
        assert_eq!(
            receipt["oxidation_number"], expected_number,
            "{name}: {receipt}"
        );
        assert_eq!(receipt["source_revision"], 0, "{name}: {receipt}");
        assert_eq!(
            receipt["source_digest_hex"], expected_digest,
            "{name}: {receipt}"
        );
        assert_eq!(receipt["molecule_id"], molecule_id, "{name}: {receipt}");
        assert_eq!(receipt["atom_id"], atom_id, "{name}: {receipt}");
    }
}

#[test]
fn public_operation_returns_closed_unavailable_and_distinct_refusals() {
    let outside_atoms = numbered_atoms(&[("O", 0), ("S", 0)], "outside-profile");
    let outside = cdml("outside-profile", &outside_atoms, &[(0, 1, "n1")]);
    let unavailable = observation(request(&outside, 0, 0, digest(&outside)));
    assert_eq!(unavailable["status"], "unavailable");
    assert_eq!(unavailable["unavailable_reason"], "element_outside_profile");

    let water_atoms = numbered_atoms(&[("O", 0), ("H", 0), ("H", 0)], "water");
    let water = cdml("water", &water_atoms, &[(0, 1, "n1"), (0, 2, "n1")]);
    let changed_water = water.replace("x=\"1\"", "x=\"2\"");
    let stale = request(&water, 0, 0, digest(&changed_water));
    assert_eq!(refusal_category(stale), "stale_document");

    let mut wrong_selection = request(&water, 0, 0, digest(&water));
    let (_, atom_id) = selection(&water, 0);
    wrong_selection["operation"]["molecule_id"] = json!(atom_id);
    assert_eq!(
        refusal_category(wrong_selection),
        "molecule_not_direct_root"
    );

    let large = resource_limited_document();
    let resource = execute_operation_v1(&request(&large, 0, 0, digest(&large)).to_string())
        .expect("resource request decodes");
    let resource = serde_json::to_value(resource).expect("resource response serializes");
    assert_eq!(resource["error"]["category"], "resource_limit");
    assert_eq!(
        resource["error"]["resource_limit"]["reason"],
        "oxidation_root_atoms_exceeded"
    );
    assert_eq!(
        resource["error"]["resource_limit"]["recovery"],
        "use_smaller_root"
    );
}
