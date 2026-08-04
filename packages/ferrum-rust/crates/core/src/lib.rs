//! Shared, chemistry-independent Ferrum molecule types.
//!
//! This crate owns validated record identity and graph shape. Serde here is
//! internal persistence/testing only, not the M17 wire ABI.

use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error returned when source text is blank.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{kind} identifier must contain at least one non-whitespace character")]
pub struct InvalidIdentifier {
    /// The rejected identity domain.
    pub kind: &'static str,
}

/// Exact nonblank source text, never normalized by the core.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Identifier(String);

impl Identifier {
    /// Construct nonblank source text.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidIdentifier> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(InvalidIdentifier { kind: "source" });
        }
        Ok(Self(value))
    }

    /// Return the exact source text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// The CDML record class carried by an internal identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum RecordKind {
    /// Molecule record.
    Molecule,
    /// Atom vertex.
    Atom,
    /// Group vertex.
    Group,
    /// Molecule-local text vertex.
    Text,
    /// Query vertex.
    Query,
    /// Bond record.
    Bond,
}

/// A versioned canonical encoding for an idless record's carried source facts.
///
/// The text is internally constructed with length-prefixed UTF-8 fields, so
/// delimiter-containing input cannot alias another field sequence.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LegacyFingerprint(String);

impl LegacyFingerprint {
    fn new(kind: RecordKind, fields: &[String]) -> Self {
        let mut encoded = format!("ferrum-core-legacy-v1:{kind:?}");
        for field in fields {
            encoded.push(':');
            encoded.push_str(&field.len().to_string());
            encoded.push(':');
            encoded.push_str(field);
        }
        Self(encoded)
    }

    #[cfg(test)]
    fn test_encoding(kind: RecordKind, fields: &[String]) -> Self {
        Self::new(kind, fields)
    }

    fn kind(&self) -> Result<RecordKind, ModelError> {
        Self::parse(&self.0).map(|parsed| parsed.kind)
    }

    fn parse(value: &str) -> Result<ParsedFingerprint, ModelError> {
        const PREFIX: &str = "ferrum-core-legacy-v1:";
        let remainder = value
            .strip_prefix(PREFIX)
            .ok_or(ModelError::MalformedLegacyFingerprint)?;
        let (kind_text, mut encoded) = remainder
            .split_once(':')
            .ok_or(ModelError::MalformedLegacyFingerprint)?;
        let kind = match kind_text {
            "Molecule" => RecordKind::Molecule,
            "Atom" => RecordKind::Atom,
            "Group" => RecordKind::Group,
            "Text" => RecordKind::Text,
            "Query" => RecordKind::Query,
            "Bond" => RecordKind::Bond,
            _ => return Err(ModelError::MalformedLegacyFingerprint),
        };
        let mut field_count = 0;
        while !encoded.is_empty() {
            let (length, after_length) = encoded
                .split_once(':')
                .ok_or(ModelError::MalformedLegacyFingerprint)?;
            if length.is_empty() || !length.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(ModelError::MalformedLegacyFingerprint);
            }
            let length: usize = length
                .parse()
                .map_err(|_| ModelError::MalformedLegacyFingerprint)?;
            if after_length.len() < length || !after_length.is_char_boundary(length) {
                return Err(ModelError::MalformedLegacyFingerprint);
            }
            let (_, remaining) = after_length.split_at(length);
            encoded = if remaining.is_empty() {
                ""
            } else {
                remaining
                    .strip_prefix(':')
                    .ok_or(ModelError::MalformedLegacyFingerprint)?
            };
            field_count += 1;
        }
        let shape_valid = match kind {
            RecordKind::Molecule => field_count >= 2,
            RecordKind::Atom => field_count == 9,
            RecordKind::Group | RecordKind::Text | RecordKind::Query => field_count == 1,
            RecordKind::Bond => field_count == 7,
        };
        if shape_valid {
            Ok(ParsedFingerprint { kind })
        } else {
            Err(ModelError::MalformedLegacyFingerprint)
        }
    }
}

struct ParsedFingerprint {
    kind: RecordKind,
}

impl<'de> Deserialize<'de> for LegacyFingerprint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let fingerprint = Self(String::deserialize(deserializer)?);
        Self::parse(&fingerprint.0).map_err(serde::de::Error::custom)?;
        Ok(fingerprint)
    }
}

/// Origin of a structurally typed internal identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum RecordOrigin {
    /// Identity derived exactly from a present source `@id`.
    Source(Identifier),
    /// Identity derived from source facts that carried no `@id`.
    Legacy {
        /// Canonical source-fact fingerprint.
        fingerprint: LegacyFingerprint,
        /// Occurrence only among exact same-fingerprint siblings in one session.
        occurrence: u32,
    },
}

/// A stable internal identity whose kind and origin are executable invariants.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RecordId {
    kind: RecordKind,
    origin: RecordOrigin,
}

#[derive(Deserialize)]
struct WireRecordId {
    kind: RecordKind,
    origin: RecordOrigin,
}

