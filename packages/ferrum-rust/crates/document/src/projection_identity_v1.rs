//! Durable and projection-local identities carried by document projections.

use serde::Serialize;
use thiserror::Error;

use crate::TypedRecord;

/// An opaque, versioned selector for one durable typed record.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct DocumentObjectIdV1(String);

impl DocumentObjectIdV1 {
    pub(crate) fn from_record(record: &TypedRecord) -> Option<Self> {
        let source_id = record.attribute("id")?;
        Some(Self::from_class_source(record.class().name(), source_id))
    }

    pub(crate) fn from_class_source(class: &str, source_id: &str) -> Self {
        Self(format!(
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

fn valid_hex(value: &str) -> bool {
    !value.is_empty()
        && value.len().is_multiple_of(2)
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// An explicitly projection-local key that never selects a session operation.
///
/// It distinguishes records which lack an authored durable source identity. Its
/// structural spelling is intentionally local to this immutable projection and
/// may change after structural edits.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ProjectionLocalObjectKeyV1(String);

impl ProjectionLocalObjectKeyV1 {
    pub(crate) fn from_record(record: &TypedRecord) -> Self {
        let path = record
            .path()
            .components()
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(".");
        Self(format!("ferrum-projection-local-v1/{path}"))
    }

    pub(crate) fn parse(value: String) -> Option<Self> {
        let path = value.strip_prefix("ferrum-projection-local-v1/")?;
        (!path.is_empty() && path.split('.').all(valid_local_path_component)).then_some(Self(value))
    }

    /// Return the projection-local wire key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn valid_local_path_component(component: &str) -> bool {
    component
        .parse::<u32>()
        .ok()
        .is_some_and(|number| number.to_string() == component)
}
