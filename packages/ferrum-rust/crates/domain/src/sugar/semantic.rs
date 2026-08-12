//! Immutable semantic values produced by the versioned compact importer.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

use super::syntax::{LegacyCompactSugarCodeV1Error, parse_legacy_compact_v1};

/// The oxidation family declared by the beginning of a sugar code.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum SugarPrefix {
    /// An aldose-style backbone.
    Aldo,
    /// A ketose-style backbone.
    Keto,
    /// A 3-ketose-style backbone.
    ThreeKeto,
}

/// The explicitly encoded carbohydrate series.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum SugarSeries {
    /// D-series configuration.
    D,
    /// L-series configuration.
    L,
    /// A code whose grammar explicitly selects the meso form.
    Meso,
}

/// A ring-size request made by a later depiction planner.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum RingForm {
    /// A six-membered carbohydrate ring.
    Pyranose,
    /// A five-membered carbohydrate ring.
    Furanose,
}

/// An explicit anomeric choice for a later depiction planner.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Anomer {
    /// Alpha anomer.
    Alpha,
    /// Beta anomer.
    Beta,
}

/// One validated backbone symbol.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum BackboneToken {
    /// Aldehyde-bearing terminus.
    Aldehyde,
    /// Keto-bearing carbon.
    Keto,
    /// Right-facing hydroxyl configuration marker.
    Right,
    /// Left-facing hydroxyl configuration marker.
    Left,
    /// D-series marker in the penultimate backbone position.
    DSeries,
    /// Hydroxymethyl terminus.
    Hydroxymethyl,
    /// Deoxy modification.
    Deoxy,
    /// Amino modification.
    Amino,
    /// N-acetyl modification.
    NAcetyl,
    /// Phosphate modification.
    Phosphate,
    /// Left-oriented phosphate modification.
    PhosphateLeft,
    /// Fluoro modification.
    Fluoro,
    /// Carboxyl modification.
    Carboxyl,
    /// A position marker with a separately typed footnote declaration.
    PositionMarker(u8),
}

impl BackboneToken {
    pub(crate) fn from_char(character: char, position: u8) -> Option<Self> {
        match character {
            'A' => Some(Self::Aldehyde),
            'K' => Some(Self::Keto),
            'R' => Some(Self::Right),
            'L' => Some(Self::Left),
            'D' => Some(Self::DSeries),
            'M' => Some(Self::Hydroxymethyl),
            'd' => Some(Self::Deoxy),
            'a' => Some(Self::Amino),
            'n' => Some(Self::NAcetyl),
            'p' => Some(Self::Phosphate),
            'P' => Some(Self::PhosphateLeft),
            'f' => Some(Self::Fluoro),
            'c' => Some(Self::Carboxyl),
            '1'..='9' if character.to_digit(10) == Some(u32::from(position)) => {
                Some(Self::PositionMarker(position))
            }
            _ => None,
        }
    }
}

/// A backbone token with its one-based position.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SugarPosition {
    /// One-based position in the compact code's backbone.
    pub position: u8,
    /// Typed meaning of the code character at `position`.
    pub token: BackboneToken,
}

/// The compatible declaration family for a position-marker footnote.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum FootnoteFamily {
    /// One whole-position substituent declaration.
    Plain,
    /// A carbon-state declaration.
    CarbonState,
    /// A left-side substituent declaration.
    Left,
    /// A right-side substituent declaration.
    Right,
}

impl FootnoteFamily {
    pub(crate) const fn suffix(self) -> &'static str {
        match self {
            Self::Plain => "",
            Self::CarbonState => "C",
            Self::Left => "L",
            Self::Right => "R",
        }
    }
}

/// A canonical position-marker footnote key.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct FootnoteKey {
    /// One-based marker position.
    pub position: u8,
    /// Semantically compatible declaration family.
    pub family: FootnoteFamily,
}

/// A validated compact code from the historical, version-one import format.
///
/// This is not a generic Ferrum sugar representation. Its deserializer accepts
/// only the canonical compact string and revalidates it through this type's
/// parser, so persisted data cannot bypass syntax or normalization rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyCompactSugarCodeV1 {
    body: String,
    prefix: SugarPrefix,
    series: SugarSeries,
    positions: Vec<SugarPosition>,
    footnotes: BTreeMap<FootnoteKey, String>,
}

impl LegacyCompactSugarCodeV1 {
    /// Parse and normalize one value from the historical compact v1 notation.
    pub fn parse(input: &str) -> Result<Self, LegacyCompactSugarCodeV1Error> {
        parse_legacy_compact_v1(input)
    }

    /// Return the normalized backbone body without the footnote block.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Return the declared oxidation family.
    #[must_use]
    pub const fn prefix(&self) -> SugarPrefix {
        self.prefix
    }

    /// Return the explicitly encoded D, L, or meso series.
    #[must_use]
    pub const fn series(&self) -> SugarSeries {
        self.series
    }

    /// Return the immutable, one-based backbone description.
    #[must_use]
    pub fn positions(&self) -> &[SugarPosition] {
        &self.positions
    }

    /// Return normalized footnotes, including a completed `H` side when needed.
    #[must_use]
    pub fn footnotes(&self) -> &BTreeMap<FootnoteKey, String> {
        &self.footnotes
    }

    /// Return a deterministic, whitespace-free code suitable for storage.
    #[must_use]
    pub fn canonical_code(&self) -> String {
        if self.footnotes.is_empty() {
            return self.body.clone();
        }
        let entries = self
            .footnotes
            .iter()
            .map(|(key, value)| format!("{}{}={value}", key.position, key.family.suffix()))
            .collect::<Vec<_>>();
        format!("{}[{}]", self.body, entries.join(","))
    }

    pub(crate) fn new(
        body: String,
        prefix: SugarPrefix,
        series: SugarSeries,
        positions: Vec<SugarPosition>,
        footnotes: BTreeMap<FootnoteKey, String>,
    ) -> Self {
        Self {
            body,
            prefix,
            series,
            positions,
            footnotes,
        }
    }
}

impl Serialize for LegacyCompactSugarCodeV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.canonical_code())
    }
}

impl<'de> Deserialize<'de> for LegacyCompactSugarCodeV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let compact_code = String::deserialize(deserializer)?;
        Self::parse(&compact_code).map_err(D::Error::custom)
    }
}

/// Explicit renderer input whose code uses the historical compact v1 importer.
///
/// It deliberately contains no SMILES and no mutable molecule.  A later
/// bridge converts it to the distinct `haworth` module's topology request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LegacyCompactSugarRenderRequestV1 {
    /// Validated structural syntax.
    pub code: LegacyCompactSugarCodeV1,
    /// User-selected ring family.
    pub ring: RingForm,
    /// User-selected anomer.
    pub anomer: Anomer,
}

impl LegacyCompactSugarRenderRequestV1 {
    /// Create an explicit render request without invoking a molecular codec.
    #[must_use]
    pub const fn new(code: LegacyCompactSugarCodeV1, ring: RingForm, anomer: Anomer) -> Self {
        Self { code, ring, anomer }
    }
}
