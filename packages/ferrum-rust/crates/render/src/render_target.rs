//! Stable render-target identity shared by plan construction and consumers.

use ferrum_core::RecordId;
use ferrum_document_projection::DocumentObjectIdV1;
use serde::{Deserialize, Serialize};

/// One render target's visual identity, durable document identity, and projection order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderTarget {
    record_id: RecordId,
    source_order: u32,
    document_object_id: Option<DocumentObjectIdV1>,
    owner_molecule_object_id: Option<DocumentObjectIdV1>,
}

impl RenderTarget {
    /// Construct a presentation-only target with no document-object identity.
    #[must_use]
    pub fn new(record_id: RecordId, source_order: u32) -> Self {
        Self {
            record_id,
            source_order,
            document_object_id: None,
            owner_molecule_object_id: None,
        }
    }

    /// Construct one structural document target from exact projection identities.
    #[must_use]
    pub fn document_object(
        record_id: RecordId,
        source_order: u32,
        document_object_id: DocumentObjectIdV1,
        owner_molecule_object_id: Option<DocumentObjectIdV1>,
    ) -> Self {
        Self {
            record_id,
            source_order,
            document_object_id: Some(document_object_id),
            owner_molecule_object_id,
        }
    }

    /// Return the stable source record identity.
    #[must_use]
    pub fn record_id(&self) -> &RecordId {
        &self.record_id
    }

    /// Return the deterministic document projection order.
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }

    /// Return the exact durable document object identity when this target is structural.
    #[must_use]
    pub fn document_object_id(&self) -> Option<&DocumentObjectIdV1> {
        self.document_object_id.as_ref()
    }

    /// Return the durable direct-root molecule that owns this structural target.
    #[must_use]
    pub fn owner_molecule_object_id(&self) -> Option<&DocumentObjectIdV1> {
        self.owner_molecule_object_id.as_ref()
    }
}
