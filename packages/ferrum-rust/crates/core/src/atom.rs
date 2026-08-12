use serde::{Deserialize, Serialize};

use crate::{
    Identifier, LegacyFingerprint, ModelError, Position, RecordId, RecordKind, RecordOrigin,
    formatting::{option_number, option_text},
};

/// A validated atom whose optional fields preserve source absence.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Atom {
    identity: RecordId,
    source_id: Option<Identifier>,
    element: Option<String>,
    position: Position,
    formal_charge: Option<i32>,
    isotope: Option<u16>,
    explicit_hydrogens: Option<u16>,
    valence: Option<u16>,
    multiplicity: Option<u16>,
    free_sites: Option<u16>,
    legacy_occurrence: Option<u32>,
}

impl Atom {
    /// Construct an atom; idless records require a same-fingerprint occurrence.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_id: Option<Identifier>,
        element: Option<String>,
        position: Position,
        formal_charge: Option<i32>,
        isotope: Option<u16>,
        explicit_hydrogens: Option<u16>,
        valence: Option<u16>,
        multiplicity: Option<u16>,
        free_sites: Option<u16>,
        legacy_occurrence: Option<u32>,
    ) -> Result<Self, ModelError> {
        let fingerprint = Self::fingerprint(
            &source_id,
            &element,
            position,
            formal_charge,
            isotope,
            explicit_hydrogens,
            valence,
            multiplicity,
            free_sites,
        );
        let identity = match (&source_id, legacy_occurrence) {
            (Some(id), None) => RecordId::from_source(RecordKind::Atom, id),
            (None, Some(occurrence)) => {
                RecordId::from_legacy(RecordKind::Atom, fingerprint, occurrence)
            }
            (Some(_), Some(_)) => {
                return Err(ModelError::SourceRecordHasLegacyOccurrence {
                    kind: RecordKind::Atom,
                });
            }
            (None, None) => {
                return Err(ModelError::MissingLegacyOccurrence {
                    kind: RecordKind::Atom,
                });
            }
        };
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
            legacy_occurrence,
        };
        atom.validate()?;
        Ok(atom)
    }
    #[allow(clippy::too_many_arguments)]
    fn fingerprint(
        source_id: &Option<Identifier>,
        element: &Option<String>,
        position: Position,
        charge: Option<i32>,
        isotope: Option<u16>,
        hydrogens: Option<u16>,
        valence: Option<u16>,
        multiplicity: Option<u16>,
        free_sites: Option<u16>,
    ) -> LegacyFingerprint {
        LegacyFingerprint::new(
            RecordKind::Atom,
            &[
                option_text(source_id.as_ref().map(Identifier::as_str)),
                option_text(element.as_deref()),
                position.canonical(),
                option_number(charge),
                option_number(isotope),
                option_number(hydrogens),
                option_number(valence),
                option_number(multiplicity),
                option_number(free_sites),
            ],
        )
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
        self.validate_identity()
    }
    fn validate_identity(&self) -> Result<(), ModelError> {
        match (
            &self.source_id,
            &self.identity.origin,
            self.legacy_occurrence,
        ) {
            (Some(source), RecordOrigin::Source(actual), None)
                if source == actual && self.identity.kind == RecordKind::Atom =>
            {
                Ok(())
            }
            (
                None,
                RecordOrigin::Legacy {
                    fingerprint,
                    occurrence,
                },
                Some(field_occurrence),
            ) if *occurrence == field_occurrence
                && fingerprint.kind()? == RecordKind::Atom
                && self.identity.kind == RecordKind::Atom =>
            {
                Ok(())
            }
            _ => Err(ModelError::IdentityMismatch {
                kind: RecordKind::Atom,
            }),
        }
    }
    /// Return internal identity.
    #[must_use]
    pub fn identity(&self) -> &RecordId {
        &self.identity
    }
    /// Return literal source ID, if supplied.
    #[must_use]
    pub fn source_id(&self) -> Option<&Identifier> {
        self.source_id.as_ref()
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
    /// Return a validated immutable replacement retaining this session anchor.
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
            legacy_occurrence: self.legacy_occurrence,
        };
        replacement.validate()?;
        Ok(replacement)
    }
}

#[derive(Deserialize)]
struct WireAtom {
    identity: RecordId,
    source_id: Option<Identifier>,
    element: Option<String>,
    position: Position,
    formal_charge: Option<i32>,
    isotope: Option<u16>,
    explicit_hydrogens: Option<u16>,
    valence: Option<u16>,
    multiplicity: Option<u16>,
    free_sites: Option<u16>,
    legacy_occurrence: Option<u32>,
}
impl<'de> Deserialize<'de> for Atom {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let w = WireAtom::deserialize(d)?;
        let atom = Self {
            identity: w.identity,
            source_id: w.source_id,
            element: w.element,
            position: w.position,
            formal_charge: w.formal_charge,
            isotope: w.isotope,
            explicit_hydrogens: w.explicit_hydrogens,
            valence: w.valence,
            multiplicity: w.multiplicity,
            free_sites: w.free_sites,
            legacy_occurrence: w.legacy_occurrence,
        };
        atom.validate().map_err(serde::de::Error::custom)?;
        Ok(atom)
    }
}
