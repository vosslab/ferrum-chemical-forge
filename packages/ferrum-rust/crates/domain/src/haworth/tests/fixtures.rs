use ferrum_core::{Atom, Bond, BondOrder, Identifier, Molecule, Position, VertexRef};

use crate::haworth::{HaworthLayoutRequest, HaworthTopologyBuilder, HaworthVertex, RingForm};

fn atom(index: usize, element: &str) -> Atom {
    Atom::new(
        Some(Identifier::new(format!("a{index}")).expect("identifier")),
        Some(element.to_owned()),
        Position::new(index as f64, 0.0, 0.0).expect("position"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("atom")
}

fn bond(index: usize, start: &Atom, end: &Atom) -> Bond {
    Bond::new(
        Some(Identifier::new(format!("b{index}")).expect("identifier")),
        VertexRef::Atom(start.identity().clone()),
        VertexRef::Atom(end.identity().clone()),
        None,
        Some(BondOrder::Single),
        None,
        Some(false),
        None,
    )
    .expect("bond")
}

pub(super) fn molecule(
    form: RingForm,
    reordered_storage: bool,
    extra_elements: &[&str],
    extra_edges: &[(usize, usize)],
) -> (Molecule, Vec<HaworthVertex>) {
    let elements = match form {
        RingForm::Pyranose => ["O", "C", "C", "C", "C", "C"].as_slice(),
        RingForm::Furanose => ["O", "C", "C", "C", "C"].as_slice(),
    };
    let mut atoms: Vec<_> = elements
        .iter()
        .chain(extra_elements.iter())
        .enumerate()
        .map(|(index, element)| atom(index, element))
        .collect();
    let vertices: Vec<_> = atoms[..elements.len()]
        .iter()
        .map(|atom| HaworthVertex {
            atom: atom.identity().clone(),
        })
        .collect();
    let mut bonds: Vec<_> = (0..elements.len())
        .map(|index| bond(index, &atoms[index], &atoms[(index + 1) % elements.len()]))
        .collect();
    bonds.extend(
        extra_edges
            .iter()
            .enumerate()
            .map(|(index, &(start, end))| bond(elements.len() + index, &atoms[start], &atoms[end])),
    );
    if reordered_storage {
        atoms.reverse();
    }
    (
        Molecule::new(
            Some(Identifier::new("molecule").expect("identifier")),
            None,
            atoms,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            bonds,
            None,
        )
        .expect("molecule"),
        vertices,
    )
}

pub(super) fn request(
    form: RingForm,
    scale: f64,
    reordered_storage: bool,
    reverse_cycle: bool,
) -> HaworthLayoutRequest {
    let (molecule, mut vertices) = molecule(form, reordered_storage, &[], &[]);
    let anomeric_atom = vertices[1].atom.clone();
    if reverse_cycle {
        vertices.reverse();
    }
    HaworthLayoutRequest {
        topology: HaworthTopologyBuilder::new(form, anomeric_atom, vertices)
            .build(&molecule)
            .expect("topology"),
        scale,
    }
}
