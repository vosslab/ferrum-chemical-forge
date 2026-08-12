//! Validation and canonicalization of caller-selected Haworth ring cycles.

use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};

use ferrum_core::{BondOrder, Molecule, RecordId, RecordKind, VertexRef};

use crate::haworth::{HaworthError, HaworthTopology, HaworthVertex, RingForm};

pub(crate) fn validate_topology(
    ring_form: RingForm,
    anomeric_atom: RecordId,
    selected_cycle: Vec<HaworthVertex>,
    molecule: &Molecule,
) -> Result<HaworthTopology, HaworthError> {
    let vertices = canonicalize_vertices(ring_form, anomeric_atom, selected_cycle, molecule)?;
    let mut bond_ids = Vec::with_capacity(vertices.len());
    for index in 0..vertices.len() {
        let start = &vertices[index].atom;
        let end = &vertices[(index + 1) % vertices.len()].atom;
        let matching: Vec<_> = molecule
            .bonds()
            .iter()
            .filter(|bond| endpoints_match(bond.start(), bond.end(), start, end))
            .collect();
        if matching.len() != 1 {
            return Err(HaworthError::UnsupportedTopology(
                "cycle edges must map to one bond",
            ));
        }
        let bond = matching[0];
        if bond.order() != Some(BondOrder::Single) || bond.aromatic() != Some(false) {
            return Err(HaworthError::UnsupportedTopology(
                "cycle bonds must be non-aromatic single bonds",
            ));
        }
        bond_ids.push(bond.identity().clone());
    }
    ensure_isolated_cycle(&vertices, molecule)?;
    Ok(HaworthTopology::from_validated(
        ring_form, vertices, bond_ids,
    ))
}

fn canonicalize_vertices(
    ring_form: RingForm,
    anomeric_atom: RecordId,
    mut vertices: Vec<HaworthVertex>,
    molecule: &Molecule,
) -> Result<Vec<HaworthVertex>, HaworthError> {
    if vertices.len() != ring_form.vertex_count() {
        return Err(HaworthError::UnsupportedTopology(
            "ring size does not match form",
        ));
    }
    let distinct: HashSet<_> = vertices.iter().map(|vertex| &vertex.atom).collect();
    if distinct.len() != vertices.len() {
        return Err(HaworthError::UnsupportedTopology(
            "ring vertices must be distinct",
        ));
    }
    if vertices
        .iter()
        .any(|vertex| vertex.atom.kind() != RecordKind::Atom)
        || anomeric_atom.kind() != RecordKind::Atom
    {
        return Err(HaworthError::UnsupportedTopology(
            "ring vertices and anomeric atom must be atoms",
        ));
    }
    let oxygen_index = vertices
        .iter()
        .position(|vertex| atom_element(molecule, &vertex.atom) == Ok("O"))
        .ok_or(HaworthError::UnsupportedTopology(
            "ring must contain exactly one oxygen",
        ))?;
    if vertices
        .iter()
        .filter(|vertex| atom_element(molecule, &vertex.atom) == Ok("O"))
        .count()
        != 1
    {
        return Err(HaworthError::UnsupportedTopology(
            "ring must contain exactly one oxygen",
        ));
    }
    for vertex in &vertices {
        match atom_element(molecule, &vertex.atom)? {
            "O" | "C" => {}
            _ => {
                return Err(HaworthError::UnsupportedTopology(
                    "ring atoms must be carbon or oxygen",
                ));
            }
        }
    }
    if atom_element(molecule, &anomeric_atom)? != "C" {
        return Err(HaworthError::InvalidSpec("anomeric atom must be carbon"));
    }
    vertices.rotate_left(oxygen_index);
    let anomeric_index = vertices
        .iter()
        .position(|vertex| vertex.atom == anomeric_atom)
        .ok_or(HaworthError::InvalidSpec(
            "anomeric atom must belong to selected cycle",
        ))?;
    if anomeric_index == 1 {
        let oxygen = vertices.remove(0);
        vertices.reverse();
        vertices.insert(0, oxygen);
    } else if anomeric_index != vertices.len() - 1 {
        return Err(HaworthError::InvalidSpec(
            "anomeric atom must be adjacent to ring oxygen",
        ));
    }
    Ok(vertices)
}

