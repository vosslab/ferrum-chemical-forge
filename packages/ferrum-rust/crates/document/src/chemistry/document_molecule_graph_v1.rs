//! Document façade over the shared capability-free molecule graph lowerer.

use ferrum_chemistry::{MolGraph, MolGraphError};
use ferrum_core::{Molecule, RecordId, VertexRef};
use ferrum_document_projection::{
    DirectMoleculeGraphAtomFact, DirectMoleculeGraphAtomInput, DirectMoleculeGraphBondFact,
    DirectMoleculeGraphEndpoint, DirectMoleculeGraphFacts, NonAtomVertexFact, NonAtomVertexKindV1,
};
use ferrum_graph_lowering::{
    MoleculeGraphLoweringError, MoleculeGraphLoweringWithAtomError,
    lower_direct_molecule_graph_with_validated_atom,
};
use std::collections::{HashMap, HashSet};
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
    pub fn into_parts_with_atom_records(self) -> (MolGraph, Vec<(usize, usize)>, Vec<RecordId>) {
        (self.graph, self.edges, self.graph_position_to_record_id)
    }
}
pub fn document_molecule_graph_v1(
    molecule: &Molecule,
) -> Result<DocumentMoleculeGraphV1, DocumentMoleculeGraphError> {
    lower(molecule, false)
}
pub fn document_molecule_coordinate_graph_v1(
    molecule: &Molecule,
) -> Result<DocumentMoleculeGraphV1, DocumentMoleculeGraphError> {
    lower(molecule, true)
}

