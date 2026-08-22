//! Safe publication of one authenticated multi-record SDF receipt.

use std::path::PathBuf;

use thiserror::Error;

use crate::{
    DocumentMoleculesSdfV2,
    artifact_publication_v1::{
        ArtifactPublicationErrorV1, ArtifactPublicationOutcomeV1, ArtifactPublicationRequestV1,
        publish_artifact_v1,
    },
};

/// Publish one completed multi-record SDF receipt without reading a session.
pub fn publish_document_molecules_sdf_v2(
    receipt: &DocumentMoleculesSdfV2,
    destination: PathBuf,
) -> Result<ArtifactPublicationOutcomeV1, DocumentMoleculesSdfPublicationErrorV2> {
    let mut bytes = Vec::new();
    if bytes.try_reserve_exact(receipt.sdf().len()).is_err() {
        return Err(DocumentMoleculesSdfPublicationErrorV2::ResourceAllocation { destination });
    }
    bytes.extend_from_slice(receipt.sdf().as_bytes());
    publish_artifact_v1(ArtifactPublicationRequestV1::new(destination, bytes)).map_err(Into::into)
}

/// Failure while materializing or safely publishing one multi-record SDF receipt.
#[derive(Debug, Error)]
pub enum DocumentMoleculesSdfPublicationErrorV2 {
    /// Exact output bytes could not be allocated.
    #[error("selected SDF publication to {destination} could not reserve output storage")]
    ResourceAllocation {
        /// Requested destination, retained because publication never started.
        destination: PathBuf,
    },
    /// The shared secure artifact publisher rejected or could not finish the write.
    #[error(transparent)]
    Publication(#[from] ArtifactPublicationErrorV1),
}
