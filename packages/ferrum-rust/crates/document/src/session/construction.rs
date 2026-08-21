//! Authoritative document-session construction paths.

use std::sync::atomic::{AtomicU64, Ordering};

use super::{
    DocumentSession, DocumentSessionError, GeneratedIdSequences, RevisionState, SavedBaseline,
    SessionHistory, TypedDocument,
};
use crate::direct_bond_gesture_v1::DirectBondSessionOriginV1;
use crate::presentation_creation_gesture_v1::PresentationGestureSessionOriginV1;
use crate::text_placement_gesture_v1::TextPlacementSessionOriginV1;

pub(crate) const EMPTY_DOCUMENT_SOURCE_V1: &str =
    r#"<cdml xmlns="http://www.freesoftware.fsf.org/bkchem/cdml" version="26.07"/>"#;

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
            bridge_session_origin: next_bridge_session_origin(),
            history: SessionHistory::new(initial, 20),
            saved_baseline,
            generated_ids: GeneratedIdSequences::initial(),
            direct_bond_origin: DirectBondSessionOriginV1::issue(),
            presentation_gesture_origin: PresentationGestureSessionOriginV1::issue(),
            text_placement_origin: TextPlacementSessionOriginV1::issue(),
            text_placement_consumed: std::collections::HashSet::new(),
        })
    }
}

fn next_bridge_session_origin() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}