impl<'de> Deserialize<'de> for RecordId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = WireRecordId::deserialize(deserializer)?;
        if let RecordOrigin::Legacy { fingerprint, .. } = &wire.origin
            && fingerprint.kind().map_err(serde::de::Error::custom)? != wire.kind
        {
            return Err(serde::de::Error::custom(
                "legacy fingerprint kind does not match record kind",
            ));
        }
        Ok(Self {
            kind: wire.kind,
            origin: wire.origin,
        })
    }
}

impl RecordId {
    /// Derive a source-backed identity without fabricating a source ID.
    #[must_use]
    pub fn from_source(kind: RecordKind, source_id: &Identifier) -> Self {
        Self {
            kind,
            origin: RecordOrigin::Source(source_id.clone()),
        }
    }

    fn from_legacy(kind: RecordKind, fingerprint: LegacyFingerprint, occurrence: u32) -> Self {
        Self {
            kind,
            origin: RecordOrigin::Legacy {
                fingerprint,
                occurrence,
            },
        }
    }

    /// Return the record class encoded in this identity.
    #[must_use]
    pub fn kind(&self) -> RecordKind {
        self.kind
    }

    /// Return its structural origin.
    #[must_use]
    pub fn origin(&self) -> &RecordOrigin {
        &self.origin
    }

    fn canonical(&self) -> String {
        match &self.origin {
            RecordOrigin::Source(id) => {
                format!("{:?}:source:{}:{}", self.kind, id.as_str().len(), id)
            }
            RecordOrigin::Legacy {
                fingerprint,
                occurrence,
            } => {
                format!(
                    "{:?}:legacy:{}:{}",
                    self.kind,
                    fingerprint.0.len(),
                    fingerprint.0
                ) + &format!(":{occurrence}")
            }
        }
    }
}

/// A finite 3D coordinate retained without assigning chemistry meaning.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}

impl Position {
    /// Construct a finite position.
    pub fn new(x: f64, y: f64, z: f64) -> Result<Self, ModelError> {
        let value = Self { x, y, z };
        value.validate()?;
        Ok(value)
    }
    /// Return x.
    #[must_use]
    pub fn x(self) -> f64 {
        self.x
    }
    /// Return y.
    #[must_use]
    pub fn y(self) -> f64 {
        self.y
    }
    /// Return z.
    #[must_use]
    pub fn z(self) -> f64 {
        self.z
    }
    fn validate(self) -> Result<(), ModelError> {
        for (axis, value) in [("x", self.x), ("y", self.y), ("z", self.z)] {
            if !value.is_finite() {
                return Err(ModelError::NonFiniteCoordinate { axis });
            }
        }
        Ok(())
    }
    fn canonical(self) -> String {
        format!(
            "{:016x}{:016x}{:016x}",
            self.x.to_bits(),
            self.y.to_bits(),
            self.z.to_bits()
        )
    }
}

#[derive(Deserialize)]
struct WirePosition {
    x: f64,
    y: f64,
    z: f64,
}
impl<'de> Deserialize<'de> for Position {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = WirePosition::deserialize(deserializer)?;
        Self::new(wire.x, wire.y, wire.z).map_err(serde::de::Error::custom)
    }
}

/// Observed bond order, deliberately optional when source type was absent.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BondOrder {
    Single,
    Double,
    Triple,
    Aromatic,
    Other(u8),
}

/// Observed bond depiction style, deliberately optional when source type was absent.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BondStyle {
    Normal,
    Wedge,
    Hashed,
    Adder,
    Bold,
    Dashed,
    Dotted,
    Wavy,
    HaworthFront,
    Other(String),
}

/// A typed molecule-local endpoint reference.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub enum VertexRef {
    Atom(RecordId),
    Group(RecordId),
    Text(RecordId),
    Query(RecordId),
}

#[derive(Deserialize)]
enum WireVertexRef {
    Atom(RecordId),
    Group(RecordId),
    Text(RecordId),
    Query(RecordId),
}

