use serde::{Deserialize, Serialize};

use crate::{Identifier, ModelError, Position, RecordId, RecordKind};

/// A validated atom whose optional fields preserve source absence.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Atom {
    identity: RecordId,
    source_id: Identifier,
    element: Option<String>,
    position: Position,
    formal_charge: Option<i32>,
    isotope: Option<u16>,
    explicit_hydrogens: Option<u16>,
    valence: Option<u16>,
    multiplicity: Option<u16>,
    free_sites: Option<u16>,
}

impl Atom {
    /// Construct an atom from its required typed-source locator.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_id: Identifier,
        element: Option<String>,
        position: Position,
        formal_charge: Option<i32>,
        isotope: Option<u16>,
        explicit_hydrogens: Option<u16>,
        valence: Option<u16>,
        multiplicity: Option<u16>,
        free_sites: Option<u16>,
    ) -> Result<Self, ModelError> {
        let identity = RecordId::new(RecordKind::Atom, source_id.clone()).map_err(|_| {
            ModelError::InvalidSourceIdentity {
                kind: RecordKind::Atom,
            }
        })?;
        let atom = Self {
            identity,
            source_id,
            element,
            position,
            formal_charge,
            isotope,
            explicit_hydrogens,
            valence,
            multiplicity,
            free_sites,
        };
        atom.validate()?;
        Ok(atom)
    }

    pub(crate) fn validate(&self) -> Result<(), ModelError> {
        if self
            .element
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(ModelError::BlankAtomElement);
        }
        if self.multiplicity == Some(0) {
            return Err(ModelError::ZeroMultiplicity);
        }
        self.position.validate()?;
        if self.identity.kind() == RecordKind::Atom && self.identity.source_id() == &self.source_id
        {
            Ok(())
        } else {
            Err(ModelError::IdentityMismatch {
                kind: RecordKind::Atom,
            })
        }
    }

    /// Return internal identity.
    #[must_use]
    pub fn identity(&self) -> &RecordId {
        &self.identity
    }
    /// Return its required literal source ID.
    #[must_use]
    pub fn source_id(&self) -> &Identifier {
        &self.source_id
    }
    /// Return source element presence/value.
    #[must_use]
    pub fn element(&self) -> Option<&str> {
        self.element.as_deref()
    }
    /// Return the finite source position.
    #[must_use]
    pub fn position(&self) -> Position {
        self.position
    }
    /// Return source formal-charge presence/value.
    #[must_use]
    pub fn formal_charge(&self) -> Option<i32> {
        self.formal_charge
    }
    /// Return source isotope presence/value.
    #[must_use]
    pub fn isotope(&self) -> Option<u16> {
        self.isotope
    }
    /// Return source explicit-hydrogen presence/value.
    #[must_use]
    pub fn explicit_hydrogens(&self) -> Option<u16> {
        self.explicit_hydrogens
    }
    /// Return source valence presence/value.
    #[must_use]
    pub fn valence(&self) -> Option<u16> {
        self.valence
    }
    /// Return source multiplicity presence/value.
    #[must_use]
    pub fn multiplicity(&self) -> Option<u16> {
        self.multiplicity
    }
    /// Return source free-sites presence/value.
    #[must_use]
    pub fn free_sites(&self) -> Option<u16> {
        self.free_sites
    }

    /// Return a validated immutable replacement retaining this source locator.
    #[allow(clippy::too_many_arguments)]
    pub fn replace_source_fields(
        &self,
        element: Option<String>,
        position: Position,
        formal_charge: Option<i32>,
        isotope: Option<u16>,
        explicit_hydrogens: Option<u16>,
        valence: Option<u16>,
        multiplicity: Option<u16>,
        free_sites: Option<u16>,
    ) -> Result<Self, ModelError> {
        let replacement = Self {
            identity: self.identity.clone(),
            source_id: self.source_id.clone(),
            element,
            position,
            formal_charge,
            isotope,
            explicit_hydrogens,
            valence,
            multiplicity,
            free_sites,
        };
        replacement.validate()?;
        Ok(replacement)
    }
}

#[derive(Deserialize)]
struct WireAtom {
    identity: RecordId,
    source_id: Identifier,
    element: Option<String>,
    position: Position,
    formal_charge: Option<i32>,
    isotope: Option<u16>,
    explicit_hydrogens: Option<u16>,
    valence: Option<u16>,
    multiplicity: Option<u16>,
    free_sites: Option<u16>,
}
impl<'de> Deserialize<'de> for Atom {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = WireAtom::deserialize(deserializer)?;
        let atom = Self {
            identity: wire.identity,
            source_id: wire.source_id,
            element: wire.element,
            position: wire.position,
            formal_charge: wire.formal_charge,
            isotope: wire.isotope,
            explicit_hydrogens: wire.explicit_hydrogens,
            valence: wire.valence,
            multiplicity: wire.multiplicity,
            free_sites: wire.free_sites,
        };
        atom.validate().map_err(serde::de::Error::custom)?;
        Ok(atom)
    }
}