fn atom_element<'a>(molecule: &'a Molecule, id: &RecordId) -> Result<&'a str, HaworthError> {
    molecule
        .atoms()
        .iter()
        .find(|atom| atom.identity() == id)
        .ok_or(HaworthError::StaleTopology("selected atom is absent"))?
        .element()
        .ok_or(HaworthError::UnsupportedTopology(
            "ring atoms must declare elements",
        ))
}

fn endpoints_match(
    first: &VertexRef,
    second: &VertexRef,
    start: &RecordId,
    end: &RecordId,
) -> bool {
    let start_ref = VertexRef::Atom(start.clone());
    let end_ref = VertexRef::Atom(end.clone());
    (first == &start_ref && second == &end_ref) || (first == &end_ref && second == &start_ref)
}

fn ensure_isolated_cycle(
    vertices: &[HaworthVertex],
    molecule: &Molecule,
) -> Result<(), HaworthError> {
    let selected: BTreeSet<_> = vertices.iter().map(|vertex| vertex.atom.clone()).collect();
    let cycle_edges: BTreeSet<_> = vertices
        .iter()
        .enumerate()
        .map(|(index, vertex)| {
            canonical_edge(&vertex.atom, &vertices[(index + 1) % vertices.len()].atom)
        })
        .collect();
    let adjacency = non_cycle_atom_adjacency(molecule, &cycle_edges);

    for selected_vertex in &selected {
        if reaches_other_selected(selected_vertex, &selected, &adjacency) {
            return Err(HaworthError::UnsupportedTopology(
                "initial profile requires an isolated chordless single cycle",
            ));
        }
        if lies_on_external_cycle(selected_vertex, &adjacency) {
            return Err(HaworthError::UnsupportedTopology(
                "initial profile excludes fused and spiro ring reuse",
            ));
        }
    }
    Ok(())
}

fn non_cycle_atom_adjacency(
    molecule: &Molecule,
    cycle_edges: &BTreeSet<(RecordId, RecordId)>,
) -> BTreeMap<RecordId, Vec<RecordId>> {
    let mut adjacency = BTreeMap::<RecordId, Vec<RecordId>>::new();
    for bond in molecule.bonds() {
        let (VertexRef::Atom(start), VertexRef::Atom(end)) = (bond.start(), bond.end()) else {
            continue;
        };
        if cycle_edges.contains(&canonical_edge(start, end)) {
            continue;
        }
        adjacency
            .entry(start.clone())
            .or_default()
            .push(end.clone());
        adjacency
            .entry(end.clone())
            .or_default()
            .push(start.clone());
    }
    adjacency
}

fn reaches_other_selected(
    start: &RecordId,
    selected: &BTreeSet<RecordId>,
    adjacency: &BTreeMap<RecordId, Vec<RecordId>>,
) -> bool {
    let mut visited = BTreeSet::from([start.clone()]);
    let mut queue = VecDeque::from([start.clone()]);
    while let Some(current) = queue.pop_front() {
        for neighbor in adjacency.get(&current).into_iter().flatten() {
            if selected.contains(neighbor) && neighbor != start {
                return true;
            }
            if visited.insert(neighbor.clone()) {
                queue.push_back(neighbor.clone());
            }
        }
    }
    false
}

fn lies_on_external_cycle(
    vertex: &RecordId,
    adjacency: &BTreeMap<RecordId, Vec<RecordId>>,
) -> bool {
    let Some(neighbors) = adjacency.get(vertex) else {
        return false;
    };
    for (index, start) in neighbors.iter().enumerate() {
        if reaches_any_without(vertex, start, &neighbors[index + 1..], adjacency) {
            return true;
        }
    }
    false
}

fn reaches_any_without(
    excluded: &RecordId,
    start: &RecordId,
    targets: &[RecordId],
    adjacency: &BTreeMap<RecordId, Vec<RecordId>>,
) -> bool {
    let targets: BTreeSet<_> = targets.iter().collect();
    let mut visited = BTreeSet::from([excluded.clone(), start.clone()]);
    let mut queue = VecDeque::from([start.clone()]);
    while let Some(current) = queue.pop_front() {
        for neighbor in adjacency.get(&current).into_iter().flatten() {
            if targets.contains(neighbor) {
                return true;
            }
            if visited.insert(neighbor.clone()) {
                queue.push_back(neighbor.clone());
            }
        }
    }
    false
}

fn canonical_edge(first: &RecordId, second: &RecordId) -> (RecordId, RecordId) {
    if first <= second {
        (first.clone(), second.clone())
    } else {
        (second.clone(), first.clone())
    }
}
