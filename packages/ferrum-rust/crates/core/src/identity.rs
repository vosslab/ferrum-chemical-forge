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
        let identifier = Self(value.into());
        identifier.validate()?;
        Ok(identifier)
    }

    fn validate(&self) -> Result<(), InvalidIdentifier> {
        if self.0.trim().is_empty() {
            return Err(InvalidIdentifier { kind: "source" });
        }
        Ok(())
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

/// A source-only internal record locator.
///
/// This identity is intentionally separate from document-owned
/// `DocumentObjectIdV1`, which identifies durable interaction targets.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RecordId {
    pub(crate) kind: RecordKind,
    pub(crate) source_id: Identifier,
}

#[derive(Deserialize)]
struct WireRecordId {
    kind: RecordKind,
    source_id: Identifier,
}

impl<'de> Deserialize<'de> for RecordId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = WireRecordId::deserialize(deserializer)?;
        Self::new(wire.kind, wire.source_id).map_err(serde::de::Error::custom)
    }
}

impl RecordId {
    /// Construct a source-only record locator from an explicit nonblank ID.
    pub fn new(kind: RecordKind, source_id: Identifier) -> Result<Self, InvalidIdentifier> {
        source_id.validate()?;
        Ok(Self { kind, source_id })
    }

    /// Return the record class encoded in this identity.
    #[must_use]
    pub fn kind(&self) -> RecordKind {
        self.kind
    }

    /// Return the exact typed-source locator.
    #[must_use]
    pub fn source_id(&self) -> &Identifier {
        &self.source_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_id_serde_refuses_blank_source_id() {
        let wire = serde_json::json!({"kind": "Atom", "source_id": "  "});
        assert!(serde_json::from_value::<RecordId>(wire).is_err());
    }
}
