use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    Atom, Bond, Identifier, LegacyFingerprint, ModelError, NonAtomVertex, RecordId, RecordKind,
    RecordOrigin, VertexRef, formatting::option_text,
};

/// Immutable ordered molecule graph. Revision means validated replacement.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Molecule {
    identity: RecordId,
    source_id: Option<Identifier>,
    name: Option<String>,
    atoms: Vec<Atom>,
    groups: Vec<NonAtomVertex>,
    texts: Vec<NonAtomVertex>,
    queries: Vec<NonAtomVertex>,
    bonds: Vec<Bond>,
    legacy_occurrence: Option<u32>,
}
impl Molecule {
    /// Construct a complete validated graph without exposing an edit API.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_id: Option<Identifier>,
        name: Option<String>,
        atoms: Vec<Atom>,
        groups: Vec<NonAtomVertex>,
        texts: Vec<NonAtomVertex>,
        queries: Vec<NonAtomVertex>,
        bonds: Vec<Bond>,
        legacy_occurrence: Option<u32>,
    ) -> Result<Self, ModelError> {
        let identity = Self::make_identity(
            &source_id,
            name.as_deref(),
            &atoms,
            &groups,
            &texts,
            &queries,
            &bonds,
            legacy_occurrence,
        )?;
        let molecule = Self {
            identity,
            source_id,
            name,
            atoms,
            groups,
            texts,
            queries,
            bonds,
            legacy_occurrence,
        };
        molecule.validate()?;
        Ok(molecule)
    }

    #[allow(clippy::too_many_arguments)]
    fn make_identity(
        source_id: &Option<Identifier>,
        name: Option<&str>,
        atoms: &[Atom],
        groups: &[NonAtomVertex],
        texts: &[NonAtomVertex],
        queries: &[NonAtomVertex],
        bonds: &[Bond],
        legacy_occurrence: Option<u32>,
    ) -> Result<RecordId, ModelError> {
        let mut children: Vec<String> = atoms
            .iter()
            .map(|item| item.identity().canonical())
            .chain(groups.iter().map(|item| item.identity().canonical()))
            .chain(texts.iter().map(|item| item.identity().canonical()))
            .chain(queries.iter().map(|item| item.identity().canonical()))
            .chain(bonds.iter().map(|item| item.identity().canonical()))
            .collect();
        children.sort();
        let mut fields = vec![
            option_text(source_id.as_ref().map(Identifier::as_str)),
            option_text(name),
        ];
        fields.extend(children);
        let fingerprint = LegacyFingerprint::new(RecordKind::Molecule, &fields);
        match (source_id, legacy_occurrence) {
            (Some(id), None) => Ok(RecordId::from_source(RecordKind::Molecule, id)),
            (None, Some(occurrence)) => Ok(RecordId::from_legacy(
                RecordKind::Molecule,
                fingerprint,
                occurrence,
            )),
            (Some(_), Some(_)) => Err(ModelError::SourceRecordHasLegacyOccurrence {
                kind: RecordKind::Molecule,
            }),
            (None, None) => Err(ModelError::MissingLegacyOccurrence {
                kind: RecordKind::Molecule,
            }),
        }
    }
    /// Return molecule identity.
    #[must_use]
    pub fn identity(&self) -> &RecordId {
        &self.identity
    }
    /// Return literal molecule source ID if present.
    #[must_use]
    pub fn source_id(&self) -> Option<&Identifier> {
        self.source_id.as_ref()
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
    /// Return a validated immutable replacement retaining this molecule anchor.
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
            legacy_occurrence: self.legacy_occurrence,
        };
        replacement.validate()?;
        Ok(replacement)
    }
    fn validate(&self) -> Result<(), ModelError> {
        match (
            &self.source_id,
            &self.identity.origin,
            self.legacy_occurrence,
        ) {
            (Some(source), RecordOrigin::Source(actual), None)
                if source == actual && self.identity.kind == RecordKind::Molecule => {}
            (
                None,
                RecordOrigin::Legacy {
                    fingerprint,
                    occurrence,
                },
                Some(value),
            ) if *occurrence == value
                && fingerprint.kind()? == RecordKind::Molecule
                && self.identity.kind == RecordKind::Molecule => {}
            _ => {
                return Err(ModelError::IdentityMismatch {
                    kind: RecordKind::Molecule,
                });
            }
        }
        let mut identities = HashSet::new();
        let mut source_ids = HashSet::new();
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
        id: Option<&Identifier>,
        all: &mut HashSet<Identifier>,
    ) -> Result<(), ModelError> {
        if id.is_none_or(|value| all.insert(value.clone())) {
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
    source_id: Option<Identifier>,
    name: Option<String>,
    atoms: Vec<Atom>,
    groups: Vec<NonAtomVertex>,
    texts: Vec<NonAtomVertex>,
    queries: Vec<NonAtomVertex>,
    bonds: Vec<Bond>,
    legacy_occurrence: Option<u32>,
}
impl<'de> Deserialize<'de> for Molecule {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let w = WireMolecule::deserialize(d)?;
        let result = Self {
            identity: w.identity,
            source_id: w.source_id,
            name: w.name,
            atoms: w.atoms,
            groups: w.groups,
            texts: w.texts,
            queries: w.queries,
            bonds: w.bonds,
            legacy_occurrence: w.legacy_occurrence,
        };
        result.validate().map_err(serde::de::Error::custom)?;
        Ok(result)
    }
}
