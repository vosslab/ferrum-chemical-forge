//! Exact document-to-chemistry conversion for molecule composition.

use std::collections::HashMap;

use ferrum_chemistry::{
    AtomicNumber, BondOrder as ChemistryBondOrder, MolAtom, MolBond, MolGraph, MolGraphError,
};
use ferrum_core::{BondOrder, BondStyle, Molecule, RecordId, VertexRef};
use thiserror::Error;

pub(crate) fn document_molecule_composition_graph_v1(
    molecule: &Molecule,
) -> Result<MolGraph, DocumentMoleculeCompositionGraphErrorV1> {
    if molecule.atoms().is_empty() {
        return Err(DocumentMoleculeCompositionGraphErrorV1::EmptyMolecule);
    }
    for (kind, count) in [
        ("group", molecule.groups().len()),
        ("molecule text", molecule.texts().len()),
        ("query", molecule.queries().len()),
    ] {
        if count != 0 {
            return Err(DocumentMoleculeCompositionGraphErrorV1::UnsupportedVertex { kind, count });
        }
    }

    let mut atom_indices = HashMap::new();
    atom_indices
        .try_reserve(molecule.atoms().len())
        .map_err(|_| DocumentMoleculeCompositionGraphErrorV1::ResourceAllocation)?;
    for (index, atom) in molecule.atoms().iter().enumerate() {
        let identity = atom
            .identity()
            .try_clone()
            .map_err(|_| DocumentMoleculeCompositionGraphErrorV1::ResourceAllocation)?;
        if atom_indices.insert(identity, index).is_some() {
            return Err(
                DocumentMoleculeCompositionGraphErrorV1::DuplicateAtomIdentity {
                    atom_index: index,
                },
            );
        }
    }

    let mut bonds = Vec::new();
    bonds
        .try_reserve_exact(molecule.bonds().len())
        .map_err(|_| DocumentMoleculeCompositionGraphErrorV1::ResourceAllocation)?;
    let mut aromatic_atoms = Vec::new();
    aromatic_atoms
        .try_reserve_exact(molecule.atoms().len())
        .map_err(|_| DocumentMoleculeCompositionGraphErrorV1::ResourceAllocation)?;
    aromatic_atoms.resize(molecule.atoms().len(), false);
    for (bond_index, bond) in molecule.bonds().iter().enumerate() {
        match bond.style() {
            Some(
                BondStyle::Normal
                | BondStyle::Wedge
                | BondStyle::Hashed
                | BondStyle::Adder
                | BondStyle::Bold
                | BondStyle::Dashed
                | BondStyle::Dotted
                | BondStyle::Wavy
                | BondStyle::HaworthFront,
            ) => {}
            Some(BondStyle::Other(_)) | None => {
                return Err(
                    DocumentMoleculeCompositionGraphErrorV1::UnsupportedBondStyle { bond_index },
                );
            }
        }
        let start = atom_endpoint(bond.start(), &atom_indices, bond_index)?;
        let end = atom_endpoint(bond.end(), &atom_indices, bond_index)?;
        let (order, aromatic) = match (bond.order(), bond.aromatic()) {
            (Some(BondOrder::Aromatic), Some(true)) => (ChemistryBondOrder::Aromatic, true),
            (Some(BondOrder::Single), None | Some(false)) => (ChemistryBondOrder::Single, false),
            (Some(BondOrder::Double), None | Some(false)) => (ChemistryBondOrder::Double, false),
            (Some(BondOrder::Triple), None | Some(false)) => (ChemistryBondOrder::Triple, false),
            (Some(BondOrder::Other(_)) | None, _) => {
                return Err(
                    DocumentMoleculeCompositionGraphErrorV1::UnsupportedBondOrder { bond_index },
                );
            }
            _ => {
                return Err(
                    DocumentMoleculeCompositionGraphErrorV1::InconsistentAromaticity { bond_index },
                );
            }
        };
        if aromatic {
            aromatic_atoms[start] = true;
            aromatic_atoms[end] = true;
        }
        bonds.push(MolBond::new(start, end, order, aromatic));
    }

    let mut atoms = Vec::new();
    atoms
        .try_reserve_exact(molecule.atoms().len())
        .map_err(|_| DocumentMoleculeCompositionGraphErrorV1::ResourceAllocation)?;
    for (atom_index, atom) in molecule.atoms().iter().enumerate() {
        for (fact, present) in [
            ("authored valence", atom.valence().is_some()),
            ("authored multiplicity", atom.multiplicity().is_some()),
            ("authored free sites", atom.free_sites().is_some()),
        ] {
            if present {
                return Err(
                    DocumentMoleculeCompositionGraphErrorV1::UnsupportedAtomFact {
                        atom_index,
                        fact,
                    },
                );
            }
        }
        let element = atom
            .element()
            .ok_or(DocumentMoleculeCompositionGraphErrorV1::MissingElement { atom_index })?;
        let atomic_number = AtomicNumber::from_symbol(element).map_err(|source| {
            DocumentMoleculeCompositionGraphErrorV1::InvalidElement { atom_index, source }
        })?;
        atoms.push(
            MolAtom::new(
                atomic_number,
                atom.formal_charge(),
                atom.isotope(),
                atom.explicit_hydrogens(),
                aromatic_atoms[atom_index],
            )
            .map_err(DocumentMoleculeCompositionGraphErrorV1::Graph)?,
        );
    }

    MolGraph::new(atoms, bonds, None).map_err(DocumentMoleculeCompositionGraphErrorV1::Graph)
}

