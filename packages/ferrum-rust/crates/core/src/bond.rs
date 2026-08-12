use serde::{Deserialize, Serialize};

use crate::{
    Identifier, LegacyFingerprint, ModelError, RecordId, RecordKind, RecordOrigin, VertexRef,
    formatting::{option_debug, option_number, option_text},
};

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
