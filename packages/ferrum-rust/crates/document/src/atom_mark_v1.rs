//! Closed atom-mark operation and projection facts.

use serde::Serialize;

use super::PositiveFiniteV1;

/// One supported authored atom-mark kind.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AtomMarkKindV1 {
    /// Circled positive charge annotation.
    Plus,
    /// Circled negative charge annotation.
    Minus,
    /// One unpaired-electron dot.
    Radical,
    /// Two unpaired-electron dots.
    Biradical,
    /// Lone pair rendered as one line.
    Electronpair,
    /// Lone pair rendered as two dots.
    DottedElectronpair,
    /// Two-lobed p-orbital annotation.
    PzOrbital,
}

impl AtomMarkKindV1 {
    /// Parse the exact supported CDML type spelling.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "plus" => Some(Self::Plus),
            "minus" => Some(Self::Minus),
            "radical" => Some(Self::Radical),
            "biradical" => Some(Self::Biradical),
            "electronpair" => Some(Self::Electronpair),
            "dotted_electronpair" => Some(Self::DottedElectronpair),
            "pz_orbital" => Some(Self::PzOrbital),
            _ => None,
        }
    }

    /// Return the exact authored CDML type spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Plus => "plus",
            Self::Minus => "minus",
            Self::Radical => "radical",
            Self::Biradical => "biradical",
            Self::Electronpair => "electronpair",
            Self::DottedElectronpair => "dotted_electronpair",
            Self::PzOrbital => "pz_orbital",
        }
    }
}

/// Exact intent for one atom-mark operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AtomMarkActionV1 {
    /// Append one new authored mark.
    Add,
    /// Remove one matching authored mark.
    Remove,
}

/// Normalized immutable display and operation facts for one supported direct mark.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AtomMarkProjectionV1 {
    pub(crate) kind: AtomMarkKindV1,
    pub(crate) source_order: u32,
    pub(crate) same_type_ordinal: u32,
    pub(crate) angle_degrees: f64,
    pub(crate) radial_offset: f64,
    pub(crate) size: PositiveFiniteV1,
    pub(crate) draw_circle: bool,
    pub(crate) line_width: PositiveFiniteV1,
}

impl AtomMarkProjectionV1 {
    /// Return the closed mark kind.
    #[must_use]
    pub fn kind(&self) -> AtomMarkKindV1 {
        self.kind
    }
    /// Return this mark's direct atom-child position.
    #[must_use]
    pub fn source_order(&self) -> u32 {
        self.source_order
    }
    /// Return its zero-based ordinal among direct marks of this exact kind.
    #[must_use]
    pub fn same_type_ordinal(&self) -> u32 {
        self.same_type_ordinal
    }
    /// Return the normalized atom-to-mark angle in degrees.
    #[must_use]
    pub fn angle_degrees(&self) -> f64 {
        self.angle_degrees
    }
    /// Return the normalized atom-to-mark distance in scene points.
    #[must_use]
    pub fn radial_offset(&self) -> f64 {
        self.radial_offset
    }
    /// Return the normalized positive mark size in scene points.
    #[must_use]
    pub fn size(&self) -> PositiveFiniteV1 {
        self.size
    }
    /// Return whether a charge mark requests its surrounding circle.
    #[must_use]
    pub fn draw_circle(&self) -> bool {
        self.draw_circle
    }
    /// Return the normalized positive stroke width in scene points.
    #[must_use]
    pub fn line_width(&self) -> PositiveFiniteV1 {
        self.line_width
    }
}
