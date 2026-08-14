//! Neutral conversion from a typed document molecule to the native graph contract.

use std::collections::HashMap;

use ferrum_chemistry::{
    AtomicNumber, BondOrder as ChemistryBondOrder, MolAtom, MolBond, MolGraph, MolGraphError,
};
use ferrum_core::{BondOrder, BondStyle, Molecule, RecordId, VertexRef};
use thiserror::Error;

pub(crate) struct DocumentMoleculeGraphV1 {
    graph: MolGraph,
    edges: Vec<(usize, usize)>,
}

impl DocumentMoleculeGraphV1 {
    pub(crate) fn into_parts(self) -> (MolGraph, Vec<(usize, usize)>) {
        (self.graph, self.edges)
    }
}

pub(crate) fn document_molecule_graph_v1(
    molecule: &Molecule,
) -> Result<DocumentMoleculeGraphV1, DocumentMoleculeGraphError> {
    if molecule.atoms().is_empty() {
        return Err(DocumentMoleculeGraphError::EmptyMolecule);
    }
    for (kind, count) in [
        ("group", molecule.groups().len()),
        ("molecule text", molecule.texts().len()),
        ("query", molecule.queries().len()),
    ] {
        if count != 0 {
            return Err(DocumentMoleculeGraphError::UnsupportedVertex { kind, count });
        }
    }
    let atoms = molecule
        .atoms()
        .iter()
        .enumerate()
        .map(|(index, atom)| {
            if atom.valence().is_some() {
                return Err(DocumentMoleculeGraphError::UnsupportedAtomFact {
                    atom_index: index,
                    fact: "authored valence",
                });
            }
            if atom.multiplicity().is_some() {
                return Err(DocumentMoleculeGraphError::UnsupportedAtomFact {
                    atom_index: index,
                    fact: "authored multiplicity",
                });
            }
            if atom.free_sites().is_some() {
                return Err(DocumentMoleculeGraphError::UnsupportedAtomFact {
                    atom_index: index,
                    fact: "authored free sites",
                });
            }
            let element = atom
                .element()
                .ok_or(DocumentMoleculeGraphError::MissingElement { atom_index: index })?;
            let atomic_number = AtomicNumber::from_symbol(element).map_err(|source| {
                DocumentMoleculeGraphError::InvalidElement {
                    atom_index: index,
                    element: element.to_owned(),
                    source,
                }
            })?;
            MolAtom::new(
                atomic_number,
                atom.formal_charge(),
                atom.isotope(),
                atom.explicit_hydrogens(),
                false,
            )
            .map_err(DocumentMoleculeGraphError::Graph)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let atom_indices = molecule
        .atoms()
        .iter()
        .enumerate()
        .map(|(index, atom)| (atom.identity().clone(), index))
        .collect::<HashMap<RecordId, usize>>();
    let mut bonds = Vec::with_capacity(molecule.bonds().len());
    let mut edges = Vec::with_capacity(molecule.bonds().len());
    for (bond_index, bond) in molecule.bonds().iter().enumerate() {
        if bond
            .style()
            .is_some_and(|style| style != &BondStyle::Normal)
        {
            return Err(DocumentMoleculeGraphError::UnsupportedBondStyle { bond_index });
        }
        let start = atom_endpoint(bond.start(), &atom_indices, bond_index)?;
        let end = atom_endpoint(bond.end(), &atom_indices, bond_index)?;
        let order = match bond.order() {
            Some(BondOrder::Single) => ChemistryBondOrder::Single,
            Some(BondOrder::Double) => ChemistryBondOrder::Double,
            Some(BondOrder::Triple) => ChemistryBondOrder::Triple,
            Some(BondOrder::Aromatic | BondOrder::Other(_)) | None => {
                return Err(DocumentMoleculeGraphError::UnsupportedBondOrder { bond_index });
            }
        };
        bonds.push(MolBond::new(start, end, order, false));
        edges.push((start, end));
    }
    Ok(DocumentMoleculeGraphV1 {
        graph: MolGraph::new(atoms, bonds, None)?,
        edges,
    })
}

fn atom_endpoint(
    endpoint: &VertexRef,
    atom_indices: &HashMap<RecordId, usize>,
    bond_index: usize,
) -> Result<usize, DocumentMoleculeGraphError> {
    let VertexRef::Atom(identifier) = endpoint else {
        return Err(DocumentMoleculeGraphError::UnsupportedBondEndpoint { bond_index });
    };
    atom_indices
        .get(identifier)
        .copied()
        .ok_or(DocumentMoleculeGraphError::UnsupportedBondEndpoint { bond_index })
}

/// Failure while converting a typed document molecule to a complete native graph.
#[derive(Debug, Error)]
pub enum DocumentMoleculeGraphError {
    /// A native chemistry graph requires at least one atom.
    #[error("native chemistry requires a molecule with at least one atom")]
    EmptyMolecule,
    /// The chemistry graph cannot silently discard a typed non-atom vertex.
    #[error("native chemistry does not yet support {count} {kind} vertices")]
    UnsupportedVertex { kind: &'static str, count: usize },
    /// An atom omitted its required chemical element.
    #[error("atom {atom_index} has no element for native chemistry")]
    MissingElement { atom_index: usize },
    /// An element spelling is outside the native engine's exact element domain.
    #[error("atom {atom_index} element {element:?} is not supported: {source}")]
    InvalidElement {
        atom_index: usize,
        element: String,
        #[source]
        source: MolGraphError,
    },
    /// An authored atom fact has no exact native graph mapping yet.
    #[error("atom {atom_index} has unsupported {fact}")]
    UnsupportedAtomFact {
        atom_index: usize,
        fact: &'static str,
    },
    /// A bond endpoint is not an ordinary atom in the selected molecule.
    #[error("bond {bond_index} has a non-atom or unresolved endpoint")]
    UnsupportedBondEndpoint { bond_index: usize },
    /// A drawing-specific bond style cannot cross a chemistry-only boundary.
    #[error("bond {bond_index} has a drawing style not supported by native chemistry")]
    UnsupportedBondStyle { bond_index: usize },
    /// A bond order has no exact native graph mapping.
    #[error("bond {bond_index} has an unsupported or absent bond order")]
    UnsupportedBondOrder { bond_index: usize },
    /// The selected graph facts violated chemistry invariants.
    #[error(transparent)]
    Graph(#[from] MolGraphError),
}
