//! Durable and projection-local identities without document traversal.

use serde::Serialize;
use thiserror::Error;

/// An opaque, versioned selector for one durable typed record.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct DocumentObjectIdV1(String);

impl DocumentObjectIdV1 {
    /// Derive a durable identity from a recognized class and authored source ID.
    pub fn from_class_source(
        class: &str,
        source_id: &str,
    ) -> Result<Self, DocumentObjectIdV1Error> {
        Self::parse(format!(
            "ferrum-document-object-v1/{}/source/{}",
            hex(class.as_bytes()),
            hex(source_id.as_bytes())
        ))
    }

    /// Parse a validated V1 document-object selector.
    pub fn parse(value: impl Into<String>) -> Result<Self, DocumentObjectIdV1Error> {
        let value = value.into();
        let mut components = value.split('/');
        let Some(prefix) = components.next() else {
            return Err(DocumentObjectIdV1Error::InvalidWireKey);
        };
        let Some(class) = components.next() else {
            return Err(DocumentObjectIdV1Error::InvalidWireKey);
        };
        let Some(origin) = components.next() else {
            return Err(DocumentObjectIdV1Error::InvalidWireKey);
        };
        let Some(payload) = components.next() else {
            return Err(DocumentObjectIdV1Error::InvalidWireKey);
        };
        if prefix != "ferrum-document-object-v1"
            || components.next().is_some()
            || !valid_hex(class)
            || origin != "source"
            || !valid_hex(payload)
        {
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

/// Failure while decoding an opaque V1 document-object selector.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DocumentObjectIdV1Error {
    /// The selector did not use the exact closed V1 grammar.
    #[error("invalid ferrum document object V1 wire key")]
    InvalidWireKey,
}

/// A projection-local key that never selects a session operation.
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

fn valid_hex(value: &str) -> bool {
    !value.is_empty()
        && value.len().is_multiple_of(2)
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
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
        DocumentObjectIdV1, DocumentObjectIdV1Error, ProjectionLocalObjectKeyV1,
        ProjectionLocalObjectKeyV1Error,
    };

    #[test]
    fn document_object_id_requires_nonempty_primitives_and_round_trips() {
        assert_eq!(
            DocumentObjectIdV1::from_class_source("", "source"),
            Err(DocumentObjectIdV1Error::InvalidWireKey),
        );
        assert_eq!(
            DocumentObjectIdV1::from_class_source("cdml/molecule", ""),
            Err(DocumentObjectIdV1Error::InvalidWireKey),
        );

        let identity = DocumentObjectIdV1::from_class_source("cdml/molecule", "molecule-1")
            .expect("nonempty primitives produce a canonical durable identity");
        assert_eq!(DocumentObjectIdV1::parse(identity.as_str()), Ok(identity),);
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
