use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ModelError;

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
pub struct LegacyFingerprint(pub(crate) String);

impl LegacyFingerprint {
    pub(crate) fn new(kind: RecordKind, fields: &[String]) -> Self {
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
    pub(crate) fn test_encoding(kind: RecordKind, fields: &[String]) -> Self {
        Self::new(kind, fields)
    }

    pub(crate) fn kind(&self) -> Result<RecordKind, ModelError> {
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
    pub(crate) kind: RecordKind,
    pub(crate) origin: RecordOrigin,
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

    pub(crate) fn from_legacy(
        kind: RecordKind,
        fingerprint: LegacyFingerprint,
        occurrence: u32,
    ) -> Self {
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

    pub(crate) fn canonical(&self) -> String {
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
