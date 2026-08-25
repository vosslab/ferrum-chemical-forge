//! Immutable bracket-pair projection values and validated wire conversion.

use serde::{Deserialize, Deserializer, Serialize};

use super::{PresentationBracketStyleV1, PresentationStackProjectionV1Error};
use crate::{DocumentObjectIdV1, PositiveFiniteV1, Rgb24V1};
/// One structurally valid durable bracket pair.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BracketPairProjectionV1 {
    members: [DocumentObjectIdV1; 2],
    style: PresentationBracketStyleV1,
    line_width: Option<PositiveFiniteV1>,
    line_color: Option<Rgb24V1>,
}

impl<'de> Deserialize<'de> for BracketPairProjectionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BracketPairWireV1::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

impl BracketPairProjectionV1 {
    /// Construct one validated durable left/right bracket pair.
    pub fn try_new(
        members: [DocumentObjectIdV1; 2],
        style: PresentationBracketStyleV1,
        line_width: Option<PositiveFiniteV1>,
        line_color: Option<Rgb24V1>,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        if members[0] == members[1] {
            return Err(PresentationStackProjectionV1Error::InvalidBracketPair);
        }
        Ok(Self {
            members,
            style,
            line_width,
            line_color,
        })
    }
    /// Return left and right durable document object IDs in side order.
    #[must_use]
    pub fn members(&self) -> &[DocumentObjectIdV1; 2] {
        &self.members
    }

    /// Return the exact shared spline family.
    #[must_use]
    pub fn style(&self) -> PresentationBracketStyleV1 {
        self.style
    }

    /// Return the common resolved width, or `None` when the two sides differ.
    #[must_use]
    pub fn line_width(&self) -> Option<PositiveFiniteV1> {
        self.line_width
    }

    /// Return the common resolved colour, or `None` when the two sides differ.
    #[must_use]
    pub fn line_color(&self) -> Option<&Rgb24V1> {
        self.line_color.as_ref()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BracketPairWireV1 {
    pub members: [String; 2],
    pub style: PresentationBracketStyleV1,
    pub line_width: Option<f64>,
    pub line_color: Option<String>,
}

impl TryFrom<BracketPairWireV1> for BracketPairProjectionV1 {
    type Error = PresentationStackProjectionV1Error;

    fn try_from(value: BracketPairWireV1) -> Result<Self, Self::Error> {
        let [left, right] = value.members;
        let members = [
            DocumentObjectIdV1::parse(left)
                .map_err(|_| PresentationStackProjectionV1Error::InvalidBracketPair)?,
            DocumentObjectIdV1::parse(right)
                .map_err(|_| PresentationStackProjectionV1Error::InvalidBracketPair)?,
        ];
        let line_width = match value.line_width {
            Some(width) => Some(
                PositiveFiniteV1::new(width)
                    .ok_or(PresentationStackProjectionV1Error::InvalidBracketPair)?,
            ),
            None => None,
        };
        let line_color = match value.line_color {
            Some(color) => Some(
                Rgb24V1::new(color)
                    .ok_or(PresentationStackProjectionV1Error::InvalidBracketPair)?,
            ),
            None => None,
        };
        Self::try_new(members, value.style, line_width, line_color)
    }
}