impl<'de> Deserialize<'de> for VertexRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = WireVertexRef::deserialize(deserializer)?;
        let value = match wire {
            WireVertexRef::Atom(id) => Self::Atom(id),
            WireVertexRef::Group(id) => Self::Group(id),
            WireVertexRef::Text(id) => Self::Text(id),
            WireVertexRef::Query(id) => Self::Query(id),
        };
        value.validate_kind().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl VertexRef {
    /// Return whether this reference names a chemistry atom.
    #[must_use]
    pub fn is_atom(&self) -> bool {
        matches!(self, Self::Atom(_))
    }
    fn canonical(&self) -> String {
        let tag = match self {
            Self::Atom(_) => "atom",
            Self::Group(_) => "group",
            Self::Text(_) => "text",
            Self::Query(_) => "query",
        };
        format!("{tag}:{}", self.record_id().canonical())
    }
    fn record_id(&self) -> &RecordId {
        match self {
            Self::Atom(id) | Self::Group(id) | Self::Text(id) | Self::Query(id) => id,
        }
    }
    fn validate_kind(&self) -> Result<(), ModelError> {
        let expected = match self {
            Self::Atom(_) => RecordKind::Atom,
            Self::Group(_) => RecordKind::Group,
            Self::Text(_) => RecordKind::Text,
            Self::Query(_) => RecordKind::Query,
        };
        if self.record_id().kind() == expected {
            Ok(())
        } else {
            Err(ModelError::VertexKindMismatch)
        }
    }
}

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
    fn validate(&self) -> Result<(), ModelError> {
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

/// Minimal group/text/query carrier needed for typed endpoints.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NonAtomVertex {
    identity: RecordId,
    source_id: Option<Identifier>,
    legacy_occurrence: Option<u32>,
}
impl NonAtomVertex {
    /// Construct an idless or source-backed non-atom vertex.
    pub fn new(
        kind: RecordKind,
        source_id: Option<Identifier>,
        legacy_occurrence: Option<u32>,
    ) -> Result<Self, ModelError> {
        if !matches!(
            kind,
            RecordKind::Group | RecordKind::Text | RecordKind::Query
        ) {
            return Err(ModelError::InvalidVertexKind { kind });
        }
        let fingerprint = LegacyFingerprint::new(
            kind,
            &[option_text(source_id.as_ref().map(Identifier::as_str))],
        );
        let identity = match (&source_id, legacy_occurrence) {
            (Some(id), None) => RecordId::from_source(kind, id),
            (None, Some(occurrence)) => RecordId::from_legacy(kind, fingerprint, occurrence),
            (Some(_), Some(_)) => return Err(ModelError::SourceRecordHasLegacyOccurrence { kind }),
            (None, None) => return Err(ModelError::MissingLegacyOccurrence { kind }),
        };
        Ok(Self {
            identity,
            source_id,
            legacy_occurrence,
        })
    }
    fn validate(&self, kind: RecordKind) -> Result<(), ModelError> {
        let expected = Self::new(kind, self.source_id.clone(), self.legacy_occurrence)?;
        if expected.identity == self.identity {
            Ok(())
        } else {
            Err(ModelError::IdentityMismatch { kind })
        }
    }
    /// Return internal identity.
    #[must_use]
    pub fn identity(&self) -> &RecordId {
        &self.identity
    }
    /// Return literal source ID if present.
    #[must_use]
    pub fn source_id(&self) -> Option<&Identifier> {
        self.source_id.as_ref()
    }
}
#[derive(Deserialize)]
struct WireNonAtomVertex {
    identity: RecordId,
    source_id: Option<Identifier>,
    legacy_occurrence: Option<u32>,
}
impl<'de> Deserialize<'de> for NonAtomVertex {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let w = WireNonAtomVertex::deserialize(d)?;
        let kind = w.identity.kind;
        let result =
            Self::new(kind, w.source_id, w.legacy_occurrence).map_err(serde::de::Error::custom)?;
        if result.identity != w.identity {
            return Err(serde::de::Error::custom(
                "vertex identity does not match carried source fields",
            ));
        }
        Ok(result)
    }
}

