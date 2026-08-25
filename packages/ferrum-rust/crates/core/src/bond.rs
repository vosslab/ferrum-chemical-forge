use serde::{Deserialize, Serialize};

use crate::{Identifier, ModelError, RecordId, RecordKind, VertexRef};

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

/// A validated bond with typed, ordered endpoints and source-type presence.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Bond {
    identity: RecordId,
    source_id: Identifier,
    start: VertexRef,
    end: VertexRef,
    source_type: Option<String>,
    order: Option<BondOrder>,
    style: Option<BondStyle>,
    aromatic: Option<bool>,
}
impl Bond {
    /// Construct a bond from its required typed-source locator.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_id: Identifier,
        start: VertexRef,
        end: VertexRef,
        source_type: Option<String>,
        order: Option<BondOrder>,
        style: Option<BondStyle>,
        aromatic: Option<bool>,
    ) -> Result<Self, ModelError> {
        let identity = RecordId::new(RecordKind::Bond, source_id.clone()).map_err(|_| {
            ModelError::InvalidSourceIdentity {
                kind: RecordKind::Bond,
            }
        })?;
        let bond = Self {
            identity,
            source_id,
            start,
            end,
            source_type,
            order,
            style,
            aromatic,
        };
        bond.validate()?;
        Ok(bond)
    }
    pub(crate) fn validate(&self) -> Result<(), ModelError> {
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
        if self.identity.kind() == RecordKind::Bond && self.identity.source_id() == &self.source_id
        {
            Ok(())
        } else {
            Err(ModelError::IdentityMismatch {
                kind: RecordKind::Bond,
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
    /// Return a validated immutable replacement retaining this source locator.
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
        };
        replacement.validate()?;
        Ok(replacement)
    }
}
#[derive(Deserialize)]
struct WireBond {
    identity: RecordId,
    source_id: Identifier,
    start: VertexRef,
    end: VertexRef,
    source_type: Option<String>,
    order: Option<BondOrder>,
    style: Option<BondStyle>,
    aromatic: Option<bool>,
}
impl<'de> Deserialize<'de> for Bond {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = WireBond::deserialize(deserializer)?;
        let bond = Self {
            identity: wire.identity,
            source_id: wire.source_id,
            start: wire.start,
            end: wire.end,
            source_type: wire.source_type,
            order: wire.order,
            style: wire.style,
            aromatic: wire.aromatic,
        };
        bond.validate().map_err(serde::de::Error::custom)?;
        Ok(bond)
    }
}
