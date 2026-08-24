//! Closed document-owned values for compact known-group records.
//!
//! Compact groups are authored document objects, not chemistry graph vertices.
//! Their immutable catalog facts stay in this module so projection, future
//! placement, and later materialization use one definition table.

pub use ferrum_document_projection::{
    CompactGroupAttachmentV1, CompactGroupCatalogKeyV1, CompactGroupV1, CompactGroupV1Error,
};

/// One closed atom fact in the internal compact-group materialization catalog.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CompactGroupMaterializedAtomV1 {
    pub(crate) element: &'static str,
    pub(crate) formal_charge: i8,
    pub(crate) explicit_hydrogens: u16,
    pub(crate) local_x: f64,
    pub(crate) local_y: f64,
}

/// One closed internal bond fact in the internal compact-group materialization catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CompactGroupMaterializedBondV1 {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) cdml_type: &'static str,
}

/// Immutable Rust-owned materialization facts for the bounded experiment.
///
/// The first index is always the exterior attachment atom. Later catalog entries
/// remain unavailable until their complete chemistry and rendering contracts are
/// deliberately admitted rather than inferred from a label.
pub(crate) struct CompactGroupMaterializationDefinitionV1 {
    pub(crate) atoms: &'static [CompactGroupMaterializedAtomV1],
    pub(crate) bonds: &'static [CompactGroupMaterializedBondV1],
}

const ME_ATOMS: [CompactGroupMaterializedAtomV1; 1] = [CompactGroupMaterializedAtomV1 {
    element: "C",
    formal_charge: 0,
    explicit_hydrogens: 3,
    local_x: 0.0,
    local_y: 0.0,
}];
const NO2_ATOMS: [CompactGroupMaterializedAtomV1; 3] = [
    CompactGroupMaterializedAtomV1 {
        element: "N",
        formal_charge: 1,
        explicit_hydrogens: 0,
        local_x: 0.0,
        local_y: 0.0,
    },
    CompactGroupMaterializedAtomV1 {
        element: "O",
        formal_charge: 0,
        explicit_hydrogens: 0,
        local_x: 24.0,
        local_y: 12.0,
    },
    CompactGroupMaterializedAtomV1 {
        element: "O",
        formal_charge: -1,
        explicit_hydrogens: 0,
        local_x: 24.0,
        local_y: -12.0,
    },
];
const NO2_BONDS: [CompactGroupMaterializedBondV1; 2] = [
    CompactGroupMaterializedBondV1 {
        start: 0,
        end: 1,
        cdml_type: "n2",
    },
    CompactGroupMaterializedBondV1 {
        start: 0,
        end: 2,
        cdml_type: "n1",
    },
];

/// Return the closed recipe admitted by the internal materialization experiment.
#[must_use]
pub(crate) const fn materialization_definition_v1(
    catalog_key: CompactGroupCatalogKeyV1,
) -> Option<CompactGroupMaterializationDefinitionV1> {
    match catalog_key {
        CompactGroupCatalogKeyV1::Methyl => Some(CompactGroupMaterializationDefinitionV1 {
            atoms: &ME_ATOMS,
            bonds: &[],
        }),
        CompactGroupCatalogKeyV1::Nitro => Some(CompactGroupMaterializationDefinitionV1 {
            atoms: &NO2_ATOMS,
            bonds: &NO2_BONDS,
        }),
        CompactGroupCatalogKeyV1::Ethyl
        | CompactGroupCatalogKeyV1::Phenyl
        | CompactGroupCatalogKeyV1::Methoxy
        | CompactGroupCatalogKeyV1::Cyano
        | CompactGroupCatalogKeyV1::Carboxyl
        | CompactGroupCatalogKeyV1::AcylChloride
        | CompactGroupCatalogKeyV1::Hydroxymethyl => None,
    }
}