fn atom_endpoint(
    endpoint: &VertexRef,
    atom_indices: &HashMap<RecordId, usize>,
    bond_index: usize,
) -> Result<usize, DocumentMoleculeCompositionGraphErrorV1> {
    let VertexRef::Atom(identifier) = endpoint else {
        return Err(
            DocumentMoleculeCompositionGraphErrorV1::UnsupportedBondEndpoint { bond_index },
        );
    };
    atom_indices
        .get(identifier)
        .copied()
        .ok_or(DocumentMoleculeCompositionGraphErrorV1::UnsupportedBondEndpoint { bond_index })
}

/// Failure while converting one exact document molecule for composition.
#[derive(Debug, Error)]
pub enum DocumentMoleculeCompositionGraphErrorV1 {
    /// Composition requires at least one physical atom.
    #[error("molecule composition requires at least one atom")]
    EmptyMolecule,
    /// A typed non-atom vertex cannot be silently discarded.
    #[error("molecule composition does not support {count} {kind} vertices")]
    UnsupportedVertex { kind: &'static str, count: usize },
    /// The retained core graph unexpectedly repeated an atom identity.
    #[error("atom {atom_index} repeats an earlier durable identity")]
    DuplicateAtomIdentity { atom_index: usize },
    /// An atom omitted its required chemical element.
    #[error("atom {atom_index} has no element for molecule composition")]
    MissingElement { atom_index: usize },
    /// An element spelling is outside the native engine's exact element domain.
    #[error("atom {atom_index} has an unsupported element: {source}")]
    InvalidElement {
        atom_index: usize,
        #[source]
        source: MolGraphError,
    },
    /// An authored atom fact has no exact native graph representation.
    #[error("atom {atom_index} has unsupported {fact}")]
    UnsupportedAtomFact {
        atom_index: usize,
        fact: &'static str,
    },
    /// A bond endpoint is not an ordinary atom in this molecule.
    #[error("bond {bond_index} has a non-atom or unresolved endpoint")]
    UnsupportedBondEndpoint { bond_index: usize },
    /// A bond style is absent or outside the closed composition whitelist.
    #[error("bond {bond_index} has an unsupported or absent drawing style")]
    UnsupportedBondStyle { bond_index: usize },
    /// A bond order is absent or outside the composition graph vocabulary.
    #[error("bond {bond_index} has an unsupported or absent bond order")]
    UnsupportedBondOrder { bond_index: usize },
    /// Aromatic order and the retained aromatic flag disagree.
    #[error("bond {bond_index} has inconsistent aromatic order and flag facts")]
    InconsistentAromaticity { bond_index: usize },
    /// The resulting native graph violates an engine-independent invariant.
    #[error(transparent)]
    Graph(#[from] MolGraphError),
    /// Exact owned conversion storage could not be allocated.
    #[error("molecule composition graph could not reserve owned storage")]
    ResourceAllocation,
}
