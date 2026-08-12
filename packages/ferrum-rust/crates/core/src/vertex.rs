use serde::{Deserialize, Serialize};

use crate::{
    Identifier, LegacyFingerprint, ModelError, RecordId, RecordKind, formatting::option_text,
};

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
    pub(crate) fn canonical(&self) -> String {
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
    pub(crate) fn validate_kind(&self) -> Result<(), ModelError> {
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
    pub(crate) fn validate(&self, kind: RecordKind) -> Result<(), ModelError> {
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
