//! Session-provenance wrapper around the renderer's immutable resolved value.

use crate::{DocumentSession, DocumentSessionError, SessionDocumentObservationV1};
use ferrum_render::{
    DepictionProfileV1, ResolvedDocumentRenderErrorV1, ResolvedDocumentRenderV1,
    ResolvedDocumentRenderWireV1, resolve_document_render_v1,
};
use thiserror::Error;

/// Closed schema identifier for a session-authenticated render observation.
pub const DOCUMENT_RENDER_OBSERVATION_SCHEMA_V1: &str = "ferrum-document-render-observation-v1";

/// One authoritative document observation paired with the exact pure render result.
#[derive(Debug)]
pub struct DocumentRenderObservationV1 {
    document: SessionDocumentObservationV1,
    resolved: ResolvedDocumentRenderV1,
}

impl DocumentRenderObservationV1 {
    fn from_document(
        document: SessionDocumentObservationV1,
        profile: DepictionProfileV1,
    ) -> Result<Self, DocumentRenderObservationErrorV1> {
        if document.snapshot().revision() != document.projection().revision()
            || document.snapshot().digest() != document.projection().digest()
        {
            return Err(DocumentRenderObservationErrorV1::ProvenanceMismatch);
        }
        let resolved = resolve_document_render_v1(document.projection().clone(), profile)?;
        if resolved.projection().revision() != document.snapshot().revision()
            || resolved.projection().digest() != document.snapshot().digest()
        {
            return Err(DocumentRenderObservationErrorV1::ProvenanceMismatch);
        }
        Ok(Self { document, resolved })
    }

    /// Return the immutable session observation that establishes document authority.
    #[must_use]
    pub const fn document(&self) -> &SessionDocumentObservationV1 {
        &self.document
    }

    /// Return the immutable pure renderer result authenticated to this observation.
    #[must_use]
    pub const fn resolved(&self) -> &ResolvedDocumentRenderV1 {
        &self.resolved
    }

    /// Return the wire representation of the immutable resolver result.
    #[must_use]
    pub fn wire(&self) -> ResolvedDocumentRenderWireV1 {
        self.resolved.wire()
    }
}

/// Derive a session-authenticated render observation from an accepted operation.
pub fn derive_document_render_observation_from_accepted_operation_v1(
    observation: &SessionDocumentObservationV1,
) -> Result<DocumentRenderObservationV1, DocumentRenderObservationErrorV1> {
    DocumentRenderObservationV1::from_document(
        observation.clone(),
        DepictionProfileV1::ferrum_default(),
    )
}

/// Acquire and resolve the exact current revision of one document session.
pub fn observe_document_render_v1(
    session: &DocumentSession,
    expected_revision: u64,
) -> Result<DocumentRenderObservationV1, DocumentRenderObservationErrorV1> {
    let document = session.observe(expected_revision)?;
    DocumentRenderObservationV1::from_document(document, DepictionProfileV1::ferrum_default())
}

impl DocumentSession {
    /// Acquire the authoritative observation and its pure render result at one revision.
    pub fn observe_render_v1(
        &self,
        expected_revision: u64,
    ) -> Result<DocumentRenderObservationV1, DocumentRenderObservationErrorV1> {
        observe_document_render_v1(self, expected_revision)
    }
}

/// Failure while authenticating a pure renderer result to document authority.
#[derive(Debug, Error)]
pub enum DocumentRenderObservationErrorV1 {
    /// The requested revision was not the session's current revision.
    #[error(transparent)]
    Document(#[from] DocumentSessionError),
    /// The immutable renderer resolver refused the exact document projection.
    #[error(transparent)]
    Render(#[from] ResolvedDocumentRenderErrorV1),
    /// Snapshot, projection, and resolved result did not share exact provenance.
    #[error("document render observation provenance did not match")]
    ProvenanceMismatch,
}

/// The document-owned wire view is exactly the pure immutable renderer wire format.
pub type DocumentRenderObservationWireV1 = ResolvedDocumentRenderWireV1;
