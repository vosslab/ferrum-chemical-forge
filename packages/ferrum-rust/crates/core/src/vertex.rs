use serde::{Deserialize, Serialize};

use crate::{Identifier, ModelError, RecordId, RecordKind};

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
        let value = match WireVertexRef::deserialize(deserializer)? {
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
    source_id: Identifier,
}
impl NonAtomVertex {
    /// Construct a source-identified non-atom vertex.
    pub fn new(kind: RecordKind, source_id: Identifier) -> Result<Self, ModelError> {
        if !matches!(
            kind,
            RecordKind::Group | RecordKind::Text | RecordKind::Query
        ) {
            return Err(ModelError::InvalidVertexKind { kind });
        }
        let identity = RecordId::new(kind, source_id.clone())
            .map_err(|_| ModelError::InvalidSourceIdentity { kind })?;
        Ok(Self {
            identity,
            source_id,
        })
    }
    pub(crate) fn validate(&self, kind: RecordKind) -> Result<(), ModelError> {
        if self.identity.kind() == kind && self.identity.source_id() == &self.source_id {
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
    /// Return its required literal source ID.
    #[must_use]
    pub fn source_id(&self) -> &Identifier {
        &self.source_id
    }
}
#[derive(Deserialize)]
struct WireNonAtomVertex {
    identity: RecordId,
    source_id: Identifier,
}
impl<'de> Deserialize<'de> for NonAtomVertex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = WireNonAtomVertex::deserialize(deserializer)?;
        let result =
            Self::new(wire.identity.kind(), wire.source_id).map_err(serde::de::Error::custom)?;
        if result.identity != wire.identity {
            return Err(serde::de::Error::custom(
                "vertex identity does not match carried source fields",
            ));
        }
        Ok(result)
    }
}
