//! Emit the corpus-comparison shape through the authoritative typed document reader.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::process;

use ferrum_core::{Atom, Bond, BondOrder, BondStyle, Molecule, NonAtomVertex, RecordId, VertexRef};
use ferrum_document::TypedDocument;
use serde_json::{Value, json};

fn main() {
    let mut arguments = env::args().skip(1);
    let Some(corpus_path) = arguments.next() else {
        eprintln!("usage: corpus_projection <corpus.cdml>");
        process::exit(2);
    };
    let source = match fs::read_to_string(&corpus_path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("cannot read {corpus_path}: {error}");
            process::exit(2);
        }
    };
    match project_corpus(&source, &corpus_path) {
        Ok(result) => println!(
            "{}",
            serde_json::to_string(&result).expect("serialize corpus projection")
        ),
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

fn project_corpus(source: &str, corpus_path: &str) -> Result<Value, String> {
    let document = TypedDocument::parse(source).map_err(|error| error.to_string())?;
    let projection = document
        .core_projection()
        .map_err(|error| error.to_string())?;
    let molecules = projection
        .molecules()
        .iter()
        .enumerate()
        .map(|(index, molecule)| project_molecule(molecule, index))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({
        "engine": "ferrum-core",
        "facts": {
            "crate_version": env!("CARGO_PKG_VERSION"),
            "rust_version_floor": env!("CARGO_PKG_RUST_VERSION"),
        },
        "projection": {
            "capability": "corpus-molecule-core",
            "corpus_path": corpus_path,
            "coordinate_unit": "postscript_point",
            "document_version": projection.document_version(),
            "molecules": molecules,
        },
    }))
}

fn project_molecule(molecule: &Molecule, molecule_index: usize) -> Result<Value, String> {
    let atoms = molecule
        .atoms()
        .iter()
        .enumerate()
        .map(|(index, atom)| project_atom(atom, index))
        .collect::<Vec<_>>();
    let mut vertex_positions = HashMap::new();
    collect_vertex_positions(&mut vertex_positions, molecule.groups(), "group");
    collect_vertex_positions(&mut vertex_positions, molecule.texts(), "text");
    collect_vertex_positions(&mut vertex_positions, molecule.queries(), "query");
    for (index, atom) in molecule.atoms().iter().enumerate() {
        vertex_positions.insert(atom.identity(), ("atom", index));
    }
    let bonds = molecule
        .bonds()
        .iter()
        .map(|bond| project_bond(bond, &vertex_positions))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({
        "index": molecule_index,
        "id": molecule.source_id().as_str(),
        "name": molecule.name(),
        "atoms": atoms,
        "bonds": bonds,
        "groups": project_non_atom_vertices(molecule.groups()),
        "texts": project_non_atom_vertices(molecule.texts()),
        "queries": project_non_atom_vertices(molecule.queries()),
    }))
}

fn project_atom(atom: &Atom, atom_index: usize) -> Value {
    let position = atom.position();
    json!({
        "index": atom_index,
        "id": atom.source_id().as_str(),
        "symbol": atom.element(),
        "element": atom.element(),
        "formal_charge": atom.formal_charge(),
        "explicit_hydrogens": atom.explicit_hydrogens(),
        "isotope": atom.isotope(),
        "valence": atom.valence(),
        "multiplicity": atom.multiplicity(),
        "free_sites": atom.free_sites(),
        "x": round_coordinate(position.x()),
        "y": round_coordinate(position.y()),
        "z": round_coordinate(position.z()),
    })
}

fn project_bond(
    bond: &Bond,
    vertex_positions: &HashMap<&RecordId, (&str, usize)>,
) -> Result<Value, String> {
    let (start_kind, start_index) = endpoint_position(bond.start(), vertex_positions)?;
    let (end_kind, end_index) = endpoint_position(bond.end(), vertex_positions)?;
    let order = bond.order().map(|order| match order {
        BondOrder::Single => 1,
        BondOrder::Double => 2,
        BondOrder::Triple => 3,
        BondOrder::Aromatic => 4,
        BondOrder::Other(value) => value,
    });
    Ok(json!({
        "id": bond.source_id().as_str(),
        "start": start_index,
        "end": end_index,
        "start_kind": start_kind,
        "end_kind": end_kind,
        "type": bond.source_type(),
        "order": order,
        "style": bond.style().map(bond_style_name),
        "aromatic": bond.aromatic(),
    }))
}

fn endpoint_position<'a>(
    endpoint: &VertexRef,
    vertex_positions: &HashMap<&RecordId, (&'a str, usize)>,
) -> Result<(&'a str, usize), String> {
    let identity = match endpoint {
        VertexRef::Atom(identity)
        | VertexRef::Group(identity)
        | VertexRef::Text(identity)
        | VertexRef::Query(identity) => identity,
    };
    vertex_positions
        .get(identity)
        .copied()
        .ok_or_else(|| "bond endpoint resolves to no projected vertex".to_owned())
}

fn collect_vertex_positions<'a>(
    positions: &mut HashMap<&'a RecordId, (&'static str, usize)>,
    vertices: &'a [NonAtomVertex],
    kind: &'static str,
) {
    for (index, vertex) in vertices.iter().enumerate() {
        positions.insert(vertex.identity(), (kind, index));
    }
}

fn project_non_atom_vertices(vertices: &[NonAtomVertex]) -> Vec<Value> {
    vertices
        .iter()
        .enumerate()
        .map(|(index, vertex)| json!({"index": index, "id": vertex.source_id().as_str()}))
        .collect()
}

fn bond_style_name(style: &BondStyle) -> String {
    match style {
        BondStyle::Normal => "normal".to_owned(),
        BondStyle::Wedge => "wedge".to_owned(),
        BondStyle::Hashed => "hashed".to_owned(),
        BondStyle::Adder => "adder".to_owned(),
        BondStyle::Bold => "bold".to_owned(),
        BondStyle::Dashed => "dashed".to_owned(),
        BondStyle::Dotted => "dotted".to_owned(),
        BondStyle::Wavy => "wavy".to_owned(),
        BondStyle::HaworthFront => "haworth_front".to_owned(),
        BondStyle::Other(value) => value.clone(),
    }
}

fn round_coordinate(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
