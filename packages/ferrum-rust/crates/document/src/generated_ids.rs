//! Collision-safe session-owned persistent identity allocation.

use super::{IndexedDocument, PersistentId, SessionOperationError};
use std::fmt::Write;

#[derive(Clone, Copy, Debug)]
pub(super) struct GeneratedIdSequences {
    molecule: Option<u64>,
    atom: Option<u64>,
    bond: Option<u64>,
    presentation: Option<u64>,
    fragment: Option<u64>,
    clipboard: Option<u64>,
}

impl GeneratedIdSequences {
    pub(super) const fn initial() -> Self {
        Self {
            molecule: Some(0),
            atom: Some(0),
            bond: Some(0),
            presentation: Some(0),
            fragment: Some(0),
            clipboard: Some(0),
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
                fragment: self.fragment,
                clipboard: self.clipboard,
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

    /// Reserve one opaque-safe generated fragment identity without installing the
    /// returned sequence. Prepared operations carry the returned state until their
    /// authenticated commit succeeds.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn reserve_fragment(
        self,
        indexed: &IndexedDocument,
    ) -> Result<(PersistentId, Self), SessionOperationError> {
        let (fragments, fragment) = allocate(indexed, GeneratedIdKind::Fragment, self.fragment, 1)?;
        let [identifier] = fragments
            .try_into()
            .expect("one requested fragment identity produces one result");
        Ok((identifier, Self { fragment, ..self }))
    }

    pub(super) fn reserve_clipboard(
        self,
        indexed: &IndexedDocument,
        count: usize,
    ) -> Result<(Vec<PersistentId>, Self), SessionOperationError> {
        let (identifiers, clipboard) =
            allocate(indexed, GeneratedIdKind::Clipboard, self.clipboard, count)?;
        Ok((identifiers, Self { clipboard, ..self }))
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

    #[cfg(test)]
    fn with_fragment_sequence(self, fragment: Option<u64>) -> Self {
        Self { fragment, ..self }
    }

    #[cfg(test)]
    fn with_clipboard_sequence(self, clipboard: Option<u64>) -> Self {
        Self { clipboard, ..self }
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
    #[cfg_attr(not(test), allow(dead_code))]
    Fragment,
    Clipboard,
}

impl GeneratedIdKind {
    const fn prefix(self) -> &'static str {
        match self {
            Self::Molecule => "ferrum-molecule-v1-",
            Self::Atom => "ferrum-atom-v1-",
            Self::Bond => "ferrum-bond-v1-",
            Self::Presentation => "ferrum-presentation-v1-",
            Self::Fragment => "ferrum-fragment-v1-",
            Self::Clipboard => "ferrum-paste-v1-",
        }
    }

    const fn exhausted(self) -> SessionOperationError {
        match self {
            Self::Molecule => SessionOperationError::MoleculeIdentifierExhausted,
            Self::Atom => SessionOperationError::AtomIdentifierExhausted,
            Self::Bond => SessionOperationError::BondIdentifierExhausted,
            Self::Presentation => SessionOperationError::PresentationIdentifierExhausted,
            Self::Fragment => SessionOperationError::FragmentIdentifierExhausted,
            Self::Clipboard => SessionOperationError::ClipboardIdentifierExhausted,
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
    let mut identifiers = Vec::new();
    identifiers
        .try_reserve_exact(count)
        .map_err(|_| SessionOperationError::GeneratedIdentifierAllocationFailed)?;
    while identifiers.len() < count {
        let current = sequence.ok_or_else(|| kind.exhausted())?;
        let identifier = generated_identifier(kind, current)?;
        sequence = current.checked_add(1);
        if indexed.resolve_id(&identifier).is_none() {
            identifiers.push(identifier);
        }
    }
    Ok((identifiers, sequence))
}

fn generated_identifier(
    kind: GeneratedIdKind,
    sequence: u64,
) -> Result<PersistentId, SessionOperationError> {
    let prefix = kind.prefix();
    let digits = decimal_digits(sequence);
    let capacity = prefix
        .len()
        .checked_add(digits)
        .ok_or_else(|| kind.exhausted())?;
    let mut spelling = String::new();
    spelling
        .try_reserve_exact(capacity)
        .map_err(|_| SessionOperationError::GeneratedIdentifierAllocationFailed)?;
    spelling.push_str(prefix);
    write!(&mut spelling, "{sequence}").map_err(|_| kind.exhausted())?;
    PersistentId::new(spelling).map_err(|_| kind.exhausted())
}

const fn decimal_digits(value: u64) -> usize {
    if value < 10 {
        1
    } else if value < 100 {
        2
    } else if value < 1_000 {
        3
    } else if value < 10_000 {
        4
    } else if value < 100_000 {
        5
    } else if value < 1_000_000 {
        6
    } else if value < 10_000_000 {
        7
    } else if value < 100_000_000 {
        8
    } else if value < 1_000_000_000 {
        9
    } else if value < 10_000_000_000 {
        10
    } else if value < 100_000_000_000 {
        11
    } else if value < 1_000_000_000_000 {
        12
    } else if value < 10_000_000_000_000 {
        13
    } else if value < 100_000_000_000_000 {
        14
    } else if value < 1_000_000_000_000_000 {
        15
    } else if value < 10_000_000_000_000_000 {
        16
    } else if value < 100_000_000_000_000_000 {
        17
    } else if value < 1_000_000_000_000_000_000 {
        18
    } else if value < 10_000_000_000_000_000_000 {
        19
    } else {
        20
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fragment_exhaustion_has_its_own_typed_error() {
        let indexed = IndexedDocument::parse("<cdml/>").expect("valid empty document");
        let exhausted = GeneratedIdSequences::initial().with_fragment_sequence(None);

        assert!(matches!(
            exhausted.reserve_fragment(&indexed),
            Err(SessionOperationError::FragmentIdentifierExhausted)
        ));
    }

    #[test]
    fn fragment_allocation_skips_opaque_declarations() {
        let indexed = IndexedDocument::parse(
            "<cdml><molecule id=\"m\"><vendor id=\"ferrum-fragment-v1-0\"/></molecule></cdml>",
        )
        .expect("valid opaque declaration");

        let (identifier, _) = GeneratedIdSequences::initial()
            .reserve_fragment(&indexed)
            .expect("fragment identity");

        assert_eq!(identifier.as_str(), "ferrum-fragment-v1-1");
    }

    #[test]
    fn fragment_reservation_is_tentative_until_its_returned_sequence_is_installed() {
        let indexed = IndexedDocument::parse("<cdml/>").expect("valid empty document");
        let original = GeneratedIdSequences::initial();

        let (first, tentative) = original
            .reserve_fragment(&indexed)
            .expect("first fragment identity");
        let (repeated, _) = original
            .reserve_fragment(&indexed)
            .expect("uninstalled sequence remains unchanged");
        let (next, _) = tentative
            .reserve_fragment(&indexed)
            .expect("installed tentative sequence advances");

        assert_eq!(first.as_str(), "ferrum-fragment-v1-0");
        assert_eq!(repeated.as_str(), "ferrum-fragment-v1-0");
        assert_eq!(next.as_str(), "ferrum-fragment-v1-1");
    }

    #[test]
    fn clipboard_exhaustion_has_its_own_typed_error() {
        let indexed = IndexedDocument::parse("<cdml/>").expect("valid empty document");
        let exhausted = GeneratedIdSequences::initial().with_clipboard_sequence(None);

        assert!(matches!(
            exhausted.reserve_clipboard(&indexed, 1),
            Err(SessionOperationError::ClipboardIdentifierExhausted)
        ));
    }

    #[test]
    fn clipboard_allocation_skips_opaque_declarations() {
        let indexed =
            IndexedDocument::parse("<cdml><info><vendor id=\"ferrum-paste-v1-0\"/></info></cdml>")
                .expect("valid opaque declaration");

        let (identifiers, _) = GeneratedIdSequences::initial()
            .reserve_clipboard(&indexed, 1)
            .expect("clipboard identity");

        assert_eq!(identifiers[0].as_str(), "ferrum-paste-v1-1");
    }

    #[test]
    fn clipboard_reservation_is_tentative_until_its_sequence_is_installed() {
        let indexed = IndexedDocument::parse("<cdml/>").expect("valid empty document");
        let original = GeneratedIdSequences::initial();

        let (first, tentative) = original
            .reserve_clipboard(&indexed, 1)
            .expect("first clipboard identity");
        let (repeated, _) = original
            .reserve_clipboard(&indexed, 1)
            .expect("uninstalled sequence remains unchanged");
        let (next, _) = tentative
            .reserve_clipboard(&indexed, 1)
            .expect("installed tentative sequence advances");

        assert_eq!(first[0].as_str(), "ferrum-paste-v1-0");
        assert_eq!(repeated[0].as_str(), "ferrum-paste-v1-0");
        assert_eq!(next[0].as_str(), "ferrum-paste-v1-1");
    }
}
