//! User-template placement through the generic admitted transition boundary.

use std::path::Path;

use ferrum_geometry::Point2;
use thiserror::Error;

use crate::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityV1, DocumentUserTemplateErrorV1,
    DocumentUserTemplateInsertedMoleculeV1, DocumentUserTemplatePlanV1, Publication,
};

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentSession, DocumentSessionError, RevisionState,
    SessionOperationResultV1,
};

/// Frozen one-use authority to publish the current live document as a reusable template.
///
/// The receipt deliberately contains neither CDML nor a mutable document alias.  Its
/// private capability proves session ownership, while its revision and digest bind it
/// to one exact retained state.
#[derive(Debug)]
pub struct PreparedDocumentUserTemplatePublicationV1 {
    capability: AuthoringCapabilityV1,
    revision: u64,
    digest: [u8; 32],
    display_name: Option<String>,
}

impl PreparedDocumentUserTemplatePublicationV1 {
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    #[must_use]
    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }
}

/// Closed failures for live user-template publication.
#[derive(Debug, Error)]
pub enum DocumentUserTemplatePublicationErrorV1 {
    /// The live document is not reusable-template content.
    #[error(transparent)]
    Ineligible(#[from] DocumentUserTemplateErrorV1),
    /// The receipt was minted by a different live document session.
    #[error("user-template publication receipt belongs to another document session")]
    ForeignSession,
    /// The receipt was already used to publish a template.
    #[error("user-template publication receipt was already consumed")]
    Consumed,
    /// The live document revision changed after receipt preparation.
    #[error(transparent)]
    Session(#[from] DocumentSessionError),
}

/// Exact authoritative outcome of one accepted user-template insertion.
#[derive(Debug)]
pub struct DocumentUserTemplateResultV1 {
    operation: SessionOperationResultV1,
    inserted_molecule: DocumentUserTemplateInsertedMoleculeV1,
}

impl DocumentUserTemplateResultV1 {
    #[must_use]
    pub fn operation_result(&self) -> &SessionOperationResultV1 {
        &self.operation
    }
    #[must_use]
    pub fn into_operation_result(self) -> SessionOperationResultV1 {
        self.operation
    }
    #[must_use]
    pub fn inserted_molecule(&self) -> &DocumentUserTemplateInsertedMoleculeV1 {
        &self.inserted_molecule
    }
}

impl DocumentSession {
    /// Prepare a one-use, exact-fence receipt for publishing the live document.
    ///
    /// Eligibility is derived from the retained typed document directly. No
    /// frontend snapshot, CDML string, or external-template plan participates.
    pub fn prepare_document_user_template_publication_v1(
        &self,
    ) -> Result<PreparedDocumentUserTemplatePublicationV1, DocumentUserTemplatePublicationErrorV1>
    {
        let current = self.current_state_v1();
        let display_name = super::super::user_template_v1::inspect_live_document_user_template_v1(
            current.document(),
        )?;
        Ok(PreparedDocumentUserTemplatePublicationV1 {
            capability: self.authoring_capability_issuer_v1().issue(),
            revision: current.revision(),
            digest: *current.digest(),
            display_name,
        })
    }

    /// Publish the exact live state authorized by one saved-template receipt.
    ///
    /// A failed pre-replacement publication releases the receipt for a safe retry.
    /// Any completed replacement, including directory-entry uncertainty, consumes it.
    pub fn publish_document_user_template_v1(
        &self,
        prepared: &PreparedDocumentUserTemplatePublicationV1,
        path: &Path,
    ) -> Result<Publication, DocumentUserTemplatePublicationErrorV1> {
        let claim = prepared
            .capability
            .claim_for_commit(&self.authoring_capability_issuer_v1())
            .map_err(|error| match error {
                AuthoringCapabilityAccessErrorV1::ForeignSession => {
                    DocumentUserTemplatePublicationErrorV1::ForeignSession
                }
                AuthoringCapabilityAccessErrorV1::Consumed => {
                    DocumentUserTemplatePublicationErrorV1::Consumed
                }
            })?;
        self.require_current(prepared.revision)?;
        if self.current_state_v1().digest() != prepared.digest() {
            return Err(DocumentUserTemplateErrorV1::DigestMismatch.into());
        }
        let snapshot = self.snapshot()?;
        let durability = super::super::publication::publish_snapshot(path, snapshot.cdml())?;
        claim.consume();
        Ok(Publication::from_durability(
            snapshot.clone(),
            snapshot,
            durability,
        ))
    }

    /// Place one immutable user template through renderer admission.
    pub fn insert_document_user_template_v1(
        &mut self,
        expected_revision: u64,
        expected_digest: &[u8; 32],
        plan: &DocumentUserTemplatePlanV1,
        anchor: Point2,
    ) -> Result<DocumentUserTemplateResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if self.current_state_v1().digest() != expected_digest {
            return Err(DocumentUserTemplateErrorV1::DigestMismatch.into());
        }
        let (generated, effects, source_revision, source_digest, revision) = {
            let (generated, effects) =
                self.reserve_generated_ids_for_transition_v1(|ids, indexed| {
                    ids.reserve_fragment_import(indexed, plan.declared_id_count())
                })?;
            let current = self.current_state_v1();
            (
                generated,
                effects,
                current.revision(),
                *current.digest(),
                current.next_revision(),
            )
        };
        let revision = revision.ok_or(DocumentSessionError::RevisionExhausted)?;
        let (candidate, inserted_molecule) =
            super::super::user_template_v1::compose_document_user_template_candidate_v1(
                self.current_state_v1().document(),
                plan,
                &generated,
                anchor,
            )?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let mut transition = self.prepare_changed_session_transition_v1(
            source_revision,
            source_digest,
            state,
            effects,
        )?;
        let operation = self
            .commit_session_operation_transition_v1(&mut transition)
            .map_err(|refusal| map_transition_refusal(self, expected_revision, refusal))?;
        Ok(DocumentUserTemplateResultV1 {
            operation,
            inserted_molecule,
        })
    }
}

fn map_transition_refusal(
    session: &DocumentSession,
    expected_revision: u64,
    refusal: AdmittedSessionTransitionRefusalV1,
) -> DocumentSessionError {
    match refusal {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            DocumentSessionError::PreparedOperationForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Consumed
        | AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            DocumentSessionError::PreparedOperationConsumed
        }
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            DocumentSessionError::RevisionConflict {
                expected: expected_revision,
                actual: session.current_revision_v1(),
            }
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            DocumentSessionError::RendererAdmission
        }
    }
}

