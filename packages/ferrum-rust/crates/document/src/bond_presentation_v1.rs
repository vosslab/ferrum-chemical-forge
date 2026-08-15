//! Closed native authoring vocabulary for persistent bond presentation.

use super::DocumentBondOrderV1;

/// One presentation that ordinary native bond creation can persist faithfully.
///
/// The directed variants always write a single bond.  Their caller-supplied
/// endpoint order is therefore retained as the CDML tip-to-base direction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentBondPresentationV1 {
    /// A normal bond with one of the supported covalent orders.
    Normal(DocumentBondOrderV1),
    /// A solid wedge from the first endpoint (tip) to the second (base).
    SolidWedge,
    /// A hashed wedge from the first endpoint (tip) to the second (base).
    HashedWedge,
}

impl DocumentBondPresentationV1 {
    /// Return the exact closed CDML token authored by this presentation.
    #[must_use]
    pub(crate) const fn cdml_token(self) -> &'static str {
        match self {
            Self::Normal(order) => order.cdml_token(),
            Self::SolidWedge => "w1",
            Self::HashedWedge => "h1",
        }
    }
}
