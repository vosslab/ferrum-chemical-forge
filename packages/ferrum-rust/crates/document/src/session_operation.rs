//! Narrow, typed operations accepted by the document transaction session.

use thiserror::Error;

use super::{PersistentId, TypedDocument, TypedDocumentError, XmlSerializationError};

/// Immutable revision-bound observation envelope for frontend projection staging.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionObservationV1 {
    snapshot: super::DocumentSnapshot,
}

impl SessionObservationV1 {
    pub(super) fn new(snapshot: super::DocumentSnapshot) -> Self {
        Self { snapshot }
    }

    /// Return the complete snapshot from which every projection fact must derive.
    #[must_use]
    pub fn snapshot(&self) -> &super::DocumentSnapshot {
        &self.snapshot
    }
}

/// Versioned session operation staging the initial supported document mutation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionOperation {
    /// The only currently supported protocol version.
    V1(SessionOperationV1),
}

/// First version of Rust-owned typed document operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionOperationV1 {
    /// Replace the element spelling of an existing typed atom.
    SetAtomElement { atom_id: String, element: String },
}

/// Typed operation failure before an accepted state transition.
#[derive(Debug, Error)]
pub enum SessionOperationError {
    /// A requested element spelling is empty or has invalid XML-like content.
    #[error("atom element must be a nonblank plain element spelling")]
    InvalidAtomElement,
    /// The requested typed atom does not occur in the retained document.
    #[error("typed atom does not exist: {0}")]
    UnknownAtom(String),
    /// Candidate construction or retained-document validation failed.
    #[error("cannot prepare document candidate: {0}")]
    Candidate(#[from] TypedDocumentError),
    /// Candidate comparison could not serialize retained CDML.
    #[error("cannot serialize document candidate: {0}")]
    Serialize(#[from] XmlSerializationError),
}

/// One detached candidate outcome.
pub(super) enum Candidate {
    /// The requested semantic change leaves canonical content unchanged.
    NoChange,
    /// A fully validated retained tree ready for atomic acceptance.
    Changed(Box<TypedDocument>),
}

impl SessionOperation {
    pub(super) fn prepare(
        &self,
        current: &TypedDocument,
    ) -> Result<Candidate, SessionOperationError> {
        match self {
            Self::V1(SessionOperationV1::SetAtomElement { atom_id, element }) => {
                if element.trim().is_empty()
                    || element
                        .chars()
                        .any(|character| !character.is_ascii_alphabetic())
                {
                    return Err(SessionOperationError::InvalidAtomElement);
                }
                let identifier = PersistentId::new(atom_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let candidate = current.with_atom_element(&identifier, element)?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
        }
    }
}