/// A validated bond with typed, ordered endpoints and source-type presence.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Bond {
    identity: RecordId,
    source_id: Option<Identifier>,
    start: VertexRef,
    end: VertexRef,
    source_type: Option<String>,
    order: Option<BondOrder>,
    style: Option<BondStyle>,
    aromatic: Option<bool>,
    legacy_occurrence: Option<u32>,
}
impl Bond {
    /// Construct a bond; absent source type remains absent rather than defaulted.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_id: Option<Identifier>,
        start: VertexRef,
        end: VertexRef,
        source_type: Option<String>,
        order: Option<BondOrder>,
        style: Option<BondStyle>,
        aromatic: Option<bool>,
        legacy_occurrence: Option<u32>,
    ) -> Result<Self, ModelError> {
        let identity = Self::make_identity(
            &source_id,
            &start,
            &end,
            &source_type,
            order,
            &style,
            aromatic,
            legacy_occurrence,
        )?;
        let bond = Self {
            identity,
            source_id,
            start,
            end,
            source_type,
            order,
            style,
            aromatic,
            legacy_occurrence,
        };
        bond.validate()?;
        Ok(bond)
    }
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    fn make_identity(
        source_id: &Option<Identifier>,
        start: &VertexRef,
        end: &VertexRef,
        source_type: &Option<String>,
        order: Option<BondOrder>,
        style: &Option<BondStyle>,
        aromatic: Option<bool>,
        legacy_occurrence: Option<u32>,
    ) -> Result<RecordId, ModelError> {
        let fingerprint =
            Self::fingerprint(source_id, start, end, source_type, order, style, aromatic);
        match (source_id, legacy_occurrence) {
            (Some(id), None) => Ok(RecordId::from_source(RecordKind::Bond, id)),
            (None, Some(occurrence)) => Ok(RecordId::from_legacy(
                RecordKind::Bond,
                fingerprint,
                occurrence,
            )),
            (Some(_), Some(_)) => Err(ModelError::SourceRecordHasLegacyOccurrence {
                kind: RecordKind::Bond,
            }),
            (None, None) => Err(ModelError::MissingLegacyOccurrence {
                kind: RecordKind::Bond,
            }),
        }
    }
    fn fingerprint(
        source_id: &Option<Identifier>,
        start: &VertexRef,
        end: &VertexRef,
        source_type: &Option<String>,
        order: Option<BondOrder>,
        style: &Option<BondStyle>,
        aromatic: Option<bool>,
    ) -> LegacyFingerprint {
        LegacyFingerprint::new(
            RecordKind::Bond,
            &[
                option_text(source_id.as_ref().map(Identifier::as_str)),
                start.canonical(),
                end.canonical(),
                option_text(source_type.as_deref()),
                option_debug(order),
                option_debug(style.clone()),
                option_number(aromatic.map(u8::from)),
            ],
        )
    }
    fn validate(&self) -> Result<(), ModelError> {
        self.start.validate_kind()?;
        self.end.validate_kind()?;
        if self
            .source_type
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(ModelError::BlankBondType);
        }
        if self.start == self.end {
            return Err(ModelError::SelfBond);
        }
        match (
            &self.source_id,
            &self.identity.origin,
            self.legacy_occurrence,
        ) {
            (Some(source), RecordOrigin::Source(actual), None)
                if source == actual && self.identity.kind == RecordKind::Bond =>
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
                && fingerprint.kind()? == RecordKind::Bond
                && self.identity.kind == RecordKind::Bond =>
            {
                Ok(())
            }
            _ => Err(ModelError::IdentityMismatch {
                kind: RecordKind::Bond,
            }),
        }
    }
    /// Return internal identity.
    #[must_use]
    pub fn identity(&self) -> &RecordId {
        &self.identity
    }
    /// Return literal source ID if present.
    #[must_use]
    pub fn source_id(&self) -> Option<&Identifier> {
        self.source_id.as_ref()
    }
    /// Return ordered first endpoint.
    #[must_use]
    pub fn start(&self) -> &VertexRef {
        &self.start
    }
    /// Return ordered second endpoint.
    #[must_use]
    pub fn end(&self) -> &VertexRef {
        &self.end
    }
    /// Return exact source `type` presence/value.
    #[must_use]
    pub fn source_type(&self) -> Option<&str> {
        self.source_type.as_deref()
    }
    /// Return observed/normalized order only when supplied by a codec.
    #[must_use]
    pub fn order(&self) -> Option<BondOrder> {
        self.order
    }
    /// Return observed/normalized style only when supplied by a codec.
    #[must_use]
    pub fn style(&self) -> Option<&BondStyle> {
        self.style.as_ref()
    }
    /// Return source aromatic-flag presence/value.
    #[must_use]
    pub fn aromatic(&self) -> Option<bool> {
        self.aromatic
    }
    /// Return a validated immutable replacement retaining this session anchor.
    pub fn replace_source_fields(
        &self,
        start: VertexRef,
        end: VertexRef,
        source_type: Option<String>,
        order: Option<BondOrder>,
        style: Option<BondStyle>,
        aromatic: Option<bool>,
    ) -> Result<Self, ModelError> {
        let replacement = Self {
            identity: self.identity.clone(),
            source_id: self.source_id.clone(),
            start,
            end,
            source_type,
            order,
            style,
            aromatic,
            legacy_occurrence: self.legacy_occurrence,
        };
        replacement.validate()?;
        Ok(replacement)
    }
}
#[derive(Deserialize)]
struct WireBond {
    identity: RecordId,
    source_id: Option<Identifier>,
    start: VertexRef,
    end: VertexRef,
    source_type: Option<String>,
    order: Option<BondOrder>,
    style: Option<BondStyle>,
    aromatic: Option<bool>,
    legacy_occurrence: Option<u32>,
}
impl<'de> Deserialize<'de> for Bond {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let w = WireBond::deserialize(d)?;
        let bond = Self {
            identity: w.identity,
            source_id: w.source_id,
            start: w.start,
            end: w.end,
            source_type: w.source_type,
            order: w.order,
            style: w.style,
            aromatic: w.aromatic,
            legacy_occurrence: w.legacy_occurrence,
        };
        bond.validate().map_err(serde::de::Error::custom)?;
        Ok(bond)
    }
}

/// Core structural validation errors.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ModelError {
    #[error("legacy fingerprint has an invalid version, kind, length encoding, or field shape")]
    MalformedLegacyFingerprint,
    #[error("{axis} coordinate must be finite")]
    NonFiniteCoordinate { axis: &'static str },
    #[error("present atom element is blank")]
    BlankAtomElement,
    #[error("present atom multiplicity is zero")]
    ZeroMultiplicity,
    #[error("present bond type is blank")]
    BlankBondType,
    #[error("{kind:?} source record cannot carry a legacy occurrence")]
    SourceRecordHasLegacyOccurrence { kind: RecordKind },
    #[error("idless {kind:?} record needs an occurrence among equal fingerprints")]
    MissingLegacyOccurrence { kind: RecordKind },
    #[error("invalid non-atom vertex kind {kind:?}")]
    InvalidVertexKind { kind: RecordKind },
    #[error("{kind:?} identity does not match its kind, origin, or carried fields")]
    IdentityMismatch { kind: RecordKind },
    #[error("duplicate internal identity")]
    DuplicateIdentity,
    #[error("duplicate molecule-local source identifier")]
    DuplicateSourceId,
    #[error("bond has identical typed endpoints")]
    SelfBond,
    #[error("bond endpoint does not resolve to its declared typed vertex")]
    UnresolvedBondEndpoint,
    #[error("vertex reference variant does not match its record kind")]
    VertexKindMismatch,
}

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
    /// Construct a complete validated graph. M2 deliberately provides no edit API.
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

