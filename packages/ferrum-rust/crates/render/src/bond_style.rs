//! Closed bond-style vocabulary shared by independent renderer lowerers.

/// Bond style carried by the source projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BondStyle {
    /// The sole supported bond style in this vertical slice.
    NormalSingle,
    /// An ordinary single carrier bond with an explicit positive-normal E/Z accent.
    DoubleBondCarrierUp,
    /// An ordinary single carrier bond with an explicit negative-normal E/Z accent.
    DoubleBondCarrierDown,
    /// A parallel double bond.
    Double,
    /// A parallel triple bond.
    Triple,
    /// An aromatic bond.
    Aromatic,
    /// A solid stereochemical wedge.
    SolidWedge,
    /// A hashed stereochemical wedge.
    HashedWedge,
    /// A declared `q1` front edge in a Haworth depiction.
    HaworthFrontStroke,
    /// A declared directed `w1` front shoulder in a Haworth depiction.
    HaworthFrontWedge,
    /// A dashed bond.
    Dashed,
    /// An exact source depiction that V1 intentionally cannot lower.
    Unsupported { detail: String },
}

impl BondStyle {
    pub(crate) fn unsupported_name(&self) -> Option<&str> {
        match self {
            Self::NormalSingle
            | Self::DoubleBondCarrierUp
            | Self::DoubleBondCarrierDown
            | Self::Double
            | Self::Triple
            | Self::SolidWedge
            | Self::HashedWedge
            | Self::HaworthFrontStroke
            | Self::HaworthFrontWedge => None,
            Self::Aromatic => Some("aromatic bond"),
            Self::Dashed => Some("dashed bond"),
            Self::Unsupported { detail } => Some(detail.as_str()),
        }
    }
}