#[cfg(test)]
mod publication_tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    use ferrum_geometry::Point2;

    use super::*;
    use crate::{
        DocumentSession, DocumentUserTemplatePublicationErrorV1, prepare_user_template_v1,
    };

    static NEXT_TARGET: AtomicU64 = AtomicU64::new(0);

    const TEMPLATE: &str = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\" name=\"Example\">",
        "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "</molecule></cdml>",
    );

    fn target(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let directory = std::env::temp_dir()
            .canonicalize()
            .expect("temporary root must resolve without a symbolic link")
            .join(format!(
                "ferrum-user-template-publication-{name}-{}-{}",
                std::process::id(),
                NEXT_TARGET.fetch_add(1, Ordering::Relaxed),
            ));
        fs::create_dir(&directory).expect("test directory");
        let destination = directory.join("template.cdml");
        (directory, destination)
    }

    #[test]
    fn eligible_live_document_receipt_publishes_without_a_snapshot_payload() {
        let session = DocumentSession::load(TEMPLATE).expect("valid live document");
        let receipt = session
            .prepare_document_user_template_publication_v1()
            .expect("eligible live document receipt");
        let (directory, destination) = target("eligible");

        let publication = session
            .publish_document_user_template_v1(&receipt, &destination)
            .expect("receipt publishes once");

        assert_eq!(receipt.display_name(), Some("Example"));
        assert_eq!(
            fs::read_to_string(&destination).expect("published file"),
            publication.published_snapshot().cdml(),
        );
        fs::remove_dir_all(directory).expect("test artifact cleanup");
    }

    #[test]
    fn ineligible_live_document_refuses_before_destination_publication() {
        let session = DocumentSession::create_empty_document_v1().expect("empty session");
        let (directory, destination) = target("ineligible");

        assert!(matches!(
            session.prepare_document_user_template_publication_v1(),
            Err(DocumentUserTemplatePublicationErrorV1::Ineligible(_))
        ));
        assert!(!destination.exists());
        fs::remove_dir_all(directory).expect("test artifact cleanup");
    }

    #[test]
    fn stale_receipt_cannot_publish_any_file() {
        let mut session = DocumentSession::load(TEMPLATE).expect("valid live document");
        let receipt = session
            .prepare_document_user_template_publication_v1()
            .expect("eligible live document receipt");
        let plan = prepare_user_template_v1(TEMPLATE).expect("valid detached template");
        let snapshot = session.snapshot().expect("source snapshot");
        session
            .insert_document_user_template_v1(
                snapshot.revision(),
                snapshot.digest(),
                &plan,
                Point2::new(20.0, 20.0).expect("finite anchor"),
            )
            .expect("mutation after receipt");
        let (directory, destination) = target("stale");

        assert!(matches!(
            session.publish_document_user_template_v1(&receipt, &destination),
            Err(DocumentUserTemplatePublicationErrorV1::Session(
                DocumentSessionError::RevisionConflict { .. }
            ))
        ));
        assert!(!destination.exists());
        fs::remove_dir_all(directory).expect("test artifact cleanup");
    }
}
