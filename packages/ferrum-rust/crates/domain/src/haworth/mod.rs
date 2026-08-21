//! Typed, pure Haworth-projection planning.

mod authoring_receipt;
mod direct_glycosidic;
mod direct_glycosidic_fragment;
mod direct_glycosidic_layout;
mod direct_glycosidic_spec;
mod durable_reobservation;
mod layout;
mod placement;
mod smiles;
mod standalone_glucose_recipe_v1;
mod tree;
mod types;
mod validate;
mod wire_validation;

pub use authoring_receipt::{
    direct_glycosidic_haworth_authoring_receipt_v1, AuthoredDirectGlycosidicHaworthBondRoleV1,
    AuthoredDirectGlycosidicHaworthBridgeBondV1, AuthoredDirectGlycosidicHaworthCanonicalAtomV1,
    AuthoredDirectGlycosidicHaworthCanonicalBondV1, AuthoredDirectGlycosidicHaworthDepictionV1,
    AuthoredDirectGlycosidicHaworthRingBondV1, AuthoredDirectGlycosidicHaworthRingV1,
    DirectGlycosidicHaworthAuthoringAtomElementV1, DirectGlycosidicHaworthAuthoringReceiptV1,
    DirectGlycosidicHaworthSelectedAtomFactV1, DirectGlycosidicHaworthSelectedBondFactV1,
};
pub use direct_glycosidic::{
    DirectGlycosidicBridgeV1, DirectGlycosidicHaworthTopologyV1, DirectGlycosidicRingV1,
};
pub use direct_glycosidic_fragment::{
    assemble_direct_glycosidic_haworth_fragment_v1, DirectGlycosidicHaworthFragmentRequestV1,
    DirectGlycosidicHaworthFragmentV1,
};
pub use direct_glycosidic_layout::{
    layout_direct_glycosidic_haworth_v1, DirectGlycosidicHaworthLayoutRequestV1,
    DirectGlycosidicHaworthLayoutV1,
};
pub use direct_glycosidic_spec::{
    direct_glycosidic_haworth_depiction_spec_v1, DirectGlycosidicHaworthBondStyleV1,
    DirectGlycosidicHaworthBridgeBondSpecV1, DirectGlycosidicHaworthDepictionSpecV1,
    DirectGlycosidicHaworthPositionV1, DirectGlycosidicHaworthRingBondSpecV1,
    DirectGlycosidicHaworthRingSpecV1,
};
pub use durable_reobservation::{
    authored_direct_glycosidic_haworth_depiction_from_durable_profile_v1,
    DurableDirectGlycosidicHaworthAtomFactV1, DurableDirectGlycosidicHaworthBondFactV1,
    DurableDirectGlycosidicHaworthProfileV1, DurableDirectGlycosidicHaworthRingFactV1,
};
pub use layout::layout_single_ring;
pub use smiles::{
    build_direct_haworth_from_smiles_v1, build_direct_haworth_from_text_smiles_v1,
    DirectHaworthFromSmilesBuildErrorV1, PreparedDirectHaworthFromSmilesV1,
};
pub use standalone_glucose_recipe_v1::{
    standalone_d_glucose_haworth_recipe_v1, StandaloneDGlucoseHaworthErrorV1,
    StandaloneDGlucoseHaworthReceiptV1, StandaloneDGlucoseHaworthRecipeV1, StandaloneHaworthAtomV1,
    StandaloneHaworthBondTokenV1, StandaloneHaworthBondV1, StandaloneHaworthPositionV1,
};
pub use tree::{
    layout_tree, GlycosidicLink, HaworthAttachment, HaworthFragment, HaworthLinkGeometry,
    HaworthLinkTopology, HaworthRingNode, HaworthRingTopology, HaworthTreeRequest, MAX_TREE_RINGS,
};
pub use types::{
    BondDepiction, CanonicalOrientation, Face, HaworthDepiction, HaworthError,
    HaworthLayoutRequest, HaworthPoint, HaworthTopology, HaworthTopologyBuilder, HaworthVertex,
    RingForm, WedgeEdgeRole,
};

#[cfg(test)]
mod tests;
