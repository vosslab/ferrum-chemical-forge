//! Durable, diagnostic, and projection-local identities without document traversal.

use serde::{Deserialize, Serialize};
use thiserror::Error;

const DOCUMENT_OBJECT_ID_PREFIX: &str = "ferrum-document-object-v1/";
const DOCUMENT_OBJECT_ID_ENTROPY_BYTES: usize = 16;
const DOCUMENT_LOCATION_MAX_CHILD_PATH_DEPTH: usize = 64;

/// An opaque, versioned selector for one durable typed record.
///
/// The owning document allocates this from independent entropy and persists it.
/// It deliberately carries no record class, source ID, path, or authored content.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct DocumentObjectIdV1(String);

impl DocumentObjectIdV1 {
    /// Create the canonical selector for exactly 128 bits of document-owned entropy.
    #[must_use]
    pub fn from_entropy_bytes(entropy: [u8; DOCUMENT_OBJECT_ID_ENTROPY_BYTES]) -> Self {
        let mut value = String::with_capacity(DOCUMENT_OBJECT_ID_PREFIX.len() + entropy.len() * 2);
        value.push_str(DOCUMENT_OBJECT_ID_PREFIX);
        for byte in entropy {
            use std::fmt::Write as _;
            write!(value, "{byte:02x}").expect("writing to String cannot fail");
        }
        Self(value)
    }

    /// Parse a validated V1 document-object selector.
    pub fn parse(value: impl Into<String>) -> Result<Self, DocumentObjectIdV1Error> {
        let value = value.into();
        let Some(entropy) = value.strip_prefix(DOCUMENT_OBJECT_ID_PREFIX) else {
            return Err(DocumentObjectIdV1Error::InvalidWireKey);
        };
        if !is_canonical_entropy(entropy) {
            return Err(DocumentObjectIdV1Error::InvalidWireKey);
        }
        Ok(Self(value))
    }

    /// Return the stable opaque wire key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for DocumentObjectIdV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Failure while decoding an opaque V1 document-object selector.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DocumentObjectIdV1Error {
    /// The selector did not use the exact closed V1 grammar.
    #[error("invalid ferrum document object V1 wire key")]
    InvalidWireKey,
}

/// The source-free class of a diagnostic location before a durable ID exists.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentLocationKindV1 {
    /// A structural direct root such as a molecule.
    Structural,
    /// A recognized direct-root presentation record.
    Presentation,
}

/// A bounded source-free location for admission and diagnostic reporting.
///
/// This value has no source identifier or document content and is never a
/// mutation selector. Durable records use [`DocumentObjectIdV1`] instead.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct DocumentLocationV1 {
    kind: DocumentLocationKindV1,
    root_ordinal: u32,
    child_path: Vec<u32>,
}

impl DocumentLocationV1 {
    /// Create a location from a closed class and root-relative numeric coordinates.
    pub fn try_new(
        kind: DocumentLocationKindV1,
        root_ordinal: u32,
        child_path: Vec<u32>,
    ) -> Result<Self, DocumentLocationV1Error> {
        if child_path.len() > DOCUMENT_LOCATION_MAX_CHILD_PATH_DEPTH {
            return Err(DocumentLocationV1Error::ChildPathTooDeep);
        }
        Ok(Self {
            kind,
            root_ordinal,
            child_path,
        })
    }

    /// Return the closed class of the located record.
    #[must_use]
    pub const fn kind(&self) -> DocumentLocationKindV1 {
        self.kind
    }

    /// Return the root-relative numeric direct-root ordinal.
    #[must_use]
    pub const fn root_ordinal(&self) -> u32 {
        self.root_ordinal
    }

    /// Return the bounded numeric path below the direct root.
    #[must_use]
    pub fn child_path(&self) -> &[u32] {
        &self.child_path
    }
}

impl<'de> Deserialize<'de> for DocumentLocationV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = DocumentLocationV1Wire::deserialize(deserializer)?;
        Self::try_new(wire.kind, wire.root_ordinal, wire.child_path)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
struct DocumentLocationV1Wire {
    kind: DocumentLocationKindV1,
    root_ordinal: u32,
    child_path: Vec<u32>,
}

/// Failure while constructing a bounded source-free V1 location.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DocumentLocationV1Error {
    /// The root-relative numeric path exceeded the V1 resource bound.
    #[error("ferrum document location V1 child path exceeds the maximum depth")]
    ChildPathTooDeep,
}

/// A projection-internal preview/layout key, never a durable document selector.
///
/// It can describe a projected child while rendering, but callers must use a
/// [`DocumentObjectIdV1`] for persisted targets and all session mutations.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ProjectionLocalObjectKeyV1(String);

