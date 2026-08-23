//! Authoritative document-session construction paths.

use super::{
    DocumentSession, DocumentSessionError, GeneratedIdSequences, RevisionState, SavedBaseline,
    SessionHistory, TypedDocument,
};
use crate::AuthoringCapabilityIssuerV1;

pub(crate) const EMPTY_DOCUMENT_SOURCE_V1: &str =
    r#"<cdml xmlns="urn:ferrum:cdml" version="26.07"/>"#;

impl DocumentSession {
    /// Create the canonical revision-zero baseline for one new empty CDML document.
    ///
    /// The root namespace and document version are backend-owned. This constructor
    /// does not expose an XML template, assign selectable roots, or assert the
    /// nonempty `authored-26.07` profile.
    pub fn create_empty_document_v1() -> Result<Self, DocumentSessionError> {
        let document =
            TypedDocument::parse(EMPTY_DOCUMENT_SOURCE_V1).map_err(DocumentSessionError::Load)?;
        Self::from_admitted_document(document)
    }

    /// Parse CDML into the sole authoritative retained tree.
    pub fn load(source: &str) -> Result<Self, DocumentSessionError> {
        let document = TypedDocument::parse(source).map_err(DocumentSessionError::Load)?;
        Self::from_admitted_document(document)
    }

    /// Start a revision-zero session from one backend-owned, already admitted document.
    ///
    /// This constructor accepts no source text or external storage. It initializes
    /// the same baseline and history state as [`Self::load`] without reparsing.
    pub fn from_admitted_document(document: TypedDocument) -> Result<Self, DocumentSessionError> {
        let initial =
            RevisionState::from_document(0, document).map_err(DocumentSessionError::Load)?;
        let saved_baseline = SavedBaseline::from_state(&initial);
        Ok(Self {
            authoring_capability_issuer: AuthoringCapabilityIssuerV1::new(),
            history: SessionHistory::new(initial, 20),
            saved_baseline,
            generated_ids: GeneratedIdSequences::initial(),
        })
    }
}
