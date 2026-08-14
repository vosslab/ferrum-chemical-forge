//! Closed, validated bond-properties intent for one atomic session operation.

use std::collections::HashSet;

use serde::Serialize;
use thiserror::Error;

use super::{DocumentBondOrderV1, PersistentId, PositiveFiniteV1, Rgb24V1};

/// A finite scalar whose sign is meaningful and whose magnitude is nonzero.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct NonZeroFiniteV1(f64);

impl NonZeroFiniteV1 {
    /// Construct a finite nonzero scalar without discarding its sign.
    pub fn new(value: f64) -> Option<Self> {
        (value.is_finite() && value != 0.0).then_some(Self(value))
    }

    /// Return the carried signed scalar.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

/// A closed CDML bond depiction prefix supported by the V1 editor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentBondStyleV1 {
    /// Normal covalent depiction (`n`).
    Normal,
    /// Filled wedge (`w`).
    Wedge,
    /// Hashed wedge (`h`).
    HashedWedge,
    /// Adder (`a`).
    Adder,
    /// Bold (`b`).
    Bold,
    /// Dashed (`d`).
    Dashed,
    /// Dotted (`o`).
    Dotted,
    /// Wavy (`s`).
    Wavy,
    /// Haworth front (`q`).
    HaworthFront,
}

impl DocumentBondStyleV1 {
    /// Return whether this closed depiction style can author the supplied order.
    ///
    /// The CDML V1 grammar makes Haworth front a depiction token rather than a
    /// generic bond-order prefix: only `q1` is authored. Other closed styles
    /// retain the existing V1 orders until the format specifies otherwise.
    pub(crate) const fn supports_order(self, order: DocumentBondOrderV1) -> bool {
        !matches!(
            (self, order),
            (
                Self::HaworthFront,
                DocumentBondOrderV1::Double | DocumentBondOrderV1::Triple
            )
        )
    }

    pub(crate) const fn cdml_prefix(self) -> char {
        match self {
            Self::Normal => 'n',
            Self::Wedge => 'w',
            Self::HashedWedge => 'h',
            Self::Adder => 'a',
            Self::Bold => 'b',
            Self::Dashed => 'd',
            Self::Dotted => 'o',
            Self::Wavy => 's',
            Self::HaworthFront => 'q',
        }
    }

    pub(crate) const fn from_cdml_prefix(value: char) -> Option<Self> {
        match value {
            'n' => Some(Self::Normal),
            'w' => Some(Self::Wedge),
            'h' => Some(Self::HashedWedge),
            'a' => Some(Self::Adder),
            'b' => Some(Self::Bold),
            'd' => Some(Self::Dashed),
            'o' => Some(Self::Dotted),
            's' => Some(Self::Wavy),
            'q' => Some(Self::HaworthFront),
            _ => None,
        }
    }
}

/// One supported durable bond-property change.
#[derive(Clone, Debug, PartialEq)]
pub enum BondPropertyChangeV1 {
    /// Replace the bond order while retaining the source style.
    Order(DocumentBondOrderV1),
    /// Replace the bond style while retaining the source order.
    Style(DocumentBondStyleV1),
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
            Self::Order(_) => BondPropertyKindV1::Order,
            Self::Style(_) => BondPropertyKindV1::Style,
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
    Order,
    Style,
    Center,
    LineWidth,
    BondWidth,
    WedgeWidth,
    Color,
}

impl BondPropertyKindV1 {
    fn name(self) -> &'static str {
        match self {
            Self::Order => "order",
            Self::Style => "style",
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
        let order = changes.iter().find_map(|change| match change {
            BondPropertyChangeV1::Order(value) => Some(*value),
            _ => None,
        });
        let style = changes.iter().find_map(|change| match change {
            BondPropertyChangeV1::Style(value) => Some(*value),
            _ => None,
        });
        if let (Some(order), Some(style)) = (order, style)
            && !style.supports_order(order)
        {
            return Err(BondPropertiesPatchV1Error::UnsupportedStyleOrder);
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
    /// The supplied closed style/order pair has no authored V1 CDML form.
    #[error("Haworth front bond style can only be authored with single order")]
    UnsupportedStyleOrder,
}
