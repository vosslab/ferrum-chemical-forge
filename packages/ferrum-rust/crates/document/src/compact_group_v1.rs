//! Closed document-owned values for compact known-group records.
//!
//! Compact groups are authored document objects, not chemistry graph vertices.
//! Their catalog identity stays in this module for typed document state and
//! projection.

pub use ferrum_document_projection::{
    CompactGroupAttachmentV1, CompactGroupCatalogKeyV1, CompactGroupV1, CompactGroupV1Error,
};
