//! Safe publication of one authenticated selected-molecule SDF receipt.

use std::path::PathBuf;

use crate::artifact_publication_v1::{
    ArtifactPublicationErrorV1, ArtifactPublicationOutcomeV1, ArtifactPublicationRequestV1,
    publish_artifact_v1,
};
use thiserror::Error;

use crate::DocumentMoleculeSdfV1;

/// Publish one immutable receipt as the exact completed SDF bytes.
///
/// The receipt was authenticated when it was created. This operation publishes
/// the frozen result and never reads or mutates a document session.
pub fn publish_document_molecule_sdf_v1(
    receipt: &DocumentMoleculeSdfV1,
    destination: PathBuf,
) -> Result<ArtifactPublicationOutcomeV1, DocumentMoleculeSdfPublicationErrorV1> {
    let mut bytes = Vec::new();
    if bytes.try_reserve_exact(receipt.sdf().len()).is_err() {
        return Err(DocumentMoleculeSdfPublicationErrorV1::ResourceAllocation { destination });
    }
    bytes.extend_from_slice(receipt.sdf().as_bytes());
    let request = ArtifactPublicationRequestV1::new(destination, bytes);
    publish_artifact_v1(request).map_err(Into::into)
}

/// Failure while materializing or safely publishing one exact SDF receipt.
#[derive(Debug, Error)]
pub enum DocumentMoleculeSdfPublicationErrorV1 {
    /// Exact output bytes could not be allocated.
    #[error("SDF publication to {destination} could not reserve output storage")]
    ResourceAllocation {
        /// Requested destination, retained because publication never started.
        destination: PathBuf,
    },
    /// The shared secure artifact publisher rejected or could not finish the write.
    #[error(transparent)]
    Publication(#[from] ArtifactPublicationErrorV1),
}
