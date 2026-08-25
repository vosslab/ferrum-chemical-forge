use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{Atom, Bond, Identifier, ModelError, NonAtomVertex, RecordId, RecordKind, VertexRef};

/// Immutable ordered molecule graph. Revision means validated replacement.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Molecule {
    identity: RecordId,
    source_id: Identifier,
    name: Option<String>,
    atoms: Vec<Atom>,
    groups: Vec<NonAtomVertex>,
    texts: Vec<NonAtomVertex>,
    queries: Vec<NonAtomVertex>,
    bonds: Vec<Bond>,
}
impl Molecule {
    /// Construct a complete validated source-identified graph without an edit API.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_id: Identifier,
        name: Option<String>,
        atoms: Vec<Atom>,
        groups: Vec<NonAtomVertex>,
        texts: Vec<NonAtomVertex>,
        queries: Vec<NonAtomVertex>,
        bonds: Vec<Bond>,
    ) -> Result<Self, ModelError> {
        let identity = RecordId::new(RecordKind::Molecule, source_id.clone()).map_err(|_| {
            ModelError::InvalidSourceIdentity {
                kind: RecordKind::Molecule,
            }
        })?;
        let molecule = Self {
            identity,
            source_id,
            name,
            atoms,
            groups,
            texts,
            queries,
            bonds,
        };
        molecule.validate()?;
        Ok(molecule)
    }
    /// Return molecule identity.
    #[must_use]
    pub fn identity(&self) -> &RecordId {
        &self.identity
    }
    /// Return its required literal source ID.
    #[must_use]
    pub fn source_id(&self) -> &Identifier {
        &self.source_id
    }
    /// Return source molecule name presence/value.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    /// Return atoms in source order.
    #[must_use]
    pub fn atoms(&self) -> &[Atom] {
        &self.atoms
    }
    /// Return group vertices in source order.
    #[must_use]
    pub fn groups(&self) -> &[NonAtomVertex] {
        &self.groups
    }
    /// Return molecule-local text vertices in source order.
    #[must_use]
    pub fn texts(&self) -> &[NonAtomVertex] {
        &self.texts
    }
    /// Return query vertices in source order.
    #[must_use]
    pub fn queries(&self) -> &[NonAtomVertex] {
        &self.queries
    }
    /// Return bonds in source order.
    #[must_use]
    pub fn bonds(&self) -> &[Bond] {
        &self.bonds
    }
    /// Return a validated immutable replacement retaining this source locator.
    #[allow(clippy::too_many_arguments)]
    pub fn replace_records(
        &self,
        name: Option<String>,
        atoms: Vec<Atom>,
        groups: Vec<NonAtomVertex>,
        texts: Vec<NonAtomVertex>,
        queries: Vec<NonAtomVertex>,
        bonds: Vec<Bond>,
    ) -> Result<Self, ModelError> {
        let replacement = Self {
            identity: self.identity.clone(),
            source_id: self.source_id.clone(),
            name,
            atoms,
            groups,
            texts,
            queries,
            bonds,
        };
        replacement.validate()?;
        Ok(replacement)
    }
    fn validate(&self) -> Result<(), ModelError> {
        if self.identity.kind() != RecordKind::Molecule
            || self.identity.source_id() != &self.source_id
        {
            return Err(ModelError::IdentityMismatch {
                kind: RecordKind::Molecule,
            });
        }
        let mut identities = HashSet::new();
        let mut source_ids = HashSet::new();
        self.insert_source(&self.source_id, &mut source_ids)?;
        for atom in &self.atoms {
            atom.validate()?;
            self.insert_identity(atom.identity(), &mut identities)?;
            self.insert_source(atom.source_id(), &mut source_ids)?;
        }
        for (kind, vertices) in [
            (RecordKind::Group, &self.groups),
            (RecordKind::Text, &self.texts),
            (RecordKind::Query, &self.queries),
        ] {
            for vertex in vertices {
                vertex.validate(kind)?;
                self.insert_identity(vertex.identity(), &mut identities)?;
                self.insert_source(vertex.source_id(), &mut source_ids)?;
            }
        }
        for bond in &self.bonds {
            bond.validate()?;
            self.insert_identity(bond.identity(), &mut identities)?;
            self.insert_source(bond.source_id(), &mut source_ids)?;
            self.resolve(bond.start())?;
            self.resolve(bond.end())?;
        }
        Ok(())
    }
    fn insert_identity(
        &self,
        id: &RecordId,
        all: &mut HashSet<RecordId>,
    ) -> Result<(), ModelError> {
        if all.insert(id.clone()) {
            Ok(())
        } else {
            Err(ModelError::DuplicateIdentity)
        }
    }
    fn insert_source(
        &self,
        id: &Identifier,
        all: &mut HashSet<Identifier>,
    ) -> Result<(), ModelError> {
        if all.insert(id.clone()) {
            Ok(())
        } else {
            Err(ModelError::DuplicateSourceId)
        }
    }
    fn resolve(&self, endpoint: &VertexRef) -> Result<(), ModelError> {
        let found = match endpoint {
            VertexRef::Atom(id) => self.atoms.iter().any(|item| item.identity() == id),
            VertexRef::Group(id) => self.groups.iter().any(|item| item.identity() == id),
            VertexRef::Text(id) => self.texts.iter().any(|item| item.identity() == id),
            VertexRef::Query(id) => self.queries.iter().any(|item| item.identity() == id),
        };
        if found {
            Ok(())
        } else {
            Err(ModelError::UnresolvedBondEndpoint)
        }
    }
}
#[derive(Deserialize)]
struct WireMolecule {
    identity: RecordId,
    source_id: Identifier,
    name: Option<String>,
    atoms: Vec<Atom>,
    groups: Vec<NonAtomVertex>,
    texts: Vec<NonAtomVertex>,
    queries: Vec<NonAtomVertex>,
    bonds: Vec<Bond>,
}
impl<'de> Deserialize<'de> for Molecule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = WireMolecule::deserialize(deserializer)?;
        let molecule = Self {
            identity: wire.identity,
            source_id: wire.source_id,
            name: wire.name,
            atoms: wire.atoms,
            groups: wire.groups,
            texts: wire.texts,
            queries: wire.queries,
            bonds: wire.bonds,
        };
        molecule.validate().map_err(serde::de::Error::custom)?;
        Ok(molecule)
    }
}
