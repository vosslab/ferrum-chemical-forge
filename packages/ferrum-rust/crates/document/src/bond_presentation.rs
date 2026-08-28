//! Closed native authoring vocabulary for persistent bond presentation.

use super::DocumentBondOrderV1;
use ferrum_core::{BondOrder, BondStyle};

/// One persistent CDML bond presentation accepted by the document authority.
///
/// Every non-normal variant is intrinsically a single bond. The closed type is
/// shared by insertion and property mutation so invalid style/order pairs have
/// no document-owned representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentBondPresentationV1 {
    /// A normal bond with one of the supported covalent orders.
    Normal(DocumentBondOrderV1),
    /// A solid wedge from the first endpoint (tip) to the second (base).
    SolidWedge,
    /// A hashed wedge from the first endpoint (tip) to the second (base).
    HashedWedge,
    /// A Haworth-front single-bond depiction.
    HaworthFront,
    /// A bold single-bond depiction.
    Bold,
    /// A dashed single-bond depiction.
    Dashed,
    /// A wavy single-bond depiction.
    Wavy,
}

/// Project one current-format source token into observable core semantics.
///
/// This decoder is intentionally broader than [`DocumentBondPresentationV1`]:
/// the document authority authors only the closed presentation vocabulary,
/// while read-only projection must preserve unsupported source facts for
/// diagnostics and typed refusal. Version-0.8's bare `s` and `d` compatibility
/// tokens are handled by the callers that own that historical format rule.
#[must_use]
pub(crate) fn project_source_bond_semantics(value: &str) -> (Option<BondOrder>, Option<BondStyle>) {
    let Some(digits) = value.get(1..) else {
        return (None, None);
    };
    let order = if digits.is_empty() {
        None
    } else {
        let Ok(number) = digits.parse::<u8>() else {
            return (None, None);
        };
        Some(match number {
            1 => BondOrder::Single,
            2 => BondOrder::Double,
            3 => BondOrder::Triple,
            4 => BondOrder::Aromatic,
            other => BondOrder::Other(other),
        })
    };
    let style = value.chars().next().map(|character| match character {
        'n' => BondStyle::Normal,
        'w' => BondStyle::Wedge,
        'h' | 'l' | 'r' => BondStyle::Hashed,
        'a' => BondStyle::Adder,
        'b' => BondStyle::Bold,
        'd' => BondStyle::Dashed,
        'o' => BondStyle::Dotted,
        's' => BondStyle::Wavy,
        'q' => BondStyle::HaworthFront,
        other => BondStyle::Other(other.to_string()),
    });
    (order, style)
}

impl DocumentBondPresentationV1 {
    /// Return the exact closed CDML token authored by this presentation.
    #[must_use]
    pub(crate) const fn cdml_token(self) -> &'static str {
        match self {
            Self::Normal(order) => order.cdml_token(),
            Self::SolidWedge => "w1",
            Self::HashedWedge => "h1",
            Self::HaworthFront => "q1",
            Self::Bold => "b1",
            Self::Dashed => "d1",
            Self::Wavy => "s1",
        }
    }

    /// Decode one closed CDML bond token through the durable presentation model.
    #[must_use]
    pub(crate) const fn from_cdml_token(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"n1" => Some(Self::Normal(DocumentBondOrderV1::Single)),
            b"n2" => Some(Self::Normal(DocumentBondOrderV1::Double)),
            b"n3" => Some(Self::Normal(DocumentBondOrderV1::Triple)),
            b"w1" => Some(Self::SolidWedge),
            b"h1" => Some(Self::HashedWedge),
            b"q1" => Some(Self::HaworthFront),
            b"b1" => Some(Self::Bold),
            b"d1" => Some(Self::Dashed),
            b"s1" => Some(Self::Wavy),
            _ => None,
        }
    }
}