fn option_text(value: Option<&str>) -> String {
    value.map_or_else(|| "none".to_owned(), |item| format!("some:{item}"))
}
fn option_number<T: fmt::Display>(value: Option<T>) -> String {
    value.map_or_else(|| "none".to_owned(), |item| format!("some:{item}"))
}
fn option_debug<T: fmt::Debug>(value: Option<T>) -> String {
    value.map_or_else(|| "none".to_owned(), |item| format!("some:{item:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    fn source(value: &str) -> Identifier {
        Identifier::new(value).expect("test id is valid")
    }
    fn atom(id: Option<&str>, occurrence: Option<u32>, x: f64) -> Atom {
        Atom::new(
            id.map(source),
            Some("C".to_owned()),
            Position::new(x, 0.0, 0.0).expect("finite"),
            None,
            None,
            None,
            None,
            None,
            None,
            occurrence,
        )
        .expect("valid atom")
    }
    fn vertex(kind: RecordKind, id: Option<&str>, occurrence: Option<u32>) -> NonAtomVertex {
        NonAtomVertex::new(kind, id.map(source), occurrence).expect("valid vertex")
    }
    fn bond(start: VertexRef, end: VertexRef, occurrence: Option<u32>) -> Bond {
        Bond::new(None, start, end, None, None, None, None, occurrence).expect("valid bond")
    }
    fn molecule(
        atoms: Vec<Atom>,
        groups: Vec<NonAtomVertex>,
        texts: Vec<NonAtomVertex>,
        queries: Vec<NonAtomVertex>,
        bonds: Vec<Bond>,
    ) -> Result<Molecule, ModelError> {
        Molecule::new(
            Some(source("m")),
            None,
            atoms,
            groups,
            texts,
            queries,
            bonds,
            None,
        )
    }

    #[test]
    fn serde_rejects_spoofed_source_identity() {
        let atom = atom(Some("a"), None, 0.0);
        let mut json = serde_json::to_value(&atom).expect("serialize");
        json["identity"]["origin"]["Source"] = serde_json::json!("other");
        assert!(serde_json::from_value::<Atom>(json).is_err());

        let mut json = serde_json::to_value(&atom).expect("serialize");
        json["identity"]["kind"] = serde_json::json!("Bond");
        assert!(serde_json::from_value::<Atom>(json).is_err());
    }
    #[test]
    fn bond_rejects_every_wrong_vertex_reference_kind() {
        let atom = atom(Some("a"), None, 0.0);
        let correct = VertexRef::Atom(atom.identity().clone());
        for wrong in [
            VertexRef::Group(atom.identity().clone()),
            VertexRef::Text(atom.identity().clone()),
            VertexRef::Query(atom.identity().clone()),
        ] {
            let result = Bond::new(
                Some(source("b")),
                correct.clone(),
                wrong,
                None,
                None,
                None,
                None,
                None,
            );
            assert!(matches!(result, Err(ModelError::VertexKindMismatch)));
        }
    }
    #[test]
    fn canonical_encoding_separates_delimiter_inputs() {
        assert_ne!(
            LegacyFingerprint::test_encoding(RecordKind::Atom, &["a:b".to_owned(), "c".to_owned()]),
            LegacyFingerprint::test_encoding(RecordKind::Atom, &["a".to_owned(), "b:c".to_owned()])
        );
    }
    #[test]
    fn exact_idless_duplicates_have_session_occurrences() {
        let first = atom(None, Some(0), 0.0);
        let second = atom(None, Some(1), 0.0);
        assert_ne!(first.identity(), second.identity());
        let model = molecule(vec![first, second], vec![], vec![], vec![], vec![])
            .expect("exact idless duplicates load with occurrences");
        assert_eq!(
            serde_json::from_str::<Molecule>(&serde_json::to_string(&model).expect("serialize"))
                .expect("deserialize"),
            model
        );
    }
    #[test]
    fn nonfinite_position_and_missing_bond_facts_reject_or_preserve() {
        assert!(Position::new(f64::NAN, 0.0, 0.0).is_err());
        let left = atom(Some("a"), None, 0.0);
        let right = atom(Some("b"), None, 1.0);
        let bond = bond(
            VertexRef::Atom(left.identity().clone()),
            VertexRef::Atom(right.identity().clone()),
            Some(0),
        );
        assert!(bond.source_type().is_none());
        assert!(bond.order().is_none());
        assert!(bond.style().is_none());
    }
    #[test]
    fn idless_bond_identity_includes_ordered_endpoint_identities() {
        let left = atom(None, Some(0), 0.0);
        let middle = atom(None, Some(0), 1.0);
        let right = atom(None, Some(0), 2.0);
        let first = bond(
            VertexRef::Atom(left.identity().clone()),
            VertexRef::Atom(middle.identity().clone()),
            Some(0),
        );
        let second = bond(
            VertexRef::Atom(left.identity().clone()),
            VertexRef::Atom(right.identity().clone()),
            Some(0),
        );
        assert_ne!(first.identity(), second.identity());
    }
    #[test]
    fn idless_molecule_anchor_includes_sorted_child_identities() {
        let left = atom(None, Some(0), 0.0);
        let right = atom(None, Some(0), 1.0);
        let reordered = Molecule::new(
            None,
            Some("same".to_owned()),
            vec![right.clone(), left.clone()],
            vec![],
            vec![],
            vec![],
            vec![],
            Some(0),
        )
        .expect("valid idless molecule");
        let original = Molecule::new(
            None,
            Some("same".to_owned()),
            vec![left.clone(), right.clone()],
            vec![],
            vec![],
            vec![],
            vec![],
            Some(0),
        )
        .expect("valid idless molecule");
        let distinct = Molecule::new(
            None,
            Some("same".to_owned()),
            vec![left],
            vec![],
            vec![],
            vec![],
            vec![],
            Some(0),
        )
        .expect("different child set is valid");
        assert_eq!(original.identity(), reordered.identity());
        assert_ne!(original.identity(), distinct.identity());
        let replacement = original
            .replace_records(
                Some("edited".to_owned()),
                vec![right],
                vec![],
                vec![],
                vec![],
                vec![],
            )
            .expect("replacement remains valid");
        assert_eq!(original.identity(), replacement.identity());
    }
    #[test]
    fn edited_idless_anchor_rehydrates_without_reseeding() {
        let first = atom(None, Some(0), 0.0);
        let second = atom(None, Some(0), 1.0);
        let edited_atom = first
            .replace_source_fields(
                Some("N".to_owned()),
                Position::new(2.0, 0.0, 0.0).expect("finite"),
                Some(1),
                None,
                None,
                None,
                None,
                None,
            )
            .expect("replacement is valid");
        let edited_bond = bond(
            VertexRef::Atom(first.identity().clone()),
            VertexRef::Atom(second.identity().clone()),
            Some(0),
        )
        .replace_source_fields(
            VertexRef::Atom(first.identity().clone()),
            VertexRef::Atom(second.identity().clone()),
            Some("n1".to_owned()),
            Some(BondOrder::Single),
            Some(BondStyle::Normal),
            None,
        )
        .expect("replacement is valid");
        let original = Molecule::new(
            None,
            Some("before".to_owned()),
            vec![first, second],
            vec![],
            vec![],
            vec![],
            vec![],
            Some(0),
        )
        .expect("valid idless molecule");
        let edited_molecule = original
            .replace_records(
                Some("after".to_owned()),
                vec![edited_atom.clone()],
                vec![],
                vec![],
                vec![],
                vec![],
            )
            .expect("replacement is valid");

        let restored_atom: Atom =
            serde_json::from_str(&serde_json::to_string(&edited_atom).expect("serialize"))
                .expect("rehydrate");
        assert_eq!(restored_atom, edited_atom);
        let restored_bond: Bond =
            serde_json::from_str(&serde_json::to_string(&edited_bond).expect("serialize"))
                .expect("rehydrate");
        assert_eq!(restored_bond, edited_bond);
        let restored_molecule: Molecule =
            serde_json::from_str(&serde_json::to_string(&edited_molecule).expect("serialize"))
                .expect("rehydrate");
        assert_eq!(restored_molecule, edited_molecule);
    }
    #[test]
    fn serde_rejects_spoofed_legacy_occurrence() {
        let atom = atom(None, Some(0), 0.0);
        let mut json = serde_json::to_value(&atom).expect("serialize");
        json["legacy_occurrence"] = serde_json::json!(1);
        assert!(serde_json::from_value::<Atom>(json).is_err());
    }
    #[test]
    fn serde_rejects_malformed_or_wrong_kind_legacy_anchors_everywhere() {
        let first_atom = atom(None, Some(0), 0.0);
        let other = atom(None, Some(0), 1.0);
        let bond = bond(
            VertexRef::Atom(first_atom.identity().clone()),
            VertexRef::Atom(other.identity().clone()),
            Some(0),
        );
        let molecule = Molecule::new(
            None,
            None,
            vec![first_atom.clone()],
            vec![],
            vec![],
            vec![],
            vec![],
            Some(0),
        )
        .expect("valid molecule");
        for bad in [
            "wrong-version",
            "ferrum-core-legacy-v1:Bond:1:x",
            "ferrum-core-legacy-v1:Atom:1:x:garbage",
        ] {
            let mut atom_json = serde_json::to_value(&first_atom).expect("serialize");
            atom_json["identity"]["origin"]["Legacy"]["fingerprint"] = serde_json::json!(bad);
            assert!(serde_json::from_value::<Atom>(atom_json).is_err());
            let mut bond_json = serde_json::to_value(&bond).expect("serialize");
            bond_json["identity"]["origin"]["Legacy"]["fingerprint"] = serde_json::json!(bad);
            assert!(serde_json::from_value::<Bond>(bond_json).is_err());
            let mut molecule_json = serde_json::to_value(&molecule).expect("serialize");
            molecule_json["identity"]["origin"]["Legacy"]["fingerprint"] = serde_json::json!(bad);
            assert!(serde_json::from_value::<Molecule>(molecule_json).is_err());
        }
        let wrong_kind =
            LegacyFingerprint::test_encoding(RecordKind::Bond, &vec!["x".to_owned(); 7]).0;
        let mut atom_json = serde_json::to_value(&first_atom).expect("serialize");
        atom_json["identity"]["origin"]["Legacy"]["fingerprint"] = serde_json::json!(wrong_kind);
        assert!(serde_json::from_value::<Atom>(atom_json).is_err());
        let mut nested = serde_json::to_value(&molecule).expect("serialize");
        nested["atoms"][0]["identity"]["origin"]["Legacy"]["fingerprint"] =
            serde_json::json!("ferrum-core-legacy-v1:Bond:1:x");
        assert!(serde_json::from_value::<Molecule>(nested).is_err());
    }
    #[test]
    fn standalone_record_and_vertex_deserialization_enforce_kinds() {
        let legacy_atom = atom(None, Some(0), 0.0).identity().clone();
        let source_atom = atom(Some("source"), None, 0.0).identity().clone();
        for identity in [&legacy_atom, &source_atom] {
            let restored: RecordId =
                serde_json::from_str(&serde_json::to_string(identity).expect("serialize"))
                    .expect("valid record identity");
            assert_eq!(restored, *identity);
        }
        let valid_vertex = VertexRef::Atom(legacy_atom.clone());
        let restored: VertexRef =
            serde_json::from_str(&serde_json::to_string(&valid_vertex).expect("serialize"))
                .expect("valid vertex reference");
        assert_eq!(restored, valid_vertex);

        let wrong_fingerprint =
            LegacyFingerprint::test_encoding(RecordKind::Bond, &vec!["x".to_owned(); 7]).0;
        let wrong_record = serde_json::json!({"kind":"Atom", "origin":{"Legacy":{"fingerprint":wrong_fingerprint, "occurrence":0}}});
        assert!(serde_json::from_value::<RecordId>(wrong_record).is_err());
        for (variant, kind, fields) in [
            ("Atom", RecordKind::Bond, 7usize),
            ("Group", RecordKind::Atom, 9usize),
            ("Text", RecordKind::Atom, 9usize),
            ("Query", RecordKind::Atom, 9usize),
        ] {
            let fingerprint =
                LegacyFingerprint::test_encoding(kind, &vec!["x".to_owned(); fields]).0;
            let identity = serde_json::json!({"kind": format!("{kind:?}"), "origin":{"Legacy":{"fingerprint":fingerprint, "occurrence":0}}});
            assert!(
                serde_json::from_value::<VertexRef>(serde_json::json!({variant: identity}))
                    .is_err()
            );
        }
    }
    proptest! {
        #[test] fn nonidentical_idless_identity_is_reorder_independent(x in -9999i64..9999, y in -9999i64..9999) { prop_assume!(x != y); let first = atom(None, Some(0), x as f64); let second = atom(None, Some(0), y as f64); let forward = molecule(vec![first.clone(), second.clone()], vec![], vec![], vec![], vec![]).expect("valid"); let reverse = molecule(vec![second, first], vec![], vec![], vec![], vec![]).expect("valid"); prop_assert!(forward.atoms().iter().all(|a| reverse.atoms().iter().any(|b| a.identity() == b.identity()))); }
        #[test]
        fn carried_optional_scalars_keep_absence_distinct_from_present_default(
            charge in proptest::option::weighted(0.5, -8i32..8),
            isotope in proptest::option::weighted(0.5, 1u16..300),
            hydrogens in proptest::option::weighted(0.5, 0u16..8),
            valence in proptest::option::weighted(0.5, 0u16..8),
            multiplicity in proptest::option::weighted(0.5, 1u16..4),
            free_sites in proptest::option::weighted(0.5, 0u16..8),
            order in proptest::option::weighted(0.5, prop_oneof![
                Just(BondOrder::Single),
                Just(BondOrder::Double),
                Just(BondOrder::Triple),
                Just(BondOrder::Aromatic),
                (0u8..4).prop_map(BondOrder::Other),
            ]),
            style in proptest::option::weighted(0.5, prop_oneof![
                Just(BondStyle::Normal),
                Just(BondStyle::Wedge),
                Just(BondStyle::Hashed),
                Just(BondStyle::Other("custom".to_owned())),
            ]),
            aromatic in proptest::option::weighted(0.5, any::<bool>()),
        ) {
            prop_assert!(!(charge.is_some() && isotope.is_some() && hydrogens.is_some() && valence.is_some() && multiplicity.is_some() && free_sites.is_some() && aromatic.is_some() && matches!(order, Some(BondOrder::Other(_))) && matches!(style, Some(BondStyle::Other(_)))));
            let origin = Position::new(0.0, 0.0, 0.0).expect("finite");
            let carrier = Atom::new(
                None, Some("C".to_owned()), origin, charge, isotope, hydrogens,
                valence, multiplicity, free_sites, Some(0),
            ).expect("valid atom");
            let partner = atom(None, Some(0), 1.0);
            let link = Bond::new(
                None,
                VertexRef::Atom(carrier.identity().clone()),
                VertexRef::Atom(partner.identity().clone()),
                None, order, style.clone(), aromatic, Some(0),
            ).expect("valid bond");
            let model = molecule(
                vec![carrier.clone(), partner.clone()], vec![], vec![], vec![], vec![link.clone()],
            ).expect("valid molecule");
            let restored: Molecule =
                serde_json::from_str(&serde_json::to_string(&model).expect("serialize"))
                    .expect("deserialize");

            // A round trip reproduces each carried option exactly, absence included.
            let restored_atom = &restored.atoms()[0];
            prop_assert_eq!(restored_atom.formal_charge(), charge);
            prop_assert_eq!(restored_atom.isotope(), isotope);
            prop_assert_eq!(restored_atom.explicit_hydrogens(), hydrogens);
            prop_assert_eq!(restored_atom.valence(), valence);
            prop_assert_eq!(restored_atom.multiplicity(), multiplicity);
            prop_assert_eq!(restored_atom.free_sites(), free_sites);
            prop_assert_eq!(restored_atom.identity(), carrier.identity());
            let restored_bond = &restored.bonds()[0];
            prop_assert_eq!(restored_bond.order(), order);
            prop_assert_eq!(restored_bond.style(), style.as_ref());
            prop_assert_eq!(restored_bond.aromatic(), aromatic);
            prop_assert_eq!(restored_bond.identity(), link.identity());

            // The same record with every absence filled by a present default is a
            // different record, and stays different across the same round trip.
            let filled_atom = Atom::new(
                None, Some("C".to_owned()), origin,
                Some(charge.unwrap_or(0)), Some(isotope.unwrap_or(0)),
                Some(hydrogens.unwrap_or(0)), Some(valence.unwrap_or(0)),
                Some(multiplicity.unwrap_or(1)), Some(free_sites.unwrap_or(0)), Some(0),
            ).expect("valid atom");
            // The atom and the bond vary in separate molecules, so a changed atom
            // identity cannot be what makes the compared bond identities differ.
            let filled_atom_model = molecule(
                vec![filled_atom, partner.clone()], vec![], vec![], vec![], vec![],
            ).expect("valid molecule");
            let restored_filled: Molecule =
                serde_json::from_str(&serde_json::to_string(&filled_atom_model).expect("serialize"))
                    .expect("deserialize");
            let restored_filled_atom = &restored_filled.atoms()[0];
            prop_assert!(restored_filled_atom.formal_charge().is_some());
            prop_assert!(restored_filled_atom.isotope().is_some());
            prop_assert!(restored_filled_atom.explicit_hydrogens().is_some());
            prop_assert!(restored_filled_atom.valence().is_some());
            prop_assert!(restored_filled_atom.multiplicity().is_some());
            prop_assert!(restored_filled_atom.free_sites().is_some());

            let filled_bond = Bond::new(
                None,
                VertexRef::Atom(carrier.identity().clone()),
                VertexRef::Atom(partner.identity().clone()),
                None,
                Some(order.unwrap_or(BondOrder::Single)),
                Some(style.clone().unwrap_or(BondStyle::Normal)),
                Some(aromatic.unwrap_or(false)),
                Some(0),
            ).expect("valid bond");
            let filled_bond_model = molecule(
                vec![carrier, partner], vec![], vec![], vec![], vec![filled_bond],
            ).expect("valid molecule");
            let restored_filled_bonds: Molecule =
                serde_json::from_str(&serde_json::to_string(&filled_bond_model).expect("serialize"))
                    .expect("deserialize");
            let restored_filled_bond = &restored_filled_bonds.bonds()[0];
            prop_assert!(restored_filled_bond.order().is_some());
            prop_assert!(restored_filled_bond.style().is_some());
            prop_assert!(restored_filled_bond.aromatic().is_some());
            let atom_absence = charge.is_none() || isotope.is_none() || hydrogens.is_none()
                || valence.is_none() || multiplicity.is_none() || free_sites.is_none();
            if atom_absence {
                prop_assert_ne!(restored_filled_atom.identity(), restored_atom.identity());
            }
            if order.is_none() || style.is_none() || aromatic.is_none() {
                prop_assert_ne!(restored_filled_bond.identity(), restored_bond.identity());
            }
        }
        #[test] fn endpoint_and_source_absence_round_trip(x in -9999i64..9999) { let a = atom(None, Some(0), x as f64); let g = vertex(RecordKind::Group, None, Some(0)); let q = vertex(RecordKind::Query, None, Some(0)); let b = bond(VertexRef::Group(g.identity().clone()), VertexRef::Query(q.identity().clone()), Some(0)); let model = molecule(vec![a], vec![g], vec![], vec![q], vec![b]).expect("typed endpoints resolve"); let restored: Molecule = serde_json::from_str(&serde_json::to_string(&model).expect("serialize")).expect("deserialize"); prop_assert!(restored.bonds()[0].source_id().is_none()); prop_assert!(!restored.bonds()[0].start().is_atom()); }
    }
}
