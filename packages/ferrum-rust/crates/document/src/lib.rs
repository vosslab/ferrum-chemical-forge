//! CDML document storage, typed recognition, and session services.
//!
//! The crate retains one authoritative XML tree, offers a typed CDML view, and exposes
//! revision-bound snapshots for safe publication.  Its public API is intentionally
//! independent from the private module tree.

mod cdsvg;
mod core_projection;
mod identity_index;
mod publication;
mod session;
mod session_history;
mod session_operation;
mod session_state;
mod typed;
mod typed_class;

pub use cdsvg::{CdsvgExtractionError, extract_cdml_from_svg};
pub use core_projection::{CoreProjection, CoreProjectionError};
pub use identity_index::{
    DocumentIdentityError, DocumentRecord, ElementPath, IndexedDocument, IndexedDocumentError,
    PersistentId, ProvisionalToken, ResolvedId, SourceOrder, XmlDocument, XmlSerializationError,
};
pub use publication::PublicationDurability;
pub use session::{
    DocumentSession, DocumentSessionError, DocumentSnapshot, PendingCreateAtom, Publication,
    SaveOutcome,
};
pub use session_operation::{
    SessionObservationV1, SessionOperation, SessionOperationError, SessionOperationV1,
};
pub use typed::{
    ExpandedName, NamespaceBinding, TypedChild, TypedClass, TypedDiagnostic, TypedDiagnosticKind,
    TypedDocument, TypedDocumentError, TypedRecord, TypedText, UnknownAttribute, UnrecognizedChild,
    UnrecognizedNode,
};

pub(crate) use identity_index::{CDML_NAMESPACE, element_name};

#[cfg(test)]
mod compatibility_tests;

#[cfg(test)]
mod cdsvg_tests;

#[cfg(test)]
mod identity_index_tests;

#[cfg(test)]
mod session_tests;

#[cfg(test)]
mod publication_tests;

#[cfg(test)]
mod session_semantics_tests;

#[cfg(test)]
mod typed_tests;
