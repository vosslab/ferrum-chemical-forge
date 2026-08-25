//! Document-owned conversion from a typed molecule to the native graph contract.

use std::collections::HashMap;

use ferrum_chemistry::{
    AtomicNumber, BondOrder as ChemistryBondOrder, Coordinates, MolAtom, MolBond, MolGraph,
    MolGraphError, Point2,
};
use ferrum_core::{BondOrder, BondStyle, Molecule, RecordId, VertexRef};
use thiserror::Error;

pub struct DocumentMoleculeGraphV1 {
    graph: MolGraph,
    edges: Vec<(usize, usize)>,
    graph_position_to_record_id: Vec<RecordId>,
}

impl DocumentMoleculeGraphV1 {
    pub fn into_parts(self) -> (MolGraph, Vec<(usize, usize)>) {
        (self.graph, self.edges)
    }

    /// Consume this lowering with the exact atom identity that created each
    /// graph position. This is not a later source-order reconstruction.
    pub fn into_parts_with_atom_records(self) -> (MolGraph, Vec<(usize, usize)>, Vec<RecordId>) {
        (self.graph, self.edges, self.graph_position_to_record_id)
    }
}

pub fn document_molecule_graph_v1(
    molecule: &Molecule,
) -> Result<DocumentMoleculeGraphV1, DocumentMoleculeGraphError> {
    document_molecule_graph(molecule, false)
}

/// Convert a document molecule while retaining its exact atom coordinates.
///
/// CDML and Qt use a downward-positive y axis. The chemistry ABI and molfile
/// writer use an upward-positive y axis, so this boundary performs the inverse
/// of Ferrum's chemistry-to-document placement transform.
pub fn document_molecule_coordinate_graph_v1(
    molecule: &Molecule,
) -> Result<DocumentMoleculeGraphV1, DocumentMoleculeGraphError> {
    document_molecule_graph(molecule, true)
}

fn document_molecule_graph(
    molecule: &Molecule,
    include_coordinates: bool,
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
    let mut atoms = Vec::new();
    atoms
        .try_reserve_exact(molecule.atoms().len())
        .map_err(|_| DocumentMoleculeGraphError::ResourceAllocation)?;
    let mut atom_indices = HashMap::new();
    atom_indices
        .try_reserve(molecule.atoms().len())
        .map_err(|_| DocumentMoleculeGraphError::ResourceAllocation)?;
    let mut graph_position_to_record_id = Vec::new();
    graph_position_to_record_id
        .try_reserve_exact(molecule.atoms().len())
        .map_err(|_| DocumentMoleculeGraphError::ResourceAllocation)?;
    let mut points = Vec::new();
    if include_coordinates {
        points
            .try_reserve_exact(molecule.atoms().len())
            .map_err(|_| DocumentMoleculeGraphError::ResourceAllocation)?;
    }
    for (index, atom) in molecule.atoms().iter().enumerate() {
        for (fact, present) in [
            ("authored valence", atom.valence().is_some()),
            ("authored multiplicity", atom.multiplicity().is_some()),
            ("authored free sites", atom.free_sites().is_some()),
        ] {
            if present {
                return Err(DocumentMoleculeGraphError::UnsupportedAtomFact {
                    atom_index: index,
                    fact,
                });
            }
        }
        let element = atom
            .element()
            .ok_or(DocumentMoleculeGraphError::MissingElement { atom_index: index })?;
        let atomic_number = match AtomicNumber::from_symbol(element) {
            Ok(atomic_number) => atomic_number,
            Err(source) => {
                return Err(DocumentMoleculeGraphError::InvalidElement {
                    atom_index: index,
                    element: copied(element)?,
                    source,
                });
            }
        };
        atoms.push(
            MolAtom::new(
                atomic_number,
                atom.formal_charge(),
                atom.isotope(),
                atom.explicit_hydrogens(),
                false,
            )
            .map_err(DocumentMoleculeGraphError::Graph)?,
        );
        let identity = atom.identity().clone();
        if atom_indices.insert(identity.clone(), index).is_some() {
            return Err(DocumentMoleculeGraphError::DuplicateAtomIdentity { atom_index: index });
        }
        graph_position_to_record_id.push(identity);
        if include_coordinates {
            points.push(Point2::new(atom.position().x(), -atom.position().y())?);
        }
    }
    let mut bonds = Vec::new();
    bonds
        .try_reserve_exact(molecule.bonds().len())
        .map_err(|_| DocumentMoleculeGraphError::ResourceAllocation)?;
    let mut edges = Vec::new();
    edges
        .try_reserve_exact(molecule.bonds().len())
        .map_err(|_| DocumentMoleculeGraphError::ResourceAllocation)?;
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
    let coordinates = include_coordinates.then(|| Coordinates::new(points));
    let graph = MolGraph::new(atoms, bonds, coordinates)?;
    if graph_position_to_record_id.len() != graph.atoms().len() {
        return Err(DocumentMoleculeGraphError::ResourceAllocation);
    }
    Ok(DocumentMoleculeGraphV1 {
        graph,
        edges,
        graph_position_to_record_id,
    })
}

fn copied(value: &str) -> Result<String, DocumentMoleculeGraphError> {
    let mut owned = String::new();
    owned
        .try_reserve_exact(value.len())
        .map_err(|_| DocumentMoleculeGraphError::ResourceAllocation)?;
    owned.push_str(value);
    Ok(owned)
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
    /// The retained core graph unexpectedly repeated an atom identity.
    #[error("atom {atom_index} repeats an earlier durable identity")]
    DuplicateAtomIdentity { atom_index: usize },
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
    /// Exact owned conversion storage could not be allocated.
    #[error("native chemistry graph could not reserve owned storage")]
    ResourceAllocation,
}
