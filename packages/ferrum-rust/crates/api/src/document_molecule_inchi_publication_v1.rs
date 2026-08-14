//! Safe publication of one exact-source InChI receipt.

use std::path::PathBuf;

use ferrum_document::artifact_publication_v1::{
    ArtifactPublicationErrorV1, ArtifactPublicationOutcomeV1, ArtifactPublicationRequestV1,
    publish_artifact_v1,
};
use thiserror::Error;

use crate::DocumentMoleculeInchiV1;

/// Publish one immutable InChI receipt as one newline-terminated UTF-8 file.
pub fn publish_document_molecule_inchi_v1(
    receipt: &DocumentMoleculeInchiV1,
    destination: PathBuf,
) -> Result<ArtifactPublicationOutcomeV1, DocumentMoleculeInchiPublicationErrorV1> {
    let Some(byte_len) = receipt.inchi().len().checked_add(1) else {
        return Err(DocumentMoleculeInchiPublicationErrorV1::ResourceAllocation { destination });
    };
    let mut bytes = Vec::new();
    if bytes.try_reserve_exact(byte_len).is_err() {
        return Err(DocumentMoleculeInchiPublicationErrorV1::ResourceAllocation { destination });
    }
    bytes.extend_from_slice(receipt.inchi().as_bytes());
    bytes.push(b'\n');
    publish_artifact_v1(ArtifactPublicationRequestV1::new(destination, bytes)).map_err(Into::into)
}

/// Failure while materializing or safely publishing one InChI receipt.
#[derive(Debug, Error)]
pub enum DocumentMoleculeInchiPublicationErrorV1 {
    /// Exact output bytes could not be allocated.
    #[error("InChI publication to {destination} could not reserve output storage")]
    ResourceAllocation {
        /// Requested destination, retained because publication never started.
        destination: PathBuf,
    },
    /// The shared secure artifact publisher rejected or could not finish the write.
    #[error(transparent)]
    Publication(#[from] ArtifactPublicationErrorV1),
}
