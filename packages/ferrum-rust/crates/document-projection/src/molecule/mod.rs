//! Immutable molecule-adjacent projection values.

mod atom;
mod bond;
mod compact_group;
mod mark;
mod stereo_depiction;

pub use atom::AtomProjectionV1;
pub use bond::{
    BondEndpointKindV1, BondEndpointV1, BondProjectionV1, DocumentHaworthPositionV1,
    MoleculeProjectionV1, MoleculeProjectionV1Error,
};
pub use compact_group::{
    CompactGroupAttachmentV1, CompactGroupCatalogKeyV1, CompactGroupProjectionV1, CompactGroupV1,
    CompactGroupV1Error,
};
pub use mark::{AtomMarkKindV1, AtomMarkProjectionV1};
pub use stereo_depiction::{
    DoubleBondCarrierMarkProjectionV1, DoubleBondCarrierMarkProjectionV1Error,
    DoubleBondCarrierMarkV1,
};
