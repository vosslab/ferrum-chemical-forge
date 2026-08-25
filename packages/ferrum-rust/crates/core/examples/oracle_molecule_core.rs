//! Isolated projection for comparing the ordered molecule-core carrier.

use std::collections::HashMap;
use std::env;
use std::io::{self, Read};

use ferrum_core::{Atom, Bond, BondOrder, Identifier, Molecule, Position, VertexRef};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Request {
    capability: String,
    atoms: Vec<AtomRequest>,
    bonds: Vec<BondRequest>,
}

#[derive(Deserialize)]
struct AtomRequest {
    element: String,
    formal_charge: i32,
    explicit_hydrogens: u16,
}

#[derive(Deserialize)]
struct BondRequest {
    start: usize,
    end: usize,
    order: u8,
    r#type: String,
}

fn bond_order(value: u8) -> BondOrder {
    match value {
        1 => BondOrder::Single,
        2 => BondOrder::Double,
        3 => BondOrder::Triple,
        4 => BondOrder::Aromatic,
        other => BondOrder::Other(other),
    }
}

fn source_id(prefix: &str, index: usize) -> Identifier {
    let value = format!("{prefix}-{index}");
    Identifier::new(value).expect("constant source identifier is valid")
}

fn project(request: Request) -> Result<Value, String> {
    let mut atoms = Vec::new();
    for (index, atom) in request.atoms.iter().enumerate() {
        let atom = Atom::new(
            source_id("oracle-atom", index),
            Some(atom.element.clone()),
            Position::new(0.0, 0.0, 0.0).map_err(|error| error.to_string())?,
            Some(atom.formal_charge),
            Some(atom.explicit_hydrogens),
            None,
            None,
            None,
            None,
        )
        .map_err(|error| error.to_string())?;
        atoms.push(atom);
    }
    let mut bonds = Vec::new();
    for (index, bond) in request.bonds.iter().enumerate() {
        let start = atoms
            .get(bond.start)
            .ok_or_else(|| format!("bond {index} start is outside atom list"))?;
        let end = atoms
            .get(bond.end)
            .ok_or_else(|| format!("bond {index} end is outside atom list"))?;
        let bond = Bond::new(
            source_id("oracle-bond", index),
            VertexRef::Atom(start.identity().clone()),
            VertexRef::Atom(end.identity().clone()),
            Some(bond.r#type.clone()),
            Some(bond_order(bond.order)),
            None,
            None,
        )
        .map_err(|error| error.to_string())?;
        bonds.push(bond);
    }
    let _molecule = Molecule::new(
        Identifier::new("oracle-molecule").expect("constant id is valid"),
        None,
        atoms.clone(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        bonds.clone(),
    )
    .map_err(|error| error.to_string())?;
    let atom_projection: Vec<Value> = atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| {
            json!({
                "index": index,
                "element": atom.element(),
                "formal_charge": atom.formal_charge(),
                "explicit_hydrogens": atom.explicit_hydrogens(),
            })
        })
        .collect();
    let by_identity: HashMap<_, _> = atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| (atom.identity(), index))
        .collect();
    let bond_projection: Result<Vec<Value>, String> = bonds
        .iter()
        .map(|bond| {
            let VertexRef::Atom(start) = bond.start() else {
                return Err("oracle capability requires atom endpoints".to_owned());
            };
            let VertexRef::Atom(end) = bond.end() else {
                return Err("oracle capability requires atom endpoints".to_owned());
            };
            let start = by_identity
                .get(start)
                .ok_or_else(|| "bond start identity was not constructed".to_owned())?;
            let end = by_identity
                .get(end)
                .ok_or_else(|| "bond end identity was not constructed".to_owned())?;
            let order = match bond.order() {
                Some(BondOrder::Single) => 1,
                Some(BondOrder::Double) => 2,
                Some(BondOrder::Triple) => 3,
                Some(BondOrder::Aromatic) => 4,
                Some(BondOrder::Other(value)) => value,
                None => return Err("oracle capability requires bond order".to_owned()),
            };
            let source_type = bond
                .source_type()
                .ok_or_else(|| "oracle capability requires bond type".to_owned())?;
            Ok(json!({"start": start, "end": end, "order": order, "type": source_type}))
        })
        .collect();
    Ok(json!({
        "capability": request.capability,
        "atoms": atom_projection,
        "bonds": bond_projection?,
    }))
}

fn main() {
    let mut request_text = String::new();
    io::stdin()
        .read_to_string(&mut request_text)
        .expect("read oracle request");
    let request: Request = serde_json::from_str(&request_text).expect("parse oracle request");
    let mut projection = project(request).expect("construct ferrum core records");
    if env::args().any(|argument| argument == "--mutate-projection") {
        projection["bonds"][0]["order"] = json!(99);
    }
    let result = json!({
        "engine": "ferrum-core",
        "facts": {
            "crate_version": env!("CARGO_PKG_VERSION"),
            "rust_version_floor": env!("CARGO_PKG_RUST_VERSION"),
        },
        "projection": projection,
    });
    println!(
        "{}",
        serde_json::to_string(&result).expect("serialize oracle result")
    );
}