fn lower(
    molecule: &Molecule,
    coordinates: bool,
) -> Result<DocumentMoleculeGraphV1, DocumentMoleculeGraphError> {
    let mut positions = HashMap::new();
    let atoms = molecule
        .atoms()
        .iter()
        .enumerate()
        .map(|(index, atom)| {
            let identity = atom.identity().clone();
            positions.entry(identity).or_insert(index);
            DirectMoleculeGraphAtomFact::new(DirectMoleculeGraphAtomInput {
                element: atom.element().map(str::to_owned),
                position: ferrum_document_projection::Point3V1::new(
                    atom.position().x(),
                    atom.position().y(),
                    atom.position().z(),
                )
                .expect("typed core atom coordinates are finite"),
                formal_charge: atom.formal_charge(),
                isotope: atom.isotope(),
                explicit_hydrogens: atom.explicit_hydrogens(),
                valence: atom.valence(),
                multiplicity: atom.multiplicity(),
                free_sites: atom.free_sites(),
            })
        })
        .collect::<Vec<_>>();
    let non_atoms = molecule
        .groups()
        .iter()
        .map(|_| NonAtomVertexFact::new(NonAtomVertexKindV1::CompactGroup, 0))
        .chain(
            molecule
                .texts()
                .iter()
                .map(|_| NonAtomVertexFact::new(NonAtomVertexKindV1::MoleculeText, 0)),
        )
        .chain(
            molecule
                .queries()
                .iter()
                .map(|_| NonAtomVertexFact::new(NonAtomVertexKindV1::Query, 0)),
        )
        .collect();
    let endpoint = |reference: &VertexRef| match reference {
        VertexRef::Atom(id) => positions
            .get(id)
            .copied()
            .map(DirectMoleculeGraphEndpoint::Atom)
            .unwrap_or(DirectMoleculeGraphEndpoint::Unknown),
        _ => DirectMoleculeGraphEndpoint::NonAtom,
    };
    let bonds = molecule
        .bonds()
        .iter()
        .map(|bond| {
            DirectMoleculeGraphBondFact::new(
                endpoint(bond.start()),
                endpoint(bond.end()),
                bond.order(),
                bond.style().cloned(),
            )
        })
        .collect();
    let facts = DirectMoleculeGraphFacts::new(atoms, bonds, non_atoms, coordinates);
    let mut accepted_identities = HashSet::new();
    let mut records = Vec::new();
    let (graph, edges) = lower_direct_molecule_graph_with_validated_atom(&facts, |atom_index| {
        let identity = molecule.atoms()[atom_index].identity().clone();
        if !accepted_identities.insert(identity.clone()) {
            return Err(DocumentMoleculeGraphError::DuplicateAtomIdentity { atom_index });
        }
        records.push(identity);
        Ok(())
    })
    .map_err(|error| match error {
        MoleculeGraphLoweringWithAtomError::Lowering(error) => map_error(error),
        MoleculeGraphLoweringWithAtomError::AtomCallback(error) => error,
    })?
    .into_parts();
    if graph.atoms().len() != records.len() {
        return Err(DocumentMoleculeGraphError::ResourceAllocation);
    }
    Ok(DocumentMoleculeGraphV1 {
        graph,
        edges,
        graph_position_to_record_id: records,
    })
}
fn map_error(error: MoleculeGraphLoweringError) -> DocumentMoleculeGraphError {
    match error {
        MoleculeGraphLoweringError::EmptyMolecule => DocumentMoleculeGraphError::EmptyMolecule,
        MoleculeGraphLoweringError::UnsupportedVertex { kind, count } => {
            DocumentMoleculeGraphError::UnsupportedVertex { kind, count }
        }
        MoleculeGraphLoweringError::MissingElement { atom_index } => {
            DocumentMoleculeGraphError::MissingElement { atom_index }
        }
        MoleculeGraphLoweringError::InvalidElement {
            atom_index,
            element,
            source,
        } => DocumentMoleculeGraphError::InvalidElement {
            atom_index,
            element,
            source,
        },
        MoleculeGraphLoweringError::UnsupportedAtomFact { atom_index, fact } => {
            DocumentMoleculeGraphError::UnsupportedAtomFact { atom_index, fact }
        }
        MoleculeGraphLoweringError::UnsupportedBondEndpoint { bond_index } => {
            DocumentMoleculeGraphError::UnsupportedBondEndpoint { bond_index }
        }
        MoleculeGraphLoweringError::UnsupportedBondStyle { bond_index } => {
            DocumentMoleculeGraphError::UnsupportedBondStyle { bond_index }
        }
        MoleculeGraphLoweringError::UnsupportedBondOrder { bond_index } => {
            DocumentMoleculeGraphError::UnsupportedBondOrder { bond_index }
        }
        MoleculeGraphLoweringError::Graph(error) => DocumentMoleculeGraphError::Graph(error),
        MoleculeGraphLoweringError::ResourceAllocation => {
            DocumentMoleculeGraphError::ResourceAllocation
        }
    }
}
#[derive(Debug, Error)]
pub enum DocumentMoleculeGraphError {
    #[error("native chemistry requires a molecule with at least one atom")]
    EmptyMolecule,
    #[error("native chemistry does not yet support {count} {kind} vertices")]
    UnsupportedVertex { kind: &'static str, count: usize },
    #[error("atom {atom_index} repeats an earlier durable identity")]
    DuplicateAtomIdentity { atom_index: usize },
    #[error("atom {atom_index} has no element for native chemistry")]
    MissingElement { atom_index: usize },
    #[error("atom {atom_index} element {element:?} is not supported: {source}")]
    InvalidElement {
        atom_index: usize,
        element: String,
        #[source]
        source: MolGraphError,
    },
    #[error("atom {atom_index} has unsupported {fact}")]
    UnsupportedAtomFact {
        atom_index: usize,
        fact: &'static str,
    },
    #[error("bond {bond_index} has a non-atom or unresolved endpoint")]
    UnsupportedBondEndpoint { bond_index: usize },
    #[error("bond {bond_index} has a drawing style not supported by native chemistry")]
    UnsupportedBondStyle { bond_index: usize },
    #[error("bond {bond_index} has an unsupported or absent bond order")]
    UnsupportedBondOrder { bond_index: usize },
    #[error(transparent)]
    Graph(#[from] MolGraphError),
    #[error("native chemistry graph could not reserve owned storage")]
    ResourceAllocation,
}
