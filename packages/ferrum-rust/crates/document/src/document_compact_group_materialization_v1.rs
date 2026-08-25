//! Public fenced contract for materializing one typed compact group.

use thiserror::Error;

use crate::DocumentObjectIdV1;

/// One exact-document request to replace an attached typed compact group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentCompactGroupMaterializationRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    compact_group_id: DocumentObjectIdV1,
}

/// Closed reasons durable live targets cannot lower to one compact-group request.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DocumentCompactGroupMaterializationTargetErrorV1 {
    #[error("selected molecule does not occur in the current document")]
    UnknownMolecule,
    #[error("selected molecule is not a typed molecule")]
    InvalidMolecule,
    #[error("selected compact group does not occur in the selected molecule")]
    UnknownOrForeignCompactGroup,
    #[error("selected target is not a typed compact group")]
    InvalidCompactGroup,
}

impl DocumentCompactGroupMaterializationRequestV1 {
    #[must_use]
    pub const fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_id: DocumentObjectIdV1,
        compact_group_id: DocumentObjectIdV1,
    ) -> Self {
        Self {
            expected_revision,
            expected_digest,
            molecule_id,
            compact_group_id,
        }
    }
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
    #[must_use]
    pub const fn expected_digest(&self) -> &[u8; 32] {
        &self.expected_digest
    }
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
    #[must_use]
    pub const fn compact_group_id(&self) -> &DocumentObjectIdV1 {
        &self.compact_group_id
    }
}

/// Closed reasons the source compact group cannot become ordinary editable chemistry.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DocumentCompactGroupMaterializationRefusalV1 {
    #[error("document revision is stale")]
    StaleObservation,
    #[error("document digest is stale")]
    DigestMismatch,
    #[error("compact-group target is not one typed direct molecule child")]
    InvalidTarget,
    #[error("compact-group recipe is not available")]
    UnsupportedRecipe,
    #[error("compact-group exterior bond topology is invalid")]
    InvalidTopology,
    #[error("compact-group candidate IDs are invalid")]
    InvalidCandidate,
    #[error("compact-group candidate was refused by renderer admission")]
    RendererAdmission,
}

/// Durable identities from one committed materialization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentCompactGroupMaterializationResultV1 {
    molecule_id: DocumentObjectIdV1,
    compact_group_id: DocumentObjectIdV1,
    focus_atom_id: DocumentObjectIdV1,
}

impl DocumentCompactGroupMaterializationResultV1 {
    pub(crate) const fn new(
        molecule_id: DocumentObjectIdV1,
        compact_group_id: DocumentObjectIdV1,
        focus_atom_id: DocumentObjectIdV1,
    ) -> Self {
        Self {
            molecule_id,
            compact_group_id,
            focus_atom_id,
        }
    }
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
    #[must_use]
    pub const fn compact_group_id(&self) -> &DocumentObjectIdV1 {
        &self.compact_group_id
    }
    #[must_use]
    pub const fn focus_atom_id(&self) -> &DocumentObjectIdV1 {
        &self.focus_atom_id
    }
}
