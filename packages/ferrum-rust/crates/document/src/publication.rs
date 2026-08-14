//! CDML/session adapter for the generic artifact publisher.
//!
//! The generic publisher owns the descriptor transaction.  This module deliberately
//! owns only the CDML snapshot-to-owned-bytes copy and preserves the historical
//! `DocumentSessionError` and saved-baseline contract.

use std::path::Path;

#[cfg(test)]
use super::artifact_publication_v1::ArtifactPrepublicationPhaseV1;
use super::artifact_publication_v1::{
    ArtifactDestinationRejectionV1, ArtifactPublicationDurabilityV1, ArtifactPublicationErrorV1,
    ArtifactPublicationOutcomeV1, ArtifactPublicationRequestV1,
};
use super::session::DocumentSessionError;

/// Whether the directory entry replacement received platform-supported confirmation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicationDurability {
    /// Both data and the containing directory entry were synchronized.
    Confirmed,
    /// The replacement succeeded, but directory-entry durability is not confirmed.
    DirectoryEntryUnconfirmed,
}

pub(crate) fn publish_snapshot(
    path: &Path,
    cdml: &str,
) -> Result<PublicationDurability, DocumentSessionError> {
    let request = ArtifactPublicationRequestV1::new(path.to_path_buf(), cdml.as_bytes().to_vec());
    super::artifact_publication_v1::publish_artifact_v1(request)
        .map(outcome_to_durability)
        .map_err(map_error)
}

#[cfg(test)]
pub(crate) fn publish_snapshot_with_after_parent_open<H>(
    path: &Path,
    cdml: &str,
    mut after_parent_open: H,
) -> Result<PublicationDurability, DocumentSessionError>
where
    H: FnMut(),
{
    let request = ArtifactPublicationRequestV1::new(path.to_path_buf(), cdml.as_bytes().to_vec());
    let mut invoked = false;
    super::artifact_publication_v1::publish_artifact_with_test_seams_v1(
        request,
        |phase| {
            if !invoked && phase == ArtifactPrepublicationPhaseV1::ValidateBeforeTemporary {
                invoked = true;
                after_parent_open();
            }
        },
        |directory| {
            // Preserve production semantics while the phase seam lets the existing
            // descriptor-retention characterization test replace the visible parent.
            #[cfg(target_os = "macos")]
            {
                match rustix::fs::fsync(directory) {
                    Ok(()) => Ok(ArtifactPublicationDurabilityV1::Confirmed),
                    Err(rustix::io::Errno::INVAL) => {
                        Ok(ArtifactPublicationDurabilityV1::DirectoryEntryUnconfirmed)
                    }
                    Err(error) => Err(std::io::Error::from(error)),
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                rustix::fs::fsync(directory)
                    .map(|()| ArtifactPublicationDurabilityV1::Confirmed)
                    .map_err(std::io::Error::from)
            }
        },
    )
    .map(outcome_to_durability)
    .map_err(map_error)
}

fn outcome_to_durability(outcome: ArtifactPublicationOutcomeV1) -> PublicationDurability {
    match outcome.durability() {
        ArtifactPublicationDurabilityV1::Confirmed => PublicationDurability::Confirmed,
        ArtifactPublicationDurabilityV1::DirectoryEntryUnconfirmed => {
            PublicationDurability::DirectoryEntryUnconfirmed
        }
    }
}

fn map_error(error: ArtifactPublicationErrorV1) -> DocumentSessionError {
    match error {
        ArtifactPublicationErrorV1::RejectedDestination {
            destination,
            reason,
        } => DocumentSessionError::InvalidDestination {
            path: destination,
            reason: rejection_text(reason),
        },
        ArtifactPublicationErrorV1::NotPublished {
            destination,
            source,
            ..
        } => DocumentSessionError::PublishNotStarted {
            path: destination,
            source,
        },
        ArtifactPublicationErrorV1::NotPublishedTemporaryMayRemain {
            destination,
            source,
            cleanup,
            ..
        } => DocumentSessionError::PublishNotStartedWithCleanup {
            path: destination,
            source,
            cleanup,
        },
        ArtifactPublicationErrorV1::RejectedDestinationTemporaryMayRemain {
            destination,
            reason,
            cleanup,
        } => DocumentSessionError::ReplacementRejectedWithCleanup {
            path: destination,
            reason: rejection_text(reason).to_owned(),
            cleanup,
        },
        ArtifactPublicationErrorV1::TemporaryName {
            destination,
            source,
        } => DocumentSessionError::TemporaryName {
            path: destination,
            detail: source.to_string(),
        },
        ArtifactPublicationErrorV1::TemporaryNameExhausted { destination } => {
            DocumentSessionError::TemporaryNameExhausted { path: destination }
        }
        ArtifactPublicationErrorV1::PossiblyPublished { receipt, source } => {
            DocumentSessionError::PublishPossiblyCompleted {
                path: receipt.destination().to_path_buf(),
                source,
            }
        }
    }
}

fn rejection_text(reason: ArtifactDestinationRejectionV1) -> &'static str {
    match reason {
        ArtifactDestinationRejectionV1::MissingFileName => "destination must name a file",
        ArtifactDestinationRejectionV1::ParentTraversesSymlink => {
            "destination parent must not traverse a symbolic link"
        }
        ArtifactDestinationRejectionV1::ParentIsNotDirectory => {
            "destination parent must be a directory"
        }
        ArtifactDestinationRejectionV1::FinalIsSymlink => "destination must not be a symbolic link",
        ArtifactDestinationRejectionV1::FinalIsNotRegular => {
            "destination exists but is not a regular file"
        }
        ArtifactDestinationRejectionV1::SourceAliasesDestination => {
            "destination aliases retained source file"
        }
    }
}
