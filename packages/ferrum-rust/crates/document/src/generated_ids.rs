//! Collision-safe session-owned persistent identity allocation.

use super::{IndexedDocument, PersistentId, SessionOperationError};

#[derive(Clone, Copy, Debug)]
pub(super) struct GeneratedIdSequences {
    molecule: Option<u64>,
    atom: Option<u64>,
    bond: Option<u64>,
    presentation: Option<u64>,
}

impl GeneratedIdSequences {
    pub(super) const fn initial() -> Self {
        Self {
            molecule: Some(0),
            atom: Some(0),
            bond: Some(0),
            presentation: Some(0),
        }
    }

    pub(super) fn reserve_molecule(
        self,
        indexed: &IndexedDocument,
        atom_count: usize,
        bond_count: usize,
    ) -> Result<(GeneratedMoleculeIdentities, Self), SessionOperationError> {
        let (molecules, molecule) = allocate(indexed, GeneratedIdKind::Molecule, self.molecule, 1)?;
        let (atoms, atom) = allocate(indexed, GeneratedIdKind::Atom, self.atom, atom_count)?;
        let (bonds, bond) = allocate(indexed, GeneratedIdKind::Bond, self.bond, bond_count)?;
        let [molecule_id] = molecules
            .try_into()
            .expect("one requested molecule identity produces one result");
        Ok((
            GeneratedMoleculeIdentities {
                molecule: molecule_id,
                atoms,
                bonds,
            },
            Self {
                molecule,
                atom,
                bond,
                presentation: self.presentation,
            },
        ))
    }

    pub(super) fn reserve_atom(
        self,
        indexed: &IndexedDocument,
    ) -> Result<(PersistentId, Self), SessionOperationError> {
        let (atoms, atom) = allocate(indexed, GeneratedIdKind::Atom, self.atom, 1)?;
        let [identifier] = atoms
            .try_into()
            .expect("one requested atom identity produces one result");
        Ok((identifier, Self { atom, ..self }))
    }

    pub(super) fn reserve_bond(
        self,
        indexed: &IndexedDocument,
    ) -> Result<(PersistentId, Self), SessionOperationError> {
        let (bonds, bond) = allocate(indexed, GeneratedIdKind::Bond, self.bond, 1)?;
        let [identifier] = bonds
            .try_into()
            .expect("one requested bond identity produces one result");
        Ok((identifier, Self { bond, ..self }))
    }

    pub(super) fn reserve_bonded_atom(
        self,
        indexed: &IndexedDocument,
    ) -> Result<(GeneratedBondedAtomIdentities, Self), SessionOperationError> {
        let (atoms, atom) = allocate(indexed, GeneratedIdKind::Atom, self.atom, 1)?;
        let (bonds, bond) = allocate(indexed, GeneratedIdKind::Bond, self.bond, 1)?;
        let [atom_id] = atoms
            .try_into()
            .expect("one requested atom identity produces one result");
        let [bond_id] = bonds
            .try_into()
            .expect("one requested bond identity produces one result");
        Ok((
            GeneratedBondedAtomIdentities {
                atom: atom_id,
                bond: bond_id,
            },
            Self { atom, bond, ..self },
        ))
    }

    pub(super) fn reserve_presentation(
        self,
        indexed: &IndexedDocument,
    ) -> Result<(PersistentId, Self), SessionOperationError> {
        let (presentations, presentation) =
            allocate(indexed, GeneratedIdKind::Presentation, self.presentation, 1)?;
        let [identifier] = presentations
            .try_into()
            .expect("one requested presentation identity produces one result");
        Ok((
            identifier,
            Self {
                presentation,
                ..self
            },
        ))
    }

    pub(super) fn reserve_presentations<const N: usize>(
        self,
        indexed: &IndexedDocument,
    ) -> Result<([PersistentId; N], Self), SessionOperationError> {
        let (presentations, presentation) =
            allocate(indexed, GeneratedIdKind::Presentation, self.presentation, N)?;
        let identifiers = presentations
            .try_into()
            .unwrap_or_else(|_| unreachable!("requested identity count matches array size"));
        Ok((
            identifiers,
            Self {
                presentation,
                ..self
            },
        ))
    }

    #[cfg(test)]
    pub(super) fn with_atom_sequence(self, atom: Option<u64>) -> Self {
        Self { atom, ..self }
    }

    #[cfg(test)]
    pub(super) fn with_bond_sequence(self, bond: Option<u64>) -> Self {
        Self { bond, ..self }
    }
}

pub(super) struct GeneratedMoleculeIdentities {
    pub(super) molecule: PersistentId,
    pub(super) atoms: Vec<PersistentId>,
    pub(super) bonds: Vec<PersistentId>,
}

pub(super) struct GeneratedBondedAtomIdentities {
    pub(super) atom: PersistentId,
    pub(super) bond: PersistentId,
}

#[derive(Clone, Copy)]
enum GeneratedIdKind {
    Molecule,
    Atom,
    Bond,
    Presentation,
}

impl GeneratedIdKind {
    const fn prefix(self) -> &'static str {
        match self {
            Self::Molecule => "ferrum-molecule-v1-",
            Self::Atom => "ferrum-atom-v1-",
            Self::Bond => "ferrum-bond-v1-",
            Self::Presentation => "ferrum-presentation-v1-",
        }
    }

    const fn exhausted(self) -> SessionOperationError {
        match self {
            Self::Molecule => SessionOperationError::MoleculeIdentifierExhausted,
            Self::Atom => SessionOperationError::AtomIdentifierExhausted,
            Self::Bond => SessionOperationError::BondIdentifierExhausted,
            Self::Presentation => SessionOperationError::PresentationIdentifierExhausted,
        }
    }
}

fn allocate(
    indexed: &IndexedDocument,
    kind: GeneratedIdKind,
    start: Option<u64>,
    count: usize,
) -> Result<(Vec<PersistentId>, Option<u64>), SessionOperationError> {
    let mut sequence = start;
    let mut identifiers = Vec::with_capacity(count);
    while identifiers.len() < count {
        let current = sequence.ok_or_else(|| kind.exhausted())?;
        let identifier = PersistentId::new(format!("{}{current}", kind.prefix()))
            .map_err(|_| kind.exhausted())?;
        sequence = current.checked_add(1);
        if indexed.resolve_id(&identifier).is_none() {
            identifiers.push(identifier);
        }
    }
    Ok((identifiers, sequence))
}