impl ProjectionLocalObjectKeyV1 {
    /// Derive a structural key from a nonempty source-order component path.
    pub fn from_path_components(
        path_components: &[u32],
    ) -> Result<Self, ProjectionLocalObjectKeyV1Error> {
        if path_components.is_empty() {
            return Err(ProjectionLocalObjectKeyV1Error::EmptyPath);
        }
        let path = path_components
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(".");
        Ok(Self(format!("ferrum-projection-local-v1/{path}")))
    }

    /// Parse a closed V1 projection-local wire key.
    #[must_use]
    pub fn parse(value: String) -> Option<Self> {
        let path = value.strip_prefix("ferrum-projection-local-v1/")?;
        (!path.is_empty() && path.split('.').all(valid_local_path_component)).then_some(Self(value))
    }

    /// Return the projection-local wire key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Failure while constructing a V1 projection-local selector.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ProjectionLocalObjectKeyV1Error {
    /// A projection-local selector requires at least one source-path component.
    #[error("ferrum projection-local V1 path must not be empty")]
    EmptyPath,
}

fn is_canonical_entropy(value: &str) -> bool {
    value.len() == DOCUMENT_OBJECT_ID_ENTROPY_BYTES * 2
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (byte.is_ascii_lowercase() && byte <= b'f'))
}

fn valid_local_path_component(component: &str) -> bool {
    component
        .parse::<u32>()
        .ok()
        .is_some_and(|number| number.to_string() == component)
}

#[cfg(test)]
mod tests {
    use super::{
        DOCUMENT_LOCATION_MAX_CHILD_PATH_DEPTH, DocumentLocationKindV1, DocumentLocationV1,
        DocumentLocationV1Error, DocumentObjectIdV1, DocumentObjectIdV1Error,
        ProjectionLocalObjectKeyV1, ProjectionLocalObjectKeyV1Error,
    };

    #[test]
    fn document_object_id_round_trips_the_canonical_opaque_grammar() {
        let identity = DocumentObjectIdV1::from_entropy_bytes([0xab; 16]);

        assert_eq!(
            identity.as_str(),
            "ferrum-document-object-v1/abababababababababababababababab"
        );
        assert_eq!(DocumentObjectIdV1::parse(identity.as_str()), Ok(identity));
    }

    #[test]
    fn document_object_id_refuses_old_or_malformed_grammars() {
        for value in [
            "ferrum-document-object-v1/abababababababababababababababa",
            "ferrum-document-object-v1/ABABABABABABABABABABABABABABABAB",
            "ferrum-document-object-v1/abababababababababababababababab/extra",
            "ferrum-document-object-v1/63646d6c2f6d6f6c6563756c65/source/6d6f6c6563756c652d31",
        ] {
            assert_eq!(
                DocumentObjectIdV1::parse(value),
                Err(DocumentObjectIdV1Error::InvalidWireKey),
            );
        }
    }

    #[test]
    fn document_object_id_entropy_is_distinct_and_has_no_source_text() {
        let first = DocumentObjectIdV1::from_entropy_bytes([0; 16]);
        let second = DocumentObjectIdV1::from_entropy_bytes([1; 16]);

        assert_ne!(first, second);
        assert!(!first.as_str().contains("molecule-1"));
        assert!(!second.as_str().contains("cdml"));
    }

    #[test]
    fn document_location_is_bounded_source_free_and_serde_round_trips() {
        let location =
            DocumentLocationV1::try_new(DocumentLocationKindV1::Presentation, 7, vec![2, 3])
                .expect("a bounded numeric child path is valid");
        let wire = serde_json::to_string(&location).expect("location serializes");

        assert_eq!(location.kind(), DocumentLocationKindV1::Presentation);
        assert_eq!(location.root_ordinal(), 7);
        assert_eq!(location.child_path(), [2, 3]);
        assert!(!wire.contains("source"));
        assert_eq!(
            serde_json::from_str::<DocumentLocationV1>(&wire).expect("location deserializes"),
            location
        );
    }

    #[test]
    fn document_location_refuses_paths_beyond_its_resource_bound() {
        assert_eq!(
            DocumentLocationV1::try_new(
                DocumentLocationKindV1::Structural,
                0,
                vec![0; DOCUMENT_LOCATION_MAX_CHILD_PATH_DEPTH + 1],
            ),
            Err(DocumentLocationV1Error::ChildPathTooDeep),
        );
    }

    #[test]
    fn projection_local_key_refuses_an_empty_component_path() {
        assert_eq!(
            ProjectionLocalObjectKeyV1::from_path_components(&[]),
            Err(ProjectionLocalObjectKeyV1Error::EmptyPath),
        );
    }

    #[test]
    fn projection_local_key_requires_canonical_decimal_path_components() {
        assert!(
            ProjectionLocalObjectKeyV1::parse("ferrum-projection-local-v1/1".to_owned()).is_some()
        );
        assert!(
            ProjectionLocalObjectKeyV1::parse("ferrum-projection-local-v1/01".to_owned()).is_none()
        );
    }
}
