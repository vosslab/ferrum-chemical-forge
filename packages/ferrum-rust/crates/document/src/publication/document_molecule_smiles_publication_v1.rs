//! Safe publication of one authenticated canonical SMILES receipt.

use std::path::PathBuf;

use crate::artifact_publication_v1::{
    ArtifactPublicationErrorV1, ArtifactPublicationOutcomeV1, ArtifactPublicationRequestV1,
    publish_artifact_v1,
};
use thiserror::Error;

use crate::DocumentMoleculeSmilesV1;

/// Publish one immutable canonical SMILES receipt as one newline-terminated file.
///
/// The receipt was authenticated when it was created. This operation publishes
/// exactly that frozen result and never reads or mutates a document session.
pub fn publish_document_molecule_smiles_v1(
    receipt: &DocumentMoleculeSmilesV1,
    destination: PathBuf,
) -> Result<ArtifactPublicationOutcomeV1, DocumentMoleculeSmilesPublicationErrorV1> {
    let Some(byte_len) = receipt.smiles().len().checked_add(1) else {
        return Err(DocumentMoleculeSmilesPublicationErrorV1::ResourceAllocation { destination });
    };
    let mut bytes = Vec::new();
    if bytes.try_reserve_exact(byte_len).is_err() {
        return Err(DocumentMoleculeSmilesPublicationErrorV1::ResourceAllocation { destination });
    }
    bytes.extend_from_slice(receipt.smiles().as_bytes());
    bytes.push(b'\n');
    let request = ArtifactPublicationRequestV1::new(destination, bytes);
    publish_artifact_v1(request).map_err(Into::into)
}

/// Failure while materializing or safely publishing a canonical SMILES receipt.
#[derive(Debug, Error)]
pub enum DocumentMoleculeSmilesPublicationErrorV1 {
    /// Exact output bytes could not be allocated.
    #[error("canonical SMILES publication to {destination} could not reserve output storage")]
    ResourceAllocation {
        /// Requested destination, retained because publication never started.
        destination: PathBuf,
    },
    /// The shared secure artifact publisher rejected or could not finish the write.
    #[error(transparent)]
    Publication(#[from] ArtifactPublicationErrorV1),
}
