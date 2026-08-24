//! Closed compact-group identity and geometry values.

pub use ferrum_document_model::CompactGroupCatalogKeyV1;
use serde::Serialize;
use thiserror::Error;

use crate::{DocumentObjectIdV1, Point3V1};

/// One compact group's finite orientation and selected attachment site.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct CompactGroupAttachmentV1 {
    attachment_index: u8,
    orientation_degrees: f64,
}

impl CompactGroupAttachmentV1 {
    /// Construct a normalized, definition-validated compact-group attachment.
    pub fn new(
        catalog_key: CompactGroupCatalogKeyV1,
        attachment_index: u8,
        orientation_degrees: f64,
    ) -> Result<Self, CompactGroupV1Error> {
        if !catalog_key.supports_attachment_index(attachment_index) {
            return Err(CompactGroupV1Error::InvalidAttachmentIndex {
                catalog_key: catalog_key.as_str(),
                attachment_index,
            });
        }
        if !orientation_degrees.is_finite() {
            return Err(CompactGroupV1Error::NonFiniteOrientation);
        }
        Ok(Self {
            attachment_index,
            orientation_degrees: orientation_degrees.rem_euclid(360.0),
        })
    }

    /// Return the closed attachment-site index.
    #[must_use]
    pub const fn attachment_index(self) -> u8 {
        self.attachment_index
    }

    /// Return the canonical orientation in `[0.0, 360.0)`.
    #[must_use]
    pub const fn orientation_degrees(self) -> f64 {
        self.orientation_degrees
    }
}

/// One first-class compact group authored in a Ferrum document.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CompactGroupV1 {
    id: DocumentObjectIdV1,
    catalog_key: CompactGroupCatalogKeyV1,
    anchor: Point3V1,
    attachment: CompactGroupAttachmentV1,
}

impl CompactGroupV1 {
    /// Construct a validated compact group from immutable source facts.
    #[must_use]
    pub const fn new(
        id: DocumentObjectIdV1,
        catalog_key: CompactGroupCatalogKeyV1,
        anchor: Point3V1,
        attachment: CompactGroupAttachmentV1,
    ) -> Self {
        Self {
            id,
            catalog_key,
            anchor,
            attachment,
        }
    }

    /// Return the durable document identity.
    #[must_use]
    pub const fn id(&self) -> &DocumentObjectIdV1 {
        &self.id
    }

    /// Return the closed catalog definition key.
    #[must_use]
    pub const fn catalog_key(&self) -> CompactGroupCatalogKeyV1 {
        self.catalog_key
    }

    /// Return the finite document anchor.
    #[must_use]
    pub const fn anchor(&self) -> Point3V1 {
        self.anchor
    }

    /// Return the validated attachment facts.
    #[must_use]
    pub const fn attachment(&self) -> CompactGroupAttachmentV1 {
        self.attachment
    }
}

/// One first-class compact-group view derived from source facts.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CompactGroupProjectionV1 {
    id: DocumentObjectIdV1,
    catalog_key: CompactGroupCatalogKeyV1,
    label: String,
    anchor: Point3V1,
    attachment_index: u8,
    orientation_degrees: f64,
    source_order: u32,
}

impl CompactGroupProjectionV1 {
    /// Derive a source-ordered projection value from a validated compact group.
    #[must_use]
    pub fn from_group(group: &CompactGroupV1, source_order: u32) -> Self {
        Self {
            id: group.id().clone(),
            catalog_key: group.catalog_key(),
            label: group.catalog_key().label().to_owned(),
            anchor: group.anchor(),
            attachment_index: group.attachment().attachment_index(),
            orientation_degrees: group.attachment().orientation_degrees(),
            source_order,
        }
    }

    /// Return the durable compact-group object key.
    #[must_use]
    pub const fn id(&self) -> &DocumentObjectIdV1 {
        &self.id
    }

    /// Return the closed catalog definition key.
    #[must_use]
    pub const fn catalog_key(&self) -> CompactGroupCatalogKeyV1 {
        self.catalog_key
    }

    /// Return the canonical label derived from the closed catalog key.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Return the finite group anchor.
    #[must_use]
    pub const fn anchor(&self) -> Point3V1 {
        self.anchor
    }

    /// Return the selected definition attachment site.
    #[must_use]
    pub const fn attachment_index(&self) -> u8 {
        self.attachment_index
    }

    /// Return the normalized group orientation.
    #[must_use]
    pub const fn orientation_degrees(&self) -> f64 {
        self.orientation_degrees
    }

    /// Return the direct-child source order within the molecule.
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
}

/// Closed refusal taxonomy for compact-group record facts.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum CompactGroupV1Error {
    /// The persisted key is outside Ferrum's V1 compact catalog.
    #[error("unsupported compact-group catalog key: {0}")]
    UnsupportedCatalogKey(String),
    /// The persisted attachment index is not supplied by the selected definition.
    #[error("compact-group attachment index {attachment_index} is invalid for {catalog_key}")]
    InvalidAttachmentIndex {
        catalog_key: &'static str,
        attachment_index: u8,
    },
    /// The persisted orientation is not a finite scalar.
    #[error("compact-group orientation must be finite")]
    NonFiniteOrientation,
}
