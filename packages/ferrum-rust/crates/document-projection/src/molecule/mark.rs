//! Immutable display facts for one atom-owned mark.

use serde::Serialize;

use crate::PositiveFiniteV1;

/// One supported authored atom-mark kind.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AtomMarkKindV1 {
    Plus,
    Minus,
    Radical,
    Biradical,
    Electronpair,
    DottedElectronpair,
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

/// Normalized immutable display facts for one supported direct mark.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AtomMarkProjectionV1 {
    kind: AtomMarkKindV1,
    source_order: u32,
    same_type_ordinal: u32,
    angle_degrees: f64,
    radial_offset: f64,
    size: PositiveFiniteV1,
    draw_circle: bool,
    line_width: PositiveFiniteV1,
}

impl AtomMarkProjectionV1 {
    /// Construct one finite, nonnegative atom-mark placement from typed facts.
    #[expect(
        clippy::too_many_arguments,
        reason = "each immutable atom-mark fact remains explicit at the projection boundary"
    )]
    #[must_use]
    pub fn new(
        kind: AtomMarkKindV1,
        source_order: u32,
        same_type_ordinal: u32,
        angle_degrees: f64,
        radial_offset: f64,
        size: PositiveFiniteV1,
        draw_circle: bool,
        line_width: PositiveFiniteV1,
    ) -> Option<Self> {
        (angle_degrees.is_finite() && radial_offset.is_finite() && radial_offset >= 0.0).then_some(
            Self {
                kind,
                source_order,
                same_type_ordinal,
                angle_degrees,
                radial_offset,
                size,
                draw_circle,
                line_width,
            },
        )
    }

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

#[cfg(test)]
mod tests {
    use super::{AtomMarkKindV1, AtomMarkProjectionV1};
    use crate::PositiveFiniteV1;

    #[test]
    fn atom_mark_constructor_refuses_nonfinite_or_negative_geometry() {
        let positive = PositiveFiniteV1::new(1.0).expect("one is positive and finite");
        assert!(
            AtomMarkProjectionV1::new(
                AtomMarkKindV1::Plus,
                0,
                0,
                f64::NAN,
                0.0,
                positive,
                true,
                positive,
            )
            .is_none()
        );
        assert!(
            AtomMarkProjectionV1::new(
                AtomMarkKindV1::Plus,
                0,
                0,
                0.0,
                -1.0,
                positive,
                true,
                positive,
            )
            .is_none()
        );
    }
}
