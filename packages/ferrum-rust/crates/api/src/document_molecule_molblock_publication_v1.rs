//! Safe publication of one authenticated Molfile receipt.

use std::path::PathBuf;

use ferrum_document::artifact_publication_v1::{
    ArtifactPublicationErrorV1, ArtifactPublicationOutcomeV1, ArtifactPublicationRequestV1,
    publish_artifact_v1,
};
use thiserror::Error;

use crate::DocumentMoleculeMolblockV1;

/// Publish one immutable receipt as the exact native Molfile bytes.
///
/// The receipt was authenticated when it was created. This operation publishes
/// the frozen result and never reads or mutates a document session.
pub fn publish_document_molecule_molblock_v1(
    receipt: &DocumentMoleculeMolblockV1,
    destination: PathBuf,
) -> Result<ArtifactPublicationOutcomeV1, DocumentMoleculeMolblockPublicationErrorV1> {
    let mut bytes = Vec::new();
    if bytes.try_reserve_exact(receipt.molblock().len()).is_err() {
        return Err(DocumentMoleculeMolblockPublicationErrorV1::ResourceAllocation { destination });
    }
    bytes.extend_from_slice(receipt.molblock().as_bytes());
    let request = ArtifactPublicationRequestV1::new(destination, bytes);
    publish_artifact_v1(request).map_err(Into::into)
}

/// Failure while materializing or safely publishing one Molfile receipt.
#[derive(Debug, Error)]
pub enum DocumentMoleculeMolblockPublicationErrorV1 {
    /// Exact output bytes could not be allocated.
    #[error("Molfile publication to {destination} could not reserve output storage")]
    ResourceAllocation {
        /// Requested destination, retained because publication never started.
        destination: PathBuf,
    },
    /// The shared secure artifact publisher rejected or could not finish the write.
    #[error(transparent)]
    Publication(#[from] ArtifactPublicationErrorV1),
}
