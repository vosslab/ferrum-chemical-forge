//! Closed, validated bond-properties intent for one atomic session operation.

use std::collections::HashSet;

use thiserror::Error;

use super::{DocumentBondPresentationV1, PersistentId, PositiveFiniteV1, Rgb24V1};
pub use ferrum_document_projection::NonZeroFiniteV1;

/// One supported durable bond-property change.
#[derive(Clone, Debug, PartialEq)]
pub enum BondPropertyChangeV1 {
    /// Replace the complete closed bond presentation.
    Presentation(DocumentBondPresentationV1),
    /// Replace or clear the authored centered-double-bond flag.
    Center(Option<bool>),
    /// Replace or clear the authored positive line width.
    LineWidth(Option<PositiveFiniteV1>),
    /// Replace or clear signed parallel-lane spacing.
    BondWidth(Option<NonZeroFiniteV1>),
    /// Replace or clear the authored positive wedge width.
    WedgeWidth(Option<PositiveFiniteV1>),
    /// Replace or clear the authored line colour.
    Color(Option<Rgb24V1>),
}

impl BondPropertyChangeV1 {
    fn kind(&self) -> BondPropertyKindV1 {
        match self {
            Self::Presentation(_) => BondPropertyKindV1::Presentation,
            Self::Center(_) => BondPropertyKindV1::Center,
            Self::LineWidth(_) => BondPropertyKindV1::LineWidth,
            Self::BondWidth(_) => BondPropertyKindV1::BondWidth,
            Self::WedgeWidth(_) => BondPropertyKindV1::WedgeWidth,
            Self::Color(_) => BondPropertyKindV1::Color,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum BondPropertyKindV1 {
    Presentation,
    Center,
    LineWidth,
    BondWidth,
    WedgeWidth,
    Color,
}

impl BondPropertyKindV1 {
    fn name(self) -> &'static str {
        match self {
            Self::Presentation => "presentation",
            Self::Center => "center",
            Self::LineWidth => "line width",
            Self::BondWidth => "bond width",
            Self::WedgeWidth => "wedge width",
            Self::Color => "color",
        }
    }
}

/// One validated, source-ID-targeted bond-properties patch.
#[derive(Clone, Debug, PartialEq)]
pub struct BondPropertiesPatchV1 {
    bond_id: PersistentId,
    changes: Vec<BondPropertyChangeV1>,
}

impl BondPropertiesPatchV1 {
    /// Validate one complete dialog intent without reading or changing a document.
    pub fn new(
        bond_id: impl Into<String>,
        changes: Vec<BondPropertyChangeV1>,
    ) -> Result<Self, BondPropertiesPatchV1Error> {
        let bond_id = PersistentId::new(bond_id.into())
            .map_err(|_| BondPropertiesPatchV1Error::InvalidBondId)?;
        let mut kinds = HashSet::with_capacity(changes.len());
        for change in &changes {
            let kind = change.kind();
            if !kinds.insert(kind) {
                return Err(BondPropertiesPatchV1Error::DuplicateChange {
                    property: kind.name(),
                });
            }
        }
        Ok(Self { bond_id, changes })
    }

    /// Return the durable authored bond identifier.
    #[must_use]
    pub fn bond_id(&self) -> &PersistentId {
        &self.bond_id
    }

    /// Return unique property changes in caller order.
    #[must_use]
    pub fn changes(&self) -> &[BondPropertyChangeV1] {
        &self.changes
    }
}

/// Invalid bond-properties intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum BondPropertiesPatchV1Error {
    /// The durable bond identifier is empty or otherwise invalid.
    #[error("bond properties require a valid persistent bond ID")]
    InvalidBondId,
    /// One closed property appeared more than once in a single patch.
    #[error("bond property change is duplicated: {property}")]
    DuplicateChange { property: &'static str },
}
